"""Execute unchanged pinned zone event tables, without reading halo_api output.
Usage: python3 export_reference.py PINNED_LIBFILMSHELL > reference.json
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

PIN = '8f0f8d078e513c800baa21fac416fd4c0c127d03'
BLOB = '4701db66d587260404006301be25b4fb2ae1931f'
raw = (Path(sys.argv[1]) / 'src/zones.rs').read_bytes()
assert hashlib.sha1(f'blob {len(raw)}\0'.encode() + raw).hexdigest() == BLOB
source = raw.decode()
# Exact contiguous source section: constants, Mode, About, table and ZONED.
section = source[source.index('/// Event kinds, as serialized.'):source.index('/// A PvP King of the Hill')]
main = r'''
fn main() {
    for (name, mode) in [("Strongholds", Mode::Strongholds),
        ("FirefightKingOfTheHill", Mode::FirefightKoth), ("Oddball", Mode::Oddball),
        ("KingOfTheHill", Mode::Koth)] {
        for id in 0..1024 {
            if let Some(&(_, kind, about)) = table(mode).iter().find(|r| r.0 == id) {
                println!("{}\t{}\t{}\t{:?}\t{}", name, id, kind, about, ZONED.contains(&kind));
            }
        }
    }
}
'''
with tempfile.TemporaryDirectory(prefix='halo-zone-oracle-') as tmp:
    p = Path(tmp)
    (p / 'main.rs').write_text(section + main)
    subprocess.run(['rustc', '--edition=2024', '-Awarnings', str(p / 'main.rs'), '-o', str(p / 'oracle')], check=True)
    output = subprocess.check_output([str(p / 'oracle')], text=True)
rows = []
for line in output.splitlines():
    mode, event_id, kind, audience, names_zone = line.split('\t')
    rows.append(dict(mode=mode, id=int(event_id), kind=int(kind), audience=audience, names_zone=names_zone == 'true'))
print(json.dumps(dict(commit=PIN, blob=BLOB, queried_ids=[0, 1024], rows=rows), indent=2))
