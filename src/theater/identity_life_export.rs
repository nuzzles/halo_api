//! Native named-life export on the match clock, retaining naming provenance.
use super::IdentityRegistryOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedIdentityLife {
    pub xuid: u64,
    pub start_ms: i64,
    pub end_ms: i64,
    pub cause: String,
    pub named_by: String,
}
impl IdentityRegistryOutput {
    /// Export only when the native death-naming gate is established. Anonymous
    /// lives are omitted. Ties retain original life order after sorting by
    /// start time, XUID, then end time. Naming provenance is not upgraded.
    pub fn named_lives(&self) -> Vec<NamedIdentityLife> {
        if self.owners.deaths_named == 0 {
            return Vec::new();
        }
        let offset = self.owners.clock.offset_ms;
        let mut lives: Vec<_> = self
            .owners
            .state
            .lives()
            .iter()
            .filter(|l| l.xuid != 0)
            .map(|l| NamedIdentityLife {
                xuid: l.xuid,
                start_ms: (l.from / 1000).wrapping_sub(offset),
                end_ms: (l.to / 1000).wrapping_sub(offset),
                cause: l.cause.clone(),
                named_by: l.named_by.clone(),
            })
            .collect();
        lives.sort_by_key(|l| (l.start_ms, l.xuid, l.end_ms));
        lives
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        lives: Vec<IdentityLife>,
        offset: i64,
        deaths_named: usize,
        expected: Vec<NamedIdentityLife>,
    }
    #[test]
    fn native_named_life_export() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/named-life-export-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 512);
        for (i, c) in rows.into_iter().enumerate() {
            let mut owners = IdentityOwnerOutput {
                state: ReplayIdentityState::from_lives(c.lives, &Default::default()),
                deaths_named: c.deaths_named,
                ..Default::default()
            };
            owners.clock.offset_ms = c.offset;
            let registry = IdentityRegistryOutput {
                owners,
                tables: compose_identity_tables(&Default::default(), &Default::default()),
                scoreboard: Default::default(),
            };
            let out = registry.named_lives();
            assert_eq!(out, c.expected, "named lives {i}");
            assert_eq!(
                serde_json::from_slice::<Vec<NamedIdentityLife>>(
                    &serde_json::to_vec(&out).unwrap()
                )
                .unwrap(),
                out
            );
        }
    }
}
