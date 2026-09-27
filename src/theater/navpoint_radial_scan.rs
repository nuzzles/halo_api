//! Radial-progress observations from native delta anchors and full-state keyframes.
use super::bits::{Bits, Cursor};
use super::fire_events::{native_chunk_packets, native_chunk_prefix};
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
const MAX_READS: usize = 3_000_000;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NavpointRadialScan {
    /// Ordered attempts, including reads excluded by publication gates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attempts: Vec<NavpointScanAttempt>,
    pub reads: Vec<NavpointRadialRead>,
    pub slots_observed: usize,
    pub slots_band: usize,
    pub key_census: usize,
    pub records: usize,
    pub walked: usize,
    pub broken: usize,
    pub chained: usize,
    pub key_records: usize,
    pub key_walked: usize,
    pub key_broken: usize,
    pub key_chained: usize,
    pub key_closed: usize,
    pub key_bounded: usize,
    pub blocked: BTreeMap<i64, usize>,
    pub packets_no_clock: usize,
    pub truncated: bool,
}
/// Source evidence from a candidate walk, not accepted radial progress.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavpointScanAttempt {
    pub source: Option<FilmPacket>,
    pub packet_index: Option<usize>,
    /// Native chunk-clock projection; source.timestamp_us retains the wire clock.
    pub time_ms: i32,
    pub record_start_bit: usize,
    pub slot: u32,
    pub data: NavpointAttemptData,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NavpointAttemptData {
    Delta {
        component_index: usize,
        status: Option<bool>,
        in_bounds: bool,
        component: DecodedComponent,
    },
    Keyframe {
        record: Option<KeyframeRecord>,
    },
}
pub(super) fn read(pay: &[u8], p: usize, n: usize) -> u64 {
    Cursor::new_padded(pay, p).read(n).unwrap_or(0)
}
pub(super) fn indices(pay: &[u8], at: usize, count: usize) -> Option<Vec<usize>> {
    let mut out = Vec::new();
    for k in 0..count {
        let index = read(pay, at + 6 * k, 6) as usize;
        if out.last().is_some_and(|&v| index <= v) {
            return None;
        }
        out.push(index);
    }
    Some(out)
}
pub(super) fn header_at(pay: &[u8], at: usize) -> bool {
    let total = pay.len() * 8;
    if at + 27 > total || read(pay, at, 1) != 1 || read(pay, at + 16, 2) != 0 {
        return false;
    }
    let count = read(pay, at + 18, 3) as usize;
    count > 0 && at + 21 + 6 * count <= total && indices(pay, at + 21, count).is_some()
}
impl NavpointRadialScan {
    pub fn keyframe_closure_counters(&self) -> [(&'static str, usize); 2] {
        [
            ("filmdec_keyframe_ti12_closed", self.key_closed),
            ("filmdec_keyframe_ti12_total", self.key_bounded),
        ]
    }
    fn chunk(
        &mut self,
        data: &[u8],
        packets: &[FilmPacket],
        band: &BTreeSet<u32>,
        start: Option<i64>,
        registry: &FilmRegistry,
        encoding: &FrameEncoding,
    ) {
        let base = packets
            .iter()
            .find(|p| p.packet_type == 0)
            .map(|p| p.timestamp_us);
        for (index, p) in packets.iter().enumerate() {
            let (Some(base), Some(start)) = (base, start) else {
                self.packets_no_clock += 1;
                continue;
            };
            let ms =
                start.wrapping_add((p.timestamp_us as i64).wrapping_sub(base as i64) / 1000) as i32;
            let pay = &data[p.payload_offset..p.payload_offset + p.payload_size];
            match p.packet_type {
                0 => self.delta(
                    pay,
                    band,
                    (ms, Some((*p, index))),
                    &registry.archetypes[12],
                    encoding,
                ),
                2 => self.keyframe(pay, (ms, Some((*p, index))), registry, encoding),
                _ => {}
            }
        }
    }
    fn add(&mut self, r: NavpointRadialRead) {
        if self.reads.len() >= MAX_READS {
            self.truncated = true;
        } else {
            self.reads.push(r);
        }
    }
    fn delta(
        &mut self,
        pay: &[u8],
        band: &BTreeSet<u32>,
        (time_ms, source): (i32, Option<(FilmPacket, usize)>),
        arch: &FilmArchetype,
        encoding: &FrameEncoding,
    ) {
        self.delta_with(pay, band, (time_ms, source), arch, |at, name, level| {
            Ok::<_, std::convert::Infallible>(consume_component_at(
                pay, at, name, 12, level, encoding,
            ))
        })
        .unwrap();
    }
    fn delta_with<E>(
        &mut self,
        pay: &[u8],
        band: &BTreeSet<u32>,
        (time_ms, source): (i32, Option<(FilmPacket, usize)>),
        arch: &FilmArchetype,
        mut consume: impl FnMut(usize, &str, u32) -> Result<(Option<bool>, DecodedComponent), E>,
    ) -> Result<(), E> {
        let total = pay.len() * 8;
        let Some(limit) = total.checked_sub(27) else {
            return Ok(());
        };
        let mut p = 0;
        while p <= limit {
            let start = p;
            p += 1;
            if read(pay, start, 1) != 1 || read(pay, start + 16, 2) != 0 {
                continue;
            }
            let slot = read(pay, start + 1, 13) as u32;
            if !band.contains(&slot) {
                continue;
            }
            let count = read(pay, start + 18, 3) as usize;
            if count == 0 {
                continue;
            }
            let Some(ids) = indices(pay, start + 21, count) else {
                continue;
            };
            if ids.iter().any(|&i| i >= arch.components.len()) {
                continue;
            }
            let after = start + 21 + 6 * count;
            self.records += 1;
            let first = self.reads.len();
            let mut at = after;
            let mut done = true;
            for id in ids {
                let name = &arch.components[id];
                if name.is_empty() || at > total {
                    *self.blocked.entry(id as i64).or_default() += 1;
                    done = false;
                    break;
                }
                // Direct native dispatch inherits its simulation completion gate,
                // but does not apply traversal calibration/stub overrides.
                let (status, c) = consume(at, name, arch.levels.get(id).copied().unwrap_or(0))?;
                self.attempts.push(NavpointScanAttempt {
                    source: source.map(|(p, _)| p),
                    packet_index: source.map(|(_, i)| i),
                    time_ms,
                    record_start_bit: start,
                    slot,
                    data: NavpointAttemptData::Delta {
                        component_index: id,
                        status,
                        in_bounds: c.end_bit >= 0 && c.end_bit <= total as i64,
                        component: c.clone(),
                    },
                });
                if status != Some(true) || c.end_bit < 0 || c.end_bit > total as i64 {
                    *self.blocked.entry(id as i64).or_default() += 1;
                    done = false;
                    break;
                }
                at = usize::try_from(c.end_bit).expect("in-bounds component endpoint");
                if name == "managed-navpoint-radial-progress"
                    && let Some(f) = c.fields.iter().find(|f| f.name == "progress")
                {
                    self.add(NavpointRadialRead {
                        slot,
                        time_ms,
                        q: f.raw as u8,
                        chained: false,
                    });
                }
            }
            if done {
                self.walked += 1;
                if header_at(pay, at) {
                    self.chained += 1;
                    for r in &mut self.reads[first..] {
                        r.chained = true;
                    }
                }
            } else {
                self.broken += 1;
            }
            p = after + 1;
        }
        Ok(())
    }
    fn keyframe(
        &mut self,
        pay: &[u8],
        (time_ms, source): (i32, Option<(FilmPacket, usize)>),
        registry: &FilmRegistry,
        encoding: &FrameEncoding,
    ) {
        self.keyframe_with(pay, (time_ms, source), |at| {
            Ok::<_, std::convert::Infallible>(super::components::decode_native_keyframe_record(
                pay, at, registry, encoding,
            ))
        })
        .unwrap();
    }
    fn keyframe_with<E>(
        &mut self,
        pay: &[u8],
        (time_ms, source): (i32, Option<(FilmPacket, usize)>),
        mut consume: impl FnMut(usize) -> Result<Option<KeyframeRecord>, E>,
    ) -> Result<(), E> {
        let mut anchors = recover_keyframe_anchors(pay);
        anchors.sort_by_key(|a| a.bit);
        for (i, a) in anchors.iter().enumerate() {
            if a.archetype != 12 {
                continue;
            }
            self.key_records += 1;
            let want = anchors.get(i + 1).map(|a| a.bit);
            if want.is_some() {
                self.key_bounded += 1;
            }
            let first = self.reads.len();
            let rec = consume(a.bit)?;
            self.attempts.push(NavpointScanAttempt {
                source: source.map(|(p, _)| p),
                packet_index: source.map(|(_, i)| i),
                time_ms,
                record_start_bit: a.bit,
                slot: a.id & 0x3fff_ffff,
                data: NavpointAttemptData::Keyframe {
                    record: rec.clone(),
                },
            });
            let Some(rec) = rec else {
                self.key_broken += 1;
                *self.blocked.entry(0).or_default() += 1;
                continue;
            };
            // The native hook appends during the component walk, even when that
            // walk later fails. Rollback keeps truncation sticky under the cap.
            for c in &rec.components {
                if c.name == "managed-navpoint-radial-progress"
                    && let Some(f) = rec
                        .fields
                        .iter()
                        .find(|f| f.name == "progress" && f.bit >= c.start_bit && f.bit < c.end_bit)
                {
                    self.add(NavpointRadialRead {
                        slot: a.id & 0x3fff_ffff,
                        time_ms,
                        q: f.raw as u8,
                        chained: false,
                    });
                }
            }
            if rec.stop != KeyframeStop::Complete
                || rec.end_bit < 0
                || rec.end_bit > (pay.len() * 8) as i64
            {
                self.key_broken += 1;
                let index = match rec.stop {
                    KeyframeStop::UnsupportedComponent { index, .. } => index as i64,
                    KeyframeStop::Complete => -1,
                    _ => 0,
                };
                *self.blocked.entry(index).or_default() += 1;
                self.reads.truncate(first);
                continue;
            }
            self.key_walked += 1;
            if Some(rec.end_bit) == want.map(|bit| bit as i64) {
                self.key_closed += 1;
            }
            if super::recovery::anchor(
                Bits(pay),
                usize::try_from(rec.end_bit).expect("in-bounds keyframe endpoint"),
                None,
            )
            .is_some()
            {
                self.key_chained += 1;
                for r in &mut self.reads[first..] {
                    r.chained = true;
                }
            }
        }
        Ok(())
    }
}
pub fn scan_navpoint_radial(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    chunk_start_ms: &BTreeMap<i32, i64>,
) -> Result<NavpointRadialScan, DecodeError> {
    let (scan, error) =
        scan_navpoint_radial_with_diagnostics(chunks, registry, encoding, chunk_start_ms);
    error.map_or(Ok(scan), Err)
}

/// Retain the native slot/keyframe census even when subsequent setup fails.
pub fn scan_navpoint_radial_with_diagnostics(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    chunk_start_ms: &BTreeMap<i32, i64>,
) -> (NavpointRadialScan, Option<DecodeError>) {
    let mut out = NavpointRadialScan::default();
    let error = (|| -> Result<(), DecodeError> {
        if registry.major_version != 41 {
            return Err(DecodeError::UnsupportedVersion(
                registry.major_version as i32,
            ));
        }
        let prefix = native_chunk_prefix(chunks)?;
        let kf = scan_world_object_keyframes(chunks, 12);
        let band: BTreeSet<_> = kf
            .seen_us
            .iter()
            .map(|s| s.slot)
            .filter(|s| kf.band.contains(s))
            .collect();
        out.slots_observed = band.len();
        out.slots_band = kf.band.len();
        out.key_census = kf.seen_us.len();
        if band.is_empty() {
            return Ok(());
        }
        if !encoding.valid() {
            return Err(DecodeError::Inconsistent(
                "invalid navpoint precision profile".into(),
            ));
        }
        registry
            .archetype(12)
            .ok_or(DecodeError::Missing("navpoint archetype 12"))?;
        for c in prefix {
            out.chunk(
                &c.data,
                &native_chunk_packets(c),
                &band,
                chunk_start_ms.get(&c.metadata.index).copied(),
                registry,
                encoding,
            );
        }
        Ok(())
    })()
    .err();
    (out, error)
}

#[derive(Debug, thiserror::Error)]
pub enum ContextNavpointRadialError {
    #[error("aucun chunk de donnees dans le film")]
    NoChunks,
    #[error("archetype ti=12 absent du registre")]
    NoArchetype,
    #[error(transparent)]
    Registry(#[from] NativeContextRegistryError),
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
    #[error(transparent)]
    Reader(#[from] NativeReaderProfileError),
}
/// Native loaded radial scan. Manifest time is a projection; source packets keep
/// wire timestamps. Empty observed bands succeed before registry/profile access.
pub fn scan_context_navpoint_radial(
    context: &NativeFilmContext<'_>,
    chunk_start_ms: &BTreeMap<i64, i64>,
) -> (NavpointRadialScan, Option<ContextNavpointRadialError>) {
    let mut out = NavpointRadialScan::default();
    let result = (|| -> Result<(), ContextNavpointRadialError> {
        let chunks = context.chunk_numbers();
        if chunks.is_empty() {
            return Err(ContextNavpointRadialError::NoChunks);
        }
        let census = scan_source_world_object_keyframes(context.source(), &[12]);
        let kf = &census.archetypes[&12];
        let band: BTreeSet<_> = kf
            .seen_us
            .iter()
            .map(|life| life.slot)
            .filter(|slot| kf.band.contains(slot))
            .collect();
        out.slots_observed = band.len();
        out.slots_band = kf.band.len();
        out.key_census = kf.seen_us.len();
        if band.is_empty() {
            return Ok(());
        }
        let registry = context.registry().map_err(|e| *e)?;
        let registry = &registry.registry;
        let arch = registry
            .archetype(12)
            .ok_or(ContextNavpointRadialError::NoArchetype)?;
        let observer = NativeFilmObserver::default();
        observer.set_hook(NativeHookKind::Navpoint, Some(std::sync::Arc::new(|_| {})));
        let reader_context = NativeReaderContext {
            profile: context.scan_profile()?,
            observer: Some(observer),
        };
        for &chunk in chunks {
            let Some((data, packets)) = context.chunk_at(chunk) else {
                continue;
            };
            let base = packets
                .iter()
                .find(|p| p.packet_type == 0)
                .map(|p| p.timestamp_us);
            let start = chunk_start_ms.get(&chunk).copied();
            for (index, &packet) in packets.iter().enumerate() {
                let (Some(base), Some(start)) = (base, start) else {
                    out.packets_no_clock += 1;
                    continue;
                };
                let ms = start
                    .wrapping_add((packet.timestamp_us as i64).wrapping_sub(base as i64) / 1000)
                    as i32;
                let pay = &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
                let source = (ms, Some((packet, index)));
                match packet.packet_type {
                    0 => out.delta_with(pay, &band, source, arch, |at, name, level| {
                        let mut reader =
                            NativeFilmReader::with_context(pay, reader_context.clone());
                        reader.set_native_bit_position(at as i64);
                        reader.read_component(name, level, 12)
                    })?,
                    2 => out.keyframe_with(pay, source, |at| {
                        NativeFrameConfig {
                            context: reader_context.clone(),
                            ..Default::default()
                        }
                        .read_keyframe_record(pay, at, registry)
                        .map(Some)
                    })?,
                    _ => {}
                }
            }
        }
        Ok(())
    })();
    (out, result.err())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_navpoint_setup_preserves_partial_census() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/navpoint-setup-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(rows.len(), 256);
        for (case, row) in rows.iter().enumerate() {
            let chunks: Vec<_> = row["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|input| {
                    let hex = input["hex"].as_str().unwrap();
                    let data: Vec<_> = (0..hex.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                        .collect();
                    let index = input["index"].as_i64().unwrap() as i32;
                    FilmChunkData {
                        metadata: crate::clients::hi::models::FilmChunk {
                            index,
                            chunk_type: if index < 1 { 1 } else { 2 },
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let mut encoding: FrameEncoding = serde_json::from_value(serde_json::json!({
                "ids": {"low_bits": 13, "base": 0}, "mpp_widths": [9, 5],
                "extra_fields": false, "corruption_check": false,
                "position": row["encoding"]
            }))
            .unwrap();
            encoding.corruption_check = false;
            let mut registry = FilmRegistry {
                archetypes: (0..13)
                    .map(|index| FilmArchetype {
                        index,
                        components: vec![],
                        levels: vec![],
                    })
                    .collect(),
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            if row["registryCase"].as_u64().unwrap() == 1 {
                registry.archetypes.truncate(12);
            }
            let (scan, error) = scan_navpoint_radial_with_diagnostics(
                &chunks,
                &registry,
                &encoding,
                &BTreeMap::new(),
            );
            let kind = match error.as_ref() {
                None => "",
                Some(DecodeError::Missing("readable film chunks")) => "source",
                Some(DecodeError::Missing("navpoint archetype 12")) => "archetype",
                other => panic!("unexpected case {case} error: {other:?}"),
            };
            assert_eq!(kind, row["errorKind"].as_str().unwrap(), "case {case}");
            let mut expected = row["scan"].clone();
            if expected["Reads"].is_null() {
                expected["Reads"] = serde_json::json!([]);
            }
            assert_eq!(
                serde_json::to_value(&scan).unwrap(),
                expected,
                "case {case}"
            );
            let restored: NavpointRadialScan =
                serde_json::from_value(serde_json::to_value(&scan).unwrap()).unwrap();
            assert_eq!(restored, scan);
        }
    }
    #[test]
    fn native_radial_packet_scan() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/navpoint-radial-scan-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.as_array().unwrap().len(), 1024);
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let names: Vec<String> = serde_json::from_value(r["names"].clone()).unwrap();
            let mut archetypes: Vec<_> = (0..13)
                .map(|index| FilmArchetype {
                    index,
                    components: Vec::new(),
                    levels: Vec::new(),
                })
                .collect();
            archetypes[12] = FilmArchetype {
                index: 12,
                components: names,
                levels: vec![0; 3],
            };
            let registry = FilmRegistry {
                archetypes,
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: r["simulation_complete"].as_bool(),
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: r["simulation_complete"].as_bool().map(|_| {
                    FilmMapBounds {
                        module: String::new(),
                        min: [0.0; 3],
                        max: [1.0; 3],
                        axis_widths: [22; 3],
                        region: 0,
                        region_index_bits: 1,
                    }
                    .position_encoding()
                }),
                extra_fields: false,
                corruption_check: r["check"].as_bool().unwrap(),
            };
            let mut scan = NavpointRadialScan::default();
            for input in r["inputs"].as_array().unwrap() {
                let hex = input["hex"].as_str().unwrap();
                let bytes: Vec<_> = (0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                    .collect();
                if input["chunk"].as_bool() == Some(true) {
                    let packets: Vec<_> = input["packets"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|p| FilmPacket {
                            chunk_index: 9,
                            packet_type: p["Type"].as_u64().unwrap() as u16,
                            byte_2: 0,
                            byte_3: 0,
                            payload_offset: p["Start"].as_u64().unwrap() as usize,
                            payload_size: p["Size"].as_u64().unwrap() as usize,
                            timestamp_us: p["TimestampUS"].as_u64().unwrap(),
                        })
                        .collect();
                    scan.chunk(
                        &bytes,
                        &packets,
                        &BTreeSet::from([512, 513]),
                        input["start"].as_i64(),
                        &registry,
                        &encoding,
                    );
                    continue;
                }
                let ms = input["ms"].as_i64().unwrap() as i32;
                if input["key"].as_bool().unwrap() {
                    scan.keyframe(&bytes, (ms, None), &registry, &encoding);
                } else {
                    scan.delta(
                        &bytes,
                        &BTreeSet::from([512, 513]),
                        (ms, None),
                        &registry.archetypes[12],
                        &encoding,
                    );
                }
            }
            for ((name, value), expected) in scan
                .keyframe_closure_counters()
                .into_iter()
                .zip(r["closure"].as_array().unwrap())
            {
                assert_eq!(name, expected["Name"].as_str().unwrap());
                assert_eq!(value as u64, expected["Value"].as_u64().unwrap());
            }
            let mut published = scan.clone();
            published.attempts.clear();
            assert_eq!(
                published,
                serde_json::from_value::<NavpointRadialScan>(r["output"].clone()).unwrap(),
                "radial scan {i}"
            );
            let mut emissions = Vec::new();
            for a in &scan.attempts {
                let (keyframe, diagnostics) = match &a.data {
                    NavpointAttemptData::Delta { component, .. } => (false, &component.diagnostics),
                    NavpointAttemptData::Keyframe {
                        record: Some(record),
                    } => (true, &record.diagnostics),
                    NavpointAttemptData::Keyframe { record: None } => continue,
                };
                for o in &diagnostics.component_observations {
                    if let FilmComponentObservation::Navpoint { field, values } = o {
                        let field = match field {
                            NativeNavpointField::RadialProgress => 0,
                            NativeNavpointField::ManualTimerInitial => 1,
                            NativeNavpointField::ManualTimerCurrent => 2,
                        };
                        emissions.push(
                            serde_json::json!({"keyframe":keyframe,"field":field,"values":values}),
                        );
                    }
                }
            }
            assert_eq!(
                serde_json::json!(emissions),
                r["emissions"],
                "all navpoint callbacks {i}"
            );
            assert_eq!(
                serde_json::from_value::<NavpointRadialScan>(serde_json::to_value(&scan).unwrap())
                    .unwrap(),
                scan
            );
        }
    }
}
