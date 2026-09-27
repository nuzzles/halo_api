"""Bounded reader for the observed Forge navmesh.blob companion format.

The first Havok object supplies navigation polygons, not collision/object bounds.
Only the captured hkaiNavMesh and spatial-tree schemas are supported; other
schemas fail closed. Samples are linked to cells and navigation faces.
All geometry spans address the decompressed container, never the film or the
compressed source. No coordinate transform or triangulation is inferred.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import struct
import zlib

from bond_compact import DecodeError, Reader
from navmesh_samples import analyze_samples

MAX_BYTES = 64 * 1024 * 1024
SCHEMA_HASHES = {
    'TST1': '7ec16cff4ee4e2c3ecaab6e6c32f176361e033669bbce12bdc25ec022c283236',
    'TNA1': 'cfd616461c1d86b7548359fc3b5a94ad43b0473ff0c52ebbc72d2739e7c08877',
    'FST1': 'e3a84554f07455509cf466df3aa15f4de8a725c06e5b045e6e08433070056860',
    'TBDY': 'd456cf51d1a9f16594fd42b31237e722b9f708b1487eda42a3a5b1da9e9cad71',
}
TREE_SCHEMA_HASHES = {
    'TST1': '182094452305d5dfe696dc1f7c2e7a4f3cc741cfca810911b0c26bfc92854347',
    'TNA1': '65f186c5813a87e290b873d4ac32b3b96629c342f487eebde806a88f0b850ffd',
    'FST1': 'adabb9e310c8af5c12f85e390aef05405c0380af06bbae8153da40e80c266fe5',
    'TBDY': '9ea7253d81c59cfb8f7b8a8269cac7c0600118bb0c2e9cdd700a4f50e6f4fcb7',
}
TREE_REFERENCE = ('https://github.com/kishimisu/Crash-NST-Level-Editor/blob/'
                  'f1fe4f3e295789c3402d914f0571628d2f7709ef/src/Havok/BVH.cs')


def float32(value):
    try:
        rounded = struct.unpack('<f', struct.pack('<f', value))[0]
    except OverflowError as error:
        raise DecodeError('float32 tree arithmetic overflow') from error
    require(math.isfinite(rounded), 'nonfinite tree arithmetic')
    return rounded


def unpack_tree_bounds(xyz, parent_min, parent_max):
    """Squared-nibble / 226 codec with explicit, unfused float32 operations.

    Each axis byte packs a minimum inset in its high nibble and maximum inset
    in its low nibble. Bounds are conservative, not exact source-cell bounds.
    """
    lo, hi = [], []
    for code, a, b in zip(xyz, parent_min, parent_max):
        scale = float32(float32(b - a) * float32(1 / 226))
        lo.append(float32(a + float32((code >> 4) ** 2 * scale)))
        hi.append(float32(b - float32((code & 15) ** 2 * scale)))
    require(all(a <= b for a, b in zip(lo, hi)), 'inverted compressed tree bounds')
    return lo, hi


def require(condition, reason):
    if not condition:
        raise DecodeError(reason)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def unwrap(blob):
    """Read the BE envelope and a narrow Bond v2 struct, without per-byte nodes."""
    require(12 <= len(blob) <= MAX_BYTES, 'navmesh source size outside experiment limit')
    version, length, flags = struct.unpack_from('>III', blob)
    require(version == 2 and length == len(blob) - 12 and flags == 0x1fffff,
            'unsupported or inconsistent navmesh envelope')
    reader = Reader(blob)
    reader.pos = 12
    struct_length = reader.varuint(32, len(blob))
    require(reader.pos + struct_length == len(blob), 'Bond root length mismatch')
    # field 0:int32=1, field 1:list<int8>, field 2:uint32=inflated length.
    # The meaning of field 0 is not established beyond this observed version.
    require(reader.byte(len(blob)) == 0x10 and reader.varuint(32, len(blob)) == 2,
            'unsupported navmesh Bond field 0')
    require(reader.byte(len(blob)) == 0x2b, 'missing compressed byte list')
    list_header = reader.byte(len(blob))
    require(list_header & 31 == 14, 'compressed payload must be list<int8>')
    count = ((list_header >> 5) - 1 if list_header >> 5
             else reader.varuint(32, len(blob)))
    start = reader.pos
    compressed = reader.take(count, len(blob))
    end = reader.pos
    require(reader.byte(len(blob)) == 0x45, 'missing inflated length')
    size_start = reader.pos
    inflated_size = reader.varuint(32, len(blob))
    size_end = reader.pos
    require(reader.byte(len(blob)) == 0 and reader.pos == len(blob),
            'missing final Bond STOP or trailing data')
    require(0 < inflated_size <= MAX_BYTES, 'inflated size outside experiment limit')
    decoder = zlib.decompressobj()
    try:
        data = decoder.decompress(compressed, inflated_size + 1)
    except zlib.error as error:
        raise DecodeError(f'invalid navmesh zlib stream: {error}') from error
    require(decoder.eof and not decoder.unused_data and not decoder.unconsumed_tail,
            'incomplete, oversized or trailing zlib stream')
    require(len(data) == inflated_size, 'inflated length mismatch')
    return data, dict(version=version, flags_raw=flags, bond_field0=1,
                      compressed_byte_range=[start, end],
                      inflated_length=inflated_size,
                      inflated_length_bytes=[size_start, size_end])


def sections(data, start, end, *, depth=0, budget=None):
    if budget is None:
        budget = [4096]
    require(depth <= 16, 'Havok section depth limit')
    result = []
    while start < end:
        require(budget[0] > 0 and end - start >= 8, 'truncated/excessive Havok sections')
        budget[0] -= 1
        word = struct.unpack_from('>I', data, start)[0]
        size, kind = word & 0x3fffffff, word >> 30
        require(kind in (0, 1) and 8 <= size <= end - start, 'invalid Havok section boundary')
        tag = data[start + 4:start + 8]
        require(all(32 <= b <= 126 for b in tag), 'invalid Havok section tag')
        node = dict(tag=tag.decode('ascii'), byte_range=[start, start + size], kind=kind)
        if kind == 0:
            node['children'] = sections(data, start + 8, start + size,
                                        depth=depth + 1, budget=budget)
        result.append(node)
        start += size
    return result


def child(node, tag, kind):
    matches = [s for s in node.get('children', []) if s['tag'] == tag]
    require(len(matches) == 1 and matches[0]['kind'] == kind, f'missing/ambiguous {tag} section')
    return matches[0]


def payload(data, node):
    start, end = node['byte_range']
    return data[start + 8:end]


def read_navmesh(data, start=0, end=None):
    """Read one complete TAG0; start/end and returned spans share one byte space."""
    end = len(data) if end is None else end
    require(0 <= start <= end <= len(data) <= MAX_BYTES, 'invalid TAG0 input range')
    roots = sections(data, start, end)
    require(len(roots) == 1 and roots[0]['tag'] == 'TAG0' and roots[0]['kind'] == 0,
            'expected one TAG0 container')
    root = roots[0]
    require(payload(data, child(root, 'SDKV', 1)) == b'20220100', 'unsupported Havok SDK')
    types = child(root, 'TYPE', 0)
    for tag, expected in SCHEMA_HASHES.items():
        require(sha256(payload(data, child(types, tag, 1))) == expected,
                f'unsupported hkaiNavMesh schema: {tag}')
    data_section = child(root, 'DATA', 1)
    data_start = data_section['byte_range'][0] + 8
    values = payload(data, data_section)
    require(len(values) >= 240, 'truncated hkaiNavMesh root')
    index = child(root, 'INDX', 0)
    item_section = child(index, 'ITEM', 1)
    item_bytes = payload(data, item_section)
    require(len(item_bytes) % 12 == 0 and len(item_bytes) >= 24, 'invalid ITEM table')
    items = list(struct.iter_unpack('<III', item_bytes))
    require(items[0] == (0, 0, 0) and items[1] == (0x10000001, 0, 1),
            'unsupported root ITEM')
    for kind, offset, count in items[1:]:
        require(kind >> 28 in (1, 2) and 0 < (kind & 0x0fffffff) < 67
                and offset < len(values) and count > 0, 'invalid ITEM descriptor')
    patch_section = child(index, 'PTCH', 1)
    patch_bytes = payload(data, patch_section)
    patches, pos = {}, 0
    while pos < len(patch_bytes):
        require(len(patch_bytes) - pos >= 8, 'truncated PTCH group')
        type_id, count = struct.unpack_from('<II', patch_bytes, pos)
        pos += 8
        require(0 < type_id < 67 and 0 < count <= (len(patch_bytes) - pos) // 4,
                'invalid PTCH count/type')
        for _ in range(count):
            slot = struct.unpack_from('<I', patch_bytes, pos)[0]
            pos += 4
            require(slot not in patches and slot + 8 <= len(values), 'invalid PTCH slot')
            require(struct.unpack_from('<Q', values, slot)[0] < len(items),
                    'PTCH points outside ITEM table')
            patches[slot] = type_id

    arrays = {}
    occupied = [(0, 240)]
    for name, slot, patch_type, item_type, stride in [
        ('faces', 24, 3, 4, 12), ('edges', 40, 6, 7, 20), ('vertices', 56, 8, 9, 16),
    ]:
        require(patches.get(slot) == patch_type, f'missing {name} root relocation')
        item_id = struct.unpack_from('<Q', values, slot)[0]
        kind, offset, count = items[item_id]
        require(kind == 0x20000000 | item_type, f'wrong {name} ITEM type')
        require(count <= 1_000_000 and offset + count * stride <= len(values),
                f'{name} array exceeds DATA')
        stop = offset + count * stride
        require(all(stop <= a or offset >= b for a, b in occupied), 'overlapping geometry arrays')
        occupied.append((offset, stop))
        arrays[name] = dict(count=count, stride=stride, item_id=item_id,
                            byte_range=[data_start + offset, data_start + stop],
                            root_pointer_bytes=[data_start + slot, data_start + slot + 8],
                            item_bytes=[item_section['byte_range'][0] + 8 + item_id * 12,
                                        item_section['byte_range'][0] + 20 + item_id * 12])

    def rows(name, fmt):
        a, b = arrays[name]['byte_range']
        return list(struct.iter_unpack(fmt, data[a:b]))

    vertices = rows('vertices', '<4f')
    edges = rows('edges', '<4IBBH')
    faces = rows('faces', '<iiHH')
    aabb = struct.unpack_from('<8f', values, 144)
    up = struct.unpack_from('<4f', values, 192)
    require(all(math.isfinite(v) for row in vertices + [aabb, up] for v in row),
            'nonfinite navigation geometry')
    require(all(aabb[i] <= aabb[i + 4] for i in range(3)), 'inverted navigation AABB')
    require(all(all(aabb[i] <= v[i] <= aabb[i + 4] for i in range(3)) for v in vertices),
            'vertex outside navigation AABB')
    owners, polygons = {}, []
    for face_id, (first, user_first, count, user_count) in enumerate(faces):
        require(user_first == -1 and user_count == 0, 'user edges are not supported yet')
        require(3 <= count and 0 <= first and first + count <= len(edges), 'invalid face edge range')
        indices = []
        for edge_id in range(first, first + count):
            require(edge_id not in owners, 'overlapping face edge ranges')
            owners[edge_id] = face_id
            a, b = edges[edge_id][:2]
            require(a < len(vertices) and b < len(vertices) and a != b, 'invalid edge vertices')
            next_id = first + (edge_id - first + 1) % count
            require(b == edges[next_id][0], 'face edge loop is not closed')
            indices.append(a)
        polygons.append(indices)
    require(len(owners) == len(edges), 'unowned navigation edges')
    paired = 0
    for edge_id, edge in enumerate(edges):
        a, b, opposite, opposite_face = edge[:4]
        if opposite == 0xffffffff:
            require(opposite_face == 0xffffffff, 'incomplete boundary sentinel')
        else:
            require(opposite < len(edges) and opposite_face < len(faces),
                    'external/invalid opposite edge is not supported')
            other = edges[opposite]
            require(other[:2] == (b, a) and other[2] == edge_id
                    and owners[opposite] == opposite_face and other[3] == owners[edge_id],
                    'nonreciprocal opposite edge/face')
            paired += 1
    return dict(sdk='20220100', schema_sha256=SCHEMA_HASHES, sections=root, arrays=arrays,
                vertices=vertices, faces=faces, edges=edges, polygons=polygons,
                edge_fields=['a', 'b', 'opposite_edge', 'opposite_face', 'flags_raw',
                             'padding_byte_raw', 'user_edge_cost_raw'],
                face_fields=['start_edge', 'start_user_edge', 'edge_count', 'user_edge_count'],
                navigation_aabb=dict(min=aabb[:4], max=aabb[4:],
                                     byte_range=[data_start + 144, data_start + 176]),
                up=dict(value=up, byte_range=[data_start + 192, data_start + 208]),
                vertex_extents=dict(min=[min(v[i] for v in vertices) for i in range(3)],
                                    max=[max(v[i] for v in vertices) for i in range(3)]),
                validation=dict(closed_polygons=len(polygons), shared_edge_pairs=paired // 2))


def read_spatial_tree(data, start, end):
    """Read the fourth TAG0's schema-gated hkcdStaticAabbTree and six-byte nodes."""
    require(0 <= start <= end <= len(data) <= MAX_BYTES, 'invalid tree TAG0 range')
    roots = sections(data, start, end)
    require(len(roots) == 1 and roots[0]['tag'] == 'TAG0' and roots[0]['kind'] == 0,
            'expected one tree TAG0')
    root = roots[0]
    require(payload(data, child(root, 'SDKV', 1)) == b'20220100', 'unsupported tree SDK')
    types = child(root, 'TYPE', 0)
    for tag, expected in TREE_SCHEMA_HASHES.items():
        require(sha256(payload(data, child(types, tag, 1))) == expected,
                f'unsupported spatial tree schema: {tag}')
    data_section = child(root, 'DATA', 1)
    base = data_section['byte_range'][0] + 8
    values = payload(data, data_section)
    index = child(root, 'INDX', 0)
    item_section = child(index, 'ITEM', 1)
    item_bytes = payload(data, item_section)
    require(len(item_bytes) == 48 and len(values) >= 32, 'unsupported tree ITEM/root shape')
    items = list(struct.iter_unpack('<III', item_bytes))
    require(items[0] == (0, 0, 0) and items[1] == (0x10000001, 0, 1), 'invalid tree root ITEM')
    require(payload(data, child(index, 'PTCH', 1)) == struct.pack('<III', 3, 1, 24),
            'unsupported tree pointer relocation')
    impl_id = struct.unpack_from('<Q', values, 24)[0]
    require(1 < impl_id < len(items), 'invalid tree implementation pointer')
    kind, impl, count = items[impl_id]
    require(kind == 0x10000004 and count == 1 and impl >= 32 and impl + 80 <= len(values),
            'invalid tree implementation ITEM')
    tree_start = impl + 32
    relative, count, capacity_raw = struct.unpack_from('<qII', values, tree_start)
    nodes_start = tree_start + relative
    require(relative >= 48 and 0 < count <= 1_000_000 and count % 2 == 1
            and nodes_start + count * 6 <= len(values), 'invalid relative tree node array')
    matches = [i for i, row in enumerate(items) if row == (0x2000000c, nodes_start, count)]
    require(len(matches) == 1, 'tree relative array disagrees with ITEM')
    domain = struct.unpack_from('<8f', values, tree_start + 16)
    require(all(math.isfinite(v) for v in domain)
            and all(domain[i] <= domain[i + 4] for i in range(3)), 'invalid tree domain')
    # Subtree end boundaries prove that every record belongs to the DFS tree,
    # and that no child is skipped, shared, overlapping, or cyclic.
    stack = [(0, count, None, 0, domain[:3], domain[4:7])]
    nodes = [None] * count
    leaf_ids = set()
    while stack:
        node_id, subtree_end, parent, depth, lo, hi = stack.pop()
        require(0 <= node_id < subtree_end <= count and nodes[node_id] is None and depth <= 64,
                'invalid/excessive tree traversal')
        offset = nodes_start + node_id * 6
        x, y, z, high, low = struct.unpack_from('<4BH', values, offset)
        lo, hi = unpack_tree_bounds([x, y, z], lo, hi)
        value = ((high & 0x7f) << 16) | low
        node = dict(byte_range=[base + offset, base + offset + 6], parent=parent, depth=depth,
                    xyz_codes=[x, y, z], high_raw=high, low_raw=low, bounds_min=lo, bounds_max=hi,
                    children=None, cell_index=None)
        if high & 0x80:
            left, right = node_id + 1, node_id + value * 2
            require(left < right < subtree_end, 'invalid tree right-child offset')
            node['children'] = [left, right]
            stack.extend([(right, subtree_end, node_id, depth + 1, lo, hi),
                          (left, right, node_id, depth + 1, lo, hi)])
        else:
            require(node_id + 1 == subtree_end and value not in leaf_ids,
                    'truncated tree subtree or duplicate cell index')
            node['cell_index'] = value
            leaf_ids.add(value)
        nodes[node_id] = node
    require(all(n is not None for n in nodes), 'unvisited tree nodes')
    return dict(schema_sha256=TREE_SCHEMA_HASHES, codec_reference=TREE_REFERENCE, sections=root,
                domain=dict(min=domain[:4], max=domain[4:],
                            byte_range=[base + tree_start + 16, base + tree_start + 48]),
                node_array=dict(byte_range=[base + nodes_start, base + nodes_start + count * 6],
                                stride=6, count=count, item_id=matches[0],
                                relative_pointer_bytes=[base + tree_start, base + tree_start + 8],
                                count_bytes=[base + tree_start + 8, base + tree_start + 12],
                                capacity_flags_raw=capacity_raw),
                node_count=count, leaf_count=len(leaf_ids), max_depth=max(n['depth'] for n in nodes),
                arithmetic='Squared nibble / 226; unfused float32 intermediate operations. '
                           'Conservative navigation-cell bounds, not Forge-object geometry.',
                nodes=nodes)


def link_tree_cells(tree, cells):
    require(tree['leaf_count'] == cells['cell_count'], 'tree leaf count differs from spatial cell count')
    by_cell = [None] * cells['cell_count']
    for index, node in enumerate(tree['nodes']):
        cell_id = node['cell_index']
        if cell_id is None:
            continue
        require(cell_id < len(by_cell) and by_cell[cell_id] is None, 'invalid/duplicate tree cell index')
        cell = cells['cells'][cell_id]
        require(all(node['bounds_min'][i] <= cell['bounds_min'][i]
                    <= cell['bounds_max'][i] <= node['bounds_max'][i] for i in range(3)),
                'tree leaf bounds do not contain referenced cell')
        by_cell[cell_id] = index
    require(all(i is not None for i in by_cell), 'spatial cells missing from tree')
    return dict(cells_matched=len(by_cell), all_leaf_bounds_contain_cells=True,
                leaf_node_by_cell=by_cell)


def read_spatial_cells(data, start, end):
    """Read the observed fifth segment: count, then (max/min XYZ, count, samples).

    This structural pass reads only XYZ from each 44-byte sample. The subsequent
    analyze_samples pass identifies the face index and boundary distance fields.
    Uniform boxes and contained points support spatial cells, not Forge-object
    identity, collision volumes, or a known navigation-build schema.
    """
    require(0 <= start <= end <= len(data) <= MAX_BYTES and end - start >= 4,
            'truncated/oversized spatial cell block')
    count = struct.unpack_from('<I', data, start)[0]
    require(count <= (end - start - 4) // 28, 'spatial cell count exceeds block')
    pos, samples, cells = start + 4, 0, []
    lo, hi = [math.inf] * 3, [-math.inf] * 3
    for _ in range(count):
        require(end - pos >= 28, 'truncated spatial cell header')
        cell_start = pos
        bounds = struct.unpack_from('<6f', data, pos)
        sample_count = struct.unpack_from('<I', data, pos + 24)[0]
        require(all(math.isfinite(v) for v in bounds)
                and all(bounds[i] >= bounds[i + 3] for i in range(3)), 'invalid spatial cell bounds')
        pos += 28
        require(sample_count <= (end - pos) // 44, 'spatial sample count exceeds block')
        samples_start = pos
        for _ in range(sample_count):
            xyz = struct.unpack_from('<3f', data, pos)
            require(all(math.isfinite(v) for v in xyz), 'nonfinite spatial sample position')
            require(all(bounds[i + 3] <= xyz[i] <= bounds[i] for i in range(3)),
                    'spatial sample outside enclosing bounds')
            for i in range(3):
                lo[i], hi[i] = min(lo[i], xyz[i]), max(hi[i], xyz[i])
            pos += 44
        cells.append(dict(byte_range=[cell_start, pos], bounds_max=bounds[:3], bounds_min=bounds[3:],
                          sample_count=sample_count, samples_byte_range=[samples_start, pos]))
        samples += sample_count
    require(pos == end, 'trailing bytes after spatial cell records')
    return dict(byte_range=[start, end], cell_count=count, sample_count=samples,
                sample_stride=44, sample_xyz_range=[0, 12], sample_opaque_range=[12, 44],
                scope='Spatial sample cells; not identified Forge objects or collision bounds. '
                      'Only sample XYZ and enclosing min/max are interpreted; all other sample words stay opaque.',
                byte_accounting=dict(geometry_values=count * 24 + samples * 12,
                                     checked_counts=4 + count * 4, opaque_sample_fields=samples * 32),
                sample_extents=dict(min=lo, max=hi) if samples else None, cells=cells)


def write_point_cloud(data, cells, path):
    """Export only original XYZ floats; no inferred edges/normals or downsampling."""
    header = ('ply\nformat binary_little_endian 1.0\n'
              'comment Navigation companion spatial samples; not collision geometry.\n'
              'comment Original world coordinates, Z up; no inferred connectivity.\n'
              f"element vertex {cells['sample_count']}\n"
              'property float x\nproperty float y\nproperty float z\nend_header\n')
    with path.open('wb') as handle:
        handle.write(header.encode('ascii'))
        for cell in cells['cells']:
            start, end = cell['samples_byte_range']
            for pos in range(start, end, 44):
                handle.write(data[pos:pos + 12])


def decode(blob, source):
    data, envelope = unwrap(blob)
    require(len(data) >= 32 and struct.unpack_from('>II', data) == (2, 1),
            'unsupported decompressed container header')
    sizes = struct.unpack_from('>5I', data, 8)
    require(28 + sum(sizes) + 4 == len(data) and data[-4:] == bytes(4),
            'container segment lengths/trailer mismatch')
    segments, start = [], 28
    for i, size in enumerate(sizes):
        end = start + size
        segment = dict(index=i, byte_range=[start, end])
        if i < 4:
            roots = sections(data, start, end)
            require(len(roots) == 1 and roots[0]['tag'] == 'TAG0' and roots[0]['kind'] == 0,
                    'expected four TAG0 segments')
            segment.update(status='selected geometry decoded' if i == 0 else 'checked sections; opaque values',
                           sections=roots[0])
        else:
            segment['status'] = 'spatial bounds and sample XYZ; remaining sample fields opaque'
        segments.append(segment)
        start = end
    mesh = read_navmesh(data, *segments[0]['byte_range'])
    cells = read_spatial_cells(data, *segments[4]['byte_range'])
    tree = read_spatial_tree(data, *segments[3]['byte_range'])
    tree['cell_links'] = link_tree_cells(tree, cells)
    sample_links = analyze_samples(data, cells, mesh)
    cells.pop('sample_opaque_range')
    cells.update(sample_face_index_range=[16, 20], sample_boundary_distance_squared_range=[40, 44],
                 sample_opaque_ranges=[[12, 16], [20, 40]],
                 scope='Spatial cells linked by tree leaf indices. Samples reference navigation faces '
                       'and may supply squared XY boundary distances. Not Forge objects or collision bounds.')
    cells['byte_accounting'].update(decoded_face_indices=cells['sample_count'] * 4,
                                    decoded_boundary_distance_fields=cells['sample_count'] * 4,
                                    opaque_sample_fields=cells['sample_count'] * 24)
    segments[3]['status'] = 'spatial tree nodes, relative bounds, and cell indices decoded; other fields opaque'
    segments[4]['status'] = 'cell bounds, sample XYZ, face indices, boundary distances; 24 bytes per sample opaque'
    return dict(source=source, source_bytes=len(blob), source_sha256=sha256(blob),
                decompressed_bytes=len(data), decompressed_sha256=sha256(data),
                scope='External map navigation surfaces, not full map/collision/object bounds. '
                      'Not film semantic coverage. No replay coordinate transform is applied.',
                byte_space='All section/geometry ranges address the decompressed container; '
                           'envelope ranges alone address the original navmesh.blob.',
                envelope=envelope, segments=segments, navmesh=mesh, spatial_cells=cells,
                spatial_tree=tree, sample_links=sample_links)


def obj_text(mesh):
    lines = ['# Recorded navigation polygons; no triangulation or coordinate transform.',
             '# Not collision geometry or all-object bounds. Z is up.', 'o navigation_surfaces']
    lines += ['v ' + ' '.join(format(v, '.9g') for v in vertex[:3]) for vertex in mesh['vertices']]
    for index, polygon in enumerate(mesh['polygons']):
        lines += [f'g face_{index}', 'f ' + ' '.join(str(v + 1) for v in polygon)]
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('path', type=Path)
    parser.add_argument('--output', type=Path, required=True, help='JSON geometry with byte provenance')
    parser.add_argument('--obj', type=Path, help='Optional navigation polygons in original world coordinates')
    parser.add_argument('--preview', type=Path, help='Optional standalone HTML orbit viewer')
    parser.add_argument('--points', type=Path, help='Optional binary PLY of all spatial sample XYZs')
    args = parser.parse_args()
    try:
        with args.path.open('rb') as handle:
            blob = handle.read(MAX_BYTES + 1)
        report = decode(blob, str(args.path))
        args.output.write_text(json.dumps(report, indent=2, allow_nan=False) + '\n')
        mesh = report['navmesh']
        if args.obj:
            args.obj.write_text(obj_text(mesh))
        if args.preview:
            from navmesh_preview import html_text
            args.preview.write_text(html_text(report))
        if args.points:
            data, _ = unwrap(blob)
            write_point_cloud(data, report['spatial_cells'], args.points)
    except (OSError, DecodeError) as error:
        parser.exit(1, f'{error}\n')
    print(f"{len(mesh['vertices'])} vertices, {len(mesh['polygons'])} closed polygons, "
          f"{mesh['validation']['shared_edge_pairs']} shared edge pairs; "
          f"{report['spatial_cells']['cell_count']} spatial cells, "
          f"{report['spatial_cells']['sample_count']} samples. "
          f"{report['spatial_tree']['node_count']} tree nodes linked to all cells. "
          'Other sample fields and Havok object values remain opaque.')


if __name__ == '__main__':
    main()
