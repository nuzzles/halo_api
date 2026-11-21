//! Known match-option hashes and the game script names they identify.
//! This is the name catalogue only; it never supplies missing recorded values.
use super::OptionHash;

macro_rules! define_known_options {
    ($($variant:ident = $hash:literal, $name:literal;)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub enum KnownMatchOption {
            $(#[serde(rename = $name)] $variant,)*
        }

        /// Catalogue entries in ascending hash order.
        pub const ALL: &[KnownMatchOption] = &[$(KnownMatchOption::$variant,)*];

        impl KnownMatchOption {
            pub const fn hash(self) -> OptionHash {
                match self { $(Self::$variant => OptionHash($hash),)* }
            }

            /// Original game script spelling.
            pub const fn name(self) -> &'static str {
                match self { $(Self::$variant => $name,)* }
            }
        }
    };
}

define_known_options! {
    FirefightAutoReviveTimerSec = 0x01232435, "FirefightOptions_AutoReviveTimerSec";
    FirefightCustomSquadSelectionOrder = 0x03590249, "FirefightOptions_customSquadSelectionOrder";
    StrongholdsSuddenDeathTrailingScoreThreshold = 0x05063395, "StrongholdsOptions_suddenDeathTrailingScoreThreshold";
    StrongholdsScoreIntervalSec = 0x09c666be, "StrongholdsOptions_scoreIntervalSec";
    FirefightBonusLivesPerSet = 0x0c06e820, "FirefightOptions_BonusLivesPerSet";
    BastionControlAreaAttributeShieldRechargeRate = 0x0ed758f2, "BastionOptions_controlAreaAttributeShieldRechargeRate";
    SlayerBeatdownBonus = 0x12e5cc5c, "SlayerOptions_BeatdownBonus";
    MinigameScorePerKill = 0x146c15bc, "MinigameOptions_scorePerKill";
    FirefightLimitEagleRespawns = 0x155ec7ab, "FirefightOptions_LimitEagleRespawns";
    StrongholdsDenySpawningWithinStrongholds = 0x174dd1bf, "StrongholdsOptions_DenySpawningWithinStrongholds";
    SlayerScorePerKill = 0x185f4770, "SlayerOptions_scorePerKill";
    OddballScoreIntervalSec = 0x1c43311a, "OddballOptions_scoreIntervalSec";
    CtfFlagCarrierSpottedOnMotionTrackerReveal = 0x203d64f6, "CTFOptions_flagCarrierSpottedOnMotionTrackerReveal";
    MinigameScorePerBetrayal = 0x23c9c488, "MinigameOptions_scorePerBetrayal";
    BastionControlAreaAttributeShieldMultiplier = 0x28c9cbdd, "BastionOptions_controlAreaAttributeShieldMultiplier";
    StrongholdsMinCapturedStrongholdsToScore = 0x29dd302e, "StrongholdsOptions_minCapturedStrongholdsToScore";
    CtfFlagReturnTime = 0x2b16b76f, "CTFOptions_FlagReturnTime";
    BastionLegacyScorePerInterval = 0x2c3acf66, "BastionOptions_legacyScorePerInterval";
    SlayerScorePerAssist = 0x2edd6f0a, "SlayerOptions_ScorePerAssist";
    FirefightPreventItemRespawns = 0x319ee3aa, "FirefightOptions_PreventItemRespawns";
    MinigameMaxSpawnVehiclesPerTeam = 0x320420a7, "MinigameOptions_maxSpawnVehiclesPerTeam";
    CtfFlagCarrierShieldPercent = 0x335e1c17, "CTFOptions_FlagCarrierShieldPercent";
    FirefightOverrideAiPlacementsSource = 0x360e8ebc, "FirefightOptions_overrideAIPlacementsSource";
    MinigameMinigameIncludeLabel2 = 0x3694e32e, "MinigameOptions_MinigameIncludeLabel2";
    FirefightSetsAreSequential = 0x3702bd82, "FirefightOptions_setsAreSequential";
    FirefightFixedSetCount = 0x392b0e0e, "FirefightOptions_fixedSetCount";
    CtfFlagCarrierGravityScalarPercent = 0x39a2a695, "CTFOptions_FlagCarrierGravityScalarPercent";
    StrongholdsInitialStrongholdOwnership = 0x3c47d503, "StrongholdsOptions_initialStrongholdOwnership";
    StrongholdsCaptureTimeSec = 0x3ca8dc42, "StrongholdsOptions_captureTimeSec";
    SpartanChatterDisabledByVariant = 0x3ffe4893, "SpartanChatterDisabledByVariant";
    FirefightAutoReviveTimerEnabled = 0x4802a6d3, "FirefightOptions_AutoReviveTimerEnabled";
    CtfFlagCarrierMovespeedPercent = 0x486738e0, "CTFOptions_flagCarrierMovespeedPercent";
    BastionLegacyScoringEnabled = 0x495626e1, "BastionOptions_LegacyScoringEnabled";
    SlayerScorePerBetrayal = 0x4c4efbf5, "SlayerOptions_scorePerBetrayal";
    MinigameScorePerAssist = 0x4cb2458d, "MinigameOptions_ScorePerAssist";
    BastionStrongholdLifetimeSec = 0x4dd357b6, "BastionOptions_strongholdLifetimeSec";
    SlayerScorePerDeath = 0x4eec1dcb, "SlayerOptions_ScorePerDeath";
    FirefightAllySecureScalarEnabled = 0x523734bb, "FirefightOptions_allySecureScalarEnabled";
    CtfNumberOfFlagsPerTeam = 0x5593370e, "CTFOptions_numberOfFlagsPerTeam";
    CtfFlagCarrierHealthPercent = 0x57ca980e, "CTFOptions_FlagCarrierHealthPercent";
    BastionControlAreaAttributeMovementSpeed = 0x59d1b6e6, "BastionOptions_controlAreaAttributeMovementSpeed";
    FirefightSkullStreakType = 0x5aa619d5, "FirefightOptions_SkullStreakType";
    SlayerSplatterBonus = 0x67e1535a, "SlayerOptions_splatterBonus";
    MinigameSpawnInVehicleFillType = 0x688d293d, "MinigameOptions_spawnInVehicleFillType";
    CtfInstantTouchReturnEnabled = 0x6d597d79, "CTFOptions_instantTouchReturnEnabled";
    FirefightLimitOtherRespawns = 0x6e666162, "FirefightOptions_LimitOtherRespawns";
    FirefightAllySecureAccelerationScalar = 0x6ecc87ba, "FirefightOptions_AllySecureAccelerationScalar";
    MinigameScorePerSuicide = 0x6edb7718, "MinigameOptions_scorePerSuicide";
    BastionControlAreaAttributeBodyRechargeRate = 0x6f92a41c, "BastionOptions_controlAreaAttributeBodyRechargeRate";
    StrongholdsMultipleCapturerBehavior = 0x70519e56, "StrongholdsOptions_multipleCapturerBehavior";
    StrongholdsPointsPerScore = 0x7062053f, "StrongholdsOptions_pointsPerScore";
    FirefightSetDefaultTeamOnJoin = 0x71395ce9, "FirefightOptions_setDefaultTeamOnJoin";
    BastionControlAreaAttributeSprintEnabled = 0x74703beb, "BastionOptions_controlAreaAttributeSprintEnabled";
    MinigameMinigameIncludeLabel4 = 0x74e88540, "MinigameOptions_minigameIncludeLabel4";
    MinigameSplatterBonus = 0x76105331, "MinigameOptions_splatterBonus";
    FirefightReviveEnabled = 0x7615c072, "FirefightOptions_ReviveEnabled";
    MinigameSpawnInVehicleCreation = 0x798db005, "MinigameOptions_SpawnInVehicleCreation";
    BastionControlAreaAttributeMeleeDamageMultiplier = 0x7baf2ce9, "BastionOptions_controlAreaAttributeMeleeDamageMultiplier";
    SlayerScorePerSuicide = 0x7c1360a2, "SlayerOptions_scorePerSuicide";
    MinigameBeatdownBonus = 0x7cf0c2fa, "MinigameOptions_BeatdownBonus";
    MinigameStickyBonus = 0x7e73ddcf, "MinigameOptions_StickyBonus";
    FirefightOverrideMapAiPlacements = 0x804ec21d, "FirefightOptions_overrideMapAIPlacements";
    SlayerHeadshotBonus = 0x8056d833, "SlayerOptions_HeadshotBonus";
    CtfFlagCarrierSpottedOnDamage = 0x80a98a8f, "CTFOptions_flagCarrierSpottedOnDamage";
    CtfFlagMustBeHomeToScoreOnFlagSpawn = 0x85030c58, "CTFOptions_flagMustBeHomeToScoreOnFlagSpawn";
    StrongholdsStrongholdContestType = 0x9d933798, "StrongholdsOptions_StrongholdContestType";
    FirefightUseResupplyDrops = 0xa3a5a643, "FirefightOptions_useResupplyDrops";
    FirefightSkullStreakPenaltyType = 0xb9ba89d5, "FirefightOptions_skullStreakPenaltyType";
    MinigameSpawnInVehicleType = 0xb9e1d4d7, "MinigameOptions_spawnInVehicleType";
    BastionControlAreaAttributeWeaponDamageMultiplier = 0xbff5c0ab, "BastionOptions_controlAreaAttributeWeaponDamageMultiplier";
    MinigameMaxSpawnPlayersPerVehicle = 0xc173d3e8, "MinigameOptions_MaxSpawnPlayersPerVehicle";
    MinigameMinigameIncludeLabel3 = 0xc57f2026, "MinigameOptions_minigameIncludeLabel3";
    StrongholdsContestingPausesScoring = 0xc8831e9c, "StrongholdsOptions_ContestingPausesScoring";
    BastionScoreTickIntervalSec = 0xc8ff7749, "BastionOptions_scoreTickIntervalSec";
    FirefightSaveReviveInteractProgress = 0xc95d4843, "FirefightOptions_SaveReviveInteractProgress";
    CtfScorePerFlagCap = 0xcd284eea, "CTFOptions_scorePerFlagCap";
    CompetitiveBattleRifleWeaponRackOverride = 0xd53749a6, "CompetitiveBattleRifleWeaponRackOverride";
    FirefightHillIncomingTimer = 0xd6c47fc7, "FirefightOptions_HillIncomingTimer";
    FirefightReviveGrantsOvershield = 0xd74fb3e5, "FirefightOptions_reviveGrantsOvershield";
    FirefightHillCaptureDuration = 0xd9cd1adc, "FirefightOptions_HillCaptureDuration";
    MinigameMinigameIncludeLabel1 = 0xdd6adfc4, "MinigameOptions_minigameIncludeLabel1";
    CtfNavpointVisTypeEnum = 0xdf247dd7, "CTFOptions_navpointVisTypeEnum";
    BastionControlAreaAttributeSlideDurationScalar = 0xe211f1ae, "BastionOptions_controlAreaAttributeSlideDurationScalar";
    SlayerStickyBonus = 0xe35ebfea, "SlayerOptions_StickyBonus";
    OddballOddballIncomingTime = 0xe5463b68, "OddballOptions_oddballIncomingTime";
    CtfFlagSightSpottedDuration = 0xe6f47bc1, "CTFOptions_flagSightSpottedDuration";
    CtfFlagCarrierCanSprint = 0xf0623736, "CTFOptions_flagCarrierCanSprint";
    OddballBallCarrierCanSprint = 0xf216b08a, "OddballOptions_ballCarrierCanSprint";
    StrongholdsSuddenDeathRespawnPenaltySeconds = 0xf2393376, "StrongholdsOptions_suddenDeathRespawnPenaltySeconds";
    OddballScoreIncrement = 0xf37e4ee6, "OddballOptions_scoreIncrement";
    BastionControlAreaAttributeSlideSpeedScalar = 0xf386d3cc, "BastionOptions_controlAreaAttributeSlideSpeedScalar";
    BastionControlAreaAttributeSprintFullSpeed = 0xf663e7fc, "BastionOptions_controlAreaAttributeSprintFullSpeed";
    MinigameHeadshotBonus = 0xf8afbb0c, "MinigameOptions_HeadshotBonus";
    MinigameMinigameIncludeLabel5 = 0xf8eaafc4, "MinigameOptions_minigameIncludeLabel5";
    FirefightTeamRespawnCount = 0xfcbeec04, "FirefightOptions_teamRespawnCount";
    MinigameScorePerDeath = 0xfe0fb57c, "MinigameOptions_ScorePerDeath";
    BastionControlAreaAttributeHealthMultiplier = 0xfe38d1be, "BastionOptions_controlAreaAttributeHealthMultiplier";
    FirefightReviveTimeSec = 0xff5aeb0c, "FirefightOptions_reviveTimeSec";
}
