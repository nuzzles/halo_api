"""Unchanged pinned energy/delay readers and complete equipment ammo projection."""
import io, json, subprocess, sys, tarfile, tempfile
from pathlib import Path
PIN = '8f0f8d078e513c800baa21fac416fd4c0c127d03'
cases = []
for energy in [0, 1, 2, 81, 82, 8191, 8192, 16381, 16382, 16383]:
    for delay in [0, 1023]:
        cases.append((energy, delay))
cases.extend([(8192, 10), (None, 9), (None, 8), (8192, None), (0, 0), (0, 0)])
rows = ['equipment-energy-component', 'equipment-energy-delay-ticks-left-component']
inputs = []
for frame, pair in enumerate(cases):
    bits = ''.join(format(value, f'0{width}b') for value, width in zip(pair, [14, 10]) if value is not None)
    length = (len(bits) + 7) // 8
    inputs.append(dict(frame=frame, values=pair, list=[i for i, v in enumerate(pair) if v is not None],
                       hex=int(bits.ljust(length * 8, '0'), 2).to_bytes(length, 'big').hex(), bit_len=len(bits)))
with tempfile.TemporaryDirectory(prefix='halo-equipment-oracle-') as tmp:
    root = Path(tmp)
    archive = subprocess.check_output(['git', '-C', sys.argv[1], 'archive', PIN])
    with tarfile.open(fileobj=io.BytesIO(archive)) as archive:
        archive.extractall(root, filter='data')
    (root/'Cargo.lock').unlink(missing_ok=True)
    (root/'src/equipment-cases.json').write_text(json.dumps(dict(rows=rows, cases=inputs)))
    (root/'src/equipment_oracle.rs').write_text(r'''
#[test]
fn export_equipment_oracle() {
    use crate::{components::{Table, Params, Fields, Parent}, payload::{EntityOp, OpKind, Payload}, types::PlayerPath};
    let input: serde_json::Value = serde_json::from_str(include_str!("equipment-cases.json")).unwrap();
    let rows: Vec<String> = serde_json::from_value(input["rows"].clone()).unwrap();
    let table = Table::from_rows(&rows, 37);
    let paths = [PlayerPath { player_index: 7, is_bot: false, team_id: None,
        positions: vec![], life_slots: Some(vec![0]), life_starts: None }];
    let op = |index, class, fields| EntityOp { at: 0, index, generation: 1, entity_type: Some(class),
        kind: OpKind::Update { list: vec![], fields }, init: None, shooter: (None, None) };
    let mut out = Vec::new();
    let mut read = crate::ammo::reader(&paths, &mut out);
    read(0, &[], &Payload { ops: vec![op(512, 35, Fields::default())], ..Default::default() });
    let mut cases = Vec::new();
    for case in input["cases"].as_array().unwrap() {
        let data: Vec<u8> = case["hex"].as_str().unwrap().as_bytes().chunks(2)
            .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap()).collect();
        let list: Vec<u32> = serde_json::from_value(case["list"].clone()).unwrap();
        let mut parsed = crate::components::parse(&data, 0, &list, &table, &Params::new([15,15,17]));
        assert_eq!(parsed.end, Some(case["bit_len"].as_i64().unwrap()));
        let mut result = case.clone();
        result["energy_bits"] = serde_json::json!(parsed.fields.energy.map(f64::to_bits));
        result["percent"] = serde_json::json!(parsed.fields.energy.map(|v| (v * 100.0).round() as u32));
        result["delay"] = serde_json::json!(parsed.fields.recharge);
        result["end"] = serde_json::json!(parsed.end);
        cases.push(result);
        parsed.fields.parent = Some(Some(Parent::Unit(0)));
        read(case["frame"].as_u64().unwrap() as usize, &data, &Payload {
            ops: vec![op(1024, 37, parsed.fields)], ..Default::default()
        });
    }
    drop(read);
    println!("ORACLE\t{}", serde_json::json!({ "cases": cases, "equipment": out[0].equipment }));
}
''')
    with (root/'src/lib.rs').open('a') as f:
        f.write('\n#[cfg(test)] mod equipment_oracle;\n')
    output = subprocess.check_output(['cargo', '+stable', 'test', '--offline', '--config', 'build.incremental=false',
                                     '--lib', 'export_equipment_oracle', '--', '--nocapture'], cwd=root, text=True)
    records = [json.loads(line.split('\t', 1)[1]) for line in output.splitlines() if line.startswith('ORACLE\t')]
    assert len(records) == 1
    print(json.dumps(dict(commit=PIN, **records[0]), indent=2))
