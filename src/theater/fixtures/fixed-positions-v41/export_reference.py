"""Execute original fixed-bound/width functions and LevelBox::engine on authored codes."""
from pathlib import Path
import hashlib
import json
import os
import random
import re
import subprocess
import sys
import tempfile

LEVELUP = "413971bcc485be53a2fa8cf229e170539a5146f8"
LIBFILMSHELL = "8f0f8d078e513c800baa21fac416fd4c0c127d03"
BLOBS = {
    "profile/loi_largeurs.go": "623a55741a4578c57bacb3a7abf47fffe8d5737d",
    "profile/plages_quant.go": "185a6387d1852015f89e1363b9bf1475fd2b292a",
    "lecteur_position.go": "6033b0366c28cf899883ab35681ba4eb1f5f8b16",
    "src/game/levels.rs": "13d578d46185d47a51a6867322a0849b149c460a",
}
grammar, libfilmshell = map(Path, sys.argv[1:3])
go = sys.argv[3] if len(sys.argv) > 3 else "go"
sources = {}
for name, expected in BLOBS.items():
    root = libfilmshell if name.startswith("src/") else grammar
    data = (root / name).read_bytes()
    assert hashlib.sha1(f"blob {len(data)}\0".encode() + data).hexdigest() == expected, name
    sources[name] = data.decode()

def block(source, prefix):
    start = source.index(prefix)
    end = source.index("{", start) + 1
    depth = 1
    while depth:
        depth += (source[end] == "{") - (source[end] == "}")
        end += 1
    return source[start:end]

# The original +/-100 constant is the bound passed by lireF85880; the harness
# supplies its three-axis envelope and asks the unchanged width law for level 16.
local_bound = re.search(r"const bornePrecHaut = float32\(\d+\)", sources["lecteur_position.go"]).group()
go_main = r'''
package main
import ("encoding/json"; "math"; "os"; "fixedoracle/profile")
BOUND
type Row struct { Domain string `json:"domain"`; MinBits [3]uint32 `json:"min_bits"`; MaxBits [3]uint32 `json:"max_bits"`; Widths [3]uint `json:"widths"` }
func main() {
    var rows []Row
    for _, domain := range []string{"default_world", "parent_local"} {
        b := profile.BornesParDefautDuBuild()
        if domain == "parent_local" { for i := range b { b[i] = [2]float32{-bornePrecHaut, bornePrecHaut} } }
        r := Row{Domain: domain, Widths: profile.LargeursAxeDuNiveau(b, 16)}
        for i := range b { r.MinBits[i] = math.Float32bits(b[i][0]); r.MaxBits[i] = math.Float32bits(b[i][1]) }
        rows = append(rows, r)
    }
    if err := json.NewEncoder(os.Stdout).Encode(rows); err != nil { panic(err) }
}
'''.replace("BOUND", local_bound)
rust_source = sources["src/game/levels.rs"]
rust_main = r'''
use std::io::BufRead;
fn main() {
    for line in std::io::stdin().lock().lines() {
        let a: Vec<u32> = line.unwrap().split_whitespace().map(|s| s.parse().unwrap()).collect();
        let b = LevelBox { index: 0, min: [f32::from_bits(a[0]); 3], max: [f32::from_bits(a[1]); 3], widths: [a[2]; 3] };
        println!("{} {} {}", b.engine(0, a[3]).to_bits(), b.engine(1, a[4]).to_bits(), b.engine(2, a[5]).to_bits());
    }
}
'''
with tempfile.TemporaryDirectory(prefix="halo-fixed-position-oracle-") as tmp:
    root = Path(tmp)
    (root / "profile").mkdir()
    for name in ["profile/loi_largeurs.go", "profile/plages_quant.go"]:
        (root / name).write_text(sources[name])
    (root / "go.mod").write_text("module fixedoracle\n\ngo 1.23\n")
    (root / "main.go").write_text(go_main)
    env = dict(os.environ, GOTOOLCHAIN="local", GOPROXY="off", GOWORK="off", GOCACHE=str(root / "gocache"))
    domains = json.loads(subprocess.check_output([go, "run", "."], cwd=root, env=env, text=True))
    cases = []
    rng = random.Random(413971)
    for domain in domains:
        n = 1 << domain["widths"][0]
        edges = [0, 1, 2, n // 4, n // 2 - 2, n // 2 - 1, n // 2, n // 2 + 1, n - 3, n - 2, n - 1]
        codes = [[q, n - 1 - q, edges[(i + 3) % len(edges)]] for i, q in enumerate(edges)]
        codes += [[rng.randrange(n) for _ in range(3)] for _ in range(64)]
        for i, q in enumerate(codes):
            cases.append(dict(name=f"{domain['domain']}/{i}", domain=domain["domain"], codes=q, widths=domain["widths"]))
    (root / "main.rs").write_text(block(rust_source, "pub struct LevelBox") + "\n" + block(rust_source, "impl LevelBox") + rust_main)
    subprocess.run(["rustc", "--edition=2024", str(root / "main.rs"), "-o", str(root / "oracle")], check=True)
    by_domain = {d["domain"]: d for d in domains}
    lines = []
    for case in cases:
        d = by_domain[case["domain"]]
        lines.append(" ".join(map(str, [d["min_bits"][0], d["max_bits"][0], d["widths"][0], *case["codes"]])))
    output = subprocess.check_output([str(root / "oracle")], input="\n".join(lines) + "\n", text=True).splitlines()
    assert len(output) == len(cases)
    for case, line in zip(cases, output):
        case["midpoint_bits"] = list(map(int, line.split()))
print(json.dumps(dict(levelup=LEVELUP, libfilmshell=LIBFILMSHELL, blobs=BLOBS, domains=domains, cases=cases), indent=2))
