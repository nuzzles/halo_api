"""Retain a captured snapshot refusal independently traced through libfilmshell.

python3 export_reference.py LIBFILMSHELL CORPUS
Writes packet.bin and reference.json beside this file. Uses the original
sequential snapshot walk; asserts no vehicle occurs, hence no vehicle probing.
Only test serialization and primitive/component trace callbacks are added.
"""
import hashlib
import io
import json
from pathlib import Path
import struct
import subprocess
import sys
import tarfile
import tempfile
import zlib

PIN='18067987ebc3628ad499c819c6aee814692df43b'
FILM='ranked-arena/02-oddball'
OFFSET=343051
root=Path(__file__).parent
registry=(Path(sys.argv[2])/FILM/'chunk-000-type-1.bin').read_bytes()
chunk=(Path(sys.argv[2])/FILM/'chunk-057-type-2.bin').read_bytes()
try: chunk=zlib.decompress(chunk)
except zlib.error: pass
kind,unknown,size,timestamp=struct.unpack_from('<HHIQ',chunk,OFFSET-16)
assert kind==2 and OFFSET+size<=len(chunk)
packet=chunk[OFFSET:OFFSET+size]
archive=subprocess.check_output(['git','-C',sys.argv[1],'archive',PIN])
with tempfile.TemporaryDirectory(prefix='halo-snapshot-refusal-') as tmp:
 project=Path(tmp)
 with tarfile.open(fileobj=io.BytesIO(archive)) as tar:tar.extractall(project,filter='data')
 (project/'src/refusal-reg.bin').write_bytes(registry)
 (project/'src/refusal-packet.bin').write_bytes(packet)
 path=project/'src/components.rs';source=path.read_text()
 for signature in ['fn bits(&mut self, w: u32) -> u32 {','fn skip(&mut self, w: u32) {']:
  assert source.count(signature)==1
  source=source.replace(signature,signature+'\n #[cfg(test)] crate::snapshot::refusal_oracle::trace(self.at,w);')
 old='r.block(comp, &row, &mut fields)';assert source.count(old)==1
 source=source.replace(old,'''{
 #[cfg(test)] crate::snapshot::refusal_oracle::begin();
 let start=r.at;let result=r.block(comp,&row,&mut fields);
 #[cfg(test)] crate::snapshot::refusal_oracle::component(i,start,r.at,result.is_some());
 result
 }''');path.write_text(source)
 path=project/'src/snapshot.rs'
 path.write_text(path.read_text()+r'''
#[cfg(test)] pub(crate) mod refusal_oracle {
use super::*;
use std::cell::RefCell;
thread_local!{static TRACE:RefCell<Vec<(i64,u32)>>=const{RefCell::new(Vec::new())};}
pub(crate) fn trace(at:i64,width:u32){TRACE.with(|t|t.borrow_mut().push((at,width)))}
pub(crate) fn begin(){TRACE.with(|t|t.borrow_mut().clear())}
pub(crate) fn component(i:u32,start:i64,end:i64,complete:bool){
 if start<226358{return}
 let trace=TRACE.with(|t|t.borrow().iter().map(|(a,w)|format!("[{a},{w}]")).collect::<Vec<_>>().join(","));
 println!("COMP\t{i}\t{start}\t{end}\t{complete}\t[{trace}]");
}
#[test] fn export_snapshot_refusal(){
 let reg=include_bytes!("refusal-reg.bin");let c=include_bytes!("refusal-packet.bin");
 let header=crate::header::read(reg);let level=crate::levels::level(header.level.unwrap()).unwrap();
 let mut params=Params::new(level.widths);params.set_level(Some(level));
 let ctx=Context::new(reg,params,level.box_bits);
 let w=walk(c,0,c.len(),&ctx);
 assert!(!w.entities.iter().any(|e|e.entity_type==40),"no vehicle branch probing");
 assert_eq!(w.stopped,Some((35,7)));assert_eq!(w.slots,756);assert_eq!(w.entities.len(),129);
 let e=w.entities.last().unwrap();assert_eq!(e.at,226358);
 println!("RECORD\t{}\t{}\t{}\t{}",e.at,e.index,e.generation,e.entity_type);
 let blocks=registry::blocks(reg);let versions=registry::versions(reg);
 println!("REG\t{:?}\t{:?}",blocks[35],versions[35]);
}
}
''')
 output=subprocess.check_output(['cargo','test','--offline','--lib','export_snapshot_refusal','--','--nocapture'],cwd=project,text=True)
out=dict(commit=PIN,file=FILM+'/chunk-057-type-2.bin',payload_byte=OFFSET,timestamp_us=timestamp,
         registry_sha256=hashlib.sha256(registry).hexdigest(),packet_sha256=hashlib.sha256(packet).hexdigest(),
         slots=756,live_records=129,components=[])
for line in output.splitlines():
 p=line.split('\t')
 if p[0]=='COMP':out['components'].append(dict(index=int(p[1]),start=int(p[2]),end=int(p[3]),complete=p[4]=='true',trace=json.loads(p[5])))
 elif p[0]=='RECORD':out.update(start=int(p[1]),index=int(p[2]),generation=int(p[3]),archetype=int(p[4]))
 elif p[0]=='REG':out['registry']=list(zip(json.loads(p[1]),json.loads(p[2])))
assert len(out['components'])==8 and out['components'][-1]['end']==227003
(root/'packet.bin').write_bytes(packet)
(root/'reference.json').write_text(json.dumps(out,indent=2)+'\n')
