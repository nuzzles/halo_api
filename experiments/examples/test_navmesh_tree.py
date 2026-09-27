import base64
import copy
import hashlib
import json
from pathlib import Path
import struct
import unittest
import zlib

from bond_compact import DecodeError
from decode_navmesh import link_tree_cells, read_spatial_tree, unpack_tree_bounds


def fixture_cells(record):
    headers = zlib.decompress(base64.b64decode(record['cell_headers_zlib_base64']))
    if hashlib.sha256(headers).hexdigest() != record['cell_headers_sha256']:
        raise AssertionError('captured cell header hash mismatch')
    cells = []
    offset = record['cell_block_offset'] + 4
    for hi_x, hi_y, hi_z, lo_x, lo_y, lo_z, count in struct.iter_unpack('<6fI', headers):
        end = offset + 28 + 44 * count
        cells.append(dict(bounds_min=[lo_x, lo_y, lo_z], bounds_max=[hi_x, hi_y, hi_z],
                          byte_range=[offset, end], samples_byte_range=[offset + 28, end], sample_count=count))
        offset = end
    return dict(cell_count=len(cells), cells=cells)


class NavmeshTreeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.records = json.loads((Path(__file__).resolve().parents[1] /
                                  'fixtures/navmesh_tree_records.json').read_text())['records']
        cls.tag = zlib.decompress(base64.b64decode(cls.records[0]['tag_zlib_base64']))
        cls.tree = read_spatial_tree(cls.tag, 0, len(cls.tag))
        cls.cells = fixture_cells(cls.records[0])

    def test_both_complete_captured_trees_link_every_cell(self):
        for record in self.records:
            with self.subTest(source=record['source_url']):
                tag = zlib.decompress(base64.b64decode(record['tag_zlib_base64']))
                self.assertEqual(hashlib.sha256(tag).hexdigest(), record['tag_sha256'])
                data = bytes(record['offset']) + tag
                tree = read_spatial_tree(data, record['offset'], record['end'])
                self.assertEqual(tree['node_count'], record['expected_node_count'])
                self.assertEqual(tree['leaf_count'], record['expected_leaf_count'])
                self.assertEqual(tree['node_count'], tree['leaf_count'] * 2 - 1)
                self.assertEqual(tree['max_depth'], record['expected_depth'])
                self.assertEqual(list(tree['domain']['min']), record['expected_domain']['min'])
                self.assertEqual(tree['domain']['byte_range'], record['expected_domain']['byte_range'])
                links = link_tree_cells(tree, fixture_cells(record))
                self.assertEqual(links['cells_matched'], record['expected_leaf_count'])
                self.assertEqual(len(set(links['leaf_node_by_cell'])), record['expected_leaf_count'])
                for i, node in enumerate(tree['nodes']):
                    a, b = node['byte_range']
                    self.assertEqual(list(data[a:a + 3]), node['xyz_codes'])
                    self.assertEqual(b - a, 6)
                    if node['children']:
                        for child in node['children']:
                            self.assertEqual(tree['nodes'][child]['parent'], i)

    def test_tree_schema_relocations_counts_and_bounds_fail_closed(self):
        data_start = self.tree['sections']['children'][1]['byte_range'][0] + 8
        nodes_start = self.tree['node_array']['byte_range'][0]
        changes = [('<Q', data_start + 24, 9999), ('<q', data_start + 64, -1),
                   ('<I', data_start + 72, 0xffffffff), ('<f', data_start + 80, float('nan')),
                   ('<f', data_start + 96, -100), ('<H', nodes_start + 4, 0),
                   ('<H', nodes_start + 4, 0xffff), ('<B', nodes_start + 3, 0),
                   ('<B', nodes_start, 0xff)]
        for fmt, offset, value in changes:
            data = bytearray(self.tag)
            struct.pack_into(fmt, data, offset, value)
            with self.subTest(offset=offset, value=value), self.assertRaises(DecodeError):
                read_spatial_tree(data, 0, len(data))
        for tag in ['SDKV', 'TST1', 'TNA1', 'FST1', 'TBDY', 'PTCH']:
            data = bytearray(self.tag)
            data[data.index(tag.encode()) + 4] ^= 1
            with self.subTest(tag=tag), self.assertRaises(DecodeError):
                read_spatial_tree(data, 0, len(data))
        for end in range(len(self.tag)):
            with self.assertRaises(DecodeError):
                read_spatial_tree(self.tag[:end], 0, end)

    def test_duplicate_cell_and_subtree_overlap_rejected(self):
        leaves = [n for n in self.tree['nodes'] if n['cell_index'] is not None]
        data = bytearray(self.tag)
        # Copy just the first leaf's cell-index bytes into a different leaf.
        a, b = leaves[0]['byte_range'][0], leaves[1]['byte_range'][0]
        data[b + 3:b + 6] = data[a + 3:a + 6]
        with self.assertRaises(DecodeError):
            read_spatial_tree(data, 0, len(data))
        data = bytearray(self.tag)
        root = self.tree['node_array']['byte_range'][0]
        struct.pack_into('<H', data, root + 4, 1)  # Leaves no room for recorded left subtree.
        with self.assertRaises(DecodeError):
            read_spatial_tree(data, 0, len(data))

    def test_cell_reordering_or_changed_bounds_cannot_silently_relink(self):
        cells = copy.deepcopy(self.cells)
        cells['cells'][0], cells['cells'][-1] = cells['cells'][-1], cells['cells'][0]
        with self.assertRaises(DecodeError):
            link_tree_cells(self.tree, cells)
        cells = copy.deepcopy(self.cells)
        cells['cells'][0]['bounds_max'][0] += 10
        with self.assertRaises(DecodeError):
            link_tree_cells(self.tree, cells)
        cells['cell_count'] -= 1
        with self.assertRaises(DecodeError):
            link_tree_cells(self.tree, cells)

    def test_squared_nibbles_are_not_linear_bytes(self):
        lo, hi = unpack_tree_bounds([0x12, 0x34, 0], [0, 0, 0], [226, 226, 226])
        self.assertEqual(lo, [1, 9, 0])
        self.assertEqual(hi, [222, 210, 226])
        with self.assertRaises(DecodeError):
            unpack_tree_bounds([0xff] * 3, [0] * 3, [1] * 3)


if __name__ == '__main__':
    unittest.main()
