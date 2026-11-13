use crate::{components, container, header, levels, payload, snapshot};
use std::{collections::BTreeMap, fs, path::Path};

#[test]
fn halo_api_study_probe() {
    let input_root =
        std::env::var("LIBFILMSHELL_STUDY_INPUTS").expect("set LIBFILMSHELL_STUDY_INPUTS");
    let root = Path::new(&input_root);
    let mut dirs: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    assert_eq!(dirs.len(), 15, "the study requires all 15 prepared samples");
    for dir in dirs {
        let h = fs::read(dir.join("header.bin")).unwrap();
        let c = fs::read(dir.join("replay.bin")).unwrap();
        let head = header::read(&h);
        let level = head
            .level
            .and_then(levels::level)
            .expect("sample level must be catalogued");
        let mut p = components::Params::new(level.widths);
        p.set_level(Some(level));
        let packets = container::walk_packets(&c).unwrap();
        let ctx = snapshot::Context::new(&h, p, p.box_bits);
        let pk = packets.iter().find(|x| x.kind == 2).unwrap();
        let w = snapshot::walk(&c, pk.start, pk.end, &ctx);
        let seeds = snapshot::entities(&c, pk.start, pk.end, &ctx);
        println!("STUDY {} level={:08x} widths={:?} snapshot_sequential_entities={} snapshot_stop={:?} snapshot_with_recovery_entities={}", dir.file_name().unwrap().to_string_lossy(), head.level.unwrap(), level.widths, w.entities.len(), w.stopped, seeds.len());
        let mut walker = payload::Walker::new(&h, p);
        walker.seed(&seeds);
        let mut counts = BTreeMap::<String, usize>::new();
        let mut controls = BTreeMap::<u32, usize>::new();
        let mut extra = 0;
        for pk in packets.iter().filter(|x| x.kind == 0) {
            *counts.entry("frames".into()).or_default() += 1;
            let q = walker.walk(&c, pk.start, pk.end);
            if let Some(s) = q.event_stop {
                *counts.entry(format!("event_stop:{s:?}")).or_default() += 1;
            }
            if !q.skipped.is_empty() {
                *counts.entry("tick_resumed_frames".into()).or_default() += 1;
            }
            if let Some(s) = q.stop {
                *counts.entry(format!("entity_stop:{s:?}")).or_default() += 1;
            } else {
                *counts.entry("entity_terminated".into()).or_default() += 1;
                let (messages, end) = components::control_view(&c, q.end, &p);
                if end.is_some_and(|e| (0..8).contains(&(pk.end as i64 * 8 - e))) {
                    *counts
                        .entry("control_reaches_last_byte".into())
                        .or_default() += 1;
                    if q.skipped.is_empty() {
                        *counts
                            .entry("control_endpoint_without_tick_resume".into())
                            .or_default() += 1;
                    }
                    for m in messages {
                        if m.command_at.is_some() {
                            *controls.entry(1).or_default() += 1;
                        } else {
                            *controls.entry(0).or_default() += 1;
                        }
                        extra += usize::from(m.state.is_some());
                    }
                }
            }
        }
        println!("STUDY counts={counts:?} control_messages={controls:?} secondary_blocks={extra}");
    }
}
