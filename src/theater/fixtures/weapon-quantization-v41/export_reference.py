"""Execute unchanged weapon components/quantization and the complete ammo reader."""
import io
import json
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

PIN = '8f0f8d078e513c800baa21fac416fd4c0c127d03'
BLOBS = {
    'src/components.rs': '6acd5fbb992f7d65ff8fdbd88a6b7b739a88a1a4',
    'src/ammo.rs': 'd1d7b803a5ec6eb72bc2b931a3db5b553dadb9e8',
}
rows = ['weapon-state-ammo', 'weapon-state-overheated', 'weapon-state-type-info',
        'weapon-state-overheated', 'weapon-state-ammo', 'weapon-state-overheated',
        'weapon-state-type-info', 'weapon-state-ammo', 'weapon-state-overheated',
        'weapon-state-ammo', 'weapon-state-type-info', 'weapon-state-type-info']
updates = [
    [(0, [0, 0]), (1, [0, 0]), (3, [1, 1]), (4, [255, 4095]),
     (5, [126, 2]), (7, [None, 1]), (8, [127, 3]), (9, [12, 2048])],
    [(1, [63, 1])],
    [(1, [63, 1]), (9, [12, 2048])],
    [(4, [None, None])],
    [(4, [0, 0])],
    [],
    [(5, [64, 3]), (7, [2, 17])],
    [(8, [0, 2]), (9, [255, 4095])],
    [(0, [None, 4094]), (3, [2, 0])],
]
cases = []
for frame, entries in enumerate(updates):
    bits = ''
    for index, values in entries:
        if rows[index] == 'weapon-state-ammo':
            for value, width in zip(values, [8, 12]):
                bits += '1' if value is None else '0' + format(value, f'0{width}b')
        else:
            bits += format(values[0], '07b') + format(values[1], '02b')
    size = (len(bits) + 7) // 8
    cases.append(dict(frame=frame, entries=entries, list=[i for i, _ in entries],
                      hex=int(bits.ljust(size * 8, '0') or '0', 2).to_bytes(size, 'big').hex(),
                      bit_len=len(bits)))

for path, blob in BLOBS.items():
    actual = subprocess.check_output(['git', '-C', sys.argv[1], 'rev-parse', f'{PIN}:{path}'], text=True).strip()
    assert actual == blob, (path, actual)
with tempfile.TemporaryDirectory(prefix='halo-weapon-quantization-oracle-') as tmp:
    root = Path(tmp)
    archive = subprocess.check_output(['git', '-C', sys.argv[1], 'archive', PIN])
    with tarfile.open(fileobj=io.BytesIO(archive)) as archive:
        archive.extractall(root, filter='data')
    (root / 'Cargo.lock').unlink(missing_ok=True)
    (root / 'src/weapon-cases.json').write_text(json.dumps(dict(rows=rows, cases=cases)))
    (root / 'src/weapon_quantization_oracle.rs').write_text(r'''
#[test]
fn export_weapon_quantization_oracle() {
    use crate::{components::{Table, Params}, payload::{EntityOp, OpKind, Payload}, types::PlayerPath};
    let params = Params::new([15,15,17]);
    let bytes = |value: u32, width: u32| {
        let size = width.div_ceil(8);
        let word = value << (size * 8 - width);
        (0..size).rev().map(|i| (word >> (i * 8)) as u8).collect::<Vec<_>>()
    };
    let charge_table = Table::from_rows(&["weapon-state-ammo".into()], 35);
    let heat_table = Table::from_rows(&["weapon-state-overheated".into()], 35);
    let mut charge = Vec::new();
    for q in 0..4096 {
        // Closed magazine gate, open charge gate, then the twelve-bit word.
        let parsed = crate::components::parse(&bytes((1 << 13) | q, 14), 0, &[0], &charge_table, &params);
        assert_eq!(parsed.end, Some(14));
        let (rounds, v) = parsed.fields.loaded[0].unwrap();
        assert_eq!(rounds, 0);
        let v = v.unwrap();
        charge.push((v.to_bits(), (v * 100.0).round() as u8));
    }
    let mut heat = Vec::new();
    for q in 0..128 {
        for flags in 0..4 {
            let parsed = crate::components::parse(&bytes((q << 2) | flags, 9), 0, &[0], &heat_table, &params);
            assert_eq!(parsed.end, Some(9));
            let (v, actual_flags) = parsed.fields.heat[0].unwrap();
            assert_eq!(actual_flags, flags);
            if flags == 0 { heat.push((v.to_bits(), (v * 100.0).round() as u8)); }
        }
    }
    let input: serde_json::Value = serde_json::from_str(include_str!("weapon-cases.json")).unwrap();
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
        let parsed = crate::components::parse(&data, 0, &list, &table, &params);
        assert_eq!(parsed.end, Some(case["bit_len"].as_i64().unwrap()));
        let mut result = case.clone();
        result["charge_bits"] = serde_json::json!(parsed.fields.loaded.map(|v| v.and_then(|(_, q)| q.map(f64::to_bits))));
        result["heat_bits"] = serde_json::json!(parsed.fields.heat.map(|v| v.map(|(q, flags)| (q.to_bits(), flags))));
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
    println!("ORACLE\t{}", serde_json::json!({ "charge": charge, "heat": heat, "rows": rows,
        "cases": cases, "ammo": out[0].ammo, "energy": out[0].energy, "heat_history": out[0].heat }));
}
''')
    with (root / 'src/lib.rs').open('a') as f:
        f.write('\n#[cfg(test)] mod weapon_quantization_oracle;\n')
    output = subprocess.check_output([
        'cargo', '+stable', 'test', '--offline', '--config', 'build.incremental=false',
        '--lib', 'export_weapon_quantization_oracle', '--', '--nocapture'], cwd=root, text=True)
    records = [json.loads(line.split('\t', 1)[1]) for line in output.splitlines()
               if line.startswith('ORACLE\t')]
    assert len(records) == 1
    print(json.dumps(dict(commit=PIN, blobs=BLOBS, **records[0]), indent=2))
