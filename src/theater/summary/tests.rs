use super::*;
use crate::clients::hi::models::FilmChunk;
use crate::theater::{FILM_MEDAL_DEFINITIONS, FilmMedal, film_medal_definition};

fn fixture() -> FilmChunkData {
    FilmChunkData {
        metadata: FilmChunk {
            index: 5,
            start_time_offset_ms: 0,
            duration_ms: 0,
            size: 0,
            file_relative_path: String::new(),
            chunk_type: 3,
        },
        data: include_bytes!("../fixtures/summary-v41.bin").to_vec(),
    }
}

fn write_bits(data: &mut [u8], offset: usize, width: usize, value: u64) {
    for i in 0..width {
        let bit = offset + i;
        let mask = 1 << (7 - bit % 8);
        data[bit / 8] = (data[bit / 8] & !mask) | (((value >> (width - i - 1)) as u8 & 1) * mask);
    }
}

#[test]
fn captured_footer_decodes_without_roster_or_replication() {
    let report = decode_summary_events(&[fixture()], 41).unwrap();
    assert!(report.matches_declared_counts());
    assert_eq!(report.events.len(), 3);
    let events: Vec<_> = report
        .events
        .iter()
        .map(|e| (e.name.as_str(), e.time_us, e.kind))
        .collect();
    assert_eq!(
        events,
        [
            ("Nuzzles", 25_344_000, SummaryKind::Kill),
            ("Yet", 25_344_000, SummaryKind::Death),
            ("Nuzzles", 77_279_000, SummaryKind::Medal),
        ]
    );
    assert_eq!(report.events[0].xuid, "2535472547643888");
    assert_eq!(report.events[1].xuid, "2535443507298499");
    let medal = report.events[2].medal.as_ref().unwrap();
    assert_eq!(medal.film_id, 32);
    assert_eq!(medal.name.as_deref(), Some("Steaktacular"));
    assert_eq!(medal.name_id, Some(1169390319));
    assert_eq!(report.events[2].type_code, Some(150));
    assert!(report.events.iter().all(|e| e.player.is_none()));
    // These are actual unaligned source positions, not byte-rounded approximations.
    assert_eq!(report.events[0].source.unwrap().bit + 16 * 8, 27_532);
    assert_eq!(
        report.events[0].identity_source.unwrap().bit + 16 * 8,
        12_606
    );
}

#[test]
fn unrecognized_codes_and_distinct_duplicate_awards_are_preserved() {
    let mut chunk = fixture();
    let source = decode_summary_events(&[chunk.clone()], 41).unwrap().events[2]
        .source
        .unwrap();
    let absolute = source.payload_byte * 8 + source.bit;
    write_bits(&mut chunk.data, absolute + 59 * 8, 8, 255);
    let mut second = chunk.clone();
    second.metadata.index += 1;
    let report = decode_summary_events(&[second, chunk.clone()], 41).unwrap();
    assert!(report.matches_declared_counts());
    assert_eq!(report.events.len(), 6);
    let medals: Vec<_> = report
        .events
        .iter()
        .filter_map(|e| e.medal.as_ref())
        .collect();
    assert_eq!(medals.len(), 2);
    assert!(
        medals
            .iter()
            .all(|m| m.film_id == 255 && m.name.is_none() && m.name_id.is_none())
    );
    write_bits(&mut chunk.data, absolute + 47 * 8, 8, 77);
    write_bits(&mut chunk.data, absolute + 55 * 8, 8, 0);
    let event = decode_summary_events(&[chunk], 41)
        .unwrap()
        .events
        .pop()
        .unwrap();
    assert_eq!(event.kind, SummaryKind::Other(77));
    assert_eq!(event.metadata, 255);
    assert!(event.medal.is_none());
}

#[test]
fn corrupt_tail_is_reported_as_missing_instead_of_cross_assigned() {
    let mut chunk = fixture();
    let source = decode_summary_events(&[chunk.clone()], 41).unwrap().events[0]
        .source
        .unwrap();
    write_bits(
        &mut chunk.data,
        source.payload_byte * 8 + source.end_bit,
        32,
        0,
    );
    let report = decode_summary_events(&[chunk], 41).unwrap();
    assert!(!report.matches_declared_counts());
    assert_eq!(report.packets[0].declared_events, 3);
    assert_eq!(report.events.len(), 2);
    assert_eq!(report.events[0].name, "Yet");
    assert_eq!(report.events[0].kind, SummaryKind::Death);
}

#[test]
fn checks_packet_boundaries_versions_and_duplicate_chunks() {
    assert!(
        !decode_summary_events(&[], 41)
            .unwrap()
            .matches_declared_counts()
    );
    assert!(matches!(
        decode_summary_events(&[], 40),
        Err(DecodeError::UnsupportedVersion(40))
    ));
    assert!(matches!(
        decode_summary_events(&[fixture(), fixture()], 41),
        Err(DecodeError::DuplicateChunk(5))
    ));
    let mut chunk = fixture();
    chunk.data.truncate(chunk.data.len() - 1);
    assert!(matches!(
        decode_summary_events(&[chunk], 41),
        Err(DecodeError::Truncated { .. })
    ));
    let mut chunk = fixture();
    chunk.data[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        decode_summary_events(&[chunk], 41),
        Err(DecodeError::Truncated { .. })
    ));
}

#[test]
fn portable_events_accept_old_json_and_roundtrip_enriched_json() {
    let event: SummaryEvent = serde_json::from_value(serde_json::json!({
        "xuid":"1", "player":null, "name":"Player", "time_us":0,
        "kind":"Medal", "metadata":105, "medal_flag":1
    }))
    .unwrap();
    assert!(event.source.is_none());
    assert!(event.type_code.is_none());
    let report = decode_summary_events(&[fixture()], 41).unwrap();
    let restored: SummaryEventReport =
        serde_json::from_slice(&serde_json::to_vec(&report).unwrap()).unwrap();
    assert_eq!(report, restored);
}

#[test]
fn catalog_includes_all_cms_medals_and_preserves_unknown_ids() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/medal_catalog.json")).unwrap();
    assert_eq!(FILM_MEDAL_DEFINITIONS.len(), 155);
    assert_eq!(
        FILM_MEDAL_DEFINITIONS
            .iter()
            .filter(|m| m.name_id.is_some())
            .count(),
        151
    );
    let mut ids = BTreeSet::new();
    for row in fixture["definitions"].as_array().unwrap() {
        let id = row["film_id"].as_u64().unwrap() as u8;
        let m = film_medal_definition(id).unwrap();
        assert!(ids.insert(id));
        assert_eq!(m.name, row["name"].as_str().unwrap());
        assert_eq!(m.name_id.map(u64::from), row["name_id"].as_u64());
        assert_eq!(
            m.sorting_weight.map(u64::from),
            row["sorting_weight"].as_u64()
        );
    }
    assert!(
        FILM_MEDAL_DEFINITIONS
            .windows(2)
            .all(|m| m[0].film_id < m[1].film_id)
    );
    assert_eq!(FilmMedal::from_id(105).name(), "Reversal");
    assert_eq!(FilmMedal::from_id(117).name_id(), Some(3334154676));
    assert_eq!(FilmMedal::from_id(166).name(), "Mounted & Loaded");
    assert_eq!(FilmMedal::from_id(255), FilmMedal::Unknown(255));
}

#[test]
fn validates_medal_identity_not_just_equal_totals() {
    let stats: MatchStats =
        serde_json::from_str(include_str!("../fixtures/summary-match-stats.json")).unwrap();
    let mut report = decode_summary_events(&[fixture()], 41).unwrap();
    assert!(validate_summary_events(&report, &stats).matches_stats());
    // Replace Steaktacular with Reversal: totals are identical but the identity is wrong.
    report.events[2].metadata = 105;
    let validation = validate_summary_events(&report, &stats);
    assert!(
        validation
            .players
            .iter()
            .all(|p| p.decoded == p.match_stats)
    );
    assert!(!validation.matches_stats());
    report.events[2].metadata = 255;
    let validation = validate_summary_events(&report, &stats);
    assert!(!validation.matches_stats());
    assert_eq!(
        validation
            .players
            .iter()
            .flat_map(|p| p.unknown_medal_codes.iter())
            .copied()
            .collect::<Vec<_>>(),
        [255]
    );
}
