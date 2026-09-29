//! Reference data models.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WeaponDamage {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub victim_idx: i64,
    pub responsible_idx: i64,
    pub source: u64,
    pub has_source: bool,
    pub negative: bool,
    pub mag_clear: f64,
    pub mag_raw: u64,
}
