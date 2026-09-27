//! Source-derived caches from the native FilmContext. This is the recording
//! parser's context, not the deferred resolved replay or playback model.
use super::{FilmRegistryRead, FilmRegistryReadError, FilmSource};
use std::cell::OnceCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NativeContextRegistryError {
    #[error("film has no registry chunk")]
    NoRegistryChunk,
    #[error(transparent)]
    Parse(#[from] FilmRegistryReadError),
}

/// Lazy registry and profile caches for one source.
#[derive(Debug)]
pub(crate) struct NativeFilmContext<'a> {
    source: Option<&'a FilmSource>,

    registry: OnceCell<Result<FilmRegistryRead, NativeContextRegistryError>>,
    registry_identity: OnceCell<(u64, usize)>,

    profile: OnceCell<Result<super::NativeResolvedFilmProfile, super::NativeProfileResolveError>>,
    scan: super::NativeScanProfile,
    corruption: OnceCell<super::FilmCorruptionControl>,
}
impl<'a> NativeFilmContext<'a> {
    /// Native no-catalog construction: no parsing or source derivation occurs.
    pub(crate) fn new(source: Option<&'a FilmSource>) -> Self {
        Self {
            source,

            registry: OnceCell::new(),
            registry_identity: OnceCell::new(),

            profile: OnceCell::new(),
            scan: super::NativeScanProfile::default(),
            corruption: OnceCell::new(),
        }
    }
    /// Lazily resolve from original source bytes, then return a deep owned copy.
    /// Caller edits cannot change cached identity tables or map metadata.
    pub(crate) fn profile(
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
    pub(crate) fn corruption_control(
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
    pub(crate) fn scan_profile(
        &self,
    ) -> Result<super::NativeScanProfile, super::NativeProfileResolveError> {
        let mut profile = self.scan.clone();
        profile.grammar.corruption_check = self.corruption_control()?.enabled;
        Ok(profile)
    }
    /// Return the previous effective profile before replacing settings for future
    /// readers. Scalar copies are independent; allocated calibration maps alias.
    pub(crate) fn set_scan_profile(
        &mut self,
        profile: super::NativeScanProfile,
    ) -> Result<super::NativeScanProfile, super::NativeProfileResolveError> {
        let previous = self.scan_profile()?;
        self.scan = profile;
        Ok(previous)
    }
    /// Native CadreDeBalayage inherits settings but deliberately has no observer.
    pub(crate) fn scan_frame(
        &self,
    ) -> Result<super::NativeFrameConfig, super::NativeProfileResolveError> {
        Ok(super::NativeFrameConfig {
            context: super::NativeReaderContext {
                profile: self.scan_profile()?,
            },
            ..Default::default()
        })
    }
    pub(crate) fn registry(&self) -> Result<&FilmRegistryRead, &NativeContextRegistryError> {
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
}
