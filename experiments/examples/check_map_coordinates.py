"""Fit Recharge's map/film coordinate correspondence, then check another film.

Research only: an axis-aligned least-squares fit is not a decoded BSP bounds
table. Nothing here supplies replay state. The training film is Oddball; the
Bandit film never participates in fitting or object selection for training.
"""
import argparse
import json
import math
from pathlib import Path
import statistics

from decode_map_variant import decode

ROOT = Path(__file__).resolve().parents[1]
TRAIN = 'ranked-arena/02-oddball'
VALIDATE = 'bandit/01-evo'
TYPE_VALUE = -1533673853  # Numeric Bond value; respawn-like by correspondence.


def transform(raw, scale, bias):
    return [q * s + b for q, s, b in zip(raw, scale, bias)]


def nearest(raw, objects, scale, bias):
    world = transform(raw, scale, bias)
    obj = min(objects, key=lambda o: math.dist(world, o['position']['components']))
    return obj, math.dist(world, obj['position']['components'])


def fit(raw_points, objects):
    """Initialize from extents, alternate nearest objects and per-axis regression.

    Trim inconsistent pairs progressively. This can find a local minimum; the
    independent-film check is essential. These cutoffs are research parameters.
    """
    world = [o['position']['components'] for o in objects]
    scale = [(max(q[a] for q in world) - min(q[a] for q in world)) /
             (max(p[a] for p in raw_points) - min(p[a] for p in raw_points)) for a in range(3)]
    bias = [min(q[a] for q in world) - scale[a] * min(p[a] for p in raw_points) for a in range(3)]
    for cutoff in [math.inf, .4, .1, .025]:
        for _ in range(20):
            pairs = [(p, o['position']['components']) for p in raw_points
                     for o, error in [nearest(p, objects, scale, bias)] if error < cutoff]
            if len(pairs) < 4:
                raise ValueError('too few map correspondences to fit')
            old = scale + bias
            for a in range(3):
                raw_mean = statistics.mean(p[a] for p, _ in pairs)
                world_mean = statistics.mean(q[a] for _, q in pairs)
                denominator = sum((p[a] - raw_mean) ** 2 for p, _ in pairs)
                if denominator == 0:
                    raise ValueError('map anchors do not constrain every axis')
                scale[a] = sum((p[a] - raw_mean) * (q[a] - world_mean) for p, q in pairs) / denominator
                bias[a] = world_mean - scale[a] * raw_mean
            if max(abs(a - b) for a, b in zip(old, scale + bias)) < 1e-10:
                break
    if any(s <= 0 for s in scale):
        raise ValueError('fit contradicts the checked axis order')
    return scale, bias


def load_film(folder):
    film = json.loads((folder / 'decoded-film.json').read_text())
    history = json.loads((folder / 'settings/match-history-entry.json').read_text())
    metadata = json.loads((folder / 'settings/map-variant.json').read_text())
    reference = history['MatchInfo']['MapVariant']
    if (history['MatchId'] != film['match_id'] or
            any(reference[k] != metadata[k] for k in ('AssetId', 'VersionId'))):
        raise ValueError('film/map provenance mismatch')
    spawns = {}
    for player in film['players']:
        for sample in player['positions']:
            if sample['value']['spawn']:
                spawns.setdefault(tuple(sample['value']['raw']), dict(
                    player=player['name'], time_us=sample['time_us'], source=sample['source']))
    return film['match_id'], metadata, spawns


def evaluate(spawns, objects, scale, bias):
    anchors, unmatched = [], []
    for raw, sample in sorted(spawns.items()):
        obj, error = nearest(raw, objects, scale, bias)
        row = dict(raw=raw, film_sample=sample, world_fitted=transform(raw, scale, bias),
                   object_index=obj['index'], object_bytes=obj['byte_range'],
                   world_object=obj['position']['components'], distance=error)
        (anchors if error < .025 else unmatched).append(row)
    errors = [r['distance'] for r in anchors]
    return dict(unique_spawns=len(spawns), matched=len(anchors), unmatched=len(unmatched),
                median_distance=statistics.median(errors) if errors else None,
                max_distance=max(errors) if errors else None,
                anchors=anchors, unmatched_candidates=unmatched)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--films', type=Path, default=ROOT / 'films')
    parser.add_argument('--output', type=Path, help='save full source-offset evidence')
    args = parser.parse_args()
    train_id, train_meta, train_spawns = load_film(args.films / TRAIN)
    validate_id, validate_meta, validate_spawns = load_film(args.films / VALIDATE)
    if train_id == validate_id or any(train_meta[k] != validate_meta[k] for k in ('AssetId', 'VersionId')):
        raise ValueError('validation requires a different film of the exact same map revision')
    asset = args.films / TRAIN / 'settings/map.mvar'
    if asset.name not in train_meta['Files']['FileRelativePaths']:
        raise ValueError('asset absent from referenced revision')
    decoded = decode(asset.read_bytes(), asset.name)
    objects = [o for o in decoded['objects'] if o['type_value'] == TYPE_VALUE and
               all(v is not None for v in o['position']['components'])]
    scale, bias = fit(sorted(train_spawns), objects)
    training = evaluate(train_spawns, objects, scale, bias)
    validation = evaluate(validate_spawns, objects, scale, bias)
    accepted_training = {tuple(row['raw']) for row in training['anchors']}
    novel = [row for row in validation['anchors'] if tuple(row['raw']) not in accepted_training]
    validation['matched_raw_points_absent_from_training'] = len(novel)
    validation['max_distance_on_novel_raw_points'] = max((row['distance'] for row in novel), default=None)
    report = dict(
        scope='Empirical map-coordinate calibration, not decoded BSP bounds or replay observations. '
              'Unmatched spawns are retained. No omitted map coordinates were filled in.',
        map_asset_id=train_meta['AssetId'], map_version_id=train_meta['VersionId'],
        asset_url=train_meta['Files']['Prefix'] + asset.name, asset_sha256=decoded['sha256'],
        object_type_value=TYPE_VALUE, equation='world[axis] = raw[axis] * scale[axis] + bias[axis]',
        scale=scale, bias=bias, acceptance_distance=.025,
        train_match_id=train_id, validate_match_id=validate_id, training=training, validation=validation)
    # Corpus checks protect the claimed finding, rather than treating a fit as proof.
    if not (training['matched'] >= 55 and validation['matched'] >= 42 and
            validation['max_distance'] < .015):
        raise ValueError('map correspondence no longer reproduces the documented evidence')
    if args.output:
        args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: ({kk: vv for kk, vv in v.items() if kk not in ('anchors', 'unmatched_candidates')}
                         if k in ('training', 'validation') else v)
                      for k, v in report.items()}, indent=2))


if __name__ == '__main__':
    main()
