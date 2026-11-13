"""Run unchanged pinned script search and mode classification, never halo_api.
Usage: export_reference.py PINNED_LIBFILMSHELL [CASES_OR_CAPTURED_EXPORT]
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

PIN='f050371a6cda83b9887eeb83a1884336fce28e53'
BLOBS={'src/scripts.rs':'005e1bcc84512b5d09a1919609961dedab70a32b',
       'src/model/variant.rs':'e07071372a506302f7def1a6e7afd488539fe704',
       'src/game/options.rs':'43ade2aafef5f38ca492b507e7adce03cc4be38d',
       'src/game/options_data.rs':'021e911b805a2b74c24321c7209462a76bd0a708'}
sources={}
for name,expected in BLOBS.items():
    b=(Path(sys.argv[1])/name).read_bytes()
    assert hashlib.sha1(f'blob {len(b)}\0'.encode()+b).hexdigest()==expected,name
    sources[name]=b
HARNESS=r'''
mod scripts;mod variant;mod game;
#[derive(Debug,Clone,Copy,Default,PartialEq,Eq)]pub struct TagId(pub u32);
use std::io::BufRead;
fn export(prefix:&str, chunk:&[u8]) {
    let names=scripts::script_names(chunk);
    println!("{prefix}mode\t{:?}",variant::GameMode::from_scripts(&names));
    for name in names { println!("{prefix}name\t{name}"); }
}
fn main() {
    for line in std::io::stdin().lock().lines() {
        let line=line.unwrap();let fields:Vec<_>=line.split('\t').collect();
        let hex=fields[1];let b:Vec<_>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();
        // Adapt only the original reader's fixed shift and exclusive endpoint.
        // Two zero bytes after the authored blob make its final byte observable.
        let mut chunk=vec![0u8;b.len()+3];
        for (i,&byte) in b.iter().enumerate() { chunk[i]|=byte>>3;chunk[i+1]|=byte<<5; }
        println!("case\t{}",fields[0]);export("",&chunk);
        if fields.len()==3 {export("registry-",&std::fs::read(fields[2]).unwrap());}
    }
}
'''
cases=json.loads((Path(sys.argv[2]) if len(sys.argv)>2 else Path(__file__).parent/'cases.json').read_text())
with tempfile.TemporaryDirectory(prefix='halo-script-oracle-') as tmp:
    p=Path(tmp);(p/'game').mkdir()
    for name,target in [('src/scripts.rs','scripts.rs'),('src/model/variant.rs','variant.rs'),
                        ('src/game/options.rs','game/options.rs'),('src/game/options_data.rs','game/options_data.rs')]:
        (p/target).write_bytes(sources[name])
    (p/'game/mod.rs').write_text('mod options;mod options_data;pub use options::{OptionValue,modes,murmur3};')
    (p/'main.rs').write_text(HARNESS)
    subprocess.run(['rustc','--edition=2024','-Awarnings',str(p/'main.rs'),'-o',str(p/'oracle')],check=True)
    data=''.join(c['name']+'\t'+c.get('script_blob_hex',c.get('blob_hex',''))+
                 ('\t'+c['registry_path'] if 'registry_path' in c else '')+'\n' for c in cases)
    output=subprocess.run([str(p/'oracle')],input=data,text=True,capture_output=True,check=True).stdout
out=[]
for line in output.splitlines():
    kind,value=line.split('\t')
    if kind=='case':out.append(dict(name=value,names=[],registry_names=[]))
    elif kind=='name':out[-1]['names'].append(value)
    elif kind=='mode':out[-1]['mode']=value
    elif kind=='registry-name':out[-1]['registry_names'].append(value)
    elif kind=='registry-mode':out[-1]['registry_mode']=value
    else:raise ValueError(kind)
print(json.dumps(dict(commit=PIN,blobs=BLOBS,cases=out),indent=2))
