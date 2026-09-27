//! Coverage of published vehicle lives, measured after relay merging.
use super::{ReplayVehicleCoverage, ReplayVehicleTrack};

/// Set the published denominator and accumulate native counts into an existing
/// scan coverage record. Unknown chassis counts also identify neutral-marker
/// fallbacks. Only adjacent ride overlaps count, matching native publication.
pub fn tally_replay_vehicle_coverage(
    tracks: &[ReplayVehicleTrack],
    coverage: &mut ReplayVehicleCoverage,
) {
    coverage.published = tracks.len() as i64;
    for track in tracks {
        coverage.with_spawn += i64::from(track.spawn.is_some());
        if !track.chassis.is_empty() {
            coverage.with_chassis += 1;
            if track.family.is_empty() {
                coverage.family_unknown += 1;
                *coverage
                    .unknown_chassis
                    .entry(track.chassis.clone())
                    .or_default() += 1;
            } else {
                coverage.family_resolved += 1;
            }
        }
        coverage.samples += track.samples.len() as i64;
        coverage.with_heading += track.samples.iter().filter(|s| s.h != 0.).count() as i64;
        if !track.rides.is_empty() {
            coverage.vehicles_ridden += 1;
        }
        coverage.rides += track.rides.len() as i64;
        for (i, ride) in track.rides.iter().enumerate() {
            coverage.rides_named += i64::from(!ride.xuid.is_empty());
            coverage.rides_with_seat += i64::from(ride.seat.is_some());
            coverage.aim_ride_frames = coverage
                .aim_ride_frames
                .wrapping_add(ride.t1.wrapping_sub(ride.t0).wrapping_add(1));
            coverage.rides_with_aim += i64::from(!ride.aim.is_empty());
            coverage.aim_samples += ride.aim.len() as i64;
            if ride.src == "film" {
                coverage.rides_read += 1;
            } else {
                coverage.rides_proximity += 1;
            }
            if i > 0 && ride.t0 <= track.rides[i - 1].t1 {
                coverage.ambiguous += 1;
            }
        }
        match track.end.as_str() {
            "destroyed" => coverage.end_destroyed += 1,
            "film_end" => coverage.end_film_end += 1,
            _ => coverage.end_unknown += 1,
        }
        if let Some(end) = track.t_end {
            coverage.samples_after_end += track.samples.iter().filter(|s| s.t > end).count() as i64;
        }
    }
}

impl ReplayVehicleCoverage {
    /// Emit native vehicle observations; chassis warnings use stable key order.
    pub fn log(&self) {
        tracing::info!(
            balaye = self.scanned,
            viesRecensees = self.lives,
            publiees = self.published,
            relaisFusionnes = self.merged,
            sansPosition = self.no_position,
            avecNaissance = self.with_spawn,
            avecChassis = self.with_chassis,
            famillesResolues = self.family_resolved,
            famillesInconnues = self.family_unknown,
            echantillons = self.samples,
            avecCap = self.with_heading,
            "rejeu : vehicules"
        );
        tracing::info!(
            episodes = self.rides,
            vehiculesOccupes = self.vehicles_ridden,
            occupantsNommes = self.rides_named,
            lus = self.rides_read,
            parProximite = self.rides_proximity,
            avecSiege = self.rides_with_seat,
            ambigus = self.ambiguous,
            lecturesDeViseeBrutes = self.aim_reads,
            episodesAvecVisee = self.rides_with_aim,
            pointsDeVisee = self.aim_samples,
            framesDEpisode = self.aim_ride_frames,
            "rejeu : occupation des vehicules"
        );
        tracing::info!(
            emplacements = self.cycle_locations,
            cycles = self.cycles,
            ecarts = self.cycle_gaps,
            manques = self.cycle_missing,
            "rejeu : cycle de reapparition des vehicules"
        );
        if self.rides > 0 && self.rides_with_aim == 0 {
            tracing::warn!(
                episodes = self.rides,
                lecturesBrutes = self.aim_reads,
                "rejeu : AUCUN episode d occupation ne porte de visee alors que des episodes existent — le balayage des records de visee SANS position n a rien rendu, le cone retombe partout sur le cap du chassis"
            );
        }
        for (id, count) in &self.unknown_chassis {
            tracing::warn!(
                chassis = id.as_str(),
                vies = *count,
                repli = "repli_chassis_vehicule_marqueur_neutre",
                "rejeu : chassis de vehicule ABSENT DE LA TABLE DES FAMILLES — vies publiees sans sprite, dessinees en marqueur neutre ; le mot d identite est LU, c est la table qui ne le nomme pas"
            );
        }
        if self.with_chassis > 0 && self.family_resolved == 0 {
            tracing::warn!(
                chassisLus = self.with_chassis,
                "rejeu : AUCUN chassis de vehicule resolu alors que le mot d identite a ete lu — table de familles a completer, ou lecture du bloc MPP a verifier"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn native_vehicle_coverage_observations() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-vehicle-coverage-logs-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 513);
        for (index, row) in rows.into_iter().enumerate() {
            let coverage: Option<ReplayVehicleCoverage> =
                serde_json::from_value(row["coverage"].clone()).unwrap();
            let logs = super::super::log_test_support::capture_logs(|| {
                if let Some(coverage) = coverage {
                    coverage.log();
                }
            });
            assert_eq!(
                serde_json::to_value(logs).unwrap(),
                row["logs"],
                "case {index}"
            );
        }
    }
}
