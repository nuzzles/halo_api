//! Reference data models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecordKind {
    End,
    New,
    Delete,
    Delta,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecordHeader {
    pub kind: RecordKind,
    pub id: Option<u32>,
    pub start_bit: i64,
    pub end_bit: i64,
}
