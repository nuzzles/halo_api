#!/usr/bin/env python3
"""Resolve native component dispatch names, including named string constants."""
from pathlib import Path
import re
import json
import sys


def dispatch_names(grammar):
    grammar = Path(grammar)
    constants = {}
    for source in grammar.glob('*.go'):
        if not source.name.endswith('_test.go'):
            constants.update(re.findall(r'\b(\w+)\s*(?:string)?\s*=\s*"([^"\n]+)"', source.read_text()))
    names = set()
    unresolved = set()
    for source in grammar.glob('dispatch*.go'):
        for case in re.findall(r'case ([^:]+):', source.read_text()):
            for token in case.split(','):
                token = token.strip()
                if token.startswith('"'):
                    names.add(json.loads(token))
                elif token in constants:
                    names.add(constants[token])
                elif token.isidentifier():
                    unresolved.add(token)
    assert not unresolved, f'Unresolved native dispatch constants: {sorted(unresolved)}'
    assert names, 'No native dispatch names found'
    return sorted(names)


if __name__ == '__main__':
    Path(sys.argv[2]).write_text(json.dumps(dispatch_names(sys.argv[1]), indent=2) + '\n')
