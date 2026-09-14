import json
import hashlib
from pathlib import Path
import struct
import unittest

from bond_compact import DecodeError, field, parse
from decode_map_variant import decode, object_fields, shape_parameters


class MapVariantTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.records = json.loads((Path(__file__).resolve().parents[1] /
                                  'fixtures/map_variant_records.json').read_text())['records']

    def test_captured_objects_and_absolute_scalar_offsets(self):
        for record in self.records:
            with self.subTest(source=record['source_url']):
                data = bytes.fromhex(record['hex'])
                node = parse(data, origin=record['offset'])['root']
                self.assertEqual(node['end'], record['end'])
                obj = object_fields(node, record['index'])
                self.assertEqual(obj['type_value'], record['type_value'])
                self.assertEqual(obj['position']['components'], record['position'])
                for value, span in zip(obj['position']['components'], obj['position']['component_bytes']):
                    if span is not None:
                        start, end = (v - record['offset'] for v in span)
                        self.assertEqual(struct.unpack('<f', data[start:end])[0], value)

    def test_omitted_axis_remains_unknown(self):
        node = parse(bytes.fromhex(self.records[0]['hex']))['root']
        obj = object_fields(node, 43)
        self.assertIsNone(obj['position']['components'][1])
        self.assertIsNone(obj['position']['component_bytes'][1])

    def test_complete_neutral_two_object_map(self):
        fixture = json.loads((Path(__file__).resolve().parents[1] /
                              'fixtures/minimal_map_variant.json').read_text())
        data = bytes.fromhex(fixture['hex'])
        self.assertEqual(hashlib.sha256(data).hexdigest(), fixture['sha256'])
        result = decode(data, 'map.mvar', include_tree=True)
        self.assertEqual(result['structure_bytes'], fixture['bytes'])
        self.assertEqual(len(result['objects']), 2)
        for actual, expected in zip(result['objects'], fixture['expected_objects']):
            self.assertEqual(actual['type_value'], expected['type_value'])
            self.assertEqual(actual['position']['components'], expected['position'])
        wall = field(result['tree'], 3, 11)['value']['items'][0]
        scale_property = field(field(wall, 8, 10), 23, 11)['value']['items'][0]
        vector_node = field(scale_property, 0, 10)
        self.assertEqual([field(vector_node, axis, 7)['value'] for axis in range(3)],
                         fixture['expected_objects'][0]['candidate_scale'])

    def test_captured_volume_parameters_and_missing_dimension(self):
        fixture = json.loads((Path(__file__).resolve().parents[1] /
                              'fixtures/map_shape_records.json').read_text())
        for record in fixture['records']:
            with self.subTest(source=record['source_url'], index=record['index']):
                node = parse(bytes.fromhex(record['hex']), origin=record['offset'])['root']
                shapes = shape_parameters(node)
                self.assertEqual(len(shapes), 1)
                shape = shapes[0]
                self.assertEqual(shape['family_code'], record['expected_family'])
                for parameter, raw, span in zip(shape['parameters'], record['expected_parameters'],
                                                record['parameter_byte_ranges']):
                    self.assertEqual(parameter['raw'], raw)
                    self.assertEqual(parameter['fixed16_16'], raw / 65536 if raw is not None else None)
                    self.assertEqual(parameter['value_bytes'], span)
                # Unsupported family retains its numbers without a shape label.
                bag = field(field(node, 8, 10), 0, 11)['value']['items'][0]
                volume = field(bag, 0, 11)['value']['items'][0]
                field(volume, 0, 16)['value'] = 99
                self.assertIsNone(shape_parameters(node)[0]['family_candidate'])
                # An unexpected wrapper type must not be read as a dimension.
                field(volume, 5)['type'] = 7
                self.assertIsNone(shape_parameters(node)[0]['parameters'][0]['raw'])

    def test_every_truncated_capture_is_rejected(self):
        for record in self.records:
            data = bytes.fromhex(record['hex'])
            for end in range(len(data)):
                with self.subTest(index=record['index'], end=end), self.assertRaises(DecodeError):
                    parse(data[:end])

    def test_nested_lengths_stop_and_overflow(self):
        for data in [
            '00',                      # zero-length struct, no STOP
            '020000',                  # STOP before declared boundary
            '03030101',                # STOP_BASE cannot terminate a struct
            '060a0207000000',          # float cannot consume the parent's bytes
            '0604ffffffff00',          # uint16 overflow
            '0610808080808000',        # unterminated uint32
            '030b2000',                # list cannot contain STOP values
            '050d03000100',            # map cannot contain STOP values
            '03020200',                # invalid boolean
            '021f00',                  # unknown type
            '020302',                  # no final STOP
        ]:
            with self.subTest(data=data), self.assertRaises(DecodeError):
                parse(bytes.fromhex(data))

    def test_signed_numbers_and_extended_field_ids(self):
        # int16 -2, int32 +1, int64 -1; uint8 fields 200 and 256.
        node = parse(bytes.fromhex('0e0f0330025101c3c807e300010800'))['root']
        self.assertEqual([field(node, i)['value'] for i in (0, 1, 2, 200, 256)], [-2, 1, -1, 7, 8])

    def test_lists_maps_and_base_field_ambiguity(self):
        # A uint8 list with two elements, followed by uint8->int32 map {4:-2}.
        node = parse(bytes.fromhex('0b0b6309082d031001040300'))['root']
        self.assertEqual([x['value'] for x in field(node, 0)['value']['items']], [9, 8])
        self.assertEqual([x['value'] for x in field(node, 1)['value']['pairs'][0]], [4, -2])
        base = parse(bytes.fromhex('06030101030200'))['root']
        self.assertIsNone(field(base, 0))  # same ID in different base scopes

    def test_budget_and_depth_limits(self):
        data = bytes.fromhex(self.records[0]['hex'])
        for limits in [dict(max_values=2), dict(max_depth=1)]:
            with self.assertRaises(DecodeError):
                parse(data, **limits)
        # Declared billion-element container cannot allocate from a tiny input.
        with self.assertRaises(DecodeError):
            parse(bytes.fromhex('080b038094ebdc0300'))

    def test_padding_is_explicit_and_not_structural(self):
        with self.assertRaises(DecodeError):
            parse(b'\x01\x00\x00')
        result = parse(b'\x01\x00\x00', allow_zero_padding=True)
        self.assertEqual(result['root']['end'], 2)
        self.assertEqual(result['padding_bytes'], 1)
        with self.assertRaises(DecodeError):
            parse(b'\x01\x00\x01', allow_zero_padding=True)

    def test_nan_retains_bits_and_non_map_root_rejected(self):
        result = parse(bytes.fromhex('06070100c07f00'))
        self.assertEqual(field(result['root'], 0)['value'], {'nonfinite': 'nan', 'hex': '0100c07f'})
        json.dumps(result, allow_nan=False)
        with self.assertRaises(DecodeError):
            decode(b'\x01\x00', 'empty.mvar')


if __name__ == '__main__':
    unittest.main()
