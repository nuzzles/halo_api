//! Complete native document schema. End-to-end LegacyFilm assembly is still in progress.
use super::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayDocument {
    #[serde(flatten)]
    pub content: ReplayDocumentContent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<ReplayCoverage>,
}
pub const REPLAY_SCHEMA_VERSION: i64 = 68;
impl ReplayDocument {
    /// Publish kickoff dating only when the document has an established origin.
    pub fn detect_kickoff(&mut self) {
        let Some(coverage) = self.coverage.as_mut() else {
            return;
        };
        self.content.t0_film_ms = None;
        coverage.t0_film = None;
        if let Some(origin) = self.content.origin_ms {
            let (time, result) = detect_replay_t0(
                self.content.tracks.as_deref().unwrap_or_default(),
                self.content.frame_interval_ms,
                origin,
            );
            self.content.t0_film_ms = time;
            coverage.t0_film = Some(result);
        }
    }

    /// Attach the completed fallback report last, once every layer has run.
    pub fn set_fallbacks(&mut self, hits: impl IntoIterator<Item = ReplayFallbackHit>) {
        if let Some(coverage) = self.coverage.as_mut() {
            coverage.set_fallbacks(hits);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    // Fixture numbers are exactly representable small values. Ignore only JSON's
    // integer-versus-float spelling, not missing keys, nulls, order or numeric values.
    fn numbers(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Number(n) => {
                *n = serde_json::Number::from_f64(n.as_f64().unwrap()).unwrap();
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    numbers(v);
                }
            }
            serde_json::Value::Object(o) => {
                for v in o.values_mut() {
                    numbers(v);
                }
            }
            _ => {}
        }
    }
    #[test]
    fn captured_document_schema_keeps_every_field() {
        use std::io::Read;
        fn number_shapes(value: &mut serde_json::Value) {
            match value {
                serde_json::Value::Number(_) => *value = serde_json::json!(0),
                serde_json::Value::Array(a) => a.iter_mut().for_each(number_shapes),
                serde_json::Value::Object(o) => o.values_mut().for_each(number_shapes),
                _ => {}
            }
        }
        for fixture in [
            include_bytes!("fixtures/full-document-v41.json.zlib").as_slice(),
            include_bytes!("fixtures/decoded-kill-document-v41.json.zlib").as_slice(),
        ] {
            let mut bytes = Vec::new();
            flate2::read::ZlibDecoder::new(fixture)
                .read_to_end(&mut bytes)
                .unwrap();
            let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
            for row in rows {
                let mut expected = row["document"].clone();
                let parsed: ReplayDocument = serde_json::from_value(expected.clone()).unwrap();
                let mut actual = serde_json::to_value(parsed).unwrap();
                // Captured numerical values are checked by the full constructor oracle.
                // This independently rejects every lost key, null, element or scalar,
                // without f32 JSON spelling masking a dropped native schema field.
                number_shapes(&mut expected);
                number_shapes(&mut actual);
                assert_eq!(
                    actual, expected,
                    "native field preservation: {}",
                    row["folder"]
                );
            }
        }
    }
    #[test]
    fn native_complete_document_schema() {
        let rows: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("fixtures/document-schema-v41.json")).unwrap();
        let mut failures = Vec::new();
        for (i, mut row) in rows.into_iter().enumerate() {
            match serde_json::from_value::<ReplayDocument>(row.clone()) {
                Ok(doc) => {
                    let mut actual = serde_json::to_value(doc).unwrap();
                    numbers(&mut actual);
                    numbers(&mut row);
                    if actual != row {
                        failures.push(format!("case {i}: {actual} != {row}"));
                    }
                }
                Err(e) => failures.push(format!("case {i}: {e}")),
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}
