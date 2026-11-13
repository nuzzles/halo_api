"""Reproduce film/catalog/damage and film/catalog/physics data from pinned Git blobs.

python3 generate.py LIBFILMSHELL [OUTPUT_FILM_DIRECTORY]
Only imports and formatting attributes change. In particular,
all decimal f64 literals remain the original upstream reference values.
"""
import hashlib
from pathlib import Path
import subprocess
import sys

PIN = 'f050371a6cda83b9887eeb83a1884336fce28e53'
BLOBS = {'damage': 'fd16b00a4e46f17fd4528184269a81405f07f6b9',
         'sandbox': '55bdcd998e78b364233d5a55d8f13f059f458309'}
out = Path(sys.argv[2]) if len(sys.argv) > 2 else Path(__file__).parents[2] / 'film' / 'catalog'
for name, blob in BLOBS.items():
    raw = subprocess.check_output(['git', '-C', sys.argv[1], 'show',
                                   f'{PIN}:src/game/{name}_data.rs'])
    assert hashlib.sha1(f'blob {len(raw)}\0'.encode() + raw).hexdigest() == blob
    text = raw.decode().replace(f'use super::{name}::', 'use super::')
    text = text.replace('use crate::TagId;', 'use crate::theater::film::TagId;')
    text = text.replace('pub(crate) static ', '#[rustfmt::skip]\npub(crate) static ')
    path = out / ('physics' if name == 'sandbox' else name) / 'data.rs'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    subprocess.run(['rustfmt', '--edition', '2024', str(path)], check=True)
