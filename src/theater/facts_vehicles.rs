//! Native VehicleScan cache composition in its exact section order.
use super::*;
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsVehicleScan {
    pub scanned: bool,
    pub keyframes: FactsWorldKeyframes,
    pub creations: Vec<FactsEquipmentCreation>,
    pub stats: FactsCreationStats,
    pub positions: Vec<FactsBipedPosition>,
    pub events: Vec<FactsVehicleEvent>,
    pub aims: Vec<FactsVehicleAim>,
    pub deaths: FactsObjectDeaths,
    pub occupancy: Vec<VehicleOccupancy>,
}
pub fn encode_facts_vehicle_scan(w: &mut NativeFactsWriter, s: &FactsVehicleScan) {
    w.boolean(s.scanned);
    encode_facts_keyframes(w, &s.keyframes);
    encode_facts_creations(w, &s.creations);
    encode_facts_creation_stats(w, &s.stats);
    encode_facts_positions(w, &s.positions);
    encode_facts_vehicle_events(w, &s.events);
    encode_facts_vehicle_aims(w, &s.aims);
    encode_facts_object_deaths(w, &s.deaths);
    encode_facts_vehicle_occupancy(w, &s.occupancy);
}
pub fn decode_facts_vehicle_scan(
    r: &mut NativeFactsReader<'_>,
    layout: &I0Layout,
    bounds: [[f32; 3]; 2],
) -> FactsVehicleScan {
    FactsVehicleScan {
        scanned: r.boolean(),
        keyframes: decode_facts_keyframes(r),
        creations: decode_facts_creations(r),
        stats: decode_facts_creation_stats(r),
        positions: decode_facts_positions(r, layout, bounds),
        events: decode_facts_vehicle_events(r),
        aims: decode_facts_vehicle_aims(r),
        deaths: decode_facts_object_deaths(r),
        occupancy: decode_facts_vehicle_occupancy(r),
    }
}
