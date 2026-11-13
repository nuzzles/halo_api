"""Reuse primitive-only instrumentation, pinning the newly added original readers."""
import importlib.util
import json
from pathlib import Path
p=Path(__file__).parent.parent/'events-v41/export_reference.py'
spec=importlib.util.spec_from_file_location('event_export',p)
m=importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
m.PIN='f050371a6cda83b9887eeb83a1884336fce28e53'
m.main(json.loads(Path(__file__).with_name('cases.json').read_text()))
