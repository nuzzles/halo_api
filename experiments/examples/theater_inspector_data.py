"""Read-only, evidence-backed bit annotations for Theater Lab's inspector.

Ranges are half-open, absolute bits in a decompressed chunk, MSB-first within
bytes. Unannotated bits remain unparsed. A checked length is not a semantic decode.
"""

from bisect import bisect_right
from collections import defaultdict
import csv
from functools import lru_cache
import json
from pathlib import Path
import struct

from legacy.build_bandit_scene import CLOCK, SPAWN_COORD_PREFIX, delta_fields
from legacy.build_octagon_scene import COORD_PREFIX, COORD_SUFFIX, SPAWN_BODY
from legacy.build_oddball_scene import CLOCKS, spawn_fields
from film_catalog import EXPERIMENTS_ROOT, load_catalog
from legacy.film_firing_probe import firing_fields
from legacy.film_melee_probe import melee_fields
from legacy.film_grenade_probe import grenade_fields, projectile_fields, PROJECTILE_SPAWN, PROJECTILE_END
from legacy.film_vitality_probe import vitality_window, isolated_vitality
from legacy.film_weapon_probe import reload_fields, weapon_delta
from legacy.film_zoom_probe import zoom_fields

STATUSES = ('decoded', 'structure', 'opaque', 'unparsed')
RANKED = {'bandit/01-evo': 'bandit', 'ranked-arena/02-oddball': 'oddball'}


def bit_string(data):
    return ''.join(f'{b:08b}' for b in data)


def field(start, end, label, value, status='decoded', record='Packet header', source='', note=''):
    if not 0 <= start < end:
        raise ValueError('Invalid field range')
    return dict(start=start, end=end, label=label, value=value, status=status,
                record=record, source=source, note=note)


def partition(start, end, fields):
    """Partition every bit exactly once; narrow semantic overlays replace opaque spans."""
    fields = [f for f in fields if f['start'] < end and f['end'] > start]
    edges = sorted({start, end} | {max(start, f['start']) for f in fields}
                   | {min(end, f['end']) for f in fields})
    result = []
    for a, b in zip(edges, edges[1:]):
        covering = [f for f in fields if f['start'] <= a and f['end'] >= b]
        winner = min(covering, key=lambda f: STATUSES.index(f['status'])) if covering else None
        row = dict(winner, start=a, end=b) if winner else field(
            a, b, 'Unparsed region', None, 'unparsed', 'Unparsed', note='No supported annotation covers these bits. This may include fields recognized by other probes.')
        row['field_start'] = winner['start'] if winner else a
        row['field_end'] = winner['end'] if winner else b
        if result and all(result[-1][k] == row[k] for k in row if k not in ('start', 'end', 'field_start', 'field_end')) and result[-1]['end'] == a:
            result[-1]['end'] = b
            result[-1]['field_end'] = max(result[-1]['field_end'], row['field_end'])
        else:
            result.append(row)
    return result


def coverage(spans):
    counts = dict.fromkeys(STATUSES, 0)
    for s in spans:
        counts[s['status']] += s['end'] - s['start']
    return counts


def annotate_delta(bits, offset, base, clock, source, name, serial, generation=1, vitality=False):
    parsed = delta_fields(bits, offset, clock, generation)
    if parsed is None:
        raise ValueError('Sparse delta no longer matches its export')
    result = []
    record = f'{name} · life {serial} · pawn delta'
    def add(a, n, label, value, status='decoded', note=''):
        result.append(field(base + a, base + a + n, label, value, status, record, source, note))
    add(offset, 14, 'Wire identity', int(bits[offset:offset + 14], 2) - 8704)
    add(offset + 14, 2, 'Generation tag', generation, note='Observed 01 → 10 on Oddball ID reuse; later generations are unverified.')
    add(offset + 16, 2, 'Sparse mode', '00', 'structure')
    ids = parsed['components']
    add(offset + 18, 3, 'Component count', len(ids))
    for i, component in enumerate(ids):
        add(offset + 21 + i * 6, 6, f'Component index {i}', component)
    cursor = offset + 21 + 6 * len(ids)
    for component in ids:
        if component == 0:
            add(cursor, 5, 'Position flags', '00000', 'structure')
            for delta, width, axis, value in zip((5, 23, 41), (18, 18, 15), 'XYZ', parsed['position']):
                add(cursor + delta, width, f'{axis} raw', value, note='Map-specific integer window; world units are uncalibrated.')
            add(cursor + 56, 2, 'Position suffix', '00', 'structure')
            cursor += 58
        elif component == 1:
            n = 31 if bits[cursor:cursor + 2] == '00' else 2
            add(cursor, 2, 'Velocity form', bits[cursor:cursor + 2], 'structure')
            if n > 2:
                add(cursor + 2, n - 2, 'Velocity payload', None, 'opaque', 'Length is checked; velocity encoding is not decoded.')
            cursor += n
        elif component in (4, 5):
            n = 11 if component == 4 else 29
            add(cursor, n, f'Component {component} payload', None, 'opaque', 'Checked field length; no semantic coverage unless a validated vitality window overlays it.')
            if vitality:
                try:
                    values = vitality_window(component, bits[cursor:cursor + n])
                except ValueError:
                    values = None
                if values:
                    result.extend(annotate_vitality(base + cursor, component, values, record, source))
            cursor += n
        elif component == 21:
            add(cursor, 1, 'Aim prefix', '1', 'structure')
            add(cursor + 1, 12, 'Yaw raw', parsed['aim'][0], note='Cyclic display angle: raw × 360 / 4096 degrees.')
            add(cursor + 13, 11, 'Pitch raw', parsed['aim'][1], note='Provisional display angle: (raw − 1024) × 360 / 2048 degrees.')
            add(cursor + 24, 1, 'Aim suffix', '0', 'structure')
            cursor += 25
        elif component == 25:
            add(cursor, 1, 'Tick prefix', '1', 'structure')
            add(cursor + 1, 8, 'Repeated clock counter', clock, note='Checked against the frame clock.')
            add(cursor + 9, 1, 'Tick suffix', '0', 'structure')
            break
    return result, parsed


def annotate_vitality(start, component, values, record, source):
    def f(a, n, label, value, status='decoded', note=''):
        return field(start + a, start + a + n, label, value, status, record, source, note)
    if component == 4:
        return [f(0, 1, 'Body prefix', '1', 'structure'),
                f(1, 7, 'Body health raw', values['body_raw'], note='Provisional full scale 126, not calibrated hit points.'),
                f(8, 3, 'Body state code', values['body_state'], 'structure')]
    return [f(0, 8, 'Shields raw', values['shield_raw'], note='Provisional full scale 64, not calibrated hit points.'),
            f(8, 8, 'Shield guard', '00000000', 'structure'),
            f(16, 9, 'Recharge delay ticks', values['shield_delay_ticks']),
            f(25, 4, 'Shield state code', values['shield_state'], 'structure')]


def annotate_spawn(bits, row, base, oddball=False):
    offset, serial = row['bit'], row['id']
    if oddball:
        decoded = spawn_fields(bits, offset)
        if decoded is None or any(decoded[k] != row[k] for k in decoded):
            raise ValueError('Spawn no longer matches its export')
        shift, wire, generation = row['shift'], row['wire'], row['generation']
    else:
        shift, wire, generation = 0, serial, 1
        if (bits[offset + 18:offset + 69] != SPAWN_BODY
                or int(bits[offset + 1:offset + 18], 2) != 8704 + serial
                or int(bits[offset + 69:offset + 74], 2) != row['player']):
            raise ValueError('Spawn identity mismatch')
    if (bits[offset + 192 + shift:offset + 255 + shift] != SPAWN_COORD_PREFIX
            or bits[offset + 314 + shift:offset + 344 + shift] != COORD_SUFFIX):
        raise ValueError('Spawn coordinate guard mismatch')
    result = []
    def add(a, n, label, value, status='decoded'):
        result.append(field(base + offset + a, base + offset + a + n, label, value, status,
                            f"{row['name']} · life {serial} · spawn", 'build_' + ('oddball' if oddball else 'bandit') + '_scene.py'))
    add(1, 17, 'Wire identity', wire)
    add(18, 2, 'Generation tag', generation)
    add(20, 49, 'Spawn signature', SPAWN_BODY[2:], 'structure')
    add(69, 5, 'Roster index', row['player'])
    add(192 + shift, 63, 'Coordinate prefix', SPAWN_COORD_PREFIX, 'structure')
    for a, n, axis, value in zip((263, 281, 299), (18, 18, 15), 'XYZ', row['xyz']):
        if int(bits[offset + a + shift:offset + a + shift + n], 2) != value:
            raise ValueError('Spawn coordinate mismatch')
        add(a + shift, n, f'{axis} raw', value)
    add(314 + shift, 30, 'Coordinate suffix', COORD_SUFFIX, 'structure')
    return result


class Inspector:
    def __init__(self, corpus=EXPERIMENTS_ROOT / 'films'):
        self.corpus = Path(corpus).resolve()
        self.catalog = {f"{r['group']}/{r['slug']}": r for r in load_catalog()}

    def path(self, relative):
        path = (self.corpus / relative).resolve()
        if not path.is_relative_to(self.corpus):
            raise ValueError('Path is outside the film corpus')
        return path

    @lru_cache(maxsize=2)
    def film(self, label):
        entry = self.catalog[label]
        metadata = json.loads(self.path(label + '/film.json').read_text())
        if metadata['match_id'] != entry['match_id']:
            raise ValueError('Catalog / film match ID mismatch')
        packets, origin = {}, None
        for chunk in metadata['chunks']:
            if chunk['chunk_type'] != 2:
                continue
            data = self.path(label + '/' + chunk['file']).read_bytes()
            rows, offset = [], 0
            while offset < len(data):
                if offset + 16 > len(data):
                    raise ValueError('Truncated packet header')
                kind, unknown, size, timestamp = struct.unpack_from('<HHIQ', data, offset)
                if offset + 16 + size > len(data):
                    raise ValueError('Truncated packet payload')
                rows.append(dict(offset=offset, payload=offset + 16, size=size, kind=kind,
                                 unknown=unknown, timestamp=timestamp))
                if timestamp:
                    origin = min(origin, timestamp) if origin is not None else timestamp
                offset += 16 + size
            packets[chunk['index']] = rows
        for rows in packets.values():
            for row in rows:
                row['time'] = (row['timestamp'] - origin) / 1e6 if row['timestamp'] and origin is not None else None
        return metadata, packets

    def describe(self, label):
        metadata, packets = self.film(label)
        result = {**self.catalog[label], 'label': label, 'version': metadata['film_major_version'],
                  'duration': metadata['film_length'] / 1000, 'chunks': []}
        for chunk in metadata['chunks']:
            rows = packets.get(chunk['index'], [])
            result['chunks'].append({k: chunk[k] for k in ('index', 'file', 'chunk_type', 'start_time_offset_ms', 'duration_ms')} |
                                    {'size': self.path(label + '/' + chunk['file']).stat().st_size, 'packets': len(rows)})
        return result

    @lru_cache(maxsize=3)
    def evidence(self, label, chunk):
        """Load just one chunk's exported evidence; revalidate fields when viewed."""
        result = defaultdict(lambda: defaultdict(list))
        group = RANKED.get(label)
        def csv_rows(path, kind, label_filter=True):
            if not path.exists():
                return
            with path.open(newline='') as file:
                for row in csv.DictReader(file):
                    if int(row['chunk']) == chunk and (not label_filter or row['film'] == label):
                        result[int(row.get('payload_byte', row.get('payload_offset')))][kind].append(row)
        if group:
            proof_path = self.path(f'analysis/{group}/decode_evidence.json')
            if proof_path.exists():
                proof = json.loads(proof_path.read_text())
                if proof['match_id'] != self.catalog[label]['match_id']:
                    raise ValueError('Decoder evidence belongs to a different match')
                names = [r['player'] for r in proof['players']]
                for row in proof['spawns']:
                    if row['chunk'] == chunk:
                        result[row['payload_byte']]['spawn'].append(dict(row, name=names[row['player']]))
                csv_rows(self.path(f'analysis/{group}/delta_evidence.csv'), 'delta', False)
        else:
            for kind in ('positions', 'aim', 'inputs'):
                csv_rows(self.path(f'analysis/all-films/{kind}.csv'), kind)
        firing = self.path('analysis/firing/evidence.json')
        if firing.exists():
            for row in json.loads(firing.read_text())['records']:
                if row['film'] == label and row['chunk'] == chunk:
                    result[row['payload_byte']]['firing'].append(row)
        melee = self.path('analysis/melee/evidence.json')
        if melee.exists():
            for row in json.loads(melee.read_text())['records']:
                if row['film'] == label and row['chunk'] == chunk:
                    result[row['payload_byte']]['melee'].append(row)
        grenades = self.path('analysis/grenades/evidence.json')
        if grenades.exists():
            proof = json.loads(grenades.read_text())
            audit = next((r for r in proof['audit'] if r['film'] == label), None)
            if audit and audit['match_id'] != self.catalog[label]['match_id']:
                raise ValueError('Grenade evidence belongs to a different match')
            for kind, rows in [('grenade', proof['records']), ('projectile', proof['projectiles'])]:
                for row in rows:
                    if row['film'] == label and row['chunk'] == chunk:
                        result[row['payload_byte']][kind].append(row)
        zoom = self.path('analysis/zoom/evidence.json')
        if zoom.exists():
            proof = json.loads(zoom.read_text())
            audit = next((r for r in proof['audit'] if r['film'] == label), None)
            if audit and audit['match_id'] != self.catalog[label]['match_id']:
                raise ValueError('Scope evidence belongs to a different match')
            for row in proof['records']:
                if row['film'] == label and row['chunk'] == chunk:
                    result[row['payload_byte']]['zoom'].append(row)
        weapons = self.path('analysis/weapons/evidence.json')
        if weapons.exists():
            proof = json.loads(weapons.read_text())
            audit = next((r for r in proof['audit'] if r['film'] == label), None)
            if audit and audit['match_id'] != self.catalog[label]['match_id']:
                raise ValueError('Weapon evidence belongs to a different match')
            for kind, rows in [('reload', proof['reloads']), ('weapon', proof['deltas'])]:
                for row in rows:
                    if row['film'] == label and row['chunk'] == chunk:
                        result[row['payload_byte']][kind].append(row)
        if label.startswith('octagon/'):
            csv_rows(self.path('analysis/vitality/samples.csv'), 'vitality')
            path = self.path('analysis/octagon/spawn_evidence.json')
            if path.exists():
                for row in json.loads(path.read_text()):
                    if row['film'] == label and row['chunk'] == chunk:
                        result[row['payload_offset']]['octagon_spawn'].append(row)
        return result

    @lru_cache(maxsize=3)
    def chunk(self, label, index):
        metadata, packets = self.film(label)
        chunk = next(c for c in metadata['chunks'] if c['index'] == index)
        data = self.path(label + '/' + chunk['file']).read_bytes()
        return chunk, data, packets.get(index, [])

    def chunk_info(self, label, index):
        chunk, data, rows = self.chunk(label, index)
        evidence = self.evidence(label, index) if rows else {}
        return {'index': index, 'size': len(data), 'type': chunk['chunk_type'],
                'packets': [{k: row[k] for k in ('offset', 'size', 'kind', 'time')} |
                            {'evidence': list(evidence.get(row['payload'], {}))} for row in rows]}

    def seek(self, label, time):
        _, packets = self.film(label)
        candidates = ((abs(r['time'] - time), c, r) for c, rows in packets.items()
                      for r in rows if r['kind'] == 0 and r['time'] is not None)
        _, chunk, row = min(candidates)
        return {'chunk': chunk, 'offset': row['offset']}

    def registry(self, data):
        result = []
        for block in range(len(data) // 16640):
            for slot in range(64):
                start = block * 16640 + slot * 260
                raw = data[start + 8:start + 260].split(b'\0', 1)[0]
                if not raw or not all(33 <= b <= 126 for b in raw):
                    break
                name = raw.decode('ascii')
                result.append(field((start + 8) * 8, (start + 8 + len(raw)) * 8,
                                    'Component name', name, record=f'Registry · archetype {block} · component {slot}',
                                    source='film.rs::decode_registry'))
                if len(raw) < 252:
                    result.append(field((start + 8 + len(raw)) * 8, (start + 9 + len(raw)) * 8,
                                        'String terminator', 'NUL', 'structure', result[-1]['record'], 'film.rs::decode_registry'))
        return result

    def annotations(self, label, index, packet, data):
        offset, payload, size = packet['offset'], packet['payload'], packet['size']
        base = payload * 8
        source = '16-byte packet header · little-endian'
        fields = [field(offset * 8, offset * 8 + 16, 'Packet kind', packet['kind'], source=source),
                  field(offset * 8 + 16, offset * 8 + 32, 'Unknown header word', packet['unknown'], 'opaque', source=source),
                  field(offset * 8 + 32, offset * 8 + 64, 'Payload length (bytes)', size, source=source),
                  field(offset * 8 + 64, offset * 8 + 128, 'Timestamp (µs)', str(packet['timestamp']), source=source)]
        issues = []
        if packet['kind'] != 0:
            return fields, issues
        bits = bit_string(data[payload:payload + size])
        clocks = CLOCKS if label == 'ranked-arena/02-oddball' else (CLOCK,)
        if bits.startswith(clocks) and len(bits) >= 37:
            fields += [field(base, base + 29, 'Clock signature', bits[:29], 'structure', 'Frame clock', 'Checked 37-bit clock record'),
                       field(base + 29, base + 37, 'Frame counter', int(bits[29:37], 2), record='Frame clock', source='Checked 37-bit clock record')]
        evidence = self.evidence(label, index).get(payload, {})
        for kind, rows in evidence.items():
            for row in rows:
                try:
                    more = self.record_fields(label, kind, row, bits, base)
                    fields.extend(more)
                except (ValueError, IndexError, KeyError) as error:
                    issues.append(f'{kind}: {error}; annotation withheld')
        return fields, issues

    def record_fields(self, label, kind, row, bits, base):
        if kind == 'zoom':
            offset = row['bit']
            decoded = zoom_fields(bits, offset)
            if decoded is None or any(decoded[k] != row[k] for k in decoded):
                raise ValueError('Scope stage / following boundary mismatch')
            record = f"Scope stage · roster {row['player']} · life {row['serial']}"
            def f(a, n, title, value, status='decoded', note=''):
                return field(base + a, base + a + n, title, value, status, record, 'film_zoom_probe.py', note)
            clock = decoded['clock_bit']
            annotations = [f(offset + 1, 10, 'Scope signature', '1001010110', 'structure'),
                           f(offset + 11, 8, 'Wire identity', row['wire']),
                           f(offset + 19, 2, 'Generation tag', row['generation']),
                           f(offset + 21, 2, 'Scope guard', '00', 'structure'),
                           f(offset + 23, 2, 'Zoom stage', row['level'], note='0 unscoped; 1 first zoom; 2 second zoom. Not a magnification multiplier. Last bit overlaps the following leading bit.'),
                           f(clock + 3, 26, 'Following clock signature', bits[clock + 3:clock + 29], 'structure'),
                           f(clock + 29, 8, 'Clock counter', int(bits[clock + 29:clock + 37], 2))]
            if decoded['auxiliary_bit'] is not None:
                annotations.append(f(decoded['auxiliary_bit'] + 1, 63, 'Auxiliary record', None, 'opaque', 'Checked signature and length; payload meaning remains unknown.'))
            for following in decoded['following_zoom_bits']:
                annotations.append(f(following + 1, 24, 'Following scope window', None, 'structure', 'Boundary guard; separate evidence supplies any player/value annotation.'))
            return annotations
        if kind == 'reload':
            offset = row['bit']
            decoded = reload_fields(bits, offset)
            if decoded is None or any(decoded[k] != row[k] for k in decoded):
                raise ValueError('Reload start / following boundary mismatch')
            record = f"Reload start · roster {row['player']} · life {row['serial']}"
            def f(a, n, title, value, status='decoded', note=''):
                return field(base + a, base + a + n, title, value, status, record, 'film_weapon_probe.py', note)
            clock = decoded['clock_bit']
            annotations = [f(offset + 1, 9, 'Reload event signature', '101001101', 'structure'),
                           f(offset + 10, 8, 'Wire identity', row['wire']),
                           f(offset + 18, 2, 'Generation tag', row['generation']),
                           f(offset + 20, 7, 'Reload start guard', '0001000', 'structure', 'Same form for manual and automatic reloads; cause is not decoded.'),
                           f(offset + 27, 5, 'Roster index', row['player']),
                           f(clock + 3, 26, 'Following clock signature', bits[clock + 3:clock + 29], 'structure'),
                           f(clock + 29, 8, 'Clock counter', int(bits[clock + 29:clock + 37], 2))]
            if decoded['auxiliary']:
                annotations.append(f(offset + 32, 63, 'Auxiliary record', None, 'opaque', 'Checked signature and boundary; payload meaning unknown. Leading bit overlaps roster.'))
            return annotations
        if kind == 'weapon':
            offset = row['bit']
            decoded = weapon_delta(bits, offset, label in RANKED, row['clock'])
            if (decoded is None or any(decoded[k] != row[k] for k in decoded)
                    or int(bits[offset:offset + 14], 2) != 8704 + row['wire']
                    or int(bits[offset + 14:offset + 16], 2) != row['generation']):
                raise ValueError('Weapon sparse delta mismatch')
            record = f"Weapon state · roster {row['player']} · life {row['serial']}"
            def f(a, n, title, value, status='decoded', note=''):
                return field(base + a, base + a + n, title, value, status, record, 'film_weapon_probe.py', note)
            ids = decoded['components']
            annotations = [f(offset, 14, 'Wire identity', row['wire']),
                           f(offset + 14, 2, 'Generation tag', row['generation']),
                           f(offset + 16, 2, 'Sparse mode', '00', 'structure'),
                           f(offset + 18, 3, 'Component count', len(ids))]
            for i, component in enumerate(ids):
                annotations.append(f(offset + 21 + 6 * i, 6, f'Component index {i}', component))
            first = offset + 21 + 6 * len(ids)
            if decoded['fields'][0]['bit'] > first:
                annotations.append(f(first, decoded['fields'][0]['bit'] - first, 'Checked preceding components', None, 'opaque', 'Lengths/guards checked; other annotations may decode their values.'))
            for item in decoded['fields']:
                start, width = item['bit'], item['end'] - item['bit']
                if item['kind'] == 'switch':
                    annotations += [f(start, 3, 'Selected weapon slot', item['slot'], note='Zero-based slot. Only the captured 0010011 selection form is supported.'),
                                    f(start + 3, 4, 'Selection guard', '0011', 'structure')]
                elif width == 2:
                    annotations.append(f(start, 2, 'Magazine rounds', 0, note=f"Slot {item['slot']}: two-bit zero form 11; reserve ammo is unknown."))
                else:
                    annotations += [f(start, 1, 'Magazine prefix', '0', 'structure'),
                                    f(start + 1, 8, 'Magazine rounds', item['value'], note=f"Slot {item['slot']}; controlled values 1–15. Reserve ammo is unknown."),
                                    f(start + 9, 1, 'Magazine suffix', '1', 'structure')]
            annotations.append(f(decoded['end'], 16, 'End / input guard', bits[decoded['end']:decoded['end'] + 16], 'structure'))
            return annotations
        if kind == 'grenade':
            offset = row['bit']
            decoded = grenade_fields(bits, offset)
            if decoded is None or any(decoded[k] != row[k] for k in decoded):
                raise ValueError('Grenade throw / following clock mismatch')
            record = f"Grenade throw · roster {row['player']} · life {row['serial']}"
            def f(a, n, title, value, status='decoded'):
                return field(base + offset + a, base + offset + a + n, title, value, status, record, 'film_grenade_probe.py')
            return [f(1, 9, 'Grenade throw signature', '101001111', 'structure'),
                    f(10, 8, 'Wire identity', row['wire']), f(18, 2, 'Generation tag', row['generation']),
                    f(20, 7, 'Throw guard', '0000010', 'structure'), f(27, 5, 'Roster index', row['player']),
                    f(32, 28, 'Following clock signature', bits[offset + 32:offset + 60], 'structure'),
                    f(60, 8, 'Clock counter', int(bits[offset + 60:offset + 68], 2))]
        if kind == 'projectile':
            offset = row['bit']
            record, source = 'Controlled grenade projectile · ' + row['kind'], 'film_grenade_probe.py'
            def f(a, n, title, value, status='decoded'):
                return field(base + a, base + a + n, title, value, status, record, source)
            if row['kind'] == 'terminal':
                if bits[offset:offset + len(PROJECTILE_END)] != PROJECTILE_END:
                    raise ValueError('Projectile terminal event mismatch')
                return [f(offset, len(PROJECTILE_END), 'Projectile terminal signature', PROJECTILE_END, 'structure')]
            if row['kind'] == 'spawn':
                if bits[offset:offset + len(PROJECTILE_SPAWN)] != PROJECTILE_SPAWN:
                    raise ValueError('Controlled projectile spawn mismatch')
                coordinate = offset + len(PROJECTILE_SPAWN)
                annotations = [f(offset, len(PROJECTILE_SPAWN), 'Projectile spawn signature', None, 'opaque')]
            else:
                decoded = projectile_fields(bits, offset)
                if decoded is None or any(decoded[k] != row[k] for k in decoded):
                    raise ValueError('Projectile position mismatch')
                coordinate = decoded['coordinate_bit']
                annotations = [f(offset, coordinate - offset, 'Projectile identity / component list', decoded['components'], 'structure'),
                               f(coordinate + 49, decoded['checked_end_bit'] - coordinate - 49, 'Velocity / rotation / optional components', None, 'opaque'),
                               f(decoded['checked_end_bit'], 16, 'End / input guard', '0001000000001101', 'structure')]
            for a, n, axis, value in zip((0, 15, 30), (15, 15, 17), 'XYZ', row['xyz']):
                if int(bits[coordinate + a:coordinate + a + n], 2) != value:
                    raise ValueError('Projectile coordinate mismatch')
                annotations.append(f(coordinate + a, n, 'Grenade ' + axis + ' raw', value))
            return annotations
        if kind == 'spawn':
            return annotate_spawn(bits, row, base, label == 'ranked-arena/02-oddball')
        if kind == 'delta':
            offset, serial = int(row['bit']), int(row['serial'])
            generation = int(row.get('generation', 1))
            if not bits.startswith(CLOCKS if label == 'ranked-arena/02-oddball' else (CLOCK,)):
                raise ValueError('Clock signature mismatch')
            wire = int(row.get('wire', serial))
            if int(bits[offset:offset + 14], 2) != 8704 + wire:
                raise ValueError('Wire identity mismatch')
            fields, parsed = annotate_delta(bits, offset, base, int(bits[29:37], 2),
                                            f"analysis/{RANKED[label]}/delta_evidence.csv", row['player'], serial,
                                            generation, label in RANKED)
            if (parsed['end'] != int(row['checked_end_bit'])
                    or parsed['components'] != [int(x) for x in row['components'].split(';')]
                    or parsed['position'] != ([int(row[k]) for k in ('x', 'y', 'z')] if row['x'] else None)
                    or parsed['aim'] != ([int(row[k]) for k in ('yaw', 'pitch')] if row['yaw'] else None)):
                raise ValueError('Decoded fields disagree with evidence')
            return fields
        if kind == 'firing':
            offset = row['bit']
            decoded = firing_fields(bits, offset)
            if decoded is None or any(decoded[k] != row[k] for k in decoded):
                raise ValueError('Firing event mismatch')
            record, source = f"Firing · roster {row['player']} · life {row['serial']}", 'film_firing_probe.py'
            def f(a, n, title, value, status='decoded'):
                return field(base + offset + a, base + offset + a + n, title, value, status, record, source)
            return [f(1, 11, 'Event signature', bits[offset + 1:offset + 12], 'structure'),
                    f(12, 8, 'Wire identity', row['wire']), f(20, 2, 'Generation tag', row['generation']),
                    f(22, 4, 'Event guard', '0000', 'structure'),
                    f(26, 7, 'Sequence low bits', row['sequence'] % 128), f(33, 1, 'Sequence high bit', row['sequence'] // 128),
                    f(34, 1, 'Sequence guard', 0, 'structure'), f(35, 5, 'Roster index', row['player']),
                    f(40, 40, 'Weapon window (opaque)', row['weapon_window'], 'opaque'),
                    f(80, 28, 'Event suffix', bits[offset + 80:offset + 108], 'structure')]
        if kind == 'melee':
            offset = row['bit']
            decoded = melee_fields(bits, offset)
            if decoded is None or any(decoded[k] != row[k] for k in decoded):
                raise ValueError('Melee event / companion mismatch')
            record = f"Melee · roster {row['player']} · life {row['serial']}"
            def f(a, n, title, value, status='decoded'):
                return field(base + offset + a, base + offset + a + n, title, value, status, record, 'film_melee_probe.py')
            return [f(-30, 9, 'Companion signature', '101001101', 'structure'),
                    f(-21, 8, 'Companion wire identity', row['wire']), f(-13, 2, 'Companion generation', row['generation']),
                    f(-11, 7, 'Companion guard', '0011000', 'structure'), f(-4, 5, 'Roster index', row['player']),
                    f(1, 9, 'Melee signature', '101010001', 'structure'), f(10, 8, 'Wire identity', row['wire']),
                    f(18, 2, 'Generation tag', row['generation']), f(20, 6, 'Event guard', '000010', 'structure'),
                    f(26, 40, 'Weapon window (opaque)', row['weapon_window'], 'opaque'),
                    f(66, 28, 'Event suffix', bits[offset + 66:offset + 94], 'structure')]
        if kind == 'vitality':
            offset = int(row['bit'])
            parsed = isolated_vitality(bits, offset)
            if parsed is None or parsed['serial'] != int(row['serial']):
                raise ValueError('Isolated vitality mismatch')
            fields = [field(base + offset, base + offset + 14, 'Pawn identity', parsed['serial'], record=row['name'] + ' · vitality', source='film_vitality_probe.py')]
            fields.append(field(base + offset + 14, base + offset + 21 + len(parsed['components']) * 6,
                                'Sparse list', parsed['components'], 'structure', row['name'] + ' · vitality', 'film_vitality_probe.py'))
            for component in parsed['components']:
                key = 'body' if component == 4 else 'shield'
                if parsed[key + '_raw'] != int(row[key + '_raw']):
                    raise ValueError('Vitality amount mismatch')
                fields.extend(annotate_vitality(base + parsed[key + '_bit'], component, parsed, row['name'] + ' · vitality', 'film_vitality_probe.py'))
            return fields
        if kind == 'octagon_spawn':
            if bits[18:69] != SPAWN_BODY or bits[192:255] != COORD_PREFIX or bits[310:340] != COORD_SUFFIX:
                raise ValueError('Octagon spawn guard mismatch')
            result = [field(base + 18, base + 69, 'Spawn signature', SPAWN_BODY, 'structure', row['player'] + ' · spawn', 'build_octagon_scene.py'),
                      field(base + 69, base + 74, 'Roster index', int(bits[69:74], 2), record=row['player'] + ' · spawn', source='build_octagon_scene.py'),
                      field(base + 192, base + 255, 'Coordinate prefix', COORD_PREFIX, 'structure', row['player'] + ' · spawn', 'build_octagon_scene.py')]
            for a, n, axis, value in zip((263, 278, 293), (15, 15, 17), 'XYZ', row['xyz']):
                if int(bits[a:a + n], 2) != value:
                    raise ValueError('Octagon coordinates mismatch')
                result.append(field(base + a, base + a + n, axis + ' raw', value, record=row['player'] + ' · spawn', source='build_octagon_scene.py'))
            result.append(field(base + 310, base + 340, 'Coordinate suffix', COORD_SUFFIX, 'structure', row['player'] + ' · spawn', 'build_octagon_scene.py'))
            return result
        return self.controlled_fields(kind, row, bits, base)

    def controlled_fields(self, kind, row, bits, base):
        result = []
        record, source = 'Controlled player · ' + kind, 'film_motion_probe.rs · ' + kind + '.csv'
        def f(a, n, title, value, status='decoded'):
            result.append(field(base + a, base + a + n, title, value, status, record, source))
        def check(a, n, value):
            if len(bits[a:a + n]) != n or int(bits[a:a + n], 2) != int(value):
                raise ValueError('Export / byte mismatch')
        if kind == 'inputs':
            a = int(row['input_bit_offset'])
            if bits[a - 3:a + 13] != '0001000000001101':
                raise ValueError('Input boundary mismatch')
            f(a - 3, 3, 'Entity chain End', '000', 'structure')
            f(a, 13, 'Input prefix', bits[a:a + 13], 'structure')
            for o, key in ((13, 'forward_raw'), (19, 'left_raw')):
                check(a + o, 6, row[key]); f(a + o, 6, key.replace('_', ' '), int(row[key]))
            return result
        a = int(row.get('record_bit_offset', 37))
        if not bits.startswith(CLOCK) or bits[a:a + 14] != '10001000000000':
            raise ValueError('Controlled clock / player signature mismatch')
        check(29, 8, row['clock'])
        f(a, 14, 'Pawn identity', 0)
        if kind == 'aim':
            if bits[a + 14:a + 34] != '01000100101010110011' or bits[a + 57:a + 59] != '01':
                raise ValueError('Aim guard mismatch')
            f(a + 14, 20, 'Aim signature', bits[a + 14:a + 34], 'structure')
            for o, n, k in ((34, 12, 'yaw_raw'), (46, 11, 'pitch_raw')):
                check(a + o, n, row[k]); f(a + o, n, k.replace('_', ' ').title(), int(row[k]))
            f(a + 57, 2, 'Aim suffix', '01', 'structure')
            counter = a + 59
        elif kind == 'positions':
            short = row['layout'] == 'A'
            prefix = '010001000000001100100000' if short else '010001100000000000101100100000'
            if bits[a + 14:a + 14 + len(prefix)] != prefix:
                raise ValueError('Position signature mismatch')
            coordinates = a + (38 if short else 44)
            counter = a + {'A': 88, 'B': 125, 'C': 96}[row['layout']]
            f(a + 14, len(prefix), 'Position signature', prefix, 'structure')
            for o, n, k in ((0, 15, 'x_raw'), (15, 15, 'y_raw'), (30, 17, 'z_raw')):
                check(coordinates + o, n, row[k]); f(coordinates + o, n, k.replace('_', ' ').upper(), int(row[k]))
            tail = coordinates + 47
            if counter > tail:
                f(tail, counter - tail, 'Position continuation', None, 'opaque')
        else:
            return []
        check(counter, 8, row['clock']); check(counter + 8, 1, 0)
        f(counter, 8, 'Repeated clock counter', int(row['clock']))
        f(counter + 8, 1, 'Counter suffix', '0', 'structure')
        return result

    def view(self, label, index, offset, count=256):
        chunk, data, packets = self.chunk(label, index)
        if not 0 <= offset < len(data) or not 1 <= count <= 512:
            raise ValueError('Invalid byte window')
        end = min(offset + count, len(data))
        packet = None
        if packets:
            i = bisect_right([r['offset'] for r in packets], offset) - 1
            packet = packets[i]
            scope_start, scope_end = packet['offset'] * 8, (packet['payload'] + packet['size']) * 8
            fields, issues = self.annotations(label, index, packet, data)
            # A byte page ends at its packet boundary, so every visible byte has
            # the same scope as the decoded pane and coverage denominator.
            end = min(end, scope_end // 8)
        else:
            scope_start, scope_end = offset * 8, end * 8
            fields = self.registry(data) if chunk['chunk_type'] == 1 else []
            issues = []
        spans = partition(scope_start, scope_end, fields)
        return {'film': label, 'chunk': index, 'offset': offset, 'end': end, 'bytes': list(data[offset:end]),
                'packet': packet, 'scope': [scope_start, scope_end], 'spans': spans,
                'coverage': coverage(spans), 'issues': issues, 'chunk_size': len(data)}
