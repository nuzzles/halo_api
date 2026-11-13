//! Build-specific level-tag catalogue, including every recorded BSP region.
use super::PositionBounds;
use uuid::Uuid;
mod data;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelRegion {
    pub bounds: PositionBounds,
    /// Widths read from the pinned level catalogue at precision 16.
    pub axis_bits: [usize; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelCalibration {
    pub tag: u32,
    pub structure_tag: u32,
    pub module: &'static str,
    /// English name when the game's map info supplies one.
    pub display_name: Option<&'static str>,
    /// Halo service MatchInfo.LevelId, separate from a map variant's asset ID.
    pub api_id: Option<Uuid>,
    pub index_bits: usize,
    pub play: LevelRegion,
    pub others: &'static [LevelRegion],
}
impl LevelCalibration {
    pub fn region(&self, index: u32) -> Option<&LevelRegion> {
        std::iter::once(&self.play)
            .chain(self.others)
            .find(|r| r.bounds.region == index)
    }
    pub fn regions(&self) -> impl Iterator<Item = &LevelRegion> {
        std::iter::once(&self.play).chain(self.others)
    }
}

pub const BUILD: &str = "269225.26.04.08.1618 HI_1_13_0";
/// Numeric engine build identifier, distinct from the product version string.
pub const BUILD_ID: u32 = 269225;
pub const BRANCH: &str = "HI_1_13_0";
/// Find the catalogued level matching a Halo service MatchInfo.LevelId.
/// This is a reference lookup and does not establish a film's writer build.
pub fn by_api_id(id: Uuid) -> Option<&'static LevelCalibration> {
    data::LEVELS.iter().find(|level| level.api_id == Some(id))
}

/// Exact build and level-tag lookup. A neighboring build never supplies bounds.
pub fn resolve(build: &str, tag: u32) -> Option<&'static LevelCalibration> {
    if build != BUILD {
        return None;
    }
    data::LEVELS
        .binary_search_by_key(&tag, |l| l.tag)
        .ok()
        .map(|i| &data::LEVELS[i])
}

/// Resolve the engine build identity recorded in the bootstrap header.
/// The product version (e.g. `6.10026.19225.0`) is a separate namespace.
pub(crate) fn resolve_recorded(
    build_id: u32,
    branch: &str,
    tag: u32,
) -> Option<&'static LevelCalibration> {
    (build_id == BUILD_ID && branch == BRANCH)
        .then(|| resolve(BUILD, tag))
        .flatten()
}

pub fn all() -> &'static [LevelCalibration] {
    data::LEVELS
}

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod fixtures;
