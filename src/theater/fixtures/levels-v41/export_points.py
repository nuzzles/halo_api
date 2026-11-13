"""Trace unchanged pinned point/component readers with their own level adapter.

The five mp_02 region-4 bodies are intentionally excluded: the original reader's
four-slot table cannot represent them. No upstream width table is patched.
"""
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
PIN = '18067987ebc3628ad499c819c6aee814692df43b'
cases = json.loads(Path(__file__).with_name('points.json').read_text())
archive = subprocess.check_output(['git','-C',sys.argv[1],'archive',PIN])
with tempfile.TemporaryDirectory(prefix='halo-region-oracle-') as tmp:
    root=Path(tmp)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar: tar.extractall(root,filter='data')
    (root/'src/region-cases.tsv').write_text('\n'.join(f"{c['name']}\t{c['tag']}\t{c['kind']}\t{c['precision']}\t{c['hex']}" for c in cases if c['upstream'])+'\n')
    path=root/'src/events.rs'; source=path.read_text()
    sig='pub fn bits(&mut self, n: u32) -> u32 {'
    assert source.count(sig)==1
    source=source.replace(sig,sig+'\n #[cfg(test)] region_oracle::trace(self.at,n);')
    source+=r'''
#[cfg(test)] pub(crate) mod region_oracle {
    use super::*;
    use std::cell::RefCell;
    thread_local! { static TRACE: RefCell<Vec<(i64,u32)>> = const {RefCell::new(Vec::new())}; }
    pub(crate) fn trace(at:i64,n:u32) { TRACE.with(|t|t.borrow_mut().push((at,n))); }
    #[test] fn export_region_events() {
        for line in include_str!("region-cases.tsv").lines() {
            let v:Vec<_>=line.split('\t').collect();
            if v[2]!="event" {continue;}
            let data:Vec<_>=v[4].as_bytes().chunks(2).map(|b|u8::from_str_radix(std::str::from_utf8(b).unwrap(),16).unwrap()).collect();
            let l=crate::levels::level(v[1].parse().unwrap()).unwrap();
            let mut p=crate::components::Params::new(l.widths); p.set_level(Some(l));
            let mut r=Bits{c:&data,at:0};
            TRACE.with(|t|t.borrow_mut().clear());
            let _=r.pos_codes(Layout::from(&p),v[3].parse().unwrap());
            let trace=TRACE.with(|t|t.borrow().iter().map(|(a,w)|format!("[{a},{w}]")).collect::<Vec<_>>().join(","));
            let end=r.at; let marker=r.bits(8);
            println!("ORACLE\t{}\t{}\t{}\t[{}]",v[0],end,marker,trace);
        }
    }
}
'''
    path.write_text(source)
    path=root/'src/components.rs'; source=path.read_text()
    for sig in ['fn bits(&mut self, w: u32) -> u32 {','fn skip(&mut self, w: u32) {']:
        assert source.count(sig)==1
        source=source.replace(sig,sig+'\n #[cfg(test)] region_component_oracle::trace(self.at,w);')
    source+=r'''
#[cfg(test)] mod region_component_oracle {
    use super::*;
    use std::cell::RefCell;
    thread_local! { static TRACE: RefCell<Vec<(i64,u32)>> = const {RefCell::new(Vec::new())}; }
    pub(super) fn trace(at:i64,n:u32) {TRACE.with(|t|t.borrow_mut().push((at,n)));}
    #[test] fn export_region_components() {
        for line in include_str!("region-cases.tsv").lines() {
            let v:Vec<_>=line.split('\t').collect();
            if v[2]=="event" {continue;}
            let data:Vec<_>=v[4].as_bytes().chunks(2).map(|b|u8::from_str_radix(std::str::from_utf8(b).unwrap(),16).unwrap()).collect();
            let l=crate::levels::level(v[1].parse().unwrap()).unwrap();
            let mut p=Params::new(l.widths); p.set_level(Some(l));
            let mut r=Reader{c:&data,at:0,object_type:0,slot:0,row_version:4};
            TRACE.with(|t|t.borrow_mut().clear());
            let kind=if v[2]=="dynamic" {Component::Position} else {Component::ObjectPosition};
            assert!(r.block(kind,&p,&mut Fields::default()).is_some());
            let trace=TRACE.with(|t|t.borrow().iter().map(|(a,w)|format!("[{a},{w}]")).collect::<Vec<_>>().join(","));
            let end=r.at; let marker=r.bits(8);
            println!("ORACLE\t{}\t{}\t{}\t[{}]",v[0],end,marker,trace);
        }
    }
}
'''
    path.write_text(source)
    output=subprocess.check_output(['cargo','+stable','test','--offline','--lib','export_region_','--','--nocapture','--test-threads=1'],cwd=root,text=True)
rows={}
for line in output.splitlines():
    if 'ORACLE\t' in line:
        line = 'ORACLE\t' + line.split('ORACLE\t', 1)[1]
        _,name,end,marker,trace=line.split('\t');rows[name]=dict(name=name,end=int(end),marker=int(marker),trace=json.loads(trace))
expected=[c for c in cases if c['upstream']]
assert len(rows)==len(expected)==324
for c in expected:
    assert rows[c['name']]['end']==c['bit_len'],c['name']
    assert rows[c['name']]['marker']==211,c['name']
print(json.dumps(dict(commit=PIN,cases=[rows[c['name']] for c in expected]),indent=2))
