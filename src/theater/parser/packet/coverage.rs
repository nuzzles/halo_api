//! Source coverage normalization; no byte reads or semantic reconstruction.
use super::{BitRange, SourceRegion, SourceRegionKind};
use std::collections::BTreeMap;

pub(crate) fn partition(bits: usize, regions: Vec<SourceRegion>) -> Option<Vec<SourceRegion>> {
    let mut boundaries = BTreeMap::<usize, [i64; 4]>::new();
    boundaries.insert(0, [0; 4]);
    boundaries.insert(bits, [0; 4]);
    for region in regions {
        let range = region.source;
        if range.start > range.end || range.end > bits {
            return None;
        }
        if range.start == range.end {
            continue;
        }
        let index = match region.kind {
            SourceRegionKind::Fields => 0,
            SourceRegionKind::Opaque => 1,
            SourceRegionKind::Padding => 2,
            SourceRegionKind::Unparsed => 3,
        };
        boundaries.entry(range.start).or_default()[index] += 1;
        boundaries.entry(range.end).or_default()[index] -= 1;
    }
    let mut active = [0i64; 4];
    let mut result: Vec<SourceRegion> = Vec::new();
    let mut previous = 0;
    for (bit, change) in boundaries {
        if bit > previous {
            let mut kinds = active
                .iter()
                .enumerate()
                .filter(|(_, count)| **count > 0)
                .map(|(index, _)| index);
            let first = kinds.next();
            if kinds.next().is_some() {
                return None;
            }
            let kind = match first {
                Some(0) => SourceRegionKind::Fields,
                Some(1) => SourceRegionKind::Opaque,
                Some(2) => SourceRegionKind::Padding,
                _ => SourceRegionKind::Unparsed,
            };
            if let Some(last) = result.last_mut().filter(|last| last.kind == kind) {
                last.source.end = bit;
            } else {
                result.push(SourceRegion {
                    source: BitRange {
                        start: previous,
                        end: bit,
                    },
                    kind,
                });
            }
        }
        for (count, delta) in active.iter_mut().zip(change) {
            *count += delta;
        }
        if active.iter().any(|count| *count < 0) {
            return None;
        }
        previous = bit;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn region(start: usize, end: usize, kind: SourceRegionKind) -> SourceRegion {
        SourceRegion {
            source: BitRange { start, end },
            kind,
        }
    }
    #[test]
    fn coverage_partitions_gaps_and_rejects_conflicts_or_invalid_extents() {
        use SourceRegionKind::*;
        assert_eq!(
            partition(
                8,
                vec![
                    region(0, 2, Fields),
                    region(1, 4, Fields),
                    region(6, 8, Padding)
                ]
            ),
            Some(vec![
                region(0, 4, Fields),
                region(4, 6, Unparsed),
                region(6, 8, Padding)
            ])
        );
        assert_eq!(
            partition(8, vec![region(0, 4, Fields), region(3, 8, Opaque)]),
            None
        );
        assert_eq!(partition(8, vec![region(7, 9, Fields)]), None);
        assert_eq!(partition(8, vec![region(usize::MAX, 1, Fields)]), None);
        assert_eq!(partition(0, vec![]), Some(vec![]));
    }
}
