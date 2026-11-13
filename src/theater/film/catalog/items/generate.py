"""Reproduce items/data.rs from the pinned upstream data blob, then rustfmt.

python3 generate.py /path/to/libfilmshell
Only imports, derives and formatting change; catalogue rows stay
identical. This does not establish compatibility with a recorded game build.
"""
import hashlib
from pathlib import Path
import subprocess
import sys

PIN = 'f050371a6cda83b9887eeb83a1884336fce28e53'
BLOB = '5e8a63c4e01a4ff07f0b609b9ce7929c49148568'
raw = subprocess.check_output(['git', '-C', sys.argv[1], 'show', PIN + ':src/game/items_data.rs'])
assert hashlib.sha1(f'blob {len(raw)}\0'.encode() + raw).hexdigest() == BLOB
text = raw.decode().replace('use super::items::{', 'use super::{')
text = text.replace('#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]',
                    '#[derive(serde::Serialize, serde::Deserialize)]')
text = text.replace('#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]', '#[serde(rename_all = "snake_case")]')
text = text.replace('pub(crate) static ', '#[rustfmt::skip]\npub(crate) static ')
path = Path(__file__).with_name('data.rs')
path.write_text(text)
subprocess.run(['rustfmt', '--edition', '2024', str(path)], check=True)
