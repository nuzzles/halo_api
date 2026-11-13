//! Sequential game-options prefix, Bond messages and fixed tail before players.
use super::bond::{BondBits, BondRead, BondStop};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GameOptionsRead {
    pub start_bit: usize,
    /// Only Complete establishes the following player-record boundary.
    pub end_bit: usize,
    pub entries: Vec<GameOptionsEntry>,
    pub stop: GameOptionsStop,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GameOptionsEntry {
    Field {
        kind: GameOptionsFieldKind,
        bits: BondBits,
    },
    Bond {
        kind: GameOptionsBondKind,
        read: BondRead,
    },
    /// Each byte, including the terminating NUL, has its own source extent.
    String {
        index: u8,
        bytes: Vec<BondBits>,
        terminated: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GameOptionsFieldKind {
    GameMode,
    RecorderSimulation,
    Prefix(u8),
    LevelTag,
    AfterShortName(u8),
    OptionalBlockPresent,
    OptionalBlock,
    BeforeVariant(u8),
    VariantPresent,
    TailWord,
    TailBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GameOptionsBondKind {
    Options,
    Variant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GameOptionsStop {
    Complete,
    Truncated,
    UnterminatedString,
    Bond(BondStop),
}
