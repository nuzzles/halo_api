//! Native world-scan cache section, composed in the reference field order.
use super::*;
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsWorldObjectScan {
    pub scanned: bool,
    pub creations: Vec<FactsEquipmentCreation>,
    pub stats: FactsCreationStats,
    pub keyframes: FactsWorldKeyframes,
    pub tracks: Vec<FactsProjectileTrack>,
}
pub fn encode_facts_world_scan(writer: &mut NativeFactsWriter, scan: &FactsWorldObjectScan) {
    writer.boolean(scan.scanned);
    encode_facts_creations(writer, &scan.creations);
    encode_facts_creation_stats(writer, &scan.stats);
    encode_facts_keyframes(writer, &scan.keyframes);
    encode_facts_tracks(writer, &scan.tracks);
}
pub fn decode_facts_world_scan(reader: &mut NativeFactsReader<'_>) -> FactsWorldObjectScan {
    FactsWorldObjectScan {
        scanned: reader.boolean(),
        creations: decode_facts_creations(reader),
        stats: decode_facts_creation_stats(reader),
        keyframes: decode_facts_keyframes(reader),
        tracks: decode_facts_tracks(reader),
    }
}
