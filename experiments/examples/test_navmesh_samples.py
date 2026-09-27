import hashlib
import json
from pathlib import Path
import struct
import unittest

from bond_compact import DecodeError
from decode_navmesh import read_navmesh
from navmesh_samples import analyze_samples, decode_sample, edge_distance_squared_xy


class NavmeshSampleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        folder = Path(__file__).resolve().parents[1] / 'fixtures'
        cls.records = json.loads((folder / 'navmesh_sample_records.json').read_text())['records']
        cls.meshes = [read_navmesh(bytes.fromhex(r['hex'])) for r in
                      json.loads((folder / 'navmesh_records.json').read_text())['records']]

    def analyze(self, data, mesh):
        cells = dict(cells=[dict(samples_byte_range=[0, len(data)])])
        return analyze_samples(data, cells, mesh)

    def test_captured_face_references_and_numeric_or_unavailable_distances(self):
        for record in self.records:
            with self.subTest(offset=record['offset']):
                raw = bytes.fromhex(record['hex'])
                self.assertEqual(hashlib.sha256(raw).hexdigest(), record['sha256'])
                sample = decode_sample(bytes(record['offset']) + raw, record['offset'])
                self.assertEqual(sample['face_index'], record['face_index'])
                self.assertEqual(list(sample['position']), record['position'])
                self.assertEqual(sample['boundary_distance_squared'], record['boundary_distance_squared'])
                result = self.analyze(raw, self.meshes[record['mesh_fixture_index']])
                self.assertEqual(result['samples_inside_face_xy'], 1)
                self.assertEqual(result['boundary_distance_mismatches'], 0)
                self.assertEqual(result['boundary_distances_unavailable'],
                                 int(record['boundary_distance_squared'] is None))

    def test_boundary_can_belong_to_a_connected_neighbor(self):
        record = next(r for r in self.records if r['requires_connected_region'])
        mesh = self.meshes[record['mesh_fixture_index']]
        face = mesh['faces'][record['face_index']]
        own = min(edge_distance_squared_xy(record['position'], mesh['vertices'][e[0]], mesh['vertices'][e[1]])
                  for e in mesh['edges'][face[0]:face[0] + face[2]] if e[2] == 0xffffffff)
        self.assertGreater(abs(own - record['boundary_distance_squared']), .1)
        self.assertLess(self.analyze(bytes.fromhex(record['hex']), mesh)['max_boundary_distance_absolute_error'], 1e-5)

    def test_outliers_stay_recorded_instead_of_being_replaced(self):
        record = next(r for r in self.records if r['boundary_distance_squared'] is not None)
        raw = bytearray.fromhex(record['hex'])
        struct.pack_into('<f', raw, 40, record['boundary_distance_squared'] + 100)
        result = self.analyze(raw, self.meshes[record['mesh_fixture_index']])
        self.assertEqual(result['boundary_distance_mismatches'], 1)
        self.assertEqual(result['exceptions'][0]['recorded'], decode_sample(raw, 0)['boundary_distance_squared'])
        # Valid index but wrong polygon remains an explicit geometric mismatch.
        struct.pack_into('<I', raw, 16, (record['face_index'] + 1) % len(self.meshes[0]['faces']))
        self.assertEqual(self.analyze(raw, self.meshes[0])['samples_outside_face_xy'], 1)

    def test_unknown_sample_bytes_stay_uninterpreted(self):
        raw = bytearray.fromhex(self.records[0]['hex'])
        expected = decode_sample(raw, 0)
        raw[12:16] = b'\xff' * 4
        raw[20:40] = b'\xff' * 20
        self.assertEqual(decode_sample(raw, 0), expected)

    def test_truncation_bad_index_and_invalid_distance(self):
        raw = bytes.fromhex(self.records[0]['hex'])
        for end in range(44):
            with self.assertRaises(DecodeError):
                decode_sample(raw[:end], 0)
        for value in [float('nan'), float('inf'), -1]:
            bad = bytearray(raw)
            struct.pack_into('<f', bad, 40, value)
            with self.assertRaises(DecodeError):
                decode_sample(bad, 0)
        bad = bytearray(raw)
        struct.pack_into('<I', bad, 16, 0xffffffff)
        with self.assertRaises(DecodeError):
            self.analyze(bad, self.meshes[0])
        with self.assertRaises(DecodeError):
            analyze_samples(raw, {'cells': [{'samples_byte_range': [0, len(raw)]}]},
                            self.meshes[0], edge_budget=0)


if __name__ == '__main__':
    unittest.main()
