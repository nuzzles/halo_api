//! Raw typed managed-property observations (ti=13), retained before interpretation.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManagedPropertyRead {
    pub slot: u32,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    /// Scalar is zero; per-player is one. This is a channel, not a player identity.
    pub field: i64,
    pub film_index: i64,
    pub tag: i64,
    pub value: u64,
    pub has_value: bool,
    pub chained: bool,
}
use super::fire_events::{native_chunk_packets, native_chunk_prefix};
use super::navpoint_radial_scan::{header_at, indices, read};
use super::*;
use crate::clients::hi::models::FilmChunkData;
use std::collections::BTreeSet;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManagedPropertyScan {
    /// Complete attempted reads; older exports without this trace cannot prove absence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attempts: Vec<ManagedPropertyAttempt>,
    pub reads: Vec<ManagedPropertyRead>,
    pub slots: usize,
    pub records: usize,
    pub walked: usize,
    pub broken: usize,
    pub chained: usize,
}
/// A decoded scan attempt, distinct from an accepted property publication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedPropertyAttempt {
    /// Unavailable when the caller supplied only a payload and timestamp.
    pub source: Option<FilmPacket>,
    pub packet_index: Option<usize>,
    pub timestamp_us: u64,
    pub record_start_bit: usize,
    pub slot: u32,
    pub component_index: usize,
    pub status: Option<bool>,
    pub in_bounds: bool,
    pub component: DecodedComponent,
}
impl ManagedPropertyScan {
    /// A failed later component retains earlier successful reads. Only complete
    /// record walks can give those reads the independent chaining witness.
    pub fn scan_delta(
        &mut self,
        pay: &[u8],
        band: &BTreeSet<u32>,
        timestamp_us: u64,
        arch: &FilmArchetype,
        encoding: &FrameEncoding,
    ) {
        self.scan_delta_at(pay, band, (timestamp_us, None), arch, encoding);
    }
    fn scan_delta_at(
        &mut self,
        pay: &[u8],
        band: &BTreeSet<u32>,
        (timestamp_us, source): (u64, Option<(FilmPacket, usize)>),
        arch: &FilmArchetype,
        encoding: &FrameEncoding,
    ) {
        self.scan_delta_with(
            pay,
            band,
            (timestamp_us, source),
            arch,
            |at, name, level| {
                Ok::<_, std::convert::Infallible>(consume_component_at(
                    pay, at, name, 13, level, encoding,
                ))
            },
        )
        .unwrap();
    }
    fn scan_delta_with<E>(
        &mut self,
        pay: &[u8],
        band: &BTreeSet<u32>,
        (timestamp_us, source): (u64, Option<(FilmPacket, usize)>),
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
            let after = start + 21 + 6 * count;
            self.records += 1;
            let first = self.reads.len();
            let mut at = after;
            let mut done = true;
            for id in ids {
                let Some(name) = arch.components.get(id).filter(|s| !s.is_empty()) else {
                    done = false;
                    break;
                };
                if at > total {
                    done = false;
                    break;
                }
                let (status, c) = consume(at, name, arch.levels.get(id).copied().unwrap_or(0))?;
                self.attempts.push(ManagedPropertyAttempt {
                    source: source.map(|(p, _)| p),
                    packet_index: source.map(|(_, index)| index),
                    timestamp_us,
                    record_start_bit: start,
                    slot,
                    component_index: id,
                    status,
                    in_bounds: c.end_bit >= 0 && c.end_bit <= total as i64,
                    component: c.clone(),
                });
                if status != Some(true) || c.end_bit < 0 || c.end_bit > total as i64 {
                    done = false;
                    break;
                }
                at = usize::try_from(c.end_bit).expect("in-bounds component endpoint");
                for observation in &c.diagnostics.component_observations {
                    if let FilmComponentObservation::ManagedProperty { field, values } = observation
                        && let Some(&tag) = values.first()
                    {
                        self.reads.push(ManagedPropertyRead {
                            slot,
                            timestamp_us,
                            field: match field {
                                NativeManagedPropertyField::Scalar => 0,
                                NativeManagedPropertyField::PerPlayer => 1,
                            },
                            film_index: if (2..34).contains(&id) {
                                id as i64 - 2
                            } else {
                                -1
                            },
                            tag: tag as i64,
                            value: values.get(1).copied().unwrap_or(0),
                            has_value: values.len() > 1,
                            chained: false,
                        });
                    }
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
}
pub fn scan_managed_properties(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
) -> Result<ManagedPropertyScan, DecodeError> {
    let (scan, error) = scan_managed_properties_with_diagnostics(chunks, registry, encoding);
    error.map_or(Ok(scan), Err)
}

/// Preserve the native slot count alongside subsequent setup failures.
pub fn scan_managed_properties_with_diagnostics(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
) -> (ManagedPropertyScan, Option<DecodeError>) {
    let mut out = ManagedPropertyScan::default();
    let error = (|| -> Result<(), DecodeError> {
        if registry.major_version != 41 {
            return Err(DecodeError::UnsupportedVersion(
                registry.major_version as i32,
            ));
        }
        let prefix = native_chunk_prefix(chunks)?;
        let kf = scan_world_object_keyframes(chunks, 13);
        let band: BTreeSet<_> = kf
            .seen_us
            .iter()
            .map(|s| s.slot)
            .filter(|s| kf.band.contains(s))
            .collect();
        if band.is_empty() {
            return Err(DecodeError::Missing("observed managed-property slots"));
        }
        out.slots = band.len();
        let arch = registry
            .archetype(13)
            .ok_or(DecodeError::Missing("managed-property archetype 13"))?;
        for c in prefix {
            for (index, p) in native_chunk_packets(c)
                .into_iter()
                .enumerate()
                .filter(|(_, p)| p.packet_type == 0)
            {
                out.scan_delta_at(
                    &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                    &band,
                    (p.timestamp_us, Some((p, index))),
                    arch,
                    encoding,
                );
            }
        }
        Ok(())
    })()
    .err();
    (out, error)
}
/// Loaded scan admission errors retain any already established slot count.
#[derive(Debug, thiserror::Error)]
pub enum ContextManagedPropertyError {
    #[error("aucun chunk de donnees dans le film")]
    NoChunks,
    #[error("aucun slot d'archetype ti=13 dans les keyframes du film")]
    NoSlots,
    #[error("archetype ti=13 absent du registre")]
    NoArchetype,
    #[error(transparent)]
    Registry(#[from] NativeContextRegistryError),
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
    #[error(transparent)]
    Reader(#[from] NativeReaderProfileError),
}
/// Scan native loaded ti13 records with a private observer and the full profile.
/// Component diagnostics retain publications even when the enclosing read fails;
/// only successful in-bounds components contribute to the published read list.
pub fn scan_context_managed_properties(
    context: &NativeFilmContext<'_>,
) -> (ManagedPropertyScan, Option<ContextManagedPropertyError>) {
    let mut out = ManagedPropertyScan::default();
    let result = (|| -> Result<(), ContextManagedPropertyError> {
        let chunks = context.chunk_numbers();
        if chunks.is_empty() {
            return Err(ContextManagedPropertyError::NoChunks);
        }
        let census = scan_source_world_object_keyframes(context.source(), &[13]);
        let kf = &census.archetypes[&13];
        let band: BTreeSet<_> = kf
            .seen_us
            .iter()
            .map(|life| life.slot)
            .filter(|slot| kf.band.contains(slot))
            .collect();
        if band.is_empty() {
            return Err(ContextManagedPropertyError::NoSlots);
        }
        out.slots = band.len();
        let registry = context.registry().map_err(|e| *e)?;
        let arch = registry
            .registry
            .archetype(13)
            .ok_or(ContextManagedPropertyError::NoArchetype)?;
        let observer = NativeFilmObserver::default();
        // Typed publications are harvested from each retained component's diagnostics.
        // Never forward hooks from the caller's observer into this local scan.
        observer.set_hook(
            NativeHookKind::ManagedProperty,
            Some(std::sync::Arc::new(|_| {})),
        );
        let reader_context = NativeReaderContext {
            profile: context.scan_profile()?,
            observer: Some(observer),
        };
        for &chunk in chunks {
            let Some((data, packets)) = context.chunk_at(chunk) else {
                continue;
            };
            for (index, &packet) in packets.iter().enumerate() {
                if packet.packet_type != 0 {
                    continue;
                }
                let pay = &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
                out.scan_delta_with(
                    pay,
                    &band,
                    (packet.timestamp_us, Some((packet, index))),
                    arch,
                    |at, name, level| {
                        let mut reader =
                            NativeFilmReader::with_context(pay, reader_context.clone());
                        reader.set_native_bit_position(at as i64);
                        reader.read_component(name, level, 13)
                    },
                )?;
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
    fn native_managed_setup_preserves_partial_slots() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/managed-setup-v41.json.zlib")[..])
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
                archetypes: (0..14)
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
                registry.archetypes.truncate(13);
            }
            let (scan, error) =
                scan_managed_properties_with_diagnostics(&chunks, &registry, &encoding);
            let kind = match error.as_ref() {
                None => "",
                Some(DecodeError::Missing("readable film chunks")) => "source",
                Some(DecodeError::Missing("managed-property archetype 13")) => "archetype",
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
            let restored: ManagedPropertyScan =
                serde_json::from_value(serde_json::to_value(&scan).unwrap()).unwrap();
            assert_eq!(restored, scan);
        }
    }

    #[test]
    fn native_managed_property_delta_scan() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/managed-property-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        let encoding = FrameEncoding {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: None,
            native_id_low_bits: None,
            component_widths: Default::default(),
            new_record: Default::default(),
            position_capture: None,
            ids: RecordIdLayout {
                low_bits: 13,
                base: 0,
            },
            mpp_widths: [9, 5],
            position: None,
            extra_fields: false,
            corruption_check: false,
        };
        let band = BTreeSet::from([512, 514]);
        assert_eq!(rows.len(), 1024);
        for (i, row) in rows.iter().enumerate() {
            let mut encoding = encoding.clone();
            encoding.keyframe_simulation_complete = row["simulation_complete"].as_bool();
            if row["simulation_complete"].is_boolean() {
                encoding.position = Some(
                    FilmMapBounds {
                        module: String::new(),
                        min: [0.0; 3],
                        max: [1.0; 3],
                        axis_widths: [22; 3],
                        region: 0,
                        region_index_bits: 1,
                    }
                    .position_encoding(),
                );
            }
            let arch = FilmArchetype {
                index: 13,
                components: serde_json::from_value(row["names"].clone()).unwrap(),
                levels: vec![0; 36],
            };
            let mut scan = ManagedPropertyScan {
                slots: 2,
                ..Default::default()
            };
            for p in row["inputs"].as_array().unwrap() {
                let hex = p["hex"].as_str().unwrap();
                let bytes: Vec<_> = (0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                    .collect();
                scan.scan_delta(
                    &bytes,
                    &band,
                    p["timestamp_us"].as_u64().unwrap(),
                    &arch,
                    &encoding,
                );
            }
            let mut published = scan.clone();
            published.attempts.clear();
            assert_eq!(
                published,
                serde_json::from_value(row["output"].clone()).unwrap(),
                "scan {i}"
            );
            let mut emissions = Vec::new();
            for a in &scan.attempts {
                assert!(a.source.is_none());
                assert!(a.packet_index.is_none());
                assert!(a.component.start_bit >= a.record_start_bit as i64);
                for o in &a.component.diagnostics.component_observations {
                    if let FilmComponentObservation::ManagedProperty { field, values } = o {
                        let field = match field {
                            NativeManagedPropertyField::Scalar => 0,
                            NativeManagedPropertyField::PerPlayer => 1,
                        };
                        emissions.push(serde_json::json!({"timestamp_us":a.timestamp_us,"field":field,"values":values}));
                    }
                }
            }
            assert_eq!(
                serde_json::json!(emissions),
                row["emissions"],
                "all callbacks {i}"
            );
            assert_eq!(
                serde_json::from_value::<ManagedPropertyScan>(serde_json::to_value(&scan).unwrap())
                    .unwrap(),
                scan
            );
        }
    }
}
