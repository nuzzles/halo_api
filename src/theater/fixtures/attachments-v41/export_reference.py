"""Execute unchanged pinned ParentState block in a disposable source archive."""
import hashlib,io,json,os,subprocess,sys,tarfile,tempfile
from pathlib import Path
PIN='8f0f8d078e513c800baa21fac416fd4c0c127d03'
BLOB='6acd5fbb992f7d65ff8fdbd88a6b7b739a88a1a4'
cases=json.loads((Path(sys.argv[2]) if len(sys.argv)>2 else Path(__file__).with_name('cases.json')).read_text())
with tempfile.TemporaryDirectory(prefix='halo-attachment-oracle-') as tmp:
 root=Path(tmp)
 archive=subprocess.check_output(['git','-C',sys.argv[1],'archive',PIN])
 with tarfile.open(fileobj=io.BytesIO(archive)) as tar:tar.extractall(root,filter='data')
 (root/'Cargo.lock').unlink(missing_ok=True)
 p=root/'src/components.rs';b=p.read_bytes()
 assert hashlib.sha1(f'blob {len(b)}\0'.encode()+b).hexdigest()==BLOB
 p.write_bytes(b+br'''
#[cfg(test)]
mod attachment_oracle {
 use super::*;
 fn value(v:Option<u32>)->String {v.map_or("null".into(),|v|v.to_string())}
 #[test]
 fn export_attachment_values() {
  for line in include_str!("attachment-cases.tsv").lines() {
   let parts:Vec<_>=line.split('\t').collect();
   let b:Vec<_>=parts[1].as_bytes().chunks(2).map(|v|u8::from_str_radix(std::str::from_utf8(v).unwrap(),16).unwrap()).collect();
   let version=parts[2].parse().unwrap();let object_type=parts[3].parse().unwrap();
   let mut p=Params::new([15,15,17]);p.version=version;
   let mut reader=Reader {c:&b,at:0,object_type,slot:512,row_version:version};
   let mut fields=Fields::default();
   let complete=reader.block(Component::ParentState,&p,&mut fields).is_some();
   let parent=match fields.parent {Some(Some(Parent::Unit(i)))=>format!("[true,{i}]"),Some(Some(Parent::Object(i)))=>format!("[false,{i}]"),Some(None)=>"null".into(),None=>"\"unobserved\"".into()};
   println!("PARENT\t{}\t{}\t{}\t{}\t{}\t{}\t{}",parts[0],reader.at,complete,parent,value(fields.parent_generation),value(fields.parent_node),value(fields.seat.flatten()));
  }
 }
}
''')
 (root/'src/attachment-cases.tsv').write_text(''.join(f"{c['name']}\t{c['hex']}\t{c['parameter']}\t{c['archetype']}\n" for c in cases))
 result=subprocess.check_output(['cargo','test','--offline','--lib','export_attachment_values','--','--nocapture'],cwd=root,env=dict(os.environ,CARGO_TARGET_DIR='/private/tmp/halo-attachment-oracle-target'),text=True)
 rows=[]
 for line in result.splitlines():
  if line.startswith('PARENT\t'):
   _,name,end,complete,parent,generation,node,seat=line.split('\t');rows.append(dict(name=name,end=int(end),complete=complete=='true',parent=json.loads(parent),generation=json.loads(generation),node=json.loads(node),seat=json.loads(seat)))
 assert len(rows)==len(cases)
 print(json.dumps(dict(commit=PIN,components_blob=BLOB,cases=rows),indent=2))
