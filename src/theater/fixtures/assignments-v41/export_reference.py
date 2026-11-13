"""Call unchanged original roster scanners; stubs only satisfy unused traversal."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

PIN='8f0f8d078e513c800baa21fac416fd4c0c127d03'
BLOBS={'roster.rs':'a32657daa1b5969321b96895b683c1f8c5b7df8f','bits.rs':'5d1a69e536e63ce03c366aee8ee144c8ed48ee72'}
sources={}
for name,blob in BLOBS.items():
    b=(Path(sys.argv[1])/'src'/name).read_bytes()
    assert hashlib.sha1(f'blob {len(b)}\0'.encode()+b).hexdigest()==blob
    sources[name]=b
WRAPPER=b'''
pub fn export(b:&[u8],bots:bool) {
    if bots {for (id,bot,_) in bot_records(b){println!("row\\t{id}\\t{bot}");}}
    else {for (id,xuid) in struct_players(b){println!("row\\t{id}\\t{xuid}");}}
}
'''
MAIN=r'''
mod bits;
mod roster;
mod container {
    pub struct Film {pub chunks:Vec<Vec<u8>>}
    pub struct Packet {pub kind:u16,pub start:usize,pub end:usize}
    impl Film {pub fn packets(&self,_:usize)->Option<Vec<Packet>>{None}}
}
mod highlights {pub struct HighlightEvent {pub xuid:String,pub team:u32,pub gamertag:String}}
mod identity {
    pub struct Player {pub xuid:String,pub gamertag:String}
    pub fn extract_roster(_: &[u8])->Vec<Player>{Vec::new()}
}
use std::io::BufRead;
fn main(){for line in std::io::stdin().lock().lines(){
    let line=line.unwrap();let f:Vec<_>=line.split('\t').collect();
    let b:Vec<_>=(0..f[2].len()).step_by(2).map(|i|u8::from_str_radix(&f[2][i..i+2],16).unwrap()).collect();
    println!("case\t{}",f[0]);roster::export(&b,f[1]=="1");
}}
'''
cases=json.loads((Path(sys.argv[2]) if len(sys.argv)>2 else Path(__file__).parent/'cases.json').read_text())
cases=[dict(name=c['Name'],bots=c['Bots'],hex=c['Data']) if 'Name' in c else c for c in cases]
with tempfile.TemporaryDirectory(prefix='halo-assignment-oracle-') as tmp:
    p=Path(tmp)
    for name,b in sources.items():(p/name).write_bytes(b+(WRAPPER if name=='roster.rs' else b''))
    (p/'main.rs').write_text(MAIN)
    subprocess.run(['rustc','--edition=2024','-Awarnings',str(p/'main.rs'),'-o',str(p/'oracle')],check=True)
    out=subprocess.run([str(p/'oracle')],input=''.join(c['name']+'\t'+str(int(c['bots']))+'\t'+c['hex']+'\n' for c in cases),text=True,capture_output=True,check=True).stdout
result=[]
for line in out.splitlines():
    f=line.split('\t')
    if f[0]=='case':result.append(dict(name=f[1],assignments=[]))
    else:result[-1]['assignments'].append(dict(engine=int(f[1]),identity=int(f[2])))
print(json.dumps(dict(commit=PIN,source_blobs=BLOBS,cases=result),indent=2))
