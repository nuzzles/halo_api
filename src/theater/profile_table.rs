//! Pinned native profile table and its evidence. Rows describe reference-parser
//! knowledge, including presumed values; they are not newly verified game facts.
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilmProfileProvenance {
    #[serde(rename = "relue")]
    BinaryReviewed,
    #[serde(rename = "mesuree")]
    Measured,
    #[serde(rename = "presumee")]
    Presumed,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmProfileTableRow {
    #[serde(rename = "Cle")]
    pub key: String,
    #[serde(rename = "Champ")]
    pub field: String,
    #[serde(rename = "Valeur")]
    pub value: String,
    #[serde(rename = "Source")]
    pub provenance: FilmProfileProvenance,
    #[serde(rename = "Preuve")]
    pub evidence: String,
    #[serde(rename = "Date")]
    pub date: String,
}
/// Full pinned reference table, in native order. Each call returns an independent
/// owned copy. Historical rows are retained as metadata, not added decode support.
pub fn native_film_profile_table() -> Vec<FilmProfileTableRow> {
    static ROWS: OnceLock<Vec<FilmProfileTableRow>> = OnceLock::new();
    ROWS.get_or_init(|| {
        serde_json::from_str(include_str!("reference/profile-table.json"))
            .expect("validated native profile table")
    })
    .clone()
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilmGamertagLayout {
    #[serde(rename = "gamertag_en_tete")]
    AtStart,
    #[serde(rename = "gamertag_decale_12")]
    Offset12,
}
impl FilmGamertagLayout {
    pub fn offset_bytes(self) -> usize {
        match self {
            Self::AtStart => 0,
            Self::Offset12 => 12,
        }
    }
}
/// Native profile selector only. It does not enable other major versions in the
/// v41 highlight decoder. Unknown version zero retains the native at-start layout.
pub fn native_gamertag_layout(major: i64) -> FilmGamertagLayout {
    if (39..=40).contains(&major) {
        FilmGamertagLayout::Offset12
    } else {
        FilmGamertagLayout::AtStart
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_profile_table_and_layouts() {
        let rows = native_film_profile_table();
        assert_eq!(rows.len(), 33);
        // Compare all six serialized fields and their order to the native export;
        // also catch fields accidentally dropped by the typed schema.
        let native: serde_json::Value =
            serde_json::from_str(include_str!("reference/profile-table.json")).unwrap();
        assert_eq!(serde_json::to_value(&rows).unwrap(), native);
        assert_eq!(
            [
                FilmProfileProvenance::BinaryReviewed,
                FilmProfileProvenance::Measured,
                FilmProfileProvenance::Presumed
            ]
            .map(|p| rows.iter().filter(|r| r.provenance == p).count()),
            [7, 21, 5]
        );
        assert_eq!(
            (&rows[0].key[..], &rows[0].field[..], &rows[0].value[..]),
            ("format=27", "MPP", "lead=9 index=5")
        );
        assert_eq!(rows[1].value, "INDETERMINEE (aucune largeur posee)");
        assert_eq!(rows[11].key, "majeure>=41");
        assert_eq!(rows[11].value, "gamertag_en_tete (b[0:32])");
        assert_eq!(rows[32].key, "majeure=31");
        assert!(rows[32].value.contains("0x20400"));
        let mut changed = rows.clone();
        changed[0].value.clear();
        assert_eq!(native_film_profile_table(), rows);
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/profile-layouts-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let layouts: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let layouts = layouts.as_array().unwrap();
        assert_eq!(layouts.len(), 103);
        for row in layouts {
            let layout = native_gamertag_layout(row["MajorVersion"].as_i64().unwrap());
            assert_eq!(serde_json::to_value(layout).unwrap(), row["Implantation"]);
            assert_eq!(
                layout.offset_bytes(),
                row["GamertagOffsetBytes"].as_u64().unwrap() as usize
            );
        }
    }
}
