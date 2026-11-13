//! Recorded event versions and build fields following a supported registry.
use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;

/// The fixed header prefix of a supported bootstrap layout, with the following
/// identification and game-options sections when their boundaries are established.
/// Later bootstrap sections remain in the enclosing chunk's bytes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BootstrapHeaderRead {
    pub start_bit: usize,
    /// End of the successfully consumed fields; excludes an unavailable field.
    pub end_bit: usize,
    /// Recorded order, including the first unavailable field on truncation.
    pub fields: Vec<BootstrapHeaderField>,
    pub stop: BootstrapHeaderStop,
    /// The immediately following identification fields, read only after this
    /// fixed header completes. Its optional game-options read follows a complete
    /// identification section at the established body start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identification: Option<BootstrapIdentificationRead>,
}

/// Fixed identification fields following the one-bit corruption-check flag.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BootstrapIdentificationRead {
    pub start_bit: usize,
    pub end_bit: usize,
    pub fields: Vec<BootstrapIdentificationField>,
    pub stop: BootstrapIdentificationStop,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_options: Option<super::GameOptionsRead>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BootstrapIdentificationStop {
    Complete,
    Truncated,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BootstrapIdentificationField {
    pub kind: BootstrapIdentificationFieldKind,
    pub bit: usize,
    pub width: usize,
    /// Exact bytes after removing the enclosing bit shift. Unknown blocks
    /// remain uninterpreted; None denotes an unavailable field.
    pub raw: Option<RawBits>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BootstrapIdentificationFieldKind {
    /// Two 256-byte buffers; their further meaning is not established here.
    NameBuffer(u8),
    /// Four source bytes interpreted little-endian as Unix seconds.
    StartTime,
    UnknownWord(u8),
    /// Three 4,096-byte fields, kept without interpreting their internals.
    UnknownBlock(u8),
    /// Original 16-byte GUID representation, including all-zero IDs.
    MapAssetId,
    MapVersionId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BootstrapHeaderStop {
    /// The fixed header through the corruption-check flag is complete.
    Complete,
    Truncated,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BootstrapHeaderField {
    pub kind: BootstrapHeaderFieldKind,
    pub bit: usize,
    pub width: usize,
    /// Exact MSB-first source bits. None means the field is unavailable, not zero.
    pub raw: Option<RawBits>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BootstrapHeaderFieldKind {
    /// Little-endian word at this event's table index.
    EventVersion(u8),
    /// Fixed 32-byte buffers, including NULs and all remaining bytes.
    BuildVersion,
    BuildBranch,
    BuildFlavor,
    /// Little-endian words.
    BuildId,
    Changelist,
    CorruptionChecks,
}

impl BootstrapHeaderRead {
    /// Recorded guard policy. An absent or malformed field is unknown, not false.
    pub fn corruption_checks(&self) -> Option<bool> {
        let field = self
            .fields
            .iter()
            .find(|f| f.kind == BootstrapHeaderFieldKind::CorruptionChecks)?;
        let raw = field.raw.as_ref()?;
        (field.width == 1 && raw.bit_len == 1 && raw.bytes.len() == 1)
            .then(|| raw.bytes[0] & 0x80 != 0)
    }

    /// A recorded little-endian event version. Missing or partial fields have
    /// no value; the numeric version does not imply support for that layout.
    pub fn event_version(&self, code: u8) -> Option<u32> {
        let field = self
            .fields
            .iter()
            .find(|f| f.kind == BootstrapHeaderFieldKind::EventVersion(code))?;
        let raw = field.raw.as_ref()?;
        if field.width != 32 || raw.bit_len != 32 {
            return None;
        }
        Some(u32::from_le_bytes(raw.bytes.as_slice().try_into().ok()?))
    }
}
