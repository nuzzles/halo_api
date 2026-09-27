//! Native coverage-envelope creation after score and equipment publication.
use super::*;
use std::collections::BTreeSet;

#[derive(Default)]
pub struct ReplayCoverageDocumentInput {
    pub shots: ReplayLayerCoverage,
    pub grenades: ReplayLayerCoverage,
    pub objectives: ReplayLayerCoverage,
    pub score: Option<ReplayScoreCoverage>,
    pub projectiles: Option<ReplayProjectileCoverage>,
    pub tracks: ReplayTrackCoverage,
    pub teams: ReplayTeamCoverage,
    pub stances: ReplayStanceCoverage<ReplayByteString>,
    pub seats: ReplaySeatCoverage,
    pub film_major_version: Option<i64>,
    pub kills_read: bool,
    pub deaths_paths: Option<ReplayDeathsPathsCoverage>,
}
pub struct ReplayCoverageDocumentContext<'a> {
    pub registry: &'a IdentityRegistryOutput,
    pub naming: &'a IdentityRemainingReport,
    pub closed_by_death: &'a BTreeSet<usize>,
    pub decoder_identity: Option<&'a ReplayDecoderIdentity>,
    /// Effective assembler interval; do not substitute the seeded document field.
    pub interval_ms: i64,
}
/// Replace the coverage envelope, then date kickoff from the current published
/// tracks when an origin exists. Without an origin, preserve any existing T0 on
/// the document (native composer behavior); the new coverage has no T0 measure.
/// Decoder coverage is always present, even without a FilmIdentity. Death-path
/// presence is copied independently of the equipment kill-read gate.
/// Returns native diagnostic records for the composer's runtime sink.
pub fn assemble_replay_document_coverage(
    doc: &mut ReplayDocument,
    input: ReplayCoverageDocumentInput,
    ctx: ReplayCoverageDocumentContext<'_>,
) -> Vec<StatborgDiagnostic> {
    let bridge = ctx.registry.bridge_health();
    let mut diagnostics = Vec::new();
    if bridge.death_offset_matched > 0
        && (bridge.death_offset_matched as i64)
            < (bridge.death_offset_runner_up as i64).wrapping_mul(2)
    {
        diagnostics.push(StatborgDiagnostic {
            level: "WARN".into(),
            message: "rejeu : calage du fil des morts trop peu distinct du bruit — nommage et origine suspects".into(),
            attributes: vec![
                ("apparies".into(), serde_json::json!(bridge.death_offset_matched)),
                ("second_candidat".into(), serde_json::json!(bridge.death_offset_runner_up)),
                ("marge_minimale".into(), serde_json::json!(2)),
                ("vies".into(), serde_json::json!(bridge.lives_total)),
                ("slots".into(), serde_json::json!(bridge.slots)),
            ],
        });
    }
    let mut coverage = ReplayCoverage::from_bridge(
        input.shots,
        input.grenades,
        input.objectives,
        bridge,
        doc.content.origin_ms.is_some(),
        input.score,
    );
    coverage.projectiles = input.projectiles;
    coverage.tracks = Some(input.tracks);
    coverage.teams = Some(input.teams);
    coverage.stances = Some(input.stances);
    coverage.seats = Some(input.seats);
    coverage.film_major_version = input.film_major_version;
    coverage.decoder = Some(build_replay_decoder_coverage(ctx.decoder_identity));
    ctx.naming.apply_to_health(&mut coverage.bridge);
    let mut equipment = build_replay_equipment_coverage(
        &doc.content.equipment_episodes,
        doc.content.tracks.as_deref().unwrap_or_default(),
        ctx.closed_by_death,
    );
    equipment.kills_read = input.kills_read;
    coverage.equipment = Some(equipment);
    coverage.deaths_paths = input.deaths_paths;
    doc.coverage = Some(coverage);
    if let Some(origin) = doc.content.origin_ms {
        let (time, t0) = detect_replay_t0(
            doc.content.tracks.as_deref().unwrap_or_default(),
            ctx.interval_ms,
            origin,
        );
        if !t0.detected {
            diagnostics.push(StatborgDiagnostic {
                level: "WARN".into(),
                message:
                    "rejeu : coup d'envoi NON date par le film — le T0 de l'API reste la source"
                        .into(),
                attributes: vec![
                    ("match_id".into(), serde_json::json!(doc.content.match_id)),
                    ("raison".into(), serde_json::json!(t0.reason)),
                    ("pistes".into(), serde_json::json!(t0.tracks)),
                    ("pistesEnMouvement".into(), serde_json::json!(t0.moving)),
                    ("rafale".into(), serde_json::json!(t0.burst)),
                    ("margeMs".into(), serde_json::json!(t0.margin_ms)),
                ],
            });
        }
        doc.content.t0_film_ms = time;
        doc.coverage.as_mut().unwrap().t0_film = Some(t0);
    }
    diagnostics
}
