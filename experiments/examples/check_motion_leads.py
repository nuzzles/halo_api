"""Offline speed calibration against Bazaar's recorded positions and 60 Hz clock.

Uses independent external BSP bounds with exact match/level provenance. No fitted
scale, simulated positions, gameplay-map assumptions, or numpy dependency.
"""
import json
import math
from pathlib import Path
import statistics

ROOT = Path(__file__).resolve().parents[1]


def main():
    bounds = json.loads((ROOT / 'reference/map-coordinate-bounds.json').read_text())['maps']['bazaar']
    folder = ROOT / 'films/maps/01-bazaar-idle'
    film = json.loads((folder / 'decoded-film.json').read_text())
    meta = json.loads((folder / 'film.json').read_text())
    provenance = json.loads((folder / 'settings/map-provenance.json').read_text())
    assert film['match_id'] == bounds['match_id'] == provenance['match_id']
    assert provenance['level_id'] == bounds['level_id']
    widths = [min(26, math.ceil(math.log2(math.ceil(60 * (hi - lo)))))
              for lo, hi in zip(bounds['min'], bounds['max'])]
    assert widths == bounds['axisWidths'] == [17, 17, 16]
    scale = [(hi - lo) / (1 << width) for lo, hi, width in zip(bounds['min'], bounds['max'], widths)]
    chunks = {}

    def tick(sample):
        source = sample['source']
        index = source['chunk']
        if index not in chunks:
            chunks[index] = (folder / next(c['file'] for c in meta['chunks'] if c['index'] == index)).read_bytes()
        start = source['payload_byte']
        return (int.from_bytes(chunks[index][start:start + 5], 'big') >> 3) & 255

    player = film['players'][0]
    velocities = {s['time_us']: s['value'] for s in player['velocities'] if s['value']['form'] == 'Directed'}
    positions = [s for s in player['positions'] if not s['value']['spawn']]
    errors, ratios = [], []
    for i in range(2, len(positions) - 2):
        s, a, b = positions[i], positions[i - 2], positions[i + 2]
        ticks = (tick(b) - tick(a)) % 256
        if (s['time_us'] not in velocities or not 1 <= ticks <= 10 or
                b['time_us'] - a['time_us'] > 200_000 or a['life'] != b['life']):
            continue
        v = velocities[s['time_us']]
        q = v['magnitude_code']
        predicted = .03 if q == 0 else 350 if q == 1023 else math.exp((q + .5) * math.log(350.97) / 1024) - .97
        observed = sum((bb - aa) * 60 / ticks * scale[axis] * v['direction'][axis]
                       for axis, (aa, bb) in enumerate(zip(a['value']['raw'], b['value']['raw'])))
        errors.append(abs(observed - predicted))
        ratios.append(observed / predicted)
    report = dict(comparisons=len(errors), median_observed_over_predicted=statistics.median(ratios),
                  median_absolute_error_world_units_s=statistics.median(errors),
                  p90_absolute_error_world_units_s=sorted(errors)[int(.9 * len(errors))],
                  provenance=bounds)
    assert len(errors) >= 300
    assert abs(report['median_observed_over_predicted'] - 1) < .02
    assert report['median_absolute_error_world_units_s'] < .08
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
