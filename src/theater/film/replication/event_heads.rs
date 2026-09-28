//! Native data models.
/// A guarded reference. Domain 3 uses the reference parser's measured seven bits;
/// component handle categories are a separate table and must not be substituted.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventReference {
    pub domain: u8,
    pub start_bit: usize,
    pub end_bit: usize,
    pub value: Option<EventReferenceValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventReferenceValue {
    pub index: u32,
    pub generation: u8,
    /// Domain-one probe: true selects a nine-bit index, false thirteen bits.
    pub narrow: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HeadEventPayload {
    Pickup {
        class: u8,
        catalog_id: Option<u32>,
        /// Read only when the catalog gate is set, matching the reference scanner.
        more_events: Option<bool>,
    },
    VehicleSeat {
        seat: u8,
    },
    Zoom {
        level: u8,
    },
    /// The reference exposes three references without interpreting the later body.
    SpawnedObjectReferences,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HeadEventStop {
    /// The fields exposed by the corresponding reference head scanner were read.
    /// This does not establish the boundary of the complete event or event list.
    ReferenceFieldsRead,
    UnsupportedType,
    Truncated,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DecodedHeadEvent {
    pub config: bool,
    pub kind: u8,
    pub references: Vec<EventReference>,
    pub payload: Option<HeadEventPayload>,
    /// First unread bit, or source_bits + 1 on a bounded reference refusal.
    /// That refusal marker is not a consumed bit and never proves a body boundary.
    pub end_bit: usize,
    pub stop: HeadEventStop,
}
