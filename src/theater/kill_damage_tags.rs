//! Pinned native damage-effect IDs and source labels, including publication reserves.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillDamageLabel {
    pub tag: u32,
    pub class: String,
    pub status: String,
    pub name: String,
    pub detail: String,
    pub reserve: String,
}
impl KillDamageLabel {
    pub fn publishable(&self) -> bool {
        matches!(self.status.as_str(), "VALIDE" | "SOUS_RESERVE")
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillDamageProvenance {
    pub ids_date: String,
    pub ids_count: usize,
    pub labels_date: String,
    pub label_count: usize,
}
#[derive(Debug, Clone)]
pub struct KillDamageCatalog {
    pub ids: BTreeSet<u32>,
    pub labels: BTreeMap<u32, KillDamageLabel>,
    pub provenance: KillDamageProvenance,
}
impl KillDamageCatalog {
    pub fn is_damage_effect(&self, tag: u32) -> bool {
        self.ids.contains(&tag)
    }
    pub fn lookup(&self, tag: u32) -> Option<&KillDamageLabel> {
        self.labels.get(&tag)
    }
}
/// Native strong-tag test is independent of catalog membership.
pub fn strong_kill_damage_tag(tag: u32) -> bool {
    tag >= 0x10000
}
/// Which external table failed native catalog parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KillDamageTable {
    Ids,
    Labels,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KillDamageParseFailure {
    Columns { actual: usize },
    InvalidHex,
    Overflow,
}

/// The native parser stops at the first error and returns no partial catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillDamageParseError {
    pub table: KillDamageTable,
    /// One-based physical line, including comments and empty lines.
    pub line: usize,
    pub value: String,
    pub failure: KillDamageParseFailure,
}
impl std::fmt::Display for KillDamageParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} line {}: {:?}: {:?}",
            self.table, self.line, self.value, self.failure
        )
    }
}
impl std::error::Error for KillDamageParseError {}

// Go strings.Fields/TrimSpace and Rust whitespace agree on Unicode White_Space.
// A comment's first date token wins; a later nonempty date replaces earlier dates.
fn damage_header_date(line: &str) -> Option<&str> {
    line.split_whitespace()
        .find_map(|v| v.strip_prefix("date="))
}
fn damage_hex(raw: &str) -> Result<u32, KillDamageParseFailure> {
    // strconv.ParseUint(base=16, bitSize=32) accepts neither signs nor prefixes.
    // It reports the first syntax/range failure as it walks the input.
    if raw.is_empty() {
        return Err(KillDamageParseFailure::InvalidHex);
    }
    let mut value = 0_u32;
    for c in raw.bytes() {
        let digit = match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => return Err(KillDamageParseFailure::InvalidHex),
        };
        value = value
            .checked_mul(16)
            .and_then(|v| v.checked_add(u32::from(digit)))
            .ok_or(KillDamageParseFailure::Overflow)?;
    }
    Ok(value)
}

/// Parse native damage-effect ID and six-column label tables. IDs are trimmed,
/// deduplicated and sorted; label fields are retained verbatim, including unknown
/// class/status strings and carriage returns. Duplicate labels use the last row.
/// A label need not belong to the ID set. No partial catalog escapes an error.
/// Error wording is Rust-specific; table, physical line, input and failure kind
/// preserve the native failure information.
pub fn parse_kill_damage_catalog(
    id_text: &str,
    label_text: &str,
) -> Result<KillDamageCatalog, KillDamageParseError> {
    let mut ids = BTreeSet::new();
    let mut labels = BTreeMap::new();
    let mut ids_date = String::new();
    let mut labels_date = String::new();
    for (n, line) in id_text.split('\n').enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            if let Some(date) = damage_header_date(line).filter(|v| !v.is_empty()) {
                ids_date = date.into();
            }
            continue;
        }
        ids.insert(damage_hex(line).map_err(|failure| KillDamageParseError {
            table: KillDamageTable::Ids,
            line: n + 1,
            value: line.into(),
            failure,
        })?);
    }
    for (n, line) in label_text.split('\n').enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with('#') {
            if let Some(date) = damage_header_date(line).filter(|v| !v.is_empty()) {
                labels_date = date.into();
            }
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 6 {
            return Err(KillDamageParseError {
                table: KillDamageTable::Labels,
                line: n + 1,
                value: line.into(),
                failure: KillDamageParseFailure::Columns {
                    actual: fields.len(),
                },
            });
        }
        let tag = damage_hex(fields[0]).map_err(|failure| KillDamageParseError {
            table: KillDamageTable::Labels,
            line: n + 1,
            value: fields[0].into(),
            failure,
        })?;
        labels.insert(
            tag,
            KillDamageLabel {
                tag,
                class: fields[1].into(),
                status: fields[2].into(),
                name: fields[3].into(),
                detail: fields[4].into(),
                reserve: fields[5].into(),
            },
        );
    }
    let provenance = KillDamageProvenance {
        ids_date,
        ids_count: ids.len(),
        labels_date,
        label_count: labels.len(),
    };
    Ok(KillDamageCatalog {
        ids,
        labels,
        provenance,
    })
}

pub fn pinned_kill_damage_catalog() -> &'static KillDamageCatalog {
    static CATALOG: OnceLock<KillDamageCatalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let id_text = include_str!("reference/damage-effect-ids-v75.txt");
        let label_text = include_str!("reference/damage-effect-labels-v75.tsv");
        parse_kill_damage_catalog(id_text, label_text).expect("pinned damage catalog")
    })
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillSourceTruth {
    pub tag: u32,
    pub display: String,
    pub named: bool,
    pub class: String,
    pub status: String,
    pub detail: String,
    pub reserve: String,
    pub category: i32,
}
/// Retain ambiguous/unknown classification but publish a name only for native
/// validated or reserved labels. Categories outside the known range stay raw.
pub fn kill_source_truth(tag: u32, category: i32) -> KillSourceTruth {
    let mut out = KillSourceTruth {
        tag,
        display: "Autres".into(),
        named: false,
        class: "INCONNU".into(),
        status: "INCONNU".into(),
        detail: String::new(),
        reserve: String::new(),
        category,
    };
    if let Some(label) = pinned_kill_damage_catalog().lookup(tag) {
        out.class = label.class.clone();
        out.status = label.status.clone();
        out.detail = label.detail.clone();
        out.reserve = label.reserve.clone();
        if !label.name.is_empty() && label.publishable() {
            out.display = label.name.clone();
            out.named = true;
        }
    }
    out
}
pub fn kill_damage_category_name(category: i32) -> String {
    const NAMES: [&str; 10] = [
        "None",
        "Headshot",
        "HeadshotMultiplier",
        "SilentMelee",
        "CollisionDamage",
        "AttachedDamage",
        "WeakSpot",
        "ChainedProjectile",
        "SweetHeat",
        "VehicleTransferDamage",
    ];
    usize::try_from(category)
        .ok()
        .and_then(|i| NAMES.get(i))
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("?{category}"))
}

#[cfg(test)]
mod catalog_tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn native_damage_catalog_parsing() {
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/damage-catalog-v41.json.zlib")[..],
        )
        .read_to_end(&mut data)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&data).unwrap();
        assert_eq!(rows.len(), 513);
        let mut accepted = 0;
        let mut rejected = [0; 3];
        for (i, row) in rows.iter().enumerate() {
            let result = parse_kill_damage_catalog(
                row["ids_text"].as_str().unwrap(),
                row["labels_text"].as_str().unwrap(),
            );
            if let Some(error) = row.get("error") {
                let actual = result.expect_err(&format!("case {i}"));
                assert_eq!(actual.value, error["value"], "case {i}");
                assert_eq!(
                    serde_json::to_value(actual.table).unwrap(),
                    error["table"],
                    "case {i}"
                );
                assert_eq!(
                    actual.line,
                    error["line"].as_u64().unwrap() as usize,
                    "case {i}"
                );
                let kind = match actual.failure {
                    KillDamageParseFailure::Columns { actual } => {
                        assert_eq!(actual, error["columns"].as_u64().unwrap() as usize);
                        rejected[0] += 1;
                        "columns"
                    }
                    KillDamageParseFailure::InvalidHex => {
                        rejected[1] += 1;
                        "syntax"
                    }
                    KillDamageParseFailure::Overflow => {
                        rejected[2] += 1;
                        "overflow"
                    }
                };
                assert_eq!(
                    kind,
                    error["kind"].as_str().unwrap(),
                    "case {i}: {actual:?}"
                );
                continue;
            }
            let actual = result.unwrap_or_else(|e| panic!("case {i}: {e}"));
            accepted += 1;
            assert_eq!(
                serde_json::to_value(&actual.ids).unwrap(),
                row["ids"],
                "case {i}"
            );
            assert_eq!(actual.provenance.ids_date, row["ids_date"]);
            assert_eq!(actual.provenance.labels_date, row["labels_date"]);
            assert_eq!(
                actual.provenance.ids_count,
                row["ids"].as_array().unwrap().len()
            );
            assert_eq!(
                actual.provenance.label_count,
                row["labels"].as_array().unwrap().len()
            );
            let labels: Vec<_> = actual.labels.values().map(|l| serde_json::json!({
                "Tag":l.tag,"Class":l.class,"Status":l.status,"Name":l.name,"Detail":l.detail,"Reserve":l.reserve,
            })).collect();
            assert_eq!(
                serde_json::to_value(labels).unwrap(),
                row["labels"],
                "case {i}"
            );
            assert_eq!(
                serde_json::to_value(
                    actual
                        .labels
                        .values()
                        .map(KillDamageLabel::publishable)
                        .collect::<Vec<_>>()
                )
                .unwrap(),
                row["publishable"]
            );
            if i == 512 {
                let pinned = pinned_kill_damage_catalog();
                assert_eq!(actual.ids, pinned.ids);
                assert_eq!(actual.labels, pinned.labels);
                assert_eq!(actual.provenance, pinned.provenance);
            }
        }
        assert!(accepted > 0 && rejected.iter().all(|n| *n > 0));
        println!(
            "damage catalog: {accepted} accepted, {rejected:?} column/syntax/overflow rejections"
        );
    }
}
