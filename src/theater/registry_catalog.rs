//! Native registry fingerprint catalog, including measurement provenance.
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryFingerprintEntry {
    #[serde(rename = "Cle")]
    pub key: String,
    #[serde(rename = "Empreinte")]
    pub fingerprint: u64,
    #[serde(rename = "Blocs")]
    pub blocks: usize,
    #[serde(rename = "SlotsNommes")]
    pub named_slots: usize,
    /// Coverage of the catalog measurement, distinct from a film's classification.
    #[serde(rename = "Statut")]
    pub status: String,
    #[serde(rename = "Source")]
    pub source: String,
    #[serde(rename = "Temoins")]
    pub witnesses: Vec<String>,
    #[serde(rename = "Preuve")]
    pub proof: String,
    #[serde(rename = "Date")]
    pub date: String,
}

/// All native records in catalog order. Earlier-major fingerprints remain here
/// because a v41 film may match them and be classified as presumed, not unknown.
/// This metadata does not enable decoding earlier major versions.
pub fn registry_fingerprint_catalog() -> &'static [RegistryFingerprintEntry] {
    static CATALOG: OnceLock<Vec<RegistryFingerprintEntry>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("reference/registry-fingerprints-v75.json"))
            .expect("pinned registry catalog")
    })
}

/// Native expected entry for a v41 film. A missing or unknown build never falls
/// back to another build or an earlier major-version entry.
pub fn expected_v41_registry_fingerprint(build: &str) -> Option<&'static RegistryFingerprintEntry> {
    if build.is_empty() {
        return None;
    }
    registry_fingerprint_catalog()
        .iter()
        .find(|entry| entry.key.strip_prefix("build=") == Some(build))
}
pub fn registry_fingerprint_is_catalogued(fingerprint: u64) -> bool {
    registry_fingerprint_catalog()
        .iter()
        .any(|entry| entry.fingerprint == fingerprint)
}
pub fn classify_v41_registry_fingerprint(build: &str, fingerprint: u64) -> &'static str {
    if expected_v41_registry_fingerprint(build)
        .is_some_and(|entry| entry.fingerprint == fingerprint)
    {
        "connue"
    } else if registry_fingerprint_is_catalogued(fingerprint) {
        "presumee"
    } else {
        "inconnue"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Row {
        build: String,
        fingerprint: u64,
        expected: Option<RegistryFingerprintEntry>,
        catalogued: bool,
        status: String,
    }
    #[derive(Deserialize)]
    struct Oracle {
        table: Vec<RegistryFingerprintEntry>,
        rows: Vec<Row>,
    }
    #[test]
    fn native_registry_catalog_and_v41_classification() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/registry-catalog-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let oracle: Oracle = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(registry_fingerprint_catalog(), oracle.table);
        assert_eq!(oracle.table.len(), 9);
        assert_eq!(oracle.rows.len(), 252);
        let mut statuses = std::collections::BTreeMap::new();
        for (i, row) in oracle.rows.into_iter().enumerate() {
            assert_eq!(
                expected_v41_registry_fingerprint(&row.build),
                row.expected.as_ref(),
                "expected {i}"
            );
            assert_eq!(
                registry_fingerprint_is_catalogued(row.fingerprint),
                row.catalogued,
                "catalogued {i}"
            );
            assert_eq!(
                classify_v41_registry_fingerprint(&row.build, row.fingerprint),
                row.status,
                "status {i}"
            );
            assert_eq!(
                crate::theater::replay_registry_status(&row.build, row.fingerprint),
                row.status,
                "replay {i}"
            );
            *statuses.entry(row.status).or_insert(0) += 1;
        }
        assert_eq!(statuses.len(), 3);
        let presumed = expected_v41_registry_fingerprint("HI_1_12_0").unwrap();
        assert_eq!(presumed.status, "presumee");
        assert_eq!(
            classify_v41_registry_fingerprint("HI_1_12_0", presumed.fingerprint),
            "connue"
        );
        let encoded = serde_json::to_vec(registry_fingerprint_catalog()).unwrap();
        assert_eq!(
            serde_json::from_slice::<Vec<RegistryFingerprintEntry>>(&encoded).unwrap(),
            oracle.table
        );
    }
}
