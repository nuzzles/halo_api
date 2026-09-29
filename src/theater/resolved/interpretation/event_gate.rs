//! Runtime event-layout configuration, with the reference selection evidence.
use super::{bits::Bits, scan_kill_event_chains};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventGate15Policy {
    /// Reference pickGate15: compare localized candidate counts; ties select false.
    #[default]
    CandidateCountInference,
    /// Caller-supplied grammar configuration, not a recorded film bit.
    Explicit(bool),
    /// Keep the runtime setting unknown and stop at code-15 bodies.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventGate15Selection {
    pub policy: EventGate15Policy,
    pub selected: Option<bool>,
    /// Candidate counts for false and true. Not action counts or confidence.
    /// Some([0, 0]) explicitly retains the reference no-evidence tie fallback.
    pub localized_candidate_counts: Option<[usize; 2]>,
    pub eligible_packets: usize,
}

impl Default for EventGate15Selection {
    fn default() -> Self {
        Self {
            policy: EventGate15Policy::Unknown,
            selected: None,
            localized_candidate_counts: None,
            eligible_packets: 0,
        }
    }
}

/// Reference per-film grammar inference without timestamps, roster resolution or
/// published outcomes. Input contains only type-0 payloads. Empty event lists
/// and payloads shorter than 64 bytes do not contribute, exactly as pickGate15.
/// Explicit/unknown policies do not scan the payloads. The inferred setting
/// must not be represented as recorded data or an independently proven value.
pub(crate) fn select_event_gate15<'a>(
    payloads: impl IntoIterator<Item = &'a [u8]>,
    policy: EventGate15Policy,
) -> EventGate15Selection {
    let mut out = EventGate15Selection {
        policy,
        ..Default::default()
    };
    match policy {
        EventGate15Policy::Unknown => return out,
        EventGate15Policy::Explicit(value) => {
            out.selected = Some(value);
            return out;
        }
        EventGate15Policy::CandidateCountInference => {}
    }
    let mut scores = [0; 2];
    for payload in payloads {
        if payload.len() < 64 || Bits(payload).read(1, 1) != Some(1) {
            continue;
        }
        out.eligible_packets += 1;
        for (gate, score) in scores.iter_mut().enumerate() {
            *score += scan_kill_event_chains(payload, gate == 1).len();
        }
    }
    out.selected = Some(scores[1] > scores[0]);
    out.localized_candidate_counts = Some(scores);
    out
}
