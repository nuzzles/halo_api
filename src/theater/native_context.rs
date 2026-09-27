//! Source-derived caches from the native FilmContext. This is the recording
//! parser's context, not the deferred resolved replay or playback model.
use super::{
    FilmPacket, FilmRegistryRead, FilmRegistryReadError, FilmSource, I0Layout, I0LayoutDetection,
    I0LayoutRefusal,
};
use std::cell::OnceCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NativeContextRegistryError {
    #[error("film has no registry chunk")]
    NoRegistryChunk,
    #[error(transparent)]
    Parse(#[from] FilmRegistryReadError),
}

/// Lazy source caches, scoped to one already-loaded film. Source buffers are
/// borrowed; no chunk bytes are copied. Registry mutation is explicit through
/// `registry_mut`, and subsequent accesses see the same cached registry.
///
/// This currently ports source/registry/band/layout caches and lazy resolved profiles.
/// Native scan-profile replacement and map construction are also represented.
/// Shared observer/reader construction and LegacyFilm-pass integration remain open.
#[derive(Debug)]
pub struct NativeFilmContext<'a> {
    source: Option<&'a FilmSource>,
    imposed: Option<I0Layout>,
    chunks: OnceCell<Vec<i64>>,
    registry: OnceCell<Result<FilmRegistryRead, NativeContextRegistryError>>,
    registry_identity: OnceCell<(u64, usize)>,
    biped_band: OnceCell<Option<[u32; 2]>>,
    layout: OnceCell<I0LayoutDetection>,
    profile: OnceCell<Result<super::NativeResolvedFilmProfile, super::NativeProfileResolveError>>,
    scan: super::NativeScanProfile,
    corruption: OnceCell<super::FilmCorruptionControl>,
    observer: OnceCell<super::NativeFilmObserver>,
}
impl<'a> NativeFilmContext<'a> {
    /// Native no-catalog construction: no parsing or source derivation occurs.
    pub fn new(source: Option<&'a FilmSource>) -> Self {
        Self::with_imposed_layout(source, None)
    }
    /// Native map constructor: resolve profile eagerly, select forced/catalog
    /// layout, and enable only the map-dependent simulation grammar switch.
    /// Installing the world's precision descriptor is a separate native action.
    pub fn for_map(
        source: Option<&'a FilmSource>,
        map: Option<&super::FilmMapBounds>,
        forced: Option<&I0Layout>,
    ) -> Result<Self, super::NativeProfileResolveError> {
        let catalog = map.map(super::FilmMapBounds::i0_layout);
        let imposed = super::resolve_imposed_i0_layout(forced, catalog.as_ref());
        let mut context = Self::with_imposed_layout(source, imposed.as_ref());
        context.scan.grammar.simulation_complete = imposed.is_some();
        let profile = super::resolve_native_film_profile(source, map)?;
        if profile.registry_present && !profile.issues.is_empty() {
            tracing::warn!(
                err = profile
                    .issues
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n"),
                format = profile.format_version.unwrap_or(0),
                build = profile.build(),
                "profil du film INCOMPLET — une cle ecrite dans le film est absente de la table de profil ; le film reste decode, les replis existants decident et se comptent"
            );
        }
        context
            .profile
            .set(Ok(profile))
            .expect("profile initialized once");
        Ok(context)
    }
    /// Install an already-selected layout, copying even invalid forced values.
    /// This is only the layout portion of native map-context construction, not
    /// its eager profile resolution, grammar switch or warning publication.
    pub fn with_imposed_layout(source: Option<&'a FilmSource>, imposed: Option<&I0Layout>) -> Self {
        Self {
            source,
            imposed: imposed.cloned(),
            chunks: OnceCell::new(),
            registry: OnceCell::new(),
            registry_identity: OnceCell::new(),
            biped_band: OnceCell::new(),
            layout: OnceCell::new(),
            profile: OnceCell::new(),
            scan: super::NativeScanProfile::default(),
            corruption: OnceCell::new(),
            observer: OnceCell::new(),
        }
    }
    /// Lazily resolve from original source bytes, then return a deep owned copy.
    /// Caller edits cannot change cached identity tables or map metadata.
    pub fn profile(
        &self,
    ) -> Result<super::NativeResolvedFilmProfile, super::NativeProfileResolveError> {
        self.profile
            .get_or_init(|| super::resolve_native_film_profile(self.source, None))
            .as_ref()
            .cloned()
            .map_err(|error| *error)
    }
    /// Cache the film declaration (or inherited fallback) independently of later
    /// scan-profile replacements. A declared false is not a missing declaration.
    pub fn corruption_control(
        &self,
    ) -> Result<super::FilmCorruptionControl, super::NativeProfileResolveError> {
        if let Some(control) = self.corruption.get() {
            return Ok(*control);
        }
        let profile = self.profile()?;
        let control = super::FilmCorruptionControl::from_recorded(
            profile.identity.as_ref().map(|id| id.corruption_checks),
            self.scan.grammar.corruption_check,
        );
        self.corruption
            .set(control)
            .expect("corruption initialized once");
        Ok(control)
    }
    pub fn scan_profile(
        &self,
    ) -> Result<super::NativeScanProfile, super::NativeProfileResolveError> {
        let mut profile = self.scan.clone();
        profile.grammar.corruption_check = self.corruption_control()?.enabled;
        Ok(profile)
    }
    /// Return the previous effective profile before replacing settings for future
    /// readers. Scalar copies are independent; allocated calibration maps alias.
    pub fn set_scan_profile(
        &mut self,
        profile: super::NativeScanProfile,
    ) -> Result<super::NativeScanProfile, super::NativeProfileResolveError> {
        let previous = self.scan_profile()?;
        self.scan = profile;
        Ok(previous)
    }
    pub fn set_mpp(&mut self, widths: super::FilmMppWidths) -> super::FilmMppWidths {
        std::mem::replace(&mut self.scan.mpp, widths)
    }
    pub fn set_world_precision(&mut self, precision: super::NativePrecisionDescriptor) {
        self.scan.set_world_precision(precision);
    }
    pub fn set_world_precision_from_layout(&mut self, layout: &I0Layout) -> bool {
        self.scan.set_world_precision_from_layout(layout)
    }
    /// Repeated calls share one observer. New contexts never share observers.
    pub fn observation(&self) -> super::NativeFilmObserver {
        self.observer
            .get_or_init(super::NativeFilmObserver::default)
            .clone()
    }
    pub fn reader_context(
        &self,
    ) -> Result<super::NativeReaderContext, super::NativeProfileResolveError> {
        Ok(super::NativeReaderContext {
            profile: self.scan_profile()?,
            observer: Some(self.observation()),
        })
    }
    pub fn reader<'b>(
        &self,
        data: &'b [u8],
    ) -> Result<super::NativeFilmReader<'b>, super::NativeProfileResolveError> {
        Ok(super::NativeFilmReader::with_context(
            data,
            self.reader_context()?,
        ))
    }
    /// Native CadreDeBalayage inherits settings but deliberately has no observer.
    pub fn scan_frame(&self) -> Result<super::NativeFrameConfig, super::NativeProfileResolveError> {
        Ok(super::NativeFrameConfig {
            context: super::NativeReaderContext {
                profile: self.scan_profile()?,
                observer: None,
            },
            ..Default::default()
        })
    }
    pub fn source(&self) -> Option<&'a FilmSource> {
        self.source
    }
    /// Cached native contiguous chunk prefix, borrowed without copying.
    pub fn chunk_numbers(&self) -> &[i64] {
        self.chunks.get_or_init(|| {
            self.source
                .map_or_else(Vec::new, FilmSource::data_chunk_numbers)
        })
    }
    /// Borrow cached source bytes/packet ranges. Packet chunk identities remain
    /// source positions, matching FilmSource; `number` is the requested file number.
    pub fn chunk_at(&self, number: i64) -> Option<(&'a [u8], &'a [FilmPacket])> {
        self.source?.chunk_by_number(number)
    }
    pub fn registry(&self) -> Result<&FilmRegistryRead, &NativeContextRegistryError> {
        self.registry
            .get_or_init(|| {
                let raw = self
                    .source
                    .and_then(FilmSource::registry_chunk)
                    .ok_or(NativeContextRegistryError::NoRegistryChunk)?;
                let read = super::parse_registry_chunk(raw)?;
                let identity = (
                    read.registry
                        .fingerprint()
                        .expect("parsed registry has levels"),
                    read.registry
                        .archetypes
                        .iter()
                        .map(|a| a.components.len())
                        .sum(),
                );
                self.registry_identity
                    .set(identity)
                    .expect("registry identity initialized once");
                Ok(read)
            })
            .as_ref()
    }
    /// Mutate the cached native registry rather than a detached clone. Borrowing
    /// rules prevent concurrent access while this mutable view is outstanding.
    pub fn registry_mut(&mut self) -> Result<&mut FilmRegistryRead, &NativeContextRegistryError> {
        let _ = self.registry();
        self.registry
            .get_mut()
            .expect("registry initialized")
            .as_mut()
            .map_err(|error| &*error)
    }
    /// Native registry fingerprint and named-slot count are parse-time snapshots;
    /// unlike FilmRegistry::fingerprint, they do not change after registry mutation.
    pub fn registry_identity(&self) -> Result<(u64, usize), &NativeContextRegistryError> {
        self.registry()?;
        Ok(*self
            .registry_identity
            .get()
            .expect("successful registry initialized identity"))
    }
    /// Native biped membership fills every slot between these inclusive bounds.
    /// No band is distinct from slot zero and does not invent player identities.
    pub fn biped_slot_band(&self) -> Option<[u32; 2]> {
        *self.biped_band.get_or_init(|| {
            self.source
                .and_then(|source| super::source_biped_slot_band(source, self.chunk_numbers()))
        })
    }
    /// Native ImposedLayout returns an owned copy, not mutable context state.
    pub fn imposed_layout(&self) -> Option<I0Layout> {
        self.imposed.clone()
    }
    /// Cached layout result, including refusal and diagnostic measurements.
    /// Imposed values bypass validity checks and detection, like the native API.
    /// Unlike a Result-only API, an implausible detected candidate is not lost.
    pub fn i0_layout(&self) -> &I0LayoutDetection {
        self.layout.get_or_init(|| {
            if let Some(layout) = &self.imposed {
                return I0LayoutDetection {
                    layout: Some(layout.clone()),
                    report: Default::default(),
                    refusal: None,
                };
            }
            self.source.map_or_else(
                || I0LayoutDetection {
                    layout: None,
                    report: Default::default(),
                    refusal: Some(I0LayoutRefusal::NoChunks),
                },
                super::detect_source_i0_layout,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::io::Read;
    fn bytes(h: &str) -> Vec<u8> {
        (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect()
    }
    fn layout(v: &Value) -> I0Layout {
        I0Layout {
            gate_bits: v["GateBits"].as_i64().unwrap(),
            axis_widths: serde_json::from_value(v["AxisW"].clone()).unwrap(),
            region: v["Region"].as_u64().unwrap() as u32,
        }
    }
    fn registry(r: &FilmRegistryRead, e: &Value) {
        assert_eq!(r.registry.truncated, e["Truncated"].as_bool().unwrap());
        assert_eq!(
            r.truncated_bytes as u64,
            e["TruncatedBytes"].as_u64().unwrap()
        );
        let arches = e["Archetypes"].as_array().cloned().unwrap_or_default();
        assert_eq!(r.registry.archetypes.len(), arches.len());
        for (a, e) in r.registry.archetypes.iter().zip(&arches) {
            assert_eq!(a.index as u64, e["Index"].as_u64().unwrap());
            assert_eq!(serde_json::json!(a.components), e["Components"]);
            assert_eq!(serde_json::json!(a.levels), e["Levels"]);
        }
    }
    #[test]
    fn native_map_context_eager_profile_and_warning_once() {
        let mut base = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
            .read_to_end(&mut base)
            .unwrap();
        let registry = super::super::parse_registry(&base).unwrap();
        let identity = super::super::decode_film_identity(&base, &registry)
            .unwrap()
            .unwrap();
        let rows: Vec<Value> =
            serde_json::from_str(include_str!("fixtures/profile-warning-v41.json")).unwrap();
        assert_eq!(rows.len(), 128);
        for row in rows {
            let mut data = base.clone();
            data[4..8].copy_from_slice(&(row["format"].as_u64().unwrap() as u32).to_le_bytes());
            let offset = identity.build_offset;
            data[offset..offset + 32].fill(0);
            let build = row["build"].as_str().unwrap();
            data[offset..offset + build.len()].copy_from_slice(build.as_bytes());
            let source = row["present"].as_bool().unwrap().then(|| {
                FilmSource::load(
                    &[data],
                    &[super::super::FilmSourceMetadata {
                        index: 0,
                        chunk_type: 1,
                        start_ms: 0,
                    }],
                )
                .unwrap()
            });
            let logs = super::super::log_test_support::capture_logs(|| {
                let mut context = NativeFilmContext::for_map(source.as_ref(), None, None).unwrap();
                assert!(context.profile.get().is_some());
                assert!(context.registry.get().is_none());
                context.profile().unwrap();
                context.profile().unwrap();
                context.scan_profile().unwrap();
                context
                    .set_scan_profile(super::super::NativeScanProfile::default())
                    .unwrap();
            });
            assert_eq!(serde_json::to_value(logs).unwrap(), row["logs"]);
            let logs = super::super::log_test_support::capture_logs(|| {
                let context = NativeFilmContext::new(source.as_ref());
                assert!(context.profile.get().is_none());
                context.profile().unwrap();
            });
            assert!(logs.is_empty());
        }
    }
    #[test]
    fn native_source_context_caches_and_registry_identity() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/context-cache-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 144);
        for row in &rows {
            let inputs = row["inputs"].as_array().unwrap();
            let buffers: Vec<_> = inputs
                .iter()
                .map(|v| bytes(v["hex"].as_str().unwrap()))
                .collect();
            let metadata: Vec<_> = inputs
                .iter()
                .map(|v| super::super::FilmSourceMetadata {
                    index: v["index"].as_i64().unwrap(),
                    chunk_type: 2,
                    start_ms: 0,
                })
                .collect();
            let source =
                (!buffers.is_empty()).then(|| FilmSource::load(&buffers, &metadata).unwrap());
            let imposed = (!row["imposed"].is_null()).then(|| layout(&row["imposed"]));
            let mut context =
                NativeFilmContext::with_imposed_layout(source.as_ref(), imposed.as_ref());
            assert!(
                context.chunks.get().is_none()
                    && context.registry.get().is_none()
                    && context.biped_band.get().is_none()
                    && context.layout.get().is_none()
            );
            let numbers = row["numbers"].as_array().cloned().unwrap_or_default();
            assert_eq!(
                serde_json::json!(context.chunk_numbers()),
                serde_json::json!(numbers)
            );
            assert!(std::ptr::eq(
                context.chunk_numbers(),
                context.chunk_numbers()
            ));
            if let Some(source) = source.as_ref() {
                assert!(std::ptr::eq(context.source().unwrap(), source));
                for number in context.chunk_numbers() {
                    let (bytes, packets) = context.chunk_at(*number).unwrap();
                    let (original, original_packets) = source.chunk_by_number(*number).unwrap();
                    assert!(std::ptr::eq(bytes, original));
                    assert!(std::ptr::eq(packets, original_packets));
                }
            }
            let slots: Vec<_> = context
                .biped_slot_band()
                .map(|[lo, hi]| (lo..=hi).collect())
                .unwrap_or_default();
            assert_eq!(serde_json::json!(slots), row["slots"]);
            match row["registry_error"].as_str().unwrap() {
                "" => {
                    let r = context.registry().unwrap();
                    registry(r, &row["before"]);
                    assert!(std::ptr::eq(r, context.registry().unwrap()));
                    let identity = (
                        row["fingerprint"].as_u64().unwrap(),
                        row["named"].as_u64().unwrap() as usize,
                    );
                    assert_eq!(context.registry_identity().unwrap(), identity);
                    if let Some(a) = context
                        .registry_mut()
                        .unwrap()
                        .registry
                        .archetypes
                        .first_mut()
                    {
                        a.components[0] = "mutation".into();
                        a.levels[0] = 99;
                    }
                    registry(context.registry().unwrap(), &row["after"]);
                    assert_eq!(context.registry_identity().unwrap(), identity);
                }
                kind => {
                    let error = context.registry().unwrap_err();
                    assert!(std::ptr::eq(error, context.registry().unwrap_err()));
                    assert_eq!(
                        *error,
                        if kind == "missing" {
                            NativeContextRegistryError::NoRegistryChunk
                        } else {
                            NativeContextRegistryError::Parse(
                                FilmRegistryReadError::StillCompressed,
                            )
                        }
                    );
                }
            }
            assert_eq!(context.imposed_layout(), imposed);
            if let Some(mut copy) = context.imposed_layout() {
                copy.gate_bits += 1;
                assert_ne!(context.imposed_layout(), Some(copy));
            }
            let actual = context.i0_layout();
            assert!(std::ptr::eq(actual, context.i0_layout()));
            assert_eq!(
                actual.refusal.is_some(),
                row["layout_error"].as_bool().unwrap()
            );
            assert_eq!(
                actual.layout.clone().unwrap_or(I0Layout {
                    gate_bits: 0,
                    axis_widths: [0; 3],
                    region: 0
                }),
                layout(&row["layout"])
            );
        }
        let empty = NativeFilmContext::new(None);
        assert!(empty.source().is_none());
        assert!(empty.chunk_numbers().is_empty());
        assert!(empty.chunk_at(0).is_none());
        assert_eq!(
            empty.registry().unwrap_err(),
            &NativeContextRegistryError::NoRegistryChunk
        );
        assert_eq!(empty.biped_slot_band(), None);
        assert_eq!(empty.i0_layout().refusal, Some(I0LayoutRefusal::NoChunks));
    }
}
