//! Native data models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NativeUnitReferenceKind {
    VariableWidth,
    GatedWord32,
    Word32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NativeUnitReference {
    pub kind: NativeUnitReferenceKind,
    pub start_bit: i64,
    pub end_bit: i64,
    pub present: bool,
    /// Raw index or full word; domain-specific bases are not guessed.
    pub value: u32,
    pub tail: u32,
    /// Native category-one flag; remains true even when the presence gate is closed.
    pub probe: bool,
}
