//! Decoder refusals, width evidence, and aggregate read diagnostics.
use super::observations::FilmComponentObservation;
use std::collections::BTreeMap;

/// Signed reference skip that rewinds, compacts padding, or cannot fit the source-offset model.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WidthAdjustment {
    pub component: String,
    pub calibrated: bool,
    /// None denotes the legacy calibrated/stub override. Mobility and New-record
    /// tails are identified separately rather than reported as unported stubs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<WidthPurpose>,
    pub bit: i64,
    pub width: i64,
    /// For a compact positive skip, source bits retained as scalar fields. The
    /// remaining skipped bits are synthetic zero padding, not recorded data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retained_bits: Option<usize>,
    /// Checked address-sized projection of the reference wrapping target. None
    /// includes negative targets and positive targets outside usize; use
    /// `reference_end_bit` rather than treating this as missing reference data.
    pub end_bit: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WidthPurpose {
    RecordPrefix,
    MobilityExtra,
    NewRecordTail,
    NewRecordDefault,
}

/// A reference field was reached, but its raw width exceeds this reader's address
/// domain. This is a decoder limitation, not evidence of missing source bytes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WidthRefusal {
    pub field: String,
    pub bit: i64,
    pub raw_width: u64,
    pub maximum: u64,
}

/// A bounded operation refused before advancing the cursor. This is not a
/// fabricated zero-valued field and does not include reference padded reads.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReadRefusal {
    pub field: String,
    pub bit: i64,
    pub width: u64,
    pub source_bits: usize,
    pub operation: ReadOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReadOperation {
    /// The scalar cursor returned None (source bounds or bounded width/domain).
    Scalar,
    /// The reference grouped source guard failed before any member was read.
    GroupGuard,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct FilmReadDiagnostics {
    /// Reference NEW binding refusals, in read order; parsed records are retained.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub new_binding_refusals: Vec<crate::theater::film::NewBindingRefusal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub read_refusals: Vec<ReadRefusal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub width_refusals: Vec<WidthRefusal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub width_adjustments: Vec<WidthAdjustment>,
    pub component_widths: BTreeMap<String, BTreeMap<usize, u64>>,
    pub rejected_unbound: u64,
    pub rejected_other_view: u64,
    pub anticipated_bindings: BTreeMap<u32, u64>,
    /// Reference Observation.IndexAbsolus; -1 denotes the build's default region.
    pub absolute_indices: BTreeMap<i32, u64>,
    /// Ordered reference MobilityActionHook flags, including failed/speculative reads.
    /// Unlike movement-state capture, this historical hook is never suppressed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mobility_actions: Vec<[bool; 2]>,
    /// Number of component publications preceding each mobility publication.
    /// None with nonempty mobility_actions means ordering was not retained.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobility_offsets: Option<Vec<usize>>,
    /// Ordered publications from named component hooks. Position, movement and
    /// reference callbacks obey their independent reference capture policies.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub component_observations: Vec<FilmComponentObservation>,
}

impl FilmReadDiagnostics {
    pub(crate) fn publish_mobility(&mut self, flags: [bool; 2]) {
        if self.mobility_actions.is_empty() {
            self.mobility_offsets = Some(Vec::new());
        }
        if let Some(offsets) = &mut self.mobility_offsets {
            offsets.push(self.component_observations.len());
        }
        self.mobility_actions.push(flags);
    }

    fn merge_publications(&mut self, other: &Self) {
        // Validate shape/boundary in constant time. An internally unsorted vector
        // stays unsorted after concatenation and is rejected by ordered_publications.
        // Avoid rescanning or copying the accumulated publication history per merge.
        let known = |d: &Self| {
            d.mobility_actions.is_empty()
                || d.mobility_offsets.as_ref().is_some_and(|v| {
                    v.len() == d.mobility_actions.len()
                        && v.last()
                            .is_some_and(|&n| n <= d.component_observations.len())
                })
        };
        if known(self) && known(other) {
            if self.mobility_actions.is_empty() {
                self.mobility_offsets = Some(Vec::new());
            }
            if !other.mobility_actions.is_empty() {
                self.mobility_offsets.as_mut().unwrap().extend(
                    other
                        .mobility_offsets
                        .as_ref()
                        .unwrap()
                        .iter()
                        // Imported malformed offsets may overflow when shifted.
                        // Saturation keeps them out of bounds (and order unknown)
                        // without panicking or wrapping into a plausible offset.
                        .map(|n| n.saturating_add(self.component_observations.len())),
                );
            }
        } else {
            self.mobility_offsets = None;
        }
        self.mobility_actions
            .extend_from_slice(&other.mobility_actions);
        self.component_observations
            .extend_from_slice(&other.component_observations);
        if self.mobility_actions.is_empty() {
            self.mobility_offsets = None;
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.new_binding_refusals.is_empty()
            && self.read_refusals.is_empty()
            && self.width_refusals.is_empty()
            && self.width_adjustments.is_empty()
            && self.component_widths.is_empty()
            && self.absolute_indices.is_empty()
            && self.rejected_unbound == 0
            && self.rejected_other_view == 0
            && self.anticipated_bindings.is_empty()
            && self.mobility_actions.is_empty()
            && self.component_observations.is_empty()
    }
    pub(crate) fn merge(&mut self, other: &Self) {
        self.new_binding_refusals
            .extend_from_slice(&other.new_binding_refusals);
        self.read_refusals.extend_from_slice(&other.read_refusals);
        self.width_refusals.extend_from_slice(&other.width_refusals);
        self.width_adjustments
            .extend_from_slice(&other.width_adjustments);
        self.merge_publications(other);
        for (name, widths) in &other.component_widths {
            let target = self.component_widths.entry(name.clone()).or_default();
            for (&width, &count) in widths {
                *target.entry(width).or_default() += count;
            }
        }
        self.rejected_unbound += other.rejected_unbound;
        self.rejected_other_view += other.rejected_other_view;
        for (&ti, &count) in &other.anticipated_bindings {
            *self.anticipated_bindings.entry(ti).or_default() += count;
        }
        for (&index, &count) in &other.absolute_indices {
            *self.absolute_indices.entry(index).or_default() += count;
        }
    }
    pub(crate) fn absolute(&mut self, index: i32) {
        *self.absolute_indices.entry(index).or_default() += 1;
    }
}
