//! Loaded-source position quanta with native chunk selection and isolation filtering.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceQuantizedPosition {
    /// Requested native chunk number, distinct from source buffer position.
    pub chunk_number: i64,
    /// Ordinal among all packet types in the source chunk.
    pub packet_index: usize,
    pub source: FilmPacket,
    pub record: QuantizedBipedPositionRecord,
}

/// A native RecordMaskHook publication, retained before temporal filtering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceQuantizedMaskObservation {
    pub chunk_number: i64,
    pub packet_index: usize,
    pub source: FilmPacket,
    pub component_indices: Vec<u8>,
    /// Offset in the packet payload immediately after the position quanta.
    pub after_position_bit: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceQuantizedPositionReport {
    pub positions: Vec<SourceQuantizedPosition>,
    /// Native scan order, including observations of temporally isolated positions.
    pub masks: Vec<SourceQuantizedMaskObservation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceWorldPosition {
    pub position: SourceQuantizedPosition,
    pub world: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceWorldPositionReport {
    pub positions: Vec<SourceWorldPosition>,
    pub masks: Vec<SourceQuantizedMaskObservation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceWorldScanOptions {
    pub scan: BipedScanOptions,
    pub dynamic_forward_level: Option<u32>,
    /// Native exemption map. Callers supply sorted timestamp arrays; their order
    /// is preserved, including native binary-search behavior for unsorted arrays.
    pub teleport_exemptions: std::collections::BTreeMap<u32, Vec<u64>>,
}

/// Loaded-source world scan with independent wire layout and world bounds.
/// Bounds are [minimum, maximum]. Native midpoint dequantization uses the supplied
/// layout widths, not widths inferred from those bounds. Isolation precedes speed
/// filtering; native mask publications survive both. This explicit-layout API
/// leaves automatic setup to the separately exposed discovery functions.
pub fn scan_source_world_positions_for_band(
    source: &FilmSource,
    chunks: &[i64],
    band: &BTreeSet<u32>,
    layout: &I0Layout,
    bounds: [[f32; 3]; 2],
    options: &SourceWorldScanOptions,
) -> Result<SourceWorldPositionReport, DecodeError> {
    scan_source_world_positions_for_band_observed(
        source, chunks, band, layout, bounds, options, None,
    )
}

/// World-space loaded scan with live masks emitted before isolation/speed filters.
pub fn scan_source_world_positions_for_band_observed(
    source: &FilmSource,
    chunks: &[i64],
    band: &BTreeSet<u32>,
    layout: &I0Layout,
    bounds: [[f32; 3]; 2],
    options: &SourceWorldScanOptions,
    observer: Option<&super::NativeFilmObserver>,
) -> Result<SourceWorldPositionReport, DecodeError> {
    let report = scan_source_quantized_position_report_for_band_observed(
        source,
        chunks,
        band,
        layout,
        &options.scan,
        options.dynamic_forward_level,
        observer,
    )?;
    let positions: Vec<_> = report
        .positions
        .into_iter()
        .map(|position| {
            let world = std::array::from_fn(|axis| {
                let step = (f64::from(bounds[1][axis]) - f64::from(bounds[0][axis]))
                    / ((1u64 << layout.axis_widths[axis]) as f64);
                (f64::from(bounds[0][axis])
                    + step * (f64::from(position.record.quantized[axis]) + 0.5))
                    as f32
            });
            SourceWorldPosition { position, world }
        })
        .collect();
    let rejected: BTreeSet<_> = super::biped_scan::speed_rejected_indices(
        positions.iter().enumerate().map(|(i, p)| {
            (
                i,
                p.position.record.slot,
                p.position.source.timestamp_us,
                p.world,
            )
        }),
        options.scan.max_speed,
        &options.teleport_exemptions,
    )
    .into_iter()
    .collect();
    Ok(SourceWorldPositionReport {
        positions: positions
            .into_iter()
            .enumerate()
            .filter_map(|(i, p)| (!rejected.contains(&i)).then_some(p))
            .collect(),
        masks: report.masks,
    })
}

/// Loaded world scan with optional automatic band and layout discovery. A missing
/// band selects native biped keyframes; a supplied empty band is an error. Layout
/// detection uses the film prefix, independently of the requested scan chunks.
pub fn scan_source_world_positions(
    source: &FilmSource,
    chunks: &[i64],
    band: Option<&BTreeSet<u32>>,
    layout: Option<&I0Layout>,
    bounds: [[f32; 3]; 2],
    options: &SourceWorldScanOptions,
) -> Result<SourceWorldPositionReport, DecodeError> {
    let automatic = source.data_chunk_numbers();
    let chunks = if chunks.is_empty() {
        automatic.as_slice()
    } else {
        chunks
    };
    let discovered;
    let band = match band {
        Some(band) => band,
        None => {
            discovered = automatic_source_band(source, chunks)?;
            &discovered
        }
    };
    let layout = source_scan_layout(source, chunks, band, layout)?;
    scan_source_world_positions_for_band(source, chunks, band, &layout, bounds, options)
}

pub(super) fn automatic_source_band(
    source: &FilmSource,
    chunks: &[i64],
) -> Result<BTreeSet<u32>, DecodeError> {
    if chunks.is_empty() {
        return Err(DecodeError::Missing("film chunks"));
    }
    let Some([lo, hi]) = source_biped_slot_band(source, chunks) else {
        return Err(DecodeError::Missing("biped slots"));
    };
    // Only 13-bit slots can appear in a delta header. Keep an out-of-domain
    // band nonempty without materializing its potentially enormous interval.
    Ok(if lo > 8191 {
        BTreeSet::from([lo])
    } else {
        (lo..=hi.min(8191)).collect()
    })
}

pub(super) fn source_scan_layout(
    source: &FilmSource,
    chunks: &[i64],
    band: &BTreeSet<u32>,
    layout: Option<&I0Layout>,
) -> Result<I0Layout, DecodeError> {
    if chunks.is_empty() && source.data_chunk_numbers().is_empty() {
        return Err(DecodeError::Missing("film chunks"));
    }
    if band.is_empty() {
        return Err(DecodeError::Missing("slot band"));
    }
    if let Some(layout) = layout {
        return Ok(layout.clone());
    }
    let measured = detect_source_i0_layout(source);
    if measured.refusal.is_some() {
        return Err(DecodeError::Missing("i0 layout"));
    }
    measured.layout.ok_or(DecodeError::Missing("i0 layout"))
}

/// Native biped-only loaded scan with automatic band discovery and optional
/// layout detection. Detection always uses the film prefix, independently of the
/// scan selection. Use `detect_source_i0_layout` to inspect refusal measurements.
pub fn scan_source_quantized_positions(
    source: &FilmSource,
    chunks: &[i64],
    layout: Option<&I0Layout>,
    options: &BipedScanOptions,
    dynamic_forward_level: Option<u32>,
) -> Result<SourceQuantizedPositionReport, DecodeError> {
    let automatic = source.data_chunk_numbers();
    let chunks = if chunks.is_empty() {
        automatic.as_slice()
    } else {
        chunks
    };
    let band = automatic_source_band(source, chunks)?;
    scan_source_quantized_position_report_with_layout(
        source,
        chunks,
        &band,
        layout,
        options,
        dynamic_forward_level,
    )
}

/// Native explicit-band scan with an optional measured layout. Error precedence
/// is chunks, band, layout detection, readable chunks. A measured candidate is
/// accepted only when detection succeeds, never when its report is inconclusive.
pub fn scan_source_quantized_position_report_with_layout(
    source: &FilmSource,
    chunks: &[i64],
    band: &BTreeSet<u32>,
    layout: Option<&I0Layout>,
    options: &BipedScanOptions,
    dynamic_forward_level: Option<u32>,
) -> Result<SourceQuantizedPositionReport, DecodeError> {
    let layout = source_scan_layout(source, chunks, band, layout)?;
    scan_source_quantized_position_report_for_band(
        source,
        chunks,
        band,
        &layout,
        options,
        dynamic_forward_level,
    )
}

/// Native ScanBipedPositionsForBand with QuantaOnly and an explicit wire layout.
/// Empty selection uses the native contiguous chunk prefix. Explicit selections
/// preserve order and duplicates and skip unavailable chunks. Isolation follows
/// input order per slot with native wrapping time differences; speed is ignored
/// because quanta have no world-coordinate scale. No source buffers are copied.
pub fn scan_source_quantized_positions_for_band(
    source: &FilmSource,
    chunks: &[i64],
    band: &BTreeSet<u32>,
    layout: &I0Layout,
    options: &BipedScanOptions,
    dynamic_forward_level: Option<u32>,
) -> Result<Vec<SourceQuantizedPosition>, DecodeError> {
    Ok(scan_source_quantized_position_report_for_band(
        source,
        chunks,
        band,
        layout,
        options,
        dynamic_forward_level,
    )?
    .positions)
}

/// Scan position quanta and retain native mask publications before isolation.
/// Masks are emitted only when `capture_dirs` is enabled, after saturation rejection.
/// Chunk selection and filtering follow `scan_source_quantized_positions_for_band`.
pub fn scan_source_quantized_position_report_for_band(
    source: &FilmSource,
    chunks: &[i64],
    band: &BTreeSet<u32>,
    layout: &I0Layout,
    options: &BipedScanOptions,
    dynamic_forward_level: Option<u32>,
) -> Result<SourceQuantizedPositionReport, DecodeError> {
    scan_source_quantized_position_report_for_band_observed(
        source,
        chunks,
        band,
        layout,
        options,
        dynamic_forward_level,
        None,
    )
}

/// Loaded native scan with live mask hooks before isolation filtering. Payloads
/// remain borrowed from FilmSource, and duplicate chunk selections publish again.
pub fn scan_source_quantized_position_report_for_band_observed(
    source: &FilmSource,
    chunks: &[i64],
    band: &BTreeSet<u32>,
    layout: &I0Layout,
    options: &BipedScanOptions,
    dynamic_forward_level: Option<u32>,
    observer: Option<&super::NativeFilmObserver>,
) -> Result<SourceQuantizedPositionReport, DecodeError> {
    let automatic;
    let chunks = if chunks.is_empty() {
        automatic = source.data_chunk_numbers();
        automatic.as_slice()
    } else {
        chunks
    };
    if chunks.is_empty() {
        return Err(DecodeError::Missing("film chunks"));
    }
    if band.is_empty() {
        return Err(DecodeError::Missing("slot band"));
    }
    let mut out = Vec::new();
    let mut masks = Vec::new();
    let mut read = 0;
    for &number in chunks {
        let Some((data, packets)) = source.chunk_by_number(number) else {
            continue;
        };
        read += 1;
        for (packet_index, &packet) in packets.iter().enumerate() {
            if packet.packet_type != 0 {
                continue;
            }
            let payload = &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            let records = super::biped_scan::scan_quantized_band_observed(
                payload,
                band,
                layout,
                options,
                dynamic_forward_level,
                observer,
            )?;
            for record in records {
                if options.drop_saturated
                    && record
                        .quantized
                        .iter()
                        .zip(layout.axis_widths)
                        .any(|(&q, w)| q == 0 || u64::from(q) == (1u64 << w) - 1)
                {
                    continue;
                }
                if options.capture_dirs {
                    masks.push(SourceQuantizedMaskObservation {
                        chunk_number: number,
                        packet_index,
                        source: packet,
                        component_indices: record.component_indices.clone(),
                        after_position_bit: record.end_bit,
                    });
                }
                out.push(SourceQuantizedPosition {
                    chunk_number: number,
                    packet_index,
                    source: packet,
                    record,
                });
            }
        }
    }
    if read == 0 {
        return Err(DecodeError::Missing("readable film chunks"));
    }
    let isolated: BTreeSet<_> = super::biped_scan::isolated_position_indices(
        out.iter()
            .enumerate()
            .map(|(i, p)| (i, p.record.slot, p.source.timestamp_us)),
        options.isolation_threshold_us(),
    )
    .into_iter()
    .collect();
    let positions = out
        .into_iter()
        .enumerate()
        .filter_map(|(i, p)| (!isolated.contains(&i)).then_some(p))
        .collect();
    Ok(SourceQuantizedPositionReport { positions, masks })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_loaded_quantized_positions() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/record-mask-hook-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 256);
        assert_eq!(
            rows.iter()
                .flat_map(|r| r["source_cases"].as_array().unwrap())
                .map(|c| c["world_records"].as_array().unwrap().len())
                .sum::<usize>(),
            16914
        );
        assert_eq!(
            rows.iter()
                .flat_map(|r| r["source_cases"].as_array().unwrap())
                .map(|c| c["records"].as_array().unwrap().len())
                .sum::<usize>(),
            19484
        );
        assert!(
            rows.iter()
                .flat_map(|r| r["source_cases"].as_array().unwrap())
                .any(|c| c["masks"].as_array().unwrap().len()
                    > c["records"].as_array().unwrap().len())
        );
        for (i, row) in rows.iter().enumerate() {
            let c = &row["speed_case"];
            let input = c["input"].as_array().unwrap();
            let exemptions = serde_json::from_value(c["exemptions"].clone()).unwrap();
            let rejected: BTreeSet<_> = super::super::biped_scan::speed_rejected_indices(
                input.iter().enumerate().map(|(i, p)| {
                    (
                        i,
                        p["slot"].as_u64().unwrap() as u32,
                        p["time"].as_u64().unwrap(),
                        serde_json::from_value(p["world"].clone()).unwrap(),
                    )
                }),
                c["speed"].as_f64().unwrap(),
                &exemptions,
            )
            .into_iter()
            .collect();
            let accepted: Vec<_> = (0..input.len()).filter(|i| !rejected.contains(i)).collect();
            assert_eq!(
                serde_json::json!(accepted),
                c["accepted"],
                "speed boundaries {i}"
            );
            let hex = row["source_hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|k| u8::from_str_radix(&hex[k..k + 2], 16).unwrap())
                .collect();
            let numbers: Vec<i64> = serde_json::from_value(row["source_numbers"].clone()).unwrap();
            let metadata: Vec<_> = numbers
                .into_iter()
                .map(|index| FilmSourceMetadata {
                    index,
                    chunk_type: 2,
                    start_ms: 0,
                })
                .collect();
            let source = FilmSource::load(&[Vec::new(), bytes.clone(), bytes], &metadata).unwrap();
            let flags: std::collections::BTreeMap<u32, bool> =
                serde_json::from_value(row["source_slots"].clone()).unwrap();
            let band = flags
                .into_iter()
                .filter_map(|(s, p)| p.then_some(s))
                .collect();
            let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
            let layout = I0Layout {
                gate_bits: (4 + map.region_index_bits.max(1)) as i64,
                axis_widths: map.axis_widths.map(|w| w as u64),
                region: map.region,
            };
            let options = BipedScanOptions {
                capture_dirs: row["capture"].as_bool().unwrap(),
                drop_saturated: row["drop"].as_bool().unwrap(),
                isolation_gap_us: row["gap"].as_u64().unwrap(),
                max_speed: row["speed"].as_f64().unwrap(),
                ..BipedScanOptions::native_defaults()
            };
            for c in row["source_cases"].as_array().unwrap() {
                let chunks: Vec<i64> = serde_json::from_value(c["chunks"].clone()).unwrap();
                let publications = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
                let observer = super::super::NativeFilmObserver::default();
                observer.set_hook(
                    super::super::NativeHookKind::RecordMask,
                    Some({
                        let publications = publications.clone();
                        std::sync::Arc::new(move |publication| {
                            let super::super::NativeHookPublication::RecordMask {
                                indices,
                                payload,
                                after_i0,
                            } = publication
                            else {
                                panic!("wrong hook")
                            };
                            let hex: String = payload.iter().map(|b| format!("{b:02x}")).collect();
                            publications
                                .lock()
                                .unwrap()
                                .push(serde_json::json!({"mask":indices,"end":after_i0,"hex":hex}));
                        })
                    }),
                );
                let result = scan_source_quantized_position_report_for_band_observed(
                    &source,
                    &chunks,
                    &band,
                    &layout,
                    &options,
                    row["dynamic"].as_bool().unwrap().then_some(2),
                    Some(&observer),
                );
                assert_eq!(
                    serde_json::json!(*publications.lock().unwrap()),
                    c["masks"],
                    "live source masks"
                );
                let world_options = SourceWorldScanOptions {
                    scan: options.clone(),
                    dynamic_forward_level: row["dynamic"].as_bool().unwrap().then_some(2),
                    teleport_exemptions: serde_json::from_value(c["exemptions"].clone()).unwrap(),
                };
                publications.lock().unwrap().clear();
                let world = scan_source_world_positions_for_band_observed(
                    &source,
                    &chunks,
                    &band,
                    &layout,
                    [map.min, map.max],
                    &world_options,
                    Some(&observer),
                );
                assert_eq!(
                    serde_json::json!(*publications.lock().unwrap()),
                    c["world_masks"],
                    "live world masks"
                );
                let contexts = c["contexts"].as_array().unwrap();
                assert_eq!(contexts.len(), 2);
                for context in contexts {
                    assert_eq!(context["same_records"], true);
                    assert_eq!(context["same_error"], true);
                    assert_eq!(context["hook_count"], 30);
                    assert_eq!(context["masks"], c["masks"]);
                    let calls = context["calls"].as_object().unwrap();
                    assert!(calls.keys().all(|name| name == "RecordMaskHook"));
                    assert_eq!(
                        calls
                            .get("RecordMaskHook")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize,
                        c["masks"].as_array().unwrap().len()
                    );
                }
                let expected_error = c["error"].as_str().unwrap();
                if !expected_error.is_empty() {
                    let expected = match expected_error {
                        "chunks" => "film chunks",
                        "band" => "slot band",
                        _ => "readable film chunks",
                    };
                    assert!(
                        matches!(result,Err(DecodeError::Missing(s)) if s==expected),
                        "source error {i} {c}"
                    );
                    assert!(
                        matches!(world,Err(DecodeError::Missing(s)) if s==expected),
                        "world error {i} {c}"
                    );
                    continue;
                }
                let report = result.unwrap();
                let world = world.unwrap();
                assert_eq!(world.masks, report.masks);
                assert_eq!(c["world_masks"], c["masks"]);
                let expected_world = c["world_records"].as_array().unwrap();
                assert_eq!(
                    world.positions.len(),
                    expected_world.len(),
                    "world count {i} {chunks:?}"
                );
                for (a, e) in world.positions.iter().zip(expected_world) {
                    assert_eq!(a.position.chunk_number, e["Chunk"]);
                    assert_eq!(a.position.packet_index, e["PacketIndex"]);
                    assert_eq!(a.position.source.timestamp_us, e["TimestampUS"]);
                    assert_eq!(a.position.record.slot, e["Slot"]);
                    assert_eq!(serde_json::json!(a.position.record.quantized), e["Q"]);
                    assert_eq!(e["HasWorld"], true);
                    super::super::biped_capture::tests::assert_native(
                        &a.position.record.companions,
                        e,
                        e,
                    );
                    for (axis, field) in ["X", "Y", "Z"].into_iter().enumerate() {
                        assert_eq!(
                            a.world[axis].to_bits(),
                            (e[field].as_f64().unwrap() as f32).to_bits(),
                            "world coordinate {i}"
                        );
                    }
                }
                assert_eq!(
                    world,
                    serde_json::from_slice::<SourceWorldPositionReport>(
                        &serde_json::to_vec(&world).unwrap()
                    )
                    .unwrap()
                );
                let native_masks = c["masks"].as_array().unwrap();
                assert_eq!(report.masks.len(), native_masks.len());
                for (a, e) in report.masks.iter().zip(native_masks) {
                    assert_eq!(serde_json::json!(a.component_indices), e["mask"]);
                    assert_eq!(a.after_position_bit, e["end"]);
                    let payload = source.payload(&a.source).unwrap();
                    let hex: String = payload.iter().map(|b| format!("{b:02x}")).collect();
                    assert_eq!(hex, e["hex"]);
                    assert_eq!(
                        a.source.chunk_index as usize,
                        source.chunk_position(a.chunk_number).unwrap()
                    );
                    assert_eq!(
                        source.chunk_by_number(a.chunk_number).unwrap().1[a.packet_index],
                        a.source
                    );
                }
                assert_eq!(
                    report,
                    serde_json::from_slice::<SourceQuantizedPositionReport>(
                        &serde_json::to_vec(&report).unwrap()
                    )
                    .unwrap()
                );
                let out = report.positions;
                let expected = c["records"].as_array().unwrap();
                assert_eq!(out.len(), expected.len(), "count {i} {chunks:?}");
                for (a, e) in out.iter().zip(expected) {
                    assert_eq!(a.chunk_number, e["Chunk"]);
                    assert_eq!(a.packet_index, e["PacketIndex"]);
                    assert_eq!(a.source.timestamp_us, e["TimestampUS"]);
                    assert_eq!(a.record.slot, e["Slot"]);
                    assert_eq!(serde_json::json!(a.record.quantized), e["Q"]);
                    assert_eq!(e["HasWorld"], false);
                    super::super::biped_capture::tests::assert_native(&a.record.companions, e, e);
                    assert_eq!(
                        a.source.chunk_index as usize,
                        source.chunk_position(a.chunk_number).unwrap()
                    );
                    assert!(source.payload(&a.source).is_some());
                }
                assert_eq!(
                    out,
                    serde_json::from_slice::<Vec<SourceQuantizedPosition>>(
                        &serde_json::to_vec(&out).unwrap()
                    )
                    .unwrap()
                );
            }
        }
    }
}
