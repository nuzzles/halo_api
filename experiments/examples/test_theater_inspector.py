"""Original captured records and exact coverage accounting for the inspector."""

import csv
import copy
import json
from pathlib import Path
import random
import struct
import tempfile
import unittest
from unittest.mock import patch

from theater_inspector_data import (Inspector, annotate_delta, annotate_spawn, annotate_velocity,
                                    CLOCK, STATUSES, coverage, coverage_counts, field, partition)

FIXTURES = json.loads((Path(__file__).resolve().parents[2] / 'src/theater/fixtures/oddball_records.json').read_text())


def bits(row):
    return ''.join(f'{b:08b}' for b in bytes.fromhex(row['hex']))[:row['length']]


class InspectorFields(unittest.TestCase):
    def summary_fixture(self, root):
        label = 'synthetic/summary'
        folder = root / label
        folder.mkdir(parents=True)
        data = (Path(__file__).resolve().parents[2] / 'src/theater/fixtures/summary-v41.bin').read_bytes()
        (folder / 'footer.bin').write_bytes(data)
        (folder / 'film.json').write_text(json.dumps(dict(
            match_id='fixture', film_major_version=41, film_length=80000,
            chunks=[dict(index=6, file='footer.bin', chunk_type=3, start_time_offset_ms=0, duration_ms=80000)])))
        events = []
        # Positions and values from the captured AR-kill footer, not synthetic bytes.
        for xuid, name, time, kind, code, metadata, tail, identity in [
            ('2535472547643888', 'Nuzzles', 25344000, 'Kill', 50, 0, 27404, 12478),
            ('2535443507298499', 'Yet', 25344000, 'Death', 20, 1, 43882, 28956),
            ('2535472547643888', 'Nuzzles', 77279000, 'Medal', 150, 32, 71766, 56840),
        ]:
            events.append(dict(xuid=xuid, name=name, time_us=time, kind=kind,
                type_code=code, metadata=metadata, medal_flag=int(kind=='Medal'),
                medal=dict(film_id=32, name='Steaktacular', name_id=1169390319) if kind=='Medal' else None,
                source=dict(chunk=6, payload_byte=16, bit=tail, end_bit=tail+480),
                identity_source=dict(chunk=6, payload_byte=16, bit=identity, end_bit=identity+64)))
        result = dict(match_id='fixture', major_version=41, summary=dict(events=events,
            packets=[dict(chunk=6, payload_byte=16, declared_events=3, decoded_events=3)]))
        lab = Inspector(root)
        lab.catalog[label] = dict(match_id='fixture')
        return lab, label, result

    def test_native_summary_packets_medals_exact_coverage_and_refresh(self):
        with tempfile.TemporaryDirectory() as directory:
            lab, label, result = self.summary_fixture(Path(directory))
            with patch('theater_inspector_data.decode_summary', return_value=result) as native:
                desc = lab.describe(label)
                self.assertEqual(desc['chunks'][0]['packets'], 2)
                info = lab.chunk_info(label, 6)
                self.assertEqual([p['kind'] for p in info['packets']], [9, 7])
                self.assertEqual(len(info['events']), 3)
                self.assertIn('Steaktacular', info['events'][-1]['title'])
                counts = lab.chunk_coverage(label, 6)
                self.assertEqual(counts['issues'], [])
                # Text padding is structure; the catalog NameId contributes zero bits.
                self.assertEqual(counts['coverage'], dict(decoded=872, structure=784, opaque=44946, unparsed=25934))
                self.assertEqual(sum(counts['coverage'].values()), 9067*8)
                view = lab.view(label, 6, info['events'][-1]['offset'])
                medal = next(f for f in view['spans'] if f['label']=='Medal')
                self.assertEqual((medal['start'], medal['end']), (16*8+71766+59*8, 16*8+71766+60*8))
                self.assertEqual(medal['end']-medal['start'], 8)
                self.assertIn('1169390319', medal['note'])
                self.assertEqual(view['fields_scope'], 'visible summary bytes')
                self.assertLess(len(view['spans']), 20)
                self.assertEqual(native.call_count, 1)
                lab.describe(label)
                lab.chunk_coverage(label, 6)
                self.assertEqual(native.call_count, 1)
                # Same running Inspector notices a replaced export and invalidates totals.
                lab.path(label+'/decoded-film.velocity.csv').write_text('refreshed evidence')
                self.assertNotEqual(lab.describe(label)['revision'], desc['revision'])
                lab.chunk_coverage(label, 6)
                self.assertEqual(native.call_count, 2)

    def test_summary_stale_values_withhold_fields_and_decoder_failure_is_visible(self):
        with tempfile.TemporaryDirectory() as directory:
            lab, label, result = self.summary_fixture(Path(directory))
            bad = copy.deepcopy(result)
            bad['summary']['events'][-1]['metadata'] = 255
            with patch('theater_inspector_data.decode_summary', return_value=bad):
                counts = lab.chunk_coverage(label, 6)
                self.assertEqual(counts['issue_count'], 1)
                self.assertIn('byte mismatch', counts['issues'][0]['message'])
                self.assertLess(counts['coverage']['decoded'], 872)
            lab.refresh(label, force=True)
            with patch('theater_inspector_data.decode_summary', side_effect=ValueError('decoder unavailable')):
                counts = lab.chunk_coverage(label, 6)
                self.assertIn('decoder unavailable', counts['issues'][0]['message'])
                self.assertEqual(counts['coverage']['decoded'], 256)  # Headers + declared count only.

    def test_velocity_fields_show_world_speed_and_reject_bad_evidence(self):
        fixture_path = Path(__file__).resolve().parents[2] / 'src/theater/fixtures/velocity_records.json'
        for row in json.loads(fixture_path.read_text())['records']:
            raw = ''.join(f'{b:08b}' for b in bytes.fromhex(row['hex']))
            offset, end = row['velocity_bit'], row['velocity_end']
            fields = annotate_velocity(raw, offset, 0, 'Pawn', 'fixture')
            counts = coverage(partition(offset, end, fields))
            if row['direction_code'] is None:
                self.assertEqual(counts, dict(decoded=2, structure=0, opaque=0, unparsed=0))
            else:
                self.assertEqual(counts, dict(decoded=29, structure=2, opaque=0, unparsed=0))
                self.assertEqual(fields[1]['value']['code'], row['direction_code'])
                self.assertEqual(fields[2]['value'], row['magnitude_code'])
                self.assertIn('world units/s', fields[2]['note'])
                corrupt = raw[:offset + 2] + '1' * 19 + raw[offset + 21:]
                withheld = annotate_velocity(corrupt, offset, 0, 'Pawn', 'fixture')
                self.assertEqual(coverage(partition(offset, end, withheld))['opaque'], 29)
            evidence = dict(player=0, life=row['life'], bit=offset, end_bit=end,
                            direction_code='' if row['direction_code'] is None else str(row['direction_code']),
                            magnitude_code='' if row['magnitude_code'] is None else str(row['magnitude_code']))
            self.assertTrue(Inspector().record_fields('', 'velocity', evidence, raw, 0))
            with self.assertRaises(ValueError):
                Inspector().record_fields('', 'velocity', evidence, raw[:offset], 0)

    def test_native_projectile_velocity_annotations_use_fixed_precision_and_reject_corruption(self):
        fixture = json.loads((Path(__file__).resolve().parents[2] / 'src/theater/fixtures/projectile_motion_records.json').read_text())
        inspector = Inspector()
        for r in fixture['records']:
            if r['kind'] != 'delta':
                continue
            raw = ''.join(f'{b:08b}' for b in bytes.fromhex(r['hex']))
            _, start, end = next(f for f in r['fields'] if f[0] == 1)
            stationary = r['vel'] == 'stationary'
            row = dict(track='0', player='0', life='0', bit=str(start), end_bit=str(end), kind='velocity',
                       direction_code='' if stationary else str(r['vel'][0]),
                       magnitude_code='' if stationary else str(r['vel'][1]))
            fields = inspector.record_fields('', 'native_projectile', row, raw, 0)
            counts = coverage(partition(start, end, fields))
            self.assertEqual(counts['decoded'], 1 if stationary else 29)
            self.assertEqual(counts['structure'], 0 if stationary else 1)
            corrupt = raw[:start] + ('1' if raw[start] == '0' else '0') + raw[start+1:]
            with self.assertRaises(ValueError):
                inspector.record_fields('', 'native_projectile', row, corrupt, 0)

    def test_captured_delta_fields_point_to_original_bits(self):
        for row in FIXTURES['deltas']:
            with self.subTest(source=row['source']):
                raw, base = bits(row), 800
                annotations, parsed = annotate_delta(raw, 0, base, int(row['frame_prefix'][29:], 2),
                                                     'fixture', 'Player', row['serial'], row['generation'])
                self.assertEqual(parsed['end'], row['length'])
                for axis, expected in zip('XYZ', row['position']):
                    f = next(f for f in annotations if f['label'] == axis + ' raw')
                    self.assertEqual(f['value'], expected)
                    self.assertEqual(int(raw[f['start'] - base:f['end'] - base], 2), expected)
                for label, expected in zip(('Yaw raw', 'Pitch raw'), row['aim']):
                    f = next(f for f in annotations if f['label'] == label)
                    self.assertEqual(int(raw[f['start'] - base:f['end'] - base], 2), expected)
                spans = partition(base, base + len(raw) + 13, annotations)
                self.assertEqual(sum(coverage(spans).values()), len(raw) + 13)
                self.assertEqual(spans[-1]['status'], 'unparsed')
                self.assertEqual(spans[-1]['end'] - spans[-1]['start'], 13)

    def test_spawn_generation_and_shift_use_exact_capture_offsets(self):
        for sample in FIXTURES['spawns']:
            row = dict(sample['expected'], id=sample['serial'], bit=0, name='Player')
            annotations = annotate_spawn(bits(sample), row, 128, oddball=True)
            x = next(f for f in annotations if f['label'] == 'X raw')
            self.assertEqual(x['start'], 128 + 263 + row['shift'])
            generation = next(f for f in annotations if f['label'] == 'Generation tag')
            self.assertEqual(generation['value'], row['generation'])
            self.assertEqual(generation['end'] - generation['start'], 2)

    def test_mixed_bytes_and_opaque_overlays_do_not_inflate_coverage(self):
        annotations = [field(3, 13, 'Opaque', None, 'opaque'), field(5, 10, 'Known', 17)]
        spans = partition(0, 16, annotations)
        self.assertEqual(coverage(spans), dict(decoded=5, structure=0, opaque=5, unparsed=6))
        self.assertEqual([(s['start'], s['end']) for s in spans], [(0, 3), (3, 5), (5, 10), (10, 13), (13, 16)])
        for bit in range(16):
            self.assertEqual(sum(s['start'] <= bit < s['end'] for s in spans), 1)
        self.assertEqual(sum(coverage(partition(8, 16, annotations)).values()), 8)

    def test_no_fields_means_all_bits_are_unparsed_even_if_zero(self):
        spans = partition(0, 4096, [])
        self.assertEqual(coverage(spans), dict(decoded=0, structure=0, opaque=0, unparsed=4096))

    def test_corrupt_export_and_capture_do_not_emit_annotations(self):
        row = FIXTURES['spawns'][0]
        with self.assertRaises(ValueError):
            annotate_spawn(bits(row), dict(row['expected'], id=0, bit=0, name='Player', wire=99), 0, True)
        sample = FIXTURES['deltas'][0]
        with self.assertRaises(ValueError):
            annotate_delta(bits(sample), 0, 0, (int(sample['frame_prefix'][29:], 2) + 1) % 256,
                           'fixture', 'Player', sample['serial'], sample['generation'])

    def test_registry_names_are_decoded_but_padding_is_not(self):
        lab = Inspector()
        raw = bytearray(16640)
        raw[8:12] = b'aim\0'
        spans = partition(0, len(raw) * 8, lab.registry(bytes(raw)))
        self.assertEqual(coverage(spans), dict(decoded=24, structure=8, opaque=0, unparsed=16640 * 8 - 32))
        with self.assertRaises(ValueError):
            lab.path('../outside')

    def test_recording_counts_match_independent_bit_priority_oracle(self):
        rng = random.Random(41)
        for _ in range(60):
            fields = []
            for i in range(rng.randrange(25)):
                a, b = sorted(rng.sample(range(96), 2))
                fields.append(field(a, b, str(i), None, rng.choice(STATUSES)))
            expected = dict.fromkeys(STATUSES, 0)
            for bit in range(7, 83):
                candidates = [f['status'] for f in fields if f['start'] <= bit < f['end']]
                winner = min(candidates, key=STATUSES.index) if candidates else 'unparsed'
                expected[winner] += 1
            self.assertEqual(coverage_counts(7, 83, fields), expected)
            self.assertEqual(coverage_counts(7, 83, fields), coverage(partition(7, 83, fields)))
        self.assertEqual(coverage_counts(0, 0, []), dict.fromkeys(STATUSES, 0))

    def test_whole_chunks_include_headers_padding_unknown_types_and_withheld_fields(self):
        with tempfile.TemporaryDirectory() as directory:
            root, label = Path(directory), 'synthetic/coverage'
            folder = root / label
            folder.mkdir(parents=True)
            registry = bytearray(16640)
            registry[8:12] = b'aim\0'
            clock = int(CLOCK + '00101010' + '000', 2).to_bytes(5, 'big')
            replication = (struct.pack('<HHIQ', 0, 7, len(clock), 1000000) + clock
                           + struct.pack('<HHIQ', 8, 2, 3, 1000100) + b'\0\xff\0')
            chunks = []
            for index, (kind, data) in enumerate([(1, registry), (2, replication), (99, b'\0\xff\0'), (99, b'')]):
                name = f'chunk-{index}.bin'
                (folder / name).write_bytes(data)
                chunks.append(dict(index=index, file=name, chunk_type=kind))
            (folder / 'film.json').write_text(json.dumps(dict(match_id='fixture', chunks=chunks)))
            lab = Inspector(root)
            lab.catalog[label] = dict(match_id='fixture')
            with patch.object(lab, 'evidence', return_value={}):
                registry_counts = lab.chunk_coverage(label, 0)
                self.assertEqual(registry_counts['coverage'], dict(decoded=24, structure=8, opaque=0, unparsed=133088))
                result = lab.chunk_coverage(label, 1)
                self.assertEqual(result['coverage'], dict(decoded=232, structure=29, opaque=32, unparsed=27))
                self.assertEqual(result['total_bits'], len(replication) * 8)
                views = [lab.view(label, 1, offset) for offset in (0, 21)]
                self.assertEqual(result['coverage'], {s: sum(v['coverage'][s] for v in views) for s in STATUSES})
                self.assertEqual(lab.chunk_coverage(label, 2)['coverage'], dict(decoded=0, structure=0, opaque=0, unparsed=24))
                self.assertEqual(lab.chunk_coverage(label, 3)['total_bits'], 0)
                # The summary cache avoids rescanning when returning to a recording.
                with patch.object(lab, 'annotations', side_effect=AssertionError('Unexpected rescan')):
                    self.assertEqual(lab.chunk_coverage(label, 1), result)
            lab.chunk_coverage.cache_clear()
            with patch.object(lab, 'evidence', return_value={16: {'positions': [{}]}}), patch.object(lab, 'record_fields', side_effect=ValueError('Export / byte mismatch')):
                rejected = lab.chunk_coverage(label, 1)
                self.assertEqual(rejected['coverage'], result['coverage'])
                self.assertEqual(rejected['issue_count'], 1)
                self.assertIn('annotation withheld', rejected['issues'][0]['message'])

    def test_csv_offset_index_preserves_quoted_records_and_film_chunk_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'analysis/all-films/positions.csv'
            path.parent.mkdir(parents=True)
            rows = [dict(film=film, chunk=chunk, payload_byte=payload, note=note)
                    for film, chunk, payload, note in [('a', 1, 20, 'name, comma'),
                                                     ('b', 1, 30, 'other film'),
                                                     ('a', 2, 40, 'line\nbreak'),
                                                     ('a', 1, 50, 'café')]]
            with path.open('w', newline='') as file:
                writer = csv.DictWriter(file, fieldnames=list(rows[0]))
                writer.writeheader()
                writer.writerows(rows)
            lab = Inspector(directory)
            for label, chunk in [('a', 1), ('a', 2), ('b', 1), ('absent', 1)]:
                actual = [row for packet in lab.evidence(label, chunk).values() for row in packet['positions']]
                expected = [{k: str(v) for k, v in row.items()} for row in rows if row['film'] == label and row['chunk'] == chunk]
                self.assertEqual(actual, expected)


if __name__ == '__main__':
    unittest.main()
