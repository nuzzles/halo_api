use super::*;
use std::io::Read;

fn fixture() -> (Vec<u8>, FilmRegistry, serde_json::Value) {
    let mut data = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
        .read_to_end(&mut data)
        .unwrap();
    let registry = crate::theater::parse_registry(&data).unwrap();
    let oracle = serde_json::from_str(include_str!("fixtures/bootstrap-v41-oracle.json")).unwrap();
    (data, registry, oracle)
}
fn raw(data: &[u8], bit: usize, width: usize) -> u64 {
    (bit..bit + width).fold(0, |v, i| {
        (v << 1) | u64::from((data[i / 8] >> (7 - i % 8)) & 1)
    })
}
#[test]
fn native_identity_writer_map_and_reference_projection() {
    let (data, registry, oracle) = fixture();
    let read = read_native_identity(&data, &registry);
    let identity = read.identity.as_ref().unwrap();
    assert!(read.error.is_none());
    assert_eq!(
        read.identity,
        decode_film_identity(&data, &registry).unwrap()
    );
    assert_eq!(identity.body_bit, oracle["identity"]["BodyBit"]);
    let anchor = oracle["identity"]["BuildOffset"].as_u64().unwrap() as usize;
    assert_eq!(read.build_anchor_byte, Some(anchor));
    // Independent ordered writer map from pinned film_identity.go lines 16-29.
    let mut widths = vec![32; oracle["identity"]["TypeVersions"].as_array().unwrap().len()];
    widths.extend([
        256, 256, 256, 32, 32, 1, 2048, 2048, 32, 32, 32, 32, 32768, 32768, 32768, 128, 128,
    ]);
    assert_eq!(widths.len(), read.fields.len());
    let mut bit = registry.end_byte * 8;
    for (field, width) in read.fields.iter().zip(widths) {
        assert_eq!((field.bit, field.bits), (bit, width));
        match &field.value {
            NativeIdentityValue::Scalar(value) => assert_eq!(*value, raw(&data, bit, width)),
            NativeIdentityValue::Bytes(bytes) => {
                assert_eq!(bytes.len() * 8, width);
                for (i, value) in bytes.iter().enumerate() {
                    assert_eq!(u64::from(*value), raw(&data, bit + i * 8, 8));
                }
            }
            NativeIdentityValue::Opaque => assert!(matches!(width, 32768 | 128)),
            NativeIdentityValue::Unavailable => panic!("complete fixture"),
        }
        bit += width;
    }
    assert_eq!(bit, identity.body_bit);
    let timestamp = read
        .fields
        .iter()
        .find(|f| f.name == "match_start_unix")
        .unwrap();
    assert_eq!(
        timestamp.value,
        NativeIdentityValue::Scalar(u64::from(identity.match_start_unix.swap_bytes()))
    );
    assert_eq!(
        read,
        serde_json::from_slice(&serde_json::to_vec(&read).unwrap()).unwrap()
    );
}
#[test]
fn native_identity_retains_partial_fields_and_unavailable_ranges() {
    let (data, registry, _) = fixture();
    let complete = read_native_identity(&data, &registry);
    let anchor = complete.build_anchor_byte.unwrap();
    for field in complete.fields.iter().filter(|f| f.bit > (anchor + 33) * 8) {
        // One byte short of each complete field; preserves its available prefix
        // in source while reporting the entire requested range as unavailable.
        let cut = (field.bit + field.bits).div_ceil(8) - 1;
        let partial = read_native_identity(&data[..cut], &registry);
        assert!(partial.identity.is_none());
        assert!(partial.error.is_some());
        assert_eq!(partial.build_anchor_byte, Some(anchor));
        let last = partial.fields.last().unwrap();
        assert_eq!((last.bit, last.bits), (field.bit, field.bits));
        assert_eq!(last.value, NativeIdentityValue::Unavailable);
        assert_eq!(
            &partial.fields[..partial.fields.len() - 1],
            &complete.fields[..partial.fields.len() - 1]
        );
    }
    let mut absent = data.clone();
    absent[anchor..anchor + 3].copy_from_slice(b"NO_");
    let read = read_native_identity(&absent, &registry);
    assert!(read.build_anchor_byte.is_none());
    assert!(read.fields.is_empty());
    assert!(read.identity.is_none());
}
#[test]
fn native_identity_retains_string_tails_and_uninterpreted_words() {
    let (mut data, registry, _) = fixture();
    let original = read_native_identity(&data, &registry);
    let anchor = original.build_anchor_byte.unwrap();
    // Byte after the version string terminator is not in its normalized value.
    data[anchor - 1] = 0xfd;
    let unknown = original
        .fields
        .iter()
        .find(|f| f.name == "unknown_word")
        .unwrap();
    for (i, set) in (0..32).map(|i| (i, (0xa5a55a5au32 >> (31 - i)) & 1 != 0)) {
        let bit = unknown.bit + i;
        let mask = 1 << (7 - bit % 8);
        if set {
            data[bit / 8] |= mask;
        } else {
            data[bit / 8] &= !mask;
        }
    }
    let changed = read_native_identity(&data, &registry);
    assert_eq!(changed.identity, original.identity);
    assert_ne!(changed.fields, original.fields);
    let version = changed.fields.iter().find(|f| f.name == "version").unwrap();
    let NativeIdentityValue::Bytes(bytes) = &version.value else {
        panic!()
    };
    assert_eq!(bytes[31], 0xfd);
    assert_eq!(
        changed
            .fields
            .iter()
            .find(|f| f.name == "unknown_word")
            .unwrap()
            .value,
        NativeIdentityValue::Scalar(0xa5a55a5a)
    );
}
