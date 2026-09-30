//! Dynamic orientation and simulation grammar. Raw direction/roll fields retain
//! mode information; downstream reconstruction must distinguish absolute/delta.
use super::position;
use super::reader::ComponentReader;

pub(crate) fn dynamic(r: &mut ComponentReader<'_>, level: u32) -> Option<()> {
    let mut mode = 0;
    let delta = if r.bit("raw_vectors")? {
        mode = 2;
        false
    } else {
        r.bit("delta")?
    };
    if level >= 2 && r.bit("config_precision")? {
        mode = 1;
    }
    match mode {
        2 => r.words("orientation_vector_bits", 6, 32)?,
        1 => {
            r.gate("up_direction", 30, false)?;
            r.r("roll", 30)?;
        }
        _ if delta => {
            if !r.bit("keep_direction")? {
                if !r.bit("delta_direction")? {
                    r.r("up_direction", 19)?;
                } else {
                    r.words("direction_deltas", 2, 4)?;
                }
            }
            r.gate("roll_delta", 4, true)?;
        }
        _ => {
            r.gate("up_direction", 19, false)?;
            r.r("roll", 8)?;
        }
    }
    Some(())
}

pub(crate) fn media_frame(r: &mut ComponentReader<'_>) -> Option<()> {
    let encoding = r.position_encoding?;
    if encoding.full_precision_gate() {
        r.words("media_frame.vector_bits", 3, 32)?;
    } else {
        position::absolute_payload(r, encoding)?;
    }
    Some(())
}

pub(crate) fn simulation(r: &mut ComponentReader<'_>) -> Option<bool> {
    if r.position_encoding.is_none() {
        return Some(false);
    }
    if !r.bit("present")? {
        return Some(true);
    }
    r.gate("index_a", 5, false)?;
    r.gate("index_b", 5, false)?;
    r.words("state_a", 4, 16)?;
    r.words("kinds", 2, 2)?;
    r.words("state_b", 4, 16)?;
    r.gate("direction", 19, false)?;
    r.r("magnitude", 8)?;
    media_frame(r)?;
    Some(true)
}
