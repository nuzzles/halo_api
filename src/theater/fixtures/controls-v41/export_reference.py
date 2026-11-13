"""Export control read/skip boundaries from a pinned, disposable upstream copy.

Usage: python3 export_reference.py /path/to/libfilmshell [cases.json] > reference.json
Defaults to adjacent cases.json; never loads halo_api or changes the source checkout.
"""
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

PIN = "18067987ebc3628ad499c819c6aee814692df43b"


def main():
    cases_path = Path(sys.argv[2]) if len(sys.argv) > 2 else Path(__file__).with_name("cases.json")
    cases = json.loads(cases_path.read_text())
    archive = subprocess.check_output(["git", "-C", sys.argv[1], "archive", PIN])
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
    for line in include_str!("control-cases.tsv").lines() {
        let values:Vec<_> = line.split('\t').collect();
        let data:Vec<_> = values[1].as_bytes().chunks(2).map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(),16).unwrap()).collect();
        let mut p = crate::components::Params::new([15,15,17]);
        p.box_bits = values[2].parse().unwrap();
        if values[3] == "region3" { p.box_widths[3] = [19,19,16]; }
        TRACE.with(|t| t.borrow_mut().clear());
        let (_,end) = crate::components::control_view(&data,0,&p);
        let end = end.expect("fixture must finish in upstream");
        let trace = TRACE.with(|t| t.borrow().iter().map(|(s,w)| format!("[{s},{w}]")).collect::<Vec<_>>().join(","));
        println!("ORACLE\t{}\t{}\t[{}]", values[0],end,trace);
    }
}
'''
        (root / "src/control_oracle.rs").write_text(module)
        (root / "src/control-cases.tsv").write_text("\n".join(
            f"{c['name']}\t{c['hex']}\t{c['index_bits']}\t{'region3' if c['region_widths'] else 'default'}"
            for c in cases
        ) + "\n")
        with (root / "src/lib.rs").open("a") as f:
            f.write("\n#[cfg(test)] mod control_oracle;\n")
        # Resolve cached dependencies only inside this disposable harness. Reader
        # source remains pinned; no lockfile in either checkout is changed.
        (root / "Cargo.lock").unlink(missing_ok=True)
        result = subprocess.check_output([
            "cargo", "+stable", "test", "--offline", "--config", "build.incremental=false",
            "--lib", "export_controls", "--", "--nocapture"
        ], cwd=root, text=True)
        rows = []
        for line in result.splitlines():
            if line.startswith("ORACLE\t"):
                _, name, end, trace = line.split("\t")
                case = next(c for c in cases if c["name"] == name)
                if int(end) != case["bit_len"]:
                    raise ValueError(f"{name}: upstream ends at {end}, authored layout at {case['bit_len']}")
                rows.append({"name": name, "end": int(end), "trace": json.loads(trace)})
        if len(rows) != len(cases) or not rows:
            raise ValueError("Incomplete upstream oracle run")
        print(json.dumps({"repository": "https://github.com/OpenSpartan/libfilmshell", "commit": PIN, "cases": rows}, indent=2))


if __name__ == "__main__":
    main()
