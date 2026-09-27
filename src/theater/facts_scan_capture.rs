//! Native snapshot boundary between completed scanning and document assembly.
use super::*;

/// Capture already-scanned inputs using the actual context layout. Call before
/// assembly adds fallbacks. This does not scan LegacyFilm, recheck cache freshness, or
/// install the application adapter's Statborg/Kills sections. Layout failure
/// returns no facts, including when detection retained an implausible candidate.
/// Owned copies isolate this snapshot from later input/identity/counter mutation.
pub fn capture_film_scan_facts(
    match_id: &[u8],
    context: &NativeFilmContext<'_>,
    map: Option<&FactsMapEntry>,
    inputs: &NativeFilmFacts,
    mode_guards: &FactsModeGuards,
    identity: Option<&FactsFileIdentity>,
    scan_fallbacks: Option<&FallbackCounter>,
) -> Option<NativeFilmFactsFile> {
    let detected = context.i0_layout();
    if detected.refusal.is_some() {
        return None;
    }
    let layout = detected.layout.as_ref()?;
    let decoder_identity = identity.map(ReplayDecoderIdentity::from);
    let c = build_replay_decoder_coverage(decoder_identity.as_ref());
    let coverage = FactsDecoderCoverage {
        source_rev: c.source_rev.into_bytes(),
        profile_rev: c.profile_rev.into_bytes(),
        grammar_rev: c.grammar_rev.into_bytes(),
        facts_rev: c.facts_rev.into_bytes(),
        build: c.build.into_bytes(),
        registry: c.registry.map(|r| FactsRegistryCoverage {
            fingerprint: r.fingerprint.into_bytes(),
            status: r.status.into_bytes(),
            blocks: r.blocks,
            named_slots: r.named_slots,
        }),
    };
    let mut facts = inputs.clone();
    facts.header.film = match_id.to_vec();
    facts.header.map_module = map.map_or_else(Vec::new, |m| m.module.clone());
    facts.header.axis_widths = layout.axis_widths;
    facts.header.layout_detected = context.imposed_layout().is_none();
    Some(NativeFilmFactsFile {
        coverage,
        facts,
        mode_guards: mode_guards.clone(),
        identity: identity.cloned(),
        fallbacks: scan_fallbacks.map(|c| {
            c.report()
                .into_iter()
                .map(|h| FactsFallback {
                    name: h.name.0,
                    count: h.hits,
                })
                .collect()
        }),
        ..Default::default()
    })
}
