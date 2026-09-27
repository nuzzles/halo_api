//! Native file-container header and its separate freshness decision.
//! Header length marks the section start; native header reads use the full blob.
use super::*;
use serde::{Deserialize, Serialize};
use std::fmt;
pub const FILM_FACTS_FILE_MAGIC: &[u8] = b"LEVELUPFILMFACTS\n";
pub const FILM_FACTS_FILE_CODEC: i64 = 1;
pub const FILM_FACTS_FILE_SCHEMA: i64 = 3;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsRegistryCoverage {
    #[serde(rename = "Fingerprint")]
    pub fingerprint: Vec<u8>,
    #[serde(rename = "Status")]
    pub status: Vec<u8>,
    #[serde(rename = "Blocks")]
    pub blocks: i64,
    #[serde(rename = "NamedSlots")]
    pub named_slots: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsDecoderCoverage {
    #[serde(rename = "SourceRev")]
    pub source_rev: Vec<u8>,
    #[serde(rename = "ProfileRev")]
    pub profile_rev: Vec<u8>,
    #[serde(rename = "GrammarRev")]
    pub grammar_rev: Vec<u8>,
    #[serde(rename = "FactsRev")]
    pub facts_rev: Vec<u8>,
    #[serde(rename = "Build")]
    pub build: Vec<u8>,
    #[serde(rename = "Registry")]
    pub registry: Option<FactsRegistryCoverage>,
}
impl FactsDecoderCoverage {
    pub fn current() -> Self {
        let c = build_replay_decoder_coverage(None);
        Self {
            source_rev: c.source_rev.into_bytes(),
            profile_rev: c.profile_rev.into_bytes(),
            grammar_rev: c.grammar_rev.into_bytes(),
            facts_rev: c.facts_rev.into_bytes(),
            ..Default::default()
        }
    }
    pub fn same_revisions(&self, other: &Self) -> bool {
        self.source_rev == other.source_rev
            && self.profile_rev == other.profile_rev
            && self.grammar_rev == other.grammar_rev
            && self.facts_rev == other.facts_rev
    }
}
pub fn encode_facts_decoder_coverage(w: &mut NativeFactsWriter, c: &FactsDecoderCoverage) {
    for s in [
        &c.source_rev,
        &c.profile_rev,
        &c.grammar_rev,
        &c.facts_rev,
        &c.build,
    ] {
        w.string_bytes(s);
    }
    w.boolean(c.registry.is_some());
    if let Some(r) = &c.registry {
        w.string_bytes(&r.fingerprint);
        w.string_bytes(&r.status);
        w.unsigned(r.blocks as u64);
        w.unsigned(r.named_slots as u64);
    }
}
pub fn decode_facts_decoder_coverage(r: &mut NativeFactsReader<'_>) -> FactsDecoderCoverage {
    let mut c = FactsDecoderCoverage {
        source_rev: r.string_bytes().to_vec(),
        profile_rev: r.string_bytes().to_vec(),
        grammar_rev: r.string_bytes().to_vec(),
        facts_rev: r.string_bytes().to_vec(),
        build: r.string_bytes().to_vec(),
        registry: None,
    };
    if r.boolean() {
        c.registry = Some(FactsRegistryCoverage {
            fingerprint: r.string_bytes().to_vec(),
            status: r.string_bytes().to_vec(),
            blocks: r.unsigned() as i64,
            named_slots: r.unsigned() as i64,
        });
    }
    c
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsFileHeader {
    #[serde(rename = "VersionCodec")]
    pub codec: i64,
    #[serde(rename = "Schema")]
    pub schema: i64,
    #[serde(rename = "Coverage")]
    pub coverage: FactsDecoderCoverage,
    #[serde(rename = "MapModule")]
    pub map_module: Vec<u8>,
    #[serde(rename = "AxisW")]
    pub axis_widths: [u64; 3],
    #[serde(rename = "LayoutDetected")]
    pub layout_detected: bool,
    /// Native signed `corps`, retained even when a malformed length wraps.
    #[serde(rename = "BodyOffset")]
    pub body_offset: i64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FactsFileHeaderError {
    Version(Option<String>),
    /// Native %s inserts arbitrary raw revision bytes. Display is lossy for UI;
    /// message_bytes() preserves the native diagnostic without UTF-8 replacement.
    Revisions(Vec<u8>),
    Cooking(FactsHeaderError),
}
impl FactsFileHeaderError {
    pub fn message_bytes(&self) -> Vec<u8> {
        match self {
            Self::Version(detail) => {
                let mut s = "faits de film : version de codec ou de schema inconnue".to_owned();
                if let Some(d) = detail {
                    s.push_str(" : ");
                    s.push_str(d);
                }
                s.into_bytes()
            }
            Self::Revisions(bytes) => bytes.clone(),
            Self::Cooking(e) => e.to_string().into_bytes(),
        }
    }
}
impl fmt::Display for FactsFileHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf8_lossy(&self.message_bytes()))
    }
}
impl std::error::Error for FactsFileHeaderError {}
impl FactsFileHeader {
    /// Native Utilisable order: versions, four revisions, then cooking key.
    /// Build and registry metadata deliberately do not participate.
    pub fn usable(&self, entry: &FactsMapEntry) -> Result<(), FactsFileHeaderError> {
        if self.codec != FILM_FACTS_FILE_CODEC || self.schema != FILM_FACTS_FILE_SCHEMA {
            return Err(FactsFileHeaderError::Version(None));
        }
        let current = FactsDecoderCoverage::current();
        if !self.coverage.same_revisions(&current) {
            let mut message = b"faits de film : revisions de couche differentes : faits {".to_vec();
            for (i, bytes) in [
                &self.coverage.source_rev,
                &self.coverage.profile_rev,
                &self.coverage.grammar_rev,
                &self.coverage.facts_rev,
            ]
            .into_iter()
            .enumerate()
            {
                if i > 0 {
                    message.push(b' ');
                }
                message.extend_from_slice(bytes);
            }
            message.extend_from_slice(b"} contre binaire {");
            for (i, bytes) in [
                &current.source_rev,
                &current.profile_rev,
                &current.grammar_rev,
                &current.facts_rev,
            ]
            .into_iter()
            .enumerate()
            {
                if i > 0 {
                    message.push(b' ');
                }
                message.extend_from_slice(bytes);
            }
            message.push(b'}');
            return Err(FactsFileHeaderError::Revisions(message));
        }
        verify_facts_cooking_key(
            &self.map_module,
            self.axis_widths,
            self.layout_detected,
            entry,
        )
        .map_err(FactsFileHeaderError::Cooking)
    }
}
/// The native API returns the partially decoded header alongside its error.
#[derive(Debug, Clone)]
pub struct FactsFileHeaderRead {
    pub header: FactsFileHeader,
    pub error: Option<FactsFileHeaderError>,
}
pub fn decode_facts_file_header(blob: &[u8]) -> FactsFileHeaderRead {
    let mut h = FactsFileHeader::default();
    let fail = |header, detail: String| FactsFileHeaderRead {
        header,
        error: Some(FactsFileHeaderError::Version(Some(detail))),
    };
    if !blob.starts_with(FILM_FACTS_FILE_MAGIC) {
        return fail(h, "magie absente".into());
    }
    let mut r = NativeFactsReader::new(blob);
    r.section(FILM_FACTS_FILE_MAGIC.len() as i64);
    h.codec = r.unsigned() as i64;
    h.schema = r.unsigned() as i64;
    let length = r.unsigned() as i64;
    if let Some(e) = r.error() {
        return fail(h, format!("prefixe illisible ({e})"));
    }
    if h.codec != FILM_FACTS_FILE_CODEC || h.schema != FILM_FACTS_FILE_SCHEMA {
        let detail = format!(
            "codec {} schema {}, ce binaire lit codec {} schema {}",
            h.codec, h.schema, FILM_FACTS_FILE_CODEC, FILM_FACTS_FILE_SCHEMA
        );
        return fail(h, detail);
    }
    let body = (r.offset() as i64).wrapping_add(length);
    if body > blob.len() as i64 {
        return fail(
            h,
            format!(
                "en-tete annonce {length} octets, {} disponibles",
                r.remaining()
            ),
        );
    }
    h.body_offset = body;
    h.coverage = decode_facts_decoder_coverage(&mut r);
    h.map_module = r.string_bytes().to_vec();
    h.axis_widths = std::array::from_fn(|_| r.unsigned());
    h.layout_detected = r.boolean();
    if let Some(e) = r.error() {
        return fail(h, format!("en-tete illisible ({e})"));
    }
    FactsFileHeaderRead {
        header: h,
        error: None,
    }
}
/// File writers always emit current codec/schema and an exact header length.
/// The supplied header's codec/schema/body_offset describe reads, not overrides.
pub fn encode_facts_file_header(w: &mut NativeFactsWriter, h: &FactsFileHeader) {
    let mut header = NativeFactsWriter::default();
    encode_facts_decoder_coverage(&mut header, &h.coverage);
    header.string_bytes(&h.map_module);
    for a in h.axis_widths {
        header.unsigned(a);
    }
    header.boolean(h.layout_detected);
    for &b in FILM_FACTS_FILE_MAGIC {
        w.byte(b);
    }
    w.unsigned(FILM_FACTS_FILE_CODEC as u64);
    w.unsigned(FILM_FACTS_FILE_SCHEMA as u64);
    w.unsigned(header.bytes().len() as u64);
    for &b in header.bytes() {
        w.byte(b);
    }
}
