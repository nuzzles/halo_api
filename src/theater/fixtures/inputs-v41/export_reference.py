"""Run unchanged libfilmshell control_view/control::reader on authored controls."""
import io,json,subprocess,sys,tarfile,tempfile
from pathlib import Path
PIN='8f0f8d078e513c800baa21fac416fd4c0c127d03'
def word(w,v): return format(v,f'0{w}b')
def opt(w,v): return '0' if v is None else '1'+word(w,v)
def make(name, analog, stance=None, mask=0, baseline=None):
 bits='100'+opt(7,baseline)+word(5,7)+('0' if analog is None else '1'+opt(2,3 if mask&1 else None)+word(6,analog[0])+word(6,analog[1])+opt(5,29 if mask&2 else None)+opt(6,61 if mask&4 else None)+opt(5,stance)+'0')+'00'
 return dict(name=name,bit_len=len(bits),hex=int(bits.ljust((len(bits)+7)//8*8,'0'),2).to_bytes((len(bits)+7)//8,'big').hex())
cases=[make('absent',None)]
for a,b in [(0,0),(0,62),(62,0),(62,62),(31,31),(30,32),(63,63),(1,61)]: cases.append(make(f'axes-{a}-{b}',[a,b]))
for stance in range(32): cases.append(make(f'stance-{stance}',[31,32],stance))
for mask in range(8):
 for baseline in [None,127]: cases.append(make(f'heads-{mask}-{baseline}',[63,0],31,mask,baseline))
with tempfile.TemporaryDirectory(prefix='halo-input-oracle-') as tmp:
 root=Path(tmp)
 archive=subprocess.check_output(['git','-C',sys.argv[1],'archive',PIN])
 with tarfile.open(fileobj=io.BytesIO(archive)) as t:t.extractall(root,filter='data')
 # Resolve the disposable harness against cached dependency versions.
 (root/'Cargo.lock').unlink(missing_ok=True)
 (root/'src/input-cases.json').write_text(json.dumps(cases))
 (root/'src/input_oracle.rs').write_text(r'''
#[test]
fn export_input_oracle() {
 let cases:serde_json::Value=serde_json::from_str(include_str!("input-cases.json")).unwrap();
 for case in cases.as_array().unwrap() {
  let data:Vec<u8>=case["hex"].as_str().unwrap().as_bytes().chunks(2).map(|h|u8::from_str_radix(std::str::from_utf8(h).unwrap(),16).unwrap()).collect();
  let mut chunk=Vec::new();
  for _ in 0..2 { chunk.extend_from_slice(&0u32.to_le_bytes());chunk.extend_from_slice(&(data.len() as u32).to_le_bytes());chunk.extend_from_slice(&10u64.to_le_bytes());chunk.extend_from_slice(&data); }
  let film=crate::container::Film::new(vec![vec![],chunk],crate::container::Framing::Packets);
  let params=crate::components::Params::new([15,15,17]);
  let mut out=crate::control::Inputs::new();
  for (i,f) in film.frames().enumerate() {
   crate::control::reader(&film,params,&mut out)(i,&[],&crate::payload::Payload{end:(f.start*8) as i64,..Default::default()});
  }
  let (messages,end)=crate::components::control_view(&data,0,&params);
  assert_eq!(end,Some(case["bit_len"].as_i64().unwrap()));
  assert_eq!(messages.len(),1);
  println!("ORACLE\t{}",serde_json::json!({"name":case["name"],"hex":case["hex"],"bit_len":case["bit_len"],"series":out,"throttle":messages[0].throttle,"head":messages[0].head}));
 }
}
''')
 with (root/'src/lib.rs').open('a') as f:f.write('\n#[cfg(test)] mod input_oracle;\n')
 output=subprocess.check_output(['cargo','+stable','test','--offline','--lib','export_input_oracle','--','--nocapture'],cwd=root,text=True)
 rows=[json.loads(l.split('\t',1)[1]) for l in output.splitlines() if l.startswith('ORACLE\t')]
 assert len(rows)==len(cases)
 print(json.dumps({'commit':PIN,'cases':rows},indent=2))
