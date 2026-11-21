//! Parser-only references. Presence bits govern whether any reference payload is read.

/// Three positional reference slots, shared by the event envelope grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReferenceTriplet<T> {
    pub first: Option<T>,
    pub second: Option<T>,
    pub third: Option<T>,
}

/// A category-relative recorded reference, not a resolved entity or player identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EventReference {
    pub kind: u8,
    pub index: u16,
    pub generation: u8,
}

/// Read an unguarded reference; callers handle the presence bit or enclosing branch.
pub(crate) fn read_reference(
    mut kind: u8,
    read: &mut impl FnMut(usize) -> Option<u64>,
) -> Option<EventReference> {
    const WIDTHS: [usize; 9] = [13, 13, 8, 8, 9, 8, 9, 13, 13];
    if kind == 1 && read(1)? != 0 {
        kind = 4;
    }
    Some(EventReference {
        kind,
        index: read(*WIDTHS.get(usize::from(kind))?)? as u16,
        generation: read(2)? as u8,
    })
}

pub(crate) fn read_triplet(
    kinds: [i8; 3],
    mut read: impl FnMut(usize) -> Option<u64>,
) -> Option<ReferenceTriplet<EventReference>> {
    let mut optional = |kind| {
        if read(1)? == 0 {
            Some(None)
        } else {
            read_reference(u8::try_from(kind).ok()?, &mut read).map(Some)
        }
    };
    Some(ReferenceTriplet {
        first: optional(kinds[0])?,
        second: optional(kinds[1])?,
        third: optional(kinds[2])?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_words(
        kinds: [i8; 3],
        words: &[(usize, u64)],
    ) -> Option<ReferenceTriplet<EventReference>> {
        let mut words = words.iter();
        let result = read_triplet(kinds, |width| {
            let &(expected_width, value) = words.next()?;
            assert_eq!(width, expected_width);
            Some(value)
        });
        assert!(words.next().is_none());
        result
    }

    #[test]
    fn absent_slots_only_read_presence_bits() {
        assert_eq!(
            read_words([1, 8, 7], &[(1, 0), (1, 0), (1, 0)]),
            Some(ReferenceTriplet {
                first: None,
                second: None,
                third: None
            })
        );
    }

    #[test]
    fn category_widths_and_unknown_categories_are_bounded() {
        for (kind, width) in [13, 13, 8, 8, 9, 8, 9, 13, 13].into_iter().enumerate() {
            let mut words = vec![(1, 1)];
            if kind == 1 {
                words.push((1, 0));
            }
            words.extend([(width, (1 << width) - 1), (2, 3), (1, 0), (1, 0)]);
            assert_eq!(
                read_words([kind as i8, 8, 7], &words).unwrap().first,
                Some(EventReference {
                    kind: kind as u8,
                    index: (1 << width) - 1,
                    generation: 3
                })
            );
        }
        assert!(read_words([-1, 8, 7], &[(1, 1)]).is_none());
        assert!(read_words([9, 8, 7], &[(1, 1)]).is_none());
    }

    #[test]
    fn each_slot_has_independent_presence_and_keeps_its_position() {
        for mask in 0..8 {
            let mut words = Vec::new();
            for slot in 0..3 {
                let present = mask & (1 << slot) != 0;
                words.push((1, u64::from(present)));
                if present {
                    words.extend([(8, slot + 10), (2, slot)]);
                }
            }
            let refs = read_words([2, 2, 2], &words).unwrap();
            for (slot, reference) in [refs.first, refs.second, refs.third]
                .into_iter()
                .enumerate()
            {
                assert_eq!(
                    reference,
                    (mask & (1 << slot) != 0).then_some(EventReference {
                        kind: 2,
                        index: slot as u16 + 10,
                        generation: slot as u8
                    })
                );
            }
        }
    }

    #[test]
    fn category_mode_and_truncated_payloads_are_not_confused_with_absence() {
        assert_eq!(
            read_words([1, 8, 7], &[(1, 1), (1, 1), (9, 5), (2, 2), (1, 0), (1, 0)])
                .unwrap()
                .first,
            Some(EventReference {
                kind: 4,
                index: 5,
                generation: 2
            })
        );
        assert_eq!(
            read_words(
                [1, 8, 7],
                &[(1, 1), (1, 0), (13, 100), (2, 1), (1, 0), (1, 0)]
            )
            .unwrap()
            .first,
            Some(EventReference {
                kind: 1,
                index: 100,
                generation: 1
            })
        );
        assert!(read_words([2, 8, 7], &[(1, 1)]).is_none());
        assert!(read_words([2, 8, 7], &[(1, 1), (8, 7)]).is_none());
        assert!(read_words([2, 8, 7], &[(1, 0), (1, 0)]).is_none());
    }
}
