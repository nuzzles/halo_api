"""Trace only the prefix-present branch supported by unchanged libfilmshell."""
import json
from pathlib import Path
import export_reference

if __name__ == '__main__':
    cases = json.loads(Path(__file__).with_name('script.json').read_text())
    export_reference.main([c for c in cases if c['script_prefix']])
