//! Reference data models.
/// A known film-code mapping. Metadata fields are absent for medals missing from the CMS snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilmMedalDefinition {
    pub film_id: u8,
    pub name: &'static str,
    pub name_id: Option<u32>,
    pub sorting_weight: Option<u16>,
}

/// Portable medal identity. Unknown codes remain present with no invented name or NameId.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MedalAward {
    pub film_id: u8,
    pub name: Option<String>,
    pub name_id: Option<u32>,
}
