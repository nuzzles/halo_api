//! Native pickup attempts, including refusal points and synthetic zero-tail reads.
use super::{BipedPickupStats, EventReference, FilmPacket, bits::Cursor, event_heads, fire_events};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativePickupOutcome {
    EmptyList,
    BoardVehicle,
    OtherType,
    NoReference,
    NoCatalog,
    Accepted,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePickupRead {
    pub config: bool,
    pub more: bool,
    pub kind: u8,
    pub references: Vec<EventReference>,
    pub class: Option<u8>,
    pub catalog_id: Option<u32>,
    pub more_events: Option<bool>,
    pub source_bits: usize,
    pub end_bit: usize,
    pub padded_bits: usize,
    pub outcome: NativePickupOutcome,
}
impl NativePickupRead {
    /// Native slot, which may include synthetic tail bits. This is not a player identity.
    pub fn slot(&self) -> Option<u32> {
        Some(512 + self.references.first()?.value.as_ref()?.index)
    }
    pub fn unit_reference_recorded(&self) -> bool {
        self.references
            .first()
            .is_some_and(|r| r.value.is_some() && r.end_bit <= self.source_bits)
    }
    pub fn catalog_recorded(&self) -> bool {
        self.catalog_id.is_some() && self.end_bit.saturating_sub(1) <= self.source_bits
    }
}

/// Reproduce decodeBipedPickup, including its early returns. Direct decoding
/// updates only decoder counters; packet, published and off-band counts belong
/// to the scanner. The config bit is not an admission gate here.
pub fn decode_native_biped_pickup(data: &[u8], stats: &mut BipedPickupStats) -> NativePickupRead {
    let mut r = Cursor::new_padded(data, 0);
    let mut out = NativePickupRead {
        config: r.bit().unwrap(),
        more: r.bit().unwrap(),
        kind: r.read(7).unwrap() as u8,
        references: Vec::new(),
        class: None,
        catalog_id: None,
        more_events: None,
        source_bits: data.len() * 8,
        end_bit: 9,
        padded_bits: 0,
        outcome: NativePickupOutcome::EmptyList,
    };
    out.outcome = (|| {
        if !out.more {
            return NativePickupOutcome::EmptyList;
        }
        match out.kind {
            9 => stats.type_9 += 1,
            8 => {
                stats.type_8 += 1;
                return NativePickupOutcome::BoardVehicle;
            }
            _ => {
                stats.other_type += 1;
                return NativePickupOutcome::OtherType;
            }
        }
        out.references
            .push(event_heads::reference(&mut r, 2).unwrap());
        if out.references[0].value.is_none() {
            stats.refused_no_ref += 1;
            return NativePickupOutcome::NoReference;
        }
        for domain in [8, 7] {
            out.references
                .push(event_heads::reference(&mut r, domain).unwrap());
        }
        if out.references[1..].iter().any(|r| r.value.is_some()) {
            stats.unexpected_wide_ref += 1;
        }
        out.class = Some(r.read(3).unwrap() as u8);
        if !r.bit().unwrap() {
            stats.refused_no_catalog += 1;
            return NativePickupOutcome::NoCatalog;
        }
        out.catalog_id = Some(r.read(32).unwrap() as u32);
        out.more_events = Some(r.bit().unwrap());
        if out.more_events == Some(true) {
            stats.multi_event += 1;
        }
        NativePickupOutcome::Accepted
    })();
    out.end_bit = r.position;
    out.padded_bits = r.position.saturating_sub(out.source_bits);
    out
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmNativePickupAttempt {
    pub source: FilmPacket,
    pub packet_index: usize,
    pub read: NativePickupRead,
    /// Native publication, including the source slot-band sentinel. Accepted
    /// padded fields still require recordedness checks before interpretation.
    pub published: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePickupStream {
    pub slot_band: Option<[u32; 2]>,
    pub stats: BipedPickupStats,
    pub attempts: Vec<FilmNativePickupAttempt>,
    /// Native ErrNoFilmChunk, retained separately from a successful empty scan.
    pub no_film_chunks: bool,
}
/// Native packet order is preserved; this scanner does not sort by timestamp.
pub fn scan_native_biped_pickups(chunks: &[FilmChunkData]) -> NativePickupStream {
    let mut out = NativePickupStream::default();
    let Ok(selected) = fire_events::native_chunk_prefix(chunks) else {
        out.no_film_chunks = true;
        return out;
    };
    out.slot_band =
        super::biped_scan::biped_slot_band(chunks).expect("band discovery is infallible");
    for chunk in selected {
        for (packet_index, source) in fire_events::native_chunk_packets(chunk)
            .into_iter()
            .enumerate()
        {
            if source.packet_type != 0 || source.payload_size < 2 {
                continue;
            }
            let data =
                &chunk.data[source.payload_offset..source.payload_offset + source.payload_size];
            if data[0] != 0xc4 {
                continue;
            }
            out.stats.packets += 1;
            let read = decode_native_biped_pickup(data, &mut out.stats);
            let mut published = read.outcome == NativePickupOutcome::Accepted;
            if published
                && out
                    .slot_band
                    .is_some_and(|[lo, hi]| !(lo..=hi).contains(&read.slot().unwrap()))
            {
                out.stats.refused_off_band += 1;
                published = false;
            }
            out.stats.published += usize::from(published);
            out.attempts.push(FilmNativePickupAttempt {
                source,
                packet_index,
                read,
                published,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;
    use std::io::Read;
    fn bytes(h: &str) -> Vec<u8> {
        (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect()
    }
    fn stats(s: &BipedPickupStats, e: &serde_json::Value) {
        for (key, value) in [
            ("Packets", s.packets),
            ("Type9", s.type_9),
            ("Type8", s.type_8),
            ("OtherType", s.other_type),
            ("Published", s.published),
            ("MultiEvent", s.multi_event),
            ("RefusedNoRef", s.refused_no_ref),
            ("RefusedNoCatalog", s.refused_no_catalog),
            ("RefusedOffBand", s.refused_off_band),
            ("UnexpectedWideRef", s.unexpected_wide_ref),
        ] {
            assert_eq!(value as u64, e[key].as_u64().unwrap(), "{key}");
        }
        assert_eq!(s.truncated, 0);
    }
    #[test]
    fn native_pickup_padding_counters_and_source() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/pickup-padding-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let oracle: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let rows = oracle["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 3328);
        let mut padded_catalogs = 0;
        let mut early_no_ref = 0;
        for row in rows {
            let data = bytes(row["hex"].as_str().unwrap());
            let mut s = BipedPickupStats::default();
            let read = decode_native_biped_pickup(&data, &mut s);
            stats(&s, &row["stats"]);
            assert_eq!(read.end_bit as u64, row["end"].as_u64().unwrap());
            assert_eq!(
                read.outcome == NativePickupOutcome::Accepted,
                row["ok"].as_bool().unwrap()
            );
            assert_eq!(read.source_bits, data.len() * 8);
            assert_eq!(
                read.padded_bits,
                read.end_bit.saturating_sub(data.len() * 8)
            );
            if read.outcome == NativePickupOutcome::Accepted {
                assert_eq!(
                    u64::from(read.slot().unwrap()),
                    row["event"]["Slot"].as_u64().unwrap()
                );
                assert_eq!(
                    u64::from(read.class.unwrap()),
                    row["event"]["Class"].as_u64().unwrap()
                );
                assert_eq!(
                    u64::from(read.catalog_id.unwrap()),
                    row["event"]["CatalogID"].as_u64().unwrap()
                );
                padded_catalogs += usize::from(!read.catalog_recorded());
            }
            if read.outcome == NativePickupOutcome::NoReference {
                assert_eq!(read.end_bit, 10);
                assert_eq!(read.references.len(), 1);
                assert!(read.class.is_none());
                early_no_ref += 1;
            }
            let restored: NativePickupRead =
                serde_json::from_slice(&serde_json::to_vec(&read).unwrap()).unwrap();
            assert_eq!(restored, read);
        }
        assert!(padded_catalogs > 0 && early_no_ref > 0);
        for row in oracle["sources"].as_array().unwrap() {
            let data = bytes(row["hex"].as_str().unwrap());
            let chunks = [FilmChunkData {
                metadata: FilmChunk {
                    index: row["index"].as_i64().unwrap() as i32,
                    chunk_type: 2,
                    start_time_offset_ms: 90000,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            }];
            let out = scan_native_biped_pickups(&chunks);
            stats(&out.stats, &row["stats"]);
            assert_eq!(out.no_film_chunks, row["no_chunks"].as_bool().unwrap());
            let expected = row["attempts"].as_array().unwrap();
            assert_eq!(out.attempts.len(), expected.len());
            for (a, e) in out.attempts.iter().zip(expected) {
                assert_eq!(a.packet_index as u64, e["packet"].as_u64().unwrap());
                assert_eq!(a.source.payload_offset as u64, e["start"].as_u64().unwrap());
                assert_eq!(a.source.payload_size as u64, e["size"].as_u64().unwrap());
                assert_eq!(a.source.timestamp_us, e["time"].as_u64().unwrap());
                assert_eq!(a.published, e["published"].as_bool().unwrap());
            }
            let restored: NativePickupStream =
                serde_json::from_slice(&serde_json::to_vec(&out).unwrap()).unwrap();
            assert_eq!(restored, out);
        }
    }
}
