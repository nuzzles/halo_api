"""Execute the current libfilmshell StatDisplay and StatDefinition::shown APIs."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

PIN = "f050371a6cda83b9887eeb83a1884336fce28e53"
BLOBS = {
    "src/model/variant.rs": "e07071372a506302f7def1a6e7afd488539fe704",
    "src/game/options.rs": "43ade2aafef5f38ca492b507e7adce03cc4be38d",
    "src/game/options_data.rs": "021e911b805a2b74c24321c7209462a76bd0a708",
}
root = Path(sys.argv[1])
inputs = {}
for name, expected in BLOBS.items():
    raw = (root / name).read_bytes()
    actual = hashlib.sha1(f"blob {len(raw)}\0".encode() + raw).hexdigest()
    if actual != expected:
        raise SystemExit(f"unexpected {name} blob: {actual}")
    inputs[name] = raw

WRAPPER = r'''
pub fn display_case(code: i64) -> String {
    format!("{:?}", StatDisplay::from_game(code))
}
pub fn shown_case(precision: i64, stored: i32) -> u64 {
    StatDefinition {
        name: String::new(), id: 0, base: 0, aggregation: 0,
        lower_is_better: false, display: StatDisplay::Number, precision,
    }.shown(stored).to_bits()
}
'''
HARNESS = r'''
mod model; mod game;
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct TagId(pub u32);
fn main() {
    for code in [-2i64, -1, 0, 1, 2, 3, i64::MAX] {
        println!("display\t{}\t{}", code, model::display_case(code));
    }
    for precision in [-100i64, -1, 0, 1, 2, 9, 18, 19, 100] {
        for stored in [i32::MIN, -12345, -1, 0, 1, 12345, i32::MAX] {
            println!("shown\t{}\t{}\t{:016x}", precision, stored,
                model::shown_case(precision, stored));
        }
    }
}
'''
with tempfile.TemporaryDirectory(prefix="halo-stat-presentation-") as temp:
    project = Path(temp)
    (project / "game").mkdir()
    (project / "model.rs").write_bytes(inputs["src/model/variant.rs"] + WRAPPER.encode())
    for source in ["options.rs", "options_data.rs"]:
        (project / "game" / source).write_bytes(inputs["src/game/" + source])
    (project / "game/mod.rs").write_text(
        "mod options; mod options_data; pub use options::{OptionValue, murmur3, modes};\n"
    )
    (project / "main.rs").write_text(HARNESS)
    subprocess.run(["rustc", "--edition=2024", "-Awarnings", str(project / "main.rs"),
                    "-o", str(project / "oracle")], check=True)
    lines = subprocess.check_output([str(project / "oracle")], text=True).splitlines()
display, shown = [], []
for line in lines:
    kind, *values = line.split("\t")
    if kind == "display":
        code, name = values
        display.append({"code": int(code), "name": name})
    elif kind == "shown":
        precision, stored, bits = values
        shown.append({"precision": int(precision), "stored": int(stored), "bits": bits})
    else:
        raise AssertionError(line)
assert len(display) == 7 and len(shown) == 63
print(json.dumps({"commit": PIN, "blobs": BLOBS, "display": display, "shown": shown}, indent=2))
