//! Offline projection of the recorded 1,852-byte model configuration.
//! Field offsets follow the named serializer layout documented in the appearance
//! investigation; no fitted geometry, asset lookup, or current-player data enters here.
//! Layout reference: installation-08 d731fb7, film-decoder's
//! `section3_slot_perso_research_test.go` (serializer field offsets).
use crate::theater::film::ParseError;
use crate::theater::film::players::*;
use crate::theater::parser::v41::packet::BitRange;

/// Borrow aligned blocks, or realign into a bounded stack buffer. The packet
/// already owns the source; no second heap allocation is needed for appearance.
pub(super) fn decode_source(
    data: &[u8],
    source: BitRange,
    build: Option<&str>,
) -> Result<FilmCustomization, ParseError> {
    if source.end.checked_sub(source.start) != Some(1852 * 8)
        || source.end > data.len().saturating_mul(8)
    {
        return Err(ParseError::InvalidCustomization);
    }
    let first = source.start / 8;
    let shift = source.start % 8;
    let bytes = &data[first..first + 1852];
    if shift == 0 {
        return decode(bytes, build);
    }
    let mut aligned = [0; 1852];
    for (out, pair) in aligned
        .iter_mut()
        .zip(data[first..=first + 1852].windows(2))
    {
        *out = pair[0] << shift | pair[1] >> (8 - shift);
    }
    decode(&aligned, build)
}

pub(super) fn decode(raw: &[u8], build: Option<&str>) -> Result<FilmCustomization, ParseError> {
    let spartan = if matches!(build, Some("HI_1_12_0" | "HI_1_13_0")) && raw.len() == 1852 {
        appearance(raw)
    } else {
        None
    };
    Ok(FilmCustomization {
        spartan: spartan.ok_or(ParseError::InvalidCustomization)?,
    })
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
    fn source_projection_matches_owned_bits_at_every_alignment() {
        use crate::theater::parser::wire::fields::RawBits;
        let mut raw: Vec<u8> = (0..1852).map(|i| (i * 37) as u8).collect();
        raw[..4].copy_from_slice(&33u32.to_le_bytes());
        let expected = decode(&raw, Some("HI_1_13_0")).unwrap();
        for start in 0..8 {
            let end = start + raw.len() * 8;
            let mut data = vec![0; end.div_ceil(8)];
            for bit in 0..raw.len() * 8 {
                data[(start + bit) / 8] |=
                    ((raw[bit / 8] >> (7 - bit % 8)) & 1) << (7 - (start + bit) % 8);
            }
            let source = BitRange { start, end };
            assert_eq!(
                decode_source(&data, source, Some("HI_1_13_0")).unwrap(),
                expected
            );
            assert_eq!(
                RawBits::from_source(&data, start, 1852 * 8)
                    .unwrap()
                    .bytes
                    .as_slice(),
                raw
            );
            assert!(decode_source(&data[..data.len() - 1], source, Some("HI_1_13_0")).is_err());
            assert!(decode_source(&data, source, Some("future")).is_err());
            assert!(
                decode_source(
                    &data,
                    BitRange {
                        start,
                        end: end - 1
                    },
                    Some("HI_1_13_0")
                )
                .is_err()
            );
        }
        assert!(
            decode_source(
                &raw,
                BitRange {
                    start: usize::MAX,
                    end: 0
                },
                Some("HI_1_13_0")
            )
            .is_err()
        );
    }
    #[test]
    fn named_fields_are_little_endian_and_repeated_entries_keep_source_order() {
        let mut raw: Vec<u8> = (0..463u32)
            .flat_map(|i| (0xab000000 + i).to_le_bytes())
            .collect();
        raw[..4].copy_from_slice(&33u32.to_le_bytes());
        for build in ["HI_1_12_0", "HI_1_13_0"] {
            let value = decode(&raw, Some(build)).unwrap().spartan;
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
        assert!(matches!(
            decode(&old, Some("HI_1_11_0")),
            Err(ParseError::InvalidCustomization)
        ));
    }

    #[test]
    fn unknown_layout_and_invalid_count_fail_decoding() {
        let mut bytes = vec![0; 1852];
        for build in [None, Some("future")] {
            assert!(matches!(
                decode(&bytes, build),
                Err(ParseError::InvalidCustomization)
            ));
        }
        bytes[..4].copy_from_slice(&34u32.to_le_bytes());
        assert!(matches!(
            decode(&bytes, Some("HI_1_13_0")),
            Err(ParseError::InvalidCustomization)
        ));
        assert!(matches!(
            decode(&[0; 1851], Some("HI_1_13_0")),
            Err(ParseError::InvalidCustomization)
        ));
    }

    #[test]
    fn serialized_customization_requires_spartan_and_has_no_raw_copy() {
        let value = decode(&[0; 1852], Some("HI_1_13_0")).unwrap();
        let mut json = serde_json::to_value(&value).unwrap();
        assert!(json.get("raw").is_none());
        assert_eq!(
            serde_json::from_value::<FilmCustomization>(json.clone()).unwrap(),
            value
        );
        json["spartan"] = serde_json::Value::Null;
        assert!(serde_json::from_value::<FilmCustomization>(json.clone()).is_err());
        json.as_object_mut().unwrap().remove("spartan");
        assert!(serde_json::from_value::<FilmCustomization>(json).is_err());
    }
}
