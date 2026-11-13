"""Run original option decoding, GameVariant::rules and Rule::effective.
Usage: export_reference.py PINNED_LIBFILMSHELL [CASES_OR_CAPTURED_EXPORT]
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

PIN='f050371a6cda83b9887eeb83a1884336fce28e53'
BLOBS={'src/variant.rs':'6592ffba89f76bb24d2cc4abcd6c09b33b7cf088',
       'src/model/variant.rs':'e07071372a506302f7def1a6e7afd488539fe704',
       'src/game/options.rs':'43ade2aafef5f38ca492b507e7adce03cc4be38d',
       'src/game/options_data.rs':'021e911b805a2b74c24321c7209462a76bd0a708'}
sources={}
for name,expected in BLOBS.items():
    b=(Path(sys.argv[1])/name).read_bytes()
    assert hashlib.sha1(f'blob {len(b)}\0'.encode()+b).hexdigest()==expected,name
    sources[name]=b
WRAPPER=b'''
pub fn decode(b:&[u8])->Option<OptionValue> {
    option_value(i64::from_le_bytes(b.try_into().ok()?))
}
'''
HARNESS=r'''
mod wire_variant;mod variant;mod game;
#[derive(Debug,Clone,Copy,Default,PartialEq,Eq)]pub struct TagId(pub u32);
use std::io::BufRead;
fn repr(v:Option<game::OptionValue>)->String {
    match v {None=>"-".into(),Some(game::OptionValue::Bool(b))=>format!("b:{}",u8::from(b)),
        Some(game::OptionValue::Number(n))=>format!("n:{:016x}",n.to_bits())}
}
fn main() {
    for line in std::io::stdin().lock().lines() {
        let line=line.unwrap();let fields:Vec<_>=line.split('\t').collect();
        println!("case\t{}",fields[0]);
        let mut v=variant::GameVariant::default();
        if !fields[1].is_empty(){v.scripts=fields[1].split(';').map(str::to_string).collect();}
        for row in fields[2].split(';').filter(|s|!s.is_empty()) {
            let (index,row)=row.split_once(',').unwrap();
            let (hash,hex)=row.split_once(':').unwrap();let hash=hash.parse::<u32>().unwrap();
            let raw:Vec<_>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();
            let value=wire_variant::decode(&raw);println!("decoded\t{index}\t{}",repr(value));
            if let Some(value)=value{v.options.push(variant::VariantOption{hash,name:game::option_name(hash),value});}
        }
        for r in v.rules(){println!("rule\t{}\t{}\t{}\t{}",r.name,repr(r.default),repr(r.value),repr(r.effective()));}
    }
}
'''
cases=json.loads((Path(sys.argv[2]) if len(sys.argv)>2 else Path(__file__).parent/'cases.json').read_text())
with tempfile.TemporaryDirectory(prefix='halo-rules-oracle-') as tmp:
    p=Path(tmp);(p/'game').mkdir()
    for src,dst in [('src/model/variant.rs','variant.rs'),('src/game/options.rs','game/options.rs'),('src/game/options_data.rs','game/options_data.rs')]:
        (p/dst).write_bytes(sources[src])
    (p/'wire_variant.rs').write_bytes(sources['src/variant.rs']+WRAPPER)
    (p/'game/mod.rs').write_text('mod options;mod options_data;pub use options::{OptionValue,modes,murmur3,option_name};')
    harness=HARNESS.replace(
        'name:game::option_name(hash)', 'name:game::option_name(hash).map(str::to_string)')
    (p/'main.rs').write_text(harness)
    subprocess.run(['rustc','--edition=2024','-Awarnings',str(p/'main.rs'),'-o',str(p/'oracle')],check=True)
    data=''.join(c['name']+'\t'+';'.join(c.get('script_names',c.get('scripts',[])))+'\t'+
        ';'.join(str(i)+','+str(o['hash'])+':'+bytes(o['raw']).hex() for i,o in enumerate(c['options']) if o.get('hash') is not None and o.get('raw') is not None)+'\n' for c in cases)
    output=subprocess.run([str(p/'oracle')],input=data,text=True,capture_output=True,check=True).stdout
out=[]
for line in output.splitlines():
    f=line.split('\t')
    if f[0]=='case':out.append(dict(name=f[1],rules=[],decoded_rows={}))
    elif f[0]=='decoded':out[-1]['decoded_rows'][f[1]]=f[2]
    elif f[0]=='rule':out[-1]['rules'].append(dict(name=f[1],default=f[2],value=f[3],effective=f[4]))
    else:raise ValueError(f)
print(json.dumps(dict(commit=PIN,blobs=BLOBS,cases=out),indent=2))
