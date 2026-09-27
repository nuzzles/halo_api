import base64
import hashlib
import json
from pathlib import Path
import struct
import tempfile
import unittest
import zlib

from bond_compact import DecodeError
from decode_navmesh import (MAX_BYTES, decode, obj_text, read_navmesh, read_spatial_cells,
                            unwrap, write_point_cloud)
from navmesh_preview import html_text


def varuint(value):
    data = bytearray()
    while value >= 128:
        data.append((value & 127) | 128)
        value >>= 7
    data.append(value)
    return bytes(data)


def wrap(data, *, compressed=None, declared=None):
    compressed = zlib.compress(data) if compressed is None else compressed
    size = len(data) if declared is None else declared
    fields = b'\x10\x02\x2b\x0e' + varuint(len(compressed)) + compressed + b'\x45' + varuint(size) + b'\x00'
    root = varuint(len(fields)) + fields
    return struct.pack('>III', 2, len(root), 0x1fffff) + root


class NavmeshTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.records = json.loads((Path(__file__).resolve().parents[1] /
                                  'fixtures/navmesh_records.json').read_text())['records']
        cls.tag = bytes.fromhex(cls.records[0]['hex'])
        cls.mesh = read_navmesh(cls.tag)
        # Captured mesh/tree plus original cell bounds with synthetic zero
        # sample counts. Other TAG0 envelopes are synthetic; full files are
        # also checked offline, without requiring downloaded assets in tests.
        fixture = json.loads((Path(__file__).resolve().parents[1] /
                              'fixtures/navmesh_tree_records.json').read_text())['records'][0]
        tree = zlib.decompress(base64.b64decode(fixture['tag_zlib_base64']))
        headers = zlib.decompress(base64.b64decode(fixture['cell_headers_zlib_base64']))
        cells = (struct.pack('<I', len(headers) // 28)
                 + b''.join(headers[i:i + 24] + bytes(4) for i in range(0, len(headers), 28)))
        empty_tag = struct.pack('>I', 8) + b'TAG0'
        cls.inner = (struct.pack('>7I', 2, 1, len(cls.tag), 8, 8, len(tree), len(cells))
                     + cls.tag + empty_tag * 2 + tree + cells + bytes(4))

    def test_captured_geometry_and_decompressed_byte_provenance(self):
        for record in self.records:
            with self.subTest(name=record['name']):
                tag = bytes.fromhex(record['hex'])
                self.assertEqual(hashlib.sha256(tag).hexdigest(), record['sha256'])
                data = bytes(record['offset']) + tag
                mesh = read_navmesh(data, record['offset'], record['end'])
                self.assertEqual(mesh['polygons'], record['expected_polygons'])
                self.assertEqual(mesh['vertex_extents'], record['expected_extents'])
                self.assertEqual(len(mesh['vertices']), record['expected_vertices'])
                self.assertEqual(mesh['validation']['shared_edge_pairs'], record['expected_shared_edge_pairs'])
                start, end = mesh['arrays']['vertices']['byte_range']
                self.assertEqual(list(struct.iter_unpack('<4f', data[start:end])), mesh['vertices'])

    def test_wrapper_uses_lengths_instead_of_a_fixed_offset(self):
        for data in [b'x', b'abc' * 600, self.inner]:
            blob = wrap(data)
            actual, envelope = unwrap(blob)
            self.assertEqual(actual, data)
            a, b = envelope['compressed_byte_range']
            self.assertEqual(zlib.decompress(blob[a:b]), data)
        report = decode(wrap(self.inner), 'synthetic.blob')
        self.assertEqual(len(report['navmesh']['polygons']), 5)
        self.assertIn('opaque', report['segments'][-1]['status'])
        self.assertIn('opaque', report['segments'][1]['status'])

    def test_zlib_checksum_eof_extra_stream_and_size_limits(self):
        compressed = zlib.compress(self.inner)
        bad_checksum = compressed[:-1] + bytes([compressed[-1] ^ 1])
        for blob in [wrap(self.inner, compressed=compressed[:-1]),
                     wrap(self.inner, compressed=bad_checksum),
                     wrap(self.inner, compressed=compressed + zlib.compress(b'extra')),
                     wrap(self.inner, declared=len(self.inner) - 1),
                     wrap(self.inner, declared=len(self.inner) + 1),
                     wrap(self.inner, declared=MAX_BYTES + 1)]:
            with self.subTest(size=len(blob)), self.assertRaises(DecodeError):
                unwrap(blob)

    def test_wrapper_and_segment_boundaries(self):
        blob = wrap(self.inner)
        for end in range(len(blob)):
            with self.assertRaises(DecodeError):
                unwrap(blob[:end])
        for offset in [0, 4, 8, 12, len(blob) - 1]:
            bad = bytearray(blob)
            bad[offset] ^= 1
            with self.assertRaises(DecodeError):
                unwrap(bad)
        for offset in [0, 4, 8, 24, len(self.inner) - 1]:
            bad = bytearray(self.inner)
            bad[offset] ^= 1
            with self.assertRaises(DecodeError):
                decode(wrap(bad), 'bad')

    def test_every_truncated_tag_is_rejected(self):
        for record in self.records:
            tag = bytes.fromhex(record['hex'])
            for end in range(len(tag)):
                with self.assertRaises(DecodeError):
                    read_navmesh(tag[:end])

    def test_schema_and_section_changes_fail_closed(self):
        for name in ['TST1', 'TNA1', 'FST1', 'TBDY', 'SDKV']:
            bad = bytearray(self.tag)
            bad[bad.index(name.encode()) + 4] ^= 1
            with self.subTest(name=name), self.assertRaises(DecodeError):
                read_navmesh(bad)
        bad = bytearray(self.tag)
        struct.pack_into('>I', bad, bad.index(b'DATA') - 4, 0x40000000 | len(bad))
        with self.assertRaises(DecodeError):
            read_navmesh(bad)

    def test_relocation_type_count_bounds_and_overlap(self):
        arrays = self.mesh['arrays']
        data_start = self.mesh['sections']['children'][1]['byte_range'][0] + 8
        modifications = [
            ('<Q', data_start + 24, 999999),
            ('<Q', data_start + 24, arrays['vertices']['item_id']),
            ('<I', arrays['faces']['item_bytes'][0] + 8, 0xffffffff),
            ('<I', arrays['faces']['item_bytes'][0] + 4, 0),
            ('<I', self.tag.index(b'PTCH') + 4, 9),
        ]
        for fmt, offset, value in modifications:
            bad = bytearray(self.tag)
            struct.pack_into(fmt, bad, offset, value)
            with self.subTest(offset=offset), self.assertRaises(DecodeError):
                read_navmesh(bad)

    def test_invalid_vertex_loop_ownership_and_opposites(self):
        first_edge = self.mesh['arrays']['edges']['byte_range'][0]
        first_face = self.mesh['arrays']['faces']['byte_range'][0]
        paired = next(i for i, e in enumerate(self.mesh['edges']) if e[2] != 0xffffffff)
        modifications = [
            ('<I', first_edge, 999999),
            ('<I', first_edge + 4, self.mesh['edges'][0][0]),
            ('<i', first_face + 12, 0),  # overlaps face 0
            ('<H', first_face + 10, 1),  # unsupported user edge
            ('<I', first_edge + paired * 20 + 8, paired),
            ('<I', first_edge + paired * 20 + 12, 999999),
        ]
        for fmt, offset, value in modifications:
            bad = bytearray(self.tag)
            struct.pack_into(fmt, bad, offset, value)
            with self.subTest(offset=offset), self.assertRaises(DecodeError):
                read_navmesh(bad)

    def test_nonfinite_and_out_of_aabb_vertices(self):
        start = self.mesh['arrays']['vertices']['byte_range'][0]
        for value in [float('nan'), float('inf'), 1e20]:
            bad = bytearray(self.tag)
            struct.pack_into('<f', bad, start, value)
            with self.subTest(value=value), self.assertRaises(DecodeError):
                read_navmesh(bad)

    def test_obj_preserves_polygons_and_preview_escapes_source(self):
        lines = obj_text(self.mesh).splitlines()
        actual = [[int(x) - 1 for x in line.split()[1:]] for line in lines if line.startswith('f ')]
        self.assertEqual(actual, self.mesh['polygons'])
        self.assertEqual(len([s for s in lines if s.startswith('v ')]), 29)
        report = decode(wrap(self.inner), '</script><script>bad()</script>')
        html = html_text(report)
        self.assertNotIn(report['source'], html)
        self.assertNotIn('/*NAVMESH_DATA*/', html)
        self.assertEqual(html.count('</script>'), 1)

    def test_captured_spatial_cells_and_exact_point_export(self):
        fixtures = json.loads((Path(__file__).resolve().parents[1] /
                               'fixtures/navmesh_cell_records.json').read_text())['records']
        for record in fixtures:
            raw = bytes.fromhex(record['hex'])
            self.assertEqual(hashlib.sha256(raw).hexdigest(), record['sha256'])
            # Keep captured cells unmodified, prepend a synthetic one-cell count.
            data = struct.pack('<I', 1) + raw
            result = read_spatial_cells(data, 0, len(data))
            self.assertEqual(result['sample_count'], record['sample_count'])
            self.assertEqual(result['cells'][0]['bounds_min'], tuple(record['bounds_min']))
            self.assertEqual(result['cells'][0]['bounds_max'], tuple(record['bounds_max']))
            self.assertEqual(sum(result['byte_accounting'].values()), len(data))
            with tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / 'points.ply'
                write_point_cloud(data, result, path)
                points = path.read_bytes().split(b'end_header\n', 1)[1]
                self.assertEqual(points, b''.join(raw[i:i + 12] for i in range(28, len(raw), 44)))

    def test_spatial_counts_truncation_bounds_and_opaque_fields(self):
        cell = struct.pack('<6fI', 2, 2, 2, 0, 0, 0, 1) + struct.pack('<3f', 1, 1, 1) + bytes(32)
        data = struct.pack('<I', 1) + cell
        for end in range(len(data)):
            with self.assertRaises(DecodeError):
                read_spatial_cells(data[:end], 0, end)
        modifications = [('<I', 0, 2), ('<I', 28, 2), ('<f', 4, -1),
                         ('<f', 32, float('nan')), ('<f', 32, 3)]
        for fmt, offset, value in modifications:
            bad = bytearray(data)
            struct.pack_into(fmt, bad, offset, value)
            with self.assertRaises(DecodeError):
                read_spatial_cells(bad, 0, len(bad))
        with self.assertRaises(DecodeError):
            read_spatial_cells(data + b'\0', 0, len(data) + 1)
        # Every opaque word may vary, including float NaN bit patterns.
        varied = data[:-32] + bytes([255]) * 32
        self.assertEqual(read_spatial_cells(varied, 0, len(varied))['sample_count'], 1)


if __name__ == '__main__':
    unittest.main()
