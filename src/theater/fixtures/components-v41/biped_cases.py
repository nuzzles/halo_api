"""Biped envelopes from pinned readers; predicted tag 3 has separate source traces."""
import json
from pathlib import Path


def append_cases(case, bits, handle):
    predicted = []
    for snapshot in [False, True]:
        for version in [0, 1, 2, 4]:
            def add(label, name, wire):
                return case(f'biped/{snapshot}/{version}/{label}', name, wire,
                            version=version, snapshot=snapshot)

            def point(form):
                if snapshot:
                    return ''.join(bits(32, v) for v in [0, 0x80, 0x4523C17F])
                if form == 'default':
                    return '1' + bits(22, 0) + bits(22, 4194303) + bits(22, 0x123456)
                return '00' + bits(15, 0) + bits(15, 32767) + bits(17, 0x13579)

            for tag in [0, 1, 2]:
                for sub in (range(4) if tag == 1 else [0]):
                    add(f'predicted/{tag}/{sub}', 'biped-spartan-ability-component',
                        bits(2, tag) + (bits(2, sub) + bits(24, 0xABCDEF) if tag == 1 else ''))
            for value in [None, 0, 1, 16, 17, 62, 63]:
                for form in (['absent', 'raw'] if snapshot else ['absent', 'default', 'indexed']):
                    wire = '11' + bits(1, value is not None)
                    if value is not None:
                        wire += bits(6, value)
                    wire += bits(1, form != 'absent')
                    if form != 'absent':
                        wire += point(form)
                    padded = wire.ljust((len(wire) + 7) // 8 * 8, '0')
                    predicted.append(dict(name=f'predicted-source/{snapshot}/{version}/{value}/{form}',
                                          component='biped-spartan-ability-component', version=version,
                                          snapshot=snapshot, complete=True, bit_len=len(wire),
                                          hex=int(padded, 2).to_bytes(len(padded) // 8, 'big').hex()))
                    # Upstream differs on earlier versions and flag bit 0.
                    if version > 1 and (value is None or value & 1 == 0):
                        add(f'predicted/shared/{value}/{form}', 'biped-spartan-ability-component', wire)

            for flag in [False, True]:
                add(f'mobility/inactive/{flag}', 'biped-mobility-action-component', '0' + bits(1, flag))
            forms = ['raw'] if snapshot else ['default', 'indexed']
            for form in forms:
                for keep in [False, True]:
                    for generation in range(4):
                        for present in [False, True]:
                            wire = '11' + bits(1, present)
                            if present:
                                wire += handle(13, [0, 8191, 0x1357, 1][generation], generation)
                            wire += bits(1, present) + (bits(10, 0x2A5) if present else '') + bits(1, keep)
                            if not keep:
                                wire += point(form) + bits(1, not present)
                                wire += bits(19, 0x51357) if present else ''
                                wire += bits(8, 0xA5)
                            wire += ''.join(bits(32, v) for v in [0, 0x80, 0x4523C17F]) + point(form)
                            wire += ''.join(bits(12, i * 317) for i in range(9))
                            wire += bits(24, 0x123456) + bits(24, 0xFEDCBA)
                            wire += bits(12, 0xA35) * 3 + bits(10, 0xA5) + bits(10, 0x35A)
                            wire += '1' + bits(7, 0x53) + '10' + '0'
                            add(f'mobility/{form}/{keep}/{generation}/{present}', 'biped-mobility-action-component', wire)

                for sub in range(4):
                    wire = '01' + bits(2, sub)
                    if sub:
                        wire += bits(15, 0x5135) + bits(2, 2) + bits(2, 3) + point(form) + bits(19, 0x53157)
                    add(f'posture/1/{form}/{sub}', 'biped-posture-physics-component', wire)
                for mode in ['word', 'absent', 'handle']:
                    wire = '10' + point(form) + bits(19, 0x13579) + bits(1, mode != 'word')
                    wire += bits(32, 0x12345678) if mode == 'word' else bits(1, mode == 'handle')
                    if mode == 'handle':
                        wire += handle(13, 8191, 0)
                    wire += bits(15, 0x2531)
                    add(f'posture/2/{form}/{mode}', 'biped-posture-physics-component', wire)
                for present in [False, True]:
                    wire = '11' + bits(15, 0x5135) + bits(1, present)
                    if present:
                        wire += point(form) + bits(19, 0x53157)
                    wire += bits(32, 0x12345678) + '10'
                    add(f'posture/3/{form}/{present}', 'biped-posture-physics-component', wire)

            add('posture/0/absent', 'biped-posture-physics-component', '001')
            for sub in range(4):
                for present in ([False, True] if sub >= 2 else [False]):
                    wire = '000' + bits(2, sub)
                    if sub == 1:
                        wire += bits(15, 0x2135) + '10' + bits(2, 3)
                    elif sub >= 2:
                        wire += bits(15, 0x5135) + bits(2, 2) + '1' + bits(32, 0x12345678)
                        wire += bits(1, present) + (bits(19, 0x13579) if present else '')
                    add(f'posture/0/{sub}/{present}', 'biped-posture-physics-component', wire)

            for pattern in range(3):
                def word(width, offset=0):
                    return bits(width, ([0, (1 << width) - 1, ((1 << width) - 1) // 3][pattern] + offset) % (1 << width))

                def opt(width, index=0, inverted=False):
                    yes = pattern == 1 or pattern == 2 and index % 2 == 0
                    return bits(1, not yes if inverted else yes) + (word(width, index) if yes else '')

                for name, widths in {
                    'biped-control-context-component': [2, 1],
                    'biped-map-editor-flag-component': [8],
                    'biped-emp-timer-component': [8],
                    'vehicle-emp-timer-component': [8],
                    'biped-desired-grenade-set-component': [6, 3],
                }.items():
                    add(f'{name}/{pattern}', name, ''.join(word(w, i) for i, w in enumerate(widths)))
                add(f'weapon-set/{pattern}', 'biped-desired-weapon-set', word(3) + opt(2, 0, True) + opt(2, 1, True))
                add(f'ability-set/{pattern}', 'biped-desired-ability-set-component', word(3) + opt(6, inverted=True))
                add(f'low-frequency/{pattern}', 'biped-low-frequency-data-component', word(3) + opt(64))
                add(f'ability-malleable/{pattern}', 'biped-spartan-ability-malleable-property-component',
                    opt(12) + opt(8, 1) + ''.join(opt(12, i) for i in range(2, 5)) + word(2))
                for count in range(32):
                    wire = opt(8) + ''.join(opt(12, i) for i in range(11))
                    wire += word(8 + (version > 1)) + bits(5, count)
                    wire += word(count) if count else ''
                    add(f'malleable/{pattern}/{count}', 'biped-malleable-property-component', wire)
                for mask in range(8):
                    add(f'energy/{pattern}/{mask}', 'biped-spartan-ability-energy-component',
                        bits(3, mask) + ''.join(word(7, i) for i in range(3) if mask & (1 << i)))
                add(f'slide/inactive/{pattern}', 'biped-slide-component', '0')
                for zero in [False, True]:
                    wire = '1' + bits(1, zero) + ('' if zero else word(19) + word(10))
                    wire += word(8) + (word(8, 1) if version else '') + word(8, 2)
                    add(f'slide/{pattern}/{zero}', 'biped-slide-component', wire)
    assert len({c['name'] for c in predicted}) == len(predicted)
    Path(__file__).with_name('predicted-ability-source.json').write_text(json.dumps(predicted, indent=2) + '\n')
