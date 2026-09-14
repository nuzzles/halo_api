"""Compare map spawn-like placements with navigation polygons in world space.

This does not transform film coordinates or decide which placements are enabled.
The two numeric types are spawn-like from earlier controls, not a full Forge schema.
"""
import argparse
import json
from pathlib import Path

from bond_compact import DecodeError
from decode_map_variant import decode as decode_map
from decode_navmesh import MAX_BYTES, decode as decode_navmesh
from navmesh_samples import inside_polygon_xy

SPAWN_TYPES = {-361555940, -1533673853}


def compare(mesh, objects, max_height):
    polygons = [[mesh['vertices'][i] for i in polygon] for polygon in mesh['polygons']]
    rows, omitted = [], []
    for obj in objects:
        if obj['type_value'] not in SPAWN_TYPES:
            continue
        position = obj['position']['components']
        row = dict(object_index=obj['index'], type_value=obj['type_value'], position=position,
                   object_bytes=obj['byte_range'], position_bytes=obj['position']['component_bytes'], matches=[])
        if any(v is None for v in position):
            omitted.append(row)
            continue
        for index, polygon in enumerate(polygons):
            # Preserve a range for non-flat faces instead of inventing a plane.
            gap = [position[2] - max(v[2] for v in polygon),
                   position[2] - min(v[2] for v in polygon)]
            if 0 <= gap[0] <= gap[1] <= max_height and inside_polygon_xy(position, polygon):
                row['matches'].append(dict(face_index=index, height_above_vertex_z_range=gap))
        rows.append(row)
    return dict(max_height_threshold=max_height, placements=rows, omitted_coordinates=omitted,
                matched=sum(bool(r['matches']) for r in rows), unmatched=sum(not r['matches'] for r in rows))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('settings', type=Path, help='Folder with exact-revision map.mvar and navmesh.blob')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        with (args.settings / 'map.mvar').open('rb') as handle:
            variant = decode_map(handle.read(MAX_BYTES + 1), 'map.mvar')
        with (args.settings / 'navmesh.blob').open('rb') as handle:
            nav = decode_navmesh(handle.read(MAX_BYTES + 1), 'navmesh.blob')
        report = compare(nav['navmesh'], variant['objects'], .5)
        report.update(map_sha256=variant['sha256'], navmesh_sha256=nav['source_sha256'],
                      scope='Same-world-coordinate placement comparison. The 0.5-unit height threshold is an '
                            'explicit analysis choice, not an engine constant. Spawn-type labels remain provisional; '
                            'matching surfaces does not establish active spawns or film-coordinate alignment.')
        args.output.write_text(json.dumps(report, indent=2, allow_nan=False) + '\n')
    except (OSError, DecodeError) as error:
        parser.exit(1, f'{error}\n')
    print(f"{report['matched']} spawn-like placements above polygons; "
          f"{report['unmatched']} unmatched; {len(report['omitted_coordinates'])} with missing coordinates.")


if __name__ == '__main__':
    main()
