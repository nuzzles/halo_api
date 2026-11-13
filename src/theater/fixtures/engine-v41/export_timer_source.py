"""Export every round-timer quantum through unchanged, blob-checked LevelUp readers.

Usage: GO_BIN=... GOCACHE=... python3 export_timer_source.py PINNED_TREE > timer.f32le
The bounded bit reader is a harness; decoder, timer helper and conversion are
original source. Each output word is the little-endian IEEE754 float32 encoding
for the corresponding u16 quantum. Both timer words and all five-bit states
are exercised. No halo_api implementation supplies expected values.
Execution is pinned to amd64/v1 to preserve separate scalar float32 rounding.
Go's ARM64 compiler contracts multiply/add into FMA, changing 21,018 outputs.
On ARM macOS this requires the system's x86 execution support (Rosetta).
"""
import hashlib
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

PIN = '413971bcc485be53a2fa8cf229e170539a5146f8'
BLOBS = {
    'quantize_endpoint.go': '5d7c33a94b6a73012a07d6b74c5da7bfa2469a2c',
    'vitality.go': '2caa36888f53b767db48bf16b5a391fe7fa4b205',
    'lecteur_minuteur.go': 'dc382adaa1bbfe38e4f8004f094aaf04da636bd6',
}
SELECT = {
    'quantize_endpoint.go': ['func DequantEndpoint('],
    'vitality.go': ['type RoundTimer struct', 'func decodeGameEngineRoundTimer('],
    'lecteur_minuteur.go': ['type Minuteur struct', 'func lireMinuteur140d580d0('],
}


def extract(source, prefix):
    start = source.index(prefix)
    opening = source.index('{', start)
    depth, end = 1, opening + 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[start:end]


selected = []
for name, blob in BLOBS.items():
    raw = (Path(sys.argv[1]) / name).read_bytes()
    assert hashlib.sha1(f'blob {len(raw)}\0'.encode() + raw).hexdigest() == blob
    source = raw.decode()
    selected.extend(extract(source, prefix) for prefix in SELECT[name])
    if name == 'quantize_endpoint.go':
        for constant in ['RoundTimerMax', 'roundTimerBits']:
            selected.append('const ' + re.search(r'^\s*' + constant + r'\s+\w+\s*=\s*\d+', source, re.M).group().strip())
    if name == 'lecteur_minuteur.go':
        selected.append(re.search(r'^const largeurQueueMinuteur uint = \d+$', source, re.M).group())

HARNESS = r'''
package main
import("encoding/binary";"math";"os")
type Lecteur struct{ data uint64; at uint }
func(r *Lecteur) ReadBits(n uint)uint64{
 if n>37-r.at{panic("overread")};r.at+=n
 return (r.data>>(37-r.at))&((uint64(1)<<n)-1)
}
func main(){
 words:=make([]byte,65536*4)
 for q:=uint64(0);q<65536;q++{
  for tail:=uint64(0);tail<32;tail++{
   r:=&Lecteur{data:q<<21|(65535-q)<<5|tail}
   v:=decodeGameEngineRoundTimer(r)
   if r.at!=37 || v.QA!=uint16(q) || v.QB!=uint16(65535-q) || v.Tail!=uint8(tail){panic("reader mismatch")}
   a:=math.Float32bits(v.A);b:=math.Float32bits(v.B)
   if b!=math.Float32bits(DequantEndpoint(65535-q,0,RoundTimerMax,roundTimerBits,false,true)){panic("second word")}
   binary.LittleEndian.PutUint32(words[q*4:],a)
  }
 }
 if _,e:=os.Stdout.Write(words);e!=nil{panic(e)}
}
'''
with tempfile.TemporaryDirectory(prefix='halo-timer-source-') as tmp:
    root = Path(tmp)
    (root / 'main.go').write_text(HARNESS)
    (root / 'source.go').write_text('package main\n' + '\n\n'.join(selected))
    output = subprocess.check_output([
        os.environ.get('GO_BIN', 'go'), 'run', 'main.go', 'source.go',
    ], cwd=root, env={**os.environ, 'GOARCH': 'amd64', 'GOAMD64': 'v1', 'CGO_ENABLED': '0'})
assert len(output) == 65536 * 4
sys.stdout.buffer.write(output)
