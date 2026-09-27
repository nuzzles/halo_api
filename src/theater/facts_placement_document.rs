//! Native grapple and equipment-placement pass after coverage creation.
use super::*;

pub struct FactsReplayPlacementDocumentInput<'a> {
    pub grapple: &'a [FactsGrappleRead],
    pub map: Option<&'a FilmMapBounds>,
    /// Caller-prepared census, original chronological positions, spawn/change
    /// evidence, registry lives and title catalog. No source is scanned here.
    pub placements: FactsReplayEquipmentPlacementInput<'a>,
    pub origin_us: u64,
    pub step_us: u64,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsReplayPlacementDocumentReport {
    pub manifest_fallbacks: usize,
    /// Ordered native log publications; the final composer owns the sink.
    pub diagnostics: Vec<StatborgDiagnostic>,
}
/// Publish grapple lines only with map bounds, then replace placements and their
/// coverage. Missing map bounds preserve seeded grapple lines and coverage.
/// The placement clock takes its frame count from the document.
///
/// # Panics
/// Requires the preceding native coverage-envelope pass to have completed.
pub fn assemble_facts_replay_grapple_and_placements(
    doc: &mut ReplayDocument,
    input: FactsReplayPlacementDocumentInput<'_>,
) -> FactsReplayPlacementDocumentReport {
    let coverage = doc
        .coverage
        .as_mut()
        .expect("placement stage requires coverage envelope");
    let mut out = FactsReplayPlacementDocumentReport::default();
    if let Some(map) = input.map {
        let grapple = build_facts_replay_grapple(
            input.grapple,
            map,
            input.origin_us,
            input.step_us,
            doc.content.tracks.as_deref().unwrap_or_default(),
        );
        doc.content.grapple_lines = grapple.lines;
        let c = &grapple.coverage;
        out.diagnostics.push(StatborgDiagnostic {
            level: "INFO".into(),
            message: "rejeu : tractions de grappin".into(),
            attributes: vec![
                ("tirs".into(), serde_json::json!(c.light_reads)),
                ("accroches".into(), serde_json::json!(c.heavy_reads)),
                ("tractions".into(), serde_json::json!(c.pulls)),
                ("vies".into(), serde_json::json!(c.pull_lives)),
                ("rates".into(), serde_json::json!(c.unpaired_fires)),
                ("corpsCasses".into(), serde_json::json!(c.broken_bodies)),
            ],
        });
        coverage.grapple = Some(grapple.coverage);
    } else if !input.grapple.is_empty() {
        out.diagnostics.push(StatborgDiagnostic {
            level: "WARN".into(),
            message: "rejeu : lectures de grappin sans bornes de carte — aucune traction publiee"
                .into(),
            attributes: vec![("lectures".into(), serde_json::json!(input.grapple.len()))],
        });
    }
    let placements = build_facts_replay_equipment_placements(
        input.placements,
        IdentityClock {
            origin_us: input.origin_us,
            step_us: input.step_us,
            frame_count: doc.content.frame_count,
        },
    );
    doc.content.equipment_placements = placements.placements;
    out.manifest_fallbacks = placements.manifest_fallbacks;
    let c = &placements.coverage;
    out.diagnostics.push(StatborgDiagnostic {
        level: "INFO".into(),
        message: "rejeu : poses d'equipement".into(),
        attributes: vec![
            ("balaye".into(), serde_json::json!(c.scanned)),
            ("calibre".into(), serde_json::json!(c.calibrated)),
            ("decoupage".into(), serde_json::json!(c.widths)),
            ("poses".into(), serde_json::json!(c.placements)),
            ("nommees".into(), serde_json::json!(c.named)),
            ("autres".into(), serde_json::json!(c.other)),
            ("avecPoseur".into(), serde_json::json!(c.with_owner)),
            ("avecCap".into(), serde_json::json!(c.with_heading)),
            ("deployees".into(), serde_json::json!(c.deployed)),
            ("lachees".into(), serde_json::json!(c.dropped)),
            ("origineInconnue".into(), serde_json::json!(c.unknown)),
            ("finVue".into(), serde_json::json!(c.end_seen)),
            ("finOuverte".into(), serde_json::json!(c.end_open)),
            ("evenements103".into(), serde_json::json!(c.spawn_events)),
            ("listesEvenements".into(), serde_json::json!(c.spawn_lists)),
            ("parCause".into(), serde_json::json!(c.by_cause)),
        ],
    });
    coverage.placements = Some(placements.coverage);
    out
}
