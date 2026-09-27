//! Typed, offline Halo Infinite Theater decoding.
//!
//! Parse a recording with [`Film::parse`], then call [`Film::resolve`] to build
//! chronological query indexes and stateful playback. [`film`] owns native
//! recording data; [`resolved`] owns accumulated state and traversal.
//!
//! [`decode_summary_events`] reads stored kills, deaths, mode highlights and named
//! medal awards using only decompressed footer chunks. [`validate_summary_events`]
//! compares per-player counts and individual medal identities with match stats.
//!
//! ```no_run
//! use halo_api::theater::{Film, FilmSource, film::ParseOptions};
//! # fn example(source: &FilmSource) -> Result<(), Box<dyn std::error::Error>> {
//! let film = Film::parse(source, ParseOptions::default())?;
//! let mut resolved = film.resolve();
//! resolved.advance_to(10_000_000);
//! let current_world = resolved.current();
//! # Ok(())
//! # }
//! ```
//!
//! The former mixed parser and its compatibility namespace are not public APIs:
//!
//! ```compile_fail
//! use halo_api::theater::legacy::Film;
//! ```
//!
//! ```compile_fail
//! use halo_api::theater::{Film, DecodeOptions};
//! Film::try_from_chunks(&[], DecodeOptions::v41());
//! ```

mod ability_charges;
mod ability_states;
mod anticipated_bindings;
mod appearance;
mod biped_capture;
mod biped_channels;
mod biped_creation;
mod biped_scan;
mod i0_layout;
pub use i0_layout::*;
mod bits;
mod bootstrap;
mod bot_metadata;
mod combat;
mod components;
mod coordinates;
mod datums;
mod decode;
mod equipment_changes;
mod equipment_recovery;
mod event_heads;
mod kill_assists;
mod kill_bijection;
mod kill_calibration;
mod kill_damage_tags;
mod kill_event_chain;
mod kill_feed;
mod kill_film_evidence;
mod kill_roster;
pub use kill_calibration::*;
mod objective_source;
pub use objective_source::*;
mod objective_extract;
mod objective_scan;
pub use objective_extract::*;
pub use objective_scan::*;
mod statborg_awards;
pub use statborg_awards::*;
mod flag_grabs_net;
pub use flag_grabs_net::*;
mod replay_kill_inputs;
pub use replay_kill_inputs::*;
mod kill_decode;
pub use kill_decode::*;
mod kill_health;
pub use kill_health::*;
mod kill_hybrid;
pub use kill_hybrid::*;
mod kill_matching;
pub use kill_matching::*;
mod kill_walk;
pub use kill_walk::*;
mod kill_timeline;
pub use kill_film_evidence::*;
pub use kill_timeline::*;
mod kill_source_scan;
pub use kill_bijection::*;
pub use kill_damage_tags::*;
pub use kill_source_scan::*;
mod kill_rng;
pub use kill_roster::*;
mod kill_feed_pairs;
pub use kill_feed::*;
pub use kill_feed_pairs::*;
mod kill_event_scan;
pub use kill_assists::*;
pub use kill_event_chain::*;
pub use kill_event_scan::*;
mod ground_weapon_lifetimes;
mod ground_weapon_objects;
mod ground_weapon_pads;
mod identity_closures;
mod identity_health;
mod identity_life_export;
mod identity_owners;
mod identity_registry;
mod identity_section;
pub use identity_health::*;
pub use identity_life_export::*;
pub use identity_owners::*;
pub use identity_registry::*;
pub use identity_section::*;
mod identity_creations;
mod identity_death_clock;
mod identity_deaths;
mod identity_lifetimes;
mod identity_scoreboard;
mod identity_tables;
mod replay_vehicle_families;
pub use replay_vehicle_families::*;
mod replay_vehicle_rides;
pub use replay_vehicle_rides::*;
mod film_vehicles;
pub use film_vehicles::*;
mod replay_vehicle_publication;
pub use replay_vehicle_publication::*;
mod replay_vehicle_cycles;
pub use replay_vehicle_cycles::*;
mod replay_vehicle_coverage;
pub use replay_vehicle_coverage::*;
mod replay_vehicle_episodes;
pub use replay_vehicle_episodes::*;
mod replay_vehicle_rides_film;
pub use replay_vehicle_rides_film::*;
mod replay_vehicle_tracks;
pub use replay_vehicle_tracks::*;
mod native_march;
pub use native_march::*;
mod march_scan;
pub use march_scan::*;
mod march_walk;
pub use march_walk::*;
mod march_facts;
pub use march_facts::*;
mod biped_aim;
pub use biped_aim::*;
mod vehicle_events;
pub use vehicle_events::*;
mod vehicle_scan;
pub use vehicle_scan::*;
mod orientation_frame;
mod pad_pickup_dating;
mod pickup_origin;
mod player_indices;
mod replay_ability_impulses;
pub use replay_ability_impulses::*;
mod replay_ground_film;
mod replay_ground_items;
pub use replay_ground_film::*;
pub use replay_ground_items::*;
mod replay_pickups_film;
pub use replay_pickups_film::*;
mod replay_weapon_changes;
pub use replay_weapon_changes::*;
mod replay_equipment_placements;
pub use replay_equipment_placements::*;
mod replay_equipment_ends;
pub use replay_equipment_ends::*;
mod replay_equipment_origin;
pub use replay_equipment_origin::*;
mod replay_equipment_kills;
pub use replay_equipment_kills::*;
mod replay_equipment_episodes;
pub use replay_equipment_episodes::*;
mod replay_grapple;
pub use replay_grapple::*;
mod replay_translocations;
pub use replay_translocations::*;
mod replay_equipment_changes;
pub use replay_equipment_changes::*;
mod replay_inventory;
pub use replay_inventory::*;
mod replay_abilities_film;
pub use replay_abilities_film::*;
mod replay_abilities;
pub use replay_abilities::*;
mod replay_ability_charges;
mod replay_clock;
pub use replay_ability_charges::*;
mod replay_combat;
pub use orientation_frame::*;
mod replay_vehicle_lives;
mod replay_vehicle_relays;
pub use replay_vehicle_relays::*;
mod replay_vehicle_shots;
mod replay_vehicles;
pub use replay_vehicle_lives::*;
pub use replay_vehicle_shots::*;
pub use replay_vehicles::*;
mod replay_grenade_reads;
mod replay_grenades;
pub use replay_grenade_reads::*;
mod replay_loadouts;
mod replay_projectiles;
mod replay_shots;
pub use replay_combat::*;
pub use replay_grenades::*;
pub use replay_loadouts::*;
pub use replay_projectiles::*;
pub use replay_shots::*;
mod replay_from_film;
pub use replay_from_film::*;
mod replay_scope;
pub use replay_scope::*;
mod replay_death_context;
mod replay_evidence;
mod replay_identity;
mod replay_kill_positions;
pub use replay_clock::*;
pub use replay_death_context::*;
pub use replay_evidence::*;
pub use replay_kill_positions::*;
mod replay_pickups;
pub use bot_metadata::*;
pub use ground_weapon_objects::*;
pub use ground_weapon_pads::*;
pub use identity_closures::*;
pub use identity_creations::*;
pub use identity_death_clock::*;
pub use identity_deaths::*;
pub use identity_lifetimes::*;
pub use identity_scoreboard::*;
pub use identity_tables::*;
pub use pad_pickup_dating::*;
pub use pickup_origin::*;
pub use player_indices::*;
pub use replay_identity::*;
pub use replay_pickups::*;
mod film_key;
mod registry_catalog;
pub use registry_catalog::*;
mod head_observations;
mod highlight_events;
mod kill_icons;
pub use film_key::*;
mod native_medal_names;
pub use kill_icons::*;
pub use native_medal_names::*;
mod highlights;
pub use highlight_events::*;
mod input;
mod inventory_delta;
mod keyframe_closure;
mod keyframe_diagnostics;
#[cfg(not(target_arch = "wasm32"))]
mod packet_discovery;
mod profile_table;
mod profile_values;
mod slot_band;
pub use keyframe_closure::*;
pub use keyframe_diagnostics::*;
#[cfg(not(target_arch = "wasm32"))]
pub use packet_discovery::*;
pub use profile_table::*;
pub use profile_values::*;
pub use slot_band::*;
mod keyframe_datums;
mod keyframe_ground_weapons;
pub use ground_weapon_lifetimes::*;
mod keyframe_inventory;
mod keyframe_loadouts;
mod keyframe_spans;
pub use keyframe_ground_weapons::*;
pub use keyframe_spans::*;
mod context_movement;
mod map_catalog;
mod medals;
mod motion;
mod movement_states;
mod packets;
mod production_frame;
mod profile;
mod replay_movement_scan;
pub use map_catalog::*;
mod projectile;
mod quantization;
mod records;
mod recovery;
mod registry;
mod replication;
mod roster;
mod summary;
mod translocator;
mod types;
mod velocity;
mod vitality;
mod weapon_changes;
mod weapon_timing;
pub use weapon_timing::*;
mod weapon_hit_scan;
pub use weapon_hit_scan::*;
mod weapon_hits;
pub use weapon_hits::*;
mod world;
mod world_precision;
pub use world_precision::*;

pub use ability_charges::*;
pub use ability_states::*;
pub use anticipated_bindings::*;
pub use biped_capture::*;
pub use biped_channels::*;
pub use biped_creation::*;
pub use biped_scan::*;
pub use bits::Cursor as FilmBitReader;
pub use bootstrap::*;
mod native_identity;
pub use components::*;
pub use context_movement::*;
pub use coordinates::*;
pub use datums::*;
pub use equipment_changes::*;
pub use equipment_recovery::*;
pub use event_heads::*;
pub use head_observations::*;
pub use highlights::*;
pub use inventory_delta::*;
pub use keyframe_datums::*;
pub use keyframe_inventory::*;
pub use keyframe_loadouts::*;
pub use medals::*;
pub use movement_states::*;
pub use native_identity::*;
pub use production_frame::*;
pub use profile::*;
pub use quantization::*;
pub use records::*;
pub use recovery::*;
pub use registry::*;
pub use replay_movement_scan::*;
pub use replication::*;
pub use roster::*;
pub use summary::*;
pub use translocator::*;
pub(crate) use types::LegacyFilm;
pub use types::{
    Aim, AppearanceSample, ArmorAppearance, ArmorAttachments, ArmorRegion, BodyVitality,
    CheckedRegion, ClockSample, CoordinateLayout, DecodeDiagnostics, DecodeError, DecodeOptions,
    FilmEventCounts, FilmPacket, FilmSourceChunk, Firing, InputAxes, Life, Magazine, PlayerTrack,
    Position, ProjectileTrack, Sample, ShieldVitality, SourceSpan, SummaryEvent, SummaryKind,
    Velocity, WeaponObservation, ZoomStage,
};
pub use vitality::*;
pub use weapon_changes::*;
pub use world::*;

#[cfg(test)]
mod corpus_tests;
#[cfg(test)]
mod tests;

mod player_table;
pub use player_table::*;

mod roster_updates;
pub use roster_updates::*;

mod control_verdict;
#[cfg(test)]
mod native_new_binding_tests;
pub use control_verdict::*;
mod fire_events;
pub use fire_events::*;

mod context_grenades;
mod grenade_throws;
pub use context_grenades::*;
pub use grenade_throws::*;

mod world_object_research;
mod world_object_tracks;
pub use world_object_research::*;
pub use world_object_tracks::*;

mod native_sort;

mod world_object_keyframes;
pub use world_object_keyframes::*;

mod equipment_creations;
mod equipment_placements;
pub use equipment_creations::*;
pub use equipment_placements::*;

mod statborg_rounds;
pub use statborg_rounds::*;

mod statborg_source_passes;
pub use statborg_source_passes::*;
mod statborg_source;
pub use statborg_source::*;
mod statborg_decode;
pub use statborg_decode::*;

mod statborg_budget_diagnostics;
pub use statborg_budget_diagnostics::*;
mod statborg_series;
pub use statborg_series::*;

mod statborg_identity;
pub use statborg_identity::*;

mod statborg_residue;
pub use statborg_residue::*;

mod statborg_named;
pub use statborg_named::*;

mod identity_remaining;
pub use identity_remaining::*;

mod identity_tracks;
pub use identity_tracks::*;

mod identity_successions;
pub use identity_successions::*;

mod replay_tracks;
pub use replay_tracks::*;

mod replay_bounds;
pub use replay_bounds::*;

mod player_teams;
pub use player_teams::*;

mod replay_teams;
pub use replay_teams::*;

mod replay_seats;
pub use replay_seats::*;

mod replay_players;
pub use replay_players::*;

mod replay_score_series;
pub use replay_score_series::*;

mod replay_score_players;
pub use replay_score_players::*;

mod replay_score_teams;
pub use replay_score_teams::*;

mod replay_score_hold;
pub use replay_score_hold::*;
mod replay_score;
pub use replay_score::*;

mod replay_objective_objects;
pub use replay_objective_objects::*;

mod carrier_marks;
pub use carrier_marks::*;

mod replay_match_clock;
pub use replay_match_clock::*;
mod replay_vip;
pub use replay_vip::*;

mod replay_skull;
pub use replay_skull::*;
mod replay_held_object;
pub use replay_held_object::*;
mod replay_bomb_carries;
pub use replay_bomb_carries::*;
mod replay_bomb_film;
pub use replay_bomb_film::*;
mod replay_bomb_stats;
pub use replay_bomb_stats::*;
mod navpoint_radial;
pub use navpoint_radial::*;
mod replay_bomb_armings;
pub use replay_bomb_armings::*;
mod navpoint_radial_scan;
pub use navpoint_radial_scan::*;
mod replay_objective_actions;
pub use replay_objective_actions::*;

mod replay_flag_carries;
pub use replay_flag_carries::*;

mod replay_flag_home;
pub use replay_flag_home::*;

mod replay_flag_geometry;
pub use replay_flag_geometry::*;

mod replay_flag_assignment;
pub use replay_flag_assignment::*;

mod replay_flag_marks;
pub use replay_flag_marks::*;

mod replay_flag_lives;
pub use replay_flag_lives::*;

mod replay_flags;
pub use replay_flags::*;

mod replay_gauges;
pub use replay_gauges::*;
mod managed_property;
pub use managed_property::*;
mod replay_flag_gauges;
pub use replay_flag_gauges::*;

mod replay_flags_film;
pub use replay_flags_film::*;

mod capture_bursts;
pub use capture_bursts::*;

mod map_objective_geometry;
pub use map_objective_geometry::*;
mod map_objectives;
pub use map_objectives::*;

mod replay_map_objectives;
pub use replay_map_objectives::*;

mod replay_zone_series;
pub use replay_zone_series::*;

mod replay_zone_attribution;
pub use replay_zone_attribution::*;

mod replay_zone_pairing;
pub use replay_zone_pairing::*;

mod replay_zone_owners;
pub use replay_zone_owners::*;

mod replay_zone_capturers;
pub use replay_zone_capturers::*;
mod replay_zone_owner_layer;
pub use replay_zone_owner_layer::*;

mod replay_zone_hills;
pub use replay_zone_hills::*;

mod replay_zones;
pub use replay_zones::*;

mod replay_zones_film;
pub use replay_zones_film::*;

mod replay_objective_modes;
pub use replay_objective_modes::*;

mod map_background_time;
pub use map_background_time::*;
mod map_background;
pub use map_background::*;
mod map_background_calibration;
#[cfg(test)]
mod map_background_tests;
pub use map_background_calibration::*;
mod callouts_catalog;
pub use callouts_catalog::*;
#[cfg(test)]
mod callouts_catalog_tests;
mod callouts_json;
mod catalog_integer_zero;
mod replay_geometry_distance;
pub use replay_geometry_distance::*;
mod replay_geometry_csv;
pub use replay_geometry_csv::{GeometryCsvError, GeometryCsvErrorKind, GeometryCsvStage};
mod replay_geometry_float;
mod replay_geometry_loader;
pub use replay_geometry_loader::*;
mod replay_document_types;
#[cfg(test)]
mod replay_geometry_loader_tests;
pub use replay_document_types::*;

mod replay_document_content;
pub use replay_document_content::*;

mod replay_byte_string;
pub use replay_byte_string::*;
mod replay_stances;
pub use replay_stances::*;

mod replay_document_coverage;
pub use replay_document_coverage::*;
mod replay_document;
pub use replay_document::*;
mod replay_t0;
pub use replay_t0::*;
mod replay_decoder_coverage;
pub use replay_decoder_coverage::*;
mod replay_map_weapon_pads;
pub use replay_map_weapon_pads::*;
mod replay_label_catalog;
pub use replay_label_catalog::*;
mod replay_layers;
pub use replay_layers::*;
mod replay_neutral_deaths;
pub use replay_neutral_deaths::*;
mod replay_document_film;
pub use replay_document_film::*;

mod fallback;
pub use fallback::*;

mod replay_usage;
pub use replay_usage::*;

mod precision_law;
pub use precision_law::*;

mod chain_inference;
pub use chain_inference::*;

mod inference_frame;
pub use inference_frame::*;

mod frame_harvest;
pub use frame_harvest::*;

mod position_capture;
pub use position_capture::*;

mod unit_references;
pub use unit_references::*;

mod unit_equipment;
pub use unit_equipment::*;

mod keyframe_position_probe;
pub use keyframe_position_probe::*;

mod weapon_patterns;
pub use weapon_patterns::*;

mod read_diagnostics;
pub use read_diagnostics::*;

#[cfg(not(target_arch = "wasm32"))]
mod film_cache;
#[cfg(not(target_arch = "wasm32"))]
pub use film_cache::*;

mod source;
pub use source::*;

#[cfg(test)]
mod source_bridge_tests;

mod source_octets;
pub use source_octets::*;

mod source_bits;
pub use source_bits::*;

mod kill_source_film;
pub use kill_source_film::*;

#[cfg(test)]
mod statborg_chronology_tests;
mod statborg_component;
pub use statborg_component::*;

mod equipment_state;
pub use equipment_state::*;

mod quantized_source;
pub use quantized_source::*;
mod context_positions;
pub use context_positions::*;
mod context_creations;
pub use context_creations::*;
mod replay_initial_scan;
pub use replay_initial_scan::*;
#[cfg(test)]
mod context_creations_tests;
#[cfg(test)]
mod replay_initial_scan_tests;
mod source_events;
pub use source_events::*;
mod source_inventory;
pub use source_inventory::*;
mod context_pickups;
pub use context_pickups::*;
mod context_ability_impulses;
#[cfg(test)]
mod context_ability_use_tests;
pub use context_ability_impulses::*;
mod context_ability_channels;
#[cfg(test)]
mod context_ability_channels_tests;
pub use context_ability_channels::*;
mod replay_carried_scan;
#[cfg(test)]
mod replay_carried_scan_tests;
pub use replay_carried_scan::*;
mod context_inventory_delta;
#[cfg(test)]
mod context_inventory_delta_tests;
pub use context_inventory_delta::*;
mod context_weapon_changes;
#[cfg(test)]
mod context_weapon_changes_tests;
pub use context_weapon_changes::*;
mod context_delta_walk;
pub use context_delta_walk::*;
#[cfg(test)]
mod context_delta_walk_tests;
#[cfg(test)]
mod source_events_tests;
#[cfg(test)]
mod source_inventory_tests;

#[cfg(test)]
mod log_test_support;

mod native_zoom;
pub use native_zoom::*;

mod native_pickups;
pub use native_pickups::*;

#[cfg(test)]
mod equipment_spawn_source_tests;

mod native_context;
pub use native_context::*;
pub mod film;
#[cfg(test)]
pub(crate) use film::Film as NativeFilmData;
pub use film::*;
pub mod resolved;
pub use resolved::ResolvedFilm;
mod native_packet_heads;
pub use native_packet_heads::*;
mod native_weapon_damage;
pub use native_weapon_damage::*;
mod native_event_gate;
pub use native_event_gate::*;

mod native_profile;
pub use native_profile::*;

mod native_scan_profile;
pub use native_scan_profile::*;

mod native_observer;
pub use native_observer::*;

mod native_reader;
#[cfg(test)]
mod native_reader_cursor_tests;
#[cfg(test)]
mod native_reader_quantization_tests;
pub use native_reader::*;

#[cfg(test)]
mod native_live_observer_tests;

#[cfg(test)]
mod native_context_frame_tests;

#[cfg(test)]
mod native_context_views_tests;

#[cfg(test)]
mod native_context_inference_tests;

#[cfg(test)]
mod native_context_chain_tests;

#[cfg(test)]
mod native_context_resync_tests;

#[cfg(test)]
mod native_context_repair_tests;

#[cfg(test)]
mod native_observer_copy_tests;

#[cfg(test)]
mod native_context_raw_resync_tests;

#[cfg(test)]
mod native_context_resync_frame_tests;

#[cfg(test)]
mod native_signed_width_tests;

#[cfg(test)]
mod native_large_width_tests;

#[cfg(test)]
mod film_movement_retention_tests;

#[cfg(test)]
mod native_frame_accumulator_tests;
#[cfg(test)]
mod native_reader_lifecycle_tests;

#[cfg(test)]
mod native_mobility_extra_tests;

#[cfg(test)]
mod native_keyframe_corpus_tests;

#[cfg(test)]
mod film_input_retention_tests;

#[cfg(test)]
mod vehicle_input_retention_tests;

mod map_background_index;
pub use map_background_index::*;
#[cfg(test)]
mod map_background_index_tests;

mod map_identity_unicode;

#[cfg(not(target_arch = "wasm32"))]
mod map_background_directory;
#[cfg(not(target_arch = "wasm32"))]
pub use map_background_directory::*;

#[cfg(all(test, unix))]
mod map_background_directory_tests;

mod facts_transport;
pub use facts_transport::*;
mod facts_header;
mod facts_quote;
mod facts_quote_tables;
pub use facts_header::*;
mod facts_events;
#[cfg(test)]
mod facts_header_tests;
pub use facts_events::*;
mod facts_inventory;
pub use facts_inventory::*;
#[cfg(test)]
mod facts_body_tests;
mod facts_state_channels;
pub use facts_state_channels::*;
#[cfg(test)]
mod facts_state_channels_tests;
mod facts_world_section;
pub use facts_world_section::*;
mod facts_queue;
pub use facts_queue::*;
mod facts;
#[cfg(test)]
mod facts_tail_tests;
pub use facts::*;
mod facts_file_header;
#[cfg(test)]
mod facts_tests;
pub use facts_file_header::*;
#[cfg(test)]
mod facts_file_header_tests;
mod facts_identity_json;
pub use facts_identity_json::*;
#[cfg(test)]
mod facts_identity_json_tests;

#[cfg(test)]
mod facts_transport_tests;

mod facts_ammo;
pub use facts_ammo::*;
#[cfg(test)]
mod facts_ammo_tests;

mod facts_tracks;
pub use facts_tracks::*;
#[cfg(test)]
mod facts_tracks_tests;

mod facts_keyframes;
pub use facts_keyframes::*;
#[cfg(test)]
mod facts_keyframes_tests;

mod facts_creations;
pub use facts_creations::*;
#[cfg(test)]
mod facts_creations_tests;

mod facts_world;
pub use facts_world::*;

mod facts_directions;
pub use facts_directions::*;
mod facts_positions;
pub use facts_positions::*;
#[cfg(test)]
mod facts_positions_tests;

mod facts_guards;
pub use facts_guards::*;
#[cfg(test)]
mod facts_guards_tests;

mod facts_placement_stats;
pub use facts_placement_stats::*;
#[cfg(test)]
mod facts_placement_stats_tests;

mod facts_json_write;
mod facts_object_deaths;
pub use facts_object_deaths::*;
#[cfg(test)]
mod facts_object_deaths_tests;

mod facts_json_float;

mod facts_json_parse;
mod facts_json_read;

mod facts_channels;
pub use facts_channels::*;
#[cfg(test)]
mod facts_channels_tests;

mod facts_vehicles;
pub use facts_vehicles::*;

mod facts_statborg_json;
pub use facts_statborg_json::*;
#[cfg(test)]
mod facts_statborg_json_tests;

mod facts_kills_json;
pub use facts_kills_json::*;
#[cfg(test)]
mod facts_kills_json_tests;

mod facts_file;
pub use facts_file::*;
#[cfg(test)]
mod facts_file_tests;

mod facts_player_inputs;
pub use facts_player_inputs::*;

#[cfg(test)]
mod facts_player_inputs_tests;

mod facts_shot_inputs;
pub use facts_shot_inputs::*;

#[cfg(test)]
mod facts_shot_inputs_tests;

#[cfg(test)]
mod facts_carried_inputs_tests;

#[cfg(test)]
mod facts_projectile_inputs_tests;

#[cfg(test)]
mod facts_inventory_inputs_tests;

#[cfg(test)]
mod facts_ability_inputs_tests;

#[cfg(test)]
mod facts_player_inventory_tests;

mod facts_equipment_inputs;
pub use facts_equipment_inputs::*;

#[cfg(test)]
mod facts_equipment_inputs_tests;

#[cfg(test)]
mod facts_pickup_inputs_tests;
#[cfg(test)]
mod facts_weapon_inputs_tests;

#[cfg(test)]
mod facts_translocation_inputs_tests;

#[cfg(test)]
mod facts_grapple_inputs_tests;

#[cfg(test)]
mod facts_episode_inputs_tests;

#[cfg(test)]
mod facts_ability_action_inputs_tests;

#[cfg(test)]
mod facts_stance_inputs_tests;

#[cfg(test)]
mod facts_placement_evidence_tests;

#[cfg(test)]
mod facts_ground_inputs_tests;

#[cfg(test)]
mod facts_vehicle_lives_tests;

mod facts_objective_inputs;
pub use facts_objective_inputs::*;
#[cfg(test)]
mod facts_objective_inputs_tests;
mod replay_bomb_document;
pub use replay_bomb_document::*;
mod facts_document;
mod facts_document_options;
#[cfg(test)]
mod facts_document_tests;
mod facts_live_objective_logs;
mod replay_player_document_logs;
#[cfg(test)]
mod replay_signed_layer_tests;
pub use facts_document::*;
pub use facts_document_options::*;
mod facts_live_objectives;
#[cfg(test)]
mod replay_bomb_document_tests;
mod replay_diagnostic_sink;
pub use facts_live_objectives::*;
mod facts_combat;
#[cfg(test)]
mod facts_live_objectives_tests;
pub use facts_combat::*;
#[cfg(test)]
mod facts_combat_tests;

mod replay_score_document;
mod replay_score_document_diagnostics;
pub use replay_score_document::*;
#[cfg(test)]
mod replay_score_document_tests;

mod facts_equipment_document;
pub use facts_equipment_document::*;
#[cfg(test)]
mod facts_equipment_document_tests;

mod replay_coverage_document;
pub use replay_coverage_document::*;
#[cfg(test)]
mod replay_coverage_document_tests;

mod facts_placement_document;
pub use facts_placement_document::*;
#[cfg(test)]
mod facts_placement_document_tests;

mod facts_ground_vehicle_document;
mod facts_pickup_document;
pub use facts_ground_vehicle_document::*;
mod facts_pickup_document_diagnostics;
pub use facts_pickup_document::*;
#[cfg(test)]
mod facts_pickup_document_tests;

#[cfg(test)]
mod facts_ground_vehicle_document_tests;

mod facts_inventory_document;
pub use facts_inventory_document::*;

#[cfg(test)]
mod facts_inventory_document_tests;

mod facts_ability_document;
pub use facts_ability_document::*;

#[cfg(test)]
mod facts_ability_document_tests;

mod replay_finalize_document;
pub use replay_finalize_document::*;

#[cfg(test)]
mod replay_finalize_document_tests;

mod facts_scan_capture;
pub use facts_scan_capture::*;
mod facts_scanned_positions;
pub use facts_scanned_positions::*;
mod facts_replay_provenance;
#[cfg(test)]
mod facts_scan_capture_tests;
#[cfg(test)]
mod facts_scanned_positions_tests;
pub use facts_replay_provenance::*;

#[cfg(test)]
mod facts_replay_provenance_tests;

mod context_ability_charges;
pub use context_ability_charges::*;

#[cfg(test)]
mod context_ability_charges_tests;
mod context_equipment_recovery;
pub use context_equipment_recovery::*;

mod context_equipment_changes;
#[cfg(test)]
mod context_equipment_recovery_tests;
pub use context_equipment_changes::*;

#[cfg(test)]
mod context_equipment_changes_tests;
mod replay_ability_scan;
pub use replay_ability_scan::*;

#[cfg(test)]
mod replay_ability_scan_tests;
mod source_world_events;
pub use source_world_events::*;

#[cfg(test)]
mod source_world_events_tests;

mod context_world_tracks;
pub use context_world_tracks::*;
#[cfg(test)]
mod context_world_tracks_tests;
mod native_equipment_creation;
#[cfg(test)]
mod native_equipment_default_tests;
pub use native_equipment_creation::*;
mod context_mpp_calibration;
#[cfg(test)]
mod native_equipment_creation_tests;
pub use context_mpp_calibration::*;
mod context_equipment_creations;
#[cfg(test)]
mod context_mpp_calibration_tests;
pub use context_equipment_creations::*;
#[cfg(test)]
mod context_equipment_creations_tests;
mod context_equipment_placements;
pub use context_equipment_placements::*;
#[cfg(test)]
mod context_equipment_placements_tests;
mod native_ground_ammo;
pub use native_ground_ammo::*;
#[cfg(test)]
mod context_ground_creations_tests;
#[cfg(test)]
mod native_ground_ammo_tests;
mod replay_pad_scan;
pub use replay_pad_scan::*;
#[cfg(test)]
mod replay_pad_scan_tests;

#[cfg(test)]
mod context_vehicle_creations_tests;

mod context_vehicle_channels;
pub use context_vehicle_channels::*;
#[cfg(test)]
mod context_vehicle_channels_tests;

mod facts_march_scan;
pub use facts_march_scan::*;
mod replay_vehicle_scan;
pub use replay_vehicle_scan::*;
#[cfg(test)]
mod replay_vehicle_scan_tests;

#[cfg(test)]
mod source_carrier_tests;

#[cfg(test)]
mod context_managed_tests;

#[cfg(test)]
mod context_navpoint_tests;

mod replay_guarded_scan;
pub use replay_guarded_scan::*;

#[cfg(test)]
mod replay_guarded_scan_tests;

mod replay_world_scan;
pub use replay_world_scan::*;

#[cfg(test)]
mod replay_world_scan_tests;

#[cfg(test)]
mod source_anticipated_tests;

#[cfg(test)]
mod native_context_strict_locator_tests;

#[cfg(test)]
mod context_movement_tests;

#[cfg(test)]
mod context_grenade_tests;

#[cfg(test)]
mod context_projectile_tests;

mod source_identity_scans;
pub use source_identity_scans::*;
#[cfg(test)]
mod source_identity_scans_tests;
