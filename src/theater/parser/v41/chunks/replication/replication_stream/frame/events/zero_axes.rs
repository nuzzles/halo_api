use super::*;
use crate::theater::film::catalog::maps::zero_fixtures;

#[test]
fn zero_axis_event_points_match_original_reads_without_consuming_following_bits() {
    for c in zero_fixtures::cases() {
        let encoding = PrecisionDescriptor {
            index_bits: 2,
            region: 3,
            axis_bits: c.widths.map(|w| w as u64),
            regions: [(3, c.widths.map(|w| w as u64))].into(),
        };
        let data = c.bytes();
        let mut r = Reader::new(&data, 0).unwrap();
        r.trace = Some(vec![]);
        let Some(EventPoint::Map { region, codes }) = read_event_point(&mut r, Some(&encoding), 22)
        else {
            panic!("{}", c.name)
        };
        assert_eq!((region, codes), (3, c.codes));
        assert_eq!(r.cursor.position, c.bit_len);
        let fields = r.trace.clone().unwrap();
        assert_eq!(
            fields.iter().map(|f| (f.bit, f.width)).collect::<Vec<_>>(),
            c.trace
        );
        for f in &fields {
            if f.width == 0 {
                assert_eq!(f.value, EventFieldValue::Scalar(0));
            }
        }
        assert_eq!(r.read(8), 211);
        let single = PrecisionDescriptor {
            regions: Default::default(),
            ..encoding.clone()
        };
        let mut r = Reader::new(&data, 0).unwrap();
        assert!(
            matches!(read_event_point(&mut r, Some(&single), 22), Some(EventPoint::Map { region: 3, codes }) if codes == c.codes)
        );
        assert_eq!(r.cursor.position, c.bit_len);
        for cut in 0..c.bit_len {
            let (b, shift) = c.cut(cut);
            let mut r = Reader::new(&b, shift).unwrap();
            r.trace = Some(vec![]);
            assert!(read_event_point(&mut r, Some(&encoding), 22).is_none());
            assert!(r.cursor.position <= b.len() * 8);
            let successful: Vec<_> = r
                .trace
                .unwrap()
                .into_iter()
                .filter(|f| !matches!(f.value, EventFieldValue::Unavailable))
                .collect();
            let expected: Vec<_> = fields
                .iter()
                .take_while(|f| f.bit + f.width <= cut)
                .collect();
            assert_eq!(successful.len(), expected.len());
            for (g, e) in successful.iter().zip(expected) {
                assert_eq!(
                    (g.bit, g.width, &g.value),
                    (e.bit + shift, e.width, &e.value)
                );
            }
        }
        let mut r = Reader::new(&data, 0).unwrap();
        assert!(read_event_point(&mut r, None, 22).is_none());
        assert_eq!(r.cursor.position, 1);
        let mut missing = encoding;
        missing.regions = [(2, [0; 3])].into();
        let mut r = Reader::new(&data, 0).unwrap();
        assert!(read_event_point(&mut r, Some(&missing), 22).is_none());
        assert_eq!(r.cursor.position, 3);
    }
}
