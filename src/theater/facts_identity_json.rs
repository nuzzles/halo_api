//! Identity and fallback JSON sections of the native facts file.
//! Raw source strings use Go JSON normalization at the encoding boundary.
use super::facts_json_read::{Cell, FactsJsonDecode, FactsJsonState, JsonPointer, Shape};
use super::facts_json_write::{FactsJsonValue, FactsJsonWriter};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsFileIdentity {
    #[serde(rename = "Version")]
    pub version: Vec<u8>,
    #[serde(rename = "Build")]
    pub build: Vec<u8>,
    #[serde(rename = "Flavor")]
    pub flavor: Vec<u8>,
    #[serde(rename = "BuildID")]
    pub build_id: u32,
    #[serde(rename = "Changelist")]
    pub changelist: u32,
    #[serde(rename = "MatchStartUnix")]
    pub match_start_unix: u32,
    #[serde(rename = "FormatVersion")]
    pub format_version: i64,
    #[serde(rename = "TypeVersions")]
    pub type_versions: Option<Vec<u32>>,
    #[serde(rename = "RegistryBlocks")]
    pub registry_blocks: i64,
    #[serde(rename = "RegistryFingerprint")]
    pub registry_fingerprint: u64,
    #[serde(rename = "RegistryNamedSlots")]
    pub registry_named_slots: i64,
    #[serde(rename = "BuildOffset")]
    pub build_offset: i64,
    #[serde(rename = "BodyBit")]
    pub body_bit: i64,
    #[serde(rename = "ControleDeCorruption")]
    pub corruption_check: bool,
}
impl FactsJsonDecode for FactsFileIdentity {
    const SHAPE: Shape = Shape::Struct {
        name: "profile.FilmIdentity",
        fields: &[
            ("Version", &Shape::String("string")),
            ("Build", &Shape::String("string")),
            ("Flavor", &Shape::String("string")),
            ("BuildID", &<u32 as FactsJsonDecode>::SHAPE),
            ("Changelist", &<u32 as FactsJsonDecode>::SHAPE),
            ("MatchStartUnix", &<u32 as FactsJsonDecode>::SHAPE),
            ("FormatVersion", &<i64 as FactsJsonDecode>::SHAPE),
            (
                "TypeVersions",
                &<Option<Vec<u32>> as FactsJsonDecode>::SHAPE,
            ),
            ("RegistryBlocks", &<i64 as FactsJsonDecode>::SHAPE),
            ("RegistryFingerprint", &<u64 as FactsJsonDecode>::SHAPE),
            ("RegistryNamedSlots", &<i64 as FactsJsonDecode>::SHAPE),
            ("BuildOffset", &<i64 as FactsJsonDecode>::SHAPE),
            ("BodyBit", &<i64 as FactsJsonDecode>::SHAPE),
            ("ControleDeCorruption", &<bool as FactsJsonDecode>::SHAPE),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            version: {
                let Cell::String(v) = cell.field("Version") else {
                    unreachable!()
                };
                v.as_bytes().to_vec()
            },
            build: {
                let Cell::String(v) = cell.field("Build") else {
                    unreachable!()
                };
                v.as_bytes().to_vec()
            },
            flavor: {
                let Cell::String(v) = cell.field("Flavor") else {
                    unreachable!()
                };
                v.as_bytes().to_vec()
            },
            build_id: FactsJsonDecode::from_cell(cell.field("BuildID")),
            changelist: FactsJsonDecode::from_cell(cell.field("Changelist")),
            match_start_unix: FactsJsonDecode::from_cell(cell.field("MatchStartUnix")),
            format_version: FactsJsonDecode::from_cell(cell.field("FormatVersion")),
            type_versions: FactsJsonDecode::from_cell(cell.field("TypeVersions")),
            registry_blocks: FactsJsonDecode::from_cell(cell.field("RegistryBlocks")),
            registry_fingerprint: FactsJsonDecode::from_cell(cell.field("RegistryFingerprint")),
            registry_named_slots: FactsJsonDecode::from_cell(cell.field("RegistryNamedSlots")),
            build_offset: FactsJsonDecode::from_cell(cell.field("BuildOffset")),
            body_bit: FactsJsonDecode::from_cell(cell.field("BodyBit")),
            corruption_check: FactsJsonDecode::from_cell(cell.field("ControleDeCorruption")),
        }
    }
}
impl FactsJsonValue for FactsFileIdentity {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Version\":");
        w.string(&self.version);
        w.raw(b",\"Build\":");
        w.string(&self.build);
        w.raw(b",\"Flavor\":");
        w.string(&self.flavor);
        w.raw(b",\"BuildID\":");
        self.build_id.write_json(w);
        w.raw(b",\"Changelist\":");
        self.changelist.write_json(w);
        w.raw(b",\"MatchStartUnix\":");
        self.match_start_unix.write_json(w);
        w.raw(b",\"FormatVersion\":");
        self.format_version.write_json(w);
        w.raw(b",\"TypeVersions\":");
        self.type_versions.write_json(w);
        w.raw(b",\"RegistryBlocks\":");
        self.registry_blocks.write_json(w);
        w.raw(b",\"RegistryFingerprint\":");
        self.registry_fingerprint.write_json(w);
        w.raw(b",\"RegistryNamedSlots\":");
        self.registry_named_slots.write_json(w);
        w.raw(b",\"BuildOffset\":");
        self.build_offset.write_json(w);
        w.raw(b",\"BodyBit\":");
        self.body_bit.write_json(w);
        w.raw(b",\"ControleDeCorruption\":");
        self.corruption_check.write_json(w);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsFallback {
    #[serde(rename = "Nom")]
    pub name: Vec<u8>,
    #[serde(rename = "Declenchements")]
    pub count: i64,
}
impl FactsJsonDecode for FactsFallback {
    const SHAPE: Shape = Shape::Struct {
        name: "fallback.Declenchement",
        fields: &[
            ("Nom", &Shape::String("fallback.Nom")),
            ("Declenchements", &<i64 as FactsJsonDecode>::SHAPE),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            name: {
                let Cell::String(v) = cell.field("Nom") else {
                    unreachable!()
                };
                v.as_bytes().to_vec()
            },
            count: FactsJsonDecode::from_cell(cell.field("Declenchements")),
        }
    }
}
impl FactsJsonValue for FactsFallback {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        w.raw(b"\"Nom\":");
        w.string(&self.name);
        w.raw(b",\"Declenchements\":");
        self.count.write_json(w);
        w.raw(b"}");
    }
}

pub fn encode_facts_identity_json(identity: Option<&FactsFileIdentity>) -> Vec<u8> {
    let mut w = FactsJsonWriter::default();
    if let Some(v) = identity {
        v.write_json(&mut w);
    } else {
        w.raw(b"null");
    }
    w.bytes
}
pub fn decode_facts_identity_json(bytes: &[u8]) -> Result<Option<FactsFileIdentity>, String> {
    let mut state = FactsJsonState::<JsonPointer<FactsFileIdentity>>::default();
    state.apply(bytes)?;
    Ok(state.value()?.0)
}
pub fn encode_facts_fallbacks_json(fallbacks: Option<&[FactsFallback]>) -> Vec<u8> {
    let mut w = FactsJsonWriter::default();
    if let Some(v) = fallbacks {
        v.write_json(&mut w);
    } else {
        w.raw(b"null");
    }
    w.bytes
}
pub fn decode_facts_fallbacks_json(bytes: &[u8]) -> Result<Option<Vec<FactsFallback>>, String> {
    let mut state = FactsJsonState::<Option<Vec<FactsFallback>>>::default();
    state.apply(bytes)?;
    state.value()
}

/// Stateful identity-section decoding. Successful reads retain native pointer
/// and slice backing state. An error terminates the file; discard this reader.
#[derive(Default)]
pub struct FactsIdentityJsonReader {
    state: FactsJsonState<JsonPointer<FactsFileIdentity>>,
}
impl FactsIdentityJsonReader {
    pub fn read(&mut self, bytes: &[u8]) -> Result<Option<FactsFileIdentity>, String> {
        self.state.apply(bytes)?;
        Ok(self.state.value()?.0)
    }
}
/// Stateful fallback-section decoding, with the same terminal-error contract.
#[derive(Default)]
pub struct FactsFallbacksJsonReader {
    state: FactsJsonState<Option<Vec<FactsFallback>>>,
}
impl FactsFallbacksJsonReader {
    pub fn read(&mut self, bytes: &[u8]) -> Result<Option<Vec<FactsFallback>>, String> {
        self.state.apply(bytes)?;
        self.state.value()
    }
}
