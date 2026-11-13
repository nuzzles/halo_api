"""Summarize HALO_TIMER_AUDIT exports without assigning timer roles or units.

Usage: python3 timer_audit.py EXPORT_DIRECTORY CORPUS_ROOT
The Rust captured-scoreboard test verifies source fields before accepting its
run. This report separates same-entity, positive-time changes across gaps/reset
from intact pairs. Even an intact pair does not establish an engine time unit.
"""
from collections import Counter
import hashlib
import json
from pathlib import Path
import sys


FILMS = ['octagon/03-first-to-50', 'ranked-arena/02-oddball']


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def summarize(export_dir, corpus_root):
    report = {'libfilmshell': '8f0f8d078e513c800baa21fac416fd4c0c127d03',
              'levelup': '413971bcc485be53a2fa8cf229e170539a5146f8', 'films': []}
    for name in FILMS:
        path = export_dir / (name.replace('/', '-') + '.json')
        exported = json.loads(path.read_text())
        rows = exported['updates']
        root = corpus_root / name
        manifest_path = root / 'film.json'
        manifest = json.loads(manifest_path.read_text())
        film = {'name': name, 'export_sha256': digest(path),
                'manifest_sha256': digest(manifest_path),
                'chunks': [{'file': c['file'], 'sha256': digest(root / c['file'])}
                           for c in manifest['chunks']],
                'opaque_packets': exported['opaque_packets'], 'fields': {}}
        for field in ['RoundTimer', 'RespawnTimer', 'Phase', 'Round']:
            samples = [(u, v) for u in rows for v in u['values'] if v['field'] == field]
            result = {'observations': len(samples)}
            if field in ['Phase', 'Round']:
                result['values'] = dict(sorted(Counter(str(v['value']) for _, v in samples).items()))
                film['fields'][field] = result
                continue
            key = 'seconds' if field == 'RoundTimer' else 'words'
            last, edges, examples = {}, Counter(), {}
            by_state = {}
            for u, v in samples:
                val = v['value']
                state = str(val.get('state', val.get('active')))
                group = by_state.setdefault(state, {'observations': 0, 'min': val[key].copy(), 'max': val[key].copy()})
                group['observations'] += 1
                group['min'] = [min(a,b) for a,b in zip(group['min'], val[key])]
                group['max'] = [max(a,b) for a,b in zip(group['max'], val[key])]
                prev = last.get(u['entity'])
                last[u['entity']] = (u,v)
                if prev is None:
                    continue
                p,w = prev
                dt = u['time_us'] - p['time_us']
                if dt <= 0:
                    continue
                delta = [a-b for a,b in zip(val[key], w['value'][key])]
                if not any(delta):
                    continue
                support = 'intact' if u['segment']==p['segment'] and not u['reset'] else 'gap_or_reset'
                edges[support] += 1
                direction = 'increasing' if delta[1] > 0 else 'decreasing' if delta[1] < 0 else 'unchanged'
                label = support + '/second_word_' + direction
                edges[label] += 1
                if label not in examples:
                    def sample(event, value):
                        return {k:event[k] for k in ['time_us','event','entity','reset','segment']} | value
                    examples[label] = {'delta_time_us': dt, 'delta_values': delta,
                                       'previous': sample(p,w), 'current': sample(u,v)}
            result.update(states=by_state, changed_positive_time_pairs=dict(sorted(edges.items())), examples=examples)
            film['fields'][field] = result
        report['films'].append(film)
    return report


if __name__ == '__main__':
    print(json.dumps(summarize(Path(sys.argv[1]), Path(sys.argv[2])), indent=2, sort_keys=True))
