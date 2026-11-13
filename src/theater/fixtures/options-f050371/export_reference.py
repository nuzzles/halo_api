"""Execute unchanged f050371 Options APIs and export their exact public rows."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

PIN = "f050371a6cda83b9887eeb83a1884336fce28e53"
BLOBS = {
    "options.rs": "43ade2aafef5f38ca492b507e7adce03cc4be38d",
    "options_data.rs": "021e911b805a2b74c24321c7209462a76bd0a708",
}
source = Path(sys.argv[1]) / "src/game"
inputs = {}
for name, expected in BLOBS.items():
    raw = (source / name).read_bytes()
    actual = hashlib.sha1(f"blob {len(raw)}\0".encode() + raw).hexdigest()
    if actual != expected:
        raise SystemExit(f"unexpected {name} blob: {actual}")
    inputs[name] = raw

HARNESS = r'''
mod game;
use game::options::{self, OptionValue};
fn main() {
    for &(hash, name) in game::options_data::OPTION_NAMES {
        assert_eq!(options::option_name(hash), Some(name));
        println!("name\t{hash}\t{name}");
    }
    for mode in options::modes() {
        println!("mode\t{}\t{}", mode.prefix, mode.script);
        for option in mode.options {
            let default = match option.default {
                None => "-".into(),
                Some(OptionValue::Number(v)) => format!("n:{:016x}", v.to_bits()),
                Some(OptionValue::Bool(v)) => format!("b:{}", u8::from(v)),
            };
            println!("option\t{}\t{}\t{}", option.name, option.keys.join("\u{1f}"), default);
        }
    }
}
'''
with tempfile.TemporaryDirectory(prefix="halo-options-f050371-") as tmp:
    root = Path(tmp)
    (root / "game").mkdir()
    for name, raw in inputs.items():
        (root / "game" / name).write_bytes(raw)
    (root / "game/mod.rs").write_text("pub mod options; pub mod options_data;\n")
    (root / "main.rs").write_text(HARNESS)
    subprocess.run(["rustc", "--edition=2024", "-Awarnings", str(root / "main.rs"),
                    "-o", str(root / "oracle")], check=True)
    lines = subprocess.run([str(root / "oracle")], capture_output=True,
                           text=True, check=True).stdout.splitlines()

names, modes = [], []
for line in lines:
    fields = line.split("\t")
    if fields[0] == "name":
        names.append({"hash": int(fields[1]), "name": fields[2]})
    elif fields[0] == "mode":
        modes.append({"prefix": fields[1], "script": fields[2], "options": []})
    elif fields[0] == "option":
        default = fields[3]
        value = (None if default == "-" else
                 {"kind": "number", "bits": default[2:]} if default.startswith("n:") else
                 {"kind": "bool", "value": default == "b:1"})
        modes[-1]["options"].append({"name": fields[1],
                                      "keys": fields[2].split("\x1f"), "default": value})
    else:
        raise ValueError(fields)
print(json.dumps({"commit": PIN, "blobs": BLOBS, "option_names": names,
                  "modes": modes}, indent=2))
