"""Execute the unchanged original unit-control block in a disposable pinned archive."""
import hashlib,io,json,os,subprocess,sys,tarfile,tempfile
from pathlib import Path
PIN='8f0f8d078e513c800baa21fac416fd4c0c127d03'
BLOB='6acd5fbb992f7d65ff8fdbd88a6b7b739a88a1a4'
cases=json.loads((Path(sys.argv[2]) if len(sys.argv)>2 else Path(__file__).with_name('cases.json')).read_text())
with tempfile.TemporaryDirectory(prefix='halo-controller-oracle-') as tmp:
 root=Path(tmp)
 archive=subprocess.check_output(['git','-C',sys.argv[1],'archive',PIN])
 with tarfile.open(fileobj=io.BytesIO(archive)) as tar:tar.extractall(root,filter='data')
 (root/'Cargo.lock').unlink(missing_ok=True) # Resolve cached harness dependencies offline.
 p=root/'src/components.rs';b=p.read_bytes()
 assert hashlib.sha1(f'blob {len(b)}\0'.encode()+b).hexdigest()==BLOB
 p.write_bytes(b+br'''
#[cfg(test)]
mod controller_oracle {
 use super::*;
 #[test]
 fn export_controller_values() {
  for line in include_str!("controller-cases.tsv").lines() {
   let parts:Vec<_>=line.split('\t').collect();
   let b:Vec<_>=parts[1].as_bytes().chunks(2).map(|v|u8::from_str_radix(std::str::from_utf8(v).unwrap(),16).unwrap()).collect();
   let mut first=None;
   for version in [1,2] {
   let mut p=Params::new([15,15,17]);p.version=version;
   let mut reader=Reader {c:&b,at:0,object_type:35,slot:512,row_version:version};
   let mut fields=Fields::default();
   let complete=reader.block(Component::UnitControl,&p,&mut fields).is_some();
   let value=fields.controller.map(|v|v.map_or("null".into(),|id|id.to_string())).unwrap_or("\"unobserved\"".into());
   if let Some(expected)=first {assert_eq!(expected,(reader.at,complete,fields.controller));}
   else {first=Some((reader.at,complete,fields.controller));}
   if version==2 {println!("CONTROLLER\t{}\t{}\t{}\t{}",parts[0],reader.at,complete,value);}
   }
  }
 }
}
''')
 (root/'src/controller-cases.tsv').write_text(''.join(c['name']+'\t'+c['hex']+'\n' for c in cases))
 env=dict(os.environ,CARGO_TARGET_DIR='/private/tmp/halo-controller-oracle-target')
 result=subprocess.check_output(['cargo','test','--offline','--lib','export_controller_values','--','--nocapture'],cwd=root,env=env,text=True)
 rows=[]
 for line in result.splitlines():
  if line.startswith('CONTROLLER\t'):
   _,name,end,complete,player=line.split('\t');rows.append(dict(name=name,end=int(end),complete=complete=='true',player=json.loads(player)))
 assert len(rows)==len(cases)
 print(json.dumps(dict(commit=PIN,components_blob=BLOB,row_parameters=[1,2],cases=rows),indent=2))
