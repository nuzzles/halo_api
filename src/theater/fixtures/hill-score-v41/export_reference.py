"""Run the original pinned hill_score_reader on authored native property bodies."""
from pathlib import Path
import hashlib,json,subprocess,sys,tempfile
PIN='8f0f8d078e513c800baa21fac416fd4c0c127d03'
ROOT=Path(sys.argv[1]);blobs={'src/zones.rs':'4701db66d587260404006301be25b4fb2ae1931f'}
z=(ROOT/'src/zones.rs').read_bytes();assert hashlib.sha1(f'blob {len(z)}\0'.encode()+z).hexdigest()==blobs['src/zones.rs'];s=z.decode()
def extract(prefix):
 a=s.index(prefix);b=s.index('{',a);depth=1;i=b+1
 while depth:depth+=(s[i]=='{')-(s[i]=='}');i+=1
 return s[a:i]
source='\n'.join(extract(p) for p in ['struct Bits<','impl Bits<','pub struct HillScore','pub fn hill_score_reader<'])
consts=s[s.index('const PROP_LEFT:'):s.index('/// Each team\'s points')]
harness=r'''
mod bits {pub fn bits_at(c:&[u8],at:i64,n:u32)->u32{let mut v=0;for i in 0..n as usize {let p=at as usize+i;assert!(p<c.len()*8);v=v<<1|u32::from((c[p/8]>>(7-p%8))&1)}v}}
mod creation {pub fn property_bits(k:u32)->u32{match k{1|11=>4,2|10=>1,3|7=>24,_=>32}}}
mod payload {pub enum OpKind{Create{},Update{fields:Fields},Delete} pub struct Fields{pub blocks:Vec<(u32,i64)>}pub struct Op{pub entity_type:Option<u32>,pub kind:OpKind,pub at:i64,pub index:u32,pub init:Option<Fields>}pub struct Payload{pub ops:Vec<Op>}}
fn put(w:&mut Vec<bool>,n:usize,v:u32){w.extend((0..n).rev().map(|i|v&(1<<i)!=0));}
fn property(name:u32,tag:u32,value:u32,index:u32)->(Vec<u8>,payload::Payload){let mut w=vec![false;21];put(&mut w,1,0);put(&mut w,32,name);let at=w.len()as i64;put(&mut w,4,tag);put(&mut w,creation::property_bits(tag)as usize,value);let mut b=vec![0;w.len().div_ceil(8)];for(i,x)in w.into_iter().enumerate(){b[i/8]|=u8::from(x)<<(7-i%8)}(b,payload::Payload{ops:vec![payload::Op{entity_type:Some(13),kind:payload::OpKind::Create{},at:0,index,init:Some(payload::Fields{blocks:vec![(2,at)]})}]})}
fn main(){for limit in [1,2,99,100,999]{for q in [0,1,0x7ffffe,0x7fffff,0x800000,0x800001,0x800100,0x8051eb,0x80a3d7,0x810624,0xfffffe,0xffffff]{let mut out=HillScore{to_win:0,hills:0,runs:vec![]};{let mut read=hill_score_reader(|_|Some(0),&mut out);let (b,p)=property(PROP_TO_WIN,8,limit,1);read(0,&b,&p);let(b,p)=property(PROP_LEFT,7,q,2);read(1,&b,&p);}assert_eq!(out.runs.len(),1);println!("{} {} {}",limit,q,out.runs[0].2)}}}
'''
with tempfile.TemporaryDirectory(prefix='halo-hill-oracle-')as tmp:
 p=Path(tmp);(p/'main.rs').write_text(consts+'\n'+source+'\n'+harness)
 subprocess.run(['rustc','--edition=2024','-Awarnings',str(p/'main.rs'),'-o',str(p/'oracle')],check=True)
 result=subprocess.check_output([str(p/'oracle')],text=True)
rows=[dict(zip(['limit','quantum','points'],map(int,line.split())))for line in result.splitlines()]
print(json.dumps(dict(commit=PIN,blobs=blobs,cases=rows),indent=2))
