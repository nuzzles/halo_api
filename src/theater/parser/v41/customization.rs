//! Offline projection of the recorded 1,852-byte model configuration.
//! Field offsets follow the named serializer layout documented in the appearance
//! investigation; no fitted geometry, asset lookup, or current-player data enters here.
//! Layout reference: installation-08 d731fb7, film-decoder's
//! `section3_slot_perso_research_test.go` (serializer field offsets).
use crate::theater::film::players::*;

pub(super) fn decode(raw: Vec<u8>, build: Option<&str>) -> FilmCustomization {
    let spartan = if matches!(build, Some("HI_1_12_0" | "HI_1_13_0")) && raw.len() == 1852 {
        appearance(&raw)
    } else {
        None
    };
    FilmCustomization { raw, spartan }
}

fn appearance(raw: &[u8]) -> Option<SpartanCustomization> {
    let word = |offset| u32::from_le_bytes(raw[offset..offset + 4].try_into().unwrap());
    let pair = |offset| ModelRegionPermutation {
        region: word(offset),
        permutation: word(offset + 4),
    };
    let count = word(0) as usize;
    // The pairs start at word 2 and end before the theme/coating/marker structure.
    if count > (0x110 - 8) / 8 {
        return None;
    }
    Some(SpartanCustomization {
        regions: (0..count).map(|i| pair(8 + i * 8)).collect(),
        variant_name: word(0x13c),
        style_name: word(0x140),
        model_theme_name: word(0x110),
        model_coating_name: word(0x114),
        model_marker_name: word(0x118),
        attachments: (0..24)
            .map(|i| {
                let o = 0x14c + i * 0x24;
                CustomizationAttachment {
                    variant_name: word(o + 4),
                    style_name: word(o + 8),
                    theme_name: word(o + 12),
                    coating_name: word(o + 16),
                    marker_name: word(o + 20),
                    region_override_name: word(o + 32),
                }
            })
            .collect(),
        objects: (0..7)
            .map(|i| {
                let o = 0x4ac + i * 0x58;
                CustomizationObject {
                    variant_name: word(o + 4),
                    style_name: word(o + 8),
                    theme_name: word(o + 12),
                    coating_name: word(o + 16),
                    marker_name: word(o + 20),
                    regions: (0..8).map(|j| pair(o + 24 + j * 8)).collect(),
                }
            })
            .collect(),
        theme_name: word(0x730),
        coating_name: word(0x734),
        action_pose: word(0x738),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn named_fields_are_little_endian_and_repeated_entries_keep_source_order() {
        let mut raw: Vec<u8> = (0..463u32)
            .flat_map(|i| (0xab000000 + i).to_le_bytes())
            .collect();
        raw[..4].copy_from_slice(&33u32.to_le_bytes());
        for build in ["HI_1_12_0", "HI_1_13_0"] {
            let decoded = decode(raw.clone(), Some(build));
            assert_eq!(decoded.raw, raw);
            let value = decoded.spartan.unwrap();
            assert_eq!(value.regions.len(), 33);
            assert_eq!(value.regions[32].permutation, 0xab000000 + 67);
            assert_eq!(value.model_theme_name, 0xab000044);
            assert_eq!(value.variant_name, 0xab00004f);
            assert_eq!(value.style_name, 0xab000050);
            assert_eq!(value.attachments.len(), 24);
            for (i, entry) in value.attachments.iter().enumerate() {
                let base = 0xab000000 + (0x14c + i as u32 * 0x24) / 4;
                assert_eq!(entry.variant_name, base + 1);
                assert_eq!(entry.region_override_name, base + 8);
            }
            assert_eq!(value.objects.len(), 7);
            for (i, entry) in value.objects.iter().enumerate() {
                let base = 0xab000000 + (0x4ac + i as u32 * 0x58) / 4;
                assert_eq!(entry.theme_name, base + 3);
                assert_eq!(entry.regions.len(), 8);
                assert_eq!(entry.regions[7].permutation, base + 21);
            }
            assert_eq!(value.theme_name, 0xab0001cc);
            assert_eq!(value.coating_name, 0xab0001cd);
            assert_eq!(value.action_pose, 0xab0001ce);
        }
        let old = vec![0x55; 1492];
        let decoded = decode(old.clone(), Some("HI_1_11_0"));
        assert!(decoded.spartan.is_none());
        assert_eq!(decoded.raw, old);
    }

    #[test]
    fn unknown_layout_and_invalid_count_preserve_bytes_without_inventing_fields() {
        let mut bytes = vec![0; 1852];
        assert!(decode(bytes.clone(), Some("future")).spartan.is_none());
        bytes[..4].copy_from_slice(&34u32.to_le_bytes());
        let value = decode(bytes.clone(), Some("HI_1_13_0"));
        assert_eq!(value.raw, bytes);
        assert!(value.spartan.is_none());
        assert!(decode(vec![0; 1851], Some("HI_1_13_0")).spartan.is_none());
    }
}
