"""Export control read/skip boundaries from a pinned, disposable upstream copy.

Usage: python3 export_captured.py /path/to/libfilmshell cases.json COMMIT > reference.json
Requires captured case JSON and an explicit old/new pin; never changes source checkouts.
"""
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import tomllib

PIN = "f050371a6cda83b9887eeb83a1884336fce28e53"


def main():
    cases_path = Path(sys.argv[2])
    pin = sys.argv[3]
    if pin not in [PIN, "18067987ebc3628ad499c819c6aee814692df43b"]:
        raise ValueError("Expected the original or f050371 reader pin")
    cases = json.loads(cases_path.read_text())
    archive = subprocess.check_output(["git", "-C", sys.argv[1], "archive", pin])
    with tempfile.TemporaryDirectory(prefix="filmshell-control-oracle-") as tmp:
        root = Path(tmp)
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(root, filter="data")
        component = root / "src/components.rs"
        source = component.read_text()
        for signature in ["fn bits(&mut self, w: u32) -> u32 {", "fn skip(&mut self, w: u32) {"]:
            if source.count(signature) != 1:
                raise ValueError(f"Unexpected upstream reader: {signature}")
            source = source.replace(signature, signature + "\n crate::control_oracle::trace(self.at, w);")
        component.write_text(source)
        module = r'''
use std::cell::RefCell;
thread_local! { static TRACE: RefCell<Vec<(i64,u32)>> = const { RefCell::new(Vec::new()) }; }
pub(crate) fn trace(at:i64, width:u32) { TRACE.with(|t| t.borrow_mut().push((at,width))); }
#[test]
fn export_controls() {
    let rows:Vec<serde_json::Value> = serde_json::from_str(include_str!("control-cases.json")).unwrap();
    for row in rows {
        let data:Vec<_> = row["hex"].as_str().unwrap().as_bytes().chunks(2).map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(),16).unwrap()).collect();
        let mut p = crate::components::Params::new(serde_json::from_value(row["widths"].clone()).unwrap());
        p.box_bits = row["index_bits"].as_u64().unwrap() as u32;
        p.box_widths = serde_json::from_value(row["boxes"].clone()).unwrap();
        TRACE.with(|t| t.borrow_mut().clear());
        let (messages,end) = crate::components::control_view(&data,0,&p);
        let trace = TRACE.with(|t| t.borrow().clone());
        let result = serde_json::json!({"name":row["name"],"end":end,"messages":messages.len(),"trace":trace});
        println!("ORACLE\t{}",result);
    }
}
'''
        (root / "src/control_oracle.rs").write_text(module)
        (root / "src/control-cases.json").write_text(json.dumps(cases))
        with (root / "src/lib.rs").open("a") as f:
            f.write("\n#[cfg(test)] mod control_oracle;\n")
        # Resolve cached dependencies only inside this disposable harness. Reader
        # source remains pinned; no lockfile in either checkout is changed.
        manifest = root / "Cargo.toml"
        if "serde_json" not in tomllib.loads(manifest.read_text()).get("dev-dependencies", {}):
            with manifest.open("a") as f:
                f.write('\n[dev-dependencies.serde_json]\nversion = "1"\n')
        (root / "Cargo.lock").unlink(missing_ok=True)
        result = subprocess.check_output([
            "cargo", "+stable", "test", "--offline", "--config", "build.incremental=false",
            "--lib", "export_controls", "--", "--nocapture"
        ], cwd=root, text=True)
        rows = [json.loads(line.split("\t", 1)[1]) for line in result.splitlines() if line.startswith("ORACLE\t")]
        if len(rows) != len(cases) or [r["name"] for r in rows] != [r["name"] for r in cases]:
            raise ValueError("Incomplete upstream capture oracle run")
        print(json.dumps({"repository": "https://github.com/OpenSpartan/libfilmshell", "commit": pin, "cases": rows}, indent=2))



if __name__ == "__main__":
    main()
