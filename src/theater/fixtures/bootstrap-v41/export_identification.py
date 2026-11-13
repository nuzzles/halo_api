"""Trace pinned libfilmshell header::read on independent identification fields."""
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import zlib

PIN = '18067987ebc3628ad499c819c6aee814692df43b'


def main():
    here = Path(__file__).parent
    cases = json.loads((here / 'identification.json').read_text())
    prefix = zlib.decompress((here / 'registry-prefix.bin.zlib').read_bytes())
    archive = subprocess.check_output(['git', '-C', sys.argv[1], 'archive', PIN])
    with tempfile.TemporaryDirectory(prefix='filmshell-identification-') as tmp:
        root = Path(tmp)
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(root, filter='data')
        path = root / 'src/header.rs'
        source = path.read_text()
        signature = 'fn bytes(c: &[u8], at: i64, n: usize) -> Vec<u8> {'
        assert source.count(signature) == 1
        source = source.replace(signature, signature + '\n #[cfg(test)] oracle::trace(at, n * 8);')
        source += r'''
#[cfg(test)]
mod oracle {
    use super::*;
    use std::cell::RefCell;
    thread_local! { static TRACE: RefCell<Vec<(i64,usize)>> = const { RefCell::new(Vec::new()) }; }
    pub fn trace(at: i64, n: usize) { TRACE.with(|t| t.borrow_mut().push((at,n))); }
    #[test]
    fn export_identification() {
        for line in include_str!("identification.tsv").lines() {
            let values: Vec<_> = line.split('\t').collect();
            let data: Vec<_> = values[1].as_bytes().chunks(2).map(|p|
                u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect();
            TRACE.with(|t| t.borrow_mut().clear());
            let h = read(&data);
            let id = |x: Option<[u8;16]>| x.map(|b| b.iter().map(|v|format!("{v:02x}")).collect::<String>()).unwrap_or_else(|| "-".into());
            let start = h.start.map(|v|v.to_string()).unwrap_or_else(|| "-".into());
            let trace = TRACE.with(|t| t.borrow().iter().map(|(at,n)|format!("[{at},{n}]")).collect::<Vec<_>>().join(","));
            println!("ORACLE\t{}\t{}\t{}\t{}\t{}\t{}\t[{}]", values[0], h.build.unwrap(), h.branch.unwrap(), start, id(h.map_asset_id), id(h.map_version_id), trace);
        }
    }
}
'''
        path.write_text(source)
        (root / 'src/identification.tsv').write_text('\n'.join(
            f"{c['name']}\t{prefix.hex()}{c['hex']}" for c in cases) + '\n')
        output = subprocess.check_output(['cargo', '+stable', 'test', '--offline', '--lib',
                                          'export_identification', '--', '--nocapture'], cwd=root, text=True)
        rows = []
        for line in output.splitlines():
            if not line.startswith('ORACLE\t'):
                continue
            _, name, build, branch, start, asset, version, trace = line.split('\t')
            case = cases[len(rows)]
            assert name == case['name']
            assert start == (str(case['timestamp']) if case['timestamp'] else '-')
            for actual, key in [(asset, 'asset_hex'), (version, 'version_hex')]:
                assert actual == ('-' if int(case[key], 16) == 0 else case[key])
            trace = [[at - len(prefix) * 8, width] for at, width in json.loads(trace)]
            expected = [[f['bit'], f['width']] for f in case['fields'] if f['kind'] in ['StartTime', 'MapAssetId', 'MapVersionId']]
            assert trace == expected
            rows.append(dict(name=name, build=build, branch=branch, timestamp=None if start == '-' else int(start),
                             asset=None if asset == '-' else asset, version=None if version == '-' else version, trace=trace))
        assert len(rows) == len(cases)
        print(json.dumps(dict(commit=PIN, cases=rows), indent=2))


if __name__ == '__main__':
    main()
