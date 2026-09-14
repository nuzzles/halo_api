"""Semantic checks for observed navigation samples, separate from film state.

Offsets 16 and 40 are corroborated against both complete Octagon navmeshes.
Distance is squared in XY to the boundary of the connected navigation region;
it is not a distance to a Forge object, a 3D collision query, or a pawn radius.
"""
import math
import struct

from bond_compact import DecodeError

UNAVAILABLE_DISTANCE = 0xff7fffff


def decode_sample(data, offset):
    if offset < 0 or offset + 44 > len(data):
        raise DecodeError('truncated navigation sample')
    xyz = struct.unpack_from('<3f', data, offset)
    face = struct.unpack_from('<I', data, offset + 16)[0]
    raw = struct.unpack_from('<I', data, offset + 40)[0]
    distance = None if raw == UNAVAILABLE_DISTANCE else struct.unpack_from('<f', data, offset + 40)[0]
    if not all(math.isfinite(v) for v in xyz) or (distance is not None and
                                               (not math.isfinite(distance) or distance < 0)):
        raise DecodeError('invalid sample position/boundary distance')
    return dict(position=xyz, face_index=face, boundary_distance_squared=distance,
                distance_raw=raw, byte_range=[offset, offset + 44])


def edge_distance_squared_xy(point, a, b):
    dx, dy = b[0] - a[0], b[1] - a[1]
    length = dx * dx + dy * dy
    t = max(0, min(1, ((point[0] - a[0]) * dx + (point[1] - a[1]) * dy) / length)) if length else 0
    return (point[0] - a[0] - t * dx) ** 2 + (point[1] - a[1] - t * dy) ** 2


def inside_polygon_xy(point, vertices):
    inside = False
    for a, b in zip(vertices, vertices[1:] + vertices[:1]):
        if edge_distance_squared_xy(point, a, b) <= 1e-12:
            return True
        if ((a[1] > point[1]) != (b[1] > point[1]) and
                point[0] < (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0]):
            inside = not inside
    return inside


def navigation_regions(mesh):
    """Group reciprocal face neighbors and collect only their boundary edges."""
    regions, face_region = [], [None] * len(mesh['faces'])
    for face_id in range(len(face_region)):
        if face_region[face_id] is not None:
            continue
        todo, faces, boundary = [face_id], [], []
        region_id = len(regions)
        while todo:
            index = todo.pop()
            if face_region[index] is not None:
                continue
            face_region[index] = region_id
            faces.append(index)
            first, _, count, _ = mesh['faces'][index]
            for edge_id in range(first, first + count):
                edge = mesh['edges'][edge_id]
                if edge[2] == 0xffffffff:
                    boundary.append(edge_id)
                else:
                    todo.append(edge[3])
        regions.append(dict(faces=sorted(faces), boundary_edges=sorted(boundary)))
    return regions, face_region


def analyze_samples(data, cells, mesh, *, edge_budget=20_000_000):
    regions, face_region = navigation_regions(mesh)
    polygons = [[mesh['vertices'][i] for i in polygon] for polygon in mesh['polygons']]
    boundary = [[(mesh['vertices'][mesh['edges'][i][0]], mesh['vertices'][mesh['edges'][i][1]])
                 for i in region['boundary_edges']] for region in regions]
    counts = [0] * len(polygons)
    outside = present = unavailable = distance_mismatches = 0
    max_xy_error = max_distance_error = max_z_range_excess = 0.0
    exceptions = []
    for cell in cells['cells']:
        face_counts = {}
        for offset in range(*cell['samples_byte_range'], 44):
            sample = decode_sample(data, offset)
            xyz, face, distance = sample['position'], sample['face_index'], sample['boundary_distance_squared']
            if face >= len(polygons):
                raise DecodeError('navigation sample face index outside navmesh')
            counts[face] += 1
            face_counts[face] = face_counts.get(face, 0) + 1
            polygon = polygons[face]
            edge_budget -= len(polygon) + (len(boundary[face_region[face]]) if distance is not None else 0)
            if edge_budget < 0:
                raise DecodeError('navigation sample geometry-check budget exceeded')
            if not inside_polygon_xy(xyz, polygon):
                outside += 1
                max_xy_error = max(max_xy_error, math.sqrt(min(
                    edge_distance_squared_xy(xyz, a, b) for a, b in zip(polygon, polygon[1:] + polygon[:1]))))
            max_z_range_excess = max(max_z_range_excess, min(v[2] for v in polygon) - xyz[2],
                                    xyz[2] - max(v[2] for v in polygon))
            if distance is None:
                unavailable += 1
                continue
            present += 1
            edges = boundary[face_region[face]]
            computed = min((edge_distance_squared_xy(xyz, a, b) for a, b in edges), default=None)
            error = abs(distance - computed) if computed is not None else None
            if error is not None:
                max_distance_error = max(max_distance_error, error)
            # A check, not a replacement measurement. Keep outliers reviewable.
            if error is None or error > max(1e-5, distance * 2 ** -21):
                distance_mismatches += 1
                if len(exceptions) < 32:
                    exceptions.append(dict(byte_offset=offset, face_index=face, recorded=distance,
                                           computed_xy_squared=computed, absolute_error=error))
        cell['face_sample_counts'] = face_counts
    return dict(face_sample_counts=counts, regions=regions,
                samples_inside_face_xy=sum(counts) - outside, samples_outside_face_xy=outside,
                max_outside_xy_distance=max_xy_error, max_z_range_excess=max_z_range_excess,
                boundary_distances_present=present, boundary_distances_unavailable=unavailable,
                boundary_distance_mismatches=distance_mismatches,
                max_boundary_distance_absolute_error=max_distance_error, exceptions=exceptions,
                interpretation='Face index and squared XY boundary distance. Connected-region boundary, '
                               'not the shared edges between neighboring faces. Unavailable values remain null. '
                               'Samples are not assumed to lie exactly on the 3D polygon surface.')
