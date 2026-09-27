//! Complete native LEVELUPFILMFACTS container. Revision freshness is a separate
//! caller decision; decoding checks versions and the map cooking key.
use super::*;
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NativeFilmFactsFile {
    pub coverage: FactsDecoderCoverage,
    pub facts: NativeFilmFacts,
    /// These native FilmInputs fields are stored after the embedded facts blob.
    pub mode_guards: FactsModeGuards,
    pub identity: Option<FactsFileIdentity>,
    pub fallbacks: Option<Vec<FactsFallback>>,
    pub statborg: FactsFileStatborg,
    pub kills: Option<FactsKillsResult>,
}
/// A failed file encode returns no bytes, including embedded blob JSON failures.
pub fn encode_film_facts_file(f: &NativeFilmFactsFile) -> Result<Vec<u8>, String> {
    let mut w = NativeFactsWriter::default();
    encode_facts_file_header(
        &mut w,
        &FactsFileHeader {
            coverage: f.coverage.clone(),
            map_module: f.facts.header.map_module.clone(),
            axis_widths: f.facts.header.axis_widths,
            layout_detected: f.facts.header.layout_detected,
            ..Default::default()
        },
    );
    let blob = encode_film_facts(&f.facts);
    if let Some(e) = blob.error() {
        return Err(e.to_owned());
    }
    let mut inputs = NativeFactsWriter::default();
    inputs.string_bytes(blob.bytes());
    encode_facts_mode_guards(&mut inputs, &f.mode_guards);
    if let Some(e) = inputs.error() {
        return Err(e.to_owned());
    }
    w.unsigned(1);
    w.string_bytes(inputs.bytes());
    w.unsigned(2);
    w.string_bytes(&encode_facts_identity_json(f.identity.as_ref()));
    w.unsigned(3);
    w.string_bytes(&encode_facts_fallbacks_json(f.fallbacks.as_deref()));
    w.unsigned(4);
    w.string_bytes(&encode_facts_statborg_json(&f.statborg));
    let kills = encode_facts_kills_json(f.kills.as_ref())
        .map_err(|e| format!("faits de film : section killsource : {e}"))?;
    w.unsigned(5);
    w.string_bytes(&kills);
    Ok(w.bytes().to_vec())
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FactsFileDecodeError {
    #[error(transparent)]
    Header(#[from] FactsFileHeaderError),
    #[error(transparent)]
    Cooking(#[from] FactsHeaderError),
    #[error(transparent)]
    Blob(#[from] FactsDecodeError),
    #[error("{0}")]
    Section(String),
    /// The pinned implementation panics when indexing at a negative body offset.
    #[error("invalid facts file body offset {0}")]
    InvalidBodyOffset(i64),
}
/// Process sections in recorded order, skipping unknown IDs. Native files have
/// no mandatory-section check. Repeated JSON sections update existing state.
/// Any error rejects the file; no partially assembled value is returned.
pub fn decode_film_facts_file(
    blob: &[u8],
    entry: &FactsMapEntry,
) -> Result<NativeFilmFactsFile, FactsFileDecodeError> {
    let read = decode_facts_file_header(blob);
    if let Some(e) = read.error {
        return Err(e.into());
    }
    let h = read.header;
    verify_facts_cooking_key(&h.map_module, h.axis_widths, h.layout_detected, entry)?;
    if h.body_offset < 0 || h.body_offset as u64 > blob.len() as u64 {
        return Err(FactsFileDecodeError::InvalidBodyOffset(h.body_offset));
    }
    let mut out = NativeFilmFactsFile {
        coverage: h.coverage,
        ..Default::default()
    };
    let mut identity = FactsIdentityJsonReader::default();
    let mut fallbacks = FactsFallbacksJsonReader::default();
    let mut statborg = FactsStatborgJsonReader::default();
    let mut kills = FactsKillsJsonReader::default();
    let mut r = NativeFactsReader::new(blob);
    r.section(h.body_offset);
    while r.remaining() > 0 && r.error().is_none() {
        let id = r.unsigned() as i64;
        let length = r.unsigned() as i64;
        let Some(charge) = r.section(length) else {
            break;
        };
        let json_error = |label: &str, e: String| {
            FactsFileDecodeError::Section(format!("faits de film : section {label} : {e}"))
        };
        match id {
            1 => {
                let mut section = NativeFactsReader::new(charge);
                let length = section.unsigned() as i64;
                let inner = section
                    .section(length)
                    .ok_or_else(|| json_error("entrees", section.error().unwrap().to_owned()))?;
                out.facts = decode_film_facts(inner, entry)?;
                out.mode_guards = decode_facts_mode_guards(&mut section);
                if let Some(e) = section.error() {
                    return Err(FactsFileDecodeError::Section(format!(
                        "faits de film : canaux gardes : {e}"
                    )));
                }
                // Native permits trailing bytes in this section.
            }
            2 => {
                out.identity = identity
                    .read(charge)
                    .map_err(|e| json_error("identite du film", e))?
            }
            3 => {
                out.fallbacks = fallbacks
                    .read(charge)
                    .map_err(|e| json_error("rapport de replis", e))?
            }
            4 => {
                out.statborg = statborg
                    .read(charge)
                    .map_err(|e| json_error("statborg", e))?
            }
            5 => {
                out.kills = kills
                    .read(charge)
                    .map_err(|e| json_error("killsource", e))?
            }
            _ => {}
        }
    }
    if let Some(e) = r.error() {
        return Err(FactsFileDecodeError::Section(format!(
            "faits de film : cadre de section illisible : {e}"
        )));
    }
    Ok(out)
}
