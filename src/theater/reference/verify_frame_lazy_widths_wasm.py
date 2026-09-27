#!/usr/bin/env python3
"""Run live frame native-oracle comparisons in an actual wasm32 Node runtime."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import zlib


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=Path, default=Path("/private/tmp/halo-frame-lazy-width-wasm"))
    parser.add_argument("--test-prefix", action="append", help="Run only registered oracle calls with this module/function prefix; default runs the complete suite")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    work = args.work_dir.resolve()
    work.mkdir(parents=True, exist_ok=True)
    theater = root / "src/theater"
    modules = []
    calls = []
    for module, names in [
        ("source_identity_scans_tests", ["native_source_identity_indices_and_clock"]),
        ("context_projectile_tests", ["native_context_projectiles"]),
        ("context_grenade_tests", ["native_context_grenade_scan"]),
        ("context_movement_tests", ["native_context_movement_states", "native_replay_movement_phase"]),
        ("native_context_strict_locator_tests", ["native_context_strict_locator"]),
        ("native_reader_quantization_tests", ["native_reader_quantized_vectors", "native_reader_quantized_cursor_overflow"]),
        ("native_reader_cursor_tests", ["native_reader_signed_cursor_sequences", "native_reader_cursor_overflow_and_recovery"]),
        ("native_context_frame_tests", ["native_context_frame_records_world_and_live_widths",
                                        "native_context_frame_unused_position_widths_are_lazy",
                                        "native_world_position_widths_match_direct_and_frame_reads",
                                        "native_new_record_tail_positive_gate",
                                        "native_new_record_tail_large_and_lazy",
                                        "native_new_default_lazy_widths_and_rewinds",
                                        "native_mpp_widths_are_lazy_and_preserve_wide_reads",
                                        "native_record_id_widths_and_wrapping_match_reference",
                                        "native_record_id_overflow_panics_preserve_cursor",
                                        "native_march_unused_widths_preserve_calibration"]),
        ("native_frame_accumulator_tests", ["native_frame_accumulator_matches_reference",
                                            "native_frame_shared_accumulator_matches_reference"]),
        ("native_context_inference_tests", ["native_context_single_step_inference_scopes_and_restoration", "native_context_single_step_inference_scopes_and_restoration_lazy_widths"]),
        ("native_context_chain_tests", ["native_context_chain_scopes_counters_and_restoration", "native_context_chain_scopes_counters_and_restoration_lazy_widths"]),
        ("native_context_resync_tests", ["native_context_resync_scopes_counters_and_restoration", "native_context_resync_scopes_counters_and_restoration_lazy_widths"]),
        ("native_context_repair_tests", ["native_context_repair_scopes_counters_and_restoration", "native_context_repair_scopes_counters_and_restoration_lazy_widths"]),
        ("native_context_views_tests", ["native_context_production_views_and_cumulative_observers", "native_context_production_views_and_cumulative_observers_lazy_widths"]),
        ("native_context_raw_resync_tests", ["native_context_raw_resync_copy_and_acceptance"]),
        ("native_context_resync_frame_tests", ["native_context_resync_frame_live_recovery", "native_resync_acceptance_removes_live_width", "native_resync_record_guard", "native_resync_final_cursor"]),
    ]:
        source = (theater / (module + ".rs")).read_text().replace("#[test]", "")
        source = source.replace("crate::clients", "halo_api::clients")
        source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
        # Reproduce the private histogram seed through the public direct reader.
        # Hooks are installed later; this observes only absolute region 7.
        source = source.replace("observer.record_absolute(7);", """
            let mut seed_profile=NativeScanProfile::default();
            seed_profile.movement.world_object.index_bits=3;
            seed_profile.movement.world_object.axis_bits=[0;3];
            let mut seed=NativeFilmReader::with_context(&[0x0e,0],NativeReaderContext{
                profile:seed_profile,observer:Some(observer.clone())});
            assert_eq!(seed.read_component("object-position-dynamic-precision-component",0,35).unwrap().0,Some(true));
            assert_eq!(observer.counters().absolute_indices.get(&7),Some(&1));
        """)
        for name in names:
            source = source.replace("fn " + name + "()", "pub fn " + name + "()")
            calls.append(module + "::" + name + "();")
        modules.append("mod " + module + " {\n" + source + "\n}")
    source = (theater / "replay_held_object.rs").read_text().split("#[cfg(test)]", 1)[1]
    source = source.replace("mod tests {", "mod native_held_object { use std::collections::BTreeMap;").replace("#[test]", "")
    source = source.replace("fn native_held_object_and_bomb_carries()", "pub fn native_held_object_and_bomb_carries()")
    source = source.replace("crate::theater", "halo_api::theater")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append(source)
    calls.append("native_held_object::native_held_object_and_bomb_carries();")
    source = (theater / "facts_objective_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_facts_flag_layer()", "pub fn native_facts_flag_layer()")
    source = source.replace("fn native_facts_vip_layer()", "pub fn native_facts_vip_layer()")
    source = source.replace("fn native_facts_skull_layer()", "pub fn native_facts_skull_layer()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_facts_objective_inputs {\n" + source + "\n}")
    calls.append("native_facts_objective_inputs::native_facts_flag_layer();")
    calls.append("native_facts_objective_inputs::native_facts_vip_layer();")
    calls.append("native_facts_objective_inputs::native_facts_skull_layer();")
    source = (theater / "replay_bomb_document_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_bomb_document_statistics()", "pub fn native_bomb_document_statistics()")
    source = source.replace("fn native_bomb_document_armings()", "pub fn native_bomb_document_armings()")
    source = source.replace("fn native_bomb_document_carries()", "pub fn native_bomb_document_carries()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_bomb_document {\n" + source + "\n}")
    calls.append("native_bomb_document::native_bomb_document_statistics();")
    calls.append("native_bomb_document::native_bomb_document_armings();")
    calls.append("native_bomb_document::native_bomb_document_carries();")
    source = (theater / "replay_signed_layer_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_signed_layer_coverage_and_caller_counters()", "pub fn native_signed_layer_coverage_and_caller_counters()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_signed_layer {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_signed_layer::native_signed_layer_coverage_and_caller_counters();")
    source = (theater / "facts_document_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_facts_complete_document_composition()", "pub fn native_facts_complete_document_composition()")
    source = source.replace("fn native_facts_document_caller_precedence()", "pub fn native_facts_document_caller_precedence()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_facts_document {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_facts_document::native_facts_complete_document_composition();")
    calls.append("native_facts_document::native_facts_document_caller_precedence();")
    source = (theater / "facts_live_objectives_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_facts_live_objective_document_pass()", "pub fn native_facts_live_objective_document_pass()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_facts_live_objectives {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_facts_live_objectives::native_facts_live_objective_document_pass();")
    source = (theater / "facts_combat_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_facts_combat_document_stage()", "pub fn native_facts_combat_document_stage()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_facts_combat {\n" + source + "\n}")
    calls.append("native_facts_combat::native_facts_combat_document_stage();")
    source = (theater / "replay_score_document_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_score_objective_document_stage()", "pub fn native_score_objective_document_stage()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_score_document {\n" + source + "\n}")
    calls.append("native_score_document::native_score_objective_document_stage();")
    source = (theater / "facts_equipment_document_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_facts_equipment_document_stage()", "pub fn native_facts_equipment_document_stage()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_equipment_document {\n" + source + "\n}")
    calls.append("native_equipment_document::native_facts_equipment_document_stage();")
    source = (theater / "replay_coverage_document_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_coverage_document_stage()", "pub fn native_coverage_document_stage()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_coverage_document {\n" + source + "\n}")
    calls.append("native_coverage_document::native_coverage_document_stage();")
    source = (theater / "facts_placement_document_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_facts_grapple_placement_document_stage()", "pub fn native_facts_grapple_placement_document_stage()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_placement_document {\n" + source + "\n}")
    calls.append("native_placement_document::native_facts_grapple_placement_document_stage();")
    source = (theater / "facts_pickup_document_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_facts_pickup_pad_document_stage()", "pub fn native_facts_pickup_pad_document_stage()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_pickup_document {\n" + source + "\n}")
    calls.append("native_pickup_document::native_facts_pickup_pad_document_stage();")
    # Geometry catalog parsing uses the actual public in-memory loader on wasm32.
    source = (theater / "facts_ground_vehicle_document_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_facts_ground_vehicle_document_stage()", "pub fn native_facts_ground_vehicle_document_stage()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_ground_vehicle_document {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_ground_vehicle_document::native_facts_ground_vehicle_document_stage();")
    source = (theater / "facts_inventory_document_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_facts_inventory_document_stage()", "pub fn native_facts_inventory_document_stage()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_inventory_document {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_inventory_document::native_facts_inventory_document_stage();")
    source = (theater / "facts_ability_document_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_facts_ability_document_stages()", "pub fn native_facts_ability_document_stages()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_ability_document {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_ability_document::native_facts_ability_document_stages();")
    source = (theater / "replay_finalize_document_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_document_finalization_and_signed_fallbacks()", "pub fn native_document_finalization_and_signed_fallbacks()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_finalize_document {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_finalize_document::native_document_finalization_and_signed_fallbacks();")
    source = (theater / "facts_scan_capture_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_scan_capture_boundary()", "pub fn native_scan_capture_boundary()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_scan_capture {\n" + source + "\n}")
    calls.append("native_scan_capture::native_scan_capture_boundary();")
    source = (theater / "facts_scanned_positions_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_scanned_positions_to_facts()", "pub fn native_scanned_positions_to_facts()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_scanned_positions {\n" + source + "\n}")
    calls.append("native_scanned_positions::native_scanned_positions_to_facts();")
    source = (theater / "source_events_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_loaded_source_events()", "pub fn native_loaded_source_events()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_source_events {\n" + source + "\n}")
    calls.append("native_source_events::native_loaded_source_events();")
    source = (theater / "context_creations_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_biped_creations()", "pub fn native_context_biped_creations()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_creations {\n" + source + "\n}")
    calls.append("native_context_creations::native_context_biped_creations();")
    source = (theater / "replay_initial_scan_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_initial_scan_phase()", "pub fn native_initial_scan_phase()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_initial_scan {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_initial_scan::native_initial_scan_phase();")
    source = (theater / "context_equipment_changes_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_equipment_scan()", "pub fn native_context_equipment_scan()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_equipment_scan {\n" + source + "\n}")
    calls.append("native_context_equipment_scan::native_context_equipment_scan();")
    source = (theater / "world_object_keyframes.rs").read_text().split("#[cfg(test)]", 1)[1]
    source = source.replace("mod tests {", "mod native_source_world_census { use std::collections::BTreeMap; use halo_api::clients::hi::models::FilmChunkData;")
    source = source.replace("#[test]", "").replace("fn native_census_order_and_exclusions()", "pub fn native_census_order_and_exclusions()")
    source = source.replace("crate::clients", "halo_api::clients").replace("super::super::FilmSource", "FilmSource")
    source = source.replace("super::super::bits::Bits(payload)", "CensusBits(payload)")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append(source)
    modules.append("struct CensusBits<'a>(&'a [u8]); impl CensusBits<'_> { fn read(&self, p:usize,n:usize)->Option<u64> { if p+n>self.0.len()*8 { return None; } Some((p..p+n).fold(0, |v,i| (v<<1)|u64::from((self.0[i/8]>>(7-i%8))&1))) } }")
    calls.append("native_source_world_census::native_census_order_and_exclusions();")
    source = (theater / "context_world_tracks_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_world_tracks()", "pub fn native_context_world_tracks()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_world_tracks {\n" + source + "\n}")
    calls.append("native_context_world_tracks::native_context_world_tracks();")
    source = (theater / "native_equipment_default_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_equipment_defaults()", "pub fn native_equipment_defaults()")
    source = source.replace("fn native_ground_weapon_defaults()", "pub fn native_ground_weapon_defaults()")
    source = source.replace("fn native_vehicle_defaults()", "pub fn native_vehicle_defaults()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_equipment_defaults {\n" + source + "\n}")
    calls.append("native_equipment_defaults::native_equipment_defaults();")
    calls.append("native_equipment_defaults::native_ground_weapon_defaults();")
    calls.append("native_equipment_defaults::native_vehicle_defaults();")
    source = (theater / "native_equipment_creation_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_equipment_creation_attempts()", "pub fn native_equipment_creation_attempts()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_equipment_creation {\n" + source + "\n}")
    calls.append("native_equipment_creation::native_equipment_creation_attempts();")
    source = (theater / "context_mpp_calibration_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_mpp_calibration()", "pub fn native_context_mpp_calibration()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_mpp_calibration {\n" + source + "\n}")
    calls.append("native_context_mpp_calibration::native_context_mpp_calibration();")
    source = (theater / "context_equipment_creations_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_equipment_creations()", "pub fn native_context_equipment_creations()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_equipment_creations {\n" + source + "\n}")
    calls.append("native_context_equipment_creations::native_context_equipment_creations();")
    source = (theater / "context_equipment_placements_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_equipment_placements()", "pub fn native_context_equipment_placements()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_equipment_placements {\n" + source + "\n}")
    calls.append("native_context_equipment_placements::native_context_equipment_placements();")
    source = (theater / "native_ground_ammo_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_ground_ammo_traversal()", "pub fn native_ground_ammo_traversal()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_ground_ammo {\n" + source + "\n}")
    calls.append("native_ground_ammo::native_ground_ammo_traversal();")
    source = (theater / "context_ground_creations_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_ground_creations()", "pub fn native_context_ground_creations()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_ground_creations {\n" + source + "\n}")
    calls.append("native_context_ground_creations::native_context_ground_creations();")
    source = (theater / "context_vehicle_creations_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_vehicle_creations()", "pub fn native_context_vehicle_creations()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_vehicle_creations {\n" + source + "\n}")
    calls.append("native_context_vehicle_creations::native_context_vehicle_creations();")
    source = (theater / "context_vehicle_channels_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_vehicle_channels()", "pub fn native_context_vehicle_channels()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_vehicle_channels {\n" + source + "\n}")
    calls.append("native_context_vehicle_channels::native_context_vehicle_channels();")
    source = (theater / "source_anticipated_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_source_anticipated_domain()", "pub fn native_source_anticipated_domain()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_source_anticipated_domain {\n" + source + "\n}")
    calls.append("native_source_anticipated_domain::native_source_anticipated_domain();")
    source = (theater / "replay_world_scan_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_world_scan_phase()", "pub fn native_world_scan_phase()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_world_scan_phase {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_world_scan_phase::native_world_scan_phase();")
    source = (theater / "replay_guarded_scan_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_guarded_scan_phase()", "pub fn native_guarded_scan_phase()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_guarded_scan_phase {\n" + source + "\n}")
    calls.append("native_guarded_scan_phase::native_guarded_scan_phase();")
    source = (theater / "context_navpoint_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_navpoint_radial()", "pub fn native_context_navpoint_radial()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_navpoint_radial {\n" + source + "\n}")
    calls.append("native_context_navpoint_radial::native_context_navpoint_radial();")
    source = (theater / "context_managed_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_managed_properties()", "pub fn native_context_managed_properties()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_managed_properties {\n" + source + "\n}")
    calls.append("native_context_managed_properties::native_context_managed_properties();")
    source = (theater / "source_carrier_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_source_carrier_marks()", "pub fn native_source_carrier_marks()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_source_carrier_marks {\n" + source + "\n}")
    calls.append("native_source_carrier_marks::native_source_carrier_marks();")
    source = (theater / "source_world_events_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_source_world_events()", "pub fn native_source_world_events()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_source_world_events {\n" + source + "\n}")
    calls.append("native_source_world_events::native_source_world_events();")
    source = (theater / "context_equipment_recovery_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_equipment_probe()", "pub fn native_context_equipment_probe()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_equipment_probe {\n" + source + "\n}")
    calls.append("native_context_equipment_probe::native_context_equipment_probe();")
    source = (theater / "context_ability_charges_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_ability_charges()", "pub fn native_context_ability_charges()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_ability_charges {\n" + source + "\n}")
    calls.append("native_context_ability_charges::native_context_ability_charges();")
    source = (theater / "context_ability_use_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_ability_use()", "pub fn native_context_ability_use()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_ability_use {\n" + source + "\n}")
    calls.append("native_context_ability_use::native_context_ability_use();")
    source = (theater / "context_ability_channels_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_ability_channels()", "pub fn native_context_ability_channels()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_ability_channels {\n" + source + "\n}")
    calls.append("native_context_ability_channels::native_context_ability_channels();")
    source = (theater / "replay_ability_scan_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_ability_scan_phase()", "pub fn native_ability_scan_phase()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_ability_scan {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_ability_scan::native_ability_scan_phase();")
    source = (theater / "replay_carried_scan_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_carried_scan_phase()", "pub fn native_carried_scan_phase()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_carried_scan {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_carried_scan::native_carried_scan_phase();")
    source = (theater / "replay_pad_scan_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_pad_scan_phase()", "pub fn native_pad_scan_phase()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_pad_scan {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_pad_scan::native_pad_scan_phase();")
    source = (theater / "replay_vehicle_scan_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_vehicle_scan_phase()", "pub fn native_vehicle_scan_phase()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_vehicle_scan {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_vehicle_scan::native_vehicle_scan_phase();")
    source = (theater / "context_inventory_delta_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_inventory_deltas()", "pub fn native_context_inventory_deltas()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_inventory {\n" + source + "\n}")
    calls.append("native_context_inventory::native_context_inventory_deltas();")
    source = (theater / "context_weapon_changes_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_held_weapons()", "pub fn native_context_held_weapons()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_weapons {\n" + source + "\n}")
    calls.append("native_context_weapons::native_context_held_weapons();")
    source = (theater / "context_delta_walk_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_context_delta_walk()", "pub fn native_context_delta_walk()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_context_delta {\n" + source + "\n}")
    calls.append("native_context_delta::native_context_delta_walk();")
    source = (theater / "source_inventory_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_loaded_source_inventory()", "pub fn native_loaded_source_inventory()")
    source = source.replace("fn native_context_pickups()", "pub fn native_context_pickups()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_source_inventory {\n" + source + "\n}")
    calls.append("native_source_inventory::native_loaded_source_inventory();")
    calls.append("native_source_inventory::native_context_pickups();")
    source = (theater / "facts_replay_provenance_tests.rs").read_text().replace("#[test]", "")
    source = source.replace("fn native_facts_entry_provenance_and_raw_fallbacks()", "pub fn native_facts_entry_provenance_and_raw_fallbacks()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("super::log_test_support::", "log_test_support::")
    support = (theater / "log_test_support.rs").read_text().replace("//!", "//")
    modules.append("mod native_facts_provenance {\n" + source + "\nmod log_test_support {\n" + support + "\n}\n}")
    calls.append("native_facts_provenance::native_facts_entry_provenance_and_raw_fallbacks();")
    # Filesystem behavior is validated on the host; no WASM host filesystem calls.
    source = (theater / "replay_geometry_loader_tests.rs").read_text()
    helpers_geometry = source.split("#[test]", 1)[0]
    test_geometry = source.split("fn native_geometry_loader_in_memory()", 1)[1].split("#[test]", 1)[0]
    source = helpers_geometry + "pub fn native_geometry_loader_in_memory()" + test_geometry
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_geometry_loader {\n" + source + "\n}")
    calls.append("native_geometry_loader::native_geometry_loader_in_memory();")
    source = (theater / "replay_geometry_distance.rs").read_text().split("#[cfg(test)]", 1)[1]
    source = source.replace("mod tests {", "mod native_geometry_distance {").replace("#[test]", "").replace("fn native_geometry_plan_distance()", "pub fn native_geometry_plan_distance()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_geometry_grenade_attribution()", "pub fn native_geometry_grenade_attribution()").replace("crate::theater", "halo_api::theater")
    modules.append(source)
    calls.append("native_geometry_distance::native_geometry_plan_distance();")
    calls.append("native_geometry_distance::native_geometry_grenade_attribution();")
    source = (theater / "callouts_catalog_tests.rs").read_text()
    source = source.split("#[test]\nfn native_callouts_catalog_filesystem", 1)[0]
    source = source.replace("#[test]", "").replace("fn native_callouts_catalog_fields_and_lookups()", "pub fn native_callouts_catalog_fields_and_lookups()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_callouts_catalog {\n" + source + "\n}")
    calls.append("native_callouts_catalog::native_callouts_catalog_fields_and_lookups();")
    source = (theater / "map_background_calibration.rs").read_text().split("#[cfg(test)]",1)[1]
    source = source.replace("mod tests {", "mod native_background_calibration {").replace("#[test]", "").replace("fn native_map_background_world_to_pixel()", "pub fn native_map_background_world_to_pixel()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append(source)
    calls.append("native_background_calibration::native_map_background_world_to_pixel();")
    source = (theater / "map_background_tests.rs").read_text().split("#[test]\nfn native_map_background_filesystem",1)[0]
    source = source.replace("#[test]", "").replace("fn native_map_background_fields_and_timestamps()", "pub fn native_map_background_fields_and_timestamps()")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_background_loader {\n" + source + "\n}")
    calls.append("native_background_loader::native_map_background_fields_and_timestamps();")
    source = (theater / "map_background_index_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    for name in ["native_background_identity_normalization", "native_background_index_lookup", "native_background_index_raw_keys"]:
        source = source.replace("fn " + name + "()", "pub fn " + name + "()")
        calls.append("native_background_index::" + name + "();")
    modules.append("mod native_background_index {\n" + source + "\n}")
    source = (theater / "facts_transport_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_transport_sequences()", "pub fn native_facts_transport_sequences()")
    modules.append("mod native_facts_transport {\n" + source + "\n}")
    calls.append("native_facts_transport::native_facts_transport_sequences();")
    source = (theater / "facts_ammo_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_ammo_codec()", "pub fn native_facts_ammo_codec()")
    modules.append("mod native_facts_ammo {\n" + source + "\n}")
    calls.append("native_facts_ammo::native_facts_ammo_codec();")
    source = (theater / "facts_tracks_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_track_codec()", "pub fn native_facts_track_codec()")
    modules.append("mod native_facts_tracks {\n" + source + "\n}")
    calls.append("native_facts_tracks::native_facts_track_codec();")
    source = (theater / "facts_keyframes_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    for name in ["native_facts_keyframe_codec", "native_facts_keyframe_trailing_byte_guard"]:
        source = source.replace("fn " + name + "()", "pub fn " + name + "()")
        calls.append("native_facts_keyframes::" + name + "();")
    modules.append("mod native_facts_keyframes {\n" + source + "\n}")
    source = (theater / "facts_creations_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_world_creation_codec()", "pub fn native_facts_world_creation_codec()")
    modules.append("mod native_facts_world {\n" + source + "\n}")
    calls.append("native_facts_world::native_facts_world_creation_codec();")
    source = (theater / "facts_vehicle_lives_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_vehicle_census_deaths_and_bounds()", "pub fn native_facts_vehicle_census_deaths_and_bounds()")
    source = source.replace("fn native_facts_vehicle_heading_grouping_and_samples()", "pub fn native_facts_vehicle_heading_grouping_and_samples()")
    calls.append("native_facts_vehicle_lives::native_facts_vehicle_heading_grouping_and_samples();")
    source = source.replace("fn native_facts_vehicle_spawn_drawable_and_track()", "pub fn native_facts_vehicle_spawn_drawable_and_track()")
    calls.append("native_facts_vehicle_lives::native_facts_vehicle_spawn_drawable_and_track();")
    source = source.replace("fn native_facts_vehicle_rider_aim()", "pub fn native_facts_vehicle_rider_aim()")
    calls.append("native_facts_vehicle_lives::native_facts_vehicle_rider_aim();")
    source = source.replace("fn native_facts_vehicle_written_rides()", "pub fn native_facts_vehicle_written_rides()")
    calls.append("native_facts_vehicle_lives::native_facts_vehicle_written_rides();")
    source = source.replace("fn native_facts_vehicle_combined_rides()", "pub fn native_facts_vehicle_combined_rides()")
    calls.append("native_facts_vehicle_lives::native_facts_vehicle_combined_rides();")
    source = source.replace("fn native_facts_vehicle_publication()", "pub fn native_facts_vehicle_publication()")
    calls.append("native_facts_vehicle_lives::native_facts_vehicle_publication();")
    source = source.replace("fn native_facts_vehicle_heading_sources()", "pub fn native_facts_vehicle_heading_sources()")
    calls.append("native_facts_vehicle_lives::native_facts_vehicle_heading_sources();")
    modules.append("mod native_facts_vehicle_lives {\n" + source + "\n}")
    calls.append("native_facts_vehicle_lives::native_facts_vehicle_census_deaths_and_bounds();")
    source = (theater / "facts_ground_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_ground_object_assembly()", "pub fn native_facts_ground_object_assembly()")
    source = source.replace("fn native_facts_ground_lifetime_resolution()", "pub fn native_facts_ground_lifetime_resolution()")
    source = source.replace("fn native_ground_pad_signed_clock_domain()", "pub fn native_ground_pad_signed_clock_domain()")
    calls.append("native_facts_ground::native_facts_ground_lifetime_resolution();")
    source = source.replace("fn native_facts_free_objectives_and_publication()", "pub fn native_facts_free_objectives_and_publication()")
    calls.append("native_facts_ground::native_facts_free_objectives_and_publication();")
    source = source.replace("fn native_facts_flag_markers_and_carries()", "pub fn native_facts_flag_markers_and_carries()")
    calls.append("native_facts_ground::native_facts_flag_markers_and_carries();")
    calls.append("native_facts_ground::native_ground_pad_signed_clock_domain();")
    source = source.replace("fn native_facts_pad_scans_and_signed_coverage()", "pub fn native_facts_pad_scans_and_signed_coverage()")
    calls.append("native_facts_ground::native_facts_pad_scans_and_signed_coverage();")
    source = source.replace("fn native_facts_ground_items_and_kind_gates()", "pub fn native_facts_ground_items_and_kind_gates()")
    calls.append("native_facts_ground::native_facts_ground_items_and_kind_gates();")
    modules.append("mod native_facts_ground {\n" + source + "\n}")
    calls.append("native_facts_ground::native_facts_ground_object_assembly();")
    source = (theater / "facts_placement_evidence_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_placement_origins_and_census_bounds()", "pub fn native_facts_placement_origins_and_census_bounds()")
    source = source.replace("fn native_facts_placement_publication_signed_stats()", "pub fn native_facts_placement_publication_signed_stats()")
    modules.append("mod native_facts_placement_evidence {\n" + source + "\n}")
    calls.append("native_facts_placement_evidence::native_facts_placement_origins_and_census_bounds();")
    calls.append("native_facts_placement_evidence::native_facts_placement_publication_signed_stats();")
    source = (theater / "facts_stance_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_stance_raw_identity_and_counters()", "pub fn native_facts_stance_raw_identity_and_counters()")
    modules.append("mod native_facts_stances {\n" + source + "\n}")
    calls.append("native_facts_stances::native_facts_stance_raw_identity_and_counters();")
    source = (theater / "facts_ability_action_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_ability_action_domains_and_lifetimes()", "pub fn native_facts_ability_action_domains_and_lifetimes()")
    modules.append("mod native_facts_ability_actions {\n" + source + "\n}")
    calls.append("native_facts_ability_actions::native_facts_ability_action_domains_and_lifetimes();")
    source = (theater / "facts_episode_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_equipment_episode_presence_and_lifetimes()", "pub fn native_facts_equipment_episode_presence_and_lifetimes()")
    modules.append("mod native_facts_episode_inputs {\n" + source + "\n}")
    calls.append("native_facts_episode_inputs::native_facts_equipment_episode_presence_and_lifetimes();")
    source = (theater / "facts_grapple_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_grapple_pairing_and_arrival()", "pub fn native_facts_grapple_pairing_and_arrival()")
    modules.append("mod native_facts_grapple_inputs {\n" + source + "\n}")
    calls.append("native_facts_grapple_inputs::native_facts_grapple_pairing_and_arrival();")
    source = (theater / "facts_translocation_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    log_start = source.index("        assert_eq!(\n            super::log_test_support")
    log_end = source.index("        let raw:", log_start)
    source = source[:log_start] + "        let _ = c.log;\n" + source[log_end:]
    source = source.replace("fn native_facts_translocation_publication()", "pub fn native_facts_translocation_publication()")
    modules.append("mod native_facts_translocation_inputs {\n" + source + "\n}")
    calls.append("native_facts_translocation_inputs::native_facts_translocation_publication();")
    for module, name in [("facts_weapon_inputs_tests", "native_facts_weapon_change_publication"), ("facts_pickup_inputs_tests", "native_facts_pickup_construction")]:
        source = (theater / (module + ".rs")).read_text().replace("#[test]", "")
        source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
        source = source.replace("fn " + name + "()", "pub fn " + name + "()")
        modules.append("mod " + module + " {\n" + source + "\n}")
        calls.append(module + "::" + name + "();")
    source = (theater / "facts_equipment_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    log_start = source.index("        assert_eq!(\n            crate::theater::log_test_support")
    log_end = source.index("        invalid +=", log_start)
    source = source[:log_start] + "        let _ = row.log;\n" + source[log_end:]
    source = source.replace("fn native_facts_equipment_domains_and_publication()", "pub fn native_facts_equipment_domains_and_publication()")
    modules.append("mod native_facts_equipment_inputs {\n" + source + "\n}")
    calls.append("native_facts_equipment_inputs::native_facts_equipment_domains_and_publication();")
    source = (theater / "facts_player_inventory_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_player_inventory_document_projection()", "pub fn native_facts_player_inventory_document_projection()")
    modules.append("mod native_facts_player_inventory {\n" + source + "\n}")
    calls.append("native_facts_player_inventory::native_facts_player_inventory_document_projection();")
    source = (theater / "facts_ability_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_ability_signed_ranks_and_publication()", "pub fn native_facts_ability_signed_ranks_and_publication()")
    modules.append("mod native_facts_ability_inputs {\n" + source + "\n}")
    calls.append("native_facts_ability_inputs::native_facts_ability_signed_ranks_and_publication();")
    source = (theater / "facts_inventory_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    # The host checks tracing output; WASM checks projection, coverage and deaths.
    log_start = source.index("        assert_eq!(\n            crate::theater::log_test_support")
    log_end = source.index('        assert_eq!(reads, c.reads', log_start)
    source = source[:log_start] + "        let _ = c.logs;\n" + source[log_end:]
    source = source.replace("fn native_facts_inventory_publication_and_death_attribution()", "pub fn native_facts_inventory_publication_and_death_attribution()")
    modules.append("mod native_facts_inventory_inputs {\n" + source + "\n}")
    calls.append("native_facts_inventory_inputs::native_facts_inventory_publication_and_death_attribution();")
    source = (theater / "facts_projectile_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    for module, name in [("projectiles", "native_facts_projectile_grid_cuts_and_links"), ("grenades", "native_facts_grenade_sources_and_projectile_links")]:
        source = source.replace("mod " + module + " {", "pub mod " + module + " {")
        source = source.replace("fn " + name + "()", "pub fn " + name + "()")
        calls.append("native_facts_projectile_inputs::" + module + "::" + name + "();")
    modules.append("mod native_facts_projectile_inputs {\n" + source + "\n}")
    source = (theater / "facts_carried_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    for name in ["native_facts_loadout_publication", "native_facts_grenade_selection_domain"]:
        source = source.replace("fn " + name + "()", "pub fn " + name + "()")
        calls.append("native_facts_carried_inputs::" + name + "();")
    modules.append("mod native_facts_carried_inputs {\n" + source + "\n}")
    source = (theater / "facts_shot_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_shot_signed_identity()", "pub fn native_facts_shot_signed_identity()")
    source = source.replace("fn native_facts_vehicle_shot_signed_identity()", "pub fn native_facts_vehicle_shot_signed_identity()")
    modules.append("mod native_facts_shot_inputs {\n" + source + "\n}")
    calls.append("native_facts_shot_inputs::native_facts_shot_signed_identity();")
    calls.append("native_facts_shot_inputs::native_facts_vehicle_shot_signed_identity();")
    source = (theater / "facts_player_inputs_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    for name in ["native_facts_player_document_projection", "native_facts_scope_signed_levels", "facts_player_projection_rejects_unrepresentable_legacy_values"]:
        source = source.replace("fn " + name + "()", "pub fn " + name + "()")
        calls.append("native_facts_player_inputs::" + name + "();")
    modules.append("mod native_facts_player_inputs {\n" + source + "\n}")
    source = (theater / "facts_identity_json_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_identity_fallback_json()", "pub fn native_facts_identity_fallback_json()")
    modules.append("mod native_facts_identity_json {\n" + source + "\n}")
    calls.append("native_facts_identity_json::native_facts_identity_fallback_json();")
    source = (theater / "facts_statborg_json_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_statborg_json()", "pub fn native_facts_statborg_json()")
    modules.append("mod native_facts_statborg_json {\n" + source + "\n}")
    calls.append("native_facts_statborg_json::native_facts_statborg_json();")
    source = (theater / "facts_kills_json_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_kills_json()", "pub fn native_facts_kills_json()")
    modules.append("mod native_facts_kills_json {\n" + source + "\n}")
    calls.append("native_facts_kills_json::native_facts_kills_json();")
    source = (theater / "facts_file_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_complete_file()", "pub fn native_facts_complete_file()")
    modules.append("mod native_facts_complete_file {\n" + source + "\n}")
    calls.append("native_facts_complete_file::native_facts_complete_file();")
    source = (theater / "facts_file_header_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_file_header_codec()", "pub fn native_facts_file_header_codec()")
    modules.append("mod native_facts_file_header {\n" + source + "\n}")
    calls.append("native_facts_file_header::native_facts_file_header_codec();")
    source = (theater / "facts_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_complete_blob()", "pub fn native_facts_complete_blob()")
    modules.append("mod native_facts_assembly {\n" + source + "\n}")
    calls.append("native_facts_assembly::native_facts_complete_blob();")
    source = (theater / "facts_tail_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("facts_creations_tests", "native_facts_world").replace("facts_placement_stats_tests", "native_facts_placement_stats")
    source = source.replace("fn native_facts_tail_codecs()", "pub fn native_facts_tail_codecs()")
    modules.append("mod native_facts_tail {\n" + source + "\n}")
    calls.append("native_facts_tail::native_facts_tail_codecs();")
    source = (theater / "facts_state_channels_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_state_channel_codecs()", "pub fn native_facts_state_channel_codecs()")
    modules.append("mod native_facts_state_channels {\n" + source + "\n}")
    calls.append("native_facts_state_channels::native_facts_state_channel_codecs();")
    source = (theater / "facts_body_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_event_inventory_codecs()", "pub fn native_facts_event_inventory_codecs()")
    modules.append("mod native_facts_body {\n" + source + "\n}")
    calls.append("native_facts_body::native_facts_event_inventory_codecs();")
    source = (theater / "facts_header_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_header_codec()", "pub fn native_facts_header_codec()")
    modules.append("mod native_facts_header {\n" + source + "\n}")
    calls.append("native_facts_header::native_facts_header_codec();")
    source = (theater / "facts_channels_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_channel_codecs()", "pub fn native_facts_channel_codecs()")
    modules.append("mod native_facts_channels {\n" + source + "\n}")
    calls.append("native_facts_channels::native_facts_channel_codecs();")
    source = (theater / "facts_object_deaths_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_object_deaths_writer()", "pub fn native_facts_object_deaths_writer()")
    source = source.replace("fn native_facts_json_float32_wire()", "pub fn native_facts_json_float32_wire()")
    source = source.replace("fn native_facts_object_death_decoder()", "pub fn native_facts_object_death_decoder()")
    source = source.replace("fn native_facts_json_syntax()", "pub fn native_facts_json_syntax()")
    source = source.replace("fn native_facts_vehicle_scan()", "pub fn native_facts_vehicle_scan()")
    modules.append("mod native_facts_object_deaths {\n" + source + "\n}")
    calls.append("native_facts_object_deaths::native_facts_object_deaths_writer();")
    calls.append("native_facts_object_deaths::native_facts_json_float32_wire();")
    calls.append("native_facts_object_deaths::native_facts_object_death_decoder();")
    calls.append("native_facts_object_deaths::native_facts_json_syntax();")
    calls.append("native_facts_object_deaths::native_facts_vehicle_scan();")
    source = (theater / "facts_placement_stats_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_placement_statistics()", "pub fn native_facts_placement_statistics()")
    modules.append("mod native_facts_placement_stats {\n" + source + "\n}")
    calls.append("native_facts_placement_stats::native_facts_placement_statistics();")
    source = (theater / "facts_guards_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    source = source.replace("fn native_facts_mode_guards()", "pub fn native_facts_mode_guards()")
    modules.append("mod native_facts_guards {\n" + source + "\n}")
    calls.append("native_facts_guards::native_facts_mode_guards();")
    source = (theater / "facts_positions_tests.rs").read_text().replace("#[test]", "")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    for name in ["native_facts_position_codec", "native_facts_direction_codec"]:
        source = source.replace("fn " + name + "()", "pub fn " + name + "()")
        calls.append("native_facts_positions::" + name + "();")
    modules.append("mod native_facts_positions {\n" + source + "\n}")
    # Exercise the public full-decoder input guards on wasm32 without copying
    # the private loaded-source adapter into the external harness.
    source = (theater / "kill_source_film.rs").read_text()
    source = source.split("    fn native_kill_decode_input_refusals()", 1)[1].split("    #[test]", 1)[0]
    source = "pub fn native_kill_decode_input_refusals()" + source
    source = source.replace("crate::theater", "halo_api::theater").replace("crate::clients", "halo_api::clients")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_kill_input_controls { use halo_api::theater::*; use std::io::Read;\n" + source + "\n}")
    calls.append("native_kill_input_controls::native_kill_decode_input_refusals();")
    source = (theater / "kill_timeline.rs").read_text()
    source = source.split("    fn native_kill_registry_admission()", 1)[1].split("    #[test]", 1)[0]
    source = "pub fn native_kill_registry_admission()" + source
    source = source.replace("crate::theater", "halo_api::theater").replace("crate::clients", "halo_api::clients")
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_kill_registry_controls { use halo_api::theater::*; use std::io::Read;\n" + source + "\n}")
    calls.append("native_kill_registry_controls::native_kill_registry_admission();")
    kill_source = (theater / "kill_decode.rs").read_text()
    for name in ("native_kill_option_normalization", "kill_option_execution_domain"):
        source = kill_source.split("    fn " + name + "()", 1)[1]
        source = source.split("    #[test]", 1)[0] if name == "native_kill_option_normalization" else source.split("    fn normalize(v:", 1)[0]
        source = "pub fn " + name + "()" + source
        source = source.replace("crate::clients", "halo_api::clients")
        source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
        modules.append("mod " + name + " { use halo_api::theater::*; use std::io::Read;\n" + source + "\n}")
        calls.append(name + "::" + name + "();")
    source = (theater / "kill_health.rs").read_text()
    source = source.split("    fn native_kill_health_methods()", 1)[1].rsplit("\n}", 1)[0]
    source = "pub fn native_kill_health_methods()" + source
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_kill_health_controls { use halo_api::theater::*; use std::io::Read;\n" + source + "\n}")
    calls.append("native_kill_health_controls::native_kill_health_methods();")
    source = (theater / "kill_hybrid.rs").read_text()
    source = source.split("    fn native_kill_provenance_display()", 1)[1].rsplit("\n}", 1)[0]
    source = "pub fn native_kill_provenance_display()" + source
    source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
    modules.append("mod native_kill_provenance_controls { use halo_api::theater::*;\n" + source + "\n}")
    calls.append("native_kill_provenance_controls::native_kill_provenance_display();")
    signed_source = (theater / "native_signed_width_tests.rs").read_text()
    for name in ("native_signed_component_reader_cursor", "native_signed_component_public_frame", "native_frame_panic_cursor_and_world", "native_source_frame_panic_cursor_and_bindings", "native_signed_record_headers", "native_signed_header_frame_entries", "native_mpp_wide_frames", "native_position_width_domain", "native_signed_inference_entries", "native_signed_inference_trials", "native_signed_resync_starts", "native_signed_resync_frames", "native_signed_harvest_successors", "native_context_harvest", "native_signed_views", "native_signed_view_readers", "native_signed_view_continuation", "native_signed_march_entries", "native_queue_unused_widths", "native_signed_wrapper_continuation", "native_march_rollback", "native_signed_keyframe_chain", "native_signed_keyframe_entries", "native_context_keyframes", "native_keyframe_layout_domain", "native_keyframe_frame_bits", "native_keyframe_words", "native_signed_validated_resync", "native_isolation_gap_options", "native_overlapping_payloads", "native_component_mask_wrap", "native_component_result_contract", "native_dead_result_retention", "native_parent_result_selection"):
        source = signed_source.split("fn " + name + "()", 1)[1].split("\n#[test]", 1)[0]
        source = "pub fn " + name + "()" + source
        source = source.replace('"fixtures/', '"' + str(theater / "fixtures") + '/')
        if name == "native_component_result_contract":
            profile_source = (theater / "native_scan_profile.rs").read_text()
            descriptor = "fn descriptor" + profile_source.split("fn descriptor", 1)[1].split("    fn layout", 1)[0]
            profile = "fn profile" + profile_source.split("pub(crate) fn profile", 1)[1].split("    #[test]", 1)[0]
            source = descriptor + profile + source.replace("native_scan_profile::tests::profile", "profile")
        modules.append("mod " + name + " { use halo_api::theater::*; use serde_json::Value; use std::io::Read;\n" + source + "\n}")
        calls.append(name + "::" + name + "();")
    rows = json.loads(zlib.decompress((theater / "fixtures/world-position-widths-v41.json.zlib").read_bytes()))
    row = next(r for r in rows if r["width"] == 6 and r["mode"] == 0 and r["start"] == 0)
    # Wide position fields are consumed on WASM without an address-width refusal.
    calls.append("""
 let data: &[u8] = &BYTES;
 let registry = FilmRegistry {
  archetypes: (0..4).map(|index| FilmArchetype {index,
   components: if index==3 {vec!["object-position-component".into()]}else{vec![]},
   levels: if index==3 {vec![0]}else{vec![]}}).collect(),
  major_version:41,format_version:27,end_byte:0,truncated:false,
 };
 let mut cfg=NativeFrameConfig{id_low_bits:5,..Default::default()};
 cfg.context.profile.movement.world_object.index_bits=6;
 cfg.context.profile.movement.world_object.axis_bits=[6,1u64<<32,8];
 let mut reader=NativeFilmReader::new(data);
 let mut world=FilmWorld::default();world.bind_full(1,3);
 let view=reader.read_frame_records(&registry,&mut world,&cfg).unwrap();
 assert!(view.records.iter().all(|r| r.diagnostics.width_refusals.is_empty()));
 assert!(reader.native_bit_position() > (1i64<<32));

 """.replace("BYTES", str(list(bytes.fromhex(row["frame_hex"])))))
    large_rows = json.loads(zlib.decompress((theater / "fixtures/new-tail-large-v41.json.zlib").read_bytes()))
    large = next(r for r in large_rows if r["tail"] == 4097 and r["mode"] == 0 and r["start"] == 0)
    calls.append("""
 cfg.packet_preamble_bits=0;
 let new_data:&[u8]=&NEW_BYTES;
 cfg.context.profile.grammar.default_state_by_archetype=false;
 cfg.context.profile.grammar.new_record_tail_bits=(1i64<<32)-21;
 let mut overflow_reader=NativeFilmReader::new(new_data);
 let overflow=overflow_reader.read_frame_records(&registry,&mut world,&cfg).unwrap();
 let adjustment=&overflow.records[0].diagnostics.width_adjustments[0];
 assert_eq!(adjustment.purpose,Some(NativeWidthPurpose::NewRecordTail));
 assert_eq!(adjustment.bit,21);
 assert_eq!(adjustment.end_bit,None);
 assert_eq!(adjustment.native_end_bit(),Some(1i64<<32));
 // The following native End header consumes three padded bits on wasm32.
 assert_eq!(overflow_reader.native_bit_position(),(1i64<<32)+3);
 assert_eq!(overflow.stop,EntityViewStop::Complete);
 """.replace("NEW_BYTES", str(list(bytes.fromhex(large["hex"])))))
    # Metadata round trip only: this does not claim an executed multi-gigabit scan.
    calls.append("""
 let bit=1i64<<32;
 let native=NativeResyncFrame{records:vec![],diagnostics:Default::default(),resync_bits:vec![bit],end_bit:bit,stop:InferenceFrameStop::PayloadBoundary};
 let raw=ResyncFrame{observations:Default::default(),records:vec![],resync_bits:vec![bit],end_bit:bit,stop:InferenceFrameStop::PayloadBoundary};
 let native_json=serde_json::to_value(&native).unwrap();
 let raw_json=serde_json::to_value(&raw).unwrap();
 assert_eq!(native_json["resync_bits"][0].as_i64(),Some(bit));
 assert_eq!(raw_json["resync_bits"][0].as_i64(),Some(bit));
 assert_eq!(serde_json::from_value::<NativeResyncFrame>(native_json).unwrap(),native);
 assert_eq!(serde_json::from_value::<ResyncFrame>(raw_json).unwrap(),raw);
 """)
    helpers = """
fn native_address(bit: impl TryInto<usize>) -> usize { bit.try_into().ok().expect("fixture source address") }
fn padded_from_native(bit: i64, source: usize) -> usize {
    if bit < 0 { return 0; }
    usize::try_from((bit as u64).saturating_sub(source as u64)).unwrap_or(usize::MAX)
}
"""
    if args.test_prefix:
        calls = [call for call in calls if call.startswith(tuple(args.test_prefix))]
        if not calls:
            parser.error("no oracle calls match --test-prefix=" + ",".join(args.test_prefix))
        print(f"Selected {len(calls)} oracle calls matching {', '.join(args.test_prefix)}", flush=True)
    generated = ("use halo_api::theater::*;\n" + helpers + "\n".join(modules)
        + '\n#[unsafe(no_mangle)]\npub extern "C" fn run() -> i32 {\n'
        + "\n".join(calls) + "\n14554\n}\n")
    generated = generated.replace("crate::theater::bits::native_address", "native_address")
    generated = generated.replace("crate::theater::bits::padded_from_native", "padded_from_native")
    (work / "lib.rs").write_text(generated)
    (work / "Cargo.toml").write_text(
        '[package]\nname="halo_frame_lazy_width_wasm"\nversion="0.0.0"\nedition="2024"\n'
        '[lib]\npath="lib.rs"\ncrate-type=["cdylib"]\n[dependencies]\nhalo_api={path='
        + json.dumps(str(root)) + '}\nserde_json="1"\nserde="1"\nflate2="1"\ntracing="0.1"\n')
    shutil.copyfile(root / "Cargo.lock", work / "Cargo.lock")
    (work / "check.cjs").write_text("""const fs=require('fs');
const mod=new WebAssembly.Module(fs.readFileSync(process.argv[2]));
const imports={};
for(const i of WebAssembly.Module.imports(mod)) {
 if(i.kind!=="function") throw new Error("unsupported test import "+JSON.stringify(i));
 (imports[i.module]??={})[i.name]=()=>{throw new Error("unexpected host call "+i.module+"."+i.name)};
}
const result=new WebAssembly.Instance(mod,imports).exports.run();
if(result!==14554)throw new Error("native frame oracle mismatch: "+result);
console.log("wasm32 frame oracle suite passed (including signed headers; panic recovery checked on host)");
""")
    if args.test_prefix:
        check = work / "check.cjs"
        check.write_text(check.read_text().replace("wasm32 frame oracle suite passed (including signed headers; panic recovery checked on host)", "wasm32 selected oracle checks passed"))
    subprocess.run(["cargo", "build", "--offline", "--manifest-path", str(work / "Cargo.toml"),
                    "--target", "wasm32-unknown-unknown"], check=True)
    target = Path(os.environ.get("CARGO_TARGET_DIR", str(work / "target"))).resolve()
    subprocess.run(["node", str(work / "check.cjs"),
                    str(target / "wasm32-unknown-unknown/debug/halo_frame_lazy_width_wasm.wasm")], check=True)


if __name__ == "__main__":
    main()
