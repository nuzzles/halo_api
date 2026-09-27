//! Shared-context world-position entry used by fresh replay scanning.
use super::*;
use std::collections::BTreeSet;

#[derive(Debug, thiserror::Error)]
pub enum ContextPositionScanError {
    #[error(transparent)]
    Scan(#[from] DecodeError),
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
}

/// World-coordinate branch of native ScanBipedPositions/ForBand. None selects
/// biped band discovery using the requested chunks (including the following
/// keyframe), not the context's cached whole-film band. An explicit empty band
/// is a refusal. Missing layout requests detection from the film prefix; it does
/// not implicitly use the context's imposed layout. The fresh replay entry must
/// pass ImposedLayout explicitly, as the native scanFilmInputs does.
///
/// Reads reuse source buffers and the context's observer. Mask hooks run before
/// temporal filtering. World bounds are mandatory here; the existing quanta-only
/// source entry remains separate. Refusals use the existing Rust DecodeError
/// categories rather than reproducing native localized error text.
pub fn scan_context_world_positions(
    context: &NativeFilmContext<'_>,
    chunks: &[i64],
    band: Option<&BTreeSet<u32>>,
    layout: Option<&I0Layout>,
    bounds: [[f32; 3]; 2],
    options: &SourceWorldScanOptions,
) -> Result<SourceWorldPositionReport, ContextPositionScanError> {
    let chunks = if chunks.is_empty() {
        context.chunk_numbers()
    } else {
        chunks
    };
    if chunks.is_empty() {
        return Err(DecodeError::Missing("film chunks").into());
    }
    let discovered;
    let band = match band {
        Some(band) => band,
        None => {
            let source = context
                .source()
                .ok_or(DecodeError::Missing("biped slots"))?;
            discovered = super::quantized_source::automatic_source_band(source, chunks)?;
            &discovered
        }
    };
    if band.is_empty() {
        return Err(DecodeError::Missing("slot band").into());
    }
    let detected;
    let layout = match layout {
        Some(layout) => layout,
        None => {
            let source = context.source().ok_or(DecodeError::Missing("i0 layout"))?;
            detected = super::quantized_source::source_scan_layout(source, chunks, band, None)?;
            &detected
        }
    };
    // Resolve the reader context at the native boundary, after setup refusals.
    // Position decoding uses only its observer; calibrated traversal settings
    // govern other scanners and do not replace the explicit i0 scan layout.
    let reader = context.reader_context()?;
    let source = context
        .source()
        .ok_or(DecodeError::Missing("readable film chunks"))?;
    Ok(scan_source_world_positions_for_band_observed(
        source,
        chunks,
        band,
        layout,
        bounds,
        options,
        reader.observer.as_ref(),
    )?)
}
