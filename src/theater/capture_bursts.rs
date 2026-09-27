//! Independent CTF capture evidence from all six score-contribution tiers.
use super::fire_events::native_chunk_packets;
use super::*;
use crate::clients::hi::models::FilmChunkData;
const TIERS: [&[u8]; 6] = [
    &[0xa4, 0, 0, 0],
    &[3, 0x48, 0, 0, 1],
    &[6, 0x90, 0, 0, 2],
    &[0x0d, 0x20, 0, 0, 5],
    &[0x1a, 0x40, 0, 0, 10],
    &[0x34, 0x80, 0, 0, 21],
];
pub fn count_capture_burst_tiers(payload: &[u8]) -> usize {
    TIERS
        .iter()
        .filter(|t| payload.windows(t.len()).any(|w| w == **t))
        .count()
}
/// The first FRAME (packet type zero) anchors the chunk clock and is skipped.
/// Timestamp subtraction wraps unsigned before being interpreted as signed.
pub fn capture_bursts_in_frames<'a>(
    frames: impl IntoIterator<Item = (u64, &'a [u8])>,
    start_ms: i64,
) -> Vec<i64> {
    let mut frames = frames.into_iter();
    let Some((first, _)) = frames.next() else {
        return Vec::new();
    };
    frames
        .filter(|(_, p)| count_capture_burst_tiers(p) >= 6)
        .map(|(ts, _)| start_ms.wrapping_add((ts.wrapping_sub(first) as i64) / 1000))
        .collect()
}
pub fn scan_capture_bursts(chunks: &[FilmChunkData]) -> Result<Vec<i64>, DecodeError> {
    let mut out = Vec::new();
    for c in chunks.iter().filter(|c| c.metadata.chunk_type == 2) {
        let packets = native_chunk_packets(c);
        out.extend(capture_bursts_in_frames(
            packets.iter().filter(|p| p.packet_type == 0).map(|p| {
                (
                    p.timestamp_us,
                    &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                )
            }),
            c.metadata.start_time_offset_ms,
        ));
    }
    out.sort_unstable();
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_capture_burst_tiers_and_clock() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/capture-bursts-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        for (i, r) in rows.iter().enumerate() {
            let mut frames = Vec::new();
            for f in r["frames"].as_array().unwrap() {
                let hex = f["hex"].as_str().unwrap();
                let data: Vec<_> = (0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                    .collect();
                assert_eq!(
                    count_capture_burst_tiers(&data),
                    f["tiers"].as_u64().unwrap() as usize
                );
                frames.push((f["ts"].as_u64().unwrap(), data));
            }
            assert_eq!(
                capture_bursts_in_frames(
                    frames.iter().map(|(ts, b)| (*ts, b.as_slice())),
                    r["start"].as_i64().unwrap()
                ),
                serde_json::from_value::<Vec<i64>>(r["bursts"].clone()).unwrap(),
                "bursts {i}"
            );
        }
    }
}
