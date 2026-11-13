//! Typed observations retained by a finished film.
use crate::theater::film::{ArchetypeId, ComponentSlot, ControlPlayerIndex, EntityId, KeyframeId};

mod event_kind;
pub use event_kind::ReplicationEventKind;

/// Source-ordered, timestamped state changes decoded from replication packets.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReplicationStream {
    pub observations: Vec<ReplicationObservation>,
}

/// One decoded change at a recorded packet timestamp. Equal timestamps retain
/// their original order in `ReplicationStream::observations`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReplicationObservation {
    pub timestamp_us: u64,
    pub change: ReplicationChange,
}

/// Owned changes available to playback. Incomplete reads remain parser evidence.
/// Entity IDs retain their generation bits; archetypes resolve against this
/// film's registry. Event kinds name only verified codes, not their payloads.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReplicationChange {
    EntityCreated {
        id: EntityId,
        archetype: ArchetypeId,
        default_fields: Vec<ReplicationField>,
        fields: Vec<ReplicationField>,
        components: Vec<ReplicationComponentChange>,
    },
    EntityUpdated {
        id: EntityId,
        fields: Vec<ReplicationField>,
        components: Vec<ReplicationComponentChange>,
    },
    EntityDeleted {
        id: EntityId,
    },
    Control {
        control: ReplicationControl,
    },
    Keyframe {
        id: KeyframeId,
        archetype: ArchetypeId,
        default_fields: Vec<ReplicationField>,
        fields: Vec<ReplicationField>,
        components: Vec<ReplicationComponentChange>,
    },
    Event {
        code: ReplicationEventKind,
        /// Reference encoding words; their target domains are not all resolved.
        references: Vec<ReplicationEventWord>,
        body: ReplicationEventBody,
    },
}

/// A decoded event scalar whose width matters for interpretation. This is a
/// value and schema width, without a parser source offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReplicationEventWord {
    pub value: u64,
    pub bit_len: usize,
}

/// Semantically known event data, or complete scalar words from another body.
/// `Fields` does not imply that opaque or non-scalar source fields were decoded.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReplicationEventBody {
    Kill(ReplicationKill),
    Fields(Vec<ReplicationEventWord>),
}

/// Verified v41 kill body. Actor slots are event-local references; they are
/// not XUIDs or established roster entries. The optional playback suffix is
/// retained separately because its gameplay meaning is still unknown.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReplicationKill {
    pub victim: Option<u8>,
    pub killer: Option<u8>,
    pub assist: Option<u8>,
    pub killer_percent: u32,
    pub assist_percent: u32,
    pub flag: bool,
    pub playback_suffix: Vec<ReplicationEventWord>,
}

/// A complete value from an established named component or record field.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReplicationField {
    pub name: String,
    pub value: ReplicationValue,
}

/// A decoded field value without a source location. `Wide` stores MSB-first
/// value bytes; unused low bits in the last byte are zero.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReplicationValue {
    /// The schema selected a field that carries no bits.
    Empty,
    /// Scalar with its recorded schema width.
    Scalar {
        value: u64,
        bit_len: usize,
    },
    /// Exact 32-bit asset tag from a field with established tag semantics.
    Tag(crate::theater::film::TagId),
    Wide {
        bit_len: usize,
        bytes: Vec<u8>,
    },
}

/// A fully read component and its decoded named fields.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReplicationComponentChange {
    /// Slot within this entity's archetype, not a global component ID.
    pub index: ComponentSlot,
    pub name: String,
    pub fields: Vec<ReplicationField>,
    pub weapon_id: Option<crate::theater::film::catalog::weapons::Weapon>,
}

impl ReplicationComponentChange {
    /// Find the component definition in this film's recorded archetype schema.
    /// A missing archetype or slot stays unresolved.
    pub fn definition<'a>(
        &self,
        registry: &'a crate::theater::film::Registry,
        archetype: ArchetypeId,
    ) -> Option<&'a crate::theater::film::RegistryComponentDefinition> {
        archetype.resolve(registry)?.component(self.index)
    }
}

/// Decoded control values; skipped command and secondary payloads are not
/// claimed as controls until their meanings are established.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReplicationControl {
    Input {
        index: ControlPlayerIndex,
        short: Option<u8>,
        analog: Option<[u8; 2]>,
        third_analog: Option<u8>,
        extra: Option<u8>,
        flags: Option<u8>,
    },
    Commands {
        index: ControlPlayerIndex,
        command_count: Option<usize>,
    },
    Kind2 {
        value: Option<u16>,
    },
}
