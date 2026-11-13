"""Tacmap layouts from pinned readers and instruction-addressed source analysis.

The three source-only aliases are absent from libfilmshell's dispatch. Their
point bytes reference independently traced tether cases; snapshots additionally
execute the unchanged original LevelUp enclosing readers.
"""
import json
from pathlib import Path


def append_cases(case, bits):
    source_cases = []
    for snapshot in [False, True]:
        def add(label, component, wire, version=4):
            case(f'tacmap/{snapshot}/{label}', component, wire,
                 version=version, snapshot=snapshot)

        for pattern in range(3):
            def word(width, offset=0):
                return bits(width, [0, (1 << width) - 1, ((1 << width) - 1) // 3 + offset][pattern])

            for form in (['raw'] if snapshot else ['default', 'indexed']):
                label = f'{form}/{pattern}'
                if snapshot:
                    point = ''.join(bits(32, v) for v in [0, 0x80, 0x4523C17F]) if pattern == 2 else word(32) * 3
                elif form == 'default':
                    point = '1' + word(22) * 3
                else:
                    point = '00' + word(15) * 2 + word(17)
                add(f'tether/{label}', 'tacmap-cooptetherarea', point + word(12) * 2)
                for version in [0, 1, 2, 4, 7]:
                    add(f'waypoint/{version}/{label}', 'tacmap-waypointstate',
                        word(1) + word(32) + point + (word(1) if version > 1 else ''), version)

                # Fixed enclosing fields come from the instruction-addressed
                # writers in lecteur_position_exceptions.go, without its legacy
                # delta compatibility readers. Tail words distinguish boundaries.
                bodies = {
                    'tacmap-displayasset': (word(32) * 2 + word(2),
                                           word(64) + word(32) + word(64) + word(32) + word(1)),
                    'tacmap-areaofinterest': (word(32) + word(3), word(12)),
                    'tacmap-poiicon': (word(32) * 2 + word(1) + word(3) + word(32) * 2 + word(9) * 2,
                                      word(32) + word(1) + word(8) * 4),
                }
                for component, (head, tail) in bodies.items():
                    for version in [0, 1, 4]:
                        wire = head + point + tail
                        padded = wire.ljust((len(wire) + 7) // 8 * 8, '0')
                        source_cases.append(dict(
                            name=f'tacmap-source/{snapshot}/{component}/{version}/{label}',
                            component=component, version=version, snapshot=snapshot,
                            complete=True, bit_len=len(wire),
                            hex=int(padded, 2).to_bytes(len(padded) // 8, 'big').hex(),
                            point_start=len(head), point_len=len(point),
                            point_case=f'tacmap/{snapshot}/tether/{label}',
                        ))

            for count in range(32):
                add(f'lod/{pattern}/{count}', 'tacmap-iconlodthresholds', bits(5, count) + word(12) * count)
            add(f'dungeon/{pattern}', 'tacmap-dungeonstate', word(1) + word(32) * 3)
            add(f'lights/{pattern}', 'tacmap-lockedlights', word(32) * 16)
            for queued in [False, True]:
                wire = ''
                for i in range(32):
                    present = pattern == 1 or (pattern == 2 and i % 2 == 0)
                    wire += bits(1, not present) + (bits(5, i) if present else '')
                    wire += word(32) if queued else word(1)
                    wire += bits(1, i % 2)
                add(f'players/{queued}/{pattern}',
                    'tacmap-queuedreplaymission' if queued else 'tacmap-backmenu-openoverride', wire)

    assert len({c['name'] for c in source_cases}) == len(source_cases)
    Path(__file__).with_name('tacmap-source.json').write_text(json.dumps(source_cases, indent=2) + '\n')
