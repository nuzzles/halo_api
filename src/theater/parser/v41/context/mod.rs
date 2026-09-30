//! Shared v41 decode profiles and explicit structural reader configuration.
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::entities;
use crate::theater::parser::v41::chunks::replication::replication_stream::keyframes::table::decode_reference_keyframe_table_contextual;
use crate::theater::parser::v41::context::map::REFERENCE_MPP_DEFAULT_WIDTHS;
pub(crate) mod layout;
pub(crate) mod map;
pub(crate) mod movement;
pub(crate) mod profile;
pub(crate) mod values;
use crate::theater::film::{FilmRegistry, FrameDecodeError, KeyframeTable, ProductionFrame};
use crate::theater::parser::v41::chunks::replication::components::position::{
    ComponentBodyPolicy, PositionEncoding,
};
use crate::theater::parser::v41::chunks::replication::components::widths::ComponentWidthOverrides;
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::header::RecordIdLayout;
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
            delta_handle_tail: self.movement.delta_has_handle_tail,
        })
    }
}
#[derive(Debug, Clone)]
pub(crate) struct V41DecodeConfig {
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
            extra_fields: false,
            id_low_bits: 13,
            id_base: 0,
            new_default_state_bits: 0,
            #[cfg(test)]
            packet_preamble_bits: 2,
            profile: DecodeProfile::default(),
        }
    }
}

impl V41DecodeConfig {
    /// Reference sequential keyframe table, including its sentinel fallback behavior.
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
            self.packet_preamble_bits,
            registry,
            world,
        )
    }

    fn decode_views(
        &self,
        data: &[u8],
        bit: i64,
        preamble_bits: i64,
        registry: &FilmRegistry,
        world: &mut ReplicationDecodeState,
    ) -> Result<ProductionFrame, ReaderProfileError> {
        if !self.profile.grammar.view_classes || !self.profile.grammar.view_tables {
            return Err(ReaderProfileError::Policy(
                "requires view classes and view tables",
            ));
        }
        let encoding = self.record_layout();
        entities::decode_production_frame_contextual(
            data,
            bit,
            registry,
            &encoding,
            world,
            entities::ProductionReaderContext {
                reader: Some(&self.profile),
                preamble_bits,
            },
            |_, _, _| {},
        )
        .ok_or(ReaderProfileError::Width("frame reader"))
    }

    /// Entity/control views after a message terminator already read on wire.
    pub(crate) fn decode_entity_control_views(
        &self,
        data: &[u8],
        bit: usize,
        registry: &FilmRegistry,
        world: &mut ReplicationDecodeState,
    ) -> Result<ProductionFrame, ReaderProfileError> {
        self.decode_views(data, bit as i64, -1, registry, world)
    }

    /// Build structural record policy without cloning or coercing the raw profile.
    /// Component widths and signed keyframe dimensions are read lazily from the
    /// profile at their wire positions. The fallback widths here are unused when
    /// that profile is supplied; they keep the bounded adapter well formed.
    pub(crate) fn record_layout(&self) -> RecordLayout {
        let p = &self.profile;
        RecordLayout {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: Some(p.grammar.simulation_complete),
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
                delta_handle_tail: p.movement.delta_has_handle_tail,
            }),
            extra_fields: self.extra_fields,
            corruption_check: p.grammar.corruption_check,
        }
    }
}
