//! Shared v41 decode profiles and explicit structural reader configuration.
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::{
    entities, events::ProjectilePositionEncoding,
};
#[cfg(test)]
use crate::theater::parser::v41::chunks::replication::replication_stream::keyframes::table::decode_reference_keyframe_table_contextual;
use crate::theater::parser::v41::context::map::REFERENCE_MPP_DEFAULT_WIDTHS;
pub(crate) mod events;
pub(crate) mod layout;
pub(crate) mod map;
pub(crate) mod movement;
pub(crate) mod players;
pub(crate) mod profile;
pub(crate) mod values;
use crate::theater::parser::v41::chunks::registry::source::FilmRegistry;
use crate::theater::parser::v41::chunks::replication::components::position::{
    ComponentBodyPolicy, PositionEncoding,
};
use crate::theater::parser::v41::chunks::replication::components::widths::ComponentWidthOverrides;
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::header::RecordIdLayout;
#[cfg(test)]
use crate::theater::parser::v41::chunks::replication::source::KeyframeTable;
use crate::theater::parser::v41::chunks::replication::source::{FrameDecodeError, ProductionFrame};
use crate::theater::parser::v41::chunks::replication::state::ReplicationDecodeState;
use crate::theater::parser::v41::context::layout::{NewRecordLayout, RecordLayout};
use crate::theater::parser::v41::context::profile::DecodeProfile;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum ReaderProfileError {
    #[error("reference profile value cannot be represented by this reader: {0}")]
    Width(&'static str),
    #[error("reference traversal policy not supported by this entry point: {0}")]
    Policy(&'static str),
}
impl From<ReaderProfileError> for FrameDecodeError {
    fn from(value: ReaderProfileError) -> Self {
        match value {
            ReaderProfileError::Width(field) => Self::Width {
                field: field.to_owned(),
            },
            ReaderProfileError::Policy(_) => Self::MissingViewTables,
        }
    }
}
impl DecodeProfile {
    // Widths are consumed from ComponentWidths only when reached on wire.
    /// Adapt raw metadata to the existing component reader's address-sized widths.
    /// The profile retains original u64 values if this conversion is refused.
    #[cfg(test)]
    pub(crate) fn component_encoding(&self) -> Result<PositionEncoding, ReaderProfileError> {
        let width = |w: u64, name| usize::try_from(w).map_err(|_| ReaderProfileError::Width(name));
        let axes = |w: [u64; 3], name| -> Result<[usize; 3], ReaderProfileError> {
            Ok([width(w[0], name)?, width(w[1], name)?, width(w[2], name)?])
        };
        let world = &self.movement.world_object;
        let traversal = &self.movement.traversal;
        let world_axes = axes(world.axis_bits, "world axes")?;
        Ok(PositionEncoding {
            bodies: ComponentBodyPolicy {
                ability_anchor: self.grammar.ability_anchor_body,
                mobility: self.grammar.mobility_action_body,
                mobility_extra_bits: self.movement.mobility_action_extra_bits,
            },
            index_bits: width(world.index_bits, "world index")?,
            world_axis_bits: Some(world_axes),
            handle_bits: width(traversal.index_bits, "traversal index")?,
            traversal_axis_bits: axes(traversal.axis_bits, "traversal axes")?,
            default_axis_bits: [22; 3],
            region_axis_bits: [(world.region, world_axes)].into(),
            // Reference deltaAxisW treats zero as absence of an override and
            // selects the corresponding axis from the traversal descriptor.
            delta_axis_bits: if self.movement.delta_axis_width == 0 {
                axes(traversal.axis_bits, "delta traversal axes")?
            } else {
                [width(self.movement.delta_axis_width, "delta axes")?; 3]
            },
            calibrated_skip: self.movement.calibrated_skip,
            baseline_scope: self.grammar.baseline_scope,
            full_precision: self.movement.full_precision,
            writer_absolute: self.grammar.writer_absolute,
            engine_one_possible: true,
            delta_handle_tail: self.movement.delta_has_handle_tail,
        })
    }
}
#[derive(Debug, Clone, Copy)]
pub(crate) enum CorruptionCheckContext {
    Profile,
    Recorded(bool),
    Unavailable,
}

#[derive(Debug, Clone)]
pub(crate) struct V41DecodeConfig {
    /// Public Film/replay admission replaces Profile with recorded evidence or
    /// Unavailable. Profile is for explicitly configured internal readers.
    pub corruption_checks: CorruptionCheckContext,
    pub event_versions: events::EventVersionContext,
    pub players: players::PlayerContext,
    pub event_variant: events::VariantContext,
    pub event_kill_tail: Option<bool>,
    /// External script-event prefix only when recorder simulation is absent.
    pub event_script_prefix: Option<bool>,
    pub control_record_limit: usize,
    /// Runtime-only legacy hypotheses, reported on the resulting event list.
    pub event_runtime_defaults: bool,
    /// Indexed event widths available from verified level data or runtime context.
    pub event_quantized_vectors: bool,
    pub event_fire_v3: bool,
    pub projectile_event_positions: Option<ProjectilePositionEncoding>,
    pub guard_rejected_frame_deletes: bool,
    /// Runtime-owned inference hook. Stored Film parsing always leaves this absent.
    pub schema_recovery: Option<entities::SchemaRecoveryReader>,
    pub extra_fields: bool,
    pub id_low_bits: i64,
    pub id_base: u32,
    pub new_default_state_bits: i64,
    #[cfg(test)]
    pub packet_preamble_bits: i64,
    pub profile: DecodeProfile,
}
impl Default for V41DecodeConfig {
    fn default() -> Self {
        Self {
            corruption_checks: CorruptionCheckContext::Profile,
            event_versions: events::EventVersionContext::Native,
            players: players::PlayerContext::default(),
            event_variant: events::VariantContext::default(),
            event_kill_tail: None,
            event_script_prefix: None,
            control_record_limit: 1 << 16,
            event_runtime_defaults: false,
            event_quantized_vectors: false,
            event_fire_v3: false,
            projectile_event_positions: None,
            guard_rejected_frame_deletes: false,
            schema_recovery: None,
            extra_fields: false,
            id_low_bits: 13,
            id_base: 0,
            new_default_state_bits: 0,
            #[cfg(test)]
            packet_preamble_bits: 2,
            profile: DecodeProfile {
                grammar: profile::DecodeGrammar {
                    simulation_complete: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

impl V41DecodeConfig {
    /// Rebuild the exact recorded context used by canonical Film parsing.
    /// Diagnostic readers can reuse it without introducing a second set of
    /// bootstrap or map assumptions.
    pub(crate) fn from_registry(
        registry: &crate::theater::parser::v41::chunks::registry::source::FilmRegistryRead,
    ) -> Self {
        let event_versions = events::EventVersionContext::recorded(registry);
        let mut config = Self {
            event_versions,
            players: players::PlayerContext::recorded(registry),
            event_variant: events::recorded_variant(registry),
            event_fire_v3: event_versions.is_native(),
            ..Default::default()
        };
        if let Some(level) = map::recorded_level(registry)
            && let (Some(world), Some(detonation), Some(impact)) = (
                map::level_precision(level, 16),
                map::level_precision(level, 15),
                map::level_precision(level, 12),
            )
        {
            config.profile.movement.world_object = world;
            config.event_quantized_vectors = true;
            config.projectile_event_positions =
                Some(ProjectilePositionEncoding { detonation, impact });
        }
        config.apply_recorded_corruption_checks(registry.bootstrap_header.as_ref());
        config
    }

    /// Bind source grammar after selecting a profile. Absence remains unknown;
    /// readers stop only when they reach a boundary affected by this policy.
    pub(crate) fn apply_recorded_corruption_checks(
        &mut self,
        header: Option<&crate::theater::parser::v41::chunks::registry::source::BootstrapHeaderRead>,
    ) {
        self.corruption_checks = match header.and_then(|h| h.corruption_checks()) {
            Some(enabled) => CorruptionCheckContext::Recorded(enabled),
            None => CorruptionCheckContext::Unavailable,
        };
        if let CorruptionCheckContext::Recorded(enabled) = self.corruption_checks {
            self.profile.grammar.corruption_check = enabled;
        }
    }

    pub(crate) fn corruption_check(&self) -> Option<bool> {
        match self.corruption_checks {
            CorruptionCheckContext::Profile => Some(self.profile.grammar.corruption_check),
            CorruptionCheckContext::Recorded(enabled) => Some(enabled),
            CorruptionCheckContext::Unavailable => None,
        }
    }

    /// Reference sequential keyframe table, including its sentinel fallback behavior.
    #[cfg(test)]
    pub(crate) fn read_keyframe_table(
        &self,
        data: &[u8],
        registry: &FilmRegistry,
    ) -> Result<KeyframeTable, ReaderProfileError> {
        Ok(decode_reference_keyframe_table_contextual(
            data,
            registry,
            &self.record_layout(),
            Some(&self.profile),
        ))
    }

    /// Decode the reference message/entity/control view classes using the default
    /// view-table admission policy. Optional inference without view tables uses
    /// a separate traversal and is refused here rather than silently ignored.
    #[cfg(test)]
    pub(crate) fn decode_production_views(
        &self,
        data: &[u8],
        skip_lead_bits: impl TryInto<i64>,
        registry: &FilmRegistry,
        world: &mut ReplicationDecodeState,
    ) -> Result<ProductionFrame, ReaderProfileError> {
        self.decode_views(
            data,
            skip_lead_bits
                .try_into()
                .map_err(|_| ReaderProfileError::Width("frame offset"))?,
            registry,
            world,
            &self.record_layout(),
            entities::ProductionReaderContext {
                schema_recovery: self.schema_recovery,
                reader: Some(&self.profile),
                preamble_bits: self.packet_preamble_bits,
                control_record_limit: self.control_record_limit,
                explicit_delta_schema: None,
                reference_controls_after_rejection: true,
            },
        )
    }

    fn decode_views(
        &self,
        data: &[u8],
        bit: i64,
        registry: &FilmRegistry,
        world: &mut ReplicationDecodeState,
        encoding: &RecordLayout,
        context: entities::ProductionReaderContext<'_>,
    ) -> Result<ProductionFrame, ReaderProfileError> {
        if !self.profile.grammar.view_classes || !self.profile.grammar.view_tables {
            return Err(ReaderProfileError::Policy(
                "requires view classes and view tables",
            ));
        }
        entities::decode_production_frame_contextual(
            data,
            bit,
            registry,
            encoding,
            world,
            context,
            |_, _, _| {},
        )
        .ok_or(ReaderProfileError::Width("frame reader"))
    }

    /// Entity/control views after a message terminator already read on wire.
    #[cfg(test)]
    pub(crate) fn decode_entity_control_views(
        &self,
        data: &[u8],
        bit: usize,
        registry: &FilmRegistry,
        world: &mut ReplicationDecodeState,
    ) -> Result<ProductionFrame, ReaderProfileError> {
        self.decode_entity_control_views_with_schemas(data, bit, registry, world, None)
    }

    pub(crate) fn decode_entity_control_views_with_schemas(
        &self,
        data: &[u8],
        bit: usize,
        registry: &FilmRegistry,
        world: &mut ReplicationDecodeState,
        explicit_delta_schema: Option<&dyn Fn(i64, u32) -> Option<u32>>,
    ) -> Result<ProductionFrame, ReaderProfileError> {
        let mut encoding = self.record_layout();
        encoding.category_references = data.first().is_some_and(|b| b & 0x80 != 0);
        self.decode_views(
            data,
            bit as i64,
            registry,
            world,
            &encoding,
            entities::ProductionReaderContext {
                schema_recovery: self.schema_recovery,
                reader: Some(&self.profile),
                preamble_bits: -1,
                control_record_limit: self.control_record_limit,
                explicit_delta_schema,
                reference_controls_after_rejection: false,
            },
        )
    }

    /// Build structural record policy without cloning or coercing the raw profile.
    /// Component widths and signed keyframe dimensions are read lazily from the
    /// profile at their wire positions. The fallback widths here are unused when
    /// that profile is supplied; they keep the bounded adapter well formed.
    pub(crate) fn record_layout(&self) -> RecordLayout {
        let p = &self.profile;
        RecordLayout {
            category_references: true,
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: Some(p.grammar.simulation_complete),
            snapshot_vehicle_type_gate: None,
            reference_id_low_bits: Some(self.id_low_bits),
            component_widths: ComponentWidthOverrides::default(),
            new_record: NewRecordLayout {
                deserialize_defaults: p.grammar.default_state_by_archetype,
                fallback_default_bits: 0,
                reference_fallback_default_bits: Some(self.new_default_state_bits),
                terminal_bits: 0,
            },
            ids: RecordIdLayout {
                low_bits: 0,
                base: self.id_base,
            },
            mpp_widths: REFERENCE_MPP_DEFAULT_WIDTHS,
            position: Some(PositionEncoding {
                bodies: ComponentBodyPolicy {
                    ability_anchor: p.grammar.ability_anchor_body,
                    mobility: p.grammar.mobility_action_body,
                    mobility_extra_bits: p.movement.mobility_action_extra_bits,
                },
                index_bits: 1,
                world_axis_bits: Some([1; 3]),
                handle_bits: 1,
                traversal_axis_bits: [1; 3],
                default_axis_bits: [22; 3],
                region_axis_bits: [(p.movement.world_object.region, [1; 3])].into(),
                delta_axis_bits: [1; 3],
                calibrated_skip: p.movement.calibrated_skip,
                baseline_scope: p.grammar.baseline_scope,
                full_precision: p.movement.full_precision,
                writer_absolute: p.grammar.writer_absolute,
                engine_one_possible: self.event_variant.engine_type.is_none()
                    || self.event_variant.engine_type == Some(1),
                delta_handle_tail: p.movement.delta_has_handle_tail,
            }),
            extra_fields: self.extra_fields,
            corruption_check: self.corruption_check(),
        }
    }
}
