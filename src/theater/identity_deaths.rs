//! Native death/life matching at an explicit match-to-film clock offset.
use super::{IdentityDeath, IdentityLife, native_sort};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityDeathPair {
    pub life: usize,
    pub death: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IdentityDeathVerification {
    pub matched: usize,
    pub concordant: usize,
    pub discordant: usize,
    pub named_by_bridge: usize,
}
/// Greedy one-to-one matching within 150ms, by distance then life index. Equal
/// distance/life keys preserve Go's unstable sort behavior, including death ties.
pub fn match_identity_deaths(
    lives: &[IdentityLife],
    deaths: &[IdentityDeath],
    offset_ms: i64,
) -> Vec<IdentityDeathPair> {
    struct Candidate {
        life: usize,
        death: usize,
        delta: i64,
    }
    let mut candidates = Vec::new();
    for (di, d) in deaths.iter().enumerate() {
        let target = d.time_ms.wrapping_add(offset_ms);
        for (li, l) in lives.iter().enumerate() {
            let delta = (l.to / 1000).wrapping_sub(target).wrapping_abs();
            if delta <= 150 {
                candidates.push(Candidate {
                    life: li,
                    death: di,
                    delta,
                });
            }
        }
    }
    native_sort::sort_by(&mut candidates, |a, b| {
        a.delta.cmp(&b.delta).then(a.life.cmp(&b.life))
    });
    let mut used_life = vec![false; lives.len()];
    let mut used_death = vec![false; deaths.len()];
    let mut out = Vec::new();
    for c in candidates {
        if used_life[c.life] || used_death[c.death] {
            continue;
        }
        used_life[c.life] = true;
        used_death[c.death] = true;
        out.push(IdentityDeathPair {
            life: c.life,
            death: c.death,
        });
    }
    out.sort_unstable_by_key(|p| p.life);
    out
}
/// Pairs must come from match_identity_deaths for these same inputs.
pub fn mark_identity_deaths(lives: &mut [IdentityLife], pairs: &[IdentityDeathPair]) {
    for p in pairs {
        lives[p.life].cause = "death".into();
    }
}
/// Only direct creation identities are compared; deductions are not witnesses.
/// Pairs must come from match_identity_deaths for these same inputs.
pub fn verify_identity_deaths(
    lives: &[IdentityLife],
    deaths: &[IdentityDeath],
    pairs: &[IdentityDeathPair],
) -> IdentityDeathVerification {
    let mut out = IdentityDeathVerification {
        matched: pairs.len(),
        ..Default::default()
    };
    for p in pairs {
        let l = &lives[p.life];
        if l.xuid == 0
            || !matches!(
                l.named_by.as_str(),
                "biped_creation" | "biped_creation_propagee"
            )
        {
            continue;
        }
        if l.xuid == deaths[p.death].xuid {
            out.concordant += 1;
        } else {
            out.discordant += 1;
        }
    }
    out
}
/// Legacy naming fallback for a pipeline with no direct creation readings.
/// The caller must enforce that gate. Pairs must reference these same inputs.
pub fn name_identity_lives_by_deaths(
    lives: &mut [IdentityLife],
    deaths: &[IdentityDeath],
    pairs: &[IdentityDeathPair],
) -> usize {
    let mut named = 0;
    for p in pairs {
        if lives[p.life].xuid != 0 {
            continue;
        }
        lives[p.life].xuid = deaths[p.death].xuid;
        lives[p.life].named_by = "death".into();
        named += 1;
    }
    named
}
