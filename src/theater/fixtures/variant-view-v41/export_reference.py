"""Execute original pinned variant primitives and named-path constants.
The original anchor-searching entrypoint is not used as a canonical boundary.
Usage: export_reference.py PINNED_LIBFILMSHELL > reference.json
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

PIN='f050371a6cda83b9887eeb83a1884336fce28e53'
BLOBS={'src/variant.rs':'6592ffba89f76bb24d2cc4abcd6c09b33b7cf088',
       'src/variant_stats.rs':'4814607a2a27f4ad2cd4ef0a16845582f73c8172',
       'src/game/options.rs':'43ade2aafef5f38ca492b507e7adce03cc4be38d',
       'src/game/options_data.rs':'021e911b805a2b74c24321c7209462a76bd0a708'}
sources={}
for name,blob in BLOBS.items():
    b=(Path(sys.argv[1])/name).read_bytes()
    if hashlib.sha1(f'blob {len(b)}\0'.encode()+b).hexdigest()!=blob:raise ValueError(name)
    sources[name]=b.decode()
WRAPPER=r'''
pub fn export(b:&[u8]) {
    let mut p=0;
    nested(b,&mut p,10,&mut Vec::new(),&mut |path,x| {
        println!("value\t{}\t{}",path.iter().map(u32::to_string).collect::<Vec<_>>().join("/"),x);
        if path==OPTION_VALUE {
            match option_value(x) {
                Some(OptionValue::Number(n))=>println!("option\tnumber\t{:016x}",n.to_bits()),
                Some(OptionValue::Bool(b))=>println!("option\tbool\t{}",b),
                None=>println!("option\tunknown\t-"),
            }
        }
    },0).unwrap();
    assert_eq!(p,b.len());
    println!("end\t{}",p);
    // Frame the unchanged body for the original heuristic text API only.
    // Local string boundaries are never derived from this anchor search.
    let mut chunk=vec![0;START];chunk.extend_from_slice(b);
    if let Some(v)=read(&chunk) {
        let hex=|s:&str|s.as_bytes().iter().map(|b|format!("{b:02x}")).collect::<String>();
        println!("text-name\t{}",hex(&v.name));
        println!("text-description\t{}",hex(&v.description));
        for label in &v.stats { println!("text-label\t{}",hex(label)); }
    }
}
pub fn schema() {
    for (path,name) in [(RESPAWN,"RespawnSeconds"),(SUICIDE_PENALTY,"SuicidePenaltySeconds"),
        (BETRAYAL_PENALTY,"BetrayalPenaltySeconds"),(SCORE_TO_WIN,"ScoreToWinRound"),(TIME_LIMIT,"RoundTimeLimitSeconds"),
        (OVERTIME,"OvertimeSeconds"),(LAST_CHANCE_IN_OVERTIME,"LastChanceEnabledInOvertime"),(LAST_CHANCE,"LastChanceSeconds")] {
        println!("path\t{}\t{}",name,path.iter().map(u32::to_string).collect::<Vec<_>>().join("/"));
    }
    for &(path,name) in RULES {println!("path\t{}\t{}",name,path.iter().map(u32::to_string).collect::<Vec<_>>().join("/"));}
    for &(path,index,name) in FLAGS {println!("flag\t{}\t{}\t{}",name,path.iter().map(u32::to_string).collect::<Vec<_>>().join("/"),index);}
}
'''
HARNESS=r'''
mod variant;
mod variant_stats;
mod game;
use std::io::BufRead;
fn main() {
    variant::schema();
    for line in std::io::stdin().lock().lines() {
        let line=line.unwrap();
        let (name,hex)=line.split_once('\t').unwrap();
        let b:Vec<_>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();
        println!("case\t{name}");
        variant::export(&b);
        variant_stats::export(&b);
    }
}
'''
STATS_WRAPPER=r'''
pub fn export(b:&[u8]) {
    // Supply only the artificial framing expected by the original scanning API.
    // The recorded BaseVariantData body and original projection are unchanged.
    let mut chunk=vec![0;REGION_START];
    chunk.extend_from_slice(ANCHOR);chunk.extend_from_slice(MODE_PATH);chunk.push(0);
    let mut p=0;varint(b,&mut p).unwrap();chunk.extend_from_slice(&b[p..]);
    if let Some(d)=read(&chunk) {
        println!("stats\t{}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}",d.server_30fps,d.tick_rate,d.kill_cam,d.kill_cam_seconds,d.play_of_the_game,d.machinima);
        for s in &d.stats {
            let hex:String=s.name.as_bytes().iter().map(|b|format!("{b:02x}")).collect();
            println!("definition\t{}\t{}\t{}\t{}\t{}\t{}\t{}",hex,s.id,s.base,s.aggregation,s.sort,s.display,s.precision);
        }
        for slot in &d.scoreboard {println!("column\t{slot}");}
        println!("spawn\t{:?}",d.spawn_settings);
    }
}
'''
cases=json.loads((Path(sys.argv[2]) if len(sys.argv)>2 else Path(__file__).parent/'cases.json').read_text())
with tempfile.TemporaryDirectory(prefix='halo-variant-oracle-') as tmp:
    p=Path(tmp);(p/'game').mkdir()
    (p/'variant.rs').write_text(sources['src/variant.rs']+WRAPPER)
    (p/'variant_stats.rs').write_text(sources['src/variant_stats.rs']+STATS_WRAPPER)
    for n in ['options.rs','options_data.rs']:(p/'game'/n).write_text(sources['src/game/'+n])
    (p/'game/mod.rs').write_text('pub mod options; pub mod options_data; pub use options::OptionValue;')
    (p/'main.rs').write_text(HARNESS)
    subprocess.run(['rustc','--edition=2024','-Awarnings',str(p/'main.rs'),'-o',str(p/'oracle')],check=True)
    output=subprocess.run([str(p/'oracle')],input=''.join(c['name']+'\t'+c['base_hex']+'\n' for c in cases),text=True,capture_output=True,check=True).stdout
paths=[];flags=[];out=[]
for line in output.splitlines():
    f=line.split('\t')
    if f[0]=='path':paths.append(dict(name=f[1],path=list(map(int,f[2].split('/')))))
    elif f[0]=='flag':flags.append(dict(name=f[1],path=list(map(int,f[2].split('/'))),index=int(f[3])))
    elif f[0]=='case':out.append(dict(name=f[1],values=[],options=[]))
    elif f[0]=='value':out[-1]['values'].append(dict(path=list(map(int,f[1].split('/'))),value=int(f[2])))
    elif f[0]=='option':out[-1]['options'].append(dict(kind=f[1],value=f[2]))
    elif f[0]=='end':out[-1]['end_byte']=int(f[1])
    elif f[0]=='stats':out[-1].update(stats=f[1:],definitions=[],columns=[])
    elif f[0]=='definition':out[-1]['definitions'].append(dict(name_hex=f[1],id=int(f[2]),base=int(f[3]),aggregation=int(f[4]),sort=int(f[5]),display=int(f[6]),precision=int(f[7])))
    elif f[0]=='column':out[-1]['columns'].append(int(f[1]))
    elif f[0]=='spawn':out[-1]['spawn']=f[1]
    elif f[0]=='text-name':out[-1]['text']=dict(name_hex=f[1],labels=[])
    elif f[0]=='text-description':out[-1]['text']['description_hex']=f[1]
    elif f[0]=='text-label':out[-1]['text']['labels'].append(f[1])
    else:raise ValueError(f)
print(json.dumps(dict(commit=PIN,blobs=BLOBS,paths=paths,flags=flags,cases=out),indent=2))
