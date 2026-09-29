//! Native data models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AnticipatedDeclaration {
    pub chunk_index: i64,
    pub archetype: u32,
}
