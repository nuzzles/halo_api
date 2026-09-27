//! Dated keyframe declarations used by the reference's later-keyframe fallback.
//! These are evidence for a binding, not decoded creation records.
use super::{DecodeError, packets, recover_keyframe_anchors};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnticipatedDeclaration {
    pub chunk_index: i64,
    pub archetype: u32,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnticipatedBindings {
    /// Full entity ID, including the two high bits, to chronologically dated declarations.
    pub entries: BTreeMap<u32, Vec<AnticipatedDeclaration>>,
    pub declarations: usize,
    pub conflicts: usize,
    pub heads: BTreeMap<u8, usize>,
}
impl AnticipatedBindings {
    /// Construct from recovered declarations. Each tuple is (full ID, chunk, archetype).
    pub fn from_declarations(declarations: impl IntoIterator<Item = (u32, i64, u32)>) -> Self {
        let mut out = Self::default();
        for (id, chunk_index, archetype) in declarations {
            out.declarations += 1;
            *out.heads.entry((id >> 30) as u8).or_default() += 1;
            out.entries
                .entry(id)
                .or_default()
                .push(AnticipatedDeclaration {
                    chunk_index,
                    archetype,
                });
        }
        for values in out.entries.values_mut() {
            super::native_sort::sort_by(values, |a, b| a.chunk_index.cmp(&b.chunk_index));
            out.conflicts += usize::from(values.iter().any(|d| d.archetype != values[0].archetype));
        }
        out
    }
    /// First declaration strictly after the current chunk. Earlier declarations do not qualify.
    pub fn after(&self, id: u32, chunk_index: i64) -> Option<AnticipatedDeclaration> {
        let values = self.entries.get(&id)?;
        values
            .get(values.partition_point(|d| d.chunk_index <= chunk_index))
            .copied()
    }
}
pub fn build_anticipated_bindings(
    chunks: &[FilmChunkData],
) -> Result<AnticipatedBindings, DecodeError> {
    let bytes: BTreeMap<_, _> = chunks.iter().map(|c| (c.metadata.index, &c.data)).collect();
    let mut declarations = Vec::new();
    for packet in packets::index(chunks)?
        .into_iter()
        .filter(|p| p.packet_type == 2)
    {
        let payload = &bytes[&packet.chunk_index]
            [packet.payload_offset..packet.payload_offset + packet.payload_size];
        declarations.extend(
            recover_keyframe_anchors(payload)
                .into_iter()
                .map(|a| (a.id, i64::from(packet.chunk_index), a.archetype)),
        );
    }
    Ok(AnticipatedBindings::from_declarations(declarations))
}

/// Native FilmContext selection and tolerant packet framing, including an empty
/// table when no contiguous data prefix exists. The strict API remains separate.
pub fn build_native_anticipated_bindings(chunks: &[FilmChunkData]) -> AnticipatedBindings {
    let mut declarations = Vec::new();
    for chunk in super::fire_events::native_chunk_prefix(chunks).unwrap_or_default() {
        for packet in super::fire_events::native_chunk_packets(chunk)
            .into_iter()
            .filter(|p| p.packet_type == 2)
        {
            let payload =
                &chunk.data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            declarations.extend(
                recover_keyframe_anchors(payload)
                    .into_iter()
                    .map(|a| (a.id, i64::from(packet.chunk_index), a.archetype)),
            );
        }
    }
    AnticipatedBindings::from_declarations(declarations)
}

/// Native context anticipation over loaded buffers. File numbers, rather than
/// buffer ordinals, date declarations; all signed native chunk values survive.
pub fn build_source_anticipated_bindings(
    source: Option<&super::FilmSource>,
) -> AnticipatedBindings {
    let mut declarations = Vec::new();
    if let Some(source) = source {
        for chunk in source.data_chunk_numbers() {
            let Some((data, packets)) = source.chunk_by_number(chunk) else {
                continue;
            };
            for packet in packets.iter().filter(|p| p.packet_type == 2) {
                let payload =
                    &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
                declarations.extend(
                    recover_keyframe_anchors(payload)
                        .into_iter()
                        .map(|a| (a.id, chunk, a.archetype)),
                );
            }
        }
    }
    AnticipatedBindings::from_declarations(declarations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;
    #[test]
    fn captured_keyframes_build_the_native_dated_declarations() {
        use crate::clients::hi::models::{FilmChunk, FilmChunkData};
        let inflate = |bytes: &[u8]| {
            let mut out = Vec::new();
            flate2::read::ZlibDecoder::new(bytes)
                .read_to_end(&mut out)
                .unwrap();
            out
        };
        let payload = inflate(include_bytes!("fixtures/captured-keyframe-v41.zlib"));
        let native: Value = serde_json::from_slice(&inflate(include_bytes!(
            "fixtures/captured-keyframe-v41-oracle.json.zlib"
        )))
        .unwrap();
        let mut data = vec![0; 16];
        data[0] = 2;
        data[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        data.extend_from_slice(&payload);
        let chunks: Vec<_> = [10, 3]
            .into_iter()
            .map(|index| FilmChunkData {
                metadata: FilmChunk {
                    index,
                    chunk_type: 2,
                    start_time_offset_ms: 0,
                    duration_ms: 0,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data: data.clone(),
            })
            .collect();
        let declarations: Vec<_> = [3, 10]
            .into_iter()
            .flat_map(|chunk| {
                native["anchors"].as_array().unwrap().iter().map(move |a| {
                    (
                        (a["Slot"].as_u64().unwrap() as u32)
                            | ((a["Gen"].as_u64().unwrap() as u32) << 30),
                        chunk,
                        a["TI"].as_u64().unwrap() as u32,
                    )
                })
            })
            .collect();
        let actual = build_anticipated_bindings(&chunks).unwrap();
        assert_eq!(actual, AnticipatedBindings::from_declarations(declarations));
        assert!(actual.entries.len() > 100);
    }
    #[test]
    fn full_id_lookup_dates_conflicts_and_counters_match_go() {
        let mut text = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/biped-scan-levelup-v41.json.zlib")[..],
        )
        .read_to_string(&mut text)
        .unwrap();
        let oracle: Value = serde_json::from_str(&text).unwrap();
        let rows = oracle["anticipated"].as_array().unwrap();
        assert_eq!(rows.len(), 128);
        for row in rows {
            let input: Vec<(u32, i64, u32)> = serde_json::from_value(row["input"].clone()).unwrap();
            let table = AnticipatedBindings::from_declarations(input.iter().copied());
            assert_eq!(
                table.entries.len(),
                row["entries"].as_u64().unwrap() as usize
            );
            assert_eq!(json!(table.heads), row["heads"]);
            assert_eq!(json!(table.conflicts), row["conflicts"]);
            assert_eq!(json!(table.declarations), row["declarations"]);
            for q in row["queries"].as_array().unwrap() {
                let actual = table.after(
                    q["id"].as_u64().unwrap() as u32,
                    q["chunk"].as_i64().unwrap(),
                );
                let expected = q["ok"].as_bool().unwrap().then(|| AnticipatedDeclaration {
                    archetype: q["ti"].as_u64().unwrap() as u32,
                    chunk_index: q["declarant"].as_i64().unwrap(),
                });
                assert_eq!(actual, expected, "{q}");
            }
            let mut shuffled = input;
            shuffled.reverse();
            assert_eq!(AnticipatedBindings::from_declarations(shuffled), table);
            assert_eq!(
                serde_json::from_value::<AnticipatedBindings>(json!(table)).unwrap(),
                table
            );
        }
    }
}
