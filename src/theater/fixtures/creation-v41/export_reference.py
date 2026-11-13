"""Trace pinned upstream object/manager creation readers; instrument bit consumption."""
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

PIN = '18067987ebc3628ad499c819c6aee814692df43b'


def main():
    cases = json.loads(Path(__file__).with_name('cases.json').read_text())
    archive = subprocess.check_output(['git', '-C', sys.argv[1], 'archive', PIN])
    with tempfile.TemporaryDirectory(prefix='filmshell-creation-oracle-') as tmp:
        root = Path(tmp)
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(root, filter='data')
        path = root / 'src/events.rs'
        source = path.read_text()
        signature = 'pub fn bits(&mut self, n: u32) -> u32 {'
        assert source.count(signature) == 1
        source = source.replace(signature, signature + '\n #[cfg(test)] crate::creation::creation_oracle::trace(self.at, n);')
        path.write_text(source)
        path = root / 'src/creation.rs'
        source = path.read_text()
        for original, replacement in [
            ('Form::Snapshot => r.at += 96,', 'Form::Snapshot => { #[cfg(test)] creation_oracle::trace(r.at, 96); r.at += 96; },'),
            ('r.at += 3 * 13;', '#[cfg(test)] creation_oracle::trace(r.at, 39); r.at += 3 * 13;'),
            ('r.at += i64::from(property_bits(kind));', '#[cfg(test)] creation_oracle::trace(r.at, property_bits(kind)); r.at += i64::from(property_bits(kind));')
        ]:
            assert source.count(original) == 1
            source = source.replace(original, replacement)
        source += r'''
#[cfg(test)]
pub(crate) mod creation_oracle {
    use super::*;
    use std::cell::RefCell;
    thread_local! { static TRACE: RefCell<Vec<(i64,u32)>> = const { RefCell::new(Vec::new()) }; }
    pub(crate) fn trace(at:i64, width:u32) { TRACE.with(|t| t.borrow_mut().push((at,width))); }
    #[test]
    fn export_creation() {
        for line in include_str!("creation-cases.tsv").lines() {
            let values:Vec<_> = line.split('\t').collect();
            let data:Vec<_> = values[3].as_bytes().chunks(2).map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(),16).unwrap()).collect();
            let form = if values[2] == "true" { Form::Snapshot } else { Form::Tick([15,15,17].into()) };
            TRACE.with(|t| t.borrow_mut().clear());
            let entity_type = values[1].parse().unwrap();
            let creation_end = if entity_type < 35 || entity_type == 47 || entity_type == 49 {
                manager_end(&data, 0, entity_type)
            } else {
                read(&data, 0, entity_type, form).map(|c| c.end)
            };
            let trace = TRACE.with(|t| t.borrow().clone());
            let end = trace.last().map_or(0, |(s,w)| s + i64::from(*w));
            if let Some(expected_end) = creation_end { assert_eq!(expected_end, end); }
            let trace = trace.iter().map(|(s,w)| format!("[{s},{w}]")).collect::<Vec<_>>().join(",");
            println!("ORACLE\t{}\t{}\t{}\t[{}]", values[0], end, creation_end.is_some(), trace);
        }
    }
}
'''
        path.write_text(source)
        (root / 'src/creation-cases.tsv').write_text('\n'.join(
            f"{c['name']}\t{c['entity_type']}\t{str(c['snapshot']).lower()}\t{c['hex']}" for c in cases) + '\n')
        output = subprocess.check_output(['cargo', '+stable', 'test', '--offline', '--lib', 'export_creation', '--', '--nocapture'], cwd=root, text=True)
        by_name = {c['name']: c for c in cases}
        rows = []
        for line in output.splitlines():
            if not line.startswith('ORACLE\t'):
                continue
            _, name, end, complete, trace = line.split('\t')
            case = by_name[name]
            if (int(end), complete == 'true') != (case['bit_len'], case['complete']):
                raise ValueError(f"{name}: upstream {end}/{complete}, authored {case['bit_len']}/{case['complete']}")
            rows.append(dict(name=name, end=int(end), complete=complete == 'true', trace=json.loads(trace)))
        assert len(rows) == len(cases) and rows
        print(json.dumps(dict(repository='https://github.com/OpenSpartan/libfilmshell', commit=PIN, cases=rows), indent=2))


if __name__ == '__main__':
    main()
