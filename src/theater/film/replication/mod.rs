//! Typed observations retained by a finished film.
use crate::theater::film::catalog::items::{Equipment, Grenade, Vehicle, Weapon};
use crate::theater::film::{
    ComponentSlot, ControlPlayerIndex, EntityId, EntitySchemaId, EntitySchemaKind, FilmName,
    KeyframeId, TagId,
};

mod event_kind;
pub use event_kind::ReplicationEventKind;
mod event;
pub use event::{ReplicationEvent, ReplicationEventHandle};
mod tag_reference;
pub use tag_reference::{
    DamageAmount, DamageCause, DamageReference, FiringReference, ProjectileReference, TagLink,
    TagReference,
};

/// Source-ordered, timestamped state changes decoded from replication packets.
/// Sequential playback assumes packet timestamps are nondecreasing.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReplicationStream {
    pub observations: Vec<ReplicationObservation>,
}

/// One decoded change at a recorded packet timestamp. Equal timestamps retain
/// their original order in `ReplicationStream::observations`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReplicationObservation {
    pub timestamp_us: u64,
    pub change: ReplicationChange,
}

/// Owned changes available to playback. Incomplete reads remain parser evidence.
/// Entity IDs retain their generation bits; schema kinds are captured from
/// this film's registry. Event kinds name only verified codes, not their payloads.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ReplicationChange {
    /// Introduce a live entity with its film-local schema and complete recorded
    /// default, entity-level, and component field bodies.
    EntityCreated {
        id: EntityId,
        schema: EntitySchemaId,
        /// Meaning established from this film's registry while parsing.
        schema_kind: EntitySchemaKind,
        default_fields: Vec<ReplicationField>,
        fields: Vec<ReplicationField>,
        components: Vec<ReplicationComponentChange>,
    },
    /// Record new entity-level or component bodies for an existing live entity.
    /// Field values may still use the game's delta encoding.
    EntityUpdated {
        id: EntityId,
        fields: Vec<ReplicationField>,
        components: Vec<ReplicationComponentChange>,
    },
    /// Remove the live entity with this exact index and generation.
    EntityDeleted { id: EntityId },
    /// Record player input or command data without implying a world mutation.
    Control { control: ReplicationControl },
    /// Record a baseline in the keyframe ID domain, separate from live entities.
    /// A keyframe is not a simulation tick.
    Keyframe {
        id: KeyframeId,
        schema: EntitySchemaId,
        /// Meaning established from this film's registry while parsing.
        schema_kind: EntitySchemaKind,
        default_fields: Vec<ReplicationField>,
        fields: Vec<ReplicationField>,
        components: Vec<ReplicationComponentChange>,
    },
    /// Record an event at this observation's timestamp. Combat bodies expose
    /// only recorded fields; other bodies retain their exact bits.
    Event { event: ReplicationEvent },
}

/// A complete value from an established named component or record field.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReplicationField {
    pub name: FilmName,
    pub value: ReplicationValue,
}

/// A decoded field value without a source location.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ReplicationValue {
    /// The schema selected a field that carries no bits.
    Empty,
    /// Exact field bits, regardless of whether they fit in a machine integer.
    Bits(BitValue),
    /// Exact 32-bit object tag and its bundled-catalogue interpretation.
    Tag {
        id: TagId,
        kind: TagKind,
        /// Reference metadata captured while parsing; absent when no table matches.
        reference: Option<Box<TagReference>>,
    },
}

/// Exact MSB-first bits from a decoded field or event payload. Unused low bits
/// in the final byte are zero.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BitValue {
    pub bit_len: usize,
    pub bytes: smallvec::SmallVec<[u8; 4]>,
}

impl BitValue {
    /// Interpret at most 64 bits as an unsigned integer, without truncation.
    pub fn as_u64(&self) -> Option<u64> {
        if self.bit_len > 64 || self.bytes.len() != self.bit_len.div_ceil(8) {
            return None;
        }
        Some((0..self.bit_len).fold(0, |value, bit| {
            (value << 1) | u64::from((self.bytes[bit / 8] >> (7 - bit % 8)) & 1)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::BitValue;

    #[test]
    fn bit_values_preserve_width_and_reject_truncating_numeric_reads() {
        let twelve_bits = BitValue {
            bit_len: 12,
            bytes: smallvec::smallvec![0xab, 0xc0],
        };
        assert_eq!(twelve_bits.as_u64(), Some(0xabc));

        let sixty_four_bits = BitValue {
            bit_len: 64,
            bytes: smallvec::SmallVec::from_slice(&u64::MAX.to_be_bytes()),
        };
        assert_eq!(sixty_four_bits.as_u64(), Some(u64::MAX));

        let wider_bits = BitValue {
            bit_len: 128,
            bytes: smallvec::SmallVec::from_elem(0xff, 16),
        };
        assert_eq!(wider_bits.as_u64(), None);
    }
}

/// Classification of a recorded object tag against the bundled catalogue.
/// This does not identify the film's game build or alter the recorded tag ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TagKind {
    Weapon(Weapon),
    Vehicle(Vehicle),
    Equipment(Equipment),
    Grenade(Grenade),
    Projectile,
    Crate,
    Machine,
    Scenery,
    Control,
    /// No named item is known for this tag; the catalogue may still know its category.
    Unknown,
}

/// A fully read component and its decoded named fields.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReplicationComponentChange {
    /// Slot within this entity's schema, not a global component ID.
    pub index: ComponentSlot,
    /// Registry component name shared across observations of this film.
    pub name: FilmName,
    pub fields: Vec<ReplicationField>,
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
