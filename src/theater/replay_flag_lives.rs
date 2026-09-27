//! Flag state timelines assembled from dated carries, returns and object births.
use super::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayFlagSpan {
    pub state: String,
    pub t0: i64,
    pub t1: i64,
    pub xuid: Option<String>,
    pub x: f32,
    pub y: f32,
    #[serde(
        default,
        rename = "returnProgress",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub return_progress: Vec<ReplayGaugePoint>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayFlagCarry {
    pub team: i64,
    pub spans: Vec<ReplayFlagSpan>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReplayFlagLifeCoverage {
    pub drops_withheld: usize,
    pub ambiguous_returns: usize,
    pub home_by_object: usize,
    pub ambiguous_homecomings: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayFlagLives {
    pub carries: Vec<ReplayFlagCarry>,
    pub coverage: ReplayFlagLifeCoverage,
}
struct Transition {
    frame: i64,
    state: String,
    xuid: Option<String>,
    x: f32,
    y: f32,
}
struct Event {
    at: i64,
    kind: u8,
    index: usize,
}
fn clamp(t: i64, frames: i64) -> i64 {
    if t < 0 {
        0
    } else if t >= frames {
        frames.wrapping_sub(1)
    } else {
        t
    }
}
fn spans(mut ts: Vec<Transition>, frames: i64) -> Vec<ReplayFlagSpan> {
    ts.sort_by_key(|t| t.frame);
    let mut out: Vec<ReplayFlagSpan> = Vec::new();
    for (i, t) in ts.iter().enumerate() {
        if t.state.is_empty() {
            continue;
        }
        let t0 = clamp(t.frame, frames);
        let t1 = ts.get(i + 1).map_or(frames.wrapping_sub(1), |n| {
            clamp(n.frame, frames).wrapping_sub(1)
        });
        if t1 < t0 {
            continue;
        }
        if let Some(last) = out
            .last_mut()
            .filter(|s| s.state == t.state && s.x == t.x && s.y == t.y && s.xuid == t.xuid)
        {
            last.t1 = t1;
            continue;
        }
        out.push(ReplayFlagSpan {
            state: t.state.clone(),
            t0,
            t1,
            xuid: t.xuid.clone(),
            x: t.x,
            y: t.y,
            return_progress: Vec::new(),
        });
    }
    out
}
pub fn assemble_replay_flag_lives(
    raw: &[ReplayFlagCarryRaw],
    spawns: &[ReplayFlagSpawn],
    returns: &[ReplayFlagHomecoming],
    homes: &[ReplayFlagHomecoming],
    clock: ReplayMatchClock,
) -> ReplayFlagLives {
    let mut out = ReplayFlagLives::default();
    if clock.frames <= 0 || raw.is_empty() {
        return out;
    }
    let n = spawns.len().max(1);
    let mut transitions: Vec<Vec<Transition>> = (0..n).map(|_| Vec::new()).collect();
    let mut states = vec!["home".to_string(); n];
    for (i, s) in spawns.iter().enumerate() {
        transitions[i].push(Transition {
            frame: 0,
            state: "home".into(),
            xuid: None,
            x: s.x,
            y: s.y,
        });
    }
    let mut events = Vec::new();
    for (i, r) in raw.iter().enumerate() {
        events.push(Event {
            at: r.t0,
            kind: 3,
            index: i,
        });
        if !r.closed {
            continue;
        }
        if !r.ends_home()
            && r.flag_index >= 0
            && raw
                .iter()
                .enumerate()
                .any(|(j, p)| j != i && p.flag_index == r.flag_index && p.t0 <= r.t1 && r.t1 < p.t1)
        {
            out.coverage.drops_withheld += 1;
            continue;
        }
        events.push(Event {
            at: r.t1,
            kind: 0,
            index: i,
        });
    }
    for (i, h) in returns.iter().enumerate() {
        events.push(Event {
            at: h.at,
            kind: 1,
            index: i,
        });
    }
    for (i, h) in homes.iter().enumerate() {
        events.push(Event {
            at: h.at,
            kind: 2,
            index: i,
        });
    }
    events.sort_by_key(|e| (e.at, e.kind));
    for e in events {
        if e.kind == 1 {
            let h = &returns[e.index];
            if states.get(h.flag as usize).is_some_and(|s| s == "home") {
                continue;
            }
            let dropped: Vec<_> = states
                .iter()
                .enumerate()
                .filter(|(_, s)| s.as_str() == "dropped")
                .map(|(i, _)| i)
                .collect();
            if dropped.len() != 1 || dropped[0] >= spawns.len() {
                out.coverage.ambiguous_returns += 1;
                continue;
            }
            let f = dropped[0];
            states[f] = "home".into();
            transitions[f].push(Transition {
                frame: clock.frame_of_match_ms(e.at),
                state: "home".into(),
                xuid: None,
                x: spawns[f].x,
                y: spawns[f].y,
            });
            continue;
        }
        if e.kind == 2 {
            let h = &homes[e.index];
            let f = h.flag as usize;
            if f >= spawns.len() || f >= states.len() || states[f] != "dropped" {
                continue;
            }
            let ambiguous = states.iter().enumerate().any(|(g, s)| {
                g != f
                    && s == "dropped"
                    && transitions[g]
                        .last()
                        .is_some_and(|p| flag_sq_distance(p.x, p.y, h.x, h.y) <= 2.25)
            });
            if ambiguous {
                out.coverage.ambiguous_homecomings += 1;
                continue;
            }
            states[f] = "home".into();
            out.coverage.home_by_object += 1;
            transitions[f].push(Transition {
                frame: clock.frame_of_match_ms(e.at),
                state: "home".into(),
                xuid: None,
                x: spawns[f].x,
                y: spawns[f].y,
            });
            continue;
        }
        let r = &raw[e.index];
        let f = r.flag_index as usize;
        if f >= n {
            continue;
        }
        let t = if e.kind == 3 {
            Transition {
                frame: clock.frame_of_match_ms(r.t0),
                state: if r.closed { "carried" } else { "carried_open" }.into(),
                xuid: Some(r.xuid.clone()),
                x: r.x0,
                y: r.y0,
            }
        } else {
            let frame = clock.frame_of_match_ms(r.t1).wrapping_add(1);
            if r.ends_home() {
                if let Some(s) = spawns.get(f) {
                    Transition {
                        frame,
                        state: "home".into(),
                        xuid: None,
                        x: s.x,
                        y: s.y,
                    }
                } else {
                    Transition {
                        frame,
                        state: String::new(),
                        xuid: None,
                        x: 0.,
                        y: 0.,
                    }
                }
            } else {
                Transition {
                    frame,
                    state: "dropped".into(),
                    xuid: None,
                    x: r.x1,
                    y: r.y1,
                }
            }
        };
        states[f] = t.state.clone();
        transitions[f].push(t);
    }
    for (f, ts) in transitions.into_iter().enumerate() {
        let spans = spans(ts, clock.frames);
        if !spans.is_empty() {
            out.carries.push(ReplayFlagCarry {
                team: spawns.get(f).map_or(-1, |s| s.team),
                spans,
            });
        }
    }
    out
}
