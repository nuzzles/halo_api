//! Caller-supplied map catalogs, with the pinned Go JSON loading semantics.
use super::{FilmMapBounds, normalize_film_map_name};
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::BTreeMap, fmt};

pub const FILM_MAP_CATALOG_SCHEMA_VERSION: i64 = 1;

#[derive(Debug, thiserror::Error)]
pub enum FilmMapCatalogError {
    #[error("invalid map catalog JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("map catalog schema {0}; expected 1")]
    Schema(i64),
    #[error("map catalog I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("unknown map bounds: catalog absent")]
    MissingCatalog,
    #[error("unknown map bounds: {0}")]
    UnknownMap(String),
    #[error("map width cannot be represented on this target: {0}")]
    Width(u64),
}

impl FilmMapCatalogError {
    /// Native errors.Is(err, ErrUnknownMapBounds), for either an absent catalog
    /// or a missing entry in a present catalog. Loading errors are separate.
    pub fn is_unknown_map_bounds(&self) -> bool {
        matches!(self, Self::MissingCatalog | Self::UnknownMap(_))
    }
}

/// Native optional-receiver lookup with value ownership. A missing catalog is
/// distinct from a present catalog lacking the requested map (including "").
/// The existing borrowed lookup remains available for callers holding a catalog.
pub fn lookup_loaded_film_map(
    catalog: Option<&LoadedFilmMapCatalog>,
    name: &str,
) -> Result<LoadedFilmMapEntry, FilmMapCatalogError> {
    catalog
        .ok_or(FilmMapCatalogError::MissingCatalog)?
        .lookup(name)
        .cloned()
}

/// Native catalog fields. Optional maps preserve the distinction between null
/// and an empty object. Native uint widths remain 64-bit on WASM as well.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedFilmMapCatalog {
    pub schema_version: i64,
    pub source: String,
    pub maps: Option<BTreeMap<String, LoadedFilmMapEntry>>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedFilmMapEntry {
    pub module: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub axis_widths: [u64; 3],
    #[serde(skip_serializing_if = "zero_u32")]
    pub region: u32,
    #[serde(skip_serializing_if = "zero_u64")]
    pub region_index_bits: u64,
}
fn zero_u32(n: &u32) -> bool {
    *n == 0
}
fn zero_u64(n: &u64) -> bool {
    *n == 0
}
impl LoadedFilmMapEntry {
    /// Explicitly project catalog widths into the existing decoder API. Large
    /// native uint values remain available above even on narrower targets.
    pub fn try_bounds(&self) -> Result<FilmMapBounds, FilmMapCatalogError> {
        let width = |n| usize::try_from(n).map_err(|_| FilmMapCatalogError::Width(n));
        Ok(FilmMapBounds {
            module: self.module.clone(),
            min: self.min,
            max: self.max,
            axis_widths: [
                width(self.axis_widths[0])?,
                width(self.axis_widths[1])?,
                width(self.axis_widths[2])?,
            ],
            region: self.region,
            region_index_bits: width(self.region_index_bits)?,
        })
    }
}
impl LoadedFilmMapCatalog {
    pub fn lookup(&self, name: &str) -> Result<&LoadedFilmMapEntry, FilmMapCatalogError> {
        self.maps
            .as_ref()
            .and_then(|m| m.get(&normalize_film_map_name(name)))
            .ok_or_else(|| FilmMapCatalogError::UnknownMap(name.into()))
    }
}
/// Decode JSON first, then validate its schema, matching LoadMapQuantCatalog.
pub fn parse_film_map_catalog(bytes: &[u8]) -> Result<LoadedFilmMapCatalog, FilmMapCatalogError> {
    let text = native_catalog_json(bytes);
    let catalog: LoadedFilmMapCatalog = serde_json::from_slice(&text)?;
    if catalog.schema_version != FILM_MAP_CATALOG_SCHEMA_VERSION {
        return Err(FilmMapCatalogError::Schema(catalog.schema_version));
    }
    Ok(catalog)
}
#[cfg(not(target_arch = "wasm32"))]
pub fn load_film_map_catalog(
    path: impl AsRef<std::path::Path>,
) -> Result<LoadedFilmMapCatalog, FilmMapCatalogError> {
    parse_film_map_catalog(&std::fs::read(path)?)
}
// encoding/json matches ASCII field names using Unicode simple folding too.
pub(super) fn field_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'ſ' => 's',
            'K' => 'k',
            _ => c.to_ascii_lowercase(),
        })
        .collect()
}
// Go encoding/json substitutes malformed UTF-8 and unpaired UTF-16 surrogates.
// Preserve valid surrogate pairs and escaped backslashes for the JSON parser.
pub(super) fn native_catalog_json(bytes: &[u8]) -> Vec<u8> {
    let mut text = String::with_capacity(bytes.len());
    let mut remaining = bytes;
    while !remaining.is_empty() {
        match std::str::from_utf8(remaining) {
            Ok(valid) => {
                text.push_str(valid);
                break;
            }
            Err(error) => {
                let valid = error.valid_up_to();
                text.push_str(std::str::from_utf8(&remaining[..valid]).unwrap());
                text.push('\u{fffd}');
                // Go utf8.DecodeRune consumes one byte per malformed rune,
                // including each byte of an incomplete multibyte sequence.
                remaining = &remaining[valid + 1..];
            }
        }
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    let mut in_string = false;
    let hex = |at: usize| -> Option<u16> {
        let digits = bytes.get(at..at.checked_add(4)?)?;
        let digits = std::str::from_utf8(digits).ok()?;
        u16::from_str_radix(digits, 16).ok()
    };
    while i < bytes.len() {
        if bytes[i] == b'"' {
            in_string = !in_string;
        }
        if in_string && bytes[i] == b'\\' {
            if bytes.get(i + 1) == Some(&b'u')
                && let Some(code) = hex(i + 2)
            {
                let paired = (0xd800..=0xdbff).contains(&code)
                    && bytes.get(i + 6..i + 8) == Some(b"\\u")
                    && hex(i + 8).is_some_and(|low| (0xdc00..=0xdfff).contains(&low));
                if paired {
                    out.extend_from_slice(&bytes[i..i + 12]);
                    i += 12;
                    continue;
                }
                if (0xd800..=0xdfff).contains(&code) {
                    out.extend_from_slice(b"\\ufffd");
                    i += 6;
                    continue;
                }
            }
            let end = (i + 2).min(bytes.len());
            out.extend_from_slice(&bytes[i..end]);
            i = end;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}
trait CatalogScalar {
    fn valid(self) -> bool;
}
impl CatalogScalar for f32 {
    fn valid(self) -> bool {
        self.is_finite()
    }
}
impl CatalogScalar for u64 {
    fn valid(self) -> bool {
        true
    }
}
struct ArrayField<'a, T>(&'a mut [T; 3]);
impl<'de, T: Deserialize<'de> + Default + Copy + CatalogScalar> DeserializeSeed<'de>
    for ArrayField<'_, T>
{
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        d.deserialize_any(self)
    }
}
impl<'de> Deserialize<'de> for LoadedFilmMapEntry {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Fields;
        impl<'de> Visitor<'de> for Fields {
            type Value = LoadedFilmMapEntry;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a map entry or null")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(Self::Value::default())
            }
            fn visit_map<M: MapAccess<'de>>(self, mut m: M) -> Result<Self::Value, M::Error> {
                let mut out = Self::Value::default();
                while let Some(k) = m.next_key::<String>()? {
                    match field_name(&k).as_str() {
                        "module" => {
                            if let Some(v) = m.next_value::<Option<String>>()? {
                                out.module = v;
                            }
                        }
                        "min" => m.next_value_seed(ArrayField(&mut out.min))?,
                        "max" => m.next_value_seed(ArrayField(&mut out.max))?,
                        "axiswidths" => m.next_value_seed(ArrayField(&mut out.axis_widths))?,
                        "region" => {
                            if let Some(v) = m.next_value::<Option<u32>>()? {
                                out.region = v;
                            }
                        }
                        "regionindexbits" => {
                            if let Some(v) = m.next_value::<Option<u64>>()? {
                                out.region_index_bits = v;
                            }
                        }
                        _ => {
                            m.next_value::<IgnoredAny>()?;
                        }
                    }
                }
                if out.min.iter().chain(&out.max).any(|v| !v.is_finite()) {
                    return Err(serde::de::Error::custom("map coordinate overflows float32"));
                }
                Ok(out)
            }
        }
        d.deserialize_any(Fields)
    }
}
struct MapsField<'a>(&'a mut Option<BTreeMap<String, LoadedFilmMapEntry>>);
impl<'de> DeserializeSeed<'de> for MapsField<'_> {
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        d.deserialize_any(self)
    }
}
impl<'de> Deserialize<'de> for LoadedFilmMapCatalog {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Fields;
        impl<'de> Visitor<'de> for Fields {
            type Value = LoadedFilmMapCatalog;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a catalog object or null")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(Self::Value::default())
            }
            fn visit_map<M: MapAccess<'de>>(self, mut m: M) -> Result<Self::Value, M::Error> {
                let mut out = Self::Value::default();
                while let Some(k) = m.next_key::<String>()? {
                    match field_name(&k).as_str() {
                        "schemaversion" => {
                            if let Some(v) = m.next_value::<Option<i64>>()? {
                                out.schema_version = v;
                            }
                        }
                        "source" => {
                            if let Some(v) = m.next_value::<Option<String>>()? {
                                out.source = v;
                            }
                        }
                        "maps" => m.next_value_seed(MapsField(&mut out.maps))?,
                        _ => {
                            m.next_value::<IgnoredAny>()?;
                        }
                    }
                }
                Ok(out)
            }
        }
        d.deserialize_any(Fields)
    }
}

impl<'de, T: Deserialize<'de> + Default + Copy + CatalogScalar> Visitor<'de> for ArrayField<'_, T> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an array or null")
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<(), A::Error> {
        for i in 0..3 {
            match a.next_element::<Option<T>>()? {
                Some(Some(v)) => {
                    if !v.valid() {
                        return Err(serde::de::Error::custom("map coordinate overflows float32"));
                    }
                    self.0[i] = v;
                }
                Some(None) => {}
                None => {
                    self.0[i..].fill(T::default());
                    return Ok(());
                }
            }
        }
        while a.next_element::<IgnoredAny>()?.is_some() {}
        Ok(())
    }
}

impl<'de> Visitor<'de> for MapsField<'_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a maps object or null")
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<(), E> {
        *self.0 = None;
        Ok(())
    }
    fn visit_map<M: MapAccess<'de>>(self, mut m: M) -> Result<(), M::Error> {
        let maps = self.0.get_or_insert_with(BTreeMap::new);
        while let Some((k, v)) = m.next_entry()? {
            maps.insert(k, v);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_map_catalog_loading() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/map-catalog-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 328);
        for (i, row) in rows.iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let out = parse_film_map_catalog(&bytes);
            let category = match &out {
                Ok(_) => "",
                Err(FilmMapCatalogError::Schema(_)) => "schema",
                Err(FilmMapCatalogError::Json(_)) => "json",
                other => panic!("unexpected {other:?}"),
            };
            assert_eq!(category, row["category"].as_str().unwrap(), "category {i}");
            for lookup in row["context_lookups"].as_array().unwrap() {
                let name = lookup["name"].as_str().unwrap();
                let catalog = lookup["present"]
                    .as_bool()
                    .unwrap()
                    .then_some(out.as_ref().ok())
                    .flatten();
                let entry = lookup_loaded_film_map(catalog, name);
                assert_eq!(
                    entry.is_ok(),
                    lookup["found"].as_bool().unwrap(),
                    "context {i}"
                );
                assert_eq!(
                    entry.as_ref().is_err_and(|e| e.is_unknown_map_bounds()),
                    lookup["unknown_bounds"].as_bool().unwrap()
                );
                assert_eq!(
                    matches!(entry, Err(FilmMapCatalogError::MissingCatalog)),
                    lookup["bare_sentinel"].as_bool().unwrap()
                );
                match entry {
                    Ok(mut entry) => {
                        let expected: LoadedFilmMapEntry =
                            serde_json::from_value(lookup["entry"].clone()).unwrap();
                        assert_eq!(entry, expected, "owned entry {i}");
                        entry.module.push_str("changed");
                        entry.min[0] = 123.;
                        assert_eq!(lookup_loaded_film_map(catalog, name).unwrap(), expected);
                    }
                    Err(FilmMapCatalogError::UnknownMap(requested)) => assert_eq!(requested, name),
                    Err(FilmMapCatalogError::MissingCatalog) => {}
                    other => panic!("unexpected lookup {other:?}"),
                }
            }
            if let Err(error) = &out {
                assert!(!error.is_unknown_map_bounds());
            }
            if let Ok(out) = out {
                let expected: LoadedFilmMapCatalog =
                    serde_json::from_value(row["catalog"].clone()).unwrap();
                assert_eq!(out, expected, "catalog {i}");
                for lookup in row["lookups"].as_array().unwrap() {
                    let entry = out.lookup(lookup["name"].as_str().unwrap());
                    assert_eq!(
                        entry.is_ok(),
                        lookup["found"].as_bool().unwrap(),
                        "lookup {i}"
                    );
                    if let Ok(entry) = entry {
                        let expected: LoadedFilmMapEntry =
                            serde_json::from_value(lookup["entry"].clone()).unwrap();
                        assert_eq!(entry, &expected, "entry {i}");
                    }
                }
            }
        }
        let catalog =
            parse_film_map_catalog(include_bytes!("reference/map_quant_bounds.json")).unwrap();
        let pinned = super::super::film_map_catalog();
        assert_eq!(catalog.maps.as_ref().unwrap().len(), pinned.maps.len());
        for (name, entry) in &pinned.maps {
            assert_eq!(&catalog.lookup(name).unwrap().try_bounds().unwrap(), entry);
        }
    }
    #[test]
    fn native_map_catalog_float32_rounding() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/map-rounding-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 1601);
        let mut rejected = 0;
        let mut negative_zero = 0;
        for (i, row) in rows.iter().enumerate() {
            let token = row["token"].as_str().unwrap();
            let input = format!(r#"{{"schemaVersion":1,"maps":{{"a":{{"min":[{token},0,0]}}}}}}"#);
            let result = parse_film_map_catalog(input.as_bytes());
            assert_eq!(
                result.is_ok(),
                row["accepted"].as_bool().unwrap(),
                "case {i}: {token}"
            );
            match result {
                Ok(catalog) => {
                    let bits = catalog.lookup("a").unwrap().min[0].to_bits();
                    assert_eq!(
                        u64::from(bits),
                        row["bits"].as_u64().unwrap(),
                        "case {i}: {token}"
                    );
                    negative_zero += usize::from(bits == 0x80000000);
                }
                Err(FilmMapCatalogError::Json(_)) => rejected += 1,
                other => panic!("unexpected {other:?}"),
            }
        }
        assert!(rejected > 0 && negative_zero > 0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn map_catalog_file_errors() {
        let path =
            std::env::temp_dir().join(format!("halo-map-catalog-{}.json", std::process::id()));
        std::fs::write(&path, br#"{"schemaVersion":1,"maps":{}}"#).unwrap();
        assert_eq!(
            load_film_map_catalog(&path).unwrap(),
            parse_film_map_catalog(br#"{"schemaVersion":1,"maps":{}}"#).unwrap()
        );
        std::fs::remove_file(&path).unwrap();
        assert!(matches!(
            load_film_map_catalog(&path),
            Err(FilmMapCatalogError::Io(_))
        ));
    }
}
