//! Native damage-tag to kill-feed sprite resolution. No assets or network needed.
use super::{KillDamageLabel, pinned_kill_damage_catalog};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KillIcon {
    pub sprite: String,
    pub weapon_key: String,
    pub genre: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KillIconRule {
    pub genre: String,
    pub key: String,
    pub sprite: String,
    pub weapon_key: String,
    pub justification: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KillIconProvenance {
    pub date: String,
    pub rule_count: usize,
    pub iconed_tags: usize,
    pub announced_nb: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillIconCatalog {
    pub rules: Vec<KillIconRule>,
    pub by_tag: BTreeMap<u32, KillIcon>,
    pub provenance: KillIconProvenance,
}
impl KillIconCatalog {
    pub fn lookup(&self, tag: u32) -> Option<&KillIcon> {
        self.by_tag.get(&tag)
    }
}

/// Read the native five-column TSV, preserving rule order and header provenance.
pub fn parse_kill_icon_rules(raw: &str) -> Result<(Vec<KillIconRule>, KillIconProvenance), String> {
    let mut rules = Vec::new();
    let mut provenance = KillIconProvenance::default();
    for (line_no, line) in raw.split('\n').enumerate() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with('#') {
            let mut date = "";
            let mut count = 0;
            for word in line.split_whitespace() {
                if let Some(d) = word.strip_prefix("date=") {
                    date = d;
                }
                if let Some(c) = word
                    .strip_prefix("regles=")
                    .and_then(|s| s.parse::<i64>().ok())
                {
                    count = c;
                }
            }
            if !date.is_empty() {
                provenance.date = date.into();
            }
            if count != 0 {
                provenance.announced_nb = count;
            }
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err(format!("line {}: expected five columns", line_no + 1));
        }
        if fields[0] == "genre" {
            continue;
        }
        if !matches!(fields[0], "NOM" | "GGGL" | "PORTEUR" | "BANQUE" | "CLASSE")
            || fields[1].is_empty()
            || fields[2].is_empty()
            || fields[4].is_empty()
            || (fields[0] == "PORTEUR"
                && (fields[1].len() != 8
                    || !fields[1]
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))))
        {
            return Err(format!("line {}: invalid icon rule", line_no + 1));
        }
        rules.push(KillIconRule {
            genre: fields[0].into(),
            key: fields[1].into(),
            sprite: fields[2].into(),
            weapon_key: fields[3].into(),
            justification: fields[4].into(),
        });
    }
    Ok((rules, provenance))
}

// Exact ASCII capture patterns used by the native RE2 expressions. Restricting
// digits to ASCII matters: Rust char::is_numeric would accept additional text.
fn captures(detail: &str, kind: u8) -> Vec<(&str, bool)> {
    let prefix = match kind {
        0 => "gggl entree ",
        1 => "vehi ",
        _ => "sb_",
    };
    let mut out = Vec::new();
    let mut end = 0;
    for (at, _) in detail.match_indices(prefix) {
        if at < end {
            continue;
        }
        let base = at + prefix.len();
        let rest = &detail[base..];
        let bytes = rest.as_bytes();
        let count = |from: usize, pred: fn(&u8) -> bool| {
            bytes[from..].iter().take_while(|b| pred(b)).count()
        };
        let (start, len, multiple, consumed) = match kind {
            0 => {
                let n = count(0, u8::is_ascii_digit);
                if n == 0 || bytes.get(n) != Some(&b'/') {
                    continue;
                }
                (0, n, false, n + 1)
            }
            1 => {
                if bytes.len() < 8
                    || !bytes[..8]
                        .iter()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b))
                {
                    continue;
                }
                let n = if bytes.get(8..10) == Some(b" +") {
                    count(10, u8::is_ascii_digit)
                } else {
                    0
                };
                (0, 8, n > 0, if n > 0 { 10 + n } else { 8 })
            }
            _ => {
                let n = count(0, u8::is_ascii_digit);
                if n == 0 || bytes.get(n) != Some(&b'_') {
                    continue;
                }
                let start = n + 1;
                let len = if kind == 2 {
                    count(start, |b| {
                        b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_'
                    })
                } else {
                    let a = count(start, u8::is_ascii_lowercase);
                    if a == 0 || bytes.get(start + a) != Some(&b'_') {
                        continue;
                    }
                    let mid = start + a + 1;
                    let b = count(mid, u8::is_ascii_lowercase);
                    if b == 0 || bytes.get(mid + b) != Some(&b'_') {
                        continue;
                    }
                    let last = mid + b + 1;
                    let c = count(last, |b| b.is_ascii_lowercase() || b.is_ascii_digit());
                    if c == 0 {
                        continue;
                    }
                    last + c - start
                };
                if len == 0 {
                    continue;
                }
                (start, len, false, start + len)
            }
        };
        out.push((&rest[start..start + len], multiple));
        end = base + consumed;
    }
    out
}
fn unique(detail: &str, kind: u8) -> Option<&str> {
    let values = captures(detail, kind);
    let first = values.first()?.0;
    values
        .iter()
        .all(|(s, multi)| *s == first && !multi)
        .then_some(first)
}

/// Resolve custom labels with native precedence and last-rule-wins indexing.
pub fn resolve_kill_icons(
    rules: &[KillIconRule],
    labels: &[KillDamageLabel],
) -> BTreeMap<u32, KillIcon> {
    let index: BTreeMap<_, _> = rules
        .iter()
        .map(|r| ((r.genre.as_str(), r.key.as_str()), r))
        .collect();
    let mut out = BTreeMap::new();
    for label in labels.iter().filter(|l| l.publishable()) {
        let candidates = [
            (
                "NOM",
                (!label.name.is_empty()).then_some(label.name.as_str()),
            ),
            ("GGGL", captures(&label.detail, 0).first().map(|v| v.0)),
            ("PORTEUR", unique(&label.detail, 1)),
            ("BANQUE", unique(&label.detail, 2)),
            ("BANQUE", unique(&label.detail, 3)),
            ("CLASSE", Some(label.class.as_str())),
        ];
        if let Some(rule) = candidates
            .iter()
            .find_map(|(genre, key)| index.get(&(*genre, (*key)?)))
        {
            out.insert(
                label.tag,
                KillIcon {
                    sprite: rule.sprite.clone(),
                    weapon_key: rule.weapon_key.clone(),
                    genre: rule.genre.clone(),
                },
            );
        }
    }
    out
}
pub fn pinned_kill_icon_catalog() -> &'static KillIconCatalog {
    static CATALOG: OnceLock<KillIconCatalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let (rules, mut provenance) =
            parse_kill_icon_rules(include_str!("reference/kill-icon-rules-v75.tsv"))
                .expect("pinned native icon rules");
        let labels: Vec<_> = pinned_kill_damage_catalog()
            .labels
            .values()
            .cloned()
            .collect();
        let by_tag = resolve_kill_icons(&rules, &labels);
        provenance.rule_count = rules.len();
        provenance.iconed_tags = by_tag.len();
        KillIconCatalog {
            rules,
            by_tag,
            provenance,
        }
    })
}
/// Only for unclaimed deaths whose dead-state names the victim; never ordinary kills.
pub fn neutral_death_icon(tag: u32) -> Option<(String, KillIcon)> {
    let (kind, sprite) = neutral_death_sprite(tag)?;
    Some((
        kind.into(),
        KillIcon {
            sprite: sprite.into(),
            weapon_key: String::new(),
            genre: "CLASSE".into(),
        },
    ))
}
pub(crate) fn neutral_death_sprite(tag: u32) -> Option<(&'static str, &'static str)> {
    let label = pinned_kill_damage_catalog().lookup(tag)?;
    match label.class.as_str() {
        "INCONNU" => None,
        "DEGAT_GLOBAL" => Some(("environment", "killfeed-55")),
        _ => Some(("suicide", "killfeed-61")),
    }
}

/// Native neutral sprite table. Returned by value so callers may modify their copy.
pub fn neutral_death_sprites() -> BTreeMap<String, String> {
    [
        ("environment".into(), "killfeed-55".into()),
        ("suicide".into(), "killfeed-61".into()),
    ]
    .into()
}

/// Aligned with attribution.kills and attribution.unclaimed, respectively.
/// Icons do not bypass the kill-source publication/health gate.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmKillIcons {
    pub kills: Vec<Option<KillIcon>>,
    pub unclaimed: Vec<Option<(String, KillIcon)>>,
}
impl FilmKillIcons {
    pub fn from_attribution(attribution: &super::KillHybridResult) -> Self {
        Self {
            kills: attribution
                .kills
                .iter()
                .map(|k| pinned_kill_icon_catalog().lookup(k.source.tag).cloned())
                .collect(),
            unclaimed: attribution
                .unclaimed
                .iter()
                .map(|d| neutral_death_icon(d.source.tag))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Custom {
        rules: Vec<KillIconRule>,
        labels: Vec<KillDamageLabel>,
        output: BTreeMap<u32, KillIcon>,
    }
    #[derive(Deserialize)]
    struct Parse {
        raw: String,
        rules: Option<Vec<KillIconRule>>,
        date: String,
        count: i64,
        error: bool,
    }
    #[derive(Deserialize)]
    struct Oracle {
        rules: Vec<KillIconRule>,
        provenance: KillIconProvenance,
        labels: Vec<KillDamageLabel>,
        resolved: BTreeMap<u32, KillIcon>,
        tags: Vec<u32>,
        neutral: BTreeMap<u32, (String, KillIcon)>,
        sprites: BTreeMap<String, String>,
        custom: Vec<Custom>,
        parses: Vec<Parse>,
    }
    #[test]
    fn native_kill_icons_catalog_and_resolution() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/kill-icons-v41.json.zlib")[..])
            .read_to_end(&mut bytes)
            .unwrap();
        let oracle: Oracle = serde_json::from_slice(&bytes).unwrap();
        let catalog = pinned_kill_icon_catalog();
        assert_eq!(catalog.rules, oracle.rules);
        assert_eq!(catalog.provenance, oracle.provenance);
        assert_eq!(catalog.by_tag, oracle.resolved);
        assert_eq!(
            catalog.by_tag.keys().copied().collect::<Vec<_>>(),
            oracle.tags
        );
        assert_eq!(
            resolve_kill_icons(&oracle.rules, &oracle.labels),
            oracle.resolved
        );
        assert_eq!(catalog.by_tag.len(), 193);
        use crate::theater::{
            AttributedFilmKill, KillHybridResult, KillReadPath, KillSourceProvenance,
            UnclaimedFilmDeath, kill_source_truth,
        };
        let mut attribution = KillHybridResult::default();
        for label in &oracle.labels {
            let read = KillSourceProvenance {
                path: KillReadPath::Walk,
                origin: String::new(),
                multiplicity: 1,
            };
            attribution.kills.push(AttributedFilmKill {
                time_ms: 1,
                victim: "victim".into(),
                killer: "killer".into(),
                feed_present: true,
                source: kill_source_truth(label.tag, 0),
                diverges: false,
                read: read.clone(),
                packet: None,
                assist: Default::default(),
                killer_damage: Default::default(),
                assist_damage: Default::default(),
            });
            attribution.unclaimed.push(UnclaimedFilmDeath {
                time_ms: 1,
                victim: "victim".into(),
                victim_xuid: 1,
                source: kill_source_truth(label.tag, 0),
                read,
            });
        }
        let icons = FilmKillIcons::from_attribution(&attribution);
        for (i, label) in oracle.labels.iter().enumerate() {
            assert_eq!(icons.kills[i].as_ref(), oracle.resolved.get(&label.tag));
            assert_eq!(icons.unclaimed[i].as_ref(), oracle.neutral.get(&label.tag));
        }
        assert_eq!(
            serde_json::from_value::<FilmKillIcons>(serde_json::to_value(&icons).unwrap()).unwrap(),
            icons
        );

        let mut neutral = BTreeMap::new();
        for label in &oracle.labels {
            if let Some(icon) = neutral_death_icon(label.tag) {
                neutral.insert(label.tag, icon);
            }
        }
        assert_eq!(neutral, oracle.neutral);
        assert_eq!(neutral.len(), 265);
        assert_eq!(neutral_death_sprites(), oracle.sprites);
        assert!(catalog.lookup(0).is_none());
        assert!(neutral_death_icon(0).is_none());
        assert_eq!(oracle.custom.len(), 1024);
        for (i, c) in oracle.custom.into_iter().enumerate() {
            assert_eq!(
                resolve_kill_icons(&c.rules, &c.labels),
                c.output,
                "custom {i}"
            );
        }
        for (i, c) in oracle.parses.into_iter().enumerate() {
            let actual = parse_kill_icon_rules(&c.raw);
            assert_eq!(actual.is_err(), c.error, "parse {i}");
            if let Ok((rules, provenance)) = actual {
                assert_eq!(rules, c.rules.unwrap_or_default(), "parse rules {i}");
                assert_eq!(provenance.date, c.date, "date {i}");
                assert_eq!(provenance.announced_nb, c.count, "count {i}");
            }
        }
        assert_eq!(
            serde_json::from_slice::<KillIconCatalog>(&serde_json::to_vec(catalog).unwrap())
                .unwrap(),
            *catalog
        );
    }
}
