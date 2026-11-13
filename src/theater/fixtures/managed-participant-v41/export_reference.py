"""Run the original f050371 creation::managed_player_participant function.

Usage: python3 export_reference.py /path/to/libfilmshell > reference.json
All instrumentation is added inside a disposable pinned Git archive.
"""
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

PIN = "f050371a6cda83b9887eeb83a1884336fce28e53"
ROOT = Path(__file__).parent


def main():
    cases = json.loads((ROOT / "cases.json").read_text())
    archive = subprocess.check_output(["git", "-C", sys.argv[1], "archive", PIN])
    with tempfile.TemporaryDirectory(prefix="halo-managed-participant-") as temp:
        checkout = Path(temp)
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(checkout, filter="data")
        (checkout / "src/managed_participant_oracle.rs").write_text(r'''
#[test]
fn export_managed_participants() {
    let rows: Vec<serde_json::Value> = serde_json::from_str(include_str!("managed-participant-cases.json")).unwrap();
    for row in rows {
        let bytes: Vec<u8> = row["hex"].as_str().unwrap().as_bytes().chunks(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()).collect();
        let at = row["at"].as_i64().unwrap();
        let value = crate::creation::managed_player_participant(&bytes, at);
        println!("ORACLE\t{}", serde_json::json!({"name": row["name"], "value": value}));
    }
}
''')
        (checkout / "src/managed-participant-cases.json").write_text(json.dumps(cases))
        with (checkout / "src/lib.rs").open("a") as lib:
            lib.write("\n#[cfg(test)] mod managed_participant_oracle;\n")
        # The pinned lockfile selects crates absent from the offline cache.
        # Resolve cached versions only in this disposable archive.
        (checkout / "Cargo.lock").unlink(missing_ok=True)
        run = subprocess.run(
            ["cargo", "test", "--offline", "--config", "build.incremental=false", "--lib", "export_managed_participants", "--", "--nocapture"],
            cwd=checkout, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        if run.returncode:
            raise RuntimeError(run.stderr[-6000:])
    rows = [json.loads(line.split("ORACLE\t", 1)[1]) for line in run.stdout.splitlines() if "ORACLE\t" in line]
    assert len(rows) == len(cases)
    for case, row in zip(cases, rows):
        assert row["name"] == case["name"]
    print(json.dumps(dict(repository="https://github.com/OpenSpartan/libfilmshell", commit=PIN, cases=rows), indent=2))


if __name__ == "__main__":
    main()
