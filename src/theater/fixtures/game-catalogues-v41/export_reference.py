"""Export original libfilmshell medal/option APIs without halo_api code.
python3 export_reference.py PINNED_LIBFILMSHELL > reference.json
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

PIN = 'f050371a6cda83b9887eeb83a1884336fce28e53'
BLOBS = {
    'medals.rs': '024d812c935c36751eaf4579bffd5a36eb32625c',
    'medals_data.rs': '8d0e40d3b075e215b94614aa43bcafbc97554f1c',
    'options.rs': '43ade2aafef5f38ca492b507e7adce03cc4be38d',
    'options_data.rs': '021e911b805a2b74c24321c7209462a76bd0a708',
}
sources = {}
for name, blob in BLOBS.items():
    data = (Path(sys.argv[1]) / 'src/game' / name).read_bytes()
    assert hashlib.sha1(f'blob {len(data)}\0'.encode() + data).hexdigest() == blob
    sources[name] = data

HARNESS = r'''
mod game;
use game::{medals::Medal, options::{self, OptionValue}};
fn main() {
    for index in 0..=65535 {
        if let Some(m) = Medal::from_index(index) {
            assert_eq!(m.index(), index);
            println!("medal\t{}\t{:?}\t{}\t{:?}\t{:?}\t{}\t{}", index, m,
                m.name(), m.kind(), m.difficulty(), m.personal_score(), m.sorting_weight());
        }
    }
    assert!(Medal::from_index(u32::MAX).is_none());
    for &(hash, name) in game::options_data::OPTION_NAMES {
        assert_eq!(options::option_name(hash), Some(name));
        assert_eq!(options::murmur3(name.to_ascii_lowercase().as_bytes()), hash);
        println!("name\t{}\t{}", hash, name);
    }
    for mode in options::modes() {
        println!("mode\t{}\t{}", mode.prefix, mode.script);
        for option in mode.options {
            let default = match option.default {
                None => "-".into(),
                Some(OptionValue::Number(v)) => format!("n:{:016x}", v.to_bits()),
                Some(OptionValue::Bool(v)) => format!("b:{}", u8::from(v)),
            };
            println!("option\t{}\t{}\t{}", option.name, option.keys.join("\u{1f}"), default);
        }
    }
    let mut seed = 0x12345678u32;
    for len in 0..=260 {
        let input: Vec<u8> = (0..len).map(|_| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 24) as u8
        }).collect();
        let hex: String = input.iter().map(|b| format!("{b:02x}")).collect();
        println!("hash\t{}\t{}", hex, options::murmur3(&input));
    }
}
'''

with tempfile.TemporaryDirectory(prefix='halo-game-catalogues-') as tmp:
    root = Path(tmp)
    (root / 'game').mkdir()
    for name, data in sources.items():
        (root / 'game' / name).write_bytes(data)
    (root / 'game/mod.rs').write_text('\n'.join(f'pub mod {Path(name).stem};' for name in sources))
    (root / 'main.rs').write_text(HARNESS)
    subprocess.run([os.environ.get('RUSTC', 'rustc'), '--edition=2024', '-Awarnings',
                    str(root / 'main.rs'), '-o', str(root / 'oracle')], check=True)
    result = subprocess.run([str(root / 'oracle')], text=True, capture_output=True, check=True)

medals, names, modes, hashes = [], [], [], []
for line in result.stdout.splitlines():
    fields = line.split('\t')
    match fields:
        case ['medal', index, variant, name, kind, difficulty, score, weight]:
            medals.append(dict(index=int(index), variant=variant, name=name, kind=kind,
                               difficulty=difficulty, personal_score=int(score), sorting_weight=int(weight)))
        case ['name', value, name]:
            names.append(dict(hash=int(value), name=name))
        case ['mode', prefix, script]:
            modes.append(dict(prefix=prefix, script=script, options=[]))
        case ['option', name, keys, default]:
            value = None if default == '-' else (
                dict(kind='number', bits=default[2:]) if default.startswith('n:')
                else dict(kind='bool', value=default == 'b:1'))
            modes[-1]['options'].append(dict(name=name, keys=keys.split('\x1f'), default=value))
        case ['hash', data, value]:
            hashes.append(dict(hex=data, hash=int(value)))
        case _:
            raise AssertionError(fields)
assert len(medals) == 157 and len(names) == 98 and len(modes) == 12
assert sum(len(m['options']) for m in modes) == 247 and len(hashes) == 261
print(json.dumps(dict(commit=PIN, blobs=BLOBS, medals=medals, option_names=names,
                     modes=modes, murmur3=hashes), indent=2))
