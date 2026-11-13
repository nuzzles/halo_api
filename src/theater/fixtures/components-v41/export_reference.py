"""Export exact primitive boundaries from the pinned upstream component readers.

Usage: python3 export_reference.py /path/to/libfilmshell > /tmp/component-reference.json
Only upstream Reader::bits/skip are instrumented in a disposable archive.
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
    cases = json.loads(Path(__file__).with_name("cases.json").read_text())
    archive = subprocess.check_output(["git", "-C", sys.argv[1], "archive", PIN])
    with tempfile.TemporaryDirectory(prefix="filmshell-component-oracle-") as tmp:
        root = Path(tmp)
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(root, filter="data")
        component = root / "src/components.rs"
        source = component.read_text()
        for signature in ["fn bits(&mut self, w: u32) -> u32 {", "fn skip(&mut self, w: u32) {"]:
            assert source.count(signature) == 1, signature
            source = source.replace(signature, signature + "\n #[cfg(test)] component_oracle::trace(self.at, w);")
        source += r'''
#[cfg(test)]
mod component_oracle {
    use super::*;
    use std::cell::RefCell;
    thread_local! { static TRACE: RefCell<Vec<(i64,u32)>> = const { RefCell::new(Vec::new()) }; }
    pub(super) fn trace(at:i64, width:u32) { TRACE.with(|t| t.borrow_mut().push((at,width))); }
    #[test]
    fn export_components() {
        for line in include_str!("component-cases.tsv").lines() {
            let values:Vec<_> = line.split('\t').collect();
            let data:Vec<_> = values[3].as_bytes().chunks(2).map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(),16).unwrap()).collect();
            let version = values[2].parse().unwrap();
            let mut p = Params::new([15,15,17]);
            p.version = version;
            p.snapshot = values[4] == "true";
            let mut reader = Reader { c:&data, at:0, object_type:values[5].parse().unwrap(), slot:0, row_version:version };
            TRACE.with(|t| t.borrow_mut().clear());
            let complete = reader.block(Component::from_name(values[1]).expect("upstream component"), &p, &mut Fields::default()).is_some();
            let trace = TRACE.with(|t| t.borrow().iter().map(|(s,w)| format!("[{s},{w}]")).collect::<Vec<_>>().join(","));
            println!("ORACLE\t{}\t{}\t{}\t[{}]", values[0], reader.at, complete, trace);
        }
    }
}
'''
        component.write_text(source)
        (root / "src/component-cases.tsv").write_text("\n".join(
            f"{c['name']}\t{c['component']}\t{c['version']}\t{c['hex']}\t{str(c['snapshot']).lower()}\t{c.get('archetype',0)}" for c in cases
        ) + "\n")
        result = subprocess.check_output([
            "cargo", "+stable", "test", "--offline", "--lib", "export_components", "--", "--nocapture"
        ], cwd=root, text=True)
        rows = []
        by_name = {c['name']: c for c in cases}
        for line in result.splitlines():
            if not line.startswith("ORACLE\t"):
                continue
            _, name, end, complete, trace = line.split("\t")
            case = by_name[name]
            if (int(end), complete == 'true') != (case['bit_len'], case['complete']):
                raise ValueError(f"{name}: upstream ({end}, {complete}), authored ({case['bit_len']}, {case['complete']})")
            rows.append(dict(name=name, end=int(end), complete=complete == 'true', trace=json.loads(trace)))
        assert len(rows) == len(cases) and rows, "Incomplete oracle"
        print(json.dumps(dict(repository="https://github.com/OpenSpartan/libfilmshell", commit=PIN, cases=rows), indent=2))


if __name__ == '__main__':
    main()
