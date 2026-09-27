//! Pinned native fallback catalog and per-decode diagnostic accumulation.
//!
//! Catalog wiring flags describe the reference implementation. They do not assert
//! that every Rust call site has been instrumented.
use super::ReplayByteString;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

/// Native fallback identifier for clamping a negative kept-equipment count.
pub const NEGATIVE_EQUIPMENT_KEPT_FALLBACK: &str = "repli_garde_equipement_negatif_a_zero";
/// Native fallback identifier for assigning an action to the slot's first life.
pub const FIRST_SLOT_LIFE_FALLBACK: &str = "repli_geste_premiere_vie_du_slot";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FallbackSite {
    #[serde(rename = "Fichier")]
    pub file: String,
    #[serde(rename = "Ancre")]
    pub anchor: String,
    pub condition: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FallbackEntry {
    #[serde(rename = "Nom")]
    pub name: String,
    #[serde(rename = "Fait")]
    pub fact: String,
    #[serde(rename = "Mecanisme")]
    pub mechanism: String,
    pub condition: String,
    #[serde(rename = "Ordre")]
    pub order: String,
    pub sites: Vec<FallbackSite>,
    #[serde(rename = "DatePose")]
    pub registered_date: String,
    #[serde(rename = "CibleRetrait")]
    pub removal_target: String,
    #[serde(rename = "CritereRetrait")]
    pub removal_criterion: String,
    #[serde(rename = "CompteurBranche")]
    pub reference_counter_wired: bool,
    #[serde(rename = "CibleComptage")]
    pub counter_target: String,
}
impl FallbackEntry {
    pub fn effective_condition<'a>(&'a self, site: &'a FallbackSite) -> &'a str {
        if site.condition.trim().is_empty() {
            &self.condition
        } else {
            &site.condition
        }
    }
    pub fn source_package(&self) -> &str {
        self.sites.first().map_or("", |s| {
            s.file.rsplit_once('/').map_or(s.file.as_str(), |(p, _)| p)
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FallbackFamily {
    #[serde(rename = "Nom")]
    pub name: String,
    #[serde(rename = "Replis")]
    pub entries: Vec<FallbackEntry>,
}
pub fn fallback_families() -> &'static [FallbackFamily] {
    static FAMILIES: OnceLock<Vec<FallbackFamily>> = OnceLock::new();
    FAMILIES.get_or_init(|| {
        serde_json::from_str(include_str!("reference/fallback-registry-v75.json"))
            .expect("pinned fallback catalog")
    })
}
pub fn fallback_table() -> Vec<&'static FallbackEntry> {
    let mut entries: Vec<_> = fallback_families()
        .iter()
        .flat_map(|f| &f.entries)
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}
pub fn fallback_entry(name: &str) -> Option<&'static FallbackEntry> {
    fallback_families()
        .iter()
        .flat_map(|f| &f.entries)
        .find(|r| r.name == name)
}
pub fn fallbacks_before_read_count() -> usize {
    fallback_families()
        .iter()
        .flat_map(|f| &f.entries)
        .filter(|r| r.order == "devant_la_lecture")
        .count()
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FallbackTrigger {
    #[serde(rename = "Nom")]
    pub name: ReplayByteString,
    #[serde(rename = "Declenchements")]
    pub hits: i64,
}
/// Each decode owns its counter. Unknown names are retained and reported to the
/// caller by `trigger_n` returning false; nonpositive increments are ignored.
#[derive(Debug, Default)]
pub struct FallbackCounter {
    counts: Mutex<BTreeMap<ReplayByteString, i64>>,
    unknown_triggers: Mutex<Vec<FallbackTrigger>>,
}
impl FallbackCounter {
    pub fn trigger(&self, name: &str) -> bool {
        self.trigger_n(name, 1)
    }
    pub fn trigger_n(&self, name: impl AsRef<[u8]>, hits: i64) -> bool {
        let name = name.as_ref();
        if hits <= 0 {
            return true;
        }
        let known = std::str::from_utf8(name)
            .ok()
            .and_then(fallback_entry)
            .is_some();
        if !known {
            tracing::error!(
                repli = ReplayByteString(name.to_vec()).json_text().as_str(),
                declenchements = hits,
                "repli hors registre declenche — le compte ne se rattache a aucune entree"
            );
            self.unknown_triggers
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(FallbackTrigger {
                    name: ReplayByteString(name.to_vec()),
                    hits,
                });
        }
        let mut counts = self.counts.lock().unwrap_or_else(|e| e.into_inner());
        let count = counts.entry(ReplayByteString(name.to_vec())).or_default();
        *count = count.wrapping_add(hits);
        known
    }
    pub fn count(&self, name: impl AsRef<[u8]>) -> i64 {
        self.counts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(name.as_ref())
            .copied()
            .unwrap_or_default()
    }
    pub fn accumulate(&self, report: &[FallbackTrigger]) {
        for entry in report {
            self.trigger_n(&entry.name, entry.hits);
        }
    }
    /// Native unknown-name error diagnostics, in observation order, including
    /// repeated triggers. Counts remain in `report` as well. Nonpositive inputs
    /// produce neither a diagnostic nor a count. Unlike report aggregation,
    /// this preserves each increment supplied to `trigger_n` or `accumulate`.
    pub fn unknown_triggers(&self) -> Vec<FallbackTrigger> {
        self.unknown_triggers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    pub fn report(&self) -> Vec<FallbackTrigger> {
        self.counts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .filter(|(_, n)| **n > 0)
            .map(|(name, &hits)| FallbackTrigger {
                name: name.clone(),
                hits,
            })
            .collect()
    }
}
/// Render the native report without normalizing identifier bytes.
pub fn fallback_report_text(report: &[FallbackTrigger]) -> ReplayByteString {
    if report.is_empty() {
        return "aucun".into();
    }
    let mut out = Vec::new();
    for (i, r) in report.iter().enumerate() {
        if i != 0 {
            out.push(b' ');
        }
        out.extend_from_slice(&r.name.0);
        out.push(b'=');
        out.extend_from_slice(r.hits.to_string().as_bytes());
    }
    ReplayByteString(out)
}

/// Validate registry declarations, including explicit counter-wiring state.
/// Problems are sorted because the native validator iterates an unordered map
/// when checking required fields.
pub fn validate_fallback_entries(entries: &[FallbackEntry]) -> Vec<String> {
    const CONDITIONS: &[&str] = &[
        "film_muet",
        "section_absente",
        "lecture_non_portee",
        "contradiction",
        "non_resolu",
        "inconditionnel",
        "carte_absente_du_catalogue",
        "format_sans_profil_relu",
        "chassis_absent_de_la_table",
    ];
    let mut seen = std::collections::BTreeSet::new();
    let mut errors = Vec::new();
    for r in entries {
        let mut add = |message: String| errors.push(format!("{} : {}", r.name, message));
        if !seen.insert(&r.name) {
            add("nom en double dans le registre".into());
        }
        let n = &r.name;
        if !n.starts_with("repli_")
            || n.split('_').count() < 3
            || n.ends_with('_')
            || n.contains("__")
            || !n
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            add("nom hors convention `repli_<fait>_<mecanisme>` (minuscules, `_`, 3 segments au moins)".into());
        }
        for (name, value) in [
            ("Fait", &r.fact),
            ("Mecanisme", &r.mechanism),
            ("DatePose", &r.registered_date),
            ("CibleRetrait", &r.removal_target),
            ("CritereRetrait", &r.removal_criterion),
        ] {
            if value.trim().is_empty() {
                add(format!("champ {name} vide"));
            }
        }
        let date = r.registered_date.as_bytes();
        if date.len() != 10
            || !date.iter().enumerate().all(|(i, b)| {
                if i == 4 || i == 7 {
                    *b == b'-'
                } else {
                    b.is_ascii_digit()
                }
            })
        {
            add(format!(
                "DatePose {:?} n'est pas au format AAAA-MM-JJ",
                r.registered_date
            ));
        }
        if !CONDITIONS.contains(&r.condition.as_str()) {
            add(format!("condition inconnue {:?}", r.condition));
        }
        if !["apres_lecture", "sans_lecture", "devant_la_lecture"].contains(&r.order.as_str()) {
            add(format!("ordre inconnu {:?}", r.order));
        }
        if r.sites.is_empty() {
            add(
                "aucun site : une entree sans site ne se verifie pas et ne se retire jamais".into(),
            );
        }
        for (i, site) in r.sites.iter().enumerate() {
            if site.file.trim().is_empty() || site.anchor.trim().is_empty() {
                add(format!(
                    "site #{i} incomplet (fichier {:?}, ancre {:?})",
                    site.file, site.anchor
                ));
            }
            if !CONDITIONS.contains(&r.effective_condition(site)) {
                add(format!(
                    "site #{i} : condition inconnue {:?}",
                    site.condition
                ));
            }
            if site.condition == r.condition && !site.condition.trim().is_empty() {
                add(format!(
                    "site #{i} : condition recopiee de l entree — la laisser vide"
                ));
            }
        }
        if !r.reference_counter_wired && r.counter_target.trim().is_empty() {
            add("compteur non branche sans CibleComptage : un zero y serait illisible".into());
        }
        if r.reference_counter_wired && !r.counter_target.trim().is_empty() {
            add("CibleComptage renseignee alors que le compteur est deja branche".into());
        }
    }
    errors.sort();
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_fallback_registry_and_counter() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/fallback-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let expected: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(
            serde_json::to_value(fallback_families()).unwrap(),
            expected["families"]
        );
        assert_eq!(
            serde_json::to_value(fallback_table()).unwrap(),
            expected["table"]
        );
        assert_eq!(
            fallbacks_before_read_count(),
            expected["before_read"].as_u64().unwrap() as usize
        );
        for (i, entry) in fallback_table().iter().enumerate() {
            assert_eq!(
                entry.source_package(),
                expected["packages"][i].as_str().unwrap()
            );
            let conditions: Vec<_> = entry
                .sites
                .iter()
                .map(|s| entry.effective_condition(s))
                .collect();
            assert_eq!(
                serde_json::to_value(conditions).unwrap(),
                expected["conditions"][i]
            );
            assert_eq!(fallback_entry(&entry.name), Some(*entry));
        }
        assert!(
            validate_fallback_entries(&fallback_table().into_iter().cloned().collect::<Vec<_>>())
                .is_empty()
        );
        for row in expected["invalid"].as_array().unwrap() {
            let mut input = row["entry"].clone();
            if input["Sites"].is_null() {
                input["Sites"] = serde_json::json!([]);
            }
            let entry: FallbackEntry = serde_json::from_value(input).unwrap();
            let single = validate_fallback_entries(std::slice::from_ref(&entry));
            let mut actual = single.clone();
            if row["duplicate"].as_bool().unwrap() {
                actual = validate_fallback_entries(&[entry.clone(), entry]);
                for error in single {
                    let index = actual.iter().position(|e| *e == error).unwrap();
                    actual.remove(index);
                }
            }
            assert_eq!(serde_json::to_value(actual).unwrap(), row["problems"]);
        }
        for row in expected["cases"].as_array().unwrap() {
            let counter = FallbackCounter::default();
            let ops: Vec<FallbackTrigger> = serde_json::from_value(row["ops"].clone()).unwrap();
            counter.accumulate(&ops);
            let report = counter.report();
            assert_eq!(
                serde_json::to_value(counter.unknown_triggers()).unwrap(),
                row["unknown_diagnostics"]
            );
            assert_eq!(serde_json::to_value(&report).unwrap(), row["report"]);
            assert_eq!(
                fallback_report_text(&report).0,
                row["text"].as_str().unwrap().as_bytes()
            );
            for hit in report {
                assert_eq!(counter.count(&hit.name), hit.hits);
            }
        }
    }
}
