"""Original captured records and exact coverage accounting for the inspector."""

import json
from pathlib import Path
import unittest

from theater_inspector_data import (Inspector, annotate_delta, annotate_spawn,
                                    coverage, field, partition)

FIXTURES = json.loads((Path(__file__).resolve().parents[2] / 'src/theater/fixtures/oddball_records.json').read_text())


def bits(row):
    return ''.join(f'{b:08b}' for b in bytes.fromhex(row['hex']))[:row['length']]


class InspectorFields(unittest.TestCase):
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


if __name__ == '__main__':
    unittest.main()
