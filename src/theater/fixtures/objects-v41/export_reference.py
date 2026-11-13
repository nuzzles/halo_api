"""Invoke unchanged world::read and snapshot::raw_position with authored inputs.

The packet/entity/default adapters below supply explicit model inputs. They are
not native packet parsing and cannot establish keyframe identity or completeness.
"""
import hashlib, json, struct, subprocess, sys, tempfile
from pathlib import Path

PIN = '8f0f8d078e513c800baa21fac416fd4c0c127d03'
BLOBS = {'src/world.rs': '6aac5301bee54d94cf150fe2ee8abd15e20dd526',
         'src/snapshot.rs': 'cd8971c6b1e53e593361fe29026324a98a861728'}
sources = {}
for name, expected in BLOBS.items():
    b = (Path(sys.argv[1])/name).read_bytes()
    assert hashlib.sha1(f'blob {len(b)}\0'.encode()+b).hexdigest() == expected
    sources[name] = b.decode()
s = sources['src/snapshot.rs']
raw_position = s[s.index('pub fn raw_position('):s.index('\n#[cfg(test)]', s.index('pub fn raw_position('))]

def obj(id=1, kind=42, position=(1., 2., 3.), tag=123, flags=3):
    return dict(id=id, entity_type=kind, flags=flags, tag=tag,
                position_bits=[struct.unpack('<I', struct.pack('<f', p))[0] for p in position])

cases = [
    dict(name='singleton', snapshots=[[obj()]]),
    dict(name='stable-open', snapshots=[[obj()], [obj()], [obj()]]),
    dict(name='anchor-drift', snapshots=[[obj(position=(0., 1., 1.))],
        [obj(position=(.04, 1., 1.))], [obj(position=(.08, 1., 1.))],
        [obj(position=(.08, 1., 1.))]]),
    dict(name='absence-reappearance', snapshots=[[obj()], [obj()], [], [obj()], [obj()]]),
    dict(name='tag-change', snapshots=[[obj()], [obj()], [obj(tag=124)], [obj(tag=124)]]),
    dict(name='flags-and-zero', snapshots=[[obj()], [obj()], [obj(flags=1)],
        [obj(position=(0., -0., 0.))], [obj()], [obj()]]),
    dict(name='all-four-types', snapshots=[[obj(id=k, kind=k) for k in [37,40,42,43]]]*2),
    dict(name='different-namespace', snapshots=[[obj()], [obj()], [obj(id=(1<<30)|1)], [obj(id=(1<<30)|1)]]),
    dict(name='nonfinite', snapshots=[[obj()], [obj()], [obj(position=(float('nan'), 1., 1.))]]),
]

harness = r'''
mod world;
mod components {#[derive(Clone,Copy)]pub struct Params{pub box_bits:u32}}
mod weapons {pub struct Tag {pub part_of:Option<u32>} pub fn tag(_:u32)->Option<Tag>{None}}
mod container {
pub struct Packet{pub kind:u32,pub start:i64,pub end:i64}
pub struct Film{pub chunks:Vec<Vec<u8>>}
impl Film{pub fn packets(&self,i:usize)->Option<Vec<Packet>>{Some(vec![Packet{kind:0,start:0,end:0},Packet{kind:2,start:0,end:(self.chunks[i].len()*8)as i64}])}}
}
mod snapshot {
use crate::components::Params;
pub const ENTITY_TABLE:u32=2;
pub struct Context;impl Context{pub fn new(_: &[u8],_:Params,_:u32)->Self{Self}}
pub struct Entity {pub index:u32,pub generation:u32,pub entity_type:u32,pub flags:u32,tag:u32,at:i64}
pub struct Creation{pub tag:u32}
fn word(c:&[u8],at:usize)->u32{u32::from_le_bytes(c[at..at+4].try_into().unwrap())}
pub fn entities(c:&[u8],_:i64,_:i64,_:&Context)->Vec<Entity>{(0..c[0]as usize).map(|i|{let at=1+i*29;let id=word(c,at);Entity{index:id&0x3fffffff,generation:id>>30,entity_type:word(c,at+4),flags:word(c,at+8),tag:word(c,at+12),at:((at+16)*8)as i64}}).collect()}
pub fn creation(_: &[u8],e:&Entity)->Option<(Creation,i64)>{Some((Creation{tag:e.tag},e.at))}
fn u32_at(c:&[u8],at:i64)->u32{let mut v=0;for i in 0..32{let p=at as usize+i;v=v<<1|u32::from((c[p/8]>>(7-p%8))&1)}v}
RAW_POSITION
}
use std::io::BufRead;
fn main(){for line in std::io::stdin().lock().lines(){let line=line.unwrap();let mut chunks=vec![vec![0]];for hex in line.split(' '){chunks.push(hex.as_bytes().chunks(2).map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect());}let result=world::read(&container::Film{chunks},components::Params{box_bits:0});let rows:Vec<_>=result.iter().map(|o|format!("{} {} {} {} {} {}",o.tag,o.entity_type,o.x.to_bits(),o.y.to_bits(),o.from,o.to.map(|v|v.to_string()).unwrap_or("-".into()))).collect();println!("{}",rows.join(";"));}}
'''.replace('RAW_POSITION', raw_position)

def wire(snapshot):
    out = bytes([len(snapshot)])
    for o in snapshot:
        out += struct.pack('<IIII', o['id'], o['entity_type'], o['flags'], o['tag'])
        lead = 3 if o['entity_type'] == 40 else 1
        bits = '0'*lead + ''.join(f'{b:08b}' for q in o['position_bits'] for b in struct.pack('<I',q))
        bits = bits.ljust(104, '0')
        out += bytes(int(bits[i:i+8],2) for i in range(0,104,8))
    return out.hex()

with tempfile.TemporaryDirectory(prefix='halo-objects-oracle-') as tmp:
    p=Path(tmp)
    (p/'world.rs').write_text(sources['src/world.rs'])
    (p/'main.rs').write_text(harness)
    subprocess.run(['rustc','--edition=2024','-Awarnings',str(p/'main.rs'),'-o',str(p/'oracle')],check=True)
    text='\n'.join(' '.join(wire(s) for s in c['snapshots']) for c in cases)+'\n'
    result=subprocess.run([str(p/'oracle')],input=text,text=True,capture_output=True,check=True).stdout.splitlines()
assert len(result)==len(cases)
for case,line in zip(cases,result):
    case['expected']=[]
    for row in line.split(';') if line else []:
        tag,kind,x,y,start,end=row.split()
        case['expected'].append(dict(tag=int(tag),entity_type=int(kind),x_bits=int(x),y_bits=int(y),
                                     start=int(start),end=None if end=='-' else int(end)))
    # Original sorting does not break equal-coordinate/tag ties by entity type;
    # HashMap iteration makes that order nondeterministic. Compare result multisets.
    case['expected'].sort(key=lambda row: json.dumps(row, sort_keys=True))
print(json.dumps(dict(commit=PIN,blobs=BLOBS,cases=cases),indent=2))
