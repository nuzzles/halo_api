"""Run the unchanged pinned libfilmshell zone angle helper on authored bytes."""
import hashlib,json,subprocess,sys,tempfile
from pathlib import Path
PIN='8f0f8d078e513c800baa21fac416fd4c0c127d03'
BLOBS={'src/zones.rs':'4701db66d587260404006301be25b4fb2ae1931f','src/bits.rs':'5d1a69e536e63ce03c366aee8ee144c8ed48ee72'}
sources={}
for name,expected in BLOBS.items():
    b=(Path(sys.argv[1])/name).read_bytes()
    assert hashlib.sha1(f'blob {len(b)}\0'.encode()+b).hexdigest()==expected
    sources[name]=b.decode()
s=sources['src/zones.rs']
section=s[s.index('struct Bits<'):s.index("/// A filter's clauses")]+s[s.index("/// A crate's orientation"):s.index("/// One zone's state")]
main=r'''
use std::io::BufRead;
fn main(){for l in std::io::stdin().lock().lines(){let l=l.unwrap();let f:Vec<_>=l.split('\t').collect();let b:Vec<_>=f[1].as_bytes().chunks(2).map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect();
println!("{}\t{}",f[0],angle_at(&b,0).map(|a|a.to_bits().to_string()).unwrap_or("None".into()));}}
'''
cases=json.loads(Path(__file__).with_name('cases.json').read_text())
with tempfile.TemporaryDirectory(prefix='halo-orientation-reference-') as tmp:
    p=Path(tmp);(p/'bits.rs').write_text(sources['src/bits.rs']);(p/'main.rs').write_text('mod bits;\n'+section+main)
    subprocess.run(['rustc','--edition=2024','-Awarnings',str(p/'main.rs'),'-o',str(p/'oracle')],check=True)
    out=subprocess.run([str(p/'oracle')],input=''.join(c['name']+'\t'+c['hex']+'\n' for c in cases),capture_output=True,text=True,check=True).stdout
rows=[]
for line in out.splitlines():
    name,value=line.split('\t');rows.append(dict(name=name,angle_bits=None if value=='None' else int(value)))
assert len(rows)==len(cases)
print(json.dumps(dict(commit=PIN,blobs=BLOBS,cases=rows),indent=2))
