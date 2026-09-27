#!/usr/bin/env python3
"""Run native-oracle direct-reader cases on actual wasm32 with Node (offline)."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import zlib

RUST = 'use halo_api::theater::*;\n#[unsafe(no_mangle)]\npub extern "C" fn run() -> i32 {\n let rows: &[(u64,u32,&str,&[u8],usize,bool,usize,u64,usize)] = &[ENTRIES];\n for (i, &(width,mode,name,data,start,status,end,tail,tail_end)) in rows.iter().enumerate() {\n  let mut profile=NativeScanProfile::default();\n  profile.movement.world_object.index_bits=width;profile.movement.world_object.axis_bits=[width;3];\n  profile.movement.traversal.index_bits=width;profile.movement.traversal.axis_bits=[width;3];\n  profile.movement.delta_axis_width=width;profile.movement.full_precision=mode==6;\n  profile.grammar.baseline_scope=false;profile.grammar.writer_absolute=false;\n  // The previous eager adapter would reject every one of these on wasm32.\n  if profile.component_encoding().is_ok() {return -2000-i as i32}\n  let mut reader=NativeFilmReader::with_context(data,NativeReaderContext{profile,observer:None});\n  reader.set_bit_position(start);\n  let Ok((s,r))=reader.read_component(name,0,35) else {return -1000-i as i32};\n  if s!=Some(status) || r.end_bit!=end || reader.read_bits(7)!=Some(tail) || reader.bit_position()!=tail_end {return i as i32}\n }\n 648\n}\n'
JAVASCRIPT = 'const fs=require(\'fs\');\nconst mod=new WebAssembly.Module(fs.readFileSync(process.argv[2]));\nconst imports={};\nfor(const i of WebAssembly.Module.imports(mod)) {\n if(i.kind!=="function") throw new Error("unsupported test import "+JSON.stringify(i));\n (imports[i.module]??={})[i.name]=()=>{throw new Error("unexpected host call "+i.module+"."+i.name)};\n}\nconst instance=new WebAssembly.Instance(mod,imports);\nconst result=instance.exports.run();\nif(result!==648)throw new Error("native oracle case mismatch: "+result);\nconsole.log("648 native-oracle cases passed in wasm32 Node runtime");\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", type=Path, default=Path("/private/tmp/halo-lazy-width-wasm"))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    work = args.work_dir.resolve()
    work.mkdir(parents=True, exist_ok=True)
    rows = json.loads(zlib.decompress((root / "src/theater/fixtures/lazy-widths-v41.json.zlib").read_bytes()))
    assert len(rows) == 648
    entries = []
    for row in rows:
        entries.append("(" + ",".join([
            str(row["width"]) + "u64", str(row["mode"]), json.dumps(row["name"]),
            "&" + str(list(bytes.fromhex(row["hex"]))), str(row["start"]),
            str(row["ok"]).lower(), str(row["end"]), str(row["tail"]), str(row["tail_end"]),
        ]) + ")")
    (work / "lib.rs").write_text(RUST.replace("ENTRIES", ",\n".join(entries)))
    (work / "check.cjs").write_text(JAVASCRIPT)
    (work / "Cargo.toml").write_text(
        '[package]\nname="halo_lazy_width_wasm"\nversion="0.0.0"\nedition="2024"\n'
        '[lib]\npath="lib.rs"\ncrate-type=["cdylib"]\n[dependencies]\nhalo_api={path='
        + json.dumps(str(root)) + '}\n')
    shutil.copyfile(root / "Cargo.lock", work / "Cargo.lock")
    subprocess.run(["cargo", "build", "--offline", "--manifest-path", str(work / "Cargo.toml"),
                    "--target", "wasm32-unknown-unknown"], check=True)
    subprocess.run(["node", str(work / "check.cjs"),
                    str(work / "target/wasm32-unknown-unknown/debug/halo_lazy_width_wasm.wasm")], check=True)


if __name__ == "__main__":
    main()
