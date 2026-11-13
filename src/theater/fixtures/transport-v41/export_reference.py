"""Check the independent streams/limits using pinned upstream inflate::zlib."""
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

PIN = '18067987ebc3628ad499c819c6aee814692df43b'
archive = subprocess.check_output(['git', '-C', sys.argv[1], 'archive', PIN])
with tempfile.TemporaryDirectory(prefix='filmshell-transport-oracle-') as tmp:
    root = Path(tmp)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        tar.extractall(root, filter='data')
    for name in ['fixed', 'dynamic']:
        (root / f'src/{name}.zlib').write_bytes(Path(__file__).with_name(f'{name}.zlib').read_bytes())
    path = root / 'src/lib.rs'
    path.write_text(path.read_text() + r'''
#[cfg(test)]
mod transport_oracle {
    #[test]
    fn export_transport() {
        let expected = b"abcde0123456789".repeat(2048);
        assert_eq!(crate::chunk::MAX_CHUNK_BYTES, 256 << 20);
        for (name, wire) in [("fixed", &include_bytes!("fixed.zlib")[..]), ("dynamic", &include_bytes!("dynamic.zlib")[..])] {
            assert!(crate::chunk::is_zlib(wire));
            for limit in [0,1,8191,8192,8193,30719,30720,30721] {
                let result = crate::inflate::zlib(wire, limit);
                let status = match result {
                    Ok(bytes) => { assert_eq!(bytes, expected); "complete" },
                    Err(crate::inflate::Fault::TooLong) => "limit",
                    Err(crate::inflate::Fault::Malformed) => panic!("valid independent stream refused"),
                };
                println!("ORACLE\t{name}\t{limit}\t{status}");
            }
        }
    }
}
''')
    output = subprocess.check_output(['cargo', '+stable', 'test', '--offline', '--lib', 'export_transport', '--', '--nocapture'], cwd=root, text=True)
    rows = []
    for line in output.splitlines():
        if line.startswith('ORACLE\t'):
            _, name, limit, status = line.split('\t')
            limit = int(limit)
            assert status == ('limit' if limit < 30720 else 'complete')
            rows.append(dict(name=name, limit=limit, status=status))
    assert len(rows) == 16
    print(json.dumps(dict(repository='https://github.com/OpenSpartan/libfilmshell', commit=PIN, cases=rows), indent=2))
