#!/usr/bin/env python3
"""Run the pinned Go reference and refresh independent Rust parity fixtures.

Requires a LevelUp checkout, Go 1.26.5+, and the downloaded v41 corpus. It writes
test harnesses in the reference checkout, temporary comparison data,
and the explicitly selected fixture directory. It never rewrites film exports.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import zlib

PIN = "43a01721e8a02c87c955e175936f0ccf8dd97a81"


# Source/publication oracles that supplement the grammar and document matrices.
# Inputs must already be generated; never reuse undeclared host /tmp artifacts.
SOURCE_PUBLICATION_ORACLES = (
    ('halo_rust_facts_ability_actions_test.go.txt', 'TestHaloRustFactsAbilityActions', 'replay', 'facts-ability-actions', 'facts-ability-actions-v41.json.zlib'),
    ('halo_rust_facts_equipment_episodes_test.go.txt', 'TestHaloRustFactsEquipmentEpisodes', 'replay', 'facts-equipment-episodes', 'facts-equipment-episodes-v41.json.zlib'),
    ('halo_rust_facts_grapple_test.go.txt', 'TestHaloRustFactsGrapple', 'replay', 'facts-grapple', 'facts-grapple-v41.json.zlib'),
    ('halo_rust_facts_weapon_changes_test.go.txt', 'TestHaloRustFactsWeaponChanges', 'replay', 'facts-weapon-changes', 'facts-weapon-changes-v41.json.zlib'),
    ('halo_rust_facts_pickups_test.go.txt', 'TestHaloRustFactsPickups', 'replay', 'facts-pickups', 'facts-pickups-v41.json.zlib'),

    ('halo_rust_facts_equipment_publication_test.go.txt', 'TestHaloRustFactsEquipmentPublication', 'replay', 'facts-equipment-publication', 'facts-equipment-publication-v41.json.zlib'),
    ('halo_rust_facts_player_inventory_test.go.txt', 'TestHaloRustFactsPlayerInventory', 'replay', 'facts-player-inventory', 'facts-player-inventory-v41.json.zlib'),
    ('halo_rust_facts_abilities_test.go.txt', 'TestHaloRustFactsAbilities', 'replay', 'facts-abilities', 'facts-abilities-v41.json.zlib'),
    ('halo_rust_facts_inventory_publication_test.go.txt', 'TestHaloRustFactsInventoryPublication', 'replay', 'facts-inventory-publication', 'facts-inventory-publication-v41.json.zlib'),
    ('halo_rust_facts_projectiles_test.go.txt', 'TestHaloRustFactsProjectiles', 'replay', 'facts-projectiles', 'facts-projectiles-v41.json.zlib'),
    ('halo_rust_facts_grenades_test.go.txt', 'TestHaloRustFactsGrenades', 'replay', 'facts-grenades', 'facts-grenades-v41.json.zlib'),
    ('halo_rust_facts_loadouts_test.go.txt', 'TestHaloRustFactsLoadouts', 'replay', 'facts-loadouts', 'facts-loadouts-v41.json.zlib'),
    ('halo_rust_facts_grenade_reads_test.go.txt', 'TestHaloRustFactsGrenadeReads', 'replay', 'facts-grenade-reads', 'facts-grenade-reads-v41.json.zlib'),
    ('halo_rust_facts_vehicle_shots_test.go.txt', 'TestHaloRustFactsVehicleShots', 'replay', 'facts-vehicle-shots', 'facts-vehicle-shots-v41.json.zlib'),
    ('halo_rust_facts_shots_test.go.txt', 'TestHaloRustFactsShots', 'replay', 'facts-shots', 'facts-shots-v41.json.zlib'),
    ('halo_rust_facts_players_test.go.txt', 'TestHaloRustFactsPlayers', 'replay', 'facts-players', 'facts-players-v41.json.zlib'),
    ('halo_rust_facts_scope_test.go.txt', 'TestHaloRustFactsScope', 'replay', 'facts-scope', 'facts-scope-v41.json.zlib'),
    ('halo_rust_facts_file_test.go.txt', 'TestHaloRustFactsFile', 'replay', 'facts-file', 'facts-file-v41.json.zlib'),
    ('halo_rust_facts_kills_json_test.go.txt', 'TestHaloRustFactsKillsJSON', 'replay', 'facts-kills-json', 'facts-kills-json-v41.json.zlib'),
    ('halo_rust_facts_statborg_json_test.go.txt', 'TestHaloRustFactsStatborgJSON', 'replay', 'facts-statborg-json', 'facts-statborg-json-v41.json.zlib'),
    ('halo_rust_facts_json_floats_test.go.txt', 'TestHaloRustFactsJSONFloats', 'replay', 'facts-json-floats', 'facts-json-floats-v41.json.zlib'),
    ('halo_rust_facts_identity_json_test.go.txt', 'TestHaloRustFactsIdentityJSON', 'replay', 'facts-identity-json', 'facts-identity-json-v41.json.zlib'),
    ('halo_rust_facts_file_header_test.go.txt', 'TestHaloRustFactsFileHeader', 'replay', 'facts-file-header', 'facts-file-header-v41.json.zlib'),
    ('halo_rust_facts_assembly_test.go.txt', 'TestHaloRustFactsAssembly', 'replay', 'facts-assembly', 'facts-assembly-v41.json.zlib'),
    ('halo_rust_facts_tail_test.go.txt', 'TestHaloRustFactsTail', 'replay', 'facts-tail', 'facts-tail-v41.json.zlib'),
    ('halo_rust_facts_state_channels_test.go.txt', 'TestHaloRustFactsStateChannels', 'replay', 'facts-state-channels', 'facts-state-channels-v41.json.zlib'),
    ('halo_rust_facts_body_test.go.txt', 'TestHaloRustFactsBody', 'replay', 'facts-body', 'facts-body-v41.json.zlib'),
    ('halo_rust_facts_header_test.go.txt', 'TestHaloRustFactsHeader', 'replay', 'facts-header', 'facts-header-v41.json.zlib'),
    ('halo_rust_facts_channels_test.go.txt', 'TestHaloRustFactsChannels', 'replay', 'facts-channels', 'facts-channels-v41.json.zlib'),
    ('halo_rust_facts_vehicles_test.go.txt', 'TestHaloRustFactsVehicles', 'replay', 'facts-vehicles', 'facts-vehicles-v41.json.zlib'),
    ('halo_rust_facts_json_syntax_test.go.txt', 'TestHaloRustFactsJSONSyntax', 'replay', 'facts-json-syntax', 'facts-json-syntax-v41.json.zlib'),
    ('halo_rust_facts_object_death_decode_test.go.txt', 'TestHaloRustFactsObjectDeathDecode', 'replay', 'facts-object-death-decode', 'facts-object-death-decode-v41.json.zlib'),
    ('halo_rust_facts_object_deaths_test.go.txt', 'TestHaloRustFactsObjectDeaths', 'replay', 'facts-object-deaths', 'facts-object-deaths-v41.json.zlib'),
    ('halo_rust_facts_placement_stats_test.go.txt', 'TestHaloRustFactsPlacementStats', 'replay', 'facts-placement-stats', 'facts-placement-stats-v41.json.zlib'),
    ('halo_rust_facts_guards_test.go.txt', 'TestHaloRustFactsGuards', 'replay', 'facts-guards', 'facts-guards-v41.json.zlib'),
    ('halo_rust_facts_positions_test.go.txt', 'TestHaloRustFactsPositions', 'replay', 'facts-positions', 'facts-positions-v41.json.zlib'),
    ('halo_rust_facts_world_test.go.txt', 'TestHaloRustFactsWorld', 'replay', 'facts-world', 'facts-world-v41.json.zlib'),
    ('halo_rust_facts_keyframe_codec_test.go.txt', 'TestHaloRustFactsKeyframeCodec', 'replay', 'facts-keyframe-codec', 'facts-keyframe-codec-v41.json.zlib'),
    ('halo_rust_facts_keyframes_test.go.txt', 'TestHaloRustFactsKeyframes', 'replay', 'facts-keyframes', 'facts-keyframe-count-v41.json.zlib'),
    ('halo_rust_facts_tracks_test.go.txt', 'TestHaloRustFactsTracks', 'replay', 'facts-tracks', 'facts-tracks-v41.json.zlib'),
    ('halo_rust_facts_ammo_test.go.txt', 'TestHaloRustFactsAmmo', 'replay', 'facts-ammo', 'facts-ammo-v41.json.zlib'),
    ('halo_rust_facts_transport_test.go.txt', 'TestHaloRustFactsTransport', 'replay', 'facts-transport', 'facts-transport-v41.json.zlib'),
    ('halo_rust_background_directory_test.go.txt', 'TestHaloRustBackgroundDirectory', 'replay', 'background-directory', 'background-directory-v41.json.zlib'),
    ('halo_rust_background_index_test.go.txt', 'TestHaloRustBackgroundIndex', 'replay', 'background-index', 'background-index-v41.json.zlib'),
    ('halo_rust_map_background_test.go.txt', 'TestHaloRustMapBackground', 'replay', 'map-background', 'map-background-v41.json.zlib'),
    ('halo_rust_map_background_calibration_test.go.txt', 'TestHaloRustMapBackgroundCalibration', 'replay', 'map-background-calibration', 'map-background-calibration-v41.json.zlib'),
    ('halo_rust_callouts_catalog_test.go.txt', 'TestHaloRustCalloutsCatalog', 'replay', 'callouts-catalog', 'callouts-catalog-v41.json.zlib'),
    ('halo_rust_geometry_distance_test.go.txt', 'TestHaloRustGeometryDistanceGrenades', 'replay', 'geometry-distance-grenades', 'geometry-distance-grenades-v41.json.zlib'),
    ('halo_rust_geometry_distance_test.go.txt', 'TestHaloRustGeometryDistance', 'replay', 'geometry-distance', 'geometry-distance-v41.json.zlib'),
    ('halo_rust_geometry_loader_test.go.txt', 'TestHaloRustGeometryLoader', 'replay', 'geometry-loader', 'geometry-loader-v41.json.zlib'),
    ('halo_rust_context_harvest_test.go.txt', 'TestHaloRustContextHarvest', 'internal/grammar', 'context-harvest', 'context-harvest-v41.json.zlib'),
    ('halo_rust_keyframe_words_test.go.txt', 'TestHaloRustKeyframeWords', 'internal/grammar', 'keyframe-words', 'keyframe-words-v41.json.zlib'),
    ('halo_rust_keyframe_layout_domain_test.go.txt', 'TestHaloRustKeyframeLayoutDomain', 'internal/grammar', 'keyframe-layout-domain', 'keyframe-layout-domain-v41.json.zlib'),
    ('halo_rust_context_keyframe_test.go.txt', 'TestHaloRustContextKeyframe', 'internal/grammar', 'context-keyframe', 'context-keyframe-v41.json.zlib'),
    ('halo_rust_context_keyframe_test.go.txt', 'TestHaloRustSignedKeyframeEntry', 'internal/grammar', 'signed-keyframe-entry', 'signed-keyframe-entry-v41.json.zlib'),
    ('halo_rust_signed_keyframe_chain_test.go.txt', 'TestHaloRustSignedKeyframeChain', 'internal/grammar', 'signed-keyframe-chain', 'signed-keyframe-chain-v41.json.zlib'),
    ('halo_rust_march_rollback_test.go.txt', 'TestHaloRustMarchRollback', 'internal/grammar', 'march-rollback', 'march-rollback-v41.json.zlib'),
    ('halo_rust_signed_wrapper_continuation_test.go.txt', 'TestHaloRustSignedWrapperContinuation', 'internal/grammar', 'signed-wrapper-continuation', 'signed-wrapper-continuation-v41.json.zlib'),
    ('halo_rust_queue_unused_widths_test.go.txt', 'TestHaloRustQueueUnusedWidths', 'internal/grammar', 'queue-unused-widths', 'queue-unused-widths-v41.json.zlib'),
    ('halo_rust_signed_march_entry_test.go.txt', 'TestHaloRustSignedMarchEntry', 'internal/grammar', 'signed-march-entry', 'signed-march-entry-v41.json.zlib'),
    ('halo_rust_signed_view_continuation_test.go.txt', 'TestHaloRustSignedViewContinuation', 'internal/grammar', 'signed-view-continuation', 'signed-view-continuation-v41.json.zlib'),
    ('halo_rust_signed_view_readers_test.go.txt', 'TestHaloRustSignedViewReaders', 'internal/grammar', 'signed-view-readers', 'signed-view-readers-v41.json.zlib'),
    ('halo_rust_signed_views_test.go.txt', 'TestHaloRustSignedViews', 'internal/grammar', 'signed-views', 'signed-views-v41.json.zlib'),
    ('halo_rust_signed_harvest_successor_test.go.txt', 'TestHaloRustSignedHarvestSuccessor', 'internal/grammar', 'signed-harvest-successor', 'signed-harvest-successor-v41.json.zlib'),
    ('halo_rust_signed_resync_frame_test.go.txt', 'TestHaloRustSignedResyncFrame', 'internal/grammar', 'signed-resync-frame', 'signed-resync-frame-v41.json.zlib'),
    ('halo_rust_keyframe_frame_bits_test.go.txt', 'TestHaloRustKeyframeFrameBits', 'internal/grammar', 'keyframe-frame-bits', 'keyframe-frame-bits-v41.json.zlib'),
    ('halo_rust_parent_result_selection_test.go.txt', 'TestHaloRustParentResultSelection', 'internal/grammar', 'parent-result-selection', 'parent-result-selection-v41.json.zlib'),
    ('halo_rust_dead_result_retention_test.go.txt', 'TestHaloRustDeadResultRetention', 'internal/grammar', 'dead-result-retention', 'dead-result-retention-v41.json.zlib'),
    ('halo_rust_component_result_contract_test.go.txt', 'TestHaloRustComponentResultContract', 'internal/grammar', 'component-result-contract', 'component-result-contract-v41.json.zlib'),
    ('halo_rust_component_mask_wrap_test.go.txt', 'TestHaloRustComponentMaskWrap', 'internal/grammar', 'component-mask-wrap', 'component-mask-wrap-v41.json.zlib'),
    ('halo_rust_overlapping_payloads_test.go.txt', 'TestHaloRustOverlappingPayloads', 'internal/grammar', 'overlapping-payloads', 'overlapping-payloads-v41.json.zlib'),
    ('halo_rust_isolation_gap_domain_test.go.txt', 'TestHaloRustIsolationGapDomain', 'internal/grammar', 'isolation-gap-domain', 'isolation-gap-domain-v41.json.zlib'),
    ('halo_rust_signed_validated_resync_test.go.txt', 'TestHaloRustSignedValidatedResync', 'internal/grammar', 'signed-validated-resync', 'signed-validated-resync-v41.json.zlib'),
    ('halo_rust_signed_resync_start_test.go.txt', 'TestHaloRustSignedResyncStart', 'internal/grammar', 'signed-resync-start', 'signed-resync-start-v41.json.zlib'),
    ('halo_rust_signed_inference_trial_test.go.txt', 'TestHaloRustSignedInferenceTrial', 'internal/grammar', 'signed-inference-trial', 'signed-inference-trial-v41.json.zlib'),
    ('halo_rust_signed_inference_entry_test.go.txt', 'TestHaloRustSignedInferenceEntry', 'internal/grammar', 'signed-inference-entry', 'signed-inference-entry-v41.json.zlib'),
    ('halo_rust_position_domain_test.go.txt', 'TestHaloRustPositionDomain', 'internal/grammar', 'position-domain', 'position-domain-v41.json.zlib'),
    ('halo_rust_mpp_domain_test.go.txt', 'TestHaloRustMPPDomain', 'internal/grammar', 'mpp-domain', 'mpp-domain-v41.json.zlib'),
    ('halo_rust_signed_header_test.go.txt', 'TestHaloRustSignedHeader', 'internal/grammar', 'signed-header', 'signed-header-v41.json.zlib'),
    ('halo_rust_frame_panic_cursor_test.go.txt', 'TestHaloRustFramePanicCursor', 'internal/grammar', 'frame-panic-cursor', 'frame-panic-cursor-v41.json.zlib'),
    ('halo_rust_signed_component_cursor_test.go.txt', 'TestHaloRustSignedComponentCursor', 'internal/grammar', 'signed-component-cursor', 'signed-component-cursor-v41.json.zlib'),
    ('halo_rust_signed_continuation_test.go.txt', 'TestHaloRustSignedContinuation', 'internal/grammar', 'signed-continuation', 'signed-continuation-v41.json.zlib'),
    ('halo_rust_kill_provenance_test.go.txt', 'TestHaloRustKillProvenance', 'internal/facts/killsource', 'kill-provenance', 'kill-provenance-v41.json'),
    ('halo_rust_resync_cursor_test.go.txt', 'TestHaloRustResyncCursor', 'internal/grammar', 'resync-cursor', 'resync-cursor-v41.json.zlib'),
    ('halo_rust_resync_guard_test.go.txt', 'TestHaloRustResyncGuard', 'internal/grammar', 'resync-guard', 'resync-guard-v41.json.zlib'),
    ('halo_rust_resync_mutation_test.go.txt', 'TestHaloRustResyncMutation', 'internal/grammar', 'resync-mutation', 'resync-mutation-v41.json.zlib'),
    ('halo_rust_reader_quantization_test.go.txt', 'TestHaloRustReaderQuantization', 'internal/grammar', 'reader-quantization', 'reader-quantization-v41.json.zlib'),
    ('halo_rust_reader_cursor_test.go.txt', 'TestHaloRustReaderCursor', 'internal/grammar', 'reader-cursor', 'reader-cursor-v41.json.zlib'),
    ('halo_rust_public_constants_test.go.txt', 'TestHaloRustPublicConstants', 'decfilm', 'public-constants', 'public-constants-v41.json'),
    ('halo_rust_kill_health_test.go.txt', 'TestHaloRustKillHealth', 'internal/facts/killsource', 'kill-health', 'kill-health-methods-v41.json.zlib'),
    ('halo_rust_kill_options_test.go.txt', 'TestHaloRustKillOptions', 'internal/facts/killsource', 'kill-options', 'kill-options-v41.json.zlib'),
    ('halo_rust_kill_registry_test.go.txt', 'TestHaloRustKillRegistry', 'internal/facts/killsource', 'kill-registry', 'kill-registry-v41.json.zlib'),
    ('halo_rust_context_constructor_test.go.txt', 'TestHaloRustContextConstructor', 'internal/grammar', 'context-constructor', 'context-constructor-v41.json.zlib'),
    ('halo_rust_vehicle_coverage_logs_test.go.txt', 'TestHaloRustVehicleCoverageLogs', 'replay', 'vehicle-coverage-logs', 'replay-vehicle-coverage-logs-v41.json.zlib'),
    ('halo_rust_vehicle_source_logs_test.go.txt', 'TestHaloRustVehicleSourceLogs', 'replay', 'vehicle-source-logs', 'vehicle-source-logs-v41.json.zlib'),
    ('halo_rust_profile_warning_test.go.txt', 'TestHaloRustProfileWarning', 'internal/grammar', 'profile-warning', 'profile-warning-v41.json'),
    ('halo_rust_named_life_export_test.go.txt', 'TestHaloRustNamedLifeExport', 'replay', 'named-life-export', 'named-life-export-v41.json.zlib'),
    ('halo_rust_vehicle_death_logs_test.go.txt', 'TestHaloRustVehicleDeathLogs', 'replay', 'vehicle-death-logs', 'vehicle-death-logs-v41.json.zlib'),
    ('halo_rust_roster_strings_test.go.txt', 'TestHaloRustRosterStrings', 'replay', 'roster-strings', 'roster-strings-v41.json'),
    ('halo_rust_navpoint_source_test.go.txt', 'TestHaloRustNavpointSource', 'internal/grammar', 'navpoint-source', 'navpoint-source-v41.json.zlib'),
    ('halo_rust_bridge_warning_test.go.txt', 'TestHaloRustBridgeWarning', 'replay', 'bridge-warning', 'bridge-warning-v41.json.zlib'),
    ('halo_rust_death_context_test.go.txt', 'TestHaloRustDeathContext', 'replay', 'death-context', 'death-context-v41.json.zlib'),
    ('halo_rust_round_identity_queries_test.go.txt', 'TestHaloRustRoundIdentityQueries', 'internal/facts/objectives', 'round-identity-queries', 'round-identity-queries-v41.json.zlib'),
    ('halo_rust_kill_positions_test.go.txt', 'TestHaloRustKillPositions', 'replay', 'kill-positions', 'kill-positions-v41.json.zlib'),
    ('halo_rust_map_entry_test.go.txt', 'TestHaloRustMapEntry', 'internal/grammar', 'map-entry', 'map-entry-v41.json.zlib'),
    ('halo_rust_starting_profile_test.go.txt', 'TestHaloRustStartingProfile', 'internal/facts/killsource', 'starting-profile', 'starting-profile-v41.json'),
)


def write_objective_input_oracle(film, source):
    target = film / "fixtures/objective-inputs-v41.json.zlib"
    target.write_bytes(zlib.compress(source.read_bytes(), 9))
    document = film / "fixtures/full-document-v41.json.zlib"
    rows = json.loads(zlib.decompress(document.read_bytes()))
    inputs = json.loads(source.read_bytes())
    assert [r["folder"] for r in rows] == [r["folder"] for r in inputs]
    for row in rows:
        row["objective_inputs_fixture"] = target.name
    document.write_bytes(zlib.compress(json.dumps(rows, separators=(",", ":")).encode(), 9))


def write_raid_vehicle_oracle(film, source):
    """Attach an independent native vehicle input to the existing raid document."""
    target = film / "fixtures/raid-vehicle-inputs-v41.json.zlib"
    target.write_bytes(zlib.compress(source.read_bytes(), 9))
    document = film / "fixtures/full-raid-document-v41.json.zlib"
    rows = json.loads(zlib.decompress(document.read_bytes()))
    assert len(rows) == 1 and rows[0]["folder"] == "raids/01-hour-long-raid"
    rows[0]["vehicle_inputs_fixture"] = target.name
    document.write_bytes(zlib.compress(json.dumps(rows, separators=(",", ":")).encode(), 9))


def write_full_document_oracles(film, source):
    """Retain full scan inputs separately, so ordinary document checks stay small."""
    rows = json.loads(source.read_bytes())
    for index, row in enumerate(rows):
        inputs = row.pop("inputs")
        assert len(inputs) == 44, "review changed native FilmInputs schema"
        name = f"full-document-inputs-{index}-v41.json.zlib"
        payload = json.dumps(inputs, separators=(",", ":"), ensure_ascii=False).encode()
        (film / "fixtures" / name).write_bytes(zlib.compress(payload, 9))
        row["inputs_fixture"] = name
    payload = json.dumps(rows, separators=(",", ":"), ensure_ascii=False).encode()
    (film / "fixtures/full-document-v41.json.zlib").write_bytes(zlib.compress(payload, 9))


def generate_source_publication_oracles(go, upstream, film, tmp, paths, *, write_fixtures=True):
    """Run retained native harnesses; optionally compare without replacing fixtures."""
    reference = film / "reference"
    instrument = tmp / "instrument_resync_cursor.go"
    instrument.write_text((reference / "instrument_resync_cursor.go.txt").read_text())
    grammar = upstream / "apps/go-api/internal/games/halo_infinite/film/internal/grammar"
    subprocess.run([go, "run", str(instrument), "--", str(grammar / "frame_harvest.go"),
                    str(grammar / "halo_rust_resync_cursor_impl_test.go")], check=True,
                   cwd=upstream / "apps/go-api")
    mapped = dict(paths)
    mapped["/private/tmp/halo-navpoint-primitive.json"] = str(tmp / "navpoint-radial-scan.json")
    for _, _, _, output, _ in SOURCE_PUBLICATION_ORACLES:
        mapped[f"/private/tmp/halo-{output}.json"] = str(tmp / f"{output}.json")
    report = []
    for harness, test, package, output, fixture in SOURCE_PUBLICATION_ORACLES:
        text = (reference / harness).read_text()
        for old, new in mapped.items():
            text = text.replace(old, new)
        destination = upstream / "apps/go-api/internal/games/halo_infinite/film" / package / harness.removesuffix(".txt")
        destination.write_text(text)
        subprocess.run([go, "test", f"./internal/games/halo_infinite/film/{package}",
                        "-run", f"^{test}$", "-count=1", "-timeout", "10m"],
                       cwd=upstream / "apps/go-api", check=True)
        raw = (tmp / f"{output}.json").read_bytes()
        target = film / "fixtures" / fixture
        previous = target.read_bytes()
        if fixture.endswith(".zlib"):
            previous = zlib.decompress(previous)
        unchanged = json.loads(previous) == json.loads(raw)
        report.append({"test": test, "fixture": fixture, "unchanged": unchanged})
        if write_fixtures:
            target.write_bytes(zlib.compress(raw, 9) if fixture.endswith(".zlib") else raw)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("levelup", type=Path)
    parser.add_argument("--go", default="go")
    parser.add_argument("--corpus", required=True, type=Path)
    parser.add_argument("--include-raid", action="store_true", help="also compare the hour-long raid; may take tens of minutes")
    args = parser.parse_args()
    reference = Path(__file__).resolve().parent
    film = reference.parent
    upstream = args.levelup.resolve()
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=upstream, text=True).strip()
    if revision != PIN:
        raise SystemExit(f"Expected LevelUp {PIN}; found {revision}")
    grammar = upstream / "apps/go-api/internal/games/halo_infinite/film/internal/grammar"
    damage_data = upstream / "apps/go-api/internal/games/halo_infinite/film/damagetag/data"
    (reference / "damage-effect-ids-v75.txt").write_bytes((damage_data / "jpt_ids.txt").read_bytes())
    (reference / "damage-effect-labels-v75.tsv").write_bytes((damage_data / "labels.tsv").read_bytes())
    with tempfile.TemporaryDirectory(prefix="halo-parity-") as directory:
        tmp = Path(directory)
        paths = {
            "/private/tmp/halo-reader-sequence.json": str(tmp / "reader-sequence.json"),
            "/private/tmp/halo-creation-source.json": str(tmp / "creation-source.json"),
            "/private/tmp/halo-managed-setup.json": str(tmp / "managed-setup.json"),
            "/private/tmp/halo-navpoint-setup.json": str(tmp / "navpoint-setup.json"),
            "/private/tmp/halo-movement-source.json": str(tmp / "movement-source.json"),
            "/private/tmp/halo-objective-diagnostics.json": str(tmp / "objective-diagnostics.json"),
            "/private/tmp/halo-identity-completion.json": str(tmp / "identity-completion.json"),
            "/private/tmp/halo-damage-catalog.json": str(tmp / "damage-catalog.json"),
            "/private/tmp/halo-packet-files.json": str(tmp / "packet-files.json"),
            "/private/tmp/halo-chunk-bridge.json": str(tmp / "chunk-bridge.json"),
            "/private/tmp/halo-ground-ammo-profile.json": str(tmp / "ground-ammo-profile.json"),
            "/private/tmp/halo-ground-creation-profile.json": str(tmp / "ground-creation-profile.json"),
            "/private/tmp/halo-leaf-readers.json": str(tmp / "leaf-readers.json"),
            "/private/tmp/halo-source-provider.json": str(tmp / "source-provider.json"),
            "/private/tmp/halo-source.json": str(tmp / "source.json"),
            "/private/tmp/halo-source-directory-lifecycle.json": str(tmp / "source-directory-lifecycle.json"),
            "/private/tmp/halo-source-directory.json": str(tmp / "source-directory.json"),
            "/private/tmp/halo-probe-trace-values.json": str(tmp / "probe-trace-values.json"),
            "/private/tmp/halo-captured-payloads.json": str(tmp / "captured-payloads.json"),
            "/private/tmp/halo-component-probe.json": str(tmp / "component-probe.json"),
            "/private/tmp/halo-keyframe-biped-probe.json": str(tmp / "keyframe-biped-probe.json"),
            "/private/tmp/halo-object-hook-repair.json": str(tmp / "object-hook-repair.json"),
            "/private/tmp/halo-managed-hook-repair.json": str(tmp / "managed-hook-repair.json"),
            "/private/tmp/halo-engine-hook-repair.json": str(tmp / "engine-hook-repair.json"),
            "/private/tmp/halo-player-hook-repair.json": str(tmp / "player-hook-repair.json"),
            "/private/tmp/halo-probe-hook-repair.json": str(tmp / "probe-hook-repair.json"),
            "/private/tmp/halo-movement-hook-harvest.json": str(tmp / "movement-hook-harvest.json"),
            "/private/tmp/halo-movement-hook-resync.json": str(tmp / "movement-hook-resync.json"),
            "/private/tmp/halo-movement-hook-chain.json": str(tmp / "movement-hook-chain.json"),
            "/private/tmp/halo-movement-hook-repair.json": str(tmp / "movement-hook-repair.json"),
            "/private/tmp/halo-movement-hook-inference.json": str(tmp / "movement-hook-inference.json"),
            "/private/tmp/halo-movement-hook-recovery-slots.json": str(tmp / "movement-hook-recovery-slots.json"),
            "/private/tmp/halo-movement-hook-generic.json": str(tmp / "movement-hook-generic.json"),
            "/private/tmp/halo-movement-hook-reads.json": str(tmp / "movement-hook-reads.json"),
            "/private/tmp/halo-movement-hook-production.json": str(tmp / "movement-hook-production.json"),
            "/private/tmp/halo-default-hook-reads.json": str(tmp / "default-hook-reads.json"),
            "/private/tmp/halo-default-hook-keyframes.json": str(tmp / "default-hook-keyframes.json"),
            "/private/tmp/halo-default-hook-chain.json": str(tmp / "default-hook-chain.json"),
            "/private/tmp/halo-default-hook-repair.json": str(tmp / "default-hook-repair.json"),
            "/private/tmp/halo-default-hook-frames.json": str(tmp / "default-hook-frames.json"),
            "/private/tmp/halo-equipment-hook-repair.json": str(tmp / "equipment-hook-repair.json"),
            "/private/tmp/halo-object-hook-harvest.json": str(tmp / "object-hook-harvest.json"),
            "/private/tmp/halo-managed-hook-harvest.json": str(tmp / "managed-hook-harvest.json"),
            "/private/tmp/halo-engine-hook-harvest.json": str(tmp / "engine-hook-harvest.json"),
            "/private/tmp/halo-player-hook-harvest.json": str(tmp / "player-hook-harvest.json"),
            "/private/tmp/halo-probe-hook-harvest.json": str(tmp / "probe-hook-harvest.json"),
            "/private/tmp/halo-equipment-hook-harvest.json": str(tmp / "equipment-hook-harvest.json"),
            "/private/tmp/halo-object-hook-resync.json": str(tmp / "object-hook-resync.json"),
            "/private/tmp/halo-managed-hook-resync.json": str(tmp / "managed-hook-resync.json"),
            "/private/tmp/halo-engine-hook-resync.json": str(tmp / "engine-hook-resync.json"),
            "/private/tmp/halo-player-hook-resync.json": str(tmp / "player-hook-resync.json"),
            "/private/tmp/halo-probe-hook-resync.json": str(tmp / "probe-hook-resync.json"),
            "/private/tmp/halo-equipment-hook-resync.json": str(tmp / "equipment-hook-resync.json"),
            "/private/tmp/halo-object-hook-chain.json": str(tmp / "object-hook-chain.json"),
            "/private/tmp/halo-managed-hook-chain.json": str(tmp / "managed-hook-chain.json"),
            "/private/tmp/halo-engine-hook-chain.json": str(tmp / "engine-hook-chain.json"),
            "/private/tmp/halo-player-hook-chain.json": str(tmp / "player-hook-chain.json"),
            "/private/tmp/halo-probe-hook-chain.json": str(tmp / "probe-hook-chain.json"),
            "/private/tmp/halo-equipment-hook-chain.json": str(tmp / "equipment-hook-chain.json"),
            "/private/tmp/halo-object-hook-reads.json": str(tmp / "object-hook-reads.json"),
            "/private/tmp/halo-managed-hook-reads.json": str(tmp / "managed-hook-reads.json"),
            "/private/tmp/halo-engine-hook-reads.json": str(tmp / "engine-hook-reads.json"),
            "/private/tmp/halo-player-hook-reads.json": str(tmp / "player-hook-reads.json"),
            "/private/tmp/halo-probe-hook-reads.json": str(tmp / "probe-hook-reads.json"),
            "/private/tmp/halo-equipment-hook-reads.json": str(tmp / "equipment-hook-reads.json"),
            "/private/tmp/halo-ability-hook-repair.json": str(tmp / "ability-hook-repair.json"),
            "/private/tmp/halo-ability-hook-harvest.json": str(tmp / "ability-hook-harvest.json"),
            "/private/tmp/halo-ability-hook-resync.json": str(tmp / "ability-hook-resync.json"),
            "/private/tmp/halo-ability-hook-chain.json": str(tmp / "ability-hook-chain.json"),
            "/private/tmp/halo-ability-hook-reads.json": str(tmp / "ability-hook-reads.json"),
            "/private/tmp/halo-component-hook-repair.json": str(tmp / "component-hook-repair.json"),
            "/private/tmp/halo-component-hook-harvest.json": str(tmp / "component-hook-harvest.json"),
            "/private/tmp/halo-component-hook-resync.json": str(tmp / "component-hook-resync.json"),
            "/private/tmp/halo-component-hook-chain.json": str(tmp / "component-hook-chain.json"),
            "/private/tmp/halo-unit-reference-hook-reads.json": str(tmp / "unit-reference-hook-reads.json"),
            "/private/tmp/halo-unit-reference-hook-chain.json": str(tmp / "unit-reference-hook-chain.json"),
            "/private/tmp/halo-unit-reference-hook-harvest.json": str(tmp / "unit-reference-hook-harvest.json"),
            "/private/tmp/halo-unit-reference-hook-resync.json": str(tmp / "unit-reference-hook-resync.json"),
            "/private/tmp/halo-unit-reference-hook-repair.json": str(tmp / "unit-reference-hook-repair.json"),
            "/private/tmp/halo-component-hook-reads.json": str(tmp / "component-hook-reads.json"),
            "/private/tmp/halo-mobility-repair.json": str(tmp / "mobility-repair.json"),
            "/private/tmp/halo-mobility-harvest.json": str(tmp / "mobility-harvest.json"),
            "/private/tmp/halo-mobility-resync.json": str(tmp / "mobility-resync.json"),
            "/private/tmp/halo-mobility-chain.json": str(tmp / "mobility-chain.json"),
            "/private/tmp/halo-mobility-reads.json": str(tmp / "mobility-reads.json"),
            "/private/tmp/halo-registry-edges.json": str(tmp / "registry-edges.json"),
            "/private/tmp/halo-registry-warning.json": str(tmp / "registry-warning.json"),
            "/private/tmp/halo-unknown-build.json": str(tmp / "unknown-build.json"),
            "/private/tmp/halo-raw-resync-diagnostics.json": str(tmp / "raw-resync-diagnostics.json"),
            "/private/tmp/halo-harvest-diagnostics.json": str(tmp / "harvest-diagnostics.json"),
            "/private/tmp/halo-repair-diagnostics.json": str(tmp / "repair-diagnostics.json"),
            "/private/tmp/halo-resync-diagnostics.json": str(tmp / "resync-diagnostics.json"),

            "/private/tmp/halo-frame-diagnostics.json": str(tmp / "frame-diagnostics.json"),
            "/private/tmp/halo-read-diagnostics.json": str(tmp / "read-diagnostics.json"),
            "/private/tmp/halo-chain-diagnostics.json": str(tmp / "chain-diagnostics.json"),

            "/private/tmp/halo-weapon-patterns.json": str(tmp / "weapon-patterns.json"),
            "/private/tmp/halo-weapon-patterns-corpus.json": str(tmp / "weapon-patterns-corpus.json"),
            "/private/tmp/halo-filmshell-catalog.json": str(tmp / "filmshell-catalog.json"),

            "/private/tmp/halo-keyframe-position-corpus.json": str(tmp / "keyframe-position-corpus.json"),
            "/private/tmp/halo-keyframe-position-probe.json": str(tmp / "keyframe-position-probe.json"),
            "/private/tmp/halo-unit-equipment-corpus.json": str(tmp / "unit-equipment-corpus.json"),
            "/private/tmp/halo-unit-equipment.json": str(tmp / "unit-equipment.json"),
            "/private/tmp/halo-keyframe-references.json": str(tmp / "keyframe-references.json"),
            "/private/tmp/halo-default-references.json": str(tmp / "default-references.json"),
            "/private/tmp/halo-unit-references.json": str(tmp / "unit-references.json"),
            "/private/tmp/halo-position-hook-sequence.json": str(tmp / "position-hook-sequence.json"),
            "/private/tmp/halo-position-hook-resolved.json": str(tmp / "position-hook-resolved.json"),
            "/private/tmp/halo-position-hook-production.json": str(tmp / "position-hook-production.json"),
            "/private/tmp/halo-position-hook-inference.json": str(tmp / "position-hook-inference.json"),
            "/private/tmp/halo-position-hook-march.json": str(tmp / "position-hook-march.json"),
            "/private/tmp/halo-movement-hook-march.json": str(tmp / "movement-hook-march.json"),
            "/private/tmp/halo-position-hook-locator.json": str(tmp / "position-hook-locator.json"),
            "/private/tmp/halo-body-profile-reads.json": str(tmp / "body-profile-reads.json"),
            "/private/tmp/halo-body-profile-generic.json": str(tmp / "body-profile-generic.json"),
            "/private/tmp/halo-body-profile-inference.json": str(tmp / "body-profile-inference.json"),
            "/private/tmp/halo-calibrated-position-sequence.json": str(tmp / "calibrated-position-sequence.json"),
            "/private/tmp/halo-calibrated-position-generic.json": str(tmp / "calibrated-position-generic.json"),
            "/private/tmp/halo-calibrated-position-inference.json": str(tmp / "calibrated-position-inference.json"),
            "/private/tmp/halo-calibrated-position-keyframes.json": str(tmp / "calibrated-position-keyframes.json"),
            "/private/tmp/halo-new-record-profile-generic.json": str(tmp / "new-record-profile-generic.json"),
            "/private/tmp/halo-new-record-profile-inference.json": str(tmp / "new-record-profile-inference.json"),
            "/private/tmp/halo-new-record-profile-recovery.json": str(tmp / "new-record-profile-recovery.json"),
            "/private/tmp/halo-width-override-generic.json": str(tmp / "width-override-generic.json"),
            "/private/tmp/halo-width-override-inference.json": str(tmp / "width-override-inference.json"),
            "/private/tmp/halo-width-override-keyframes.json": str(tmp / "width-override-keyframes.json"),
            "/private/tmp/halo-width-override-repair.json": str(tmp / "width-override-repair.json"),
            "/private/tmp/halo-baseline-scope-sequence.json": str(tmp / "baseline-scope-sequence.json"),
            "/private/tmp/halo-baseline-scope-generic.json": str(tmp / "baseline-scope-generic.json"),
            "/private/tmp/halo-baseline-scope-inference.json": str(tmp / "baseline-scope-inference.json"),
            "/private/tmp/halo-baseline-scope-keyframes.json": str(tmp / "baseline-scope-keyframes.json"),
            "/private/tmp/halo-registry-catalog.json": str(tmp / "registry-catalog.json"),
            "/private/tmp/halo-film-key.json": str(tmp / "film-key.json"),
            "/private/tmp/halo-film-key-base.bin": str(tmp / "film-key-base.bin"),
            "/private/tmp/halo-kill-icons.json": str(tmp / "kill-icons.json"),
            "/private/tmp/halo-medal-names.json": str(tmp / "medal-names.json"),
            "/private/tmp/halo-keyframe-native-table.json": str(tmp / "keyframe-native-table.json"),
            "/private/tmp/halo-corruption-source.json": str(tmp / "corruption-source.json"),
            "/private/tmp/halo-corruption-context.json": str(tmp / "corruption-context.json"),
            "/private/tmp/halo-keyframe-chain.json": str(tmp / "keyframe-chain.json"),
            "/private/tmp/halo-keyframe-type-word.json": str(tmp / "keyframe-type-word.json"),
            "/private/tmp/halo-keyframe-layout.json": str(tmp / "keyframe-layout.json"),
            "/private/tmp/halo-keyframe-simulation-policy.json": str(tmp / "keyframe-simulation-policy.json"),
            "/private/tmp/halo-position-accumulator-shared.json": str(tmp / "position-accumulator-shared.json"),
            "/private/tmp/halo-position-accumulator-generic.json": str(tmp / "position-accumulator-generic.json"),
            "/private/tmp/halo-position-hook-generic.json": str(tmp / "position-hook-generic.json"),
            "/private/tmp/halo-position-hook-recovery-slots.json": str(tmp / "position-hook-recovery-slots.json"),
            "/private/tmp/halo-position-hook-keyframes.json": str(tmp / "position-hook-keyframes.json"),
            "/private/tmp/halo-position-hook-harvest.json": str(tmp / "position-hook-harvest.json"),
            "/private/tmp/halo-position-hook-resync.json": str(tmp / "position-hook-resync.json"),
            "/private/tmp/halo-position-hook-chain.json": str(tmp / "position-hook-chain.json"),
            "/private/tmp/halo-position-hook-repair.json": str(tmp / "position-hook-repair.json"),
            "/private/tmp/halo-position-hook-validated-resync.json": str(tmp / "position-hook-validated-resync.json"),
            "/private/tmp/halo-position-capture.json": str(tmp / "position-capture.json"),
            "/private/tmp/halo-lazy-widths.json": str(tmp / "lazy-widths.json"),
            "/private/tmp/halo-delta-width-fallback.json": str(tmp / "delta-width-fallback.json"),
            "/private/tmp/halo-absolute-width-boundaries.json": str(tmp / "absolute-width-boundaries.json"),
            "/private/tmp/halo-frame-resync.json": str(tmp / "frame-resync.json"),
            "/private/tmp/halo-frame-targets.json": str(tmp / "frame-targets.json"),
            "/private/tmp/halo-inference-views.json": str(tmp / "inference-views.json"),
            "/private/tmp/halo-chain-new-repair.json": str(tmp / "chain-new-repair.json"),
            "/private/tmp/halo-chain-new.json": str(tmp / "chain-new.json"),
            "/private/tmp/halo-validated-resync.json": str(tmp / "validated-resync.json"),
            "/private/tmp/halo-inference-frame.json": str(tmp / "inference-frame.json"),
            "/private/tmp/halo-chain-repair.json": str(tmp / "chain-repair.json"),
            "/private/tmp/halo-chain-inference.json": str(tmp / "chain-inference.json"),
            "/private/tmp/halo-precision-law.json": str(tmp / "precision-law.json"),
            "/private/tmp/halo-raid-vehicle-inputs.json": str(tmp / "raid-vehicle-inputs.json"),
            "/private/tmp/halo-full-raid-document.json": str(tmp / "full-raid-document.json"),
            "/private/tmp/halo-i0-values.json": str(tmp / "i0-values.json"),
            "/private/tmp/halo-kill-walk-policy.json": str(tmp / "kill-walk-policy.json"),
            "/private/tmp/halo-kill-walk.json": str(tmp / "kill-walk.json"),
            "/private/tmp/halo-kill-timeline.json": str(tmp / "kill-timeline.json"),
            "/private/tmp/halo-kill-index-motif.json": str(tmp / "kill-index-motif.json"),
            "/private/tmp/halo-kill-film-evidence.json": str(tmp / "kill-film-evidence.json"),
            "/private/tmp/halo-kill-source-scan.json": str(tmp / "kill-source-scan.json"),
            "/private/tmp/halo-kill-bijection.json": str(tmp / "kill-bijection.json"),
            "/private/tmp/halo-kill-roster.json": str(tmp / "kill-roster.json"),
            "/private/tmp/halo-vehicle-creation-profile.json": str(tmp / "vehicle-creation-profile.json"),
            "/private/tmp/halo-equipment-profile.json": str(tmp / "equipment-profile.json"),
            "/private/tmp/halo-keyframe-spans.json": str(tmp / "keyframe-spans.json"),
            "/private/tmp/halo-zero-axis-document.json": str(tmp / "zero-axis-document.json"),
            "/private/tmp/halo-i0-layout-corpus.json": str(tmp / "i0-layout-corpus.json"),
            "/private/tmp/halo-i0-layout.json": str(tmp / "i0-layout.json"),
            "/private/tmp/halo-mobility-extra-large.json": str(tmp / "mobility-extra-large.json"),
            "/private/tmp/halo-mobility-extra-policy.json": str(tmp / "mobility-extra-policy.json"),
            "/private/tmp/halo-movement-positive.json": str(tmp / "movement-positive.json"),
            "/private/tmp/halo-large-width.json": str(tmp / "large-width.json"),
            "/private/tmp/halo-signed-width.json": str(tmp / "signed-width.json"),
            "/private/tmp/halo-observer-copy.json": str(tmp / "observer-copy.json"),
            "/private/tmp/halo-context-repair-lazy-widths.json": str(tmp / "context-repair-lazy-widths.json"),
            "/private/tmp/halo-context-repair.json": str(tmp / "context-repair.json"),
            "/private/tmp/halo-context-raw-resync.json": str(tmp / "context-raw-resync.json"),
            "/private/tmp/halo-context-resync-frame.json": str(tmp / "context-resync-frame.json"),
            "/private/tmp/halo-context-resync-lazy-widths.json": str(tmp / "context-resync-lazy-widths.json"),
            "/private/tmp/halo-context-resync.json": str(tmp / "context-resync.json"),
            "/private/tmp/halo-inventory-packet-source.json": str(tmp / "inventory-packet-source.json"),
            "/private/tmp/halo-equipment-recovery-source.json": str(tmp / "equipment-recovery-source.json"),
            "/private/tmp/halo-live-chain-budget.json": str(tmp / "live-chain-budget.json"),
            "/private/tmp/halo-context-chain-lazy-widths.json": str(tmp / "context-chain-lazy-widths.json"),
            "/private/tmp/halo-context-chain.json": str(tmp / "context-chain.json"),
            "/private/tmp/halo-context-inference-lazy-widths.json": str(tmp / "context-inference-lazy-widths.json"),
            "/private/tmp/halo-context-inference.json": str(tmp / "context-inference.json"),
            "/private/tmp/halo-context-views-lazy-widths.json": str(tmp / "context-views-lazy-widths.json"),
            "/private/tmp/halo-context-views.json": str(tmp / "context-views.json"),
            "/private/tmp/halo-context-frame-lazy-widths.json": str(tmp / "context-frame-lazy-widths.json"),
            "/private/tmp/halo-world-position-widths.json": str(tmp / "world-position-widths.json"),
            "/private/tmp/halo-new-tail-large.json": str(tmp / "new-tail-large.json"),
            "/private/tmp/halo-record-id-policy.json": str(tmp / "record-id-policy.json"),
            "/private/tmp/halo-mpp-width-policy.json": str(tmp / "mpp-width-policy.json"),
            "/private/tmp/halo-new-default-policy.json": str(tmp / "new-default-policy.json"),
            "/private/tmp/halo-new-tail-policy.json": str(tmp / "new-tail-policy.json"),
            "/private/tmp/halo-context-frame.json": str(tmp / "context-frame.json"),
            "/private/tmp/halo-live-observer.json": str(tmp / "live-observer.json"),
            "/private/tmp/halo-signed-variable-reader.json": str(tmp / "signed-variable-reader.json"),
            "/private/tmp/halo-context-reader-sequence.json": str(tmp / "context-reader-sequence.json"),
            "/private/tmp/halo-scan-profile-sequence.json": str(tmp / "scan-profile-sequence.json"),
            "/private/tmp/halo-profile-source.json": str(tmp / "profile-source.json"),
            "/private/tmp/halo-context-cache.json": str(tmp / "context-cache.json"),
            "/private/tmp/halo-i0-loaded.json": str(tmp / "i0-loaded.json"),
            "/private/tmp/halo-world-precision.json": str(tmp / "world-precision.json"),
            "/private/tmp/halo-weapon-helpers.json": str(tmp / "weapon-helpers.json"),
            "/private/tmp/halo-weapon-hit-corpus.json": str(tmp / "weapon-hit-corpus.json"),
            "/private/tmp/halo-weapon-hit-scan.json": str(tmp / "weapon-hit-scan.json"),
            "/private/tmp/halo-weapon-hits.json": str(tmp / "weapon-hits.json"),
            "/private/tmp/halo-flag-grab-bridge.json": str(tmp / "flag-grab-bridge.json"),
            "/private/tmp/halo-usage.json": str(tmp / "usage.json"),
            "/private/tmp/halo-usage-full-document.json": str(tmp / "full-document.json"),
            "/private/tmp/halo-usage-decoded-kill-document.json": str(tmp / "decoded-kill-document.json"),
            "/private/tmp/halo-fallback.json": str(tmp / "fallback.json"),
            "/private/tmp/halo-objective-corpus.json": str(tmp / "objective-corpus.json"),
            "/private/tmp/halo-objective-extract.json": str(tmp / "objective-extract.json"),
            "/private/tmp/halo-statborg-awards.json": str(tmp / "statborg-awards.json"),
            "/private/tmp/halo-flag-grabs-net.json": str(tmp / "flag-grabs-net.json"),
            "/private/tmp/halo-decoded-kill-document.json": str(tmp / "decoded-kill-document.json"),
            "/private/tmp/halo-replay-kill-inputs.json": str(tmp / "replay-kill-inputs.json"),
            "/private/tmp/halo-kill-decode.json": str(tmp / "kill-decode.json"),
            "/private/tmp/halo-kill-matching.json": str(tmp / "kill-matching.json"),
            "/private/tmp/halo-kill-feed-pairs.json": str(tmp / "kill-feed-pairs.json"),
            "/private/tmp/halo-kill-feed.json": str(tmp / "kill-feed.json"),
            "/private/tmp/halo-kill-event-scan.json": str(tmp / "kill-event-scan.json"),
            "/private/tmp/halo-kill-assists.json": str(tmp / "kill-assists.json"),
            "/private/tmp/halo-kill-event-chain.json": str(tmp / "kill-event-chain.json"),
            "/private/tmp/halo-objective-scan.json": str(tmp / "objective-scan.json"),
            "/private/tmp/halo-march-loaded.json": str(tmp / "march-loaded.json"),
            "/private/tmp/halo-keyframe-closure.json": str(tmp / "keyframe-closure.json"),
            "/private/tmp/halo-equipment-calibration-profile.json": str(tmp / "equipment-calibration-profile.json"),
            "/private/tmp/halo-creation-world-profile.json": str(tmp / "creation-world-profile.json"),
            "/private/tmp/halo-mpp-resolution.json": str(tmp / "mpp-resolution.json"),
            "/private/tmp/halo-unknown-format-document.json": str(tmp / "unknown-format-document.json"),
            "/private/tmp/halo-missing-identity-document.json": str(tmp / "missing-identity-document.json"),
            "/private/tmp/halo-ctf-geometry-document.json": str(tmp / "ctf-geometry-document.json"),
            "/private/tmp/halo-ctf-decoded-kill-document.json": str(tmp / "ctf-decoded-kill-document.json"),
            "/private/tmp/halo-ctf-document.json": str(tmp / "ctf-document.json"),
            "/private/tmp/halo-objective-inputs.json": str(tmp / "objective-inputs.json"),
            "/private/tmp/halo-full-document.json": str(tmp / "full-document.json"),
            "/private/tmp/halo-neutral-deaths.json": str(tmp / "neutral-deaths.json"),
            "/private/tmp/halo-layers.json": str(tmp / "layers.json"),
            "/private/tmp/halo-weapon-labels.json": str(tmp / "weapon-labels.json"),
            "/private/tmp/halo-weapon-family.json": str(tmp / "weapon-family.json"),
            "/private/tmp/halo-levelup-v75/data/titles/halo_infinite/reference/map_weapon_pads.json": str(upstream / "data/titles/halo_infinite/reference/map_weapon_pads.json"),
            "/private/tmp/halo-map-weapon-pads.json": str(tmp / "map-weapon-pads.json"),
            "/private/tmp/halo-decoder-coverage.json": str(tmp / "decoder-coverage.json"),
            "/private/tmp/halo-t0.json": str(tmp / "t0.json"),
            "/private/tmp/halo-document-schema.json": str(tmp / "document-schema.json"),
            "/private/tmp/halo-stances.json": str(tmp / "stances.json"),
            "/private/tmp/halo-document-content.json": str(tmp / "document-content.json"),
            "/private/tmp/halo-objective-modes.json": str(tmp / "objective-modes.json"),
            "/private/tmp/halo-objective-modes-table.json": str(tmp / "objective-modes-table.json"),
            "/private/tmp/halo-objective-map-selection.json": str(tmp / "objective-map-selection.json"),
            "/private/tmp/halo-zones.json": str(tmp / "zones.json"),
            "/private/tmp/halo-zone-hills.json": str(tmp / "zone-hills.json"),
            "/private/tmp/halo-zone-owner-layer.json": str(tmp / "zone-owner-layer.json"),
            "/private/tmp/halo-zone-owners.json": str(tmp / "zone-owners.json"),
            "/private/tmp/halo-zone-pairing.json": str(tmp / "zone-pairing.json"),
            "/private/tmp/halo-zone-attribution.json": str(tmp / "zone-attribution.json"),
            "/private/tmp/halo-zone-series.json": str(tmp / "zone-series.json"),
            "/private/tmp/halo-map-objectives.json": str(tmp / "map-objectives.json"),
            "/private/tmp/halo-capture-bursts.json": str(tmp / "capture-bursts.json"),
            "/private/tmp/halo-managed-property-corpus.json": str(tmp / "managed-property-corpus.json"),
            "/private/tmp/halo-managed-property.json": str(tmp / "managed-property.json"),
            "/private/tmp/halo-managed-primitive.json": str(tmp / "managed-property.json"),
            "/private/tmp/halo-managed-source.json": str(tmp / "managed-source.json"),
            "/private/tmp/halo-replay-flag-gauges.json": str(tmp / "replay-flag-gauges.json"),
            "/private/tmp/halo-replay-flag-geometry.json": str(tmp / "replay-flag-geometry.json"),
            "/private/tmp/halo-replay-flag-carries.json": str(tmp / "replay-flag-carries.json"),
            "/private/tmp/halo-replay-objective-actions.json": str(tmp / "replay-objective-actions.json"),
            "/private/tmp/halo-biped-remaining-context.json": str(tmp / "biped-remaining-context.json"),
            "/private/tmp/halo-cache-writer.json": str(tmp / "cache-writer.json"),
            "/private/tmp/halo-cache-paths.json": str(tmp / "cache-paths.json"),
            "/private/tmp/halo-map-catalog.json": str(tmp / "map-catalog.json"),
            "/private/tmp/halo-map-rounding.json": str(tmp / "map-rounding.json"),
            "/private/tmp/halo-translocator-padding.json": str(tmp / "translocator-padding.json"),
            "/private/tmp/halo-translocator-source.json": str(tmp / "translocator-source.json"),
            "/private/tmp/halo-equipment-spawn-source.json": str(tmp / "equipment-spawn-source.json"),
            "/private/tmp/halo-pickup-padding.json": str(tmp / "pickup-padding.json"),
            "/private/tmp/halo-zoom-padding.json": str(tmp / "zoom-padding.json"),
            "/private/tmp/halo-map-names.json": str(tmp / "map-names.json"),
            "/private/tmp/halo-weapon-track-range.json": str(tmp / "weapon-track-range.json"),
            "/private/tmp/halo-weapon-range.json": str(tmp / "weapon-range.json"),
            "/private/tmp/halo-registry-result.json": str(tmp / "registry-result.json"),
            "/private/tmp/halo-cache-reader.json": str(tmp / "cache-reader.json"),
            "/private/tmp/halo-cache-partial-metadata.json": str(tmp / "cache-partial-metadata.json"),
            "/private/tmp/halo-biped-context.json": str(tmp / "biped-context.json"),
            "/private/tmp/halo-navpoint-radial-scan.json": str(tmp / "navpoint-radial-scan.json"),
            "/private/tmp/halo-bomb-start-fallback.json": str(tmp / "bomb-start-fallback.json"),
            "/private/tmp/halo-replay-bomb-armings.json": str(tmp / "replay-bomb-armings.json"),
            "/private/tmp/halo-replay-bomb-stats.json": str(tmp / "replay-bomb-stats.json"),
            "/private/tmp/halo-replay-held-object.json": str(tmp / "replay-held-object.json"),
            "/private/tmp/halo-replay-skull.json": str(tmp / "replay-skull.json"),
            "/private/tmp/halo-replay-vip.json": str(tmp / "replay-vip.json"),
            "/private/tmp/halo-carrier-marks.json": str(tmp / "carrier-marks.json"),
            "/private/tmp/halo-replay-score-teams.json": str(tmp / "replay-score-teams.json"),
            "/private/tmp/halo-replay-score-players.json": str(tmp / "replay-score-players.json"),
            "/private/tmp/halo-replay-score-series.json": str(tmp / "replay-score-series.json"),
            "/private/tmp/halo-replay-weapon-changes.json": str(tmp / "replay-weapon-changes.json"),
            "/private/tmp/halo-replay-equipment-catalog.json": str(tmp / "equipment-catalog.json"),
            "/private/tmp/halo-replay-equipment-origin.json": str(tmp / "replay-equipment-origin.json"),
            "/private/tmp/halo-replay-equipment-episodes.json": str(tmp / "replay-equipment-episodes.json"),
            "/private/tmp/halo-replay-grapple.json": str(tmp / "replay-grapple.json"),
            "/private/tmp/halo-replay-translocations.json": str(tmp / "replay-translocations.json"),
            "/private/tmp/halo-replay-equipment-publication.json": str(tmp / "replay-equipment-publication.json"),
            "/private/tmp/halo-replay-inventory-publication.json": str(tmp / "replay-inventory-publication.json"),
            "/private/tmp/halo-replay-ability-catalog.json": str(tmp / "ability-catalog.json"),
            "/private/tmp/halo-replay-ability-charges.json": str(tmp / "replay-ability-charges.json"),
            "/private/tmp/halo-film-raid.json": str(tmp / "film-raid.json"),
            "/private/tmp/halo-vehicle-film-rides.json": str(tmp / "vehicle-film-rides.json"),
            "/private/tmp/halo-replay-vehicle-tracks.json": str(tmp / "replay-vehicle-tracks.json"),
            "/private/tmp/halo-march-calibration.json": str(tmp / "march-calibration.json"),
            "/private/tmp/halo-march-lazy-policy.json": str(tmp / "march-lazy-policy.json"),
            "/private/tmp/halo-march-corpus.json": str(tmp / "march-corpus.json"),
            "/private/tmp/halo-march-walk.json": str(tmp / "march-walk.json"),
            "/private/tmp/halo-march-facts.json": str(tmp / "march-facts.json"),
            "/private/tmp/halo-biped-aim.json": str(tmp / "biped-aim.json"),
            "/private/tmp/halo-vehicle-creation-layout.json": str(tmp / "vehicle-creation-layout.json"),
            "/private/tmp/halo-vehicle-creation-loaded.json": str(tmp / "vehicle-creation-loaded.json"),
            "/private/tmp/halo-quantize.json": str(tmp / "quantize.json"),
            "/private/tmp/halo-endpoint.json": str(tmp / "endpoint.json"),
            "/private/tmp/halo-vehicle-creation-corpus.json": str(tmp / "vehicle-creation-corpus.json"),
            "/private/tmp/halo-vehicle-i0.json": str(tmp / "vehicle-i0.json"),
            "/private/tmp/halo-biped-aim-loaded.json": str(tmp / "biped-aim-loaded.json"),
            "/private/tmp/halo-vehicle-events.json": str(tmp / "vehicle-events.json"),
            "/private/tmp/halo-vehicle-creations.json": str(tmp / "vehicle-creations.json"),
            "/private/tmp/halo-vehicle-scan.json": str(tmp / "vehicle-scan.json"),
            "/private/tmp/halo-orientation.json": str(tmp / "orientation.json"),
            "/private/tmp/halo-velocity-codec.json": str(tmp / "velocity-codec.json"),
            "/private/tmp/halo-offline-aim-helpers.json": str(tmp / "offline-aim-helpers.json"),
            "/private/tmp/halo-channel-independence.json": str(tmp / "channel-independence.json"),
            "/private/tmp/halo-biped-source-availability.json": str(tmp / "biped-source-availability.json"),
            "/private/tmp/halo-inventory-source-availability.json": str(tmp / "inventory-source-availability.json"),
            "/private/tmp/halo-replay-vehicle-relays.json": str(tmp / "replay-vehicle-relays.json"),
            "/private/tmp/halo-replay-vehicle-lives.json": str(tmp / "replay-vehicle-lives.json"),
            "/private/tmp/halo-replay-vehicle-shots.json": str(tmp / "replay-vehicle-shots.json"),
            "/private/tmp/halo-replay-grenade-reads.json": str(tmp / "replay-grenade-reads.json"),
            "/private/tmp/halo-replay-grenades.json": str(tmp / "replay-grenades.json"),
            "/private/tmp/halo-replay-projectiles.json": str(tmp / "replay-projectiles.json"),
            "/private/tmp/halo-replay-loadouts.json": str(tmp / "replay-loadouts.json"),
            "/private/tmp/halo-replay-shots.json": str(tmp / "replay-shots.json"),
            "/private/tmp/halo-film-players.json": str(tmp / "film-players.json"),
            "/private/tmp/halo-replay-scope.json": str(tmp / "replay-scope.json"),
            "/private/tmp/halo-replay-clock.json": str(tmp / "replay-clock.json"),
            "/private/tmp/halo-replay-evidence.json": str(tmp / "replay-evidence.json"),
            "/private/tmp/halo-highlights-corpus.json": str(tmp / "highlights-corpus.json"),
            "/private/tmp/halo-highlights.json": str(tmp / "highlights.json"),
            "/private/tmp/halo-player-teams-corpus.json": str(tmp / "player-teams-corpus.json"),
            "/private/tmp/halo-replay-players.json": str(tmp / "replay-players.json"),
            "/private/tmp/halo-replay-seats.json": str(tmp / "replay-seats.json"),
            "/private/tmp/halo-replay-teams.json": str(tmp / "replay-teams.json"),
            "/private/tmp/halo-player-teams.json": str(tmp / "player-teams.json"),
            "/private/tmp/halo-replay-bounds.json": str(tmp / "replay-bounds.json"),
            "/private/tmp/halo-replay-tracks.json": str(tmp / "replay-tracks.json"),
            "/private/tmp/halo-identity-successions.json": str(tmp / "identity-successions.json"),
            "/private/tmp/halo-identity-remaining.json": str(tmp / "identity-remaining.json"),
            "/private/tmp/halo-statborg-named.json": str(tmp / "statborg-named.json"),
            "/private/tmp/halo-statborg-residue.json": str(tmp / "statborg-residue.json"),
            "/private/tmp/halo-statborg-identity.json": str(tmp / "statborg-identity.json"),
            "/private/tmp/halo-statborg-series.json": str(tmp / "statborg-series.json"),
            "/private/tmp/halo-statborg-corpus.json": str(tmp / "statborg-corpus.json"),
            "/private/tmp/halo-statborg-decode.json": str(tmp / "statborg-decode.json"),
            "/private/tmp/halo-statborg-rounds.json": str(tmp / "statborg-rounds.json"),
            "/private/tmp/halo-identity-registry.json": str(tmp / "identity-registry.json"),
            "/private/tmp/halo-identity-owners.json": str(tmp / "identity-owners.json"),
            "/private/tmp/halo-identity-closures.json": str(tmp / "identity-closures.json"),
            "/private/tmp/halo-bot-corpus.json": str(tmp / "bot-corpus.json"),
            "/private/tmp/halo-bot-metadata.json": str(tmp / "bot-metadata.json"),
            "/private/tmp/halo-identity-scoreboard.json": str(tmp / "identity-scoreboard.json"),
            "/private/tmp/halo-identity-death-clock.json": str(tmp / "identity-death-clock.json"),
            "/private/tmp/halo-identity-lifetimes.json": str(tmp / "identity-lifetimes.json"),
            "/private/tmp/halo-identity-creations.json": str(tmp / "identity-creations.json"),
            "/private/tmp/halo-player-indices.json": str(tmp / "player-indices.json"),
            "/private/tmp/halo-identity-tables.json": str(tmp / "identity-tables.json"),
            "/private/tmp/halo-replay-identity.json": str(tmp / "replay-identity.json"),
            "/private/tmp/halo-pickup-origin.json": str(tmp / "pickup-origin.json"),
            "/private/tmp/halo-replay-pickups.json": str(tmp / "replay-pickups.json"),
            "/private/tmp/halo-pad-dating.json": str(tmp / "pad-dating.json"),
            "/private/tmp/halo-ground-pad-corpus.json": str(tmp / "ground-pad-corpus.json"),
            "/private/tmp/halo-ground-pads.json": str(tmp / "ground-pads.json"),
            "/private/tmp/halo-ground-objects-corpus.json": str(tmp / "ground-objects-corpus.json"),
            "/private/tmp/halo-ground-objects.json": str(tmp / "ground-objects.json"),
            "/private/tmp/halo-ground-lifetimes.json": str(tmp / "ground-lifetimes.json"),
            "/private/tmp/halo-research-directory.json": str(tmp / "research-directory.json"),
            "/private/tmp/halo-loadout-sources.json": str(tmp / "loadout-sources.json"),
            "/private/tmp/halo-ground-directory.json": str(tmp / "ground-directory.json"),
            "/private/tmp/halo-ground-keyframes.json": str(tmp / "ground-keyframes.json"),
            "/private/tmp/halo-ground-creations.json": str(tmp / "ground-creations.json"),
            "/private/tmp/halo-equipment-placements.json": str(tmp / "equipment-placements.json"),
            "/private/tmp/halo-equipment-state-corpus.json": str(tmp / "equipment-state-corpus.json"),
            "/private/tmp/halo-equipment-state.json": str(tmp / "equipment-state.json"),
            "/private/tmp/halo-equipment-creations.json": str(tmp / "equipment-creations.json"),
            "/private/tmp/halo-world-census.json": str(tmp / "world-census.json"),
            "/private/tmp/halo-packet-discovery.json": str(tmp / "packet-discovery.json"),
            "/private/tmp/halo-kfq-components.json": str(tmp / "kfq-components.json"),
            "/private/tmp/halo-kfq-corpus.json": str(tmp / "kfq-corpus.json"),
            "/private/tmp/halo-slot-band.json": str(tmp / "slot-band.json"),
            "/private/tmp/halo-budget-exhaustion.json": str(tmp / "budget-exhaustion.json"),
            "/private/tmp/halo-statborg-source-positive.json": str(tmp / "statborg-source-positive.json"),
            "/private/tmp/halo-round-helpers.json": str(tmp / "round-helpers.json"),
            "/private/tmp/halo-objective-source.json": str(tmp / "objective-source.json"),
            "/private/tmp/halo-statborg-source-resolved.json": str(tmp / "statborg-source-resolved.json"),
            "/private/tmp/halo-statborg-source.json": str(tmp / "statborg-source.json"),
            "/private/tmp/halo-statborg-source-limit.json": str(tmp / "statborg-source-limit.json"),
            "/private/tmp/halo-chronology.json": str(tmp / "chronology.json"),
            "/private/tmp/halo-budget-passes.json": str(tmp / "budget-passes.json"),
            "/private/tmp/halo-budget-diagnostics.json": str(tmp / "budget-diagnostics.json"),
            "/private/tmp/halo-stat-components.json": str(tmp / "stat-components.json"),
            "/private/tmp/halo-kill-chunks.json": str(tmp / "kill-chunks.json"),
            "/private/tmp/halo-type-contracts.json": str(tmp / "type-contracts.json"),
            "/private/tmp/halo-source-bits.json": str(tmp / "source-bits.json"),
            "/private/tmp/halo-source-octets.json": str(tmp / "source-octets.json"),
            "/private/tmp/halo-profile-values.json": str(tmp / "profile-values.json"),
            "/private/tmp/halo-profile-table.json": str(tmp / "profile-table.json"),
            "/private/tmp/halo-profile-layouts.json": str(tmp / "profile-layouts.json"),
            "/private/tmp/halo-kfq.json": str(tmp / "kfq.json"),
            "/private/tmp/halo-world-research.json": str(tmp / "world-research.json"),
            "/private/tmp/halo-world-tracks.json": str(tmp / "world-tracks.json"),
            "/private/tmp/halo-grenade.json": str(tmp / "grenade.json"),
            "/private/tmp/halo-fire-scanner.json": str(tmp / "fire-scanner.json"),
            "/private/tmp/halo-fire.json": str(tmp / "fire.json"),
            "/private/tmp/halo-roster.json": str(tmp / "roster.json"),
            "/Users/simbleau/git/halo_api/src/theater/fixtures/bootstrap-v41.zlib": str(film / "fixtures/bootstrap-v41.zlib"),
            "/private/tmp/halo-player-table.json": str(tmp / "player-table.json"),
            "/private/tmp/halo-keyframe-inventory.json": str(tmp / "keyframe-inventory.json"),
            "/private/tmp/halo-weapon-classification.json": str(tmp / "weapon-classification.json"),
            "/private/tmp/halo-weapon-families.json": str(tmp / "weapon-families.json"),
            "/private/tmp/halo-keyframe-loadouts.json": str(tmp / "keyframe-loadouts.json"),
            "/private/tmp/halo-native-default-fallbacks.json": str(tmp / "default-fallbacks.json"),
            "/private/tmp/halo-native-padding.json": str(tmp / "padding.json"),
            "/private/tmp/halo-levelup-jump-derivation.json": str(tmp / "jump-derivation.json"),
            "/private/tmp/halo-levelup-movement-components.json": str(tmp / "movement-components.json"),
            "/private/tmp/halo-levelup-strict-locator.json": str(tmp / "strict-locator.json"),
            "/private/tmp/halo-levelup-production-admission.json": str(tmp / "production-admission.json"),
            "/private/tmp/halo-levelup-bootstrap-oracle.json": str(tmp / "bootstrap.json"),
            "/private/tmp/halo-levelup-components-oracle.json": str(tmp / "components.json"),
            "/private/tmp/halo-components-selected.json": str(tmp / "selected.json"),
            "/private/tmp/halo-levelup-datums-oracle.json": str(tmp / "datums.json"),
            "/private/tmp/halo-levelup-datums.bin": str(tmp / "datums.bin"),
            "/private/tmp/halo-levelup-views-oracle.json": str(tmp / "views.json"),
            "/private/tmp/halo-datum-lengths.json": str(tmp / "datum-lengths.json"),
            "/private/tmp/halo-levelup-defaults-oracle.json": str(tmp / "defaults.json"),
            "/private/tmp/halo-levelup-keyframes-oracle.json": str(tmp / "keyframes.json"),
            "/private/tmp/halo-levelup-frames-oracle.json": str(tmp / "frames.json"),
            "/private/tmp/halo-levelup-captured-keyframe.json": str(tmp / "captured-keyframe.json"),
            "/private/tmp/halo-levelup-captured-keyframe.bin": str(tmp / "captured-keyframe.bin"),
            "/private/tmp/halo-record-mask-hook.json": str(tmp / "record-mask-hook.json"),
            "/private/tmp/halo-levelup-biped-scan-oracle.json": str(tmp / "biped-scan.json"),
            "/private/tmp/halo-levelup-world-oracle.json": str(tmp / "world.json"),
            "/private/tmp/halo-levelup-gameplay-oracle.json": str(tmp / "gameplay.json"),
            "/private/tmp/halo-levelup-translocator-oracle.json": str(tmp / "translocator.json"),
            "/private/tmp/halo-levelup-event-heads-oracle.json": str(tmp / "event-heads.json"),
            "/private/tmp/halo-levelup-profile-oracle.json": str(tmp / "profile.json"),
            "/private/tmp/halo-levelup-v75/data/titles/halo_infinite/reference/map_quant_bounds.json": str(upstream / "data/titles/halo_infinite/reference/map_quant_bounds.json"),
            "/private/tmp/halo-levelup-recovery-oracle.json": str(tmp / "recovery.json"),
            "/private/tmp/halo-keyframe-anchor-bodies.jsonl": str(tmp / "keyframe-anchor-bodies.jsonl"),
            "/private/tmp/halo-keyframe-table-corpus.json": str(tmp / "keyframe-table-corpus.json"),
            "/Users/simbleau/git/halo_api/experiments/films": str(args.corpus.resolve()),
        }
        for name in [
            'halo_rust_padding_test.go',
            'halo_rust_movement_test.go',
            'halo_rust_parity_test.go',
            'halo_rust_components_test.go',
            'halo_rust_datums_test.go',
            'halo_rust_views_test.go',
            'halo_rust_defaults_test.go',
            'halo_rust_keyframes_test.go',
            'halo_rust_frames_test.go',
            'halo_rust_captured_keyframe_test.go',
            'halo_rust_keyframe_table_corpus_test.go',
            'halo_rust_keyframe_anchor_bodies_test.go',
            'halo_rust_recovery_test.go',
            'halo_rust_profile_test.go',
            'halo_rust_event_heads_test.go',
            'halo_rust_translocator_test.go',
            'halo_rust_biped_scan_test.go',
            'halo_rust_record_mask_hook_test.go',
            'halo_rust_world_test.go',
            'halo_rust_loadouts_test.go',
            'halo_rust_player_table_test.go',
            'halo_rust_roster_test.go',
            'halo_rust_fire_test.go',
            'halo_rust_grenade_test.go',
            'halo_rust_world_tracks_test.go',
            'halo_rust_world_research_test.go',
            'halo_rust_kfq_test.go',
            'halo_rust_profile_table_test.go',
            'halo_rust_type_contracts_test.go',
            'halo_rust_source_bits_test.go',
            'halo_rust_source_octets_test.go',
            'halo_rust_profile_values_test.go',
            'halo_rust_slot_band_test.go',
            'halo_rust_kfq_corpus_test.go',
            'halo_rust_kfq_components_test.go',
            'halo_rust_packet_discovery_test.go',
            'halo_rust_equipment_state_corpus_test.go',
            'halo_rust_equipment_state_test.go',
            'halo_rust_equipment_creations_test.go',
            'halo_rust_ground_creations_test.go',
            'halo_rust_research_directory_test.go',
            'halo_rust_loadout_sources_test.go',
            'halo_rust_ground_directory_test.go',
            'halo_rust_ground_keyframes_test.go',
            'halo_rust_equipment_placements_test.go',
            'halo_rust_world_census_test.go',
        ]:
            text = (reference / (name + ".txt")).read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar / name).write_text(text)
        for name in ["march_calibration", "march_corpus", "march_lazy_policy"]:
            text = (reference / f"halo_rust_{name}_test.go.txt").read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar / f"halo_rust_{name}_test.go").write_text(text)
        text = (reference / "halo_rust_march_walk_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_march_walk_test.go").write_text(text)
        text = (reference / "halo_rust_march_facts_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_march_facts_test.go").write_text(text)
        text = (reference / "halo_rust_biped_aim_loaded_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_biped_aim_loaded_test.go").write_text(text)
        text = (reference / "halo_rust_biped_aim_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_biped_aim_test.go").write_text(text)
        text = (reference / "halo_rust_vehicle_events_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_vehicle_events_test.go").write_text(text)
        text = (reference / "halo_rust_vehicle_i0_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_vehicle_i0_test.go").write_text(text)
        text = (reference / "halo_rust_managed_setup_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_managed_setup_test.go").write_text(text)
        text = (reference / "halo_rust_navpoint_setup_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_navpoint_setup_test.go").write_text(text)
        text = (reference / "halo_rust_reader_sequence_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_reader_sequence_test.go").write_text(text)
        text = (reference / "halo_rust_creation_source_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_creation_source_test.go").write_text(text)
        text = (reference / "halo_rust_movement_source_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_movement_source_test.go").write_text(text)
        text = (reference / "halo_rust_quantize_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_quantize_test.go").write_text(text)
        text = (reference / "halo_rust_endpoint_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_endpoint_test.go").write_text(text)
        text = (reference / "halo_rust_vehicle_creation_corpus_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_vehicle_creation_corpus_test.go").write_text(text)
        text = (reference / "halo_rust_vehicle_creation_loaded_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_vehicle_creation_loaded_test.go").write_text(text)
        text = (reference / "halo_rust_vehicle_creation_layout_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_vehicle_creation_layout_test.go").write_text(text)
        text = (reference / "halo_rust_vehicle_creations_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_vehicle_creations_test.go").write_text(text)
        text = (reference / "halo_rust_vehicle_scan_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_vehicle_scan_test.go").write_text(text)
        text = (reference / "halo_rust_orientation_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_orientation_test.go").write_text(text)
        text = (reference / "halo_rust_channel_independence_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_channel_independence_test.go").write_text(text)
        text = (reference / "halo_rust_biped_source_availability_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_biped_source_availability_test.go").write_text(text)
        text = (reference / "halo_rust_inventory_source_availability_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_inventory_source_availability_test.go").write_text(text)
        text = (reference / "halo_rust_gameplay_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_gameplay_test.go").write_text(text)
        text = (reference / "halo_rust_inventory_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_inventory_test.go").write_text(text)
        text = (reference / "halo_rust_ground_lifetimes_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_ground_lifetimes_test.go").write_text(text)
        text = (reference / "halo_rust_ground_objects_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_ground_objects_test.go").write_text(text)
        text = (reference / "halo_rust_ground_objects_corpus_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_ground_objects_corpus_test.go").write_text(text)
        text = (reference / "halo_rust_ground_pads_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_ground_pads_test.go").write_text(text)
        text = (reference / "halo_rust_pad_dating_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_pad_dating_test.go").write_text(text)
        text = (reference / "halo_rust_replay_pickups_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_replay_pickups_test.go").write_text(text)
        text = (reference / "halo_rust_pickup_origin_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_pickup_origin_test.go").write_text(text)
        text = (reference / "halo_rust_replay_identity_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_replay_identity_test.go").write_text(text)
        text = (reference / "halo_rust_identity_tables_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_identity_tables_test.go").write_text(text)
        text = (reference / "halo_rust_player_indices_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_player_indices_test.go").write_text(text)
        text = (reference / "halo_rust_identity_creations_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_identity_creations_test.go").write_text(text)
        text = (reference / "halo_rust_identity_lifetimes_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_identity_lifetimes_test.go").write_text(text)
        text = (reference / "halo_rust_identity_death_clock_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_identity_death_clock_test.go").write_text(text)
        text = (reference / "halo_rust_identity_scoreboard_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_identity_scoreboard_test.go").write_text(text)
        text = (reference / "halo_rust_kill_chunks_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_chunks_test.go").write_text(text)
        text = (reference / "halo_rust_bot_metadata_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_bot_metadata_test.go").write_text(text)
        text = (reference / "halo_rust_bot_corpus_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_bot_corpus_test.go").write_text(text)
        text = (reference / "halo_rust_identity_closures_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_identity_closures_test.go").write_text(text)
        text = (reference / "halo_rust_identity_owners_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_identity_owners_test.go").write_text(text)
        text = (reference / "halo_rust_identity_registry_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_identity_registry_test.go").write_text(text)
        text = (reference / "halo_rust_statborg_rounds_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_statborg_rounds_test.go").write_text(text)
        text = (reference / "halo_rust_statborg_decode_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_statborg_decode_test.go").write_text(text)
        text = (reference / "halo_rust_statborg_corpus_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_statborg_corpus_test.go").write_text(text)
        text = (reference / "halo_rust_statborg_series_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_statborg_series_test.go").write_text(text)
        text = (reference / "halo_rust_statborg_identity_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_statborg_identity_test.go").write_text(text)
        text = (reference / "halo_rust_round_helpers_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_round_helpers_test.go").write_text(text)
        text = (reference / "halo_rust_objective_source_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_objective_source_test.go").write_text(text)
        text = (reference / "halo_rust_statborg_source_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_statborg_source_test.go").write_text(text)
        text = (reference / "halo_rust_chronology_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_chronology_test.go").write_text(text)
        text = (reference / "halo_rust_budget_passes_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_budget_passes_test.go").write_text(text)
        text = (reference / "halo_rust_budget_diagnostics_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_budget_diagnostics_test.go").write_text(text)
        text = (reference / "halo_rust_stat_components_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_stat_components_test.go").write_text(text)
        text = (reference / "halo_rust_statborg_residue_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_statborg_residue_test.go").write_text(text)
        text = (reference / "halo_rust_statborg_named_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_statborg_named_test.go").write_text(text)
        text = (reference / "halo_rust_identity_remaining_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_identity_remaining_test.go").write_text(text)
        text = (reference / "halo_rust_identity_successions_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_identity_successions_test.go").write_text(text)
        text = (reference / "halo_rust_replay_tracks_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_replay_tracks_test.go").write_text(text)
        text = (reference / "halo_rust_replay_bounds_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_replay_bounds_test.go").write_text(text)
        text = (reference / "halo_rust_player_teams_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_player_teams_test.go").write_text(text)
        text = (reference / "halo_rust_player_teams_corpus_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_player_teams_corpus_test.go").write_text(text)
        text = (reference / "halo_rust_replay_teams_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_replay_teams_test.go").write_text(text)
        text = (reference / "halo_rust_replay_seats_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_replay_seats_test.go").write_text(text)
        text = (reference / "halo_rust_replay_players_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_replay_players_test.go").write_text(text)
        text = (reference / "halo_rust_highlights_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_highlights_test.go").write_text(text)
        text = (reference / "halo_rust_highlights_corpus_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_highlights_corpus_test.go").write_text(text)
        for name in ["ctf_document", "full_document", "neutral_deaths", "layers", "weapon_labels", "weapon_family", "map_weapon_pads", "decoder_coverage", "t0", "document_schema", "stances", "document_content", "objective_modes", "zones", "zone_hills", "zone_owner_layer", "zone_owners", "zone_pairing", "zone_attribution", "zone_series", "map_objectives", "replay_flag_gauges", "replay_flag_geometry", "replay_flag_carries", "replay_objective_actions", "replay_bomb_armings", "bomb_start_fallback", "replay_bomb_stats", "replay_held_object", "replay_skull", "replay_vip", "replay_score_teams", "replay_score_players", "replay_score_series", "replay_weapon_changes", "replay_equipment_origin", "replay_equipment_episodes", "replay_grapple", "replay_translocations", "replay_equipment_publication", "replay_inventory_publication", "replay_ability_catalog", "replay_ability_charges", "replay_clock", "replay_evidence", "replay_scope", "film_players", "replay_shots", "replay_loadouts", "replay_projectiles", "replay_grenades", "replay_grenade_reads", "replay_vehicle_shots", "replay_vehicle_lives", "replay_vehicle_relays", "replay_vehicle_tracks", "vehicle_film_rides"]:
            text = (reference / f"halo_rust_{name}_test.go.txt").read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar.parent.parent / f"replay/halo_rust_{name}_test.go").write_text(text)
        text = (reference / "halo_rust_capture_bursts_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_capture_bursts_test.go").write_text(text)
        text = (reference / "halo_rust_managed_property_corpus_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_managed_property_corpus_test.go").write_text(text)
        text = (reference / "halo_rust_managed_property_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_managed_property_test.go").write_text(text)
        text = (reference / "halo_rust_biped_remaining_context_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_biped_remaining_context_test.go").write_text(text)
        text = (reference / "halo_rust_biped_context_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_biped_context_test.go").write_text(text)
        text = (reference / "halo_rust_navpoint_radial_scan_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_navpoint_radial_scan_test.go").write_text(text)
        text = (reference / "halo_rust_carrier_marks_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_carrier_marks_test.go").write_text(text)
        text = (reference / "halo_rust_kill_event_chain_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_event_chain_test.go").write_text(text)
        text = (reference / "halo_rust_kill_assists_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_assists_test.go").write_text(text)
        text = (reference / "halo_rust_kill_event_scan_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_event_scan_test.go").write_text(text)
        text = (reference / "halo_rust_kill_feed_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_feed_test.go").write_text(text)
        text = (reference / "halo_rust_kill_feed_pairs_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_feed_pairs_test.go").write_text(text)
        text = (reference / "halo_rust_kill_roster_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_roster_test.go").write_text(text)
        text = (reference / "halo_rust_kill_bijection_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_bijection_test.go").write_text(text)
        text = (reference / "halo_rust_kill_source_scan_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_source_scan_test.go").write_text(text)
        text = (reference / "halo_rust_kill_index_motif_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_index_motif_test.go").write_text(text)
        text = (reference / "halo_rust_kill_film_evidence_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_film_evidence_test.go").write_text(text)
        text = (reference / "halo_rust_kill_timeline_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_timeline_test.go").write_text(text)
        text = (reference / "halo_rust_keyframe_spans_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_keyframe_spans_test.go").write_text(text)
        text = (reference / "halo_rust_mpp_resolution_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (upstream / "apps/go-api/internal/games/halo_infinite/film/replay/halo_rust_mpp_resolution_test.go").write_text(text)
        text = (reference / "halo_rust_unknown_format_document_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (upstream / "apps/go-api/internal/games/halo_infinite/film/replay/halo_rust_unknown_format_document_test.go").write_text(text)
        text = (reference / "halo_rust_missing_identity_document_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (upstream / "apps/go-api/internal/games/halo_infinite/film/replay/halo_rust_missing_identity_document_test.go").write_text(text)
        text = (reference / "halo_rust_zero_axis_document_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (upstream / "apps/go-api/internal/games/halo_infinite/film/replay/halo_rust_zero_axis_document_test.go").write_text(text)
        text = (reference / "halo_rust_precision_law_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_precision_law_test.go").write_text(text)
        text = (reference / "halo_rust_chain_inference_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_chain_inference_test.go").write_text(text)
        text = (reference / "halo_rust_chain_repair_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_chain_repair_test.go").write_text(text)
        text = (reference / "halo_rust_inference_frame_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_inference_frame_test.go").write_text(text)
        text = (reference / "halo_rust_validated_resync_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_validated_resync_test.go").write_text(text)
        text = (reference / "halo_rust_chain_new_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_chain_new_test.go").write_text(text)
        text = (reference / "halo_rust_chain_new_repair_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_chain_new_repair_test.go").write_text(text)
        text = (reference / "halo_rust_inference_views_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_inference_views_test.go").write_text(text)
        text = (reference / "halo_rust_unit_references_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_unit_references_test.go").write_text(text)
        text = (reference / "halo_rust_default_references_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_default_references_test.go").write_text(text)
        text = (reference / "halo_rust_keyframe_references_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_keyframe_references_test.go").write_text(text)
        text = (reference / "halo_rust_mobility_reads_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_mobility_reads_test.go").write_text(text)
        text = (reference / "halo_rust_mobility_chain_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_mobility_chain_test.go").write_text(text)
        text = (reference / "halo_rust_mobility_resync_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_mobility_resync_test.go").write_text(text)
        text = (reference / "halo_rust_mobility_harvest_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_mobility_harvest_test.go").write_text(text)
        text = (reference / "halo_rust_mobility_repair_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_mobility_repair_test.go").write_text(text)
        (grammar / "halo_rust_component_hooks_test.go").write_text((reference / "halo_rust_component_hooks_test.go.txt").read_text())
        for kind in ("reads", "chain", "harvest", "resync", "repair"):
            text = (reference / f"halo_rust_unit_reference_hook_{kind}_test.go.txt").read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar / f"halo_rust_unit_reference_hook_{kind}_test.go").write_text(text)
        text = (reference / "halo_rust_component_hook_reads_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_component_hook_reads_test.go").write_text(text)
        text = (reference / "halo_rust_component_hook_chain_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_component_hook_chain_test.go").write_text(text)
        text = (reference / "halo_rust_component_hook_resync_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_component_hook_resync_test.go").write_text(text)
        text = (reference / "halo_rust_component_hook_harvest_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_component_hook_harvest_test.go").write_text(text)
        text = (reference / "halo_rust_component_hook_repair_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_component_hook_repair_test.go").write_text(text)
        text = (reference / "halo_rust_ability_hook_reads_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_ability_hook_reads_test.go").write_text(text)
        text = (reference / "halo_rust_ability_hook_chain_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_ability_hook_chain_test.go").write_text(text)
        text = (reference / "halo_rust_ability_hook_resync_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_ability_hook_resync_test.go").write_text(text)
        text = (reference / "halo_rust_ability_hook_harvest_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_ability_hook_harvest_test.go").write_text(text)
        text = (reference / "halo_rust_ability_hook_repair_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_ability_hook_repair_test.go").write_text(text)
        text = (reference / "halo_rust_object_hook_reads_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_object_hook_reads_test.go").write_text(text)
        text = (reference / "halo_rust_object_hook_chain_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_object_hook_chain_test.go").write_text(text)
        text = (reference / "halo_rust_object_hook_resync_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_object_hook_resync_test.go").write_text(text)
        text = (reference / "halo_rust_object_hook_harvest_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_object_hook_harvest_test.go").write_text(text)
        text = (reference / "halo_rust_object_hook_repair_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_object_hook_repair_test.go").write_text(text)
        text = (reference / "halo_rust_managed_hook_reads_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_managed_hook_reads_test.go").write_text(text)
        text = (reference / "halo_rust_managed_hook_chain_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_managed_hook_chain_test.go").write_text(text)
        text = (reference / "halo_rust_managed_hook_resync_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_managed_hook_resync_test.go").write_text(text)
        text = (reference / "halo_rust_managed_hook_harvest_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_managed_hook_harvest_test.go").write_text(text)
        text = (reference / "halo_rust_managed_hook_repair_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_managed_hook_repair_test.go").write_text(text)
        text = (reference / "halo_rust_engine_hook_reads_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_engine_hook_reads_test.go").write_text(text)
        text = (reference / "halo_rust_engine_hook_chain_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_engine_hook_chain_test.go").write_text(text)
        text = (reference / "halo_rust_engine_hook_resync_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_engine_hook_resync_test.go").write_text(text)
        text = (reference / "halo_rust_engine_hook_harvest_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_engine_hook_harvest_test.go").write_text(text)
        text = (reference / "halo_rust_engine_hook_repair_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_engine_hook_repair_test.go").write_text(text)
        text = (reference / "halo_rust_player_hook_reads_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_player_hook_reads_test.go").write_text(text)
        text = (reference / "halo_rust_player_hook_chain_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_player_hook_chain_test.go").write_text(text)
        text = (reference / "halo_rust_player_hook_resync_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_player_hook_resync_test.go").write_text(text)
        text = (reference / "halo_rust_player_hook_harvest_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_player_hook_harvest_test.go").write_text(text)
        text = (reference / "halo_rust_player_hook_repair_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_player_hook_repair_test.go").write_text(text)
        text = (reference / "halo_rust_probe_hook_reads_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_probe_hook_reads_test.go").write_text(text)
        text = (reference / "halo_rust_probe_hook_chain_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_probe_hook_chain_test.go").write_text(text)
        text = (reference / "halo_rust_probe_hook_resync_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_probe_hook_resync_test.go").write_text(text)
        text = (reference / "halo_rust_probe_hook_harvest_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_probe_hook_harvest_test.go").write_text(text)
        text = (reference / "halo_rust_probe_hook_repair_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_probe_hook_repair_test.go").write_text(text)
        (grammar / "halo_rust_movement_recovery_hooks_test.go").write_text((reference / "halo_rust_movement_recovery_hooks_test.go.txt").read_text())
        text = (reference / "halo_rust_movement_hook_recovery_slots_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_movement_hook_recovery_slots_test.go").write_text(text)
        for kind in ("reads", "production", "generic", "inference", "harvest", "resync", "chain", "repair"):
            text = (reference / f"halo_rust_movement_hook_{kind}_test.go.txt").read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar / f"halo_rust_movement_hook_{kind}_test.go").write_text(text)
        for kind in ("reads", "keyframes", "chain", "repair", "frames"):
            text = (reference / f"halo_rust_default_hook_{kind}_test.go.txt").read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar / f"halo_rust_default_hook_{kind}_test.go").write_text(text)
        text = (reference / "halo_rust_equipment_hook_reads_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_equipment_hook_reads_test.go").write_text(text)
        text = (reference / "halo_rust_equipment_hook_chain_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_equipment_hook_chain_test.go").write_text(text)
        text = (reference / "halo_rust_equipment_hook_resync_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_equipment_hook_resync_test.go").write_text(text)
        text = (reference / "halo_rust_equipment_hook_harvest_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_equipment_hook_harvest_test.go").write_text(text)
        text = (reference / "halo_rust_equipment_hook_repair_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_equipment_hook_repair_test.go").write_text(text)
        text = (reference / "halo_rust_registry_edges_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_registry_edges_test.go").write_text(text)
        text = (reference / "halo_rust_registry_warning_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_registry_warning_test.go").write_text(text)
        text = (reference / "halo_rust_unknown_build_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_unknown_build_test.go").write_text(text)
        text = (reference / "halo_rust_raw_resync_diagnostics_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_raw_resync_diagnostics_test.go").write_text(text)
        text = (reference / "halo_rust_harvest_diagnostics_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_harvest_diagnostics_test.go").write_text(text)
        text = (reference / "halo_rust_repair_diagnostics_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_repair_diagnostics_test.go").write_text(text)
        text = (reference / "halo_rust_resync_diagnostics_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_resync_diagnostics_test.go").write_text(text)
        text = (reference / "halo_rust_frame_diagnostics_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_frame_diagnostics_test.go").write_text(text)
        text = (reference / "halo_rust_read_diagnostics_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_read_diagnostics_test.go").write_text(text)
        text = (reference / "halo_rust_chain_diagnostics_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_chain_diagnostics_test.go").write_text(text)
        text = (reference / "halo_rust_weapon_patterns_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "weaponscan/halo_rust_weapon_patterns_test.go").write_text(text)
        text = (reference / "halo_rust_weapon_patterns_corpus_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "weaponscan/halo_rust_weapon_patterns_corpus_test.go").write_text(text)
        text = (reference / "halo_rust_keyframe_position_corpus_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "positions/halo_rust_keyframe_position_corpus_test.go").write_text(text)
        text = (reference / "halo_rust_keyframe_position_probe_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "positions/halo_rust_keyframe_position_probe_test.go").write_text(text)
        text = (reference / "halo_rust_unit_equipment_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_unit_equipment_test.go").write_text(text)
        text = (reference / "halo_rust_unit_equipment_corpus_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_unit_equipment_corpus_test.go").write_text(text)
        text = (reference / "halo_rust_position_hook_sequence_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_position_hook_sequence_test.go").write_text(text)
        (grammar / "halo_rust_position_hooks_test.go").write_text((reference / "halo_rust_position_hooks_test.go.txt").read_text())
        for kind in ("resolved", "production", "inference", "generic", "recovery_slots", "keyframes", "harvest", "resync", "chain", "repair", "validated_resync"):
            text = (reference / f"halo_rust_position_hook_{kind}_test.go.txt").read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar / f"halo_rust_position_hook_{kind}_test.go").write_text(text)
        text = (reference / "halo_rust_lazy_widths_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_lazy_widths_test.go").write_text(text)
        text = (reference / "halo_rust_delta_width_fallback_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_delta_width_fallback_test.go").write_text(text)
        text = (reference / "halo_rust_position_capture_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_position_capture_test.go").write_text(text)
        text = (reference / "halo_rust_frame_resync_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_frame_resync_test.go").write_text(text)
        text = (reference / "halo_rust_frame_targets_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_frame_targets_test.go").write_text(text)
        text = (reference / "halo_rust_i0_loaded_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_i0_loaded_test.go").write_text(text)
        text = (reference / "halo_rust_i0_layout_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_i0_layout_test.go").write_text(text)
        text = (reference / "halo_rust_world_precision_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (upstream / "apps/go-api/internal/games/halo_infinite/film/replay/halo_rust_world_precision_test.go").write_text(text)
        text = (reference / "halo_rust_weapon_helpers_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "weaponv3/halo_rust_weapon_helpers_test.go").write_text(text)
        text = (reference / "halo_rust_weapon_hit_corpus_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_weapon_hit_corpus_test.go").write_text(text)
        text = (reference / "halo_rust_weapon_hit_scan_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_weapon_hit_scan_test.go").write_text(text)
        text = (reference / "halo_rust_weapon_hits_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_weapon_hits_test.go").write_text(text)
        text = (reference / "halo_rust_flag_grab_bridge_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (upstream / "apps/go-api/internal/games/halo_infinite/film/replay/halo_rust_flag_grab_bridge_test.go").write_text(text)
        text = (reference / "halo_rust_vehicle_creation_profile_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_vehicle_creation_profile_test.go").write_text(text)
        text = (reference / "halo_rust_equipment_profile_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_equipment_profile_test.go").write_text(text)
        text = (reference / "halo_rust_usage_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (upstream / "apps/go-api/internal/games/halo_infinite/film/replay/halo_rust_usage_test.go").write_text(text)
        text = (reference / "halo_rust_fallback_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/fallback/halo_rust_fallback_test.go").write_text(text)
        text = (reference / "halo_rust_objective_extract_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_objective_extract_test.go").write_text(text)
        text = (reference / "halo_rust_statborg_awards_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_statborg_awards_test.go").write_text(text)
        text = (reference / "halo_rust_flag_grabs_net_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_flag_grabs_net_test.go").write_text(text)
        text = (reference / "halo_rust_decoded_kill_document_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (upstream / "apps/go-api/internal/games/halo_infinite/film/replay/halo_rust_decoded_kill_document_test.go").write_text(text)
        text = (reference / "halo_rust_replay_kill_inputs_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (upstream / "apps/go-api/internal/replaybuild/halo_rust_replay_kill_inputs_test.go").write_text(text)
        text = (reference / "halo_rust_kill_decode_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_decode_test.go").write_text(text)
        text = (reference / "halo_rust_kill_matching_test.go.txt").read_text()
        text = text.replace("/private/tmp/halo-kill-matching.json", str(tmp / "kill-matching.json"))
        (grammar.parent / "facts/killsource/halo_rust_kill_matching_test.go").write_text(text)
        text = (reference / "halo_rust_kill_walk_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_walk_test.go").write_text(text)
        text = (reference / "halo_rust_kill_walk_policy_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/killsource/halo_rust_kill_walk_policy_test.go").write_text(text)
        def run(test, package="internal/grammar", timeout="10m"):
            subprocess.run([args.go, "test", f"./internal/games/halo_infinite/film/{package}",
                            "-run", f"^{test}$", "-count=1", "-timeout", timeout], cwd=upstream / "apps/go-api", check=True)
        for name, test in [("replay-clock", "TestHaloRustReplayClock"), ("replay-evidence", "TestHaloRustReplayEvidence"), ("replay-scope", "TestHaloRustReplayScope")]:
            run(test, "replay")
            (film / f"fixtures/{name}-v41.json.zlib").write_bytes(zlib.compress((tmp / f"{name}.json").read_bytes(), 9))
        run("TestHaloRustFallback", "internal/facts/fallback")
        fallback_raw = (tmp / "fallback.json").read_bytes()
        (film / "fixtures/fallback-v41.json.zlib").write_bytes(zlib.compress(fallback_raw, 9))
        (reference / "fallback-registry-v75.json").write_text(json.dumps(json.loads(fallback_raw)["families"], ensure_ascii=False, indent=2) + "\n")
        run("TestHaloRustObjectiveCorpus", "internal/facts/objectives")
        (film / "fixtures/objective-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "objective-corpus.json").read_bytes(), 9))
        run("TestHaloRustObjectiveExtract", "internal/facts/objectives")
        (film / "fixtures/objective-extract-v41.json.zlib").write_bytes(zlib.compress((tmp / "objective-extract.json").read_bytes(), 9))
        run("TestHaloRustStatborgAwards", "internal/facts/objectives")
        (film / "fixtures/statborg-awards-v41.json.zlib").write_bytes(zlib.compress((tmp / "statborg-awards.json").read_bytes(), 9))
        run("TestHaloRustFlagGrabsNet", "internal/facts/objectives")
        (film / "fixtures/flag-grabs-net-v41.json.zlib").write_bytes(zlib.compress((tmp / "flag-grabs-net.json").read_bytes(), 9))
        run("TestHaloRustKillFeedPairs", "internal/facts/killsource")
        subprocess.run([args.go, "test", "./internal/replaybuild", "-run", "^TestHaloRustReplayKillInputs$", "-count=1"], cwd=upstream / "apps/go-api", check=True)
        (film / "fixtures/replay-kill-inputs-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-kill-inputs.json").read_bytes(), 9))
        run("TestHaloRustKillDecode", "internal/facts/killsource", "20m")
        (film / "fixtures/kill-decode-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-decode.json").read_bytes(), 9))
        run("TestHaloRustBudgetExhaustion", "internal/facts/objectives")
        (film / "fixtures/budget-exhaustion-v41.json.zlib").write_bytes(zlib.compress((tmp / "budget-exhaustion.json").read_bytes(), 9))
        run("TestHaloRustStatborgSourceNamedPositive", "internal/facts/objectives")
        (film / "fixtures/statborg-source-positive-v41.json.zlib").write_bytes(zlib.compress((tmp / "statborg-source-positive.json").read_bytes(), 9))
        run("TestHaloRustRoundHelpers", "internal/facts/objectives")
        (film / "fixtures/round-helpers-v41.json.zlib").write_bytes(zlib.compress((tmp / "round-helpers.json").read_bytes(), 9))
        run("TestHaloRustObjectiveSource", "internal/facts/objectives")
        (film / "fixtures/objective-source-v41.json.zlib").write_bytes(zlib.compress((tmp / "objective-source.json").read_bytes(), 9))
        run("TestHaloRustStatborgSource", "internal/facts/objectives")
        run("TestHaloRustStatborgSourceResolved", "internal/facts/objectives")
        (film / "fixtures/statborg-source-resolved-v41.json.zlib").write_bytes(zlib.compress((tmp / "statborg-source-resolved.json").read_bytes(), 9))
        (film / "fixtures/statborg-source-v41.json.zlib").write_bytes(zlib.compress((tmp / "statborg-source.json").read_bytes(), 9))
        run("TestHaloRustStatborgSourceLimit", "internal/facts/objectives")
        (film / "fixtures/statborg-source-limit-v41.json.zlib").write_bytes(zlib.compress((tmp / "statborg-source-limit.json").read_bytes(), 9))
        run("TestHaloRustChronology", "internal/facts/objectives")
        (film / "fixtures/chronology-v41.json.zlib").write_bytes(zlib.compress((tmp / "chronology.json").read_bytes(), 9))
        run("TestHaloRustBudgetPasses", "internal/facts/objectives")
        (film / "fixtures/budget-passes-v41.json.zlib").write_bytes(zlib.compress((tmp / "budget-passes.json").read_bytes(), 9))
        run("TestHaloRustBudgetDiagnostics", "internal/facts/objectives")
        (film / "fixtures/budget-diagnostics-v41.json.zlib").write_bytes(zlib.compress((tmp / "budget-diagnostics.json").read_bytes(), 9))
        run("TestHaloRustStatComponents", "internal/facts/objectives")
        (film / "fixtures/stat-components-v41.json.zlib").write_bytes(zlib.compress((tmp / "stat-components.json").read_bytes(), 9))
        run("TestHaloRustKillChunks", "internal/facts/killsource")
        (film / "fixtures/kill-chunks-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-chunks.json").read_bytes(), 9))
        run("TestHaloRustKillMatching", "internal/facts/killsource")
        (film / "fixtures/kill-matching-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-matching.json").read_bytes(), 9))
        (film / "fixtures/kill-feed-pairs-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-feed-pairs.json").read_bytes(), 9))
        text = (reference / "halo_rust_kill_table_source_test.go.txt").read_text()
        text = text.replace("@BOOTSTRAP_FIXTURE@", json.dumps(str(film / "fixtures/bootstrap-v41.zlib"))[1:-1])
        text = text.replace("@TABLE_FIXTURE@", json.dumps(str(film / "fixtures/kill-table-source-bootstrap-v41.zlib"))[1:-1])
        text = text.replace("/private/tmp/halo-kill-table-source.json", str(tmp / "kill-table-source.json"))
        (grammar.parent / "facts/killsource/halo_rust_kill_table_source_test.go").write_text(text)
        run("TestHaloRustKillTableSource", "internal/facts/killsource")
        (film / "fixtures/kill-table-source-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-table-source.json").read_bytes(), 9))
        run("TestHaloRustKillRoster", "internal/facts/killsource")
        (film / "fixtures/kill-roster-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-roster.json").read_bytes(), 9))
        run("TestHaloRustKillBijection", "internal/facts/killsource")
        (film / "fixtures/kill-bijection-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-bijection.json").read_bytes(), 9))
        run("TestHaloRustKillSourceScan", "internal/facts/killsource")
        (film / "fixtures/kill-source-scan-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-source-scan.json").read_bytes(), 9))
        run("TestHaloRustKillIndexMotif", "internal/facts/killsource")
        (film / "fixtures/kill-index-motif-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-index-motif.json").read_bytes(), 9))
        run("TestHaloRustKillFilmEvidence", "internal/facts/killsource")
        (film / "fixtures/kill-film-evidence-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-film-evidence.json").read_bytes(), 9))
        run("TestHaloRustKillTimeline", "internal/facts/killsource")
        (film / "fixtures/kill-timeline-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-timeline.json").read_bytes(), 9))
        run("TestHaloRustKillWalk", "internal/facts/killsource", "20m")
        (film / "fixtures/kill-walk-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-walk.json").read_bytes(), 9))
        run("TestHaloRustKillWalkPolicy", "internal/facts/killsource")
        (film / "fixtures/kill-walk-policy-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-walk-policy.json").read_bytes(), 9))
        run("TestHaloRustKillFeed", "internal/facts/killsource")
        (film / "fixtures/kill-feed-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-feed.json").read_bytes(), 9))
        run("TestHaloRustKillAssists", "internal/facts/killsource")
        (film / "fixtures/kill-assists-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-assists.json").read_bytes(), 9))
        run("TestHaloRustKillEventChain", "internal/facts/killsource")
        (film / "fixtures/kill-event-chain-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-event-chain.json").read_bytes(), 9))
        run("TestHaloRustKillEventScan", "internal/facts/killsource")
        (film / "fixtures/kill-event-scan-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-event-scan.json").read_bytes(), 9))
        run("TestHaloRustLayers", "replay")
        (film / "fixtures/layers-v41.json.zlib").write_bytes(zlib.compress((tmp / "layers.json").read_bytes(), 9))
        run("TestHaloRustNeutralDeaths", "replay")
        (film / "fixtures/neutral-deaths-v41.json.zlib").write_bytes(zlib.compress((tmp / "neutral-deaths.json").read_bytes(), 9))
        run("TestHaloRustWeaponFamily", "replay")
        (film / "fixtures/weapon-family-v41.json.zlib").write_bytes(zlib.compress((tmp / "weapon-family.json").read_bytes(), 9))
        run("TestHaloRustWeaponLabels", "replay")
        (film / "fixtures/weapon-labels-v41.json.zlib").write_bytes(zlib.compress((tmp / "weapon-labels.json").read_bytes(), 9))
        run("TestHaloRustMapWeaponPads", "replay")
        (film / "fixtures/map-weapon-pads-v41.json.zlib").write_bytes(zlib.compress((tmp / "map-weapon-pads.json").read_bytes(), 9))
        (reference / "map-weapon-pads-v75.json.zlib").write_bytes(zlib.compress((upstream / "data/titles/halo_infinite/reference/map_weapon_pads.json").read_bytes(), 9))
        run("TestHaloRustDecoderCoverage", "replay")
        (film / "fixtures/decoder-coverage-v41.json.zlib").write_bytes(zlib.compress((tmp / "decoder-coverage.json").read_bytes(), 9))
        run("TestHaloRustT0", "replay")
        (film / "fixtures/t0-v41.json.zlib").write_bytes(zlib.compress((tmp / "t0.json").read_bytes(), 9))
        run("TestHaloRustDocumentSchema", "replay")
        (film / "fixtures/document-schema-v41.json").write_bytes((tmp / "document-schema.json").read_bytes())
        run("TestHaloRustStances", "replay")
        (film / "fixtures/replay-stances-v41.json.zlib").write_bytes(zlib.compress((tmp / "stances.json").read_bytes(), 9))
        run("TestHaloRustDocumentContent", "replay")
        (film / "fixtures/document-content-v41.json").write_bytes((tmp / "document-content.json").read_bytes())
        run("TestHaloRustObjectiveModes", "replay")
        (film / "fixtures/objective-modes-v41.json").write_bytes((tmp / "objective-modes.json").read_bytes())
        (film / "fixtures/objective-map-selection-v41.json.zlib").write_bytes(zlib.compress((tmp / "objective-map-selection.json").read_bytes(), 9))
        (reference / "objective-modes-v75.json").write_bytes((tmp / "objective-modes-table.json").read_bytes())
        run("TestHaloRustZones", "replay")
        (film / "fixtures/replay-zones-v41.json.zlib").write_bytes(zlib.compress((tmp / "zones.json").read_bytes(), 9))
        run("TestHaloRustZoneHills", "replay")
        (film / "fixtures/replay-zone-hills-v41.json.zlib").write_bytes(zlib.compress((tmp / "zone-hills.json").read_bytes(), 9))
        run("TestHaloRustZoneOwnerLayer", "replay")
        (film / "fixtures/replay-zone-owner-layer-v41.json.zlib").write_bytes(zlib.compress((tmp / "zone-owner-layer.json").read_bytes(), 9))
        run("TestHaloRustZoneOwners", "replay")
        (film / "fixtures/replay-zone-owners-v41.json.zlib").write_bytes(zlib.compress((tmp / "zone-owners.json").read_bytes(), 9))
        run("TestHaloRustZonePairing", "replay")
        (film / "fixtures/replay-zone-pairing-v41.json.zlib").write_bytes(zlib.compress((tmp / "zone-pairing.json").read_bytes(), 9))
        run("TestHaloRustZoneAttribution", "replay")
        (film / "fixtures/replay-zone-attribution-v41.json.zlib").write_bytes(zlib.compress((tmp / "zone-attribution.json").read_bytes(), 9))
        run("TestHaloRustZoneSeries", "replay")
        (film / "fixtures/replay-zone-series-v41.json.zlib").write_bytes(zlib.compress((tmp / "zone-series.json").read_bytes(), 9))
        run("TestHaloRustMapObjectives", "replay")
        (film / "fixtures/map-objectives-v41.json.zlib").write_bytes(zlib.compress((tmp / "map-objectives.json").read_bytes(), 9))
        (reference / "map-objectives-v75.json.zlib").write_bytes(zlib.compress((upstream / "data/titles/halo_infinite/reference/map_objectives.json").read_bytes(), 9))
        run("TestHaloRustCaptureBursts", "internal/facts/objectives")
        (film / "fixtures/capture-bursts-v41.json.zlib").write_bytes(zlib.compress((tmp / "capture-bursts.json").read_bytes(), 9))
        run("TestHaloRustManagedPropertyCorpus")
        (film / "fixtures/managed-property-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "managed-property-corpus.json").read_bytes(), 9))
        run("TestHaloRustManagedPropertyScan")
        (film / "fixtures/managed-property-v41.json.zlib").write_bytes(zlib.compress((tmp / "managed-property.json").read_bytes(), 9))
        text = (reference / "halo_rust_managed_source_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_managed_source_test.go").write_text(text)
        run("TestHaloRustManagedSource")
        (film / "fixtures/managed-source-v41.json.zlib").write_bytes(zlib.compress((tmp / "managed-source.json").read_bytes(), 9))
        run("TestHaloRustBipedRemainingContext")
        (film / "fixtures/biped-remaining-context-v41.json.zlib").write_bytes(zlib.compress((tmp / "biped-remaining-context.json").read_bytes(), 9))
        text = (reference / "halo_rust_map_catalog_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_map_catalog_test.go").write_text(text)
        run("TestHaloRustMapCatalog")
        (film / "fixtures/map-catalog-v41.json.zlib").write_bytes(zlib.compress((tmp / "map-catalog.json").read_bytes(), 9))
        text = (reference / "halo_rust_map_rounding_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_map_rounding_test.go").write_text(text)
        run("TestHaloRustMapRounding")
        (film / "fixtures/map-rounding-v41.json.zlib").write_bytes(zlib.compress((tmp / "map-rounding.json").read_bytes(), 9))
        text = (reference / "halo_rust_map_names_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_map_names_test.go").write_text(text)
        run("TestHaloRustMapNames")
        (film / "fixtures/map-names-v41.json.zlib").write_bytes(zlib.compress((tmp / "map-names.json").read_bytes(), 9))
        text = (reference / "halo_rust_weapon_track_range_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_weapon_track_range_test.go").write_text(text)
        run("TestHaloRustWeaponTrackRange")
        (film / "fixtures/weapon-track-range-v41.json.zlib").write_bytes(zlib.compress((tmp / "weapon-track-range.json").read_bytes(), 9))
        text = (reference / "halo_rust_weapon_range_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_weapon_range_test.go").write_text(text)
        run("TestHaloRustWeaponRange")
        (film / "fixtures/weapon-range-v41.json").write_bytes((tmp / "weapon-range.json").read_bytes())
        text = (reference / "halo_rust_registry_result_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_registry_result_test.go").write_text(text)
        run("TestHaloRustRegistryResult")
        (film / "fixtures/registry-result-v41.json.zlib").write_bytes(zlib.compress((tmp / "registry-result.json").read_bytes(), 9))
        run("TestHaloRustBipedContext")
        (film / "fixtures/biped-context-v41.json.zlib").write_bytes(zlib.compress((tmp / "biped-context.json").read_bytes(), 9))
        run("TestHaloRustNavpointRadialScan")
        (film / "fixtures/navpoint-radial-scan-v41.json.zlib").write_bytes(zlib.compress((tmp / "navpoint-radial-scan.json").read_bytes(), 9))
        run("TestHaloRustCarrierMarks")
        (film / "fixtures/carrier-marks-v41.json.zlib").write_bytes(zlib.compress((tmp / "carrier-marks.json").read_bytes(), 9))
        run("TestHaloRustReplayFlagGauges", "replay")
        (film / "fixtures/replay-flag-gauges-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-flag-gauges.json").read_bytes(), 9))
        run("TestHaloRustReplayFlagGeometry", "replay")
        (film / "fixtures/replay-flag-geometry-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-flag-geometry.json").read_bytes(), 9))
        run("TestHaloRustReplayFlagCarries", "replay")
        (film / "fixtures/replay-flag-carries-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-flag-carries.json").read_bytes(), 9))
        run("TestHaloRustReplayObjectiveActions", "replay")
        (film / "fixtures/replay-objective-actions-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-objective-actions.json").read_bytes(), 9))
        run("TestHaloRustReplayBombArmings", "replay")
        (film / "fixtures/replay-bomb-armings-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-bomb-armings.json").read_bytes(), 9))
        run("TestHaloRustBombStartFallback", "replay")
        (film / "fixtures/bomb-start-fallback-v41.json.zlib").write_bytes(zlib.compress((tmp / "bomb-start-fallback.json").read_bytes(), 9))
        run("TestHaloRustReplayBombStats", "replay")
        (film / "fixtures/replay-bomb-stats-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-bomb-stats.json").read_bytes(), 9))
        run("TestHaloRustReplayHeldObject", "replay")
        (film / "fixtures/replay-held-object-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-held-object.json").read_bytes(), 9))
        run("TestHaloRustReplaySkull", "replay")
        (film / "fixtures/replay-skull-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-skull.json").read_bytes(), 9))
        text = (reference / "halo_rust_objective_diagnostics_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_objective_diagnostics_test.go").write_text(text)
        run("TestHaloRustObjectiveDiagnostics", "replay")
        (film / "fixtures/objective-diagnostics-v41.json.zlib").write_bytes(zlib.compress((tmp / "objective-diagnostics.json").read_bytes(), 9))
        run("TestHaloRustReplayVip", "replay")
        (film / "fixtures/replay-vip-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-vip.json").read_bytes(), 9))
        run("TestHaloRustReplayScoreTeams", "replay")
        (film / "fixtures/replay-score-teams-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-score-teams.json").read_bytes(), 9))
        run("TestHaloRustReplayScorePlayers", "replay")
        (film / "fixtures/replay-score-players-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-score-players.json").read_bytes(), 9))
        run("TestHaloRustReplayScoreSeries", "replay")
        (film / "fixtures/replay-score-series-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-score-series.json").read_bytes(), 9))
        run("TestHaloRustReplayWeaponChanges", "replay")
        (film / "fixtures/replay-weapon-changes-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-weapon-changes.json").read_bytes(), 9))
        run("TestHaloRustReplayEquipmentOrigin", "replay")
        (film / "fixtures/replay-equipment-origin-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-equipment-origin.json").read_bytes(), 9))
        run("TestHaloRustReplayEquipmentEpisodes", "replay")
        (film / "fixtures/replay-equipment-episodes-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-equipment-episodes.json").read_bytes(), 9))
        run("TestHaloRustReplayGrapple", "replay")
        (film / "fixtures/replay-grapple-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-grapple.json").read_bytes(), 9))
        run("TestHaloRustReplayTranslocations", "replay")
        (film / "fixtures/replay-translocations-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-translocations.json").read_bytes(), 9))
        run("TestHaloRustReplayEquipmentPublication", "replay")
        (film / "fixtures/replay-equipment-publication-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-equipment-publication.json").read_bytes(), 9))
        run("TestHaloRustReplayInventoryPublication", "replay")
        (film / "fixtures/replay-inventory-publication-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-inventory-publication.json").read_bytes(), 9))
        run("TestHaloRustReplayEquipmentCatalog", "replay")
        (reference / "equipment_catalog.json").write_bytes((tmp / "equipment-catalog.json").read_bytes())
        run("TestHaloRustReplayAbilityCatalog", "replay")
        (reference / "ability_catalog.json").write_bytes((tmp / "ability-catalog.json").read_bytes())
        run("TestHaloRustReplayAbilityCharges", "replay")
        (film / "fixtures/replay-ability-charges-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-ability-charges.json").read_bytes(), 9))
        run("TestHaloRustMarchCalibration")
        (film / "fixtures/march-calibration-v41.json.zlib").write_bytes(zlib.compress((tmp / "march-calibration.json").read_bytes(), 9))
        run("TestHaloRustQuantize")
        (film / "fixtures/quantize-v41.json.zlib").write_bytes(zlib.compress((tmp / "quantize.json").read_bytes(), 9))
        run("TestHaloRustEndpoint")
        (film / "fixtures/endpoint-v41.json.zlib").write_bytes(zlib.compress((tmp / "endpoint.json").read_bytes(), 9))
        run("TestHaloRustVehicleCreationCorpus")
        (film / "fixtures/vehicle-creation-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "vehicle-creation-corpus.json").read_bytes(), 9))
        run("TestHaloRustMarchLazyPolicy")
        (film / "fixtures/march-lazy-policy-v41.json.zlib").write_bytes(zlib.compress((tmp / "march-lazy-policy.json").read_bytes(), 9))
        run("TestHaloRustMarchCorpus")
        (film / "fixtures/march-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "march-corpus.json").read_bytes(), 9))
        run("TestHaloRustMarchWalk")
        (film / "fixtures/march-walk-v41.json.zlib").write_bytes(zlib.compress((tmp / "march-walk.json").read_bytes(), 9))
        run("TestHaloRustMarchFacts")
        (film / "fixtures/march-facts-v41.json.zlib").write_bytes(zlib.compress((tmp / "march-facts.json").read_bytes(), 9))
        run("TestHaloRustBipedAimLoaded")
        (film / "fixtures/biped-aim-loaded-v41.json.zlib").write_bytes(zlib.compress((tmp / "biped-aim-loaded.json").read_bytes(), 9))
        run("TestHaloRustBipedAim")
        (film / "fixtures/biped-aim-v41.json.zlib").write_bytes(zlib.compress((tmp / "biped-aim.json").read_bytes(), 9))
        run("TestHaloRustVehicleEvents")
        (film / "fixtures/vehicle-events-v41.json.zlib").write_bytes(zlib.compress((tmp / "vehicle-events.json").read_bytes(), 9))
        run("TestHaloRustVehicleI0")
        (film / "fixtures/vehicle-i0-v41.json.zlib").write_bytes(zlib.compress((tmp / "vehicle-i0.json").read_bytes(), 9))
        run("TestHaloRustVehicleCreationLayout")
        (film / "fixtures/vehicle-creation-layout-v41.json.zlib").write_bytes(zlib.compress((tmp / "vehicle-creation-layout.json").read_bytes(), 9))
        run("TestHaloRustVehicleCreationLoaded")
        (film / "fixtures/vehicle-creation-loaded-v41.json.zlib").write_bytes(zlib.compress((tmp / "vehicle-creation-loaded.json").read_bytes(), 9))
        run("TestHaloRustVehicleCreations")
        (film / "fixtures/vehicle-creations-v41.json.zlib").write_bytes(zlib.compress((tmp / "vehicle-creations.json").read_bytes(), 9))
        run("TestHaloRustVehicleScan")
        (film / "fixtures/vehicle-scan-v41.json.zlib").write_bytes(zlib.compress((tmp / "vehicle-scan.json").read_bytes(), 9))
        run("TestHaloRustOrientation")
        (film / "fixtures/orientation-v41.json.zlib").write_bytes(zlib.compress((tmp / "orientation.json").read_bytes(), 9))
        run("TestHaloRustVelocityCodec")
        (film / "fixtures/velocity-codec-v41.json.zlib").write_bytes(zlib.compress((tmp / "velocity-codec.json").read_bytes(), 9))
        run("TestHaloRustOfflineAimHelpers")
        (film / "fixtures/offline-aim-helpers-v41.json.zlib").write_bytes(zlib.compress((tmp / "offline-aim-helpers.json").read_bytes(), 9))
        run("TestHaloRustChannelIndependence")
        (film / "fixtures/channel-independence-v41.json.zlib").write_bytes(zlib.compress((tmp / "channel-independence.json").read_bytes(), 9))
        run("TestHaloRustManagedSetup")
        (film / "fixtures/managed-setup-v41.json.zlib").write_bytes(zlib.compress((tmp / "managed-setup.json").read_bytes(), 9))
        run("TestHaloRustNavpointSetup")
        (film / "fixtures/navpoint-setup-v41.json.zlib").write_bytes(zlib.compress((tmp / "navpoint-setup.json").read_bytes(), 9))
        run("TestHaloRustReaderSequence")
        (film / "fixtures/reader-sequence-v41.json.zlib").write_bytes(zlib.compress((tmp / "reader-sequence.json").read_bytes(), 9))
        run("TestHaloRustCreationSource")
        (film / "fixtures/creation-source-v41.json.zlib").write_bytes(zlib.compress((tmp / "creation-source.json").read_bytes(), 9))
        run("TestHaloRustMovementSource")
        (film / "fixtures/movement-source-v41.json.zlib").write_bytes(zlib.compress((tmp / "movement-source.json").read_bytes(), 9))
        run("TestHaloRustBipedSourceAvailability")
        (film / "fixtures/biped-source-availability-v41.json.zlib").write_bytes(zlib.compress((tmp / "biped-source-availability.json").read_bytes(), 9))
        run("TestHaloRustBipedDuplicateSources")
        (film / "fixtures/biped-duplicate-sources-v41.json.zlib").write_bytes(zlib.compress((tmp / "biped-duplicate-sources.json").read_bytes(), 9))
        run("TestHaloRustInventorySourceAvailability")
        (film / "fixtures/inventory-source-availability-v41.json.zlib").write_bytes(zlib.compress((tmp / "inventory-source-availability.json").read_bytes(), 9))
        run("TestHaloRustInventoryDuplicateSources")
        (film / "fixtures/inventory-duplicate-sources-v41.json.zlib").write_bytes(zlib.compress((tmp / "inventory-duplicate-sources.json").read_bytes(), 9))
        run("TestHaloRustVehicleFilmRides", "replay")
        (film / "fixtures/vehicle-film-rides-v41.json.zlib").write_bytes(zlib.compress((tmp / "vehicle-film-rides.json").read_bytes(), 9))
        run("TestHaloRustReplayVehicleTracks", "replay")
        (film / "fixtures/replay-vehicle-tracks-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-vehicle-tracks.json").read_bytes(), 9))
        run("TestHaloRustReplayVehicleRelays", "replay")
        (film / "fixtures/replay-vehicle-relays-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-vehicle-relays.json").read_bytes(), 9))
        run("TestHaloRustReplayVehicleLives", "replay")
        (film / "fixtures/replay-vehicle-lives-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-vehicle-lives.json").read_bytes(), 9))
        run("TestHaloRustReplayVehicleShots", "replay")
        (film / "fixtures/replay-vehicle-shots-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-vehicle-shots.json").read_bytes(), 9))
        run("TestHaloRustReplayGrenadeReads", "replay")
        (film / "fixtures/replay-grenade-reads-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-grenade-reads.json").read_bytes(), 9))
        run("TestHaloRustReplayGrenades", "replay")
        (film / "fixtures/replay-grenades-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-grenades.json").read_bytes(), 9))
        run("TestHaloRustReplayProjectiles", "replay")
        (film / "fixtures/replay-projectiles-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-projectiles.json").read_bytes(), 9))
        run("TestHaloRustReplayLoadouts", "replay")
        (film / "fixtures/replay-loadouts-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-loadouts.json").read_bytes(), 9))
        run("TestHaloRustReplayShots", "replay")
        (film / "fixtures/replay-shots-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-shots.json").read_bytes(), 9))
        if args.include_raid:
            run("TestHaloRustFilmRaid", "replay", "60m")
            run("TestHaloRustFullRaidDocument", "replay", "60m")
            (film / "fixtures/full-raid-document-v41.json.zlib").write_bytes(zlib.compress((tmp / "full-raid-document.json").read_bytes(), 9))
            run("TestHaloRustRaidVehicleInputs", "replay", "60m")
            write_raid_vehicle_oracle(film, tmp / "raid-vehicle-inputs.json")
            (film / "fixtures/film-raid-v41.json.zlib").write_bytes(zlib.compress((tmp / "film-raid.json").read_bytes(), 9))
        run("TestHaloRustDecodedKillDocument", "replay", "20m")
        (film / "fixtures/decoded-kill-document-v41.json.zlib").write_bytes(zlib.compress((tmp / "decoded-kill-document.json").read_bytes(), 9))
        run("TestHaloRustCTFGeometryDocument", "replay")
        (film / "fixtures/ctf-geometry-document-v41.json.zlib").write_bytes(zlib.compress((tmp / "ctf-geometry-document.json").read_bytes(), 9))
        run("TestHaloRustCTFDecodedKillDocument", "replay")
        (film / "fixtures/ctf-decoded-kill-document-v41.json.zlib").write_bytes(zlib.compress((tmp / "ctf-decoded-kill-document.json").read_bytes(), 9))
        run("TestHaloRustCTFDocument", "replay")
        (film / "fixtures/ctf-document-v41.json.zlib").write_bytes(zlib.compress((tmp / "ctf-document.json").read_bytes(), 9))
        run("TestHaloRustFullDocument", "replay")
        write_full_document_oracles(film, tmp / "full-document.json")
        run("TestHaloRustObjectiveInputs", "replay")
        write_objective_input_oracle(film, tmp / "objective-inputs.json")
        text = (reference / "halo_rust_creation_world_profile_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_creation_world_profile_test.go").write_text(text)
        text = (reference / "halo_rust_equipment_calibration_profile_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_equipment_calibration_profile_test.go").write_text(text)
        text = (reference / "halo_rust_keyframe_closure_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_keyframe_closure_test.go").write_text(text)
        text = (reference / "halo_rust_march_loaded_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_march_loaded_test.go").write_text(text)
        text = (reference / "halo_rust_objective_scan_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_objective_scan_test.go").write_text(text)
        run("TestHaloRustObjectiveScan")
        (film / "fixtures/objective-scan-v41.json.zlib").write_bytes(zlib.compress((tmp / "objective-scan.json").read_bytes(), 9))
        run("TestHaloRustMarchLoaded")
        (film / "fixtures/march-loaded-v41.json.zlib").write_bytes(zlib.compress((tmp / "march-loaded.json").read_bytes(), 9))
        run("TestHaloRustKeyframeClosure")
        (film / "fixtures/keyframe-closure-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-closure.json").read_bytes(), 9))
        run("TestHaloRustEquipmentCalibrationProfile")
        (film / "fixtures/equipment-calibration-profile-v41.json.zlib").write_bytes(zlib.compress((tmp / "equipment-calibration-profile.json").read_bytes(), 9))
        run("TestHaloRustCreationWorldProfile")
        (film / "fixtures/creation-world-profile-v41.json.zlib").write_bytes(zlib.compress((tmp / "creation-world-profile.json").read_bytes(), 9))
        run("TestHaloRustVehicleCreationProfile")
        (film / "fixtures/vehicle-creation-profile-v41.json.zlib").write_bytes(zlib.compress((tmp / "vehicle-creation-profile.json").read_bytes(), 9))
        run("TestHaloRustEquipmentProfile")
        (film / "fixtures/equipment-profile-v41.json.zlib").write_bytes(zlib.compress((tmp / "equipment-profile.json").read_bytes(), 9))
        run("TestHaloRustKeyframeSpans")
        (film / "fixtures/keyframe-spans-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-spans.json").read_bytes(), 9))
        run("TestHaloRustMPPResolution", "replay")
        (film / "fixtures/mpp-resolution-v41.json.zlib").write_bytes(zlib.compress((tmp / "mpp-resolution.json").read_bytes(), 9))
        run("TestHaloRustUnknownFormatDocument", "replay")
        (film / "fixtures/unknown-format-document-v41.json.zlib").write_bytes(zlib.compress((tmp / "unknown-format-document.json").read_bytes(), 9))
        run("TestHaloRustMissingIdentityDocument", "replay")
        (film / "fixtures/missing-identity-document-v41.json.zlib").write_bytes(zlib.compress((tmp / "missing-identity-document.json").read_bytes(), 9))
        run("TestHaloRustZeroAxisDocument", "replay")
        (film / "fixtures/zero-axis-document-v41.json.zlib").write_bytes(zlib.compress((tmp / "zero-axis-document.json").read_bytes(), 9))
        run("TestHaloRustI0LayoutCorpus")
        (film / "fixtures/i0-layout-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "i0-layout-corpus.json").read_bytes(), 9))
        run("TestHaloRustPrecisionLaw")
        (film / "fixtures/precision-law-v41.json.zlib").write_bytes(zlib.compress((tmp / "precision-law.json").read_bytes(), 9))
        run("TestHaloRustChainInference")
        (film / "fixtures/chain-inference-v41.json.zlib").write_bytes(zlib.compress((tmp / "chain-inference.json").read_bytes(), 9))
        run("TestHaloRustChainRepair")
        (film / "fixtures/chain-repair-v41.json.zlib").write_bytes(zlib.compress((tmp / "chain-repair.json").read_bytes(), 9))
        run("TestHaloRustInferenceFrame")
        (film / "fixtures/inference-frame-v41.json.zlib").write_bytes(zlib.compress((tmp / "inference-frame.json").read_bytes(), 9))
        run("TestHaloRustValidatedResync")
        (film / "fixtures/validated-resync-v41.json.zlib").write_bytes(zlib.compress((tmp / "validated-resync.json").read_bytes(), 9))
        run("TestHaloRustChainNew")
        (film / "fixtures/chain-new-v41.json.zlib").write_bytes(zlib.compress((tmp / "chain-new.json").read_bytes(), 9))
        run("TestHaloRustChainNewRepair")
        (film / "fixtures/chain-new-repair-v41.json.zlib").write_bytes(zlib.compress((tmp / "chain-new-repair.json").read_bytes(), 9))
        run("TestHaloRustInferenceViews")
        (film / "fixtures/inference-views-v41.json.zlib").write_bytes(zlib.compress((tmp / "inference-views.json").read_bytes(), 9))
        run("TestHaloRustUnitReferences")
        (film / "fixtures/unit-references-v41.json.zlib").write_bytes(zlib.compress((tmp / "unit-references.json").read_bytes(), 9))
        run("TestHaloRustDefaultReferences")
        (film / "fixtures/default-references-v41.json.zlib").write_bytes(zlib.compress((tmp / "default-references.json").read_bytes(), 9))
        run("TestHaloRustKeyframeReferences")
        (film / "fixtures/keyframe-references-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-references.json").read_bytes(), 9))
        run("TestHaloRustMobilityReads")
        (film / "fixtures/mobility-reads-v41.json.zlib").write_bytes(zlib.compress((tmp / "mobility-reads.json").read_bytes(), 9))
        run("TestHaloRustMobilityChain")
        (film / "fixtures/mobility-chain-v41.json.zlib").write_bytes(zlib.compress((tmp / "mobility-chain.json").read_bytes(), 9))
        run("TestHaloRustMobilityResync")
        (film / "fixtures/mobility-resync-v41.json.zlib").write_bytes(zlib.compress((tmp / "mobility-resync.json").read_bytes(), 9))
        run("TestHaloRustMobilityHarvest")
        (film / "fixtures/mobility-harvest-v41.json.zlib").write_bytes(zlib.compress((tmp / "mobility-harvest.json").read_bytes(), 9))
        run("TestHaloRustMobilityRepair")
        (film / "fixtures/mobility-repair-v41.json.zlib").write_bytes(zlib.compress((tmp / "mobility-repair.json").read_bytes(), 9))
        for kind in ("Reads", "Chain", "Harvest", "Resync", "Repair"):
            run(f"TestHaloRustUnitReferenceHook{kind}")
            (film / f"fixtures/unit-reference-hook-{kind.lower()}-v41.json.zlib").write_bytes(zlib.compress((tmp / f"unit-reference-hook-{kind.lower()}.json").read_bytes(), 9))
        run("TestHaloRustComponentHookReads")
        (film / "fixtures/component-hook-reads-v41.json.zlib").write_bytes(zlib.compress((tmp / "component-hook-reads.json").read_bytes(), 9))
        run("TestHaloRustComponentHookChain")
        (film / "fixtures/component-hook-chain-v41.json.zlib").write_bytes(zlib.compress((tmp / "component-hook-chain.json").read_bytes(), 9))
        run("TestHaloRustComponentHookResync")
        (film / "fixtures/component-hook-resync-v41.json.zlib").write_bytes(zlib.compress((tmp / "component-hook-resync.json").read_bytes(), 9))
        run("TestHaloRustComponentHookHarvest")
        (film / "fixtures/component-hook-harvest-v41.json.zlib").write_bytes(zlib.compress((tmp / "component-hook-harvest.json").read_bytes(), 9))
        run("TestHaloRustComponentHookRepair")
        (film / "fixtures/component-hook-repair-v41.json.zlib").write_bytes(zlib.compress((tmp / "component-hook-repair.json").read_bytes(), 9))
        for kind in ("Reads", "Generic", "Inference"):
            name = f"halo_rust_body_profile_{kind.lower()}_test.go"
            text = (reference / (name + ".txt")).read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar / name).write_text(text)
            run(f"TestHaloRustBodyProfile{kind}")
            (film / f"fixtures/body-profile-{kind.lower()}-v41.json.zlib").write_bytes(zlib.compress((tmp / f"body-profile-{kind.lower()}.json").read_bytes(), 9))
        run("TestHaloRustAbilityHookReads")
        (film / "fixtures/ability-hook-reads-v41.json.zlib").write_bytes(zlib.compress((tmp / "ability-hook-reads.json").read_bytes(), 9))
        run("TestHaloRustAbilityHookChain")
        (film / "fixtures/ability-hook-chain-v41.json.zlib").write_bytes(zlib.compress((tmp / "ability-hook-chain.json").read_bytes(), 9))
        run("TestHaloRustAbilityHookResync")
        (film / "fixtures/ability-hook-resync-v41.json.zlib").write_bytes(zlib.compress((tmp / "ability-hook-resync.json").read_bytes(), 9))
        run("TestHaloRustAbilityHookHarvest")
        (film / "fixtures/ability-hook-harvest-v41.json.zlib").write_bytes(zlib.compress((tmp / "ability-hook-harvest.json").read_bytes(), 9))
        run("TestHaloRustAbilityHookRepair")
        (film / "fixtures/ability-hook-repair-v41.json.zlib").write_bytes(zlib.compress((tmp / "ability-hook-repair.json").read_bytes(), 9))
        run("TestHaloRustObjectHookReads")
        (film / "fixtures/object-hook-reads-v41.json.zlib").write_bytes(zlib.compress((tmp / "object-hook-reads.json").read_bytes(), 9))
        run("TestHaloRustObjectHookChain")
        (film / "fixtures/object-hook-chain-v41.json.zlib").write_bytes(zlib.compress((tmp / "object-hook-chain.json").read_bytes(), 9))
        run("TestHaloRustObjectHookResync")
        (film / "fixtures/object-hook-resync-v41.json.zlib").write_bytes(zlib.compress((tmp / "object-hook-resync.json").read_bytes(), 9))
        run("TestHaloRustObjectHookHarvest")
        (film / "fixtures/object-hook-harvest-v41.json.zlib").write_bytes(zlib.compress((tmp / "object-hook-harvest.json").read_bytes(), 9))
        run("TestHaloRustObjectHookRepair")
        (film / "fixtures/object-hook-repair-v41.json.zlib").write_bytes(zlib.compress((tmp / "object-hook-repair.json").read_bytes(), 9))
        run("TestHaloRustManagedHookReads")
        (film / "fixtures/managed-hook-reads-v41.json.zlib").write_bytes(zlib.compress((tmp / "managed-hook-reads.json").read_bytes(), 9))
        run("TestHaloRustManagedHookChain")
        (film / "fixtures/managed-hook-chain-v41.json.zlib").write_bytes(zlib.compress((tmp / "managed-hook-chain.json").read_bytes(), 9))
        run("TestHaloRustManagedHookResync")
        (film / "fixtures/managed-hook-resync-v41.json.zlib").write_bytes(zlib.compress((tmp / "managed-hook-resync.json").read_bytes(), 9))
        run("TestHaloRustManagedHookHarvest")
        (film / "fixtures/managed-hook-harvest-v41.json.zlib").write_bytes(zlib.compress((tmp / "managed-hook-harvest.json").read_bytes(), 9))
        run("TestHaloRustManagedHookRepair")
        (film / "fixtures/managed-hook-repair-v41.json.zlib").write_bytes(zlib.compress((tmp / "managed-hook-repair.json").read_bytes(), 9))
        run("TestHaloRustEngineHookReads")
        (film / "fixtures/engine-hook-reads-v41.json.zlib").write_bytes(zlib.compress((tmp / "engine-hook-reads.json").read_bytes(), 9))
        run("TestHaloRustEngineHookChain")
        (film / "fixtures/engine-hook-chain-v41.json.zlib").write_bytes(zlib.compress((tmp / "engine-hook-chain.json").read_bytes(), 9))
        run("TestHaloRustEngineHookResync")
        (film / "fixtures/engine-hook-resync-v41.json.zlib").write_bytes(zlib.compress((tmp / "engine-hook-resync.json").read_bytes(), 9))
        run("TestHaloRustEngineHookHarvest")
        (film / "fixtures/engine-hook-harvest-v41.json.zlib").write_bytes(zlib.compress((tmp / "engine-hook-harvest.json").read_bytes(), 9))
        run("TestHaloRustEngineHookRepair")
        (film / "fixtures/engine-hook-repair-v41.json.zlib").write_bytes(zlib.compress((tmp / "engine-hook-repair.json").read_bytes(), 9))
        run("TestHaloRustPlayerHookReads")
        (film / "fixtures/player-hook-reads-v41.json.zlib").write_bytes(zlib.compress((tmp / "player-hook-reads.json").read_bytes(), 9))
        run("TestHaloRustPlayerHookChain")
        (film / "fixtures/player-hook-chain-v41.json.zlib").write_bytes(zlib.compress((tmp / "player-hook-chain.json").read_bytes(), 9))
        run("TestHaloRustPlayerHookResync")
        (film / "fixtures/player-hook-resync-v41.json.zlib").write_bytes(zlib.compress((tmp / "player-hook-resync.json").read_bytes(), 9))
        run("TestHaloRustPlayerHookHarvest")
        (film / "fixtures/player-hook-harvest-v41.json.zlib").write_bytes(zlib.compress((tmp / "player-hook-harvest.json").read_bytes(), 9))
        run("TestHaloRustPlayerHookRepair")
        (film / "fixtures/player-hook-repair-v41.json.zlib").write_bytes(zlib.compress((tmp / "player-hook-repair.json").read_bytes(), 9))
        run("TestHaloRustProbeHookReads")
        (film / "fixtures/probe-hook-reads-v41.json.zlib").write_bytes(zlib.compress((tmp / "probe-hook-reads.json").read_bytes(), 9))
        run("TestHaloRustProbeHookChain")
        (film / "fixtures/probe-hook-chain-v41.json.zlib").write_bytes(zlib.compress((tmp / "probe-hook-chain.json").read_bytes(), 9))
        run("TestHaloRustProbeHookResync")
        (film / "fixtures/probe-hook-resync-v41.json.zlib").write_bytes(zlib.compress((tmp / "probe-hook-resync.json").read_bytes(), 9))
        run("TestHaloRustProbeHookHarvest")
        (film / "fixtures/probe-hook-harvest-v41.json.zlib").write_bytes(zlib.compress((tmp / "probe-hook-harvest.json").read_bytes(), 9))
        run("TestHaloRustProbeHookRepair")
        (film / "fixtures/probe-hook-repair-v41.json.zlib").write_bytes(zlib.compress((tmp / "probe-hook-repair.json").read_bytes(), 9))
        run("TestHaloRustMovementHookRecoverySlots")
        (film / "fixtures/movement-hook-recovery-slots-v41.json.zlib").write_bytes(zlib.compress((tmp / "movement-hook-recovery-slots.json").read_bytes(), 9))
        for kind in ("Reads", "Production", "Generic", "Inference", "Harvest", "Resync", "Chain", "Repair"):
            run(f"TestHaloRustMovementHook{kind}")
            (film / f"fixtures/movement-hook-{kind.lower()}-v41.json.zlib").write_bytes(zlib.compress((tmp / f"movement-hook-{kind.lower()}.json").read_bytes(), 9))
        for kind in ("Reads", "Keyframes", "Chain", "Repair", "Frames"):
            run(f"TestHaloRustDefaultHook{kind}")
            (film / f"fixtures/default-hook-{kind.lower()}-v41.json.zlib").write_bytes(zlib.compress((tmp / f"default-hook-{kind.lower()}.json").read_bytes(), 9))
        run("TestHaloRustEquipmentHookReads")
        (film / "fixtures/equipment-hook-reads-v41.json.zlib").write_bytes(zlib.compress((tmp / "equipment-hook-reads.json").read_bytes(), 9))
        run("TestHaloRustEquipmentHookChain")
        (film / "fixtures/equipment-hook-chain-v41.json.zlib").write_bytes(zlib.compress((tmp / "equipment-hook-chain.json").read_bytes(), 9))
        run("TestHaloRustEquipmentHookResync")
        (film / "fixtures/equipment-hook-resync-v41.json.zlib").write_bytes(zlib.compress((tmp / "equipment-hook-resync.json").read_bytes(), 9))
        run("TestHaloRustEquipmentHookHarvest")
        (film / "fixtures/equipment-hook-harvest-v41.json.zlib").write_bytes(zlib.compress((tmp / "equipment-hook-harvest.json").read_bytes(), 9))
        run("TestHaloRustEquipmentHookRepair")
        (film / "fixtures/equipment-hook-repair-v41.json.zlib").write_bytes(zlib.compress((tmp / "equipment-hook-repair.json").read_bytes(), 9))
        run("TestHaloRustRegistryEdges")
        (film / "fixtures/registry-edges-v41.json.zlib").write_bytes(zlib.compress((tmp / "registry-edges.json").read_bytes(), 9))
        run("TestHaloRustRegistryWarning")
        (film / "fixtures/registry-warning-v41.json.zlib").write_bytes(zlib.compress((tmp / "registry-warning.json").read_bytes(), 9))
        run("TestHaloRustUnknownBuild")
        (film / "fixtures/unknown-build-v41.json.zlib").write_bytes(zlib.compress((tmp / "unknown-build.json").read_bytes(), 9))
        run("TestHaloRustRawResyncDiagnostics")
        (film / "fixtures/raw-resync-diagnostics-v41.json.zlib").write_bytes(zlib.compress((tmp / "raw-resync-diagnostics.json").read_bytes(), 9))
        run("TestHaloRustHarvestDiagnostics")
        (film / "fixtures/harvest-diagnostics-v41.json.zlib").write_bytes(zlib.compress((tmp / "harvest-diagnostics.json").read_bytes(), 9))
        run("TestHaloRustRepairDiagnostics")
        (film / "fixtures/repair-diagnostics-v41.json.zlib").write_bytes(zlib.compress((tmp / "repair-diagnostics.json").read_bytes(), 9))
        run("TestHaloRustResyncDiagnostics")
        (film / "fixtures/resync-diagnostics-v41.json.zlib").write_bytes(zlib.compress((tmp / "resync-diagnostics.json").read_bytes(), 9))
        run("TestHaloRustFrameDiagnostics")
        (film / "fixtures/frame-diagnostics-v41.json.zlib").write_bytes(zlib.compress((tmp / "frame-diagnostics.json").read_bytes(), 9))
        run("TestHaloRustReadDiagnostics")
        (film / "fixtures/read-diagnostics-v41.json.zlib").write_bytes(zlib.compress((tmp / "read-diagnostics.json").read_bytes(), 9))
        run("TestHaloRustChainDiagnostics")
        (film / "fixtures/chain-diagnostics-v41.json.zlib").write_bytes(zlib.compress((tmp / "chain-diagnostics.json").read_bytes(), 9))
        run("TestHaloRustWeaponPatterns", "internal/grammar/weaponscan")
        (film / "fixtures/weapon-patterns-v41.json.zlib").write_bytes(zlib.compress((tmp / "weapon-patterns.json").read_bytes(), 9))
        run("TestHaloRustWeaponPatternsCorpus", "internal/grammar/weaponscan")
        (film / "fixtures/weapon-patterns-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "weapon-patterns-corpus.json").read_bytes(), 9))
        (reference / "filmshell-catalog-v41.json").write_bytes((tmp / "filmshell-catalog.json").read_bytes())
        run("TestHaloRustKeyframePositionCorpus", "internal/grammar/positions")
        (film / "fixtures/keyframe-position-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-position-corpus.json").read_bytes(), 9))
        run("TestHaloRustKeyframePositionProbe", "internal/grammar/positions")
        run("TestHaloRustUnitEquipment")
        (film / "fixtures/keyframe-position-probe-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-position-probe.json").read_bytes(), 9))
        (film / "fixtures/unit-equipment-v41.json.zlib").write_bytes(zlib.compress((tmp / "unit-equipment.json").read_bytes(), 9))
        run("TestHaloRustUnitEquipmentCorpus")
        (film / "fixtures/unit-equipment-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "unit-equipment-corpus.json").read_bytes(), 9))
        for kind in ("Position", "Movement"):
            run(f"TestHaloRust{kind}HookMarch")
            (film / f"fixtures/{kind.lower()}-hook-march-v41.json.zlib").write_bytes(zlib.compress((tmp / f"{kind.lower()}-hook-march.json").read_bytes(), 9))
        text = (reference / "halo_rust_film_context_observers_test.go.txt").read_text()
        (grammar / "halo_rust_film_context_observers_test.go").write_text(text)
        run("TestHaloRustFilmContextObservers")
        for kind in ("Sequence", "Generic", "Inference", "Keyframes"):
            name = f"halo_rust_calibrated_position_{kind.lower()}_test.go"
            text = (reference / (name + ".txt")).read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar / name).write_text(text)
            run(f"TestHaloRustCalibratedPosition{kind}")
            (film / f"fixtures/calibrated-position-{kind.lower()}-v41.json.zlib").write_bytes(zlib.compress((tmp / f"calibrated-position-{kind.lower()}.json").read_bytes(), 9))
        for kind in ("Generic", "Inference", "Recovery"):
            name = f"halo_rust_new_record_profile_{kind.lower()}_test.go"
            text = (reference / (name + ".txt")).read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar / name).write_text(text)
            run(f"TestHaloRustNewRecordProfile{kind}")
            (film / f"fixtures/new-record-profile-{kind.lower()}-v41.json.zlib").write_bytes(zlib.compress((tmp / f"new-record-profile-{kind.lower()}.json").read_bytes(), 9))
        for kind in ("Generic", "Inference", "Keyframes", "Repair"):
            name = f"halo_rust_width_override_{kind.lower()}_test.go"
            text = (reference / (name + ".txt")).read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar / name).write_text(text)
            run(f"TestHaloRustWidthOverride{kind}")
            (film / f"fixtures/width-override-{kind.lower()}-v41.json.zlib").write_bytes(zlib.compress((tmp / f"width-override-{kind.lower()}.json").read_bytes(), 9))
        for kind in ("Sequence", "Generic", "Inference", "Keyframes"):
            name = f"halo_rust_baseline_scope_{kind.lower()}_test.go"
            text = (reference / (name + ".txt")).read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar / name).write_text(text)
            run(f"TestHaloRustBaselineScope{kind}")
            (film / f"fixtures/baseline-scope-{kind.lower()}-v41.json.zlib").write_bytes(zlib.compress((tmp / f"baseline-scope-{kind.lower()}.json").read_bytes(), 9))
        text = (reference / "halo_rust_keyframe_simulation_policy_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_keyframe_simulation_policy_test.go").write_text(text)
        text = (reference / "halo_rust_medal_names_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_medal_names_test.go").write_text(text)
        text = (reference / "halo_rust_kill_icons_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "killicon/halo_rust_kill_icons_test.go").write_text(text)
        (tmp / "film-key-base.bin").write_bytes(zlib.decompress((film / "fixtures/bootstrap-v41.zlib").read_bytes()))
        text = (reference / "halo_rust_film_key_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_film_key_test.go").write_text(text)
        subprocess.run(["python3", str(reference / "extract_registry_fingerprints.py"),
                        str(grammar.parent / "profile/registre_empreintes.go"),
                        str(reference / "registry-fingerprints-v75.json")], check=True)
        text = (reference / "halo_rust_registry_catalog_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_registry_catalog_test.go").write_text(text)
        run("TestHaloRustRegistryCatalog")
        (film / "fixtures/registry-catalog-v41.json.zlib").write_bytes(zlib.compress((tmp / "registry-catalog.json").read_bytes(), 9))
        run("TestHaloRustFilmKey", "replay")
        (film / "fixtures/film-key-v41.json.zlib").write_bytes(zlib.compress((tmp / "film-key.json").read_bytes(), 9))
        run("TestHaloRustKillIcons", "killicon")
        (film / "fixtures/kill-icons-v41.json.zlib").write_bytes(zlib.compress((tmp / "kill-icons.json").read_bytes(), 9))
        run("TestHaloRustMedalNames")
        (film / "fixtures/medal-names-v41.json.zlib").write_bytes(zlib.compress((tmp / "medal-names.json").read_bytes(), 9))
        text = (reference / "halo_rust_keyframe_layout_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_keyframe_layout_test.go").write_text(text)
        text = (reference / "halo_rust_keyframe_type_word_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_keyframe_type_word_test.go").write_text(text)
        text = (reference / "halo_rust_keyframe_chain_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_keyframe_chain_test.go").write_text(text)
        text = (reference / "halo_rust_keyframe_native_table_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_keyframe_native_table_test.go").write_text(text)
        text = (reference / "halo_rust_probe_trace_values_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_probe_trace_values_test.go").write_text(text)
        run("TestHaloRustProbeTraceValues")
        (film / "fixtures/probe-trace-values-v41.json.zlib").write_bytes(zlib.compress((tmp / "probe-trace-values.json").read_bytes(), 9))
        text = (reference / "halo_rust_captured_payloads_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_captured_payloads_test.go").write_text(text)
        run("TestHaloRustCapturedPayloads")
        (film / "fixtures/captured-payloads-v41.json.zlib").write_bytes(zlib.compress((tmp / "captured-payloads.json").read_bytes(), 9))
        text = (reference / "halo_rust_component_probe_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_component_probe_test.go").write_text(text)
        run("TestHaloRustComponentProbe")
        (film / "fixtures/component-probe-v41.json.zlib").write_bytes(zlib.compress((tmp / "component-probe.json").read_bytes(), 9))
        text = (reference / "halo_rust_keyframe_biped_probe_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_keyframe_biped_probe_test.go").write_text(text)
        run("TestHaloRustKeyframeBipedProbe")
        (film / "fixtures/keyframe-biped-probe-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-biped-probe.json").read_bytes(), 9))
        text = (reference / "halo_rust_cache_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "filmcache/halo_rust_cache_test.go").write_text(text)
        run("TestHaloRustCache", "filmcache")
        run("TestHaloRustCacheWrite", "filmcache")
        run("TestHaloRustCachePaths", "filmcache")
        run("TestHaloRustCachePartialMetadata", "filmcache")
        (film / "fixtures/cache-paths-v41.json").write_bytes((tmp / "cache-paths.json").read_bytes())
        (film / "fixtures/cache-writer-v41.json").write_bytes((tmp / "cache-writer.json").read_bytes())
        (film / "fixtures/cache-reader-v41.json").write_bytes((tmp / "cache-reader.json").read_bytes())
        (film / "fixtures/cache-partial-metadata-v41.json").write_bytes((tmp / "cache-partial-metadata.json").read_bytes())
        for stem, test in [("source", "TestHaloRustSource"), ("source_provider", "TestHaloRustSourceProvider"), ("source_directory", "TestHaloRustSourceDirectory"), ("source_directory_lifecycle", "TestHaloRustSourceDirectoryLifecycle")]:
            text = (reference / f"halo_rust_{stem}_test.go.txt").read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar.parent / f"source/halo_rust_{stem}_test.go").write_text(text)
            run(test, "internal/source")
        (film / "fixtures/source-provider-v41.json.zlib").write_bytes(zlib.compress((tmp / "source-provider.json").read_bytes(), 9))
        (film / "fixtures/source-v41.json.zlib").write_bytes(zlib.compress((tmp / "source.json").read_bytes(), 9))
        (film / "fixtures/source-directory-lifecycle-v41.json.zlib").write_bytes(zlib.compress((tmp / "source-directory-lifecycle.json").read_bytes(), 9))
        (film / "fixtures/source-directory-v41.json").write_bytes((tmp / "source-directory.json").read_bytes())
        text = (reference / "halo_rust_leaf_readers_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_leaf_readers_test.go").write_text(text)
        run("TestHaloRustLeafReaders")
        (film / "fixtures/leaf-readers-v41.json.zlib").write_bytes(zlib.compress((tmp / "leaf-readers.json").read_bytes(), 9))
        text = (reference / "halo_rust_damage_catalog_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "damagetag/halo_rust_damage_catalog_test.go").write_text(text)
        run("TestHaloRustDamageCatalog", "damagetag")
        (film / "fixtures/damage-catalog-v41.json.zlib").write_bytes(zlib.compress((tmp / "damage-catalog.json").read_bytes(), 9))
        for stem, test in [("ground_ammo_profile", "TestHaloRustGroundAmmoProfile"), ("ground_creation_profile", "TestHaloRustGroundCreationProfile")]:
            text = (reference / f"halo_rust_{stem}_test.go.txt").read_text()
            for old, new in paths.items():
                text = text.replace(old, new)
            (grammar / f"halo_rust_{stem}_test.go").write_text(text)
            run(test)
            name = stem.replace("_", "-")
            (film / f"fixtures/{name}-v41.json.zlib").write_bytes(zlib.compress((tmp / f"{name}.json").read_bytes(), 9))
        text = (reference / "halo_rust_chunk_bridge_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_chunk_bridge_test.go").write_text(text)
        run("TestHaloRustChunkBridge")
        (film / "fixtures/chunk-bridge-v41.json.zlib").write_bytes(zlib.compress((tmp / "chunk-bridge.json").read_bytes(), 9))
        text = (reference / "halo_rust_packet_files_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_packet_files_test.go").write_text(text)
        run("TestHaloRustPacketFiles")
        (film / "fixtures/packet-files-v41.json").write_bytes((tmp / "packet-files.json").read_bytes())
        run("TestHaloRustKeyframeNativeTable")
        (film / "fixtures/keyframe-native-table-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-native-table.json").read_bytes(), 9))
        text = (reference / "halo_rust_corruption_context_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_corruption_context_test.go").write_text(text)
        run("TestHaloRustCorruptionContext")
        text = (reference / "halo_rust_corruption_source_test.go.txt").read_text()
        text = text.replace("@BOOTSTRAP_FIXTURE@", json.dumps(str(film / "fixtures/bootstrap-v41.zlib"))[1:-1])
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_corruption_source_test.go").write_text(text)
        run("TestHaloRustCorruptionSource")
        (film / "fixtures/corruption-source-v41.json.zlib").write_bytes(zlib.compress((tmp / "corruption-source.json").read_bytes(), 9))
        (film / "fixtures/corruption-context-v41.json.zlib").write_bytes(zlib.compress((tmp / "corruption-context.json").read_bytes(), 9))
        run("TestHaloRustKeyframeChain")
        (film / "fixtures/keyframe-chain-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-chain.json").read_bytes(), 9))
        run("TestHaloRustKeyframeTypeWord")
        (film / "fixtures/keyframe-type-word-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-type-word.json").read_bytes(), 9))
        run("TestHaloRustKeyframeLayout")
        (film / "fixtures/keyframe-layout-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-layout.json").read_bytes(), 9))
        run("TestHaloRustKeyframeSimulationPolicy")
        (film / "fixtures/keyframe-simulation-policy-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-simulation-policy.json").read_bytes(), 9))
        run("TestHaloRustPositionHookSequence")
        (film / "fixtures/position-hook-sequence-v41.json.zlib").write_bytes(zlib.compress((tmp / "position-hook-sequence.json").read_bytes(), 9))
        for kind, fixture in (("Resolved","resolved"),("Production","production"),("Inference","inference"),("Generic","generic"),("Locator","locator"),("RecoverySlots","recovery-slots"),("Keyframes","keyframes"),("Harvest","harvest"),("Resync","resync"),("Chain","chain"),("Repair","repair"),("ValidatedResync","validated-resync")):
            run(f"TestHaloRustPositionHook{kind}")
            (film / f"fixtures/position-hook-{fixture}-v41.json.zlib").write_bytes(zlib.compress((tmp / f"position-hook-{fixture}.json").read_bytes(), 9))
        run("TestHaloRustAbsoluteWidthBoundaries")
        (film / "fixtures/absolute-width-boundaries-v41.json.zlib").write_bytes(zlib.compress((tmp / "absolute-width-boundaries.json").read_bytes(), 9))
        run("TestHaloRustLazyWidths")
        (film / "fixtures/lazy-widths-v41.json.zlib").write_bytes(zlib.compress((tmp / "lazy-widths.json").read_bytes(), 9))
        run("TestHaloRustDeltaWidthFallback")
        (film / "fixtures/delta-width-fallback-v41.json.zlib").write_bytes(zlib.compress((tmp / "delta-width-fallback.json").read_bytes(), 9))
        run("TestHaloRustPositionCapture")
        (film / "fixtures/position-capture-v41.json.zlib").write_bytes(zlib.compress((tmp / "position-capture.json").read_bytes(), 9))
        run("TestHaloRustFrameResync")
        (film / "fixtures/frame-resync-v41.json.zlib").write_bytes(zlib.compress((tmp / "frame-resync.json").read_bytes(), 9))
        run("TestHaloRustFrameTargets")
        (film / "fixtures/frame-targets-v41.json.zlib").write_bytes(zlib.compress((tmp / "frame-targets.json").read_bytes(), 9))
        run("TestHaloRustI0Values")
        (film / "fixtures/i0-values-v41.json.zlib").write_bytes(zlib.compress((tmp / "i0-values.json").read_bytes(), 9))
        run("TestHaloRustI0Loaded")
        (film / "fixtures/i0-loaded-v41.json.zlib").write_bytes(zlib.compress((tmp / "i0-loaded.json").read_bytes(), 9))
        text = (reference / "halo_rust_context_cache_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_context_cache_test.go").write_text(text)
        run("TestHaloRustContextCache")
        (film / "fixtures/context-cache-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-cache.json").read_bytes(), 9))
        text = (reference / "halo_rust_profile_source_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_profile_source_test.go").write_text(text)
        run("TestHaloRustProfileSource")
        (film / "fixtures/profile-source-v41.json.zlib").write_bytes(zlib.compress((tmp / "profile-source.json").read_bytes(), 9))
        text = (reference / "halo_rust_scan_profile_sequence_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_scan_profile_sequence_test.go").write_text(text)
        run("TestHaloRustScanProfileSequence")
        (film / "fixtures/scan-profile-sequence-v41.json.zlib").write_bytes(zlib.compress((tmp / "scan-profile-sequence.json").read_bytes(), 9))
        text = (reference / "halo_rust_context_reader_sequence_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_context_reader_sequence_test.go").write_text(text)
        run("TestHaloRustContextReaderSequence")
        run("TestHaloRustSignedVariableReader")
        (film / "fixtures/signed-variable-reader-v41.json.zlib").write_bytes(zlib.compress((tmp / "signed-variable-reader.json").read_bytes(), 9))
        (film / "fixtures/context-reader-sequence-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-reader-sequence.json").read_bytes(), 9))
        text = (reference / "halo_rust_live_observer_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_live_observer_test.go").write_text(text)
        run("TestHaloRustLiveObserver")
        (film / "fixtures/live-observer-v41.json.zlib").write_bytes(zlib.compress((tmp / "live-observer.json").read_bytes(), 9))
        text = (reference / "halo_rust_record_id_policy_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_record_id_policy_test.go").write_text(text)
        run("TestHaloRustRecordIDPolicy")
        (film / "fixtures/record-id-policy-v41.json.zlib").write_bytes(zlib.compress((tmp / "record-id-policy.json").read_bytes(), 9))
        text = (reference / "halo_rust_mpp_width_policy_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_mpp_width_policy_test.go").write_text(text)
        run("TestHaloRustMPPWidthPolicy")
        (film / "fixtures/mpp-width-policy-v41.json.zlib").write_bytes(zlib.compress((tmp / "mpp-width-policy.json").read_bytes(), 9))
        text = (reference / "halo_rust_new_default_policy_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_new_default_policy_test.go").write_text(text)
        run("TestHaloRustNewDefaultPolicy")
        (film / "fixtures/new-default-policy-v41.json.zlib").write_bytes(zlib.compress((tmp / "new-default-policy.json").read_bytes(), 9))
        text = (reference / "halo_rust_new_tail_policy_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_new_tail_policy_test.go").write_text(text)
        run("TestHaloRustNewTailLarge")
        (film / "fixtures/new-tail-large-v41.json.zlib").write_bytes(zlib.compress((tmp / "new-tail-large.json").read_bytes(), 9))
        run("TestHaloRustNewTailPolicy")
        (film / "fixtures/new-tail-policy-v41.json.zlib").write_bytes(zlib.compress((tmp / "new-tail-policy.json").read_bytes(), 9))
        text = (reference / "halo_rust_context_frame_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_context_frame_test.go").write_text(text)
        text = (reference / "halo_rust_world_position_widths_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_world_position_widths_test.go").write_text(text)
        run("TestHaloRustWorldPositionWidths")
        (film / "fixtures/world-position-widths-v41.json.zlib").write_bytes(zlib.compress((tmp / "world-position-widths.json").read_bytes(), 9))
        run("TestHaloRustContextFrameLazyWidths")
        (film / "fixtures/context-frame-lazy-widths-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-frame-lazy-widths.json").read_bytes(), 9))
        run("TestHaloRustContextFrame")
        (film / "fixtures/context-frame-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-frame.json").read_bytes(), 9))
        text = (reference / "halo_rust_context_views_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_context_views_test.go").write_text(text)
        run("TestHaloRustContextViewsLazyWidths")
        (film / "fixtures/context-views-lazy-widths-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-views-lazy-widths.json").read_bytes(), 9))
        run("TestHaloRustContextViews")
        (film / "fixtures/context-views-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-views.json").read_bytes(), 9))
        text = (reference / "halo_rust_context_inference_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_context_inference_test.go").write_text(text)
        run("TestHaloRustContextInferenceLazyWidths")
        (film / "fixtures/context-inference-lazy-widths-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-inference-lazy-widths.json").read_bytes(), 9))
        run("TestHaloRustContextInference")
        (film / "fixtures/context-inference-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-inference.json").read_bytes(), 9))
        text = (reference / "halo_rust_equipment_recovery_source_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_equipment_recovery_source_test.go").write_text(text)
        run("TestHaloRustEquipmentRecoverySource")
        (film / "fixtures/equipment-recovery-source-v41.json.zlib").write_bytes(zlib.compress((tmp / "equipment-recovery-source.json").read_bytes(), 9))
        text = (reference / "halo_rust_live_chain_budget_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_live_chain_budget_test.go").write_text(text)
        run("TestHaloRustLiveChainBudget")
        (film / "fixtures/live-chain-budget-v41.json.zlib").write_bytes(zlib.compress((tmp / "live-chain-budget.json").read_bytes(), 9))
        text = (reference / "halo_rust_context_chain_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_context_chain_test.go").write_text(text)
        run("TestHaloRustContextChainLazyWidths")
        (film / "fixtures/context-chain-lazy-widths-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-chain-lazy-widths.json").read_bytes(), 9))
        run("TestHaloRustContextChain")
        (film / "fixtures/context-chain-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-chain.json").read_bytes(), 9))
        text = (reference / "halo_rust_context_resync_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_context_resync_test.go").write_text(text)
        run("TestHaloRustContextResyncLazyWidths")
        (film / "fixtures/context-resync-lazy-widths-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-resync-lazy-widths.json").read_bytes(), 9))
        run("TestHaloRustContextResync")
        (film / "fixtures/context-resync-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-resync.json").read_bytes(), 9))
        text = (reference / "halo_rust_context_repair_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_context_repair_test.go").write_text(text)
        run("TestHaloRustContextRepairLazyWidths")
        (film / "fixtures/context-repair-lazy-widths-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-repair-lazy-widths.json").read_bytes(), 9))
        run("TestHaloRustContextRepair")
        (film / "fixtures/context-repair-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-repair.json").read_bytes(), 9))
        text = (reference / "halo_rust_observer_copy_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_observer_copy_test.go").write_text(text)
        run("TestHaloRustObserverCopy")
        (film / "fixtures/observer-copy-v41.json.zlib").write_bytes(zlib.compress((tmp / "observer-copy.json").read_bytes(), 9))
        text = (reference / "halo_rust_context_raw_resync_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_context_raw_resync_test.go").write_text(text)
        run("TestHaloRustContextRawResync")
        (film / "fixtures/context-raw-resync-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-raw-resync.json").read_bytes(), 9))
        text = (reference / "halo_rust_context_resync_frame_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_context_resync_frame_test.go").write_text(text)
        run("TestHaloRustContextResyncFrame")
        (film / "fixtures/context-resync-frame-v41.json.zlib").write_bytes(zlib.compress((tmp / "context-resync-frame.json").read_bytes(), 9))
        text = (reference / "halo_rust_signed_width_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_signed_width_test.go").write_text(text)
        run("TestHaloRustSignedWidth")
        (film / "fixtures/signed-width-v41.json.zlib").write_bytes(zlib.compress((tmp / "signed-width.json").read_bytes(), 9))
        text = (reference / "halo_rust_large_width_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_large_width_test.go").write_text(text)
        run("TestHaloRustLargeWidth")
        (film / "fixtures/large-width-v41.json.zlib").write_bytes(zlib.compress((tmp / "large-width.json").read_bytes(), 9))
        text = (reference / "halo_rust_movement_positive_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_movement_positive_test.go").write_text(text)
        run("TestHaloRustMovementPositive")
        (film / "fixtures/movement-positive-v41.json.zlib").write_bytes(zlib.compress((tmp / "movement-positive.json").read_bytes(), 9))
        text = (reference / "halo_rust_position_accumulator_generic_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_position_accumulator_generic_test.go").write_text(text)
        run("TestHaloRustPositionAccumulatorGeneric")
        (film / "fixtures/position-accumulator-generic-v41.json.zlib").write_bytes(zlib.compress((tmp / "position-accumulator-generic.json").read_bytes(), 9))
        text = (reference / "halo_rust_position_accumulator_shared_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_position_accumulator_shared_test.go").write_text(text)
        run("TestHaloRustPositionAccumulatorShared")
        (film / "fixtures/position-accumulator-shared-v41.json.zlib").write_bytes(zlib.compress((tmp / "position-accumulator-shared.json").read_bytes(), 9))
        text = (reference / "halo_rust_mobility_extra_policy_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_mobility_extra_policy_test.go").write_text(text)
        run("TestHaloRustMobilityExtraPolicy")
        (film / "fixtures/mobility-extra-policy-v41.json.zlib").write_bytes(zlib.compress((tmp / "mobility-extra-policy.json").read_bytes(), 9))
        text = (reference / "halo_rust_mobility_extra_large_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_mobility_extra_large_test.go").write_text(text)
        run("TestHaloRustMobilityExtraLarge")
        (film / "fixtures/mobility-extra-large-v41.json.zlib").write_bytes(zlib.compress((tmp / "mobility-extra-large.json").read_bytes(), 9))
        run("TestHaloRustI0Layout")
        (film / "fixtures/i0-layout-v41.json.zlib").write_bytes(zlib.compress((tmp / "i0-layout.json").read_bytes(), 9))
        run("TestHaloRustWorldPrecision", "replay")
        (film / "fixtures/world-precision-v41.json.zlib").write_bytes(zlib.compress((tmp / "world-precision.json").read_bytes(), 9))
        run("TestHaloRustWeaponHelpers", "internal/grammar/weaponv3")
        (film / "fixtures/weapon-helpers-v41.json.zlib").write_bytes(zlib.compress((tmp / "weapon-helpers.json").read_bytes(), 9))
        run("TestHaloRustWeaponHitCorpus")
        (film / "fixtures/weapon-hit-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "weapon-hit-corpus.json").read_bytes(), 9))
        run("TestHaloRustWeaponHitScan")
        (film / "fixtures/weapon-hit-scan-v41.json.zlib").write_bytes(zlib.compress((tmp / "weapon-hit-scan.json").read_bytes(), 9))
        run("TestHaloRustWeaponHits")
        run("TestHaloRustFlagGrabBridge", "replay")
        (film / "fixtures/weapon-hits-v41.json.zlib").write_bytes(zlib.compress((tmp / "weapon-hits.json").read_bytes(), 9))
        (film / "fixtures/flag-grab-bridge-v41.json.zlib").write_bytes(zlib.compress((tmp / "flag-grab-bridge.json").read_bytes(), 9))
        run("TestHaloRustUsage", "replay")
        (film / "fixtures/usage-v41.json.zlib").write_bytes(zlib.compress((tmp / "usage.json").read_bytes(), 9))
        run("TestHaloRustFilmPlayers", "replay")
        (film / "fixtures/film-players-v41.json.zlib").write_bytes(zlib.compress((tmp / "film-players.json").read_bytes(), 9))
        run("TestHaloRustHighlightsCorpus", "replay")
        (film / "fixtures/highlights-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "highlights-corpus.json").read_bytes(), 9))
        run("TestHaloRustHighlights")
        (film / "fixtures/highlights-v41.json.zlib").write_bytes(zlib.compress((tmp / "highlights.json").read_bytes(), 9))
        run("TestHaloRustReplayPlayers", "replay")
        (film / "fixtures/replay-players-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-players.json").read_bytes(), 9))
        run("TestHaloRustReplaySeats", "replay")
        (film / "fixtures/replay-seats-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-seats.json").read_bytes(), 9))
        run("TestHaloRustReplayTeams", "replay")
        (film / "fixtures/replay-teams-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-teams.json").read_bytes(), 9))
        run("TestHaloRustPlayerTeamsCorpus")
        (film / "fixtures/player-teams-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "player-teams-corpus.json").read_bytes(), 9))
        run("TestHaloRustPlayerTeams")
        (film / "fixtures/player-teams-v41.json.zlib").write_bytes(zlib.compress((tmp / "player-teams.json").read_bytes(), 9))
        run("TestHaloRustReplayBounds", "replay")
        (film / "fixtures/replay-bounds-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-bounds.json").read_bytes(), 9))
        run("TestHaloRustReplayTracks", "replay")
        (film / "fixtures/replay-tracks-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-tracks.json").read_bytes(), 9))
        run("TestHaloRustIdentitySuccessions", "replay")
        (film / "fixtures/identity-successions-v41.json.zlib").write_bytes(zlib.compress((tmp / "identity-successions.json").read_bytes(), 9))
        run("TestHaloRustIdentityRemaining", "replay")
        (film / "fixtures/identity-remaining-v41.json.zlib").write_bytes(zlib.compress((tmp / "identity-remaining.json").read_bytes(), 9))
        run("TestHaloRustStatborgNamed", "internal/facts/objectives")
        (film / "fixtures/statborg-named-v41.json.zlib").write_bytes(zlib.compress((tmp / "statborg-named.json").read_bytes(), 9))
        run("TestHaloRustStatborgResidue", "internal/facts/objectives")
        (film / "fixtures/statborg-residue-v41.json.zlib").write_bytes(zlib.compress((tmp / "statborg-residue.json").read_bytes(), 9))
        text = (reference / "halo_rust_identity_completion_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent / "facts/objectives/halo_rust_identity_completion_test.go").write_text(text)
        run("TestHaloRustIdentityCompletion", "internal/facts/objectives")
        (film / "fixtures/identity-completion-v41.json.zlib").write_bytes(zlib.compress((tmp / "identity-completion.json").read_bytes(), 9))
        run("TestHaloRustStatborgIdentity", "internal/facts/objectives")
        (film / "fixtures/statborg-identity-v41.json.zlib").write_bytes(zlib.compress((tmp / "statborg-identity.json").read_bytes(), 9))
        run("TestHaloRustStatborgSeries", "internal/facts/objectives")
        (film / "fixtures/statborg-series-v41.json.zlib").write_bytes(zlib.compress((tmp / "statborg-series.json").read_bytes(), 9))
        run("TestHaloRustStatborgCorpus", "internal/facts/objectives")
        (film / "fixtures/statborg-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "statborg-corpus.json").read_bytes(), 9))
        run("TestHaloRustStatborgDecode", "internal/facts/objectives")
        (film / "fixtures/statborg-decode-v41.json.zlib").write_bytes(zlib.compress((tmp / "statborg-decode.json").read_bytes(), 9))
        run("TestHaloRustStatborgRounds", "internal/facts/objectives")
        (film / "fixtures/statborg-rounds-v41.json.zlib").write_bytes(zlib.compress((tmp / "statborg-rounds.json").read_bytes(), 9))
        run("TestHaloRustIdentityRegistry", "replay")
        (film / "fixtures/identity-registry-v41.json.zlib").write_bytes(zlib.compress((tmp / "identity-registry.json").read_bytes(), 9))
        run("TestHaloRustIdentityOwners", "replay")
        (film / "fixtures/identity-owners-v41.json.zlib").write_bytes(zlib.compress((tmp / "identity-owners.json").read_bytes(), 9))
        run("TestHaloRustIdentityClosures", "replay")
        (film / "fixtures/identity-closures-v41.json.zlib").write_bytes(zlib.compress((tmp / "identity-closures.json").read_bytes(), 9))
        run("TestHaloRustBotCorpus", "internal/facts/killsource")
        (film / "fixtures/bot-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "bot-corpus.json").read_bytes(), 9))
        run("TestHaloRustBotMetadata", "internal/facts/killsource")
        (film / "fixtures/bot-metadata-v41.json.zlib").write_bytes(zlib.compress((tmp / "bot-metadata.json").read_bytes(), 9))
        run("TestHaloRustIdentityScoreboard", "replay")
        (film / "fixtures/identity-scoreboard-v41.json.zlib").write_bytes(zlib.compress((tmp / "identity-scoreboard.json").read_bytes(), 9))
        run("TestHaloRustIdentityDeathClock", "replay")
        (film / "fixtures/identity-death-clock-v41.json.zlib").write_bytes(zlib.compress((tmp / "identity-death-clock.json").read_bytes(), 9))
        run("TestHaloRustIdentityLifetimes", "replay")
        (film / "fixtures/identity-lifetimes-v41.json.zlib").write_bytes(zlib.compress((tmp / "identity-lifetimes.json").read_bytes(), 9))
        run("TestHaloRustIdentityCreations", "replay")
        (film / "fixtures/identity-creations-v41.json.zlib").write_bytes(zlib.compress((tmp / "identity-creations.json").read_bytes(), 9))
        run("TestHaloRustPlayerIndices", "replay")
        (film / "fixtures/player-indices-v41.json.zlib").write_bytes(zlib.compress((tmp / "player-indices.json").read_bytes(), 9))
        run("TestHaloRustIdentityTables", "replay")
        (film / "fixtures/identity-tables-v41.json.zlib").write_bytes(zlib.compress((tmp / "identity-tables.json").read_bytes(), 9))
        run("TestHaloRustReplayIdentity", "replay")
        (film / "fixtures/replay-identity-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-identity.json").read_bytes(), 9))
        run("TestHaloRustPickupOrigin", "replay")
        (film / "fixtures/pickup-origin-v41.json.zlib").write_bytes(zlib.compress((tmp / "pickup-origin.json").read_bytes(), 9))
        run("TestHaloRustReplayPickups", "replay")
        (film / "fixtures/replay-pickups-v41.json.zlib").write_bytes(zlib.compress((tmp / "replay-pickups.json").read_bytes(), 9))
        run("TestHaloRustPadDating", "replay")
        (film / "fixtures/pad-dating-v41.json.zlib").write_bytes(zlib.compress((tmp / "pad-dating.json").read_bytes(), 9))
        run("TestHaloRustGroundPads", "replay")
        (film / "fixtures/ground-pads-v41.json.zlib").write_bytes(zlib.compress((tmp / "ground-pads.json").read_bytes(), 9))
        run("TestHaloRustGroundObjectsCorpus", "replay")
        (film / "fixtures/ground-objects-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "ground-objects-corpus.json").read_bytes(), 9))
        run("TestHaloRustGroundPadCorpus", "replay")
        (film / "fixtures/ground-pad-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "ground-pad-corpus.json").read_bytes(), 9))
        run("TestHaloRustGroundObjects", "replay")
        (film / "fixtures/ground-objects-v41.json.zlib").write_bytes(zlib.compress((tmp / "ground-objects.json").read_bytes(), 9))
        run("TestHaloRustGroundLifetimes", "replay")
        (film / "fixtures/ground-lifetimes-v41.json.zlib").write_bytes(zlib.compress((tmp / "ground-lifetimes.json").read_bytes(), 9))
        run("TestHaloRustKeyframeInventory", "replay")
        (film / "fixtures/keyframe-inventory-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-inventory.json").read_bytes(), 9))
        text = (reference / "halo_rust_inventory_packet_source_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar.parent.parent / "replay/halo_rust_inventory_packet_source_test.go").write_text(text)
        run("TestHaloRustInventoryPacketSource", "replay")
        (film / "fixtures/inventory-packet-source-v41.json.zlib").write_bytes(zlib.compress((tmp / "inventory-packet-source.json").read_bytes(), 9))
        run("TestHaloRustWorldCensus")
        (film / "fixtures/world-census-v41.json.zlib").write_bytes(zlib.compress((tmp / "world-census.json").read_bytes(), 9))
        run("TestHaloRustEquipmentPlacements")
        (film / "fixtures/equipment-placements-v41.json.zlib").write_bytes(zlib.compress((tmp / "equipment-placements.json").read_bytes(), 9))
        run("TestHaloRustGroundKeyframes")
        (film / "fixtures/ground-keyframes-v41.json.zlib").write_bytes(zlib.compress((tmp / "ground-keyframes.json").read_bytes(), 9))
        run("TestHaloRustGroundDirectory")
        (film / "fixtures/ground-directory-v41.json.zlib").write_bytes(zlib.compress((tmp / "ground-directory.json").read_bytes(), 9))
        run("TestHaloRustLoadoutSources")
        (film / "fixtures/loadout-sources-v41.json.zlib").write_bytes(zlib.compress((tmp / "loadout-sources.json").read_bytes(), 9))
        run("TestHaloRustResearchDirectory")
        (film / "fixtures/research-directory-v41.json.zlib").write_bytes(zlib.compress((tmp / "research-directory.json").read_bytes(), 9))
        run("TestHaloRustGroundCreations")
        (film / "fixtures/ground-creations-v41.json.zlib").write_bytes(zlib.compress((tmp / "ground-creations.json").read_bytes(), 9))
        run("TestHaloRustEquipmentStateCorpus")
        (film / "fixtures/equipment-state-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "equipment-state-corpus.json").read_bytes(), 9))
        run("TestHaloRustEquipmentState")
        (film / "fixtures/equipment-state-v41.json.zlib").write_bytes(zlib.compress((tmp / "equipment-state.json").read_bytes(), 9))
        run("TestHaloRustEquipmentCreations")
        (film / "fixtures/equipment-creations-v41.json.zlib").write_bytes(zlib.compress((tmp / "equipment-creations.json").read_bytes(), 9))
        run("TestHaloRustWorldTracks")
        (film / "fixtures/world-tracks-v41.json.zlib").write_bytes(zlib.compress((tmp / "world-tracks.json").read_bytes(), 9))
        run("TestHaloRustPacketDiscovery")
        (film / "fixtures/packet-discovery-v41.json.zlib").write_bytes(zlib.compress((tmp / "packet-discovery.json").read_bytes(), 9))
        run("TestHaloRustKFQComponents")
        (film / "fixtures/kfq-components-v41.json.zlib").write_bytes(zlib.compress((tmp / "kfq-components.json").read_bytes(), 9))
        run("TestHaloRustKFQCorpus")
        (film / "fixtures/kfq-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "kfq-corpus.json").read_bytes(), 9))
        run("TestHaloRustSlotBand")
        (film / "fixtures/slot-band-v41.json.zlib").write_bytes(zlib.compress((tmp / "slot-band.json").read_bytes(), 9))
        run("TestHaloRustTypeContracts")
        (film / "fixtures/type-contracts-v41.json.zlib").write_bytes(zlib.compress((tmp / "type-contracts.json").read_bytes(), 9))
        run("TestHaloRustSourceBits")
        (film / "fixtures/source-bits-v41.json.zlib").write_bytes(zlib.compress((tmp / "source-bits.json").read_bytes(), 9))
        run("TestHaloRustSourceOctets")
        (film / "fixtures/source-octets-v41.json.zlib").write_bytes(zlib.compress((tmp / "source-octets.json").read_bytes(), 9))
        run("TestHaloRustProfileValues")
        (film / "fixtures/profile-values-v41.json.zlib").write_bytes(zlib.compress((tmp / "profile-values.json").read_bytes(), 9))
        run("TestHaloRustProfileTable")
        (reference / "profile-table.json").write_bytes((tmp / "profile-table.json").read_bytes() + b"\n")
        (film / "fixtures/profile-layouts-v41.json.zlib").write_bytes(zlib.compress((tmp / "profile-layouts.json").read_bytes(), 9))
        run("TestHaloRustKFQ")
        (film / "fixtures/kfq-v41.json.zlib").write_bytes(zlib.compress((tmp / "kfq.json").read_bytes(), 9))
        run("TestHaloRustWorldResearch")
        (film / "fixtures/world-research-v41.json.zlib").write_bytes(zlib.compress((tmp / "world-research.json").read_bytes(), 9))
        run("TestHaloRustGrenade")
        (film / "fixtures/grenade-v41.json.zlib").write_bytes(zlib.compress((tmp / "grenade.json").read_bytes(), 9))
        run("TestHaloRustFire")
        (film / "fixtures/fire-scanner-v41.json.zlib").write_bytes(zlib.compress((tmp / "fire-scanner.json").read_bytes(), 9))
        (film / "fixtures/fire-v41.json.zlib").write_bytes(zlib.compress((tmp / "fire.json").read_bytes(), 9))
        run("TestHaloRustRoster")
        (film / "fixtures/roster-v41.json.zlib").write_bytes(zlib.compress((tmp / "roster.json").read_bytes(), 9))
        run("TestHaloRustPlayerTable")
        (film / "fixtures/player-table-v41.json.zlib").write_bytes(zlib.compress((tmp / "player-table.json").read_bytes(), 9))
        text = (reference / "halo_rust_film_player_table_source_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        text = text.replace("/Users/simbleau/git/halo_api/src/theater/fixtures/", str(film / "fixtures") + "/")
        text = text.replace("/private/tmp/halo-film-player-table-source.json", str(tmp / "film-player-table-source.json"))
        (grammar.parent.parent / "replay/halo_rust_film_player_table_source_test.go").write_text(text)
        run("TestHaloRustFilmPlayerTableSource", "replay")
        (film / "fixtures/film-player-table-source-v41.json").write_bytes((tmp / "film-player-table-source.json").read_bytes())
        run("TestHaloRustLoadouts")
        (film / "fixtures/weapon-classification-v41.json.zlib").write_bytes(zlib.compress((tmp / "weapon-classification.json").read_bytes(), 9))
        (reference / "weapon-families-v41.json").write_bytes((tmp / "weapon-families.json").read_bytes() + b"\n")
        (film / "fixtures/keyframe-loadouts-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-loadouts.json").read_bytes(), 9))
        run("TestHaloRustBootstrapParity")
        run("TestHaloRustDatumParity")
        run("TestHaloRustDatumLengths")
        (film / "fixtures/datum-lengths-v41.json").write_bytes((tmp / "datum-lengths.json").read_bytes())
        run("TestHaloRustViewsParity")
        run("TestHaloRustDefaultsParity")
        run("TestHaloRustKeyframesParity")
        run("TestHaloRustPadding")
        (film / "fixtures/native-padding-v41.json.zlib").write_bytes(zlib.compress((tmp / "padding.json").read_bytes(), 9))
        run("TestHaloRustJumpDerivation")
        (film / "fixtures/jump-derivation-v41.json.zlib").write_bytes(zlib.compress((tmp / "jump-derivation.json").read_bytes(), 9))
        run("TestHaloRustMovementComponents")
        (film / "fixtures/movement-components-v41.json.zlib").write_bytes(zlib.compress((tmp / "movement-components.json").read_bytes(), 9))
        run("TestHaloRustFramesParity")
        run("TestHaloRustProductionDefaults")
        (film / "fixtures/native-default-fallbacks-v41.json.zlib").write_bytes(zlib.compress((tmp / "default-fallbacks.json").read_bytes(), 9))
        run("TestHaloRustProductionAdmission")
        run("TestHaloRustStrictLocator")
        (film / "fixtures/strict-locator-v41.json.zlib").write_bytes(zlib.compress((tmp / "strict-locator.json").read_bytes(), 9))
        (film / "fixtures/production-admission-v41.json.zlib").write_bytes(zlib.compress((tmp / "production-admission.json").read_bytes(), 9))
        run("TestHaloRustCapturedKeyframeParity")
        run("TestHaloRustKeyframeAnchorBodiesCorpus")
        (film / "fixtures/keyframe-anchor-bodies-v41.jsonl.zlib").write_bytes(zlib.compress((tmp / "keyframe-anchor-bodies.jsonl").read_bytes(), 9))
        # Keep a small captured witness of a native padded outcome distinct from
        # the bounded read. Select from native output, never from Rust results.
        witness = None
        with (tmp / "keyframe-anchor-bodies.jsonl").open() as records:
            for line in records:
                row = json.loads(line)
                if row["trace"]["EndBit"] > row["size"] * 8:
                    if witness is None or row["size"] < witness["size"]:
                        witness = row
        if witness is None:
            raise RuntimeError("no native padded keyframe witness")
        source = args.corpus.resolve() / witness["file"]
        data = source.read_bytes()
        payload = data[witness["offset"]:witness["offset"] + witness["size"]]
        (film / "fixtures/recovered-keyframe-padding-v41.zlib").write_bytes(zlib.compress(payload, 9))
        (film / "fixtures/recovered-keyframe-padding-v41.json").write_text(json.dumps(witness, indent=2) + "\n")
        metadata = json.loads((source.parent / "film.json").read_text())
        bootstrap = next(c for c in metadata["chunks"] if c["chunk_type"] == 1)
        (film / "fixtures/recovered-keyframe-padding-bootstrap-v41.zlib").write_bytes(zlib.compress((source.parent / bootstrap["file"]).read_bytes(), 9))
        run("TestHaloRustKeyframeTableCorpus")
        (film / "fixtures/keyframe-table-corpus-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframe-table-corpus.json").read_bytes(), 9))
        run("TestHaloRustRecoveryParity")
        run("TestHaloRustWorldParity")
        (film / "fixtures/world-levelup-v41.json.zlib").write_bytes(zlib.compress((tmp / "world.json").read_bytes(), 9))
        run("TestHaloRustProfileParity")
        run("TestHaloRustEventHeadsParity")
        run("TestHaloRustTranslocatorParity")
        text = (reference / "halo_rust_translocator_padding_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_translocator_padding_test.go").write_text(text)
        run("TestHaloRustTranslocatorPadding")
        (film / "fixtures/translocator-padding-v41.json.zlib").write_bytes(zlib.compress((tmp / "translocator-padding.json").read_bytes(), 9))
        text = (reference / "halo_rust_translocator_source_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_translocator_source_test.go").write_text(text)
        run("TestHaloRustTranslocatorSource")
        (film / "fixtures/translocator-source-v41.json.zlib").write_bytes(zlib.compress((tmp / "translocator-source.json").read_bytes(), 9))
        text = (reference / "halo_rust_zoom_padding_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_zoom_padding_test.go").write_text(text)
        run("TestHaloRustZoomPadding")
        (film / "fixtures/zoom-padding-v41.json.zlib").write_bytes(zlib.compress((tmp / "zoom-padding.json").read_bytes(), 9))
        text = (reference / "halo_rust_pickup_padding_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_pickup_padding_test.go").write_text(text)
        run("TestHaloRustPickupPadding")
        (film / "fixtures/pickup-padding-v41.json.zlib").write_bytes(zlib.compress((tmp / "pickup-padding.json").read_bytes(), 9))
        text = (reference / "halo_rust_equipment_spawn_source_test.go.txt").read_text()
        for old, new in paths.items():
            text = text.replace(old, new)
        (grammar / "halo_rust_equipment_spawn_source_test.go").write_text(text)
        run("TestHaloRustEquipmentSpawnSource")
        (film / "fixtures/equipment-spawn-source-v41.json.zlib").write_bytes(zlib.compress((tmp / "equipment-spawn-source.json").read_bytes(), 9))
        run("TestHaloRustRecordMaskHook")
        (film / "fixtures/record-mask-hook-v41.json.zlib").write_bytes(zlib.compress((tmp / "record-mask-hook.json").read_bytes(), 9))
        run("TestHaloRustBipedScanParity")
        (film / "fixtures/biped-scan-levelup-v41.json.zlib").write_bytes(zlib.compress((tmp / "biped-scan.json").read_bytes(), 9))
        run("TestHaloRustGameplayInputs", "replay")
        (film / "fixtures/gameplay-levelup-v41.json.zlib").write_bytes(zlib.compress((tmp / "gameplay.json").read_bytes(), 9))
        (film / "fixtures/translocator-levelup-v41.json.zlib").write_bytes(zlib.compress((tmp / "translocator.json").read_bytes(), 9))
        (film / "fixtures/event-heads-levelup-v41.json.zlib").write_bytes(zlib.compress((tmp / "event-heads.json").read_bytes(), 9))
        (film / "fixtures/profile-levelup-v41.json").write_bytes((tmp / "profile.json").read_bytes())
        (film / "fixtures/recovery-v41-oracle.json.zlib").write_bytes(zlib.compress((tmp / "recovery.json").read_bytes(), 9))
        for source, output in [("captured-keyframe.bin", "captured-keyframe-v41.zlib"), ("captured-keyframe.json", "captured-keyframe-v41-oracle.json.zlib")]:
            (film / "fixtures" / output).write_bytes(zlib.compress((tmp / source).read_bytes(), 9))
        (film / "fixtures/frames-levelup-v41.json.zlib").write_bytes(zlib.compress((tmp / "frames.json").read_bytes(), 9))
        (film / "fixtures/keyframes-levelup-v41.json.zlib").write_bytes(zlib.compress((tmp / "keyframes.json").read_bytes(), 9))
        (film / "fixtures/defaults-levelup-v41.json.zlib").write_bytes(zlib.compress((tmp / "defaults.json").read_bytes(), 9))
        (film / "fixtures/views-levelup-v41.json.zlib").write_bytes(zlib.compress((tmp / "views.json").read_bytes(), 9))
        for source, output in [("datums.bin", "datums-v41.zlib"), ("datums.json", "datums-v41-oracle.json.zlib")]:
            (film / "fixtures" / output).write_bytes(zlib.compress((tmp / source).read_bytes(), 9))
        rows = json.loads((tmp / "bootstrap.json").read_text())
        if len(rows) != 32:
            raise SystemExit(f"Expected 32 corpus films; found {len(rows)}")
        sample = next(row for row in rows if "/natural-end/01-do-nothing/" in row["path"])
        generate_source_publication_oracles(args.go, upstream, film, tmp, paths)
        sources = [film / "components.rs", *sorted((film / "components").glob("*.rs"))]
        names = set(re.findall(r'"([a-z][a-z0-9_-]+)"', "\n".join(p.read_text().split("#[cfg(test)]")[0] for p in sources)))
        contexts = set()
        for archetype in sample["registry"]["Archetypes"]:
            for i, name in enumerate(archetype["Components"] or []):
                if name in names:
                    contexts.add((name, archetype["Index"], archetype["Levels"][i]))
        # Exercise all level gates for the readers first reached by production movement traversal.
        for name, ti in [
            ("managed-player-team-designator-component", 9),
            ("managed-object-networked-splash-message-dynamic-component", 47),
            ("managed-object-property-component", 13),
            ("managed-object-player-masked-property-component", 13),
            ("flock-relevancy-component", 21), ("flock-fleeing-component", 21),
            ("flock-remembered-danger-component", 21), ("flock-destination-component", 21),
            ("game-engine-screen-sequence-component", 0), ("object-position-component", 38),
        ]:
            contexts.update((name, ti, level) for level in range(4))
        for name in [
            'biped-control-context',
            'biped-control-context-component',
            'biped-emp-timer-component',
            'biped-malleable-property',
            'biped-malleable-property-component',
            'biped-spartan-ability-malleable-property-component',
        ]:
            contexts.update((name, 35, level) for level in range(4))
        for name in [
            'equipment-deployed-component',
            'equipment-has-infinite-uses-component',
            'equipment-activated-component',
            'equipment-creator-component',
            'equipment-energy-component',
            'equipment-energy-delay-ticks-left-component',
            'equipment-charges-remaining-component',
            'equipment-tracked-object-handles-stack-component',
            'equipment-command-tick-component',
        ]:
            contexts.update((name, 37, level) for level in range(4))
        contexts.update(("object-multiplayer-properties-component", 38, level) for level in range(4))
        contexts.update(("weapon-ammo-component", 42, level) for level in range(4))
        for name, ti in [("music-variables-component", 17), ("crew-order-component", 14)]:
            contexts.update((name, ti, level) for level in range(4))
        for name, ti in [
            ('flock-destroying-component', 21),
            ('flock-position-component', 21),
            ('game-engine-alliance-component', 0),
            ('game-engine-game-finished-component', 0),
            ('game-engine-team-mapping', 0),
            ('game-engine-team-mapping-component', 0),
            ('managed-player-active-mission-name-component', 9),
            ('managed-player-back-button-scoreboard-flair-component', 9),
            ('managed-player-color-override-component', 9),
            ('managed-player-current-season-component', 9),
            ('managed-player-flags-component', 9),
            ('music-state-component', 17),
            ('nav-cutscene-flag-component', 15),
            ('player-desired-respawn-location-component', 5),
            ('player-desired-respawn-seat-component', 5),
            ('player-early-respawn-requested-component', 5),
            ('player-engine-loadout-index-component', 5),
            ('player-primary-respawn-object-component', 5),
            ('player-representation-component', 5),
            ('player-vehicle-entrance-ban-component', 5),
            ('spawn-filter-type-component', 20),
            ('statborg-entry-index-and-type-component', 6),
            ('statborg-round-outcomes-component', 6),
            ('tacmap-areaofinterest', 32),
            ('track-frame-component', 16),
            ('unit-malleable-property-component', 35),
        ]:
            contexts.update((name, ti, level) for level in range(5))
        contexts.update(("crew-marked-objects-component", 14, level) for level in range(5))
        contexts.update(("effect-state-data-component", 18, level) for level in range(5))
        for name, ti in [
            ('asset-transform-component', 44),
            ('crew-orders-off-flags-component', 14),
            ('flock-current-destination-component', 21),
            ('flock-emitting-component', 21),
            ('flock-forced-respawn-component', 21),
            ('managed-object-networked-splash-message-static-component', 47),
            ('managed-object-property-name-component', 13),
            ('managed-player-campaign-progress-component', 9),
            ('managed-player-custom-input-prompt-widget', 9),
            ('managed-player-forge-weather-effect-overrides-component', 9),
            ('managed-player-show-active-mission-name-in-hud-component', 9),
            ('player-power-frame-points-component', 5),
            ('player-supply-lines-currency-simulation-component', 5),
            ('tacmap-poiicon', 30),
            ('tacmap-poiiconoffset', 30),
            ('tacmap-poiisgoldenpath', 30),
        ]:
            contexts.update((name, ti, level) for level in range(5))
        subprocess.run(["python3", str(reference / "audit_dispatch_names.py"), str(grammar),
                        str(reference / "dispatch-names-v75.json")], check=True)
        covered_names = {entry[0] for entry in contexts}
        extra_contexts = json.loads((reference / "dispatch-extra-contexts-v75.json").read_text())
        for name in json.loads((reference / "dispatch-names-v75.json").read_text()):
            if name not in covered_names:
                # Each added context is reviewed against the native dispatch.
                # A new native name must not silently inherit an arbitrary type.
                assert name in extra_contexts, f"Unaudited native component: {name}"
                ti = extra_contexts[name]
                contexts.update((name, ti, level) for level in range(5))
        entries = [dict(name=name, archetype=ti, level=level) for name, ti, level in sorted(contexts)]
        (tmp / "selected.json").write_text(json.dumps(entries))
        run("TestHaloRustComponentsParity")
        data = (tmp / "components.json").read_bytes()
        (film / "fixtures/components-levelup-v41.json.zlib").write_bytes(zlib.compress(data, 9))
        report = {
            "commit": PIN, "bootstrap_films": len(rows), "component_names": len({e["name"] for e in entries}),
            "component_contexts": len(entries), "component_cases": len(json.loads(data)),
            "component_json_sha256": hashlib.sha256(data).hexdigest(),
        }
        (reference / "parity-cases.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
