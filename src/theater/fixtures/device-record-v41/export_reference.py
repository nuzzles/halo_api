"""Export a captured device/entity prefix through the original upstream readers.

Usage: python3 export_reference.py /path/to/libfilmshell /path/to/experiments/films
Writes packet.bin and reference.json beside this script. No halo_api code runs.
The original entry reader is called sequentially without tick-marker recovery;
only diagnostic callbacks are added to component primitive reads/dispatch.
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

PIN = '18067987ebc3628ad499c819c6aee814692df43b'
FILM = 'appearance/01-cadet-blue'
OFFSET = 567583
root = Path(__file__).parent
corpus = Path(sys.argv[2])
registry = (corpus / FILM / 'chunk-000-type-1.bin').read_bytes()
chunk = (corpus / FILM / 'chunk-001-type-2.bin').read_bytes()
kind, unknown, size, timestamp = struct.unpack_from('<HHIQ', chunk, OFFSET - 16)
assert kind == 0 and unknown == 0 and OFFSET + size <= len(chunk)
packet = chunk[OFFSET:OFFSET + size]
# Context provenance is the existing independent historical export, not local
# reconstruction. Its positions are irrelevant to this captured all-create prefix.
context = None
contexts_path = root.parent / 'reference-contexts-d61443e-v41.jsonl.zlib'
for line in zlib.decompress(contexts_path.read_bytes()).splitlines():
    row = json.loads(line)
    if row['file'] != FILM + '/chunk-001-type-2.bin':
        continue
    if row.get('context') is not None:
        context = row['context']
    if row['offset'] == OFFSET:
        break
else:
    raise AssertionError('missing independent initial context')
assert context is not None
seeds = '\n'.join(f'w.types.insert({slot},{binding["TypeIndex"]});' for slot, binding in context['slots'].items())

archive = subprocess.check_output(['git', '-C', sys.argv[1], 'archive', PIN])
with tempfile.TemporaryDirectory(prefix='halo-device-record-oracle-') as tmp:
    project = Path(tmp)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        tar.extractall(project, filter='data')
    (project / 'src/device.bin').write_bytes(packet)
    (project / 'src/registry.bin').write_bytes(registry)
    path = project / 'src/components.rs'
    source = path.read_text()
    for signature in ['fn bits(&mut self, w: u32) -> u32 {', 'fn skip(&mut self, w: u32) {']:
        assert source.count(signature) == 1
        source = source.replace(signature, signature + '\n #[cfg(test)] crate::payload::device_oracle::trace(self.at,w);')
    old = 'r.block(comp, &row, &mut fields)'
    assert source.count(old) == 1
    source = source.replace(old, '''{
        #[cfg(test)] crate::payload::device_oracle::begin();
        let start = r.at;
        let result = r.block(comp, &row, &mut fields);
        #[cfg(test)] crate::payload::device_oracle::component(table.1,i,start,r.at,result.is_some());
        result
    }''')
    path.write_text(source)
    path = project / 'src/payload.rs'
    source = path.read_text() + r'''
#[cfg(test)] pub(crate) mod device_oracle {
use super::*;
use std::cell::RefCell;
thread_local! { static TRACE: RefCell<Vec<(i64,u32)>> = const {RefCell::new(Vec::new())}; }
pub(crate) fn trace(at:i64,width:u32){TRACE.with(|t|t.borrow_mut().push((at,width)))}
pub(crate) fn begin(){TRACE.with(|t|t.borrow_mut().clear())}
pub(crate) fn component(t:u32,i:u32,start:i64,end:i64,complete:bool){
 let trace=TRACE.with(|t|t.borrow().iter().map(|(a,w)|format!("[{a},{w}]")).collect::<Vec<_>>().join(","));
 println!("COMP\t{t}\t{i}\t{start}\t{end}\t{complete}\t[{trace}]");
}
#[test] fn export_device_record(){
 let c=include_bytes!("device.bin");let reg=include_bytes!("registry.bin");
 let header=crate::header::read(reg);let tag=header.level.unwrap();
 let level=crate::levels::level(tag).unwrap();
 let mut params=Params::new(level.widths);params.set_level(Some(level));
 println!("LEVEL\t{tag}\t{:?}\t{}",params.widths,params.box_bits);
 let rows=registry::blocks(reg);let versions=registry::versions(reg);
 for t in [9usize,38,43] {println!("REG\t{t}\t{:?}\t{:?}",rows[t],versions[t]);}
 let mut w=Walker::new(reg,params);
 SEEDS
 let mut ops=Vec::new();let mut at=2;
 for _ in 0..1024 {
  let outcome=w.entry(c,at,(c.len()*8) as i64,(c.len()*8) as i64,&mut ops);
  match outcome {
   Ok(Some(end))=>{assert_eq!(ops.len(),1);let op=&ops[0];assert!(matches!(op.kind,OpKind::Create{..}));
    println!("RECORD\t{at}\t{end}\t{}\t{}\t{}",op.index,op.generation,op.entity_type.unwrap());at=end;ops.clear();},
   Err(stop)=>{println!("STOP\t{at}\t{stop:?}");return;},
   Ok(None)=>panic!("expected unresolved successor"),
  }
 }
 panic!("unbounded record loop");
}
}
'''
    path.write_text(source.replace(' SEEDS', seeds))
    output = subprocess.check_output([
        'cargo', '+stable', 'test', '--offline', '--lib', 'export_device_record', '--', '--nocapture',
    ], cwd=project, text=True)

out = dict(commit=PIN, file=FILM + '/chunk-001-type-2.bin', offset=OFFSET,
           timestamp_us=timestamp, registry_sha256=hashlib.sha256(registry).hexdigest(),
           chunk_sha256=hashlib.sha256(chunk).hexdigest(), packet_sha256=hashlib.sha256(packet).hexdigest(),
           build_id=struct.unpack_from('<I', registry, 0xCB454)[0], branch='HI_1_13_0',
           initial_context=context, registry=[], records=[])
components = []
for line in output.splitlines():
    parts = line.split('\t')
    if parts[0] == 'LEVEL':
        out.update(level_tag=int(parts[1]), axis_bits=json.loads(parts[2]), index_bits=int(parts[3]))
    elif parts[0] == 'REG':
        names, versions = json.loads(parts[2]), json.loads(parts[3])
        out['registry'].append(dict(archetype=int(parts[1]), rows=list(zip(names, versions))))
    elif parts[0] == 'COMP':
        components.append(dict(archetype=int(parts[1]), index=int(parts[2]), start=int(parts[3]),
                               end=int(parts[4]), complete=parts[5] == 'true', trace=json.loads(parts[6])))
    elif parts[0] == 'RECORD':
        out['records'].append(dict(start=int(parts[1]), end=int(parts[2]), index=int(parts[3]),
                                  generation=int(parts[4]), archetype=int(parts[5]), components=components))
        components = []
    elif parts[0] == 'STOP':
        out.update(stop_bit=int(parts[1]), stop=parts[2], attempted_components=components)
assert len(out['records']) == 90 and out['stop_bit'] == 64496 and out['stop'] == 'UnknownEntity'
assert not out['attempted_components']
assert out['build_id'] == 269225 and registry[0xCB414:0xCB434].split(b'\0')[0] == out['branch'].encode()
for record in out['records']:
    assert record['end'] <= len(packet) * 8
    for component in record['components']:
        assert component['complete'] and all(a + w <= len(packet) * 8 for a, w in component['trace'])
(root / 'packet.bin').write_bytes(packet)
(root / 'reference.json').write_text(json.dumps(out, indent=2) + '\n')
print('Exported',len(out['records']),'records through bit',out['stop_bit'])
