"""Export fields 8–11 by definition row using the original pinned Bond reader.
Usage: export_reference.py PINNED_LIBFILMSHELL [CAPTURED_VARIANTS]
This does not assign individual label roles or rewrite inherited-field policy.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

PIN='f050371a6cda83b9887eeb83a1884336fce28e53'
BLOB='4814607a2a27f4ad2cd4ef0a16845582f73c8172'
source=(Path(sys.argv[1])/'src/variant_stats.rs').read_bytes()
assert hashlib.sha1(f'blob {len(source)}\0'.encode()+source).hexdigest()==BLOB
WRAPPER=b'''
pub fn export(b:&[u8]) {
    let mut p=0;let root=value(b,&mut p,10,0).unwrap();assert_eq!(p,b.len());
    if let Some(list)=root.at(&[3,21,0]) {
        for row in list.list() {
            println!("row\\t{}",(8..12).map(|i|row.int(i).map_or_else(||"-".into(),|n|n.to_string())).collect::<Vec<_>>().join("\\t"));
        }
    }
}
'''
MAIN=r'''
mod native;
use std::io::BufRead;
fn main(){for line in std::io::stdin().lock().lines(){
    let line=line.unwrap();let (name,hex)=line.split_once('\t').unwrap();
    let b:Vec<_>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();
    println!("case\t{name}");native::export(&b);
}}
'''
cases=json.loads((Path(sys.argv[2]) if len(sys.argv)>2 else Path(__file__).parent/'cases.json').read_text())
with tempfile.TemporaryDirectory(prefix='halo-stat-text-oracle-') as tmp:
    p=Path(tmp);(p/'native.rs').write_bytes(source+WRAPPER);(p/'main.rs').write_text(MAIN)
    subprocess.run(['rustc','--edition=2024','-Awarnings',str(p/'main.rs'),'-o',str(p/'oracle')],check=True)
    out=subprocess.run([str(p/'oracle')],input=''.join(c['name']+'\t'+c['base_hex']+'\n' for c in cases),text=True,capture_output=True,check=True).stdout
result=[]
for line in out.splitlines():
    f=line.split('\t')
    if f[0]=='case':result.append(dict(name=f[1],indices=[]))
    elif f[0]=='row':result[-1]['indices'].append([None if n=='-' else int(n) for n in f[1:]])
    else:raise ValueError(f)
print(json.dumps(dict(commit=PIN,source_blob=BLOB,cases=result),indent=2))
