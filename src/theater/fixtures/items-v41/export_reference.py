"""Serialize original libfilmshell catalogue lookups; never reads local tables.

python3 export_reference.py /path/to/libfilmshell > reference.json
The pinned archive receives a test-only serialization adapter. All lookups,
enum names, relation targets and item names execute unchanged upstream code.
"""
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

PIN = 'f050371a6cda83b9887eeb83a1884336fce28e53'
BLOBS = {'src/game/items.rs': '47b901205f61d7cf3e909c8a4baa262413a8b7ad',
         'src/game/items_data.rs': '5e8a63c4e01a4ff07f0b609b9ce7929c49148568'}
archive = subprocess.check_output(['git', '-C', sys.argv[1], 'archive', PIN])
with tempfile.TemporaryDirectory(prefix='halo-item-source-') as tmp:
    root = Path(tmp)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        tar.extractall(root, filter='data')
    (root / 'Cargo.lock').unlink(missing_ok=True)
    for name, blob in BLOBS.items():
        raw = (root / name).read_bytes()
        assert hashlib.sha1(f'blob {len(raw)}\0'.encode() + raw).hexdigest() == blob
    path = root / 'src/game/items.rs'
    path.write_text(path.read_text() + r'''
#[cfg(test)] mod catalogue_export {
use super::*;
#[test] fn export_original_catalogue() {
 for input in tags() {
  let t=tag_info(input.tag).unwrap(); assert_eq!(t,input);
  let name=t.name.map_or("null".to_string(),|x|format!("{x:?}"));
  let item=item(t.tag).map_or("null".to_string(),|x|format!("{:?}",format!("{x:?}")));
  let item_name=t.item.map_or("null".to_string(),|x|format!("{:?}",x.name()));
  let parent=t.parent.map_or("null".to_string(),|x|x.get().to_string());
  let part=t.part_of.map_or("null".to_string(),|x|x.get().to_string());
  println!("TAG\t{{\"tag\":{},\"kind\":\"{:?}\",\"name\":{name},\"item\":{item},\"item_name\":{item_name},\"parent\":{parent},\"part_of\":{part},\"power_weapon\":{}}}",t.tag.get(),t.kind,t.power_weapon);
 }
 for &(id,_) in VARIANTS {
  println!("VARIANT\t[{}, {:?}]",id,weapon_variant_name(id).unwrap());
 }
 for i in 0..=5 {
  println!("GRENADE\t[{}, {}]",i,Grenade::from_type(i).map_or("null".to_string(),|g|format!("[{}, {:?}]",g.type_index(),g.name())));
 }
}
}
''')
    output = subprocess.check_output(['cargo', 'test', '--offline', '--lib',
                                     'export_original_catalogue', '--', '--nocapture'],
                                    cwd=root, text=True)
rows = {key: [] for key in ['TAG', 'VARIANT', 'GRENADE']}
for line in output.splitlines():
    key, _, data = line.partition('\t')
    if key in rows:
        rows[key].append(json.loads(data))
assert rows['TAG'] and rows['VARIANT'] and len(rows['GRENADE']) == 6
print(json.dumps(dict(commit=PIN, blobs=BLOBS, tags=rows['TAG'],
                      variants=rows['VARIANT'], grenades=rows['GRENADE']), indent=2))
