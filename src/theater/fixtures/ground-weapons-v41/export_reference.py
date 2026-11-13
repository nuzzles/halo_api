"""Execute unchanged inventory::reader on authored operation/event inputs."""
from pathlib import Path
import hashlib, json, subprocess, sys, tempfile
PIN='8f0f8d078e513c800baa21fac416fd4c0c127d03'
BLOB='c5cc80c8f3cf7093da53b73cf00ca9a4937ffc79'
source=(Path(sys.argv[1])/'src/inventory.rs').read_bytes()
assert hashlib.sha1(f'blob {len(source)}\0'.encode()+source).hexdigest()==BLOB
def new(id=43,tag=123): return ['new',id,tag]
def pos(id=43,q=(1,2,3),region=3): return ['delta',id,region,*q]
def delete(id=43): return ['delete',id]
def pickup(tag=123): return ['pickup',tag,0,1,1]
cases=[
 dict(name='pending-then-two-positions',frames=[[new()],[pos()],[pos(q=(4,5,6))],[delete()]]),
 dict(name='open-positioned',frames=[[new()],[pos()]]),
 dict(name='never-positioned',frames=[[new()],[delete()]]),
 dict(name='pickup-same-packet',frames=[[new()],[pos()],[pickup(),delete()]]),
 dict(name='pickup-other-packet',frames=[[new()],[pos()],[pickup()],[delete()]]),
 dict(name='two-identical-tags',frames=[[new(),new(44)],[pos(),pos(44)],[pickup(),delete(),delete(44)]]),
 dict(name='reuse-after-delete',frames=[[new()],[pos()],[delete()],[new(tag=124)],[pos(q=(7,8,9))]]),
]
harness=r'''
mod inventory;
mod weapons {pub fn tag(_:u32)->Option<()>{Some(())}}
mod events {#[derive(Clone,Copy)]pub enum Body{Pickup{slot:u32,tag:Option<u32>},Other} #[derive(Clone,Copy)]pub struct Handle{pub index:u32,pub generation:u32}pub struct Event{pub kind:u32,pub body:Body,pub handles:[Option<Handle>;3]}}
mod payload{pub struct Fields{pub position:Option<(u32,[u32;3])>}pub enum OpKind{Create{tag:Option<u32>},Update{fields:Fields},Delete}pub struct Op{pub index:u32,pub entity_type:Option<u32>,pub kind:OpKind}pub struct Payload{pub ops:Vec<Op>,pub events:Vec<crate::events::Event>}}
use std::io::BufRead;
fn main(){for line in std::io::stdin().lock().lines(){let line=line.unwrap();let mut inv=inventory::Inventory::default();{let mut read=inventory::reader(&mut inv);for (i,frame)in line.split('|').enumerate(){let mut p=payload::Payload{ops:vec![],events:vec![]};for op in frame.split(';').filter(|s|!s.is_empty()){let a:Vec<_>=op.split(',').collect();let n=|i:usize|a[i].parse::<u32>().unwrap();if a[0]=="pickup"{p.events.push(events::Event{kind:9,body:events::Body::Pickup{slot:n(2),tag:Some(n(1))},handles:[Some(events::Handle{index:n(3),generation:n(4)}),None,None]});continue}let kind=match a[0]{"new"=>payload::OpKind::Create{tag:Some(n(2))},"delta"=>payload::OpKind::Update{fields:payload::Fields{position:Some((n(2),[n(3),n(4),n(5)]))}},"delete"=>payload::OpKind::Delete,_=>panic!()};p.ops.push(payload::Op{index:n(1),entity_type:Some(42),kind});}read(i,&[],&p);}}let rows:Vec<_>=inv.ground.iter().map(|g|format!("{} {} {} {} {} {} {} {}",g.tag,g.from,g.to.map(|t|t.to_string()).unwrap_or("-".into()),g.at_box,g.at[0],g.at[1],g.at[2],u8::from(g.taken))).collect();println!("{}",rows.join(";"));}}
'''
with tempfile.TemporaryDirectory(prefix='halo-ground-oracle-') as tmp:
 p=Path(tmp);(p/'inventory.rs').write_bytes(source);(p/'main.rs').write_text(harness)
 subprocess.run(['rustc','--edition=2024','-Awarnings',str(p/'main.rs'),'-o',str(p/'oracle')],check=True)
 data='\n'.join('|'.join(';'.join(','.join(map(str,op))for op in frame)for frame in case['frames'])for case in cases)+'\n'
 out=subprocess.run([str(p/'oracle')],input=data,text=True,capture_output=True,check=True).stdout.splitlines()
assert len(out)==len(cases)
for case,line in zip(cases,out):
 case['expected']=[]
 for row in line.split(';')if line else []:
  tag,start,end,region,x,y,z,taken=row.split()
  case['expected'].append(dict(tag=int(tag),start=int(start),end=None if end=='-'else int(end),region=int(region),codes=list(map(int,[x,y,z])),taken=taken=='1'))
print(json.dumps(dict(commit=PIN,blobs={'src/inventory.rs':BLOB},cases=cases),indent=2))
