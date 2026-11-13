"""Run unchanged pinned damage/sandbox lookups and helpers on every source row.

python3 export_reference.py LIBFILMSHELL > reference.json
Only test serialization is added. Floating-point values are exported as their
exact IEEE754 f64 bit patterns, not rounded or reinterpreted as f32 game values.
"""
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

PIN = 'f050371a6cda83b9887eeb83a1884336fce28e53'
BLOBS = {'damage.rs': '99446dd1f29d1fc6d5700358c76dd62f437d49f1',
         'damage_data.rs': 'fd16b00a4e46f17fd4528184269a81405f07f6b9',
         'sandbox.rs': '6c029a10256c4dfd2bf6856282bfa26de8301ee8',
         'sandbox_data.rs': '55bdcd998e78b364233d5a55d8f13f059f458309'}
DAMAGE = r'''
#[cfg(test)] mod reference_export {
use super::*;
#[test] fn export_reference_damage(){
 use std::io::Write;
 let mut output=std::fs::File::create(std::path::Path::new(&std::env::var("HALO_REFERENCE_OUTPUT").unwrap()).join("damage.jsonl")).unwrap();
 macro_rules! println { ($($arg:tt)*) => { writeln!(output, $($arg)*).unwrap() }; }
 for input in DAMAGE {
  let d=damage_effect(input.tag).unwrap();assert_eq!(input,d);
  let weapon=d.weapon.map_or("null".into(),|v|v.get().to_string());
  let amount=d.amount.map_or("null".into(),|a|format!(
   "{{\"bits\":{:?},\"flags\":{},\"fixed\":{},\"has\":{:?}}}",
   [a.lower,a.upper[0],a.upper[1],a.shield_scale,a.body_scale,a.radius].map(f64::to_bits),
   a.flags,a.fixed(),(0..32).map(|b|a.has(1<<b)).collect::<Vec<_>>()));
  println!("DAMAGE\t{{\"tag\":{},\"cause\":\"{:?}\",\"weapon\":{weapon},\"amount\":{amount},\"grenade_blast\":{},\"projectiles\":{:?}}}",
   d.tag.get(),d.cause,d.grenade_blast,d.projectiles().map(|v|v.get()).collect::<Vec<_>>());
 }
 let (body,shield)=spartan_vitality();println!("SPARTAN\t{:?}",[body.to_bits(),shield.to_bits()]);
 println!("FLAGS\t{:?}",[DamageAmount::CAN_HEADSHOT,DamageAmount::SKIPS_SHIELDS,DamageAmount::NO_SPILL]);
}
}
'''
SANDBOX = r'''
#[cfg(test)] mod reference_export {
use super::*;
#[test] fn export_reference_sandbox(){
 use std::io::Write;
 let mut output=std::fs::File::create(std::path::Path::new(&std::env::var("HALO_REFERENCE_OUTPUT").unwrap()).join("sandbox.jsonl")).unwrap();
 macro_rules! println { ($($arg:tt)*) => { writeln!(output, $($arg)*).unwrap() }; }
 for input in FIRE {
  let f=fire_stats(input.tag).unwrap();assert_eq!(input,f);
  let magazine=f.magazine.map_or("null".into(),|v|format!("{v:?}"));
  let projectile=f.projectile.map_or("null".into(),|v|v.get().to_string());
  println!("FIRE\t{{\"tag\":{},\"bits\":{:?},\"burst\":{},\"magazine\":{magazine},\"projectile\":{projectile}}}",
   f.tag.get(),[f.rounds_per_second.to_bits(),f.recovery.to_bits()],f.burst);
 }
 for input in PROJECTILES {
  let p=projectile_stats(input.tag).unwrap();assert_eq!(input,p);
  let impact=p.impact.map_or("null".into(),|v|v.get().to_string());
  println!("PROJECTILE\t{{\"tag\":{},\"bits\":{:?},\"impact\":{impact}}}",
   p.tag.get(),[p.velocity.to_bits(),p.range.to_bits()]);
 }
 for &(tag,input) in VEHICLE_BOUNDS {
  let b=vehicle_bounds(tag).unwrap();assert_eq!(b,input);
  println!("BOUNDS\t[{}, {:?}]",tag.get(),b.map(f64::to_bits));
 }
}
}
'''
archive = subprocess.check_output(['git', '-C', sys.argv[1], 'archive', PIN])
with tempfile.TemporaryDirectory(prefix='halo-damage-sandbox-') as tmp:
    root = Path(tmp)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        tar.extractall(root, filter='data')
    (root / 'Cargo.lock').unlink(missing_ok=True)
    for name, blob in BLOBS.items():
        raw = (root / 'src/game' / name).read_bytes()
        assert hashlib.sha1(f'blob {len(raw)}\0'.encode() + raw).hexdigest() == blob
    for name, extra in [('damage.rs', DAMAGE), ('sandbox.rs', SANDBOX)]:
        path = root / 'src/game' / name
        path.write_text(path.read_text() + extra)
    subprocess.run(['cargo', 'test', '--offline', '--lib', 'export_reference_'],
                   cwd=root, check=True, stdout=subprocess.DEVNULL,
                   env={**os.environ, 'HALO_REFERENCE_OUTPUT': str(root)})
    output = '\n'.join((root / name).read_text() for name in ['damage.jsonl', 'sandbox.jsonl'])
rows = {name: [] for name in ['DAMAGE', 'FIRE', 'PROJECTILE', 'BOUNDS', 'SPARTAN', 'FLAGS']}
for line in output.splitlines():
    key, _, value = line.partition('\t')
    if key in rows:
        rows[key].append(json.loads(value))
assert all(rows.values()) and len(rows['SPARTAN']) == len(rows['FLAGS']) == 1
print(json.dumps(dict(commit=PIN, blobs=BLOBS, **{k.lower(): v for k, v in rows.items()}), indent=2))
