//! Header and catalog admission for the derived FilmFacts cache.
//! This format is not a lossless encoding of the native recording.
use super::{
    FilmQuantizationRange, I0Layout, LoadedFilmMapEntry, NativeFactsReader, NativeFactsWriter,
    resolve_imposed_i0_layout,
};
use serde::{Deserialize, Serialize};

pub const FILM_FACTS_MAGIC: &[u8] = b"REPLAYINPUTS25\n";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsHeader {
    pub film: Vec<u8>,
    pub map_module: Vec<u8>,
    pub axis_widths: [u64; 3],
    pub layout_detected: bool,
    pub inventory_delta_ammo_refused: bool,
    pub film_major_version: Option<i64>,
    pub film_clock_origin_us: u64,
}

/// Raw catalog identity and native widths, including values not representable
/// by the older usize-based map API on WASM. Bounds retain their float bits.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsMapEntry {
    pub module: Vec<u8>,
    pub axis_widths: [u64; 3],
    pub region: u32,
    pub region_index_bits: u64,
    pub bounds: FilmQuantizationRange,
}
impl From<&LoadedFilmMapEntry> for FactsMapEntry {
    fn from(e: &LoadedFilmMapEntry) -> Self {
        Self {
            module: e.module.as_bytes().to_vec(),
            axis_widths: e.axis_widths,
            region: e.region,
            region_index_bits: e.region_index_bits,
            bounds: std::array::from_fn(|a| [e.min[a], e.max[a]]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FactsHeaderError {
    #[error("faits de film : magie absente ou version inconnue — redecoder le film")]
    Magic,
    #[error("faits de film : carte du catalogue differente : {0}")]
    Map(String),
    #[error("faits de film : decoupage d i0 en contradiction avec le catalogue : {0}")]
    Layout(String),
}
fn widths(w: [u64; 3]) -> String {
    format!("[{} {} {}]", w[0], w[1], w[2])
}

/// Shared native cooking-key check, also used by file-header usability.
pub fn verify_facts_cooking_key(
    module: &[u8],
    axis_widths: [u64; 3],
    detected: bool,
    entry: &FactsMapEntry,
) -> Result<(), FactsHeaderError> {
    if module != entry.module {
        return Err(FactsHeaderError::Map(format!(
            "faits cuits pour {}, entree de catalogue fournie {}",
            super::facts_quote::quote(module),
            super::facts_quote::quote(&entry.module)
        )));
    }
    let layout = I0Layout {
        gate_bits: super::I0_SPINE_BITS
            .wrapping_add(super::I0_USE_DEFAULT_BITS)
            .wrapping_add(entry.region_index_bits.max(1) as i64),
        axis_widths: entry.axis_widths,
        region: entry.region,
    };
    let imposed = resolve_imposed_i0_layout(None, Some(&layout));
    if !detected
        && imposed
            .as_ref()
            .is_none_or(|l| l.axis_widths != axis_widths)
    {
        return Err(FactsHeaderError::Layout(format!(
            "faits au decoupage {} (dit du CATALOGUE), catalogue {}",
            widths(axis_widths),
            imposed.map_or_else(
                || "aucun (entree de carte invalide)".into(),
                |l| widths(l.axis_widths)
            )
        )));
    }
    if detected && let Some(l) = imposed {
        return Err(FactsHeaderError::Layout(format!(
            "faits disant leur decoupage {} AUTO-DETECTE, or le catalogue en impose un ({})",
            widths(axis_widths),
            widths(l.axis_widths)
        )));
    }
    Ok(())
}

/// Writes the fields only. The top-level blob writer prepends FILM_FACTS_MAGIC.
pub fn encode_facts_header(w: &mut NativeFactsWriter, h: &FactsHeader) {
    w.string_bytes(&h.film);
    w.string_bytes(&h.map_module);
    for width in h.axis_widths {
        w.unsigned(width);
    }
    w.boolean(h.layout_detected);
    w.boolean(h.inventory_delta_ammo_refused);
    w.boolean(h.film_major_version.is_some());
    if let Some(v) = h.film_major_version {
        w.signed(v);
    }
    w.unsigned(h.film_clock_origin_us);
}

#[derive(Debug)]
pub struct DecodedFactsHeader<'a> {
    pub header: FactsHeader,
    /// Offsets are relative to the full blob, including the magic prefix.
    /// A successful admission can still carry a sticky transport error here.
    pub reader: NativeFactsReader<'a>,
    pub layout: I0Layout,
    pub bounds: FilmQuantizationRange,
}
pub fn decode_facts_header<'a>(
    blob: &'a [u8],
    entry: &FactsMapEntry,
) -> Result<DecodedFactsHeader<'a>, FactsHeaderError> {
    if !blob.starts_with(FILM_FACTS_MAGIC) {
        return Err(FactsHeaderError::Magic);
    }
    let mut r = NativeFactsReader::new(blob);
    r.section(FILM_FACTS_MAGIC.len() as i64);
    let mut h = FactsHeader {
        film: r.string_bytes().to_vec(),
        map_module: r.string_bytes().to_vec(),
        axis_widths: std::array::from_fn(|_| r.unsigned()),
        layout_detected: r.boolean(),
        inventory_delta_ammo_refused: r.boolean(),
        film_major_version: if r.boolean() { Some(r.signed()) } else { None },
        film_clock_origin_us: 0,
    };
    // Native admission deliberately precedes both clock reading and error reporting.
    verify_facts_cooking_key(&h.map_module, h.axis_widths, h.layout_detected, entry)?;
    h.film_clock_origin_us = r.unsigned();
    let layout = I0Layout {
        axis_widths: h.axis_widths,
        gate_bits: 0,
        region: 0,
    };
    Ok(DecodedFactsHeader {
        header: h,
        reader: r,
        layout,
        bounds: entry.bounds,
    })
}
