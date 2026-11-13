use super::*;
use crate::theater::{film::catalog::maps::levels, parser::v41::context::map::level_precision};

#[test]
fn catalogue_event_points_keep_region_codes_and_following_boundary() {
    let oracles = levels::fixtures::oracles();
    let mut count = 0;
    for case in levels::fixtures::cases()
        .into_iter()
        .filter(|c| c.kind == "event")
    {
        count += 1;
        let level = levels::resolve(levels::BUILD, case.tag).unwrap();
        let encoding = level_precision(level, case.precision).unwrap();
        let bytes = case.bytes();
        let mut r = Reader::new(&bytes, 0).unwrap();
        r.trace = Some(Vec::new());
        let point =
            read_event_point(&mut r, Some(&encoding), usize::from(case.precision) + 6).unwrap();
        match point {
            EventPoint::DefaultBox => assert!(case.region.is_none()),
            EventPoint::Map { region, codes } => {
                assert_eq!(Some(region), case.region);
                assert_eq!(codes, case.codes);
            }
        }
        assert_eq!(r.cursor.position, case.bit_len, "{}", case.name);
        if case.upstream {
            let oracle = oracles.iter().find(|o| o.name == case.name).unwrap();
            assert_eq!(oracle.end, case.bit_len);
            assert_eq!(oracle.marker, 211);
            assert_eq!(
                r.trace
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|f| (f.bit, f.width))
                    .collect::<Vec<_>>(),
                oracle.trace
            );
        }
        assert_eq!(r.read(8), 211);
        for cut in 0..case.bit_len {
            let (bytes, shift) = case.cut(cut);
            let mut r = Reader::new(&bytes, shift).unwrap();
            assert!(
                read_event_point(&mut r, Some(&encoding), usize::from(case.precision) + 6)
                    .is_none(),
                "{} cut {cut}",
                case.name
            );
            assert!(r.cursor.position <= bytes.len() * 8);
        }
        // Every default-box form is self-contained, including without a catalogue.
        if case.region.is_none() {
            let mut r = Reader::new(&bytes, 0).unwrap();
            assert!(matches!(
                read_event_point(&mut r, None, usize::from(case.precision) + 6),
                Some(EventPoint::DefaultBox)
            ));
        }
    }
    assert_eq!(count, 198);
}

#[test]
fn indexed_event_points_refuse_unknown_regions_and_invalid_selected_widths() {
    let case = levels::fixtures::cases()
        .into_iter()
        .find(|c| c.kind == "event" && c.region == Some(4))
        .unwrap();
    let level = levels::resolve(levels::BUILD, case.tag).unwrap();
    let bytes = case.bytes();
    let mut encoding = level_precision(level, case.precision).unwrap();
    for invalid in [None, Some([33, 15, 15]), Some([15, 33, 15])] {
        encoding.regions.remove(&4);
        if let Some(axes) = invalid {
            encoding.regions.insert(4, axes);
        }
        let mut r = Reader::new(&bytes, 0).unwrap();
        assert!(read_event_point(&mut r, Some(&encoding), 22).is_none());
        assert_eq!(r.cursor.position, 1 + level.index_bits);
        assert!(!r.over); // Missing layout is distinct from missing source bytes.
    }
}

#[test]
fn region_detonations_preserve_following_events_and_coordinate_codes() {
    for case in levels::fixtures::cases()
        .into_iter()
        .filter(|c| c.kind == "event" && c.precision == 15)
    {
        let level = levels::resolve(levels::BUILD, case.tag).unwrap();
        let encoding = ProjectilePositionEncoding {
            detonation: level_precision(level, 15).unwrap(),
            impact: level_precision(level, 12).unwrap(),
        };
        // Detonation with three absent references, no variant and no optional tag.
        let mut bits = format!("11{:07b}00000000010", 5);
        let source = case.bytes();
        for i in 0..case.bit_len {
            bits.push(if source[i / 8] & (1 << (7 - i % 8)) != 0 {
                '1'
            } else {
                '0'
            });
        }
        bits.push_str(&"0".repeat(19 + 5 + 1 + 9 + 1 + 1 + 8 + 2));
        let event_end = bits.len();
        bits.push_str(&format!("1{:07b}000100", 21)); // zoom, then list terminator
        let mut bytes = vec![0; bits.len().div_ceil(8)];
        for (i, b) in bits.bytes().enumerate() {
            bytes[i / 8] |= (b - b'0') << (7 - i % 8);
        }
        let read = read_event_list(
            &bytes,
            1,
            100,
            EventLayout {
                projectile_positions: Some(&encoding),
                ..Default::default()
            },
        );
        assert_eq!(read.stop, EventListStop::Terminator, "{}", case.name);
        assert_eq!(read.records.len(), 2);
        assert_eq!(read.records[0].end_bit, event_end);
        assert_eq!(read.records[1].code, Some(21));
        assert_eq!(read.end_bit, bits.len());
        assert_eq!(
            detonation_region_position(&bytes, &read.records[0], &encoding),
            case.region.map(|r| (r, case.codes))
        );
        let mut incomplete = read.records[0].clone();
        incomplete.layout_complete = false;
        assert!(detonation_region_position(&bytes, &incomplete, &encoding).is_none());
        let mut wrong_end = read.records[0].clone();
        wrong_end.end_bit += 1;
        assert!(detonation_region_position(&bytes, &wrong_end, &encoding).is_none());
    }
}
