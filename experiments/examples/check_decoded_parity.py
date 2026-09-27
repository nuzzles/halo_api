"""Check that typed Film exports retain historical observations; never rerun parsers.

Run after decode_theater_film on the corpus. This optional corpus audit is separate
from the fast captured-fixture Rust tests and requires the local research files.
"""
import argparse
from collections import Counter, defaultdict
import csv
import json
from pathlib import Path


def check(root):
    analysis = root / 'analysis'
    activity = {k: json.loads((analysis / k / 'events.json').read_text()) for k in ('firing', 'melee', 'grenades', 'weapons', 'zoom')}
    motion = {k: defaultdict(list) for k in ('positions', 'aim', 'inputs')}
    for kind, group in motion.items():
        with (analysis / 'all-films' / (kind + '.csv')).open() as file:
            for r in csv.DictReader(file):
                keys = {'positions': ('x_raw', 'y_raw', 'z_raw'), 'aim': ('yaw_raw', 'pitch_raw'), 'inputs': ('forward_raw', 'left_raw')}[kind]
                group[r['film']].append((int(r['time_us']), *(int(r[k]) for k in keys)))
    vitality = defaultdict(lambda: defaultdict(list))
    with (analysis / 'vitality/samples.csv').open() as file:
        for r in csv.DictReader(file):
            key = r['film'], int(r['player'])
            if r['body_raw']:
                vitality[key]['body'].append((round(float(r['time'])*1e6), int(r['serial']), int(r['body_raw']), int(r['body_state'], 2)))
            if r['shield_raw']:
                vitality[key]['shields'].append((round(float(r['time'])*1e6), int(r['serial']), int(r['shield_raw']), int(r['shield_delay_ticks']), int(r['shield_state'], 2)))
    scenes = {}
    for group in ('bandit', 'oddball'):
        for scene in json.loads((analysis / group / 'scenes.json').read_text()): scenes[scene['id']] = scene
    compared = additional = 0
    def same(actual, expected, context):
        nonlocal compared, additional
        a, b = Counter(tuple(v) for v in actual), Counter(tuple(v) for v in expected)
        if b - a:
            raise AssertionError(f'{context}: missing {list((b-a).items())[:3]}')
        # Historical probes only recognized a subset of packet grammars. New
        # guarded observations are counted separately; old observations must survive.
        additional += sum((a-b).values())
        compared += sum(b.values())
    for path in sorted(root.glob('*/*/decoded-film.json')):
        film = json.loads(path.read_text()); label = path.parent.relative_to(root).as_posix()
        metadata = json.loads(path.with_name('film.json').read_text())
        assert film['match_id'] == metadata['match_id']
        if metadata.get('analysis_profile') == 'decoded':
            print(f'SKIP historical comparison {label}: native decoder fixtures, no legacy exports', flush=True)
            continue
        if label not in scenes:
            for kind, values in [('positions', lambda v: v['raw']), ('aim', lambda v: (v['yaw'], v['pitch'])), ('inputs', lambda v: (v['forward'], v['left']))]:
                actual = [(s['time_us'], *values(s['value'])) for p in film['players'] for s in p[kind] if kind != 'positions' or not s['value']['spawn']]
                same(actual, motion[kind][label], (label, kind))
        else:
            old = {int(p['id']): p for p in scenes[label]['players']}
            for p in film['players']:
                same([(s['time_us'], *s['value']['raw'], s['life']) for s in p['positions']],
                     [(round(r[0]*1e6), *r[1:4], r[6]) for r in old[p['id']]['samples']], (label, p['id'], 'positions'))
                same([(s['time_us'], s['value']['yaw'], s['value']['pitch'], s['life']) for s in p['aim']],
                     [(round(r[0]*1e6), *r[1:]) for r in old[p['id']]['aim']], (label, p['id'], 'aim'))
                same([(l['id'], l['start_us'], l['end_us'], l['death_us']) for l in p['lives']],
                     [(l['id'], round(l['start']*1e6), round(l['end']*1e6), None if l['death'] is None else round(l['death']*1e6)) for l in old[p['id']]['lives']], (label, p['id'], 'lives'))
        for p in film['players']:
            for kind in ('firing', 'melee', 'grenades', 'zoom'):
                expected = activity[kind].get(label, {}).get('players', {}).get(str(p['id']), [])
                actual = [(s['time_us'], s['life'], *([s['value']['sequence']] if kind == 'firing' else [{'Unscoped': 0, 'First': 1, 'Second': 2}[s['value']]] if kind == 'zoom' else [])) for s in p[kind]]
                same(actual, [(round(r[0]*1e6), *r[1:]) for r in expected], (label, p['id'], kind))
            old = activity['weapons'].get(label, {}).get('players', {}).get(str(p['id']), {})
            for new, legacy, values in [('reloads', 'reload', lambda v: []), ('magazines', 'ammo', lambda v: [v['slot'], v['rounds']]), ('selections', 'switch', lambda v: [v]), ('weapons', 'weapon', lambda v: [v['slot'], None if v['weapon_window'] is None else f"{v['weapon_window']:010x}"])]:
                same([(s['time_us'], s['life'], *values(s['value'])) for s in p[new]], [(round(r[0]*1e6), *r[1:]) for r in (r[:4] if legacy == 'weapon' else r for r in old.get(legacy, []))], (label, p['id'], new))
            for kind, keys in [('body', ('raw', 'state')), ('shields', ('raw', 'delay_ticks', 'state'))]:
                same([(s['time_us'], s['life'], *(s['value'][k] for k in keys)) for s in p[kind]], vitality[label, p['id']][kind], (label, p['id'], kind))
        old_paths = activity['grenades'].get(label, {}).get('projectiles', [])
        assert len(old_paths) == len(film['projectiles'])
        for p, old in zip(film['projectiles'], old_paths):
            same([(s['time_us'], *s['value']) for s in p['positions']], [(round(r[0]*1e6), *r[1:]) for r in old['samples']], (label, 'projectile'))
        print(f'PASS {label}', flush=True)
    print(f'Compared {compared:,} observations/lifetimes against existing exports')
    print(f'{additional:,} additional native observations (not validated by historical exports)')

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--corpus', type=Path, default=Path(__file__).resolve().parents[1] / 'films')
    check(parser.parse_args().corpus)
