//! Explicit reference medal metadata, distinct from empirical summary decoding
//! in `film::summary::medals` and Halo service medal NameIds. Catalogue availability
//! does not establish a recorded writer/build or admit a summary pair.

mod data;
use super::Catalog;
pub use data::Medal;

/// A native medal index. Unknown indices remain representable.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct MedalIndex(pub u32);

impl MedalIndex {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// A medal's category in the reference game files.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum MedalKind {
    /// Kills without dying.
    Spree,
    /// A game mode's own.
    Mode,
    /// Kills in quick succession.
    Multikill,
    /// Kills with one kind of weapon or vehicle.
    Proficiency,
    /// A particular kind of kill.
    Skill,
    /// A kill with flair.
    Style,
}

/// How hard a medal is to earn, as the reference game files rank it.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum MedalDifficulty {
    Normal,
    Heroic,
    Legendary,
    Mythic,
}

impl Medal {
    pub fn index(self) -> MedalIndex {
        MedalIndex(self as u32)
    }

    pub fn kind(self) -> MedalKind {
        self.row().0
    }

    pub fn difficulty(self) -> MedalDifficulty {
        self.row().1
    }

    /// Reference personal score, not the summary sorting weight or observed score.
    pub fn personal_score(self) -> u32 {
        self.row().2
    }

    /// Reference sorting weight, separate from the medal's personal score.
    pub fn sorting_weight(self) -> u32 {
        self.row().3
    }
}

impl Catalog {
    /// Look up metadata without admitting a summary pair or selecting a build.
    pub fn medal(self, index: MedalIndex) -> Option<Medal> {
        Medal::lookup(index.raw())
    }

    /// All defined medals in native index order; index holes are omitted.
    pub fn medals(self) -> impl Iterator<Item = Medal> {
        // This revision's source enum is repr(u8). Do not truncate lookup input.
        (0..=u32::from(u8::MAX)).filter_map(move |i| self.medal(MedalIndex(i)))
    }
}

#[cfg(test)]
mod tests;
