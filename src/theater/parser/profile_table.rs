//! Pinned native profile table and its evidence. Rows describe reference-parser
//! knowledge, including presumed values; they are not newly verified game facts.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum FilmGamertagLayout {
    #[serde(rename = "gamertag_en_tete")]
    AtStart,
    #[serde(rename = "gamertag_decale_12")]
    Offset12,
}
/// Native profile selector only. It does not enable other major versions in the
/// v41 highlight decoder. Unknown version zero retains the native at-start layout.
pub(crate) fn native_gamertag_layout(major: i64) -> FilmGamertagLayout {
    if (39..=40).contains(&major) {
        FilmGamertagLayout::Offset12
    } else {
        FilmGamertagLayout::AtStart
    }
}
