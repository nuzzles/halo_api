"""Run unchanged pinned component parsing and the complete ammo history reader."""
import io
import json
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

PIN = '8f0f8d078e513c800baa21fac416fd4c0c127d03'
# Interleaved families make absolute component index differ from reserve slot.
rows = ['weapon-state-ammo', 'weapon-state-rounds-inventory',
        'weapon-state-rounds-inventory', 'weapon-state-overheated',
        'weapon-state-rounds-inventory', 'weapon-state-ammo',
        'weapon-state-rounds-inventory']
cases = []
for frame, entries in enumerate([
    [(1, 0), (2, 17), (4, 1024), (6, 2047)],
    [(2, 17)], [(2, 0)], [(6, 1)], [], [(1, 2047)],
    [(1, 2047)], [(4, 0)], [(2, 2047), (6, 0)],
]):
    bits = ''.join(format(v, '011b') for _, v in entries)
    size = (len(bits) + 7) // 8
    data = int(bits.ljust(size * 8, '0') or '0', 2).to_bytes(size, 'big')
    cases.append(dict(frame=frame, list=[i for i, _ in entries],
                      hex=data.hex(), bit_len=len(bits)))

with tempfile.TemporaryDirectory(prefix='halo-reserves-oracle-') as tmp:
    root = Path(tmp)
    archive = subprocess.check_output(['git', '-C', sys.argv[1], 'archive', PIN])
    with tarfile.open(fileobj=io.BytesIO(archive)) as archive:
        archive.extractall(root, filter='data')
    (root / 'Cargo.lock').unlink(missing_ok=True)
    (root / 'src/reserve-cases.json').write_text(json.dumps(dict(rows=rows, cases=cases)))
    (root / 'src/reserve_oracle.rs').write_text(r'''
#[test]
fn export_reserve_oracle() {
    use crate::{components::{Table, Params}, payload::{EntityOp, OpKind, Payload}, types::PlayerPath};
    let input: serde_json::Value = serde_json::from_str(include_str!("reserve-cases.json")).unwrap();
    let rows: Vec<String> = serde_json::from_value(input["rows"].clone()).unwrap();
    let table = Table::from_rows(&rows, 35);
    let paths = [PlayerPath { player_index: 7, is_bot: false, team_id: None,
        positions: vec![], life_slots: Some(vec![0]), life_starts: None }];
    let mut out = Vec::new();
    let mut read = crate::ammo::reader(&paths, &mut out);
    let mut cases = Vec::new();
    for case in input["cases"].as_array().unwrap() {
        let data: Vec<u8> = case["hex"].as_str().unwrap().as_bytes().chunks(2)
            .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap()).collect();
        let list: Vec<u32> = serde_json::from_value(case["list"].clone()).unwrap();
        let parsed = crate::components::parse(&data, 0, &list, &table, &Params::new([15,15,17]));
        assert_eq!(parsed.end, Some(case["bit_len"].as_i64().unwrap()));
        let mut result = case.clone();
        result["reserve"] = serde_json::json!(parsed.fields.reserve);
        result["starts"] = serde_json::json!(parsed.starts);
        result["end"] = serde_json::json!(parsed.end);
        cases.push(result);
        read(case["frame"].as_u64().unwrap() as usize, &data, &Payload {
            ops: vec![EntityOp { at: 0, index: 512, generation: 1, entity_type: Some(35),
                kind: OpKind::Update { list, fields: parsed.fields }, init: None, shooter: (None, None) }],
            ..Default::default()
        });
    }
    drop(read);
    println!("ORACLE\t{}", serde_json::json!({ "rows": rows, "cases": cases, "ammo": out[0].ammo }));
}
''')
    with (root / 'src/lib.rs').open('a') as f:
        f.write('\n#[cfg(test)] mod reserve_oracle;\n')
    output = subprocess.check_output([
        'cargo', '+stable', 'test', '--offline', '--config', 'build.incremental=false',
        '--lib', 'export_reserve_oracle', '--', '--nocapture'], cwd=root, text=True)
    records = [json.loads(line.split('\t', 1)[1]) for line in output.splitlines()
               if line.startswith('ORACLE\t')]
    assert len(records) == 1
    print(json.dumps(dict(commit=PIN, **records[0]), indent=2))
