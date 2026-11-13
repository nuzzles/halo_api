//! Reference option names and mode-script defaults. These are not recorded
//! Bond values, resolved variant overrides, or evidence for a writer/build.

mod data;
use super::Catalog;
use data::{MODES, OPTION_NAMES};

/// A variant option's lowercased-name hash, distinct from asset or service IDs.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct OptionHash(pub u32);

impl OptionHash {
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Hash an ASCII option name using the game's ASCII lowercase convention.
    /// This computes a hash even when the name is absent from the catalogue.
    pub fn from_name(name: &str) -> Self {
        Self(murmur3(name.to_ascii_lowercase().as_bytes()))
    }
}

/// A literal script default. It does not supply a missing recorded value.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum OptionValue {
    Number(f64),
    Bool(bool),
}

/// One option in a mode's reference Options script.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ModeOption {
    pub name: &'static str,
    /// All aliases in source order, including older names absent from hash lookup.
    pub keys: &'static [&'static str],
    /// The script's literal default; None includes nonliteral defaults.
    pub default: Option<OptionValue>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ModeOptions {
    pub prefix: &'static str,
    /// Lowercased Options script filename.
    pub script: &'static str,
    /// Options in source script order.
    pub options: &'static [ModeOption],
}

impl Catalog {
    /// Original hash/name entries in hash order. Alias lists are separate.
    pub fn option_names(self) -> impl ExactSizeIterator<Item = (OptionHash, &'static str)> {
        OPTION_NAMES
            .iter()
            .map(|&(hash, name)| (OptionHash(hash), name))
    }

    pub fn option_name(self, hash: OptionHash) -> Option<&'static str> {
        OPTION_NAMES
            .binary_search_by_key(&hash.raw(), |o| o.0)
            .ok()
            .map(|i| OPTION_NAMES[i].1)
    }

    /// Reference scripts only; this does not select a recorded mode or apply defaults.
    pub fn mode_options(self) -> &'static [ModeOptions] {
        MODES
    }
}

/// Murmur3 x86 32-bit, seed zero. Input is hashed verbatim without lowercasing.
pub fn murmur3(data: &[u8]) -> u32 {
    let (c1, c2) = (0xcc9e_2d51u32, 0x1b87_3593u32);
    let mut h = 0u32;
    let (chunks, tail) = data.as_chunks::<4>();
    for w in chunks {
        let k = u32::from_le_bytes(*w)
            .wrapping_mul(c1)
            .rotate_left(15)
            .wrapping_mul(c2);
        h = (h ^ k)
            .rotate_left(13)
            .wrapping_mul(5)
            .wrapping_add(0xe654_6b64);
    }
    if !tail.is_empty() {
        let k = tail
            .iter()
            .enumerate()
            .fold(0u32, |k, (i, &b)| k | (b as u32) << (8 * i));
        h ^= k.wrapping_mul(c1).rotate_left(15).wrapping_mul(c2);
    }
    h ^= data.len() as u32;
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

#[cfg(test)]
mod tests;
