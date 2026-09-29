//! Native data models.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnitEquipmentEntry {
    pub value: u32,
    pub tail: u32,
    /// A closed entry remains in the list, with value and tail zero.
    pub present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnitEquipmentRead {
    pub head: u32,
    pub entries: Vec<UnitEquipmentEntry>,
}
