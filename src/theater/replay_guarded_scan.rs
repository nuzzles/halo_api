//! Mode-guarded world channels with native observation and diagnostic ordering.
use super::*;
use std::collections::BTreeMap;

pub struct ReplayGuardedScanOptions<'a> {
    pub flag_scanned: bool,
    pub flag_records: &'a [StatborgRecord],
    pub flag_bursts: &'a [i64],
    pub zones: &'a [ObjectiveZone],
    pub bomb_scanned: bool,
    pub chunk_start_ms: &'a BTreeMap<i64, i64>,
}
#[derive(Debug, Default)]
pub struct ReplayGuardedScan {
    pub carrier: Option<SourceCarrierMarkScan>,
    pub carrier_error: Option<SourceCarrierMarkError>,
    pub managed: Option<ManagedPropertyScan>,
    pub managed_error: Option<ContextManagedPropertyError>,
    pub radial: Option<NavpointRadialScan>,
    pub radial_error: Option<ContextNavpointRadialError>,
    pub diagnostics: Vec<StatborgDiagnostic>,
    pub published: FactsModeGuards,
}
/// Logs and process-counter additions are delivered at their native boundary;
/// the embedding runtime owns their sinks. Channel borrows do not clone data.
pub enum ReplayGuardedObservation<'a> {
    CarrierMarks(&'a FactsCarrierMarkScan),
    ZoneReads(Option<&'a [ManagedPropertyRead]>),
    FlagGauge(Option<&'a [ManagedPropertyRead]>),
    BombReads(Option<&'a [NavpointRadialRead]>),
    Diagnostic(&'a StatborgDiagnostic),
    Counter { name: &'static str, amount: usize },
}
impl ReplayGuardedObservation<'_> {
    pub fn channel_name(&self) -> Option<&'static str> {
        match self {
            Self::CarrierMarks(_) => Some("carrierMarks"),
            Self::ZoneReads(_) => Some("zoneReads"),
            Self::FlagGauge(_) => Some("flagGauge"),
            Self::BombReads(_) => Some("bombReads"),
            Self::Diagnostic(_) | Self::Counter { .. } => None,
        }
    }
}
fn publish_log(
    out: &mut ReplayGuardedScan,
    observe: &mut impl FnMut(ReplayGuardedObservation<'_>),
    log: StatborgDiagnostic,
) {
    observe(ReplayGuardedObservation::Diagnostic(&log));
    out.diagnostics.push(log);
}
macro_rules! log {
    ($out:expr, $observe:expr, $level:literal, $message:literal $(, $key:literal => $value:expr)* $(,)?) => {
        publish_log($out, $observe, StatborgDiagnostic {
            level: $level.into(), message: $message.into(),
            attributes: vec![$(($key.into(), serde_json::json!($value))),*],
        })
    };
}
fn flag_guard(
    options: &ReplayGuardedScanOptions<'_>,
    out: &mut ReplayGuardedScan,
    observe: &mut impl FnMut(ReplayGuardedObservation<'_>),
) -> bool {
    if !options.flag_scanned {
        return false;
    }
    // Each native guard runs NamedEventsFrom independently, including warnings.
    let (events, diagnostics) =
        statborg_named_events_with_diagnostics(options.flag_records, "flag");
    for diagnostic in diagnostics {
        publish_log(out, observe, diagnostic);
    }
    ReplayFlagFilmSignals::from_events(options.flag_bursts, &events).is_flag_film()
}
fn managed_reads(
    context: &NativeFilmContext<'_>,
    match_id: &str,
    out: &mut ReplayGuardedScan,
    observe: &mut impl FnMut(ReplayGuardedObservation<'_>),
) {
    if out.managed.is_some() {
        return;
    }
    let (scan, error) = scan_context_managed_properties(context);
    if let Some(e) = &error {
        log!(out, observe, "INFO", "rejeu : proprietes ti=13 illisibles — rejeu sans etat de zone ni jauge de retour", "err" => e.to_string(), "match_id" => match_id);
    } else {
        log!(out, observe, "INFO", "rejeu : proprietes ti=13 balayees", "match_id" => match_id,
            "slots" => scan.slots, "records" => scan.records, "marches" => scan.walked,
            "cassees" => scan.broken, "chainees" => scan.chained, "lectures" => scan.reads.len());
    }
    out.managed = Some(scan);
    out.managed_error = error;
}
fn published_managed(out: &ReplayGuardedScan) -> Option<Vec<ManagedPropertyRead>> {
    if out.managed_error.is_some() {
        return None;
    }
    out.managed
        .as_ref()
        .filter(|s| !s.reads.is_empty())
        .map(|s| s.reads.clone())
}
// A failed keyframe can roll back its values after allocating the native slice.
// Preserve that empty-but-allocated publication using the retained hook evidence.
fn radial_reads_allocated(scan: &NavpointRadialScan) -> bool {
    !scan.reads.is_empty() || scan.attempts.iter().any(|a| {
        let NavpointAttemptData::Keyframe { record: Some(record) } = &a.data else { return false; };
        record.diagnostics.component_observations.iter().any(|o| matches!(o,
            FilmComponentObservation::Navpoint { field: NativeNavpointField::RadialProgress, values }
            if !values.is_empty()))
    })
}

/// Native balayerCalquesGardes. No guard is inferred from map geometry. Failed
/// scans retain evidence but clear their publication; scanned flags report guards.
/// This phase never captures facts before the remaining phases have finished.
pub fn scan_replay_guarded_inputs(
    match_id: &[u8],
    context: &NativeFilmContext<'_>,
    options: ReplayGuardedScanOptions<'_>,
    mut observe: impl FnMut(ReplayGuardedObservation<'_>),
) -> ReplayGuardedScan {
    let match_id = ReplayByteString(match_id.to_vec()).json_text();
    let mut out = ReplayGuardedScan::default();
    if flag_guard(&options, &mut out, &mut observe) {
        match scan_source_carrier_marks(context.source()) {
            Ok(scan) => {
                out.published.flag_marks = FactsCarrierMarkScan::from(&scan.scan);
                out.carrier = Some(scan);
            }
            Err(e) => {
                log!(&mut out, &mut observe, "WARN", "drapeau : marqueur de portage illisible — calque publie sans son controle", "err" => e.to_string(), "match_id" => match_id);
                out.carrier_error = Some(e);
            }
        }
    }
    observe(ReplayGuardedObservation::CarrierMarks(
        &out.published.flag_marks,
    ));
    let zones = !options.zones.is_empty();
    let gauge = flag_guard(&options, &mut out, &mut observe);
    if zones {
        managed_reads(context, &match_id, &mut out, &mut observe);
        out.published.zone_reads = published_managed(&out);
    } else {
        log!(&mut out, &mut observe, "DEBUG", "rejeu : aucune zone au catalogue — proprietes ti=13 non consommees par les zones", "match_id" => match_id);
    }
    out.published.zone_scanned = zones;
    observe(ReplayGuardedObservation::ZoneReads(
        out.published.zone_reads.as_deref(),
    ));
    if gauge {
        managed_reads(context, &match_id, &mut out, &mut observe);
        out.published.flag_gauge = published_managed(&out);
    }
    out.published.flag_gauge_scanned = gauge;
    observe(ReplayGuardedObservation::FlagGauge(
        out.published.flag_gauge.as_deref(),
    ));
    if options.bomb_scanned {
        if options.chunk_start_ms.is_empty() {
            log!(&mut out, &mut observe, "WARN", "armement : film armable sans horloge de manifeste — calque non construit", "match_id" => match_id);
        } else {
            let (scan, error) = scan_context_navpoint_radial(context, options.chunk_start_ms);
            if let Some(e) = &error {
                log!(&mut out, &mut observe, "WARN", "armement : anneau ti=12 illisible — rejeu sans compte a rebours", "err" => e.to_string(), "match_id" => match_id);
            } else {
                if scan.truncated {
                    log!(&mut out, &mut observe, "WARN", "armement : recolte TRONQUEE au plafond de lectures — armements tardifs possibles manquants", "match_id" => match_id);
                }
                log!(&mut out, &mut observe, "INFO", "armement : anneau ti=12 balaye", "match_id" => match_id,
                    "slots" => scan.slots_observed, "records" => scan.records, "marches" => scan.walked,
                    "chainees" => scan.chained, "lectures" => scan.reads.len(), "paquetsSansHorloge" => scan.packets_no_clock,
                    "imageCleRecords" => scan.key_records, "imageCleFermees" => scan.key_closed, "imageCleBornees" => scan.key_bounded);
                for (name, amount) in scan.keyframe_closure_counters() {
                    observe(ReplayGuardedObservation::Counter { name, amount });
                }
                out.published.bomb_reads =
                    radial_reads_allocated(&scan).then(|| scan.reads.clone());
            }
            out.radial = Some(scan);
            out.radial_error = error;
        }
    }
    observe(ReplayGuardedObservation::BombReads(
        out.published.bomb_reads.as_deref(),
    ));
    out
}
