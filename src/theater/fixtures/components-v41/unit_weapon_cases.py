"""Author shared unit/weapon layouts independently of the local Rust parser."""


def append_cases(case, bits, handle):
    def opt(width, present, value=0, inverted=False):
        return bits(1, not present if inverted else present) + (bits(width, value) if present else '')

    def ref(present, value=8191, generation=0, units=None):
        return bits(1, present) + ((('' if units is None else bits(1, units)) +
               handle(9 if units else 13, value, generation)) if present else '')

    def point(snapshot, form):
        if snapshot:
            # Exact float words: zero, negative zero, a NaN payload.
            return ''.join(bits(32, v) for v in [0, 0x80, 0x4523C17F])
        if form == 'default':
            return '1' + ''.join(bits(22, v) for v in [0, (1 << 22) - 1, 0x123456])
        return '00' + bits(15, 0) + bits(15, 32767) + bits(17, 0x13579)

    for snapshot in [False, True]:
        def add(label, component, wire, version=4):
            case(f'unit-weapon/{snapshot}/{label}', component, wire,
                 version=version, snapshot=snapshot)

        fixed = {
            'equipment-being-hacked-component': [8],
            'equipment-charges-remaining-component': [8],
            'equipment-deployed-component': [1],
            'equipment-energy-component': [14],
            'equipment-energy-delay-ticks-left-component': [10],
            'equipment-has-infinite-uses-component': [1],
            'unit-stun-component': [16, 12, 12],
            'unit-crouch-component': [1, 10],
            'weapon-ammo-component': [8, 11, 12],
            'weapon-state-rounds-inventory': [11],
            'weapon-state-overheated': [7, 2],
        }
        for name, widths in fixed.items():
            for filled in [False, True]:
                add(f'fixed/{name}/{filled}', name,
                    ''.join(bits(w, (1 << w) - 1 if filled else 0) for w in widths))
        for present in [False, True]:
            add(f'creator/{present}', 'equipment-creator-component', opt(5, present, 31, True))

        for count in range(8):
            add(f'grenades/{count}', 'unit-grenade-counts-component',
                bits(3, count) + ''.join(bits(8, (i * 51) % 256) for i in range(count)))
            for mode in range(3):
                entries = ''.join(ref(mode == 1 or mode == 2 and i % 2 == 0,
                                     [0, 8191, 0x1357][i % 3], i % 4) for i in range(count))
                add(f'equipment/{count}/{mode}', 'unit-equipment-component',
                    bits(3, (7 - count)) + bits(3, count) + entries)
        for count in range(16):
            for mode in range(3):
                entries = ''.join(ref(mode == 1 or mode == 2 and i % 2 == 0,
                                     [0, 8191, 0x1357][i % 3], i % 4) for i in range(count))
                add(f'low-frequency/{count}/{mode}', 'unit-low-frequency-component',
                    '1' + ref(mode != 0, 0, count % 4) + bits(3, 7) + '0' + bits(4, count) + entries + '11')
        for version in [0, 1, 2, 3, 4, 5]:
            # None/all/alternating and each single optional property.
            for mask in [0, 0xFFF, 0xAAA] + [1 << i for i in range(12)]:
                fields = [opt(12, bool(mask & (1 << i)), (i * 313) % 4096) for i in range(12)]
                wire = fields[0] + ('1' if version > 2 else '') + ''.join(fields[1:8])
                wire += bits(4 + (version > 2) + (version > 3), 0xA) + ''.join(fields[8:])
                add(f'malleable/{version}/{mask}', 'unit-malleable-property-component', wire, version)
        for state in range(8):
            for head in ['1', '01', '00' + bits(12, 0xDBA)]:
                for mask in [0, 63, 21]:
                    wire = bits(3, state) + head + ''.join(opt(12, bool(mask & (1 << i)), i * 819) for i in range(6))
                    add(f'camo/{state}/{head}/{mask}', 'unit-active-camo-state-component', wire)
        for head in [False, True]:
            for second in ([False, True] if head else [False]):
                for tail in [False, True]:
                    wire = bits(1, head) + ((bits(5, 31) + opt(6, second, 63)) if head else '')
                    add(f'control/{head}/{second}/{tail}', 'unit-control-component', wire + opt(32, tail, 0xDEADBEEF))
        for two in [False, True]:
            add(f'aim/{two}', 'unit-desired-aiming-vector-component',
                bits(1, not two) + (bits(12, 0xABC) + bits(11, 0x456) + '1') * (2 if two else 1))
        for magazine in [False, True]:
            for fraction in [False, True]:
                add(f'ammo/{magazine}/{fraction}', 'weapon-state-ammo',
                    opt(8, magazine, 255, True) + opt(12, fraction, 4095, True))

        for filled in [False, True]:
            tail = opt(8, filled, 255) + bits(3, 7) + opt(2, filled, 3, True) + opt(2, not filled, 0, True)
            add(f'type-info/absent/{filled}', 'weapon-state-type-info', '0' + tail)
            for magazines in [-1, 0, 1, 15]:
                for elements in [0, 1, 64]:
                    wire = '1' + bits(32, 0x11223344) + bits(32, 0x55667788)
                    wire += bits(12, 4095) + bits(7, 127) + opt(10, filled, 1023) + '1'
                    if magazines < 0:
                        wire += '0' + opt(32, filled, 0xDEADBEEF)
                    else:
                        wire += '1' + bits(4, magazines)
                        wire += ''.join(opt(32, bool(i % 2) or filled, i * 0x1010101) for i in range(magazines))
                    wire += opt(32, elements != 0, 0xFEDCBA98)
                    if elements:
                        wire += bits(6, elements - 1) + ''.join(bits(1, i % 2) for i in range(elements))
                    for axis in range(3):
                        wire += opt(6, filled, 63, True) + opt(12, filled, 4095) + opt(12, not filled, axis * 1023)
                    wire += opt(5, filled, 31, True) + tail
                    add(f'type-info/{filled}/{magazines}/{elements}', 'weapon-state-type-info', wire)

        # Actor-state's >=4 width agrees with libfilmshell. Lower registry
        # versions use LevelUp's separate 8/10/11-bit threshold table.
        for form in (['raw'] if snapshot else ['default', 'indexed']):
            for slot in range(5):
                for object_ in [False, True]:
                    for full in [False, True]:
                        for present in [False, True]:
                            body = bits(1, object_) + bits(1, full) + bits(32, 0x01234567)
                            if object_:
                                body += ref(present, 8191, slot % 4)
                            else:
                                if full:
                                    body += '1' + bits(2, 2) + bits(10, 1023) + bits(10, 0)
                                body += point(snapshot, form)
                                if full:
                                    body += opt(32, present, 0x76543210) + bits(32, 0xFEEDBEEF) + ref(present, 0, slot % 4)
                            if not full:
                                body += bits(8, 0xA5)
                            body += opt(32, present, 0x89ABCDEF)
                            wire = bits(32, 0) + bits(32, 0xFFFFFFFF) + bits(12, 0xABC) + bits(4, 9)
                            wire += '0' * slot + '1' + body + '0' * (4 - slot)
                            add(f'actor-state/{form}/{slot}/{object_}/{full}/{present}', 'unit-actor-state-component', wire)
        add('actor-state/empty', 'unit-actor-state-component', bits(64) + bits(12) + bits(4) + '00000')

        # Actor-control: all selector exits, optional word/direction/fraction,
        # one/two targets and both category-1 index forms. Action body absent.
        for version in [0, 1, 2, 4]:
            for selector in range(8):
                for filled in [False, True]:
                    wire = bits(3, selector)
                    if selector != 1:
                        wire += opt(32, filled, 0xDEADBEEF) + '1' + bits(6, 63)
                        wire += ('0' + bits(19, 0x45678) + bits(10, 1023)) if filled else '1'
                        wire += bits(19, 0x54321) + '1' + opt(9, filled, 511, True) + '0'
                        wire += ref(filled, 511 if filled else 0, 0, True)
                        if version > 1:
                            wire += ref(filled, 8191, 3, False)
                    add(f'actor-control/{version}/{selector}/{filled}', 'unit-actor-control-component', wire, version)

        for form in (['raw'] if snapshot else ['default', 'indexed']):
            for mode in range(4):
                for hands in [0, 1, 3]:
                    for filled in [False, True]:
                        action = '1' + bits(3, hands & 1) + bits(3, (hands >> 1) & 1) + '0'
                        action += '1' + '10' + opt(8, filled, 255) + opt(10, filled, 1023) + bits(2, mode)
                        if mode == 0:
                            action += point(snapshot, form)
                        elif mode == 1:
                            action += bits(19, 0x54321)
                        action += bits(3, 7)
                        for hand in range(2):
                            if hands & (1 << hand):
                                action += opt(2, filled, 3, True)
                        action += '0'  # no firing target
                        header = '000' + '0' + '1' + bits(6, 63) + '1' + bits(19, 0x54321) + '1' + '1'
                        add(f'actor-action/{form}/{mode}/{hands}/{filled}', 'unit-actor-control-component',
                            header + '1' + action + '00', version=2)
