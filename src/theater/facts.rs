//! Complete REPLAYINPUTS25 derived-cache blob. This is the reference's cache
//! projection, not lossless native LegacyFilm storage or the deferred replay model.
use super::*;
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NativeFilmFacts {
    pub header: FactsHeader,
    pub positions: Vec<FactsBipedPosition>,
    pub biped_creations: Vec<FactsBipedCreation>,
    pub events: FactsEvents,
    pub weapon_changes: Vec<FactsWeaponChange>,
    pub pickups: Vec<FactsPickup>,
    pub pickup_stats: FactsPickupStats,
    pub inventory: FactsInventory,
    pub delta_channels: FactsDeltaChannels,
    pub equipment_changes: Vec<FactsEquipmentChange>,
    pub equipment_change_stats: FactsEquipmentChangeStats,
    pub abilities: FactsAbilities,
    pub movement: FactsMovement,
    pub zoom_events: Vec<FactsZoomEvent>,
    pub world: FactsWorldSection,
    pub vehicles: FactsVehicleScan,
    pub queue: FactsQueue,
}
/// Both native encoder results are available on the returned writer: bytes and
/// the optional embedded JSON error. Bytes may be partial when error() is set.
/// Callers persisting the cache must inspect that error before publishing it.
pub fn encode_film_facts(g: &NativeFilmFacts) -> NativeFactsWriter {
    let mut w = NativeFactsWriter::default();
    for &b in FILM_FACTS_MAGIC {
        w.byte(b);
    }
    encode_facts_header(&mut w, &g.header);
    encode_facts_positions(&mut w, &g.positions);
    encode_facts_biped_creations(&mut w, &g.biped_creations);
    encode_facts_events(&mut w, &g.events);
    encode_facts_weapon_changes(&mut w, &g.weapon_changes);
    encode_facts_pickups(&mut w, &g.pickups, &g.pickup_stats);
    encode_facts_inventory(&mut w, &g.inventory);
    encode_facts_delta_channels(&mut w, &g.delta_channels);
    encode_facts_equipment_changes(&mut w, &g.equipment_changes, &g.equipment_change_stats);
    encode_facts_abilities(&mut w, &g.abilities);
    encode_facts_movement(&mut w, &g.movement);
    encode_facts_zoom_events(&mut w, &g.zoom_events);
    encode_facts_world_section(&mut w, &g.world);
    encode_facts_vehicle_scan(&mut w, &g.vehicles);
    encode_facts_queue(&mut w, &g.queue);
    w
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FactsDecodeError {
    #[error(transparent)]
    Header(#[from] FactsHeaderError),
    #[error("{0}")]
    Transport(String),
    #[error("faits de film : {0} octet(s) non consomme(s) — format desynchronise")]
    Trailing(usize),
}
/// Native top-level order and rejection behavior. Partial sections remain
/// available through their individual codecs; this API returns no facts on error.
pub fn decode_film_facts(
    blob: &[u8],
    entry: &FactsMapEntry,
) -> Result<NativeFilmFacts, FactsDecodeError> {
    let DecodedFactsHeader {
        header,
        mut reader,
        layout,
        bounds,
    } = decode_facts_header(blob, entry)?;
    let bounds = std::array::from_fn(|end| std::array::from_fn(|axis| bounds[axis][end]));
    let r = &mut reader;
    let positions = decode_facts_positions(r, &layout, bounds);
    let biped_creations = decode_facts_biped_creations(r);
    let events = decode_facts_events(r);
    let weapon_changes = decode_facts_weapon_changes(r);
    let (pickups, pickup_stats) = decode_facts_pickups(r);
    let inventory = decode_facts_inventory(r);
    let delta_channels = decode_facts_delta_channels(r);
    let (equipment_changes, equipment_change_stats) = decode_facts_equipment_changes(r);
    let abilities = decode_facts_abilities(r);
    let movement = decode_facts_movement(r);
    let zoom_events = decode_facts_zoom_events(r);
    let world = decode_facts_world_section(r);
    let vehicles = decode_facts_vehicle_scan(r, &layout, bounds);
    let queue = decode_facts_queue(r);
    if let Some(error) = r.error() {
        return Err(FactsDecodeError::Transport(error.to_owned()));
    }
    if r.remaining() != 0 {
        return Err(FactsDecodeError::Trailing(r.remaining()));
    }
    Ok(NativeFilmFacts {
        header,
        positions,
        biped_creations,
        events,
        weapon_changes,
        pickups,
        pickup_stats,
        inventory,
        delta_channels,
        equipment_changes,
        equipment_change_stats,
        abilities,
        movement,
        zoom_events,
        world,
        vehicles,
        queue,
    })
}
