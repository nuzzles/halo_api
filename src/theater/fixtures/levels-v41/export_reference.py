"""Export all pinned upstream level boxes with exact f32 bit patterns."""
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
PIN='f050371a6cda83b9887eeb83a1884336fce28e53'
archive=subprocess.check_output(['git','-C',sys.argv[1],'archive',PIN])
with tempfile.TemporaryDirectory(prefix='halo-level-catalogue-') as tmp:
    root=Path(tmp)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:tar.extractall(root,filter='data')
    (root/'Cargo.lock').unlink(missing_ok=True)
    path=root/'src/game/levels.rs'
    with path.open('a') as out:out.write(r'''
#[cfg(test)] mod catalogue_oracle {
    use super::*;
    #[test] fn export_catalogue() {
        println!("BUILD\t{}",catalogue_build());
        for l in levels() {
            println!("LEVEL\t{}\t{}\t{}\t{}",l.id.0,l.name,l.structure.0,l.index_bits);
            println!("META\t{}\t{:?}",l.display_name.unwrap_or(""),l.api_id.map(|g|g.0));
            for b in std::iter::once(&l.play).chain(l.others) {
                println!("BOX\t{}\t{:?}\t{:?}\t{:?}",b.index,b.min.map(f32::to_bits),b.max.map(f32::to_bits),b.widths);
            }
        }
    }
}
''')
    output=subprocess.check_output(['cargo','+stable','test','--offline','--lib','export_catalogue','--','--nocapture'],cwd=root,text=True)
rows=[];build=None
for line in output.splitlines():
    if line.startswith('BUILD\t'):build=line.split('\t')[1]
    elif line.startswith('LEVEL\t'):
        _,tag,module,structure,index_bits=line.split('\t');rows.append(dict(tag=int(tag),module=module,structure_tag=int(structure),index_bits=int(index_bits),regions=[]))
    elif line.startswith('META\t'):
        _,display,api=line.split('\t');rows[-1]['display_name']=display or None;rows[-1]['api_id']=None if api=='None' else json.loads(api.removeprefix('Some(').removesuffix(')'))
    elif line.startswith('BOX\t'):
        _,index,low,high,widths=line.split('\t');rows[-1]['regions'].append(dict(index=int(index),min_bits=json.loads(low),max_bits=json.loads(high),axis_bits=json.loads(widths)))
assert build and len(rows)==33
print(json.dumps(dict(commit=PIN,build=build,levels=rows),indent=2))
