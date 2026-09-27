//! Checked player customization window in packet-type-8 snapshots.
use super::bits::Bits;
use super::{ArmorAppearance, ArmorAttachments, ArmorRegion};

fn id(b: Bits<'_>, o: usize) -> Option<i32> {
    Some((b.read(o, 32)? as u32).swap_bytes() as i32)
}

/// Offsets are relative to the existing roster marker, not the visible name.
/// Only the observed 22-base-region form is supported. Intervening fields remain opaque.
pub(super) fn snapshot(b: Bits<'_>, o: usize) -> Option<(u64, String, ArmorAppearance)> {
    let start = o.checked_sub(488)?;
    if !b.is(o, "0010110111000000")
        || b.read(o + 16, 30)? > 1
        || b.read(o + 46, 8)? != 22
        || b.read(o + 54, 56)? != 0
        || (0..21).any(|i| b.read(o - 232 + 8 * i, 8) != Some(0))
    {
        return None;
    }
    let xuid = b.read(o - 64, 64)?.swap_bytes();
    if xuid == 0 {
        return None;
    }
    let mut utf16 = [0; 16];
    for (i, unit) in utf16.iter_mut().enumerate() {
        *unit = (b.read(start + i * 16, 16)? as u16).swap_bytes();
    }
    let name = String::from_utf16(&utf16)
        .ok()?
        .trim_matches('\0')
        .trim()
        .to_owned();
    if name.is_empty() || name.chars().any(char::is_control) {
        return None;
    }
    let mut regions = Vec::with_capacity(22);
    for i in 0..22 {
        let region_id = id(b, o + 110 + i * 64)?;
        if region_id == 0
            || regions
                .iter()
                .any(|r: &ArmorRegion| r.region_id == region_id)
        {
            return None;
        }
        regions.push(ArmorRegion {
            region_id,
            permutation_id: id(b, o + 142 + i * 64)?,
        });
    }
    Some((
        xuid,
        name,
        ArmorAppearance {
            regions,
            visor_id: id(b, o + 2318)?,
            visor_color_id: id(b, o + 2414)?,
            armor_variant_id: id(b, o + 2574)?,
            coating_style_id: id(b, o + 2606)?,
            attachments: Some(ArmorAttachments {
                chest_tag_id: id(b, o + 2030)?,
                utility_tag_id: id(b, o + 2094)?,
                wrist_tag_id: id(b, o + 2126)?,
                left_shoulder_tag_id: id(b, o + 2158)?,
                right_shoulder_tag_id: id(b, o + 2190)?,
            }),
            mythic_effect_ids: Some([
                id(b, o + 2446)?,
                id(b, o + 2478)?,
                id(b, o + 2510)?,
                id(b, o + 2542)?,
            ]),
        },
    ))
}
