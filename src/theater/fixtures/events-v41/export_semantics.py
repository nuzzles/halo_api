"""Serialize the unchanged pinned upstream event reader's retained values.

Only test serialization derives/harness are added to a temporary Git archive.
No production reader or field interpretation is replaced.
"""
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

PIN = '18067987ebc3628ad499c819c6aee814692df43b'
cases = [c for c in json.loads(Path(__file__).with_name('cases.json').read_text()) if c['complete']]
cases += json.loads(Path(__file__).with_name('semantic-extra.json').read_text())
archive = subprocess.check_output(['git', '-C', sys.argv[1], 'archive', PIN])
with tempfile.TemporaryDirectory(prefix='filmshell-event-semantics-') as tmp:
    root = Path(tmp)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        tar.extractall(root, filter='data')
    path = root / 'src/events.rs'
    source = path.read_text()
    source = source.replace('#[derive(Clone, Copy, Debug, PartialEq, Eq)]', '#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]')
    source += r'''
#[cfg(test)]
mod semantic_oracle {
    use super::*;
    #[test]
    fn export_semantics() {
        for line in include_str!("event-semantics.tsv").lines() {
            let parts:Vec<_> = line.split('\t').collect();
            let data:Vec<_> = parts[1].as_bytes().chunks(2).map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect();
            let mut r = Bits { c: &data, at: parts[2].parse().unwrap() };
            let mut events = Vec::new();
            walk(&mut r, [15,15,17].into(), &mut events).unwrap();
            println!("SEMANTIC\t{}\t{}", parts[0], serde_json::to_string(&events[0]).unwrap());
        }
    }
}
'''
    path.write_text(source)
    path = root / 'Cargo.toml'
    path.write_text(path.read_text() + '\n[dev-dependencies]\nserde_json = "1"\n')
    (root / 'src/event-semantics.tsv').write_text('\n'.join(f"{c['name']}\t{c['hex']}\t{c.get('start_bit', 0)}" for c in cases))
    out = subprocess.check_output(['cargo', '+stable', 'test', '--offline', '--features', 'serde', '--lib', 'export_semantics', '--', '--nocapture'], cwd=root, text=True)
rows = []
for line in out.splitlines():
    if line.startswith('SEMANTIC\t'):
        _, name, event = line.split('\t')
        rows.append(dict(name=name, event=json.loads(event)))
assert len(rows) == len(cases)
print(json.dumps(dict(commit=PIN, cases=rows), indent=2))
