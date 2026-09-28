//! Query support for ResolvedFilm.
use super::*;
/// Inclusive timestamp bounds. Filtering never alters playback state.
#[derive(Debug, Clone, Copy, Default)]
pub struct EventFilter {
    pub start_us: Option<u64>,
    pub end_us: Option<u64>,
    pub kind: Option<EventKind>,
    pub category: Option<EventCategory>,
    pub entity_id: Option<u32>,
    pub player_index: Option<usize>,
}

#[derive(Debug, Default)]
pub(super) struct QueryIndices {
    kinds: BTreeMap<EventKind, Vec<usize>>,
    categories: BTreeMap<EventCategory, Vec<usize>>,
    pub(super) entities: BTreeMap<u32, Vec<usize>>,
    players: BTreeMap<usize, Vec<usize>>,
}
impl QueryIndices {
    pub(super) fn new(events: &[Event]) -> Self {
        let mut out = Self::default();
        for (i, event) in events.iter().enumerate() {
            out.kinds.entry(event.kind).or_default().push(i);
            out.categories
                .entry(event.kind.category())
                .or_default()
                .push(i);
            if let Some(id) = event.entity_id {
                out.entities.entry(id).or_default().push(i);
            }
            if let Some(id) = event.player_index {
                out.players.entry(id).or_default().push(i);
            }
        }
        out
    }
}

impl ResolvedFilm<'_> {
    /// Binary-search time bounds in the smallest applicable entity/player/kind/
    /// category index, then test the remaining predicates on those candidates.
    /// Without categorical filters, iterate the timestamp slice directly.
    pub fn query(&self, filter: EventFilter) -> impl Iterator<Item = &Event> {
        let start = filter
            .start_us
            .map_or(0, |t| self.events.partition_point(|e| e.timestamp_us < t));
        let end = filter.end_us.map_or(self.events.len(), |t| {
            self.events.partition_point(|e| e.timestamp_us <= t)
        });
        let start = start.min(end);
        let indices = &self.query_indices;
        let selected = [
            filter
                .kind
                .map(|v| indices.kinds.get(&v).map_or(&[][..], Vec::as_slice)),
            filter
                .category
                .map(|v| indices.categories.get(&v).map_or(&[][..], Vec::as_slice)),
            filter
                .entity_id
                .map(|v| indices.entities.get(&v).map_or(&[][..], Vec::as_slice)),
            filter
                .player_index
                .map(|v| indices.players.get(&v).map_or(&[][..], Vec::as_slice)),
        ]
        .into_iter()
        .flatten()
        .map(|ids| &ids[ids.partition_point(|&i| i < start)..ids.partition_point(|&i| i < end)])
        .min_by_key(|ids| ids.len());
        let mut selected = selected.map(|ids| ids.iter().copied());
        let mut range = start..end;
        std::iter::from_fn(move || {
            let index = match &mut selected {
                Some(ids) => ids.next(),
                None => range.next(),
            }?;
            Some(&self.events[index])
        })
        .filter(move |e| {
            filter.kind.is_none_or(|v| e.kind == v)
                && filter.category.is_none_or(|v| e.kind.category() == v)
                && filter.entity_id.is_none_or(|v| e.entity_id == Some(v))
                && filter
                    .player_index
                    .is_none_or(|v| e.player_index == Some(v))
        })
    }
}
