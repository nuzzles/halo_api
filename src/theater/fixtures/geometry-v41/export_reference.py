"""Execute the unchanged pinned Bits/shape_at and CBv2 reader on authored bodies."""
import hashlib,json,re,subprocess,sys,tempfile
from pathlib import Path
PIN='8f0f8d078e513c800baa21fac416fd4c0c127d03'
BLOBS={'src/zones.rs':'4701db66d587260404006301be25b4fb2ae1931f','src/bits.rs':'5d1a69e536e63ce03c366aee8ee144c8ed48ee72','src/calibrate.rs':'34641c30d1e88a6b74405023bb87a8a0805adb6f'}
sources={}
for name,expected in BLOBS.items():
 b=(Path(sys.argv[1])/name).read_bytes();assert hashlib.sha1(f'blob {len(b)}\0'.encode()+b).hexdigest()==expected
 sources[name]=b.decode()
s=sources['src/zones.rs']
section=s[s.index('struct Bits<'):s.index('/// A filter\'s clauses')]+s[s.index('/// A zone\'s shape in world units'):s.index('/// A crate\'s orientation')]
constant=re.search(r'pub\(crate\) const WORLD_PER_ENGINE: f64 = [^;]+;',sources['src/calibrate.rs']).group()
main=r'''
use std::io::BufRead;
fn main(){for l in std::io::stdin().lock().lines(){let l=l.unwrap();let f:Vec<_>=l.split('\t').collect();let b:Vec<_>=f[1].as_bytes().chunks(2).map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect();
match shape_at(&b,0){None=>println!("{}\tNone",f[0]),Some(s)=>println!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",f[0],s.cylinder,s.a.to_bits(),s.b.to_bits(),s.top.to_bits(),s.bottom.to_bits(),s.angle.to_bits(),s.team.map(|t|t.to_string()).unwrap_or("None".into()))};}}
'''
cases=json.loads(Path(__file__).with_name('cases.json').read_text())
with tempfile.TemporaryDirectory(prefix='halo-geometry-oracle-') as tmp:
 p=Path(tmp);(p/'bits.rs').write_text(sources['src/bits.rs']);(p/'main.rs').write_text('mod bits;mod calibrate{'+constant+'}\n'+section+main)
 subprocess.run(['rustc','--edition=2024','-Awarnings',str(p/'main.rs'),'-o',str(p/'oracle')],check=True)
 output=subprocess.run([str(p/'oracle')],input=''.join(c['name']+'\t'+c['hex']+'\n' for c in cases),capture_output=True,text=True,check=True).stdout
rows=[]
for line in output.splitlines():
 f=line.split('\t');row=dict(name=f[0],shape=None)
 if f[1]!='None':row['shape']=dict(cylinder=f[1]=='true',a=int(f[2]),b=int(f[3]),top=int(f[4]),bottom=int(f[5]),angle=int(f[6]),team=None if f[7]=='None' else int(f[7]))
 rows.append(row)
assert len(rows)==len(cases)
print(json.dumps(dict(commit=PIN,blobs=BLOBS,cases=rows),indent=2))
