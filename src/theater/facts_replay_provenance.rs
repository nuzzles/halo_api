//! Cache-entry provenance restored before any document assembly fallback occurs.
use super::*;

/// Accumulate persisted scan-only fallbacks into the caller's counter and restore
/// the optional native identity projection. The caller creates a counter first
/// when none was supplied. The facts header's old coverage is not authoritative
/// for the new build's decoder revisions. Statborg/Kills sections are not applied.
/// This is the provenance step of BuildFromFacts, not the full document entry.
pub fn restore_facts_replay_provenance(
    file: &NativeFilmFactsFile,
    counter: &FallbackCounter,
) -> Option<ReplayDecoderIdentity> {
    for hit in file.fallbacks.as_deref().unwrap_or_default() {
        counter.trigger_n(&hit.name, hit.count);
    }
    file.identity.as_ref().map(ReplayDecoderIdentity::from)
}
