"""Run unchanged pinned splash::text and Value::arg on independently authored bytes.
Minimal container/payload types only satisfy unused traversal code; they do not
parse, recover, group, or synthesize a message. Optional input is a capture JSON.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

PIN='18067987ebc3628ad499c819c6aee814692df43b'
BLOBS={'splash.rs':'76549535b70693bac166a8f4ee961b2b99a5800e','bits.rs':'5d1a69e536e63ce03c366aee8ee144c8ed48ee72'}
sources={}
for name,blob in BLOBS.items():
    b=(Path(sys.argv[1])/'src'/name).read_bytes()
    assert hashlib.sha1(f'blob {len(b)}\0'.encode()+b).hexdigest()==blob
    sources[name]=b
WRAPPER=b'''
pub fn export(b:&[u8]) {
    let Some((key,params,values))=text(b,0) else {println!("absent");return;};
    println!("key\\t{key}");
    for (p,v) in params.iter().zip(values) {
        match v {
            Some(v) => match v.arg() {
                Arg::Number(n)=>println!("number\\t{}",n.to_bits()),
                Arg::Key(k)=>println!("text\\t{k}"),
            },
            None => if *p>=0 {println!("player\\t{p}")} else {println!("unknown")},
        }
    }
}
'''
MAIN=r'''
mod bits;
mod splash;
mod container {
    pub struct Film;
    pub struct Frame<'a>{pub index:usize,pub chunk:&'a [u8]}
    impl Film {pub fn frames(&self)->std::iter::Empty<Frame<'_>>{std::iter::empty()}}
}
mod payload {
    pub struct Payload {pub ops:Vec<Op>}
    pub struct Op {pub entity_type:Option<u32>,pub kind:OpKind}
    pub enum OpKind {Update{fields:Fields},Other}
    pub struct Fields {pub blocks:Vec<(u32,i64)>}
}
use std::io::BufRead;
fn main(){for line in std::io::stdin().lock().lines(){
    let line=line.unwrap();let (name,hex)=line.split_once('\t').unwrap();
    let b:Vec<_>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();
    println!("case\t{name}");splash::export(&b);
}}
'''
cases=json.loads((Path(sys.argv[2]) if len(sys.argv)>2 else Path(__file__).parent/'cases.json').read_text())
with tempfile.TemporaryDirectory(prefix='halo-hud-oracle-') as tmp:
    p=Path(tmp)
    for name,b in sources.items():(p/name).write_bytes(b+(WRAPPER if name=='splash.rs' else b''))
    (p/'main.rs').write_text(MAIN)
    subprocess.run(['rustc','--edition=2024','-Awarnings',str(p/'main.rs'),'-o',str(p/'oracle')],check=True)
    out=subprocess.run([str(p/'oracle')],input=''.join(c['name']+'\t'+c['hex']+'\n' for c in cases),text=True,capture_output=True,check=True).stdout
result=[]
for line in out.splitlines():
    f=line.split('\t')
    if f[0]=='case':result.append(dict(name=f[1],message=None))
    elif f[0]=='absent':pass
    elif f[0]=='key':result[-1]['message']=dict(key=int(f[1]),arguments=[])
    else:result[-1]['message']['arguments'].append([f[0],int(f[1]) if len(f)>1 else None])
print(json.dumps(dict(commit=PIN,source_blobs=BLOBS,cases=result),indent=2))
