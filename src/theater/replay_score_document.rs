//! Native score/objective document stage, before the coverage envelope is built.
use super::*;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct ReplayScoreDocumentInput<'a> {
    pub score: Option<ReplayScoreInput<'a>>,
    pub deaths: &'a [StatborgDeathInstant],
    pub objectives: &'a [StatborgIdentifiedEvent],
    pub objectives_unnamed: i64,
    pub objectives_refused: i64,
}
pub struct ReplayScoreDocumentContext<'a, 'teams> {
    /// The assembler's effective interval, which may differ from a seeded document.
    pub interval_ms: i64,
    pub teams: &'a ReplayTeamPublication<'teams>,
    /// Counts retained by the preceding team publication, not recomputed here.
    pub track_counts: [usize; 3],
    /// The completed registry's ambiguity-filtered naming bridge.
    pub slot_xuids: &'a BTreeMap<u32, u64>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayScoreDocumentReport {
    pub clock: ReplayScoreClock,
    pub teams: ReplayTeamCoverage,
    pub objectives: ReplayLayerCoverage,
    pub score: Option<ReplayScoreCoverage>,
    pub objectives_without_track: usize,
    pub round_zero_fallbacks: usize,
    /// Native log records, including score-reducer warnings, in emission order.
    /// Exposed explicitly; the caller owns their runtime logging sink.
    pub diagnostics: Vec<StatborgDiagnostic>,
}

/// Measure teams, establish the score clock, publish objectives, then publish score.
/// Both layers replace prior values even when empty. Coverage is returned for the
/// later envelope pass; any existing document coverage is left untouched.
/// This consumes caller-prepared inputs and does not scan or read facts sections.
pub fn attach_replay_score_and_objectives_to_document(
    doc: &mut ReplayDocument,
    input: ReplayScoreDocumentInput<'_>,
    ctx: ReplayScoreDocumentContext<'_, '_>,
) -> ReplayScoreDocumentReport {
    let teams = ctx.teams.coverage(ctx.track_counts, &doc.content.roster);
    let mut diagnostics = Vec::new();
    super::replay_score_document_diagnostics::team(&mut diagnostics, &doc.content.match_id, &teams);
    let clock = ReplayScoreClock {
        origin_ms: doc.content.origin_ms.unwrap_or(0),
        interval_ms: ctx.interval_ms,
        frames: doc.content.frame_count,
    };
    super::replay_score_document_diagnostics::origin(
        &mut diagnostics,
        &doc.content.match_id,
        doc.content.origin_ms,
    );
    let mut objectives = build_replay_objective_actions(
        input.objectives,
        input.objectives_unnamed,
        input.objectives_refused,
        clock,
    );
    objectives.without_track = count_objective_actions_without_track(
        &objectives.actions,
        doc.content.tracks.as_deref().unwrap_or_default(),
        ctx.slot_xuids,
    );
    doc.content.objectives = objectives.actions;
    super::replay_score_document_diagnostics::objectives(
        &mut diagnostics,
        &doc.content.match_id,
        &objectives.coverage,
        objectives.without_track,
        doc.content.objectives.len(),
    );
    let records = input.score.as_ref().map(|s| s.teams.records);
    let (score, warnings) = build_replay_score_with_diagnostics(input.score, input.deaths, clock);
    diagnostics.extend(warnings);
    doc.content.score_timeline = score.timeline;
    super::replay_score_document_diagnostics::score(
        &mut diagnostics,
        &doc.content.match_id,
        score.coverage.as_ref(),
        doc.content.score_timeline.as_ref(),
        records,
    );
    ReplayScoreDocumentReport {
        clock,
        teams,
        objectives: objectives.coverage,
        score: score.coverage,
        objectives_without_track: objectives.without_track,
        round_zero_fallbacks: score.round_zero_fallbacks,
        diagnostics,
    }
}
