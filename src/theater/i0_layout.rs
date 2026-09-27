//! Measured i0 wire-layout detection. This never supplies world-coordinate bounds.
use super::{bits::Bits, fire_events, native_sort, recover_keyframe_anchors};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const I0_DETECT_WINDOW: usize = 72;
pub const I0_SPINE_BITS: i64 = 3;
pub const I0_USE_DEFAULT_BITS: i64 = 1;
pub const DEFAULT_I0_GATE_BITS: i64 = I0_SPINE_BITS + I0_USE_DEFAULT_BITS + 1;

/// Native FilmContext constructor precedence. A forced layout is copied even
/// when invalid; only a catalog layout is validity-gated. None requests detection.
/// This selects metadata and does not validate it for a particular decoder.
pub fn resolve_imposed_i0_layout(
    forced: Option<&I0Layout>,
    catalog_layout: Option<&I0Layout>,
) -> Option<I0Layout> {
    forced
        .or_else(|| catalog_layout.filter(|layout| layout.valid()))
        .cloned()
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct I0Layout {
    pub gate_bits: i64,
    pub axis_widths: [u64; 3],
    pub region: u32,
}
impl I0Layout {
    pub fn valid(&self) -> bool {
        self.gate_bits >= I0_SPINE_BITS + I0_USE_DEFAULT_BITS
            && self.axis_widths.iter().all(|w| (8..=26).contains(w))
    }
    pub fn total_bits(&self) -> i64 {
        self.gate_bits.wrapping_add(
            self.axis_widths
                .iter()
                .fold(0_u64, |a, w| a.wrapping_add(*w)) as i64,
        )
    }
    pub fn axis_offset(&self, axis: usize) -> Option<i64> {
        (axis <= 3).then(|| {
            self.gate_bits.wrapping_add(
                self.axis_widths[..axis]
                    .iter()
                    .fold(0_u64, |a, w| a.wrapping_add(*w)) as i64,
            )
        })
    }
    /// Native signed-axis contract: nonpositive axes return the gate offset.
    /// Axes above three return None instead of the native array-bounds panic.
    pub fn axis_offset_signed(&self, axis: i64) -> Option<i64> {
        if axis <= 0 {
            Some(self.gate_bits)
        } else {
            usize::try_from(axis)
                .ok()
                .and_then(|axis| self.axis_offset(axis))
        }
    }
}
impl std::fmt::Display for I0Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gate={}", self.gate_bits)?;
        if self.region != 0 {
            write!(f, " region={}", self.region)?;
        }
        write!(
            f,
            " {}/{}/{} (i0={} bits)",
            self.axis_widths[0],
            self.axis_widths[1],
            self.axis_widths[2],
            self.total_bits()
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct I0LayoutSample {
    pub slot: u32,
    pub chunk: i64,
    pub packet: i64,
    pub bits: [u64; 2],
    pub index_bit: u8,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct I0LayoutReport {
    pub records: usize,
    pub pairs: usize,
    pub flip_rate: Vec<f64>,
    pub boundaries: Vec<usize>,
    pub index_bit_ones: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum I0LayoutRefusal {
    NoChunks,
    NoBipeds,
    Inconclusive,
    Implausible,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct I0LayoutDetection {
    pub layout: Option<I0Layout>,
    pub report: I0LayoutReport,
    pub refusal: Option<I0LayoutRefusal>,
}
pub fn collect_i0_layout_samples(
    data: &[u8],
    band: [u32; 2],
    chunk: i64,
    packet: i64,
) -> Vec<I0LayoutSample> {
    let bits = Bits(data);
    let mut out = Vec::new();
    let mut at = 0;
    while at + 21 + 12 + I0_DETECT_WINDOW <= bits.len() {
        let Some(h) =
            super::biped_scan::match_biped_header_raw(bits, at, band, true, I0_DETECT_WINDOW)
                .filter(|h| bits.read(h.position_bit, 4) == Some(0))
        else {
            at += 1;
            continue;
        };
        let start = h.position_bit;
        out.push(I0LayoutSample {
            slot: h.slot,
            chunk,
            packet,
            bits: [
                bits.read(start, 64).unwrap(),
                bits.read(start + 64, 8).unwrap() << 56,
            ],
            index_bit: bits.read(start + 4, 1).unwrap() as u8,
        });
        at = start + I0_DETECT_WINDOW / 2;
    }
    out
}
pub fn i0_layout_boundaries(flip_rate: &[f64]) -> Vec<usize> {
    (1..flip_rate.len())
        .filter(|&i| flip_rate[i - 1] >= 0.10 && flip_rate[i] * 8. < flip_rate[i - 1])
        .collect()
}
pub fn profile_i0_layout(samples: &[I0LayoutSample]) -> I0LayoutReport {
    let mut report = I0LayoutReport {
        records: samples.len(),
        flip_rate: vec![0.; 72],
        ..Default::default()
    };
    let mut by_slot: BTreeMap<u32, Vec<I0LayoutSample>> = BTreeMap::new();
    for s in samples {
        by_slot.entry(s.slot).or_default().push(s.clone());
        report.index_bit_ones += usize::from(s.index_bit == 1);
    }
    let mut flips = [0_usize; 72];
    for ss in by_slot.values_mut() {
        native_sort::sort_by(ss, |a, b| {
            a.chunk.cmp(&b.chunk).then(a.packet.cmp(&b.packet))
        });
        for pair in ss.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            if a.chunk != b.chunk || b.packet <= a.packet || b.packet.wrapping_sub(a.packet) > 3 {
                continue;
            }
            report.pairs += 1;
            for (k, n) in flips.iter_mut().enumerate() {
                *n += (((a.bits[k / 64] ^ b.bits[k / 64]) >> (63 - k % 64)) & 1) as usize;
            }
        }
    }
    if report.pairs > 0 {
        for (rate, n) in report.flip_rate.iter_mut().zip(flips) {
            *rate = n as f64 / report.pairs as f64;
        }
    }
    report
}
pub fn detect_i0_layout_from_samples(samples: &[I0LayoutSample]) -> I0LayoutDetection {
    let mut report = profile_i0_layout(samples);
    report.boundaries = i0_layout_boundaries(&report.flip_rate);
    if report.boundaries.len() < 3 {
        return I0LayoutDetection {
            layout: None,
            report,
            refusal: Some(I0LayoutRefusal::Inconclusive),
        };
    }
    let b = &report.boundaries;
    let layout = I0Layout {
        gate_bits: DEFAULT_I0_GATE_BITS,
        axis_widths: [
            (b[0] as i64 - 5) as u64,
            (b[1] - b[0]) as u64,
            (b[2] - b[1]) as u64,
        ],
        region: 0,
    };
    let refusal = (!layout.valid()).then_some(I0LayoutRefusal::Implausible);
    I0LayoutDetection {
        layout: Some(layout),
        report,
        refusal,
    }
}
/// Native biped band discovery from the first keyframe of each selected chunk,
/// plus the chunk after the last selection. Missing chunks are skipped. The
/// inclusive bounds represent the native filled band without allocating its slots.
/// An empty selection is empty here; callers choose the native prefix explicitly.
pub fn source_biped_slot_band(source: &super::FilmSource, chunks: &[i64]) -> Option<[u32; 2]> {
    let last = *chunks.last()?;
    let mut band: Option<[u32; 2]> = None;
    for number in chunks
        .iter()
        .copied()
        .chain(std::iter::once(last.wrapping_add(1)))
    {
        let Some((data, packets)) = source.chunk_by_number(number) else {
            continue;
        };
        let Some(packet) = packets.iter().find(|p| p.packet_type == 2) else {
            continue;
        };
        for anchor in recover_keyframe_anchors(
            &data[packet.payload_offset..packet.payload_offset + packet.payload_size],
        )
        .into_iter()
        .filter(|a| a.archetype == 35)
        {
            let slot = anchor.id & 0x3fff_ffff;
            band = Some(band.map_or([slot, slot], |[lo, hi]| [lo.min(slot), hi.max(slot)]));
        }
    }
    band
}

/// Detect the native i0 layout from a loaded source without copying or inflating
/// its buffers again. Uses at most six prefix chunks and their following keyframe.
/// Refusals retain the measurements and any implausible candidate layout.
pub fn detect_source_i0_layout(source: &super::FilmSource) -> I0LayoutDetection {
    let refuse = |reason| I0LayoutDetection {
        layout: None,
        report: Default::default(),
        refusal: Some(reason),
    };
    let numbers = source.data_chunk_numbers();
    if numbers.is_empty() {
        return refuse(I0LayoutRefusal::NoChunks);
    }
    let selected = &numbers[..numbers.len().min(6)];
    let Some(band) = source_biped_slot_band(source, selected) else {
        return refuse(I0LayoutRefusal::NoBipeds);
    };
    let mut samples = Vec::new();
    for &number in selected {
        let Some((data, packets)) = source.chunk_by_number(number) else {
            continue;
        };
        for (index, packet) in packets
            .iter()
            .enumerate()
            .filter(|(_, p)| p.packet_type == 0)
        {
            samples.extend(collect_i0_layout_samples(
                &data[packet.payload_offset..packet.payload_offset + packet.payload_size],
                band,
                number,
                index as i64,
            ));
        }
    }
    detect_i0_layout_from_samples(&samples)
}

/// Native diagnostic detector uses at most six chunks, plus the next keyframe
/// for the candidate biped band. An inconclusive report retains all measurements.
pub fn detect_film_i0_layout(chunks: &[FilmChunkData]) -> I0LayoutDetection {
    let refuse = |reason| I0LayoutDetection {
        layout: None,
        report: Default::default(),
        refusal: Some(reason),
    };
    let Ok(selected) = fire_events::native_chunk_prefix(chunks) else {
        return refuse(I0LayoutRefusal::NoChunks);
    };
    let selected = &selected[..selected.len().min(6)];
    let last = selected.last().unwrap().metadata.index;
    let mut band: Option<[u32; 2]> = None;
    for c in (1..=last.saturating_add(1))
        .filter_map(|index| chunks.iter().find(|c| c.metadata.index == index))
    {
        if let Some(p) = fire_events::native_chunk_packets(c)
            .into_iter()
            .find(|p| p.packet_type == 2)
        {
            for a in recover_keyframe_anchors(
                &c.data[p.payload_offset..p.payload_offset + p.payload_size],
            )
            .into_iter()
            .filter(|a| a.archetype == 35)
            {
                let slot = a.id & 0x3fff_ffff;
                band = Some(band.map_or([slot, slot], |[lo, hi]| [lo.min(slot), hi.max(slot)]));
            }
        }
    }
    let Some(band) = band else {
        return refuse(I0LayoutRefusal::NoBipeds);
    };
    let mut samples = Vec::new();
    for c in selected {
        for (index, p) in fire_events::native_chunk_packets(c)
            .into_iter()
            .enumerate()
            .filter(|(_, p)| p.packet_type == 0)
        {
            samples.extend(collect_i0_layout_samples(
                &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                band,
                i64::from(c.metadata.index),
                index as i64,
            ));
        }
    }
    detect_i0_layout_from_samples(&samples)
}
#[cfg(test)]
mod tests {
    #[test]
    fn native_context_constructor_layout_precedence() {
        use std::io::Read;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/context-constructor-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        let layout = |v: &serde_json::Value| {
            (!v.is_null()).then(|| super::I0Layout {
                gate_bits: v["GateBits"].as_i64().unwrap(),
                axis_widths: serde_json::from_value(v["AxisW"].clone()).unwrap(),
                region: v["Region"].as_u64().unwrap() as u32,
            })
        };
        assert_eq!(rows.len(), 256);
        let mut invalid_forced = 0;
        for row in rows {
            let forced = layout(&row["forced"]);
            let catalog = layout(&row["catalog"]);
            let actual = super::resolve_imposed_i0_layout(forced.as_ref(), catalog.as_ref());
            assert_eq!(actual, layout(&row["imposed"]));
            assert_eq!(serde_json::json!(actual.is_some()), row["simulation"]);
            if forced.as_ref().is_some_and(|f| !f.valid()) {
                invalid_forced += 1;
                assert_eq!(actual, forced);
            }
            // Native witnesses establish the constructor's ownership rules;
            // the Rust helper returns an owned clone, not an observer context.
            for key in [
                "copied",
                "same_observer",
                "reader_observer",
                "frame_observer_nil",
            ] {
                assert_eq!(row[key], true, "{key}");
            }
        }
        assert!(invalid_forced > 0);
    }
    use super::*;
    use std::io::Read;
    #[test]
    fn native_i0_layout_values() {
        #[derive(Deserialize)]
        struct Case {
            layout: I0Layout,
            valid: bool,
            total: i64,
            offsets: Vec<i64>,
            signed_offsets: Vec<serde_json::Value>,
            display: String,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/i0-values-v41.json.zlib")[..])
            .read_to_string(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 512);
        for c in cases {
            assert_eq!(c.layout.valid(), c.valid);
            assert_eq!(c.layout.total_bits(), c.total);
            assert_eq!(c.layout.to_string(), c.display);
            for (axis, offset) in c.offsets.into_iter().enumerate() {
                assert_eq!(c.layout.axis_offset(axis), Some(offset));
            }
            assert_eq!(c.layout.axis_offset(4), None);
            for case in c.signed_offsets {
                let axis = case["axis"].as_i64().unwrap();
                let expected =
                    (!case["panic"].as_bool().unwrap()).then(|| case["value"].as_i64().unwrap());
                assert_eq!(c.layout.axis_offset_signed(axis), expected);
            }
        }
    }
    #[test]
    fn native_i0_sampling_and_flip_profile() {
        #[derive(Deserialize)]
        struct Case {
            samples: Vec<I0LayoutSample>,
            records: usize,
            pairs: usize,
            rates: Vec<f64>,
            boundaries: Vec<usize>,
            ones: usize,
            hex: String,
            chunk: i64,
            collected: Vec<I0LayoutSample>,
        }
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/i0-layout-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 512);
        let mut plausible = 0;
        for (i, c) in cases.into_iter().enumerate() {
            let d = detect_i0_layout_from_samples(&c.samples);
            assert_eq!(d.report.records, c.records, "records {i}");
            assert_eq!(d.report.pairs, c.pairs, "pairs {i}");
            assert_eq!(d.report.index_bit_ones, c.ones, "index ones {i}");
            assert_eq!(d.report.flip_rate, c.rates, "rates {i}");
            assert_eq!(d.report.boundaries, c.boundaries, "boundaries {i}");
            plausible += usize::from(d.refusal.is_none());
            let bytes: Vec<_> = (0..c.hex.len())
                .step_by(2)
                .map(|p| u8::from_str_radix(&c.hex[p..p + 2], 16).unwrap())
                .collect();
            assert_eq!(
                collect_i0_layout_samples(&bytes, [512, 513], c.chunk, 19),
                c.collected,
                "samples {i}"
            );
        }
        assert!(plausible > 50);
        assert_eq!(
            detect_film_i0_layout(&[]).refusal,
            Some(I0LayoutRefusal::NoChunks)
        );
    }
    #[test]
    fn native_loaded_i0_selection() {
        use crate::clients::hi::models::FilmChunk;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/i0-loaded-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 144);
        assert_eq!(
            rows.iter()
                .flat_map(|r| r["scans"].as_array().unwrap())
                .map(|c| c["world_records"].as_array().unwrap().len())
                .sum::<usize>(),
            18911
        );
        assert_eq!(
            rows.iter()
                .flat_map(|r| r["scans"].as_array().unwrap())
                .filter(|c| c["forced"] == false)
                .map(|c| c["world_records"].as_array().unwrap().len())
                .sum::<usize>(),
            1527
        );
        assert_eq!(rows.iter().filter(|r| r["error"] == false).count(), 9);
        assert_eq!(
            rows.iter()
                .flat_map(|r| r["scans"].as_array().unwrap())
                .filter(|c| c["forced"] == false && c["error"] == "")
                .count(),
            108
        );
        for (i, row) in rows.iter().enumerate() {
            let chunks: Vec<_> = row["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    let hex = c["hex"].as_str().unwrap();
                    let data: Vec<_> = (0..hex.len())
                        .step_by(2)
                        .map(|k| u8::from_str_radix(&hex[k..k + 2], 16).unwrap())
                        .collect();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: c["index"].as_i64().unwrap() as i32,
                            chunk_type: 2,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let actual = detect_film_i0_layout(&chunks);
            let mut buffers: Vec<_> = chunks.iter().map(|c| c.data.clone()).collect();
            buffers.insert(0, vec![41, 0, 0, 0, 27, 0, 0, 0]);
            let mut metadata: Vec<_> = chunks
                .iter()
                .map(|c| super::super::FilmSourceMetadata {
                    index: i64::from(c.metadata.index),
                    chunk_type: 2,
                    start_ms: 0,
                })
                .collect();
            metadata.insert(
                0,
                super::super::FilmSourceMetadata {
                    index: 0,
                    chunk_type: 1,
                    start_ms: 0,
                },
            );
            let source = super::super::FilmSource::load(&buffers, &metadata).unwrap();
            assert_eq!(
                detect_source_i0_layout(&source),
                actual,
                "source detection {i}"
            );
            for c in row["bands"].as_array().unwrap() {
                let selection: Vec<i64> = serde_json::from_value(c["chunks"].clone()).unwrap();
                let bounds = source_biped_slot_band(&source, &selection);
                let slots: Vec<u32> = bounds
                    .map(|[lo, hi]| (lo..=hi).collect())
                    .unwrap_or_default();
                assert_eq!(
                    serde_json::json!(slots),
                    c["slots"],
                    "band {i} {selection:?}"
                );
            }
            for c in row["scans"].as_array().unwrap() {
                use super::super::*;
                let selection: Vec<i64> = serde_json::from_value(c["chunks"].clone()).unwrap();
                let forced = I0Layout {
                    gate_bits: 5,
                    axis_widths: [19, 18, 20],
                    region: 0,
                };
                let layout = c["forced"].as_bool().unwrap().then_some(&forced);
                let options = BipedScanOptions {
                    isolation_gap_us: 0,
                    drop_saturated: false,
                    ..BipedScanOptions::native_defaults()
                };
                let got = if c["explicit_band"].as_bool().unwrap() {
                    scan_source_quantized_position_report_with_layout(
                        &source,
                        &selection,
                        &std::collections::BTreeSet::from([512, 513]),
                        layout,
                        &options,
                        None,
                    )
                } else {
                    scan_source_quantized_positions(&source, &selection, layout, &options, None)
                };
                let world_options = SourceWorldScanOptions {
                    scan: BipedScanOptions {
                        max_speed: c["world_speed"].as_f64().unwrap(),
                        ..options.clone()
                    },
                    dynamic_forward_level: None,
                    teleport_exemptions: serde_json::from_value(c["world_exemptions"].clone())
                        .unwrap(),
                };
                let explicit_band = std::collections::BTreeSet::from([512, 513]);
                let world = scan_source_world_positions(
                    &source,
                    &selection,
                    c["explicit_band"]
                        .as_bool()
                        .unwrap()
                        .then_some(&explicit_band),
                    layout,
                    [[-100.0, -500.0, 20.0], [300.0, 700.0, 700.0]],
                    &world_options,
                );
                let error = c["error"].as_str().unwrap();
                let context = NativeFilmContext::new(Some(&source));
                let contextual = scan_context_world_positions(
                    &context,
                    &selection,
                    c["explicit_band"]
                        .as_bool()
                        .unwrap()
                        .then_some(&explicit_band),
                    layout,
                    [[-100.0, -500.0, 20.0], [300.0, 700.0, 700.0]],
                    &world_options,
                );
                match (&world, &contextual) {
                    (Ok(a), Ok(b)) => assert_eq!(a, b, "context output {i}"),
                    (Err(a), Err(b)) => {
                        assert_eq!(a.to_string(), b.to_string(), "context refusal {i}")
                    }
                    _ => panic!("context changed outcome {i}: {contextual:?}"),
                }
                if !error.is_empty() {
                    assert!(
                        matches!(got, Err(DecodeError::Missing(s)) if s == error),
                        "scan error {i} {c}"
                    );
                    assert!(
                        matches!(world, Err(DecodeError::Missing(s)) if s == error),
                        "world scan error {i} {c}"
                    );
                    continue;
                }
                let world = world.unwrap();
                let expected_world = c["world_records"].as_array().unwrap();
                assert_eq!(
                    world.positions.len(),
                    expected_world.len(),
                    "world scan count {i} {c}"
                );
                assert!(world.masks.is_empty());
                for (a, e) in world.positions.iter().zip(expected_world) {
                    assert_eq!(a.position.chunk_number, e["Chunk"]);
                    assert_eq!(a.position.packet_index, e["PacketIndex"]);
                    assert_eq!(a.position.source.timestamp_us, e["TimestampUS"]);
                    assert_eq!(a.position.record.slot, e["Slot"]);
                    assert_eq!(serde_json::json!(a.position.record.quantized), e["Q"]);
                    assert_eq!(e["HasWorld"], true);
                    for (axis, field) in ["X", "Y", "Z"].into_iter().enumerate() {
                        assert_eq!(
                            a.world[axis].to_bits(),
                            (e[field].as_f64().unwrap() as f32).to_bits(),
                            "automatic world {i}"
                        );
                    }
                }
                let got = got.unwrap();
                assert!(got.masks.is_empty());
                let expected = c["records"].as_array().unwrap();
                assert_eq!(got.positions.len(), expected.len(), "scan count {i} {c}");
                for (a, e) in got.positions.iter().zip(expected) {
                    assert_eq!(a.chunk_number, e["Chunk"]);
                    assert_eq!(a.packet_index, e["PacketIndex"]);
                    assert_eq!(a.source.timestamp_us, e["TimestampUS"]);
                    assert_eq!(a.record.slot, e["Slot"]);
                    assert_eq!(serde_json::json!(a.record.quantized), e["Q"]);
                    assert_eq!(e["HasWorld"], false);
                }
            }
            let expected = I0LayoutReport {
                records: row["records"].as_u64().unwrap() as usize,
                pairs: row["pairs"].as_u64().unwrap() as usize,
                flip_rate: serde_json::from_value(row["rates"].clone()).unwrap(),
                boundaries: serde_json::from_value(row["boundaries"].clone()).unwrap(),
                index_bit_ones: row["ones"].as_u64().unwrap() as usize,
            };
            assert_eq!(actual.report, expected, "report {i}");
            assert_eq!(
                actual.refusal.is_some(),
                row["error"].as_bool().unwrap(),
                "error {i}"
            );
            if let Some(layout) = actual.layout {
                assert_eq!(
                    layout,
                    serde_json::from_value(row["layout"].clone()).unwrap(),
                    "layout {i}"
                );
            }
        }
    }
    #[test]
    #[ignore = "requires six downloaded v41 films"]
    fn local_i0_layout_corpus() {
        use crate::clients::hi::models::FilmChunk;
        use std::{fs, path::Path};
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/i0-layout-corpus-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        for row in rows {
            let folder = row["folder"].as_str().unwrap();
            let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(folder);
            let meta: serde_json::Value =
                serde_json::from_slice(&fs::read(dir.join("film.json")).unwrap()).unwrap();
            let chunks: Vec<_> = meta["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|c| c["index"].as_i64().unwrap() <= 7)
                .map(|c| FilmChunkData {
                    metadata: FilmChunk {
                        index: c["index"].as_i64().unwrap() as i32,
                        chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                        start_time_offset_ms: 0,
                        duration_ms: 0,
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data: fs::read(dir.join(c["file"].as_str().unwrap())).unwrap(),
                })
                .collect();
            let result = detect_film_i0_layout(&chunks);
            let expected = I0LayoutReport {
                records: row["records"].as_u64().unwrap() as usize,
                pairs: row["pairs"].as_u64().unwrap() as usize,
                flip_rate: serde_json::from_value(row["rates"].clone()).unwrap(),
                boundaries: serde_json::from_value(row["boundaries"].clone()).unwrap(),
                index_bit_ones: row["ones"].as_u64().unwrap() as usize,
            };
            assert_eq!(result.report, expected, "report {folder}");
            assert_eq!(
                result.refusal.is_some(),
                row["error"].as_bool().unwrap(),
                "refusal {folder}"
            );
            if let Some(layout) = result.layout {
                assert_eq!(
                    serde_json::to_value(layout).unwrap(),
                    row["layout"],
                    "layout {folder}"
                );
            }
            eprintln!("i0 layout matches {folder}");
        }
    }
}
