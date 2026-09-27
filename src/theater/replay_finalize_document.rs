//! Final coverage observation, fallback publication and produced-layer revisions.
use super::*;

/// Native finalization for a nonempty assembled document. Shot counts are read
/// from the current document after vehicle recovery; grenade counts come from
/// the earlier assembler measurement. Fallbacks publish after every layer has
/// had the opportunity to trigger them, then layer revisions publish last.
/// Empty-position assembly skips this function and calls `publish_layers` alone.
///
/// # Panics
/// Requires the coverage envelope produced by the preceding assembly stages.
pub fn finalize_replay_document(
    doc: &mut ReplayDocument,
    grenade_coverage: &ReplayLayerCoverage,
    fallbacks: Option<&FallbackCounter>,
    layers: ReplayLayerInputs,
) {
    let coverage = doc
        .coverage
        .as_ref()
        .expect("finalization requires coverage envelope");
    let shots = &coverage.shots;
    let verdict = |key: &str| coverage.verdict.get(key).map_or("", String::as_str);
    tracing::info!(
        tirsRattaches = shots.attached,
        tirsDisponibles = shots.available,
        tirsSansSlot = shots.no_slot,
        tirsAmbigus = shots.ambiguous,
        tirsHorsFenetre = shots.out_of_window,
        tirsNonPublies = shots.unpublished,
        grenadesRattachees = grenade_coverage.attached,
        grenadesDisponibles = grenade_coverage.available,
        verdictTirs = verdict("shots"),
        verdictGrenades = verdict("grenades"),
        verdictPont = verdict("bridge"),
        "rejeu : couverture par calque"
    );
    attach_replay_fallback_coverage(doc, fallbacks);
    doc.publish_layers(layers);
}

/// Replace and log the counter's native sorted positive report. Missing coverage
/// is a no-op. A missing counter clears a seeded report, just like a nil native
/// counter. Values remain signed 64-bit on both host and WASM.
pub fn attach_replay_fallback_coverage(
    doc: &mut ReplayDocument,
    counter: Option<&FallbackCounter>,
) {
    let Some(coverage) = doc.coverage.as_mut() else {
        return;
    };
    coverage.fallbacks = counter.map_or_else(Vec::new, |counter| {
        counter
            .report()
            .into_iter()
            .map(|h| ReplayFallbackHit {
                name: h.name,
                hits: h.hits,
            })
            .collect()
    });
    for h in &coverage.fallbacks {
        tracing::info!(
            match_id = doc.content.match_id.as_str(),
            repli = h.name.json_text().as_str(),
            declenchements = h.hits,
            "rejeu : repli declenche"
        );
    }
}
