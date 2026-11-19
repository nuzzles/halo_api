//! Shared action-variant bodies for biped lists and NetworkedActionRequest.
use super::reader::ComponentReader;

/// None: truncated field; false: unsupported variant or recorded branch.
pub(crate) fn read(r: &mut ComponentReader<'_>, variant: u64) -> Option<bool> {
    match variant {
        0 => {
            r.optional_handle("reference", 0)?;
            r.gate("value", 8, true)?;
        }
        1 => {
            r.gate("value", 8, true)?;
            r.words("bytes", 2, 8)?;
            r.r("word", 32)?;
            r.r("value15", 15)?;
        }
        2 => {
            r.r("byte", 8)?;
            r.gate("value", 8, true)?;
            r.r("word", 16)?;
        }
        3 => {
            r.r("enum", 2)?;
            r.r("value15", 15)?;
        }
        4 => {
            r.r("variant", 32)?;
            r.r("byte", 8)?;
            r.r("value15", 15)?;
            r.direction(10)?;
            r.gate("value", 8, true)?;
            r.r("word", 16)?;
        }
        5 => {
            r.r("variant", 32)?;
            r.gate("value", 8, true)?;
            r.r("word", 16)?;
            r.r("byte", 8)?;
        }
        6 => {
            r.gate("value", 8, true)?;
            r.r("word", 32)?;
        }
        7 => {
            r.gate("value", 8, true)?;
            r.r("kind", 3)?;
            r.r("value15", 15)?;
        }
        8..=10 => {
            r.gate("direction", 19, true)?;
            r.r("word", 32)?;
            r.r("kind", 4)?;
            if variant == 8 {
                r.r("extra", 4)?;
                r.bit("flag")?;
            }
        }
        11 => {
            r.gate("value", 8, true)?;
            r.r("direction", 19)?;
            r.r("kind", 2)?;
        }
        12 => {
            r.r("word", 32)?;
            r.bit("flag")?;
        }
        13 => {
            r.words("raw_words", 3, 32)?;
            r.r("direction", 19)?;
            r.gate("value", 8, true)?;
        }
        14 => {
            r.r("byte", 8)?;
        }
        15 | 22 => r.gate("value", 8, true)?,
        16 => r.direction(10)?,
        17 => {
            r.words("values15", 2, 15)?;
            r.bit("flag")?;
            r.gate("value", 8, true)?;
        }
        18 => {
            if r.bit("unsupported_branch")? {
                return Some(false);
            }
            r.r("kind", 2)?;
            r.r("word", 32)?;
            r.bit("flag")?;
            r.r("value16", 16)?;
            r.r("byte", 8)?;
            r.gate("value", 8, true)?;
        }
        19 => {
            r.r("word", 32)?;
            r.r("byte", 8)?;
            r.gate("value", 8, true)?;
        }
        20 => {
            r.r("value15", 15)?;
        }
        21 | 23 | 24 => {
            r.gate("value", 8, true)?;
            r.r("word", 32)?;
            r.r(
                "tail",
                match variant {
                    21 => 4,
                    23 => 1,
                    _ => 2,
                },
            )?;
        }
        25 => {
            r.r("word_a", 32)?;
            r.r("value16", 16)?;
            r.r("word_b", 32)?;
            r.bit("flag")?;
            r.r("byte", 8)?;
            r.gate("value", 8, true)?;
        }
        _ => return Some(false),
    }
    Some(true)
}
