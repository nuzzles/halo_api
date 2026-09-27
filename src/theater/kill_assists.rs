//! Native assist attachment after kill-feed and roster resolution.
use super::KillEventFields;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillPacketIdentity {
    pub chunk: i64,
    pub packet: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillAssist {
    pub name: String,
    pub index: i32,
    pub rejected: String,
    pub known: bool,
    /// Distinct additional assistants among attached event records only.
    pub extra: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillDamageShare {
    pub pct: u32,
    pub known: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillAssistTarget {
    pub time_ms: i64,
    pub victim: String,
    /// None represents absent kill-feed credit.
    pub killer: Option<String>,
    pub packet: Option<KillPacketIdentity>,
    pub assist: KillAssist,
    pub killer_damage: KillDamageShare,
    pub assist_damage: KillDamageShare,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillAssistEvent {
    pub time_ms: i64,
    pub packet: KillPacketIdentity,
    pub fields: KillEventFields,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillAssistStats {
    pub kill_events: usize,
    pub attached: usize,
    pub multi: usize,
    pub named: usize,
    pub no_assist: usize,
    pub rejected_self: usize,
    pub rejected_victim: usize,
    pub rejected_roster: usize,
    pub flag_set: usize,
    pub assist_multi: usize,
    pub assist_extra_total: usize,
    pub assist_field_disagree: usize,
    pub killer_pct_over100: usize,
    pub assist_pct_over100: usize,
    pub window_fallback: usize,
    pub gate15: bool,
}

/// Attach in native input order. Packet identity takes precedence over the inclusive
/// 2,500ms fallback window, and only the selected event is consumed.
/// `names` is the already-resolved index-to-name roster; absent indices resolve to `?`.
/// Publication still requires the kill-source health and bijection gates.
pub fn attach_kill_assists(
    kills: &mut [KillAssistTarget],
    events: &[KillAssistEvent],
    names: &[String],
    gate15: bool,
) -> KillAssistStats {
    let name = |index: i32| -> &str {
        usize::try_from(index)
            .ok()
            .and_then(|i| names.get(i))
            .map(String::as_str)
            .unwrap_or("?")
    };
    let mut stats = KillAssistStats {
        kill_events: events.len(),
        gate15,
        ..Default::default()
    };
    let mut used = vec![false; events.len()];
    for kill in kills {
        kill.assist.index = -1;
        let matches = |j: usize, r: &KillAssistEvent| {
            !used[j]
                && name(r.fields.victim) == kill.victim
                && kill
                    .killer
                    .as_ref()
                    .is_none_or(|killer| name(r.fields.killer) == killer)
        };
        let mut hits: Vec<usize> = events
            .iter()
            .enumerate()
            .filter(|(j, r)| matches(*j, r) && kill.packet == Some(r.packet))
            .map(|(j, _)| j)
            .collect();
        if hits.is_empty() {
            hits = events
                .iter()
                .enumerate()
                .filter(|(j, r)| matches(*j, r) && r.time_ms.abs_diff(kill.time_ms) <= 2500)
                .map(|(j, _)| j)
                .collect();
            if !hits.is_empty() {
                stats.window_fallback += 1;
            }
        }
        let Some(&first) = hits.first() else {
            continue;
        };
        let mut pick = first;
        if hits.len() > 1 {
            stats.multi += 1;
            let mut present = false;
            let mut absent = false;
            let mut distinct = BTreeSet::new();
            for &j in &hits {
                let f = &events[j].fields;
                if f.assist >= 0 {
                    if !present {
                        pick = j;
                    }
                    present = true;
                    // Native distinct count excludes the killer, but does not
                    // exclude victim/out-of-roster fields before publication.
                    if f.assist != f.killer {
                        distinct.insert(f.assist);
                    }
                } else {
                    absent = true;
                }
            }
            if present && absent {
                stats.assist_field_disagree += 1;
            }
            if distinct.len() > 1 {
                kill.assist.extra = distinct.len() - 1;
                stats.assist_multi += 1;
                stats.assist_extra_total += distinct.len() - 1;
            }
        }
        used[pick] = true;
        stats.attached += 1;
        let f = &events[pick].fields;
        kill.assist.known = true;
        kill.assist.index = f.assist;
        kill.killer_damage = KillDamageShare {
            pct: f.killer_pct,
            known: true,
        };
        if f.killer_pct > 100 {
            stats.killer_pct_over100 += 1;
        }
        if f.assist >= 0 {
            kill.assist_damage = KillDamageShare {
                pct: f.assist_pct,
                known: true,
            };
            if f.assist_pct > 100 {
                stats.assist_pct_over100 += 1;
            }
        }
        if f.assist < 0 {
            stats.no_assist += 1;
        } else if f.assist == f.killer {
            kill.assist.rejected = "assistant==tueur".into();
            stats.rejected_self += 1;
        } else if f.assist == f.victim {
            kill.assist.rejected = "assistant==victime".into();
            stats.rejected_victim += 1;
        } else if name(f.assist) == "?" {
            kill.assist.rejected = "hors-roster".into();
            stats.rejected_roster += 1;
        } else {
            kill.assist.name = name(f.assist).into();
            stats.named += 1;
        }
        if f.flag != 0 {
            stats.flag_set += 1;
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Row {
        names: Vec<String>,
        events: Vec<KillAssistEvent>,
        before: Vec<KillAssistTarget>,
        after: Vec<KillAssistTarget>,
        stats: KillAssistStats,
        gate: bool,
    }
    #[test]
    fn native_kill_assists_oracle() {
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/kill-assists-v41.json.zlib")[..])
            .read_to_end(&mut json)
            .unwrap();
        let rows: Vec<Row> = serde_json::from_slice(&json).unwrap();
        assert_eq!(rows.len(), 1024);
        let mut multi = 0;
        let mut fallback = 0;
        let mut disagreements = 0;
        for (i, mut row) in rows.into_iter().enumerate() {
            let stats = attach_kill_assists(&mut row.before, &row.events, &row.names, row.gate);
            assert_eq!(stats, row.stats, "stats {i}");
            assert_eq!(row.before, row.after, "kills {i}");
            multi += stats.multi;
            fallback += stats.window_fallback;
            disagreements += stats.assist_field_disagree;
        }
        assert!(multi > 0 && fallback > 0 && disagreements > 0);
    }
}
