//! Stateful native direct readers and scan-frame context ownership.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum NativeReaderProfileError {
    #[error("native profile value cannot be represented by this reader: {0}")]
    Width(&'static str),
    #[error("native traversal policy not supported by this entry point: {0}")]
    Policy(&'static str),
}
impl NativeScanProfile {
    // Widths are consumed from NativeComponentWidths only when reached on wire.
    /// Adapt raw metadata to the existing component reader's address-sized widths.
    /// The profile retains original u64 values if this conversion is refused.
    pub(crate) fn component_encoding(&self) -> Result<PositionEncoding, NativeReaderProfileError> {
        let width =
            |w: u64, name| usize::try_from(w).map_err(|_| NativeReaderProfileError::Width(name));
        let axes = |w: [u64; 3], name| -> Result<[usize; 3], NativeReaderProfileError> {
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
            // Native deltaAxisW treats zero as absence of an override and
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
#[derive(Debug, Clone, Default)]
pub(crate) struct NativeReaderContext {
    pub profile: NativeScanProfile,
}
#[derive(Debug, Clone)]
pub(crate) struct NativeFrameConfig {
    pub extra_fields: bool,
    pub id_low_bits: i64,
    pub id_base: u32,
    pub new_default_state_bits: i64,
    pub packet_preamble_bits: i64,
    pub context: NativeReaderContext,
}
impl Default for NativeFrameConfig {
    fn default() -> Self {
        Self {
            extra_fields: false,
            id_low_bits: 13,
            id_base: 0,
            new_default_state_bits: 0,
            packet_preamble_bits: 2,
            context: NativeReaderContext::default(),
        }
    }
}

impl NativeFrameConfig {
    /// Native sequential keyframe table, including its sentinel fallback behavior.
    pub(crate) fn read_keyframe_table(
        &self,
        data: &[u8],
        registry: &FilmRegistry,
    ) -> Result<NativeKeyframeTable, NativeReaderProfileError> {
        Ok(components::decode_native_keyframe_table_contextual(
            data,
            registry,
            &self.contextual_frame_encoding()?,
            Some(&self.context),
        ))
    }

    /// Decode the native message/entity/control view classes using the default
    /// view-table admission policy. Optional inference without view tables uses
    /// a separate traversal and is refused here rather than silently ignored.
    pub(crate) fn decode_production_views(
        &self,
        data: &[u8],
        skip_lead_bits: impl TryInto<i64>,
        registry: &FilmRegistry,
        world: &mut FilmWorld,
    ) -> Result<ProductionFrame, NativeReaderProfileError> {
        if !self.context.profile.grammar.view_classes || !self.context.profile.grammar.view_tables {
            return Err(NativeReaderProfileError::Policy(
                "requires view classes and view tables",
            ));
        }
        let encoding = self.contextual_frame_encoding()?;
        let preamble_bits = self.packet_preamble_bits;
        production_frame::decode_production_frame_contextual(
            data,
            skip_lead_bits,
            registry,
            &encoding,
            world,
            production_frame::ProductionReaderContext {
                reader: Some(&self.context),
                preamble_bits,
            },
            |_, _, _| {},
        )
        .ok_or(NativeReaderProfileError::Width("frame reader"))
    }

    /// Live component reads obtain dimensions from the raw runtime profile.
    /// These representable dimensions carry only position policies through the
    /// checked legacy adapter; they must never be used as consumed widths.
    pub(crate) fn contextual_frame_encoding(
        &self,
    ) -> Result<FrameEncoding, NativeReaderProfileError> {
        let mut policies = self.clone();
        let movement = &mut policies.context.profile.movement;
        movement.world_object.index_bits = 1;
        movement.world_object.axis_bits = [1; 3];
        movement.traversal.index_bits = 1;
        movement.traversal.axis_bits = [1; 3];
        movement.delta_axis_width = 1;
        // The terminal tail is admitted only after a successful New body.
        policies.context.profile.grammar.new_record_tail_bits = 0;
        policies.context.profile.mpp = FilmMppWidths::default();
        // Keyframe consumers use the raw context, not this legacy placeholder.
        policies.context.profile.keyframe = NativeKeyframeLayout::default();
        policies.new_default_state_bits = 0;
        policies.id_low_bits = 0;
        policies.id_base = 0;
        let mut encoding = policies.frame_encoding()?;
        encoding.new_record.native_fallback_default_bits = Some(self.new_default_state_bits);
        encoding.native_id_low_bits = Some(self.id_low_bits);
        encoding.ids.base = self.id_base;
        // Native Film retains quantized fields; map calibration belongs to resolution.
        encoding.position_capture = None;
        Ok(encoding)
    }

    /// Adapt the native frame settings to the supported v41 record reader.
    /// Raw metadata remains on this configuration when widths are unsupported.
    pub(crate) fn frame_encoding(&self) -> Result<FrameEncoding, NativeReaderProfileError> {
        let width = |value: i64, field| {
            usize::try_from(value).map_err(|_| NativeReaderProfileError::Width(field))
        };
        let p = &self.context.profile;
        let position = p.component_encoding()?;
        let encoding = FrameEncoding {
            keyframe_layout: p
                .keyframe
                .bounded()
                .ok_or(NativeReaderProfileError::Width("keyframe layout"))?,
            keyframe_simulation_complete: Some(p.grammar.simulation_complete),
            native_id_low_bits: None,
            component_widths: ComponentWidthOverrides::default(),
            new_record: NewRecordEncoding {
                deserialize_defaults: p.grammar.default_state_by_archetype,
                fallback_default_bits: width(self.new_default_state_bits, "new default state")?,
                native_fallback_default_bits: None,
                // Native TraverseEntity only skips a strictly positive tail.
                terminal_bits: width(p.grammar.new_record_tail_bits.max(0), "new record tail")?,
            },
            position_capture: Some(PositionCaptureEncoding {
                min_bits: p.movement.range.map(|r| r[0].to_bits()),
                max_bits: p.movement.range.map(|r| r[1].to_bits()),
                quantum_bits: p.movement.delta_quantum.to_bits(),
                region: p.movement.world_object.region,
                axis_widths: position.world_axis_bits.unwrap(),
                region_index_bits: position.index_bits,
            }),
            ids: RecordIdLayout {
                low_bits: width(self.id_low_bits, "record ID")?,
                base: self.id_base,
            },
            mpp_widths: [
                width(p.mpp.lead, "MPP lead")?,
                width(p.mpp.index, "MPP index")?,
            ],
            position: Some(position),
            extra_fields: self.extra_fields,
            corruption_check: p.grammar.corruption_check,
        };
        if !encoding.valid() {
            return Err(NativeReaderProfileError::Width("frame encoding"));
        }
        Ok(encoding)
    }
}
