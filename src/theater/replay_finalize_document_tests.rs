use super::*;
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
struct Case {
    before: ReplayDocument,
    after: ReplayDocument,
    logs: Vec<serde_json::Value>,
    hits: Vec<FallbackTrigger>,
    counter_present: bool,
    missing_coverage: bool,
    grenade: ReplayLayerCoverage,
    mask: u32,
}
#[test]
fn native_document_finalization_and_signed_fallbacks() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/finalize-document-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut wide = 0;
    for (i, c) in rows.into_iter().enumerate() {
        let counter = FallbackCounter::default();
        counter.accumulate(&c.hits);
        let counter = c.counter_present.then_some(&counter);
        let m = c.mask;
        let layers = ReplayLayerInputs {
            map_quant: m & 2 != 0,
            inventory: m & 4 != 0,
            ability_impulses_scanned: m & 8 != 0,
            ability_charges_scanned: m & 16 != 0,
            score: m & 32 != 0,
            flag_scanned: m & 64 != 0,
            vip_scanned: m & 128 != 0,
            skull_scanned: m & 256 != 0,
            bomb_scanned: m & 512 != 0,
            bomb_carry_scanned: m & 1024 != 0,
            zone_scanned: m & 2048 != 0,
            vehicles_scanned: m & 4096 != 0,
        };
        let mut doc = c.before;
        let logs = super::log_test_support::capture_logs(|| {
            if c.missing_coverage {
                attach_replay_fallback_coverage(&mut doc, counter);
                doc.publish_layers(layers);
            } else {
                finalize_replay_document(&mut doc, &c.grenade, counter, layers);
            }
        });
        assert_eq!(doc, c.after, "document {i}");
        assert_eq!(logs, c.logs, "logs {i}");
        if let Some(coverage) = &doc.coverage {
            wide += coverage
                .fallbacks
                .iter()
                .filter(|h| h.hits > u32::MAX as i64)
                .count();
        }
        // The convenience aggregator must preserve the same native signed domain.
        let mut cov = ReplayCoverage::default();
        cov.set_fallbacks(c.hits.into_iter().map(|h| ReplayFallbackHit {
            name: h.name,
            hits: h.hits,
        }));
        let expected = if let Some(counter) = counter {
            counter.report()
        } else {
            Vec::new()
        };
        if c.counter_present {
            assert_eq!(
                cov.fallbacks,
                expected
                    .into_iter()
                    .map(|h| ReplayFallbackHit {
                        name: h.name,
                        hits: h.hits
                    })
                    .collect::<Vec<_>>(),
                "aggregation {i}"
            );
        }
    }
    assert!(wide > 0);
}
