"""Inspect a downloaded .mvar offline; export object fields and optional Bond tree.

Numeric types, identifiers and byte ranges are retained. Missing vector axes are
null, not assumed zero. This does not load geometry or add replay observations.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path

from bond_compact import DecodeError, field, parse

MAX_ASSET_BYTES = 64 * 1024 * 1024
SHAPE_REFERENCE = ('https://github.com/JGtm/LevelUp/blob/'
                   'cf333a3889771c6462dfce9e1bc287a897043a47/'
                   '.ai/ETAT_DE_L_ART_FORGE_PALETTE_ZONES.md')


def vector(node):
    result, spans = [], []
    for axis in range(3):
        component = field(node, axis, 7)
        value = component['value'] if component else None
        if not isinstance(value, (int, float)) or not math.isfinite(value):
            value = None
        result.append(value)
        spans.append([component['offset'], component['end']] if component else None)
    return dict(components=result, component_bytes=spans)


def shape_parameters(node):
    """Preserve the separate volume bag; this is not the object's mesh bounds.

    Family 2=cylinder, 3=box follows the pinned external schema. The raw scalars
    are independently readable; shape semantics and effective gameplay usage
    have not been validated by a controlled boundary crossing in these films.
    Empty wrappers are unknown, never an implicit zero dimension.
    """
    bags = field(field(node, 8, 10), 0, 11)
    if bags is None or bags['value']['element_type'] != 10:
        return []
    result = []
    for bag in bags['value']['items']:
        shapes = field(bag, 0, 11)
        if shapes is None or shapes['value']['element_type'] != 10:
            continue
        for shape in shapes['value']['items']:
            family = field(shape, 0, 16)
            family_code = family['value'] if family else None
            parameters = []
            for field_id in (5, 6, 7, 8):
                scalar = field(field(shape, field_id, 10), 0, 16)
                parameters.append(dict(
                    field_id=field_id,
                    raw=scalar['value'] if scalar else None,
                    fixed16_16=scalar['value'] / 65536 if scalar else None,
                    value_bytes=[scalar['offset'], scalar['end']] if scalar else None))
            result.append(dict(
                byte_range=[shape['offset'], shape['end']], family_code=family_code,
                family_bytes=[family['offset'], family['end']] if family else None,
                family_candidate={2: 'cylinder', 3: 'box'}.get(family_code),
                parameters=parameters))
    return result


def object_fields(node, index):
    raw_type = field(field(node, 2, 10), 0, 16)
    return dict(index=index, byte_range=[node['offset'], node['end']],
                type_value=raw_type['value'] if raw_type else None,
                type_bytes=[raw_type['offset'], raw_type['end']] if raw_type else None,
                position=vector(field(node, 3, 10)),
                field4_vector=vector(field(node, 4, 10)),
                field5_vector=vector(field(node, 5, 10)),
                shape_parameters=shape_parameters(node))


def decode(data, source_name, *, include_tree=False):
    if len(data) > MAX_ASSET_BYTES:
        raise DecodeError('map asset exceeds 64 MiB experiment limit')
    parsed = parse(data)
    root = parsed['root']
    collection = field(root, 3, 11)
    if collection is None or collection['value']['element_type'] != 10:
        raise DecodeError('not the observed map schema: root field 3 must be list<struct>')
    level = field(field(field(root, 1, 10), 0, 10), 0, 16)
    report = dict(
        source=source_name, sha256=hashlib.sha256(data).hexdigest(), bytes=len(data),
        format='Bond Compact Binary v2', structure_bytes=root['end'],
        scope='External map asset; structural coverage is not film semantic coverage. '
              'Missing components stay null; type identifiers and orientation semantics are provisional.',
        shape_reference=SHAPE_REFERENCE,
        shape_scope='Separate volume parameters, not mesh/collision bounds or active objectives. '
                    'Family labels follow the external schema; units and effective usage remain unverified.',
        metadata_field_1_0_0=level['value'] if level else None,
        objects=[object_fields(o, i) for i, o in enumerate(collection['value']['items'])])
    if include_tree:
        report['tree'] = root
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('asset', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--tree', action='store_true', help='include all numeric fields and byte spans')
    args = parser.parse_args()
    if args.asset.resolve() == args.output.resolve():
        parser.error('output must differ from the source asset')
    try:
        with args.asset.open('rb') as source:
            data = source.read(MAX_ASSET_BYTES + 1)
        report = decode(data, args.asset.name, include_tree=args.tree)
    except (OSError, DecodeError) as error:
        parser.exit(1, f'{error}\n')
    args.output.write_text(json.dumps(report, indent=2, allow_nan=False) + '\n')
    shapes = sum(len(obj['shape_parameters']) for obj in report['objects'])
    print(f"{len(report['objects'])} object records; {report['structure_bytes']}/{report['bytes']} "
          f"bytes structurally parsed; {shapes} volume parameter records → {args.output}")


if __name__ == '__main__':
    main()
