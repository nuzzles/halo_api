//! Projectile/world tracks in the derived native facts cache.
use super::{NativeFactsReader, NativeFactsWriter, WorldObjectTrack};
#[derive(Debug, Clone, PartialEq)]
pub struct FactsProjectileSample {
    pub timestamp_us: u64,
    /// Native signed machine word, retained at 64 bits on WASM too.
    pub chunk: i64,
    pub position: [f32; 3],
    pub at_rest: bool,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsProjectileTrack {
    pub slot: u32,
    pub generation: u32,
    /// Native decode leaves points nil when zero points were decoded.
    pub points: Option<Vec<FactsProjectileSample>>,
}
impl From<&WorldObjectTrack> for FactsProjectileTrack {
    fn from(track: &WorldObjectTrack) -> Self {
        Self {
            slot: track.slot,
            generation: track.generation,
            points: Some(
                track
                    .pts
                    .iter()
                    .map(|p| FactsProjectileSample {
                        timestamp_us: p.timestamp_us,
                        chunk: p.chunk,
                        position: [p.x, p.y, p.z],
                        at_rest: p.at_rest,
                    })
                    .collect(),
            ),
        }
    }
}
pub fn encode_facts_tracks(writer: &mut NativeFactsWriter, tracks: &[FactsProjectileTrack]) {
    writer.unsigned(tracks.len() as u64);
    for track in tracks {
        writer.unsigned(u64::from(track.slot));
        writer.unsigned(u64::from(track.generation));
        let points = track.points.as_deref().unwrap_or(&[]);
        writer.unsigned(points.len() as u64);
        let mut timestamp = 0u64;
        for point in points {
            writer.unsigned(point.timestamp_us.wrapping_sub(timestamp));
            timestamp = point.timestamp_us;
            for value in point.position {
                writer.float32(value);
            }
            writer.boolean(point.at_rest);
            writer.unsigned(point.chunk as u64);
        }
    }
}
/// Retain native partial tracks/samples on failure. Inspect reader.error().
pub fn decode_facts_tracks(reader: &mut NativeFactsReader<'_>) -> Vec<FactsProjectileTrack> {
    let count = reader.count(3);
    let mut tracks = Vec::new();
    for _ in 0..count {
        if reader.error().is_some() {
            break;
        }
        let slot = reader.unsigned() as u32;
        let generation = reader.unsigned() as u32;
        let count = reader.count(2);
        let mut timestamp = 0u64;
        let mut points = vec![];
        for _ in 0..count {
            if reader.error().is_some() {
                break;
            }
            timestamp = timestamp.wrapping_add(reader.unsigned());
            let position = [reader.float32(), reader.float32(), reader.float32()];
            let at_rest = reader.boolean();
            let chunk = reader.unsigned() as i64;
            points.push(FactsProjectileSample {
                timestamp_us: timestamp,
                chunk,
                position,
                at_rest,
            });
        }
        tracks.push(FactsProjectileTrack {
            slot,
            generation,
            points: if points.is_empty() {
                None
            } else {
                Some(points)
            },
        });
    }
    tracks
}
