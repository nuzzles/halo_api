//! Weapon accuracy pairing and world-distance resolution from the native grammar.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const WEAPON_HIT_PAIR_WINDOW_US: u64 = 1_000_000;
pub const WEAPON_HIT_POS_TOLERANCE_US: u64 = 120_000;
pub const WEAPON_HIT_DISTANCE_EDGES: [f64; 6] = [2., 5., 10., 15., 25., 40.];
pub const WEAPON_HIT_SLOT_BASES: [i64; 15] = [
    0, 128, 256, 384, 448, 480, 500, 508, 510, 512, 514, 516, 520, 544, 576,
];
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WeaponShot {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub attacker: u64,
    #[serde(rename = "WeaponID")]
    pub weapon_id: u64,
    pub film_index: i64,
    pub has_pair: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WeaponDamage {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub victim_idx: i64,
    pub responsible_idx: i64,
    pub source: u64,
    pub has_source: bool,
    pub negative: bool,
    pub mag_clear: f64,
    pub mag_raw: u64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WeaponHitStats {
    pub film_index: i64,
    #[serde(rename = "WeaponID")]
    pub weapon_id: u64,
    pub shots_paired: i64,
    pub hits: i64,
    pub dist_buckets: [i64; 7],
}
/// Number of distance buckets in the native default weapon-hit histogram.
pub const fn weapon_hit_bucket_count() -> usize {
    WEAPON_HIT_DISTANCE_EDGES.len() + 1
}
pub fn weapon_hit_bucket(distance: f64) -> usize {
    WEAPON_HIT_DISTANCE_EDGES
        .iter()
        .position(|&edge| distance < edge)
        .unwrap_or(6)
}
fn nearest<T>(values: &[T], timestamp: u64, tolerance: u64, ts: impl Fn(&T) -> u64) -> Option<&T> {
    let i = values.partition_point(|v| ts(v) < timestamp);
    let mut best = None;
    let mut delta = u64::MAX;
    for j in [i.checked_sub(1), Some(i)].into_iter().flatten() {
        if let Some(v) = values.get(j) {
            let d = timestamp.abs_diff(ts(v));
            if d <= tolerance && d < delta {
                best = Some(v);
                delta = d;
            }
        }
    }
    best
}
pub type WeaponHitDistanceFn<'a> = dyn Fn(&WeaponDamage) -> Option<f64> + 'a;

/// Damage may pair with multiple shots. Healing and zero magnitude are retained,
/// matching the reference; unavailable distance does not discard a hit.
pub fn pair_weapon_hits(
    shots: &[WeaponShot],
    damages: &[WeaponDamage],
    window: u64,
    distance: Option<&WeaponHitDistanceFn<'_>>,
) -> Vec<WeaponHitStats> {
    let mut by_resp: BTreeMap<u64, Vec<WeaponDamage>> = BTreeMap::new();
    for d in damages {
        if d.responsible_idx >= 0 {
            by_resp
                .entry(d.responsible_idx as u64)
                .or_default()
                .push(d.clone());
        }
    }
    for values in by_resp.values_mut() {
        super::native_sort::sort_by(values, |a, b| a.timestamp_us.cmp(&b.timestamp_us));
    }
    let mut stats: BTreeMap<(i64, u64), WeaponHitStats> = BTreeMap::new();
    for s in shots.iter().filter(|s| s.has_pair) {
        let st = stats
            .entry((s.film_index, s.weapon_id))
            .or_insert_with(|| WeaponHitStats {
                film_index: s.film_index,
                weapon_id: s.weapon_id,
                ..Default::default()
            });
        st.shots_paired += 1;
        let values = by_resp
            .get(&s.attacker)
            .map(Vec::as_slice)
            .unwrap_or_default();
        if let Some(d) = nearest(values, s.timestamp_us, window, |d| d.timestamp_us) {
            st.hits += 1;
            if let Some(m) = distance.and_then(|f| f(d)) {
                st.dist_buckets[weapon_hit_bucket(m)] += 1;
            }
        }
    }
    let mut out: Vec<_> = stats.into_values().collect();
    out.sort_by_key(|s| (std::cmp::Reverse(s.shots_paired), s.film_index, s.weapon_id));
    out
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaponHitPositionSample {
    pub timestamp_us: u64,
    pub position: [f32; 3],
}
/// Tracks must be sorted by timestamp, as produced by the native biped scan.
pub type WeaponHitTracks = BTreeMap<u32, Vec<WeaponHitPositionSample>>;
pub fn weapon_hit_distance(
    tracks: &WeaponHitTracks,
    base: i64,
    damage: &WeaponDamage,
) -> Option<f64> {
    if damage.victim_idx < 0 || damage.responsible_idx < 0 {
        return None;
    }
    let sample = |idx| {
        nearest(
            tracks.get(&(base.wrapping_add(idx) as u32))?,
            damage.timestamp_us,
            WEAPON_HIT_POS_TOLERANCE_US,
            |p| p.timestamp_us,
        )
    };
    let victim = sample(damage.victim_idx)?;
    let attacker = sample(damage.responsible_idx)?;
    let d: [f64; 3] = std::array::from_fn(|i| f64::from(victim.position[i] - attacker.position[i]));
    Some((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
}
/// Native argmax retains the first candidate on ties, including empty inputs.
pub fn resolve_hit_distance_base(damages: &[WeaponDamage], tracks: &WeaponHitTracks) -> i64 {
    let mut best = (512, -1_i64);
    for base in WEAPON_HIT_SLOT_BASES.into_iter().filter(|b| *b >= 400) {
        let score = damages
            .iter()
            .filter(|d| weapon_hit_distance(tracks, base, d).is_some())
            .count() as i64;
        if score > best.1 {
            best = (base, score);
        }
    }
    best.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_weapon_hit_pairing_and_distance() {
        #[derive(Deserialize)]
        struct Case {
            shots: Vec<WeaponShot>,
            damages: Vec<WeaponDamage>,
            tracks: WeaponHitTracks,
            base: i64,
            window: u64,
            stats: Vec<WeaponHitStats>,
            without: Vec<WeaponHitStats>,
            distance_calls: Vec<WeaponDamage>,
            bucket_count: usize,
        }
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/weapon-hits-v41.json.zlib")[..])
            .read_to_end(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&json).unwrap();
        assert_eq!(cases.len(), 1024);
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                resolve_hit_distance_base(&c.damages, &c.tracks),
                c.base,
                "base case {i}"
            );
            assert_eq!(
                weapon_hit_bucket_count(),
                c.bucket_count,
                "bucket count {i}"
            );
            let calls = std::cell::RefCell::new(Vec::new());
            let distance = |d: &WeaponDamage| {
                calls.borrow_mut().push(d.clone());
                weapon_hit_distance(&c.tracks, c.base, d)
            };
            assert_eq!(
                pair_weapon_hits(&c.shots, &c.damages, c.window, Some(&distance)),
                c.stats,
                "case {i}"
            );
            assert_eq!(calls.into_inner(), c.distance_calls, "distance calls {i}");
            assert_eq!(
                pair_weapon_hits(&c.shots, &c.damages, c.window, None),
                c.without,
                "without case {i}"
            );
        }
        assert_eq!(resolve_hit_distance_base(&[], &BTreeMap::new()), 448);
        assert_eq!(weapon_hit_bucket(f64::NAN), 6);
        for (i, e) in WEAPON_HIT_DISTANCE_EDGES.into_iter().enumerate() {
            assert_eq!(weapon_hit_bucket(e), i + 1);
        }
    }
}
