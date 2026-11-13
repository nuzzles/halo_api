//! Tacmap component wire grammars.
use super::position;

use super::reader::ComponentReader;

pub(crate) fn read_primary(
    r: &mut ComponentReader<'_>,
    name: &str,
    level: u32,
    _archetype: u32,
) -> Option<bool> {
    match name {
        "tacmap-backmenulocked-state" => {
            r.r("state", 16)?;
        }
        "tacmap-mapdismissallock" | "tacmap-playercontrol" | "tacmap-tooltip-state" => {
            for i in 0..32 {
                r.gate(&format!("entries[{i}].player"), 5, false)?;
                if name == "tacmap-tooltip-state" {
                    r.bit(&format!("entries[{i}].flag"))?;
                    r.words(&format!("entries[{i}].values"), 2, 32)?;
                }
            }
        }
        "tacmap-queuedreplaymission" | "tacmap-backmenu-openoverride" => {
            for i in 0..32 {
                r.gate(&format!("entry[{i}].handle"), 5, false)?;
                if name == "tacmap-queuedreplaymission" {
                    r.r(&format!("entry[{i}].id"), 32)?;
                } else {
                    r.bit(&format!("entry[{i}].flag"))?;
                }
                r.bit(&format!("entry[{i}].tail"))?;
            }
        }
        "tacmap-cooptetherarea" => {
            if !position::point(r, "position")? {
                return Some(false);
            }
            r.words("extent", 2, 12)?;
        }
        "tacmap-displayasset" => {
            r.words("references", 2, 32)?;
            r.r("kind", 2)?;
            if !position::point(r, "position")? {
                return Some(false);
            }
            for i in 0..2 {
                r.r(&format!("asset[{i}].id"), 64)?;
                r.r(&format!("asset[{i}].word"), 32)?;
            }
            r.bit("flag")?;
        }
        "tacmap-waypointstate" => {
            r.bit("flag")?;
            r.r("reference", 32)?;
            if !position::point(r, "position")? {
                return Some(false);
            }
            if level > 1 {
                r.bit("tail")?;
            }
        }
        "tacmap-iconlodthresholds" => {
            let count = r.r("count", 5)? as usize;
            r.words("thresholds", count, 12)?;
        }
        "tacmap-mapscale" | "tacmap-settingstag" => {
            r.r("value", 32)?;
        }
        "tacmap-cameraheading" => {
            r.r("heading", 12)?;
        }
        "tacmap-missioncount" => {
            r.r("count", 9)?;
        }
        "tacmap-missionmarkerstate" => r.words("words", 2, 32)?,
        "tacmap-lockedlights" => {
            for i in 0..4 {
                r.r(&format!("light[{i}].id"), 32)?;
                r.words(&format!("light[{i}].vector_bits"), 3, 32)?;
            }
        }
        "tacmap-dungeonstate" => {
            r.bit("flag")?;
            r.words("vector_bits", 3, 32)?;
        }
        _ => return Some(false),
    }
    Some(true)
}
