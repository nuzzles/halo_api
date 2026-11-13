"""Run pinned upstream event grammar; instrument primitive reads/skips only."""
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

PIN = '18067987ebc3628ad499c819c6aee814692df43b'


def main(cases=None):
    if cases is None:
        cases = json.loads(Path(__file__).with_name('cases.json').read_text())
    archive = subprocess.check_output(['git', '-C', sys.argv[1], 'archive', PIN])
    with tempfile.TemporaryDirectory(prefix='filmshell-event-oracle-') as tmp:
        root = Path(tmp)
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(root, filter='data')
        path = root / 'src/events.rs'
        source = path.read_text()
        signature = 'pub fn bits(&mut self, n: u32) -> u32 {'
        assert source.count(signature) == 1
        source = source.replace(signature, signature + '\n #[cfg(test)] event_oracle::trace(self.at, n);')
        source += r'''
#[cfg(test)]
pub(crate) mod event_oracle {
    use super::*;
    use std::cell::RefCell;
    thread_local! { static TRACE: RefCell<Vec<(i64,u32)>> = const { RefCell::new(Vec::new()) }; }
    pub(crate) fn trace(at:i64, width:u32) { TRACE.with(|t| t.borrow_mut().push((at,width))); }
    #[test]
    fn export_events() {
        for code in 0..123 {
            println!("KINDS\t{code}\t{:?}", handle_kinds(code).unwrap());
        }
        for line in include_str!("event-cases.tsv").lines() {
            let values:Vec<_> = line.split('\t').collect();
            let data:Vec<_> = values[1].as_bytes().chunks(2).map(|x| u8::from_str_radix(std::str::from_utf8(x).unwrap(),16).unwrap()).collect();
            let mut r = Bits { c: &data, at: values[2].parse().unwrap() };
            TRACE.with(|t| t.borrow_mut().clear());
            let mut events = Vec::new();
            let complete = walk(&mut r, [15,15,17].into(), &mut events).is_ok();
            let trace = TRACE.with(|t| t.borrow().clone());
            // A refused nested action leaves the parent's cursor unchanged;
            // its successful prefix reads are still independently observed.
            let end = trace.last().map_or(r.at, |(s,w)| s + i64::from(*w));
            let trace = trace.iter().map(|(s,w)| format!("[{s},{w}]")).collect::<Vec<_>>().join(",");
            println!("ORACLE\t{}\t{}\t{}\t{}\t{}\t[{}]", values[0], end, r.at, complete, events.len(), trace);
        }
    }
}
'''
        path.write_text(source)
        path = root / 'src/components.rs'
        source = path.read_text()
        for signature in ['fn bits(&mut self, w: u32) -> u32 {', 'fn skip(&mut self, w: u32) {']:
            assert source.count(signature) == 1
            source = source.replace(signature, signature + '\n #[cfg(test)] crate::events::event_oracle::trace(self.at, w);')
        path.write_text(source)
        (root / 'src/event-cases.tsv').write_text('\n'.join(f"{c['name']}\t{c['hex']}\t{c.get('start_bit', 0)}" for c in cases) + '\n')
        # Resolve cached compatible dependencies only in this disposable archive.
        (root / 'Cargo.lock').unlink(missing_ok=True)
        output = subprocess.check_output(['cargo', '+stable', 'test', '--offline', '--config', 'build.incremental=false', '--lib', 'export_events', '--', '--nocapture'], cwd=root, text=True)
        by_name = {c['name']: c for c in cases}
        rows, kinds = [], []
        for line in output.splitlines():
            if line.startswith('KINDS\t'):
                _, code, raw = line.split('\t')
                assert int(code) == len(kinds)
                kinds.append(json.loads(raw))
            elif line.startswith('ORACLE\t'):
                _, name, end, reader_end, complete, count, trace = line.split('\t')
                case = by_name[name]
                expected = (case['bit_len'], case['complete'], 2 if case['complete'] else 0)
                actual = (int(end), complete == 'true', int(count))
                if actual != expected:
                    raise ValueError(f'{name}: upstream {actual}, authored {expected}')
                rows.append(dict(name=name, end=int(end), reader_end=int(reader_end), complete=complete == 'true', count=int(count), trace=json.loads(trace)))
        assert len(rows) == len(cases) and len(kinds) == 123
        print(json.dumps(dict(repository='https://github.com/OpenSpartan/libfilmshell', commit=PIN, handle_kinds=kinds, cases=rows), indent=2))


if __name__ == '__main__':
    main()
