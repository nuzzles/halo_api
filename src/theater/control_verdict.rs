//! Native view-C closure verdict; raw entries remain on the original view.
use super::*;
use serde::{Deserialize, Serialize};

/// The reference's LectureVueC publication. Closure is a parser boundary check,
/// not proof that an inferred entry offset is a canonical recording boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeControlVerdict {
    pub reached: bool,
    pub closed: bool,
    /// None when the preceding view prevented reaching the control view.
    pub stop: Option<FrameViewStop>,
    /// Only closed views publish entries. Raw/partial reads remain in DecodedFrameView.
    pub entries: Vec<NativeControlEntry>,
}
impl NativeControlVerdict {
    pub fn from_view(payload: &[u8], view: Option<&DecodedFrameView>) -> Self {
        let Some(view) = view else {
            return Self {
                reached: false,
                closed: false,
                stop: None,
                entries: Vec::new(),
            };
        };
        let closed = view.stop == FrameViewStop::Complete
            && control_view_closes_packet(payload, view.end_bit);
        Self {
            reached: true,
            closed,
            stop: Some(view.stop.clone()),
            entries: if closed {
                view.control_entries.clone()
            } else {
                Vec::new()
            },
        }
    }
}
/// The native final-view boundary rule: zero to seven remaining bits, all zero.
/// Signed invalid or padded endpoints cannot close a packet.
pub fn control_view_closes_packet(payload: &[u8], end_bit: i64) -> bool {
    let Ok(end) = usize::try_from(end_bit) else {
        return false;
    };
    let Some(remaining) = (payload.len() * 8).checked_sub(end) else {
        return false;
    };
    remaining <= 7
        && (end..payload.len() * 8).all(|bit| payload[bit / 8] & (1 << (7 - bit % 8)) == 0)
}
impl NativeFilmObserver {
    pub(crate) fn publish_control_verdict(&self, payload: &[u8], view: Option<&DecodedFrameView>) {
        if self.has_hook(NativeHookKind::ControlView) {
            self.publish(NativeHookPublication::ControlView(
                &NativeControlVerdict::from_view(payload, view),
            ));
        }
    }
}
impl ProductionFrame {
    pub fn control_verdict(&self, payload: &[u8]) -> NativeControlVerdict {
        NativeControlVerdict::from_view(payload, self.controls.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::{
        io::Read,
        sync::{Arc, Mutex},
    };
    fn bytes(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }
    fn check(actual: &NativeControlVerdict, expected: &Value) {
        assert_eq!(actual.reached, expected["Atteinte"]);
        assert_eq!(actual.closed, expected["Fermee"]);
        let stop = match actual.stop.as_ref() {
            None | Some(FrameViewStop::Complete) => 0,
            Some(FrameViewStop::Truncated) => 1,
            Some(FrameViewStop::Unsupported { reason }) if reason == "secondary control block" => 3,
            Some(FrameViewStop::Unsupported { .. }) => 2,
            Some(FrameViewStop::RecordLimit) => 4,
        };
        assert_eq!(stop, expected["Arret"]);
        let entries = expected["Entrees"].as_array().cloned().unwrap_or_default();
        assert_eq!(actual.entries.len(), entries.len());
        for (a, e) in actual.entries.iter().zip(entries) {
            assert_eq!(a.index, e["Index"]);
        }
    }
    #[test]
    fn native_control_closure_d61443e() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/control-closure-d61443e-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Value = serde_json::from_slice(&raw).unwrap();
        for row in rows["direct"].as_array().unwrap() {
            let payload = bytes(row["hex"].as_str().unwrap());
            let view = decode_control_view_signed(&payload, row["start"].as_i64().unwrap());
            assert_eq!(view.end_bit, row["end"]);
            let verdict = NativeControlVerdict::from_view(&payload, Some(&view));
            check(&verdict, &row["verdict"]);
            assert_eq!(
                serde_json::from_value::<NativeControlVerdict>(
                    serde_json::to_value(&verdict).unwrap()
                )
                .unwrap(),
                verdict
            );
        }
        for row in rows["frames"].as_array().unwrap() {
            let payload = bytes(row["hex"].as_str().unwrap());
            let mut config = NativeFrameConfig::default();
            let observer = NativeFilmObserver::default();
            let calls = Arc::new(Mutex::new(Vec::new()));
            let capture = calls.clone();
            observer.set_hook(
                NativeHookKind::ControlView,
                Some(Arc::new(move |publication| {
                    if let NativeHookPublication::ControlView(v) = publication {
                        capture.lock().unwrap().push(v.clone());
                    }
                })),
            );
            config.context.observer = Some(observer);
            let frame = config
                .decode_production_views(
                    &payload,
                    2,
                    &FilmRegistry {
                        archetypes: vec![],
                        major_version: 41,
                        format_version: 27,
                        end_byte: 0,
                        truncated: false,
                    },
                    &mut FilmWorld::default(),
                )
                .unwrap();
            assert_eq!(frame.end_bit, row["end"]);
            assert_eq!(frame.views_completed, row["views"]);
            let captured = calls.clone();
            let calls = calls.lock().unwrap();
            let expected = row["calls"].as_array().unwrap();
            assert_eq!(calls.len(), expected.len());
            assert_eq!(calls.len(), 1);
            check(&calls[0], &expected[0]);
            assert_eq!(frame.control_verdict(&payload), calls[0]);
            drop(calls);
            // The inference-view API must publish the same single native verdict.
            let inferred = config
                .decode_inference_views(
                    &payload,
                    2,
                    3,
                    &FilmRegistry {
                        archetypes: vec![],
                        major_version: 41,
                        format_version: 27,
                        end_byte: 0,
                        truncated: false,
                    },
                    &mut FilmWorld::default(),
                )
                .unwrap();
            assert_eq!(inferred.end_bit, row["end"]);
            let calls = captured.lock().unwrap();
            assert_eq!(calls.len(), 2);
            check(&calls[1], &expected[0]);
            // Compare through the same method without modifying the retained raw view.
            check(
                &NativeControlVerdict::from_view(&payload, inferred.controls.as_ref()),
                &expected[0],
            );
        }
        assert!(!control_view_closes_packet(&[0], -1));
        assert!(!control_view_closes_packet(&[0], 9));
    }
}
