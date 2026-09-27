//! Named native statistic addresses and their series policies.
use super::{
    StatborgCounterKey, StatborgRecord, StatborgRoundSeries, StatborgScorePoint, StatborgSide,
    StatborgValue,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Native player capacity of the statborg entity layout (slots 10 through 24).
pub const STATBORG_PLAYER_SLOTS: i64 = 8;

/// Test seat capacity only, matching the native helper. This is not input
/// validation: even a negative caller-supplied count passes the native test.
pub const fn roster_fits_statborg(seats: i64) -> bool {
    seats <= STATBORG_PLAYER_SLOTS
}

/// The native StatComponent contract. Unitary applies the action-counter step
/// bound; cadence counters such as personal score must not use that bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatborgComponent {
    #[serde(rename = "Comp")]
    pub component: i64,
    pub side_b: bool,
    pub strict: bool,
    pub unitary: bool,
}
impl StatborgComponent {
    pub const fn key(self) -> StatborgCounterKey {
        StatborgCounterKey {
            component: self.component,
            side: if self.side_b {
                StatborgSide::B
            } else {
                StatborgSide::A
            },
        }
    }
    pub fn series_by_round(self, records: &[StatborgRecord], teams: bool) -> StatborgRoundSeries {
        super::statborg_series_by_round(records, self.key(), teams, self.strict, self.unitary)
    }
    pub fn series_total_with_diagnostics(
        self,
        records: &[StatborgRecord],
        teams: bool,
    ) -> (super::StatborgSlotSeries, Vec<super::StatborgDiagnostic>) {
        super::statborg_series_total_with_diagnostics(
            records,
            self.key(),
            teams,
            self.strict,
            self.unitary,
        )
    }
    pub fn series_total(
        self,
        records: &[StatborgRecord],
        teams: bool,
    ) -> BTreeMap<i64, Vec<StatborgScorePoint>> {
        super::statborg_series_total(records, self.key(), teams, self.strict, self.unitary)
    }
}
pub const MODE_SCORE_COMPONENT: StatborgComponent = StatborgComponent {
    component: 0,
    side_b: false,
    strict: true,
    unitary: false,
};
pub const PERSONAL_SCORE_COMPONENT: StatborgComponent = StatborgComponent {
    component: 1,
    side_b: true,
    strict: false,
    unitary: false,
};
pub const KILLS_COMPONENT: StatborgComponent = StatborgComponent {
    component: 2,
    side_b: false,
    strict: false,
    unitary: true,
};
pub const DEATHS_COMPONENT: StatborgComponent = StatborgComponent {
    component: 2,
    side_b: true,
    strict: false,
    unitary: true,
};
pub const ASSISTS_COMPONENT: StatborgComponent = StatborgComponent {
    component: 3,
    side_b: false,
    strict: false,
    unitary: true,
};
pub const SKULL_TICKS_COMPONENT: StatborgComponent = StatborgComponent {
    component: 0,
    side_b: false,
    strict: false,
    unitary: false,
};
pub const SKULL_GRABS_COMPONENT: StatborgComponent = StatborgComponent {
    component: 21,
    side_b: true,
    strict: false,
    unitary: false,
};

/// Both channels must be in the native mode-score domain, even when only one
/// channel is requested. Presence flags for other channels do not affect this guard.
pub fn statborg_mode_score_in_domain(value: &StatborgValue) -> bool {
    (0..=250).contains(&value.a) && (0..=250).contains(&value.b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_score_components_and_queries() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/stat-components-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let fixture: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(STATBORG_PLAYER_SLOTS, fixture["player_slots"]);
        for case in fixture["roster"].as_array().unwrap() {
            assert_eq!(
                roster_fits_statborg(case["seats"].as_i64().unwrap()),
                case["fits"]
            );
        }
        let catalog: BTreeMap<String, StatborgComponent> =
            serde_json::from_value(fixture["catalog"].clone()).unwrap();
        let expected: BTreeMap<String, _> = [
            ("mode", MODE_SCORE_COMPONENT),
            ("personal", PERSONAL_SCORE_COMPONENT),
            ("kills", KILLS_COMPONENT),
            ("deaths", DEATHS_COMPONENT),
            ("assists", ASSISTS_COMPONENT),
            ("skull_ticks", SKULL_TICKS_COMPONENT),
            ("skull_grabs", SKULL_GRABS_COMPONENT),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v))
        .collect();
        assert_eq!(catalog, expected);
        let rows = fixture["cases"].as_array().unwrap();
        assert_eq!(rows.len(), 128);
        let mut queries = 0;
        let mut points = 0;
        for (i, row) in rows.iter().enumerate() {
            let records: Vec<StatborgRecord> =
                serde_json::from_value(row["records"].clone()).unwrap();
            for q in row["queries"].as_array().unwrap() {
                let c: StatborgComponent = serde_json::from_value(q["component"].clone()).unwrap();
                assert_eq!(serde_json::to_value(c).unwrap(), q["component"]);
                if q["name"] != "custom" {
                    assert_eq!(c, catalog[q["name"].as_str().unwrap()]);
                }
                let teams = q["teams"].as_bool().unwrap();
                let rounds = c.series_by_round(&records, teams);
                let total = c.series_total(&records, teams);
                assert_eq!(
                    rounds,
                    serde_json::from_value::<StatborgRoundSeries>(q["rounds"].clone()).unwrap(),
                    "rounds {i}"
                );
                assert_eq!(
                    total,
                    serde_json::from_value::<BTreeMap<i64, Vec<StatborgScorePoint>>>(
                        q["total"].clone()
                    )
                    .unwrap(),
                    "total {i}"
                );
                points += total.values().map(Vec::len).sum::<usize>();
                queries += 1;
            }
        }
        assert_eq!(queries, 1920);
        assert!(points > 1000);
        assert_eq!(fixture["domain"].as_array().unwrap().len(), 64);
        for row in fixture["domain"].as_array().unwrap() {
            let value: StatborgValue = serde_json::from_value(row["value"].clone()).unwrap();
            assert_eq!(
                statborg_mode_score_in_domain(&value),
                row["ok"].as_bool().unwrap()
            );
        }
        eprintln!("{points} total-series points compared");
    }
}
