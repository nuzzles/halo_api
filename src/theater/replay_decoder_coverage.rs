//! Provenance metadata for the pinned reference implementation ported by this module.
//! Revision strings identify the reference algorithms, not the Rust crate revision.
use super::{FactsFileIdentity, ReplayByteString, ReplayDecoderCoverage, ReplayRegistryCoverage};
use serde::{Deserialize, Serialize};

/// Pinned reference facts algorithm revision (`decfilm.Rev`), not Rust parity status.
pub const NATIVE_FACTS_REVISION: &str = "killsource-2026-09-22.2";

/// Publish the identity already read by LegacyFilm, without reopening bootstrap bytes.
#[allow(dead_code)]
pub(crate) fn build_film_replay_decoder_coverage(
    film: &super::LegacyFilm,
) -> ReplayDecoderCoverage {
    let identity = film
        .identity
        .as_ref()
        .map(|identity| ReplayDecoderIdentity {
            format_version: i64::from(film.registry.format_version),
            build: ReplayByteString(identity.build.as_bytes().to_vec()),
            registry_fingerprint: film.registry.fingerprint().unwrap_or(0),
            registry_blocks: film.registry.archetypes.len() as i64,
            registry_named_slots: film
                .registry
                .archetypes
                .iter()
                .map(|a| a.components.len() as i64)
                .sum(),
        });
    build_replay_decoder_coverage(identity.as_ref())
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayDecoderIdentity {
    pub format_version: i64,
    pub build: ReplayByteString,
    pub registry_fingerprint: u64,
    pub registry_blocks: i64,
    pub registry_named_slots: i64,
}

/// Restore the decoder projection of the cache's optional native FilmIdentity.
/// This reads the identity section, not the persisted coverage header, whose
/// revisions describe the earlier scan. No recorded identity field is normalized.
impl From<&FactsFileIdentity> for ReplayDecoderIdentity {
    fn from(identity: &FactsFileIdentity) -> Self {
        Self {
            format_version: identity.format_version,
            build: ReplayByteString(identity.build.clone()),
            registry_fingerprint: identity.registry_fingerprint,
            registry_blocks: identity.registry_blocks,
            registry_named_slots: identity.registry_named_slots,
        }
    }
}

/// Native registry classification of the recorded v41 build and fingerprint.
pub fn replay_registry_status(build: &str, fingerprint: u64) -> &'static str {
    super::classify_v41_registry_fingerprint(build, fingerprint)
}

/// Preserve native revision, build-publication and absent-registry semantics.
pub fn build_replay_decoder_coverage(
    identity: Option<&ReplayDecoderIdentity>,
) -> ReplayDecoderCoverage {
    let mut coverage = ReplayDecoderCoverage {
        source_rev: "source-2026-09-16.2".into(),
        profile_rev: "profile-2026-09-17.3".into(),
        grammar_rev: "grammar-2026-09-22.12".into(),
        facts_rev: NATIVE_FACTS_REVISION.into(),
        ..Default::default()
    };
    let Some(identity) = identity else {
        return coverage;
    };
    // Build lookup uses exact known ASCII keys. Invalid bytes cannot match a key;
    // keep the original bytes in the identity rather than normalizing them.
    let build = std::str::from_utf8(&identity.build.0).unwrap_or("");
    if matches!(identity.format_version, 20 | 21 | 24 | 25 | 27)
        && super::expected_v41_registry_fingerprint(build).is_some()
    {
        coverage.build = build.to_owned();
    }
    if identity.registry_fingerprint != 0 {
        coverage.registry = Some(ReplayRegistryCoverage {
            fingerprint: format!("0x{:016x}", identity.registry_fingerprint),
            status: replay_registry_status(build, identity.registry_fingerprint).into(),
            blocks: identity.registry_blocks,
            named_slots: identity.registry_named_slots,
        });
    }
    coverage
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        identity: Option<ReplayDecoderIdentity>,
        coverage: ReplayDecoderCoverage,
    }
    #[test]
    fn native_decoder_provenance() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/decoder-coverage-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        for (i, row) in rows.into_iter().enumerate() {
            assert_eq!(
                build_replay_decoder_coverage(row.identity.as_ref()),
                row.coverage,
                "case {i}"
            );
        }
    }

    /// Compare production constants and observable adapters with values emitted
    /// by the pinned native facade, independently of Rust's implementation.
    #[test]
    fn native_public_constant_contracts() {
        use crate::theater::*;
        use serde_json::{Value, json};
        let expected: Value =
            serde_json::from_str(include_str!("fixtures/public-constants-v41.json")).unwrap();
        let actual = json!({
            "Rev": NATIVE_FACTS_REVISION,
            "NomGardeEquipementNegatifAZero": NEGATIVE_EQUIPMENT_KEPT_FALLBACK,
            "NomGestePremiereVieDuSlot": FIRST_SLOT_LIFE_FALLBACK,
            "CoverageWarnRatio": KILL_SOURCE_COVERAGE_WARN_RATIO,
            "UnexplainedWarnRatio": KILL_SOURCE_UNEXPLAINED_WARN_RATIO,
            "UnexplainedAlertRatio": KILL_SOURCE_UNEXPLAINED_ALERT_RATIO,
            "KnownRegistryFingerprint": KNOWN_REGISTRY_FINGERPRINT,
            "WeaponHitPairWindowUS": WEAPON_HIT_PAIR_WINDOW_US,
            "BotSuffix": BOT_SUFFIX,
            "XUIDNamePrefix": XUID_NAME_PREFIX,
            "MapQuantSchemaVersion": FILM_MAP_CATALOG_SCHEMA_VERSION,
            "PIBits": PLAYER_INDEX_BITS,
            "StatPlayerSlots": STATBORG_PLAYER_SLOTS,
            "ErrNoKillFeed": NO_KILL_FEED.to_string(),
            "PathWalk": KillReadPath::Walk,
            "PathScan": KillReadPath::Scan,
        });
        for (name, value) in actual.as_object().unwrap() {
            // JSON distinguishes 1.0 from 1; compare native numeric meaning.
            if value.is_f64() {
                assert_eq!(value.as_f64(), expected[name].as_f64(), "{name}");
            } else {
                assert_eq!(value, &expected[name], "{name}");
            }
        }
        for name in [
            "None",
            "Headshot",
            "HeadshotMultiplier",
            "SilentMelee",
            "CollisionDamage",
            "AttachedDamage",
        ] {
            let code = expected[format!("Category{name}")].as_i64().unwrap();
            assert_eq!(
                kill_damage_category_name(i32::try_from(code).unwrap()),
                name
            );
        }
        for (mode, key) in [
            ("flag", "StatFlagCaptures"),
            ("zone", "StatZoneCaptures"),
            ("zone", "StatZoneSecures"),
        ] {
            assert!(statborg_known_stats(mode).contains(expected[key].as_str().unwrap()));
        }
    }
}
