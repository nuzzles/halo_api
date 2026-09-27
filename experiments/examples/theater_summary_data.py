"""Inspect the upstream summary decoder's exact source fields, not legacy exports."""

import json
import subprocess


def decode_summary(folder, experiments):
    """Cargo checks source freshness; --stdout never writes film/export files."""
    try:
        result = subprocess.run([
            'cargo', 'run', '--quiet', '--release', '--offline',
            '--manifest-path', str(experiments / 'Cargo.toml'),
            '--example', 'decode_film_events', '--', '--stdout', str(folder),
        ], capture_output=True, text=True, timeout=120, check=True)
    except (OSError, subprocess.SubprocessError) as error:
        detail = getattr(error, 'stderr', None) or str(error)
        raise ValueError('Native summary decoder unavailable: ' + str(detail)[-1200:]) from error
    return json.loads(result.stdout)


def annotate_summary_event(event, data, packet, chunk, field):
    """Revalidate native values against raw bytes before emitting bit coverage.

    Only small unaligned windows are read; even the raid footer is never expanded
    into a giant bit string. The native decoder owns discovery, guards and names.
    """
    source, identity = event['source'], event['identity_source']
    base = packet['payload'] * 8
    limit = (packet['payload'] + packet['size']) * 8
    for span, width in ((source, 480), (identity, 64)):
        if (span['chunk'] != chunk or span['payload_byte'] != packet['payload']
                or span['end_bit'] - span['bit'] != width
                or not base + 32 <= base + span['bit'] < base + span['end_bit'] <= limit):
            raise ValueError('Summary source is outside its packet')
    tail, xuid_start = base + source['bit'], base + identity['bit']

    def raw(start, size):
        end = start + size * 8
        if start < base or end > limit:
            raise ValueError('Truncated summary field')
        window = data[start // 8:(end + 7) // 8]
        value = int.from_bytes(window, 'big') >> ((-end) % 8)
        return (value & ((1 << (size * 8)) - 1)).to_bytes(size, 'big')

    buf = raw(tail, 60)
    name = event['name'].encode('utf-16-le')
    if (not name or len(name) > 32 or buf[:32] != name.ljust(32, b'\0')
            or int.from_bytes(raw(xuid_start, 8), 'little') != int(event['xuid'])
            or buf[47] != event['type_code'] or buf[55] != event['medal_flag']
            or buf[59] != event['metadata']
            or int.from_bytes(buf[48:52], 'big') * 1000 != event['time_us']
            or buf[52:55] != b'\0' * 3 or buf[56:59] != b'\0' * 3
            or raw(tail + 480, 4) != b'\0\0\x2e\xe0'
            or raw(xuid_start + 64, 2) not in (b'\x2d\xc0', b'\x25\xc0')):
        raise ValueError('Native summary export / byte mismatch')
    medal = event.get('medal')
    if medal and (medal['film_id'] != buf[59] or buf[55] != 1):
        raise ValueError('Medal identity / byte mismatch')
    title = (medal.get('name') or f"Unknown medal {buf[59]}") if medal else event['kind']
    record = f"{event['name']} · {event['time_us'] / 1e6:.3f} s · {title}"
    provenance = 'halo_api::theater::decode_summary_events (current native decoder)'
    result = []

    def add(start, size, label, value=None, status='decoded', note=''):
        result.append(field(start, start + size, label, value, status, record, provenance, note))

    add(xuid_start, 64, 'Player XUID', event['xuid'], note='Little-endian u64; preserved as decimal text.')
    add(xuid_start + 64, 16, 'Identity marker', raw(xuid_start + 64, 2).hex(' '), 'structure')
    # The captured gap has an exact boundary, but its state is not semantically decoded.
    if xuid_start + 80 < tail:
        add(xuid_start + 80, tail - xuid_start - 80, 'Intervening event state', status='opaque',
            note='Bounded by native identity/tail spans; contents remain unknown.')
    add(tail, len(name) * 8, 'Gamertag', event['name'], note='UTF-16LE')
    if len(name) < 32:
        add(tail + len(name) * 8, (32 - len(name)) * 8, 'Gamertag padding', 'NUL', 'structure')
    add(tail + 32 * 8, 15 * 8, 'Opaque event fields', status='opaque')
    add(tail + 47 * 8, 8, 'Medal sorting weight' if medal else 'Event type',
        buf[47] if medal else f"{event['kind']} ({buf[47]})",
        'opaque' if isinstance(event['kind'], dict) else 'decoded')
    add(tail + 48 * 8, 32, 'Event time (ms)', event['time_us'] // 1000, note='Big-endian u32; recorded summary timeline.')
    add(tail + 52 * 8, 24, 'Reserved field', 0, 'structure')
    add(tail + 55 * 8, 8, 'Medal flag', buf[55])
    add(tail + 56 * 8, 24, 'Reserved field', 0, 'structure')
    if medal:
        add(tail + 59 * 8, 8, 'Medal', f"{title} · film code {buf[59]}",
            note=f"Stats API NameId: {medal.get('name_id')}. NameId is a catalog mapping, not additional decoded bits.")
    else:
        add(tail + 59 * 8, 8, 'Event metadata', buf[59], 'opaque', 'Raw byte retained; objective subtypes are unresolved.')
    add(tail + 480, 32, 'Event end marker', '0x00002ee0', 'structure')
    return result
