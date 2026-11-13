//! Reference data models.
use crate::theater::parser::v41::chunks::replication::source::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProductionEntityEnd {
    Marker(RecordHeader),
    /// Admission refused after the recorded DELTA header. No body boundary is known.
    Rejected {
        header: RecordHeader,
        #[serde(default)]
        reason: DeltaRejectionReason,
    },
    Failure(EntityViewStop),
    Truncated,
    PayloadBoundary,
    RecordLimit,
}

/// Parser binding-state reason for refusing a DELTA body. This is diagnostic
/// context, not an additional field read from the recording.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeltaRejectionReason {
    /// Serialized before rejection reasons were retained; no reason can be recovered.
    #[default]
    Unknown,
    Unbound,
    OtherView,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProductionFrame {
    pub messages: Option<DecodedFrameView>,
    pub records: Vec<EntityRecord>,
    pub controls: Option<DecodedFrameView>,
    pub entity_end: Option<ProductionEntityEnd>,
    pub end_bit: i64,
    /// Externally supplied record-local schemas used to decode DELTA bodies.
    /// These are parser context, never archetypes recorded in the body.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_delta_schemas: Vec<ExternalDeltaSchemaUse>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExternalDeltaSchemaUse {
    pub start_bit: i64,
    pub id: u32,
    pub archetype: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_serialized_rejection_keeps_its_header_without_inventing_a_reason() {
        let old = serde_json::json!({"Rejected": {"header": {
            "prefix": null,
            "kind": "Delta",
            "id": 17,
            "start_bit": 3,
            "end_bit": 19
        }}});
        let decoded: ProductionEntityEnd = serde_json::from_value(old).unwrap();
        assert!(matches!(
            decoded,
            ProductionEntityEnd::Rejected {
                header: RecordHeader { id: Some(17), .. },
                reason: DeltaRejectionReason::Unknown
            }
        ));
    }
}
