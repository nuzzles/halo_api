//! Reference data models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    End,
    New,
    Delete,
    Delta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordHeader {
    /// Optional recorded 32-bit word immediately preceding this header.
    pub prefix:
        Option<crate::theater::parser::v41::replication::source::components::field::ComponentField>,
    pub kind: RecordKind,
    pub id: Option<u32>,
    pub start_bit: i64,
    pub end_bit: i64,
}
