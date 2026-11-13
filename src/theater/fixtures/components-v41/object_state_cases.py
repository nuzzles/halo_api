"""Object lifecycle/state envelopes, authored independently of halo_api output."""


def append_cases(case, bits, handle):
    for snapshot in [False, True]:
        def add(label, component, wire, version=4, archetype=0, complete=True):
            case(f'object-state/{snapshot}/{label}', component, wire,
                 version=version, archetype=archetype, snapshot=snapshot, complete=complete)

        for pattern in range(3):
            def word(width, offset=0):
                return bits(width, [0, (1 << width) - 1,
                                   (((1 << width) - 1) // 3 + offset) % (1 << width)][pattern])

            def present(index=0):
                return pattern == 1 or (pattern == 2 and index % 2 == 0)

            def opt(width, index=0, inverted=False):
                yes = present(index)
                return bits(1, not yes if inverted else yes) + (word(width, index) if yes else '')

            for count in range(33):
                wire = bits(6, count)
                for i in range(count):
                    yes = present(i)
                    wire += bits(1, yes) + (word(7, i) + word(16, i) if yes else '')
                add(f'damage/{pattern}/{count}', 'object-damage-sections-component', wire)
            # The engine's >32 region-count validity check is separate from
            # the two reference readers' consumption policy. Only legal counts
            # are asserted here; invalid region admission remains under audit.
            for count in [0, 1, 31, 32]:
                for values in [False, True]:
                    wire = bits(1, values) + bits(6, count)
                    wire += ''.join(word(3, i) for i in range(count))
                    if values:
                        wire += ''.join(word(10, i) for i in range(count))
                    add(f'region/{pattern}/{count}/{values}', 'object-region-state-component', wire)
            for count in range(32):
                wire = bits(5, count)
                if count:
                    wire += word(count) + word(count, 1)
                add(f'constraints/{pattern}/{count}', 'object-constraint-component', wire)
            for flags in range(32):
                wire = bits(5, flags)
                for i, bit in enumerate([4, 8, 16]):
                    if flags & bit:
                        wire += ''.join(opt(12, i + k) for k in range(3))
                        if bit != 16:
                            wire += word(1, i)
                wire += word(3)
                add(f'max-vitalities/{pattern}/{flags}', 'object-maximum-vitalities-component', wire)
            for state in range(16):
                wire = bits(4, state)
                if state != 13:
                    wire += word(32) * 3 + word(12) + word(1)
                add(f'dissolver/{pattern}/{state}', 'object-dissolver-component', wire)
            add(f'body/{pattern}', 'object-body-vitality-component', word(8) + word(3))
            add(f'physics-flags/{pattern}', 'object-physics-flags-component', word(5))
            for regeneration in [False, True]:
                for mask in (range(4) if regeneration else [0]):
                    wire = word(8) + bits(1, regeneration)
                    if regeneration:
                        for i in range(2):
                            yes = bool(mask & (1 << i))
                            wire += bits(1, yes) + (word(12, i) if yes else '')
                    wire += word(16) + word(4)
                    add(f'shield/{pattern}/{regeneration}/{mask}', 'object-shield-vitality-component', wire)
            for transition in [False, True]:
                wire = '0' + word(15) + bits(1, transition)
                if transition:
                    wire += word(15, 1) + word(12) + word(5)
                add(f'scale/{pattern}/{transition}', 'object-scale-component', wire)

            # NEW/DELTA schema and row version change the parent-state tails.
            for version in [0, 1, 2, 3, 4]:
                for archetype in [35, 40, 43]:
                    for mode in ['free', 'object', 'unit']:
                        attached = mode != 'free'
                        wire = bits(1, attached)
                        if attached:
                            width = 9 if mode == 'unit' else 13
                            wire += bits(1, mode == 'unit') + handle(width, int(word(width), 2), pattern)
                            wire += word(16) + opt(16) + word(2) + word(16) * 3
                            wire += opt(19, inverted=True) + word(8) + word(1)
                        elif version < 2:
                            wire += bits(1, present())
                            if present():
                                wire += handle(13, int(word(13), 2), pattern)
                            wire += opt(11)
                        wire += opt(6) + word(1)
                        if version <= 2 or archetype == 35:
                            wire += word(3)
                        add(f'parent/{pattern}/{version}/{archetype}/{mode}',
                            'object-parent-state-component', wire, version, archetype)

            for head in range(4):
                for count in [0, 1, 2, 31, 63]:
                    for mask in range(8):
                        wire = bits(2, head) + (word(7) + word(8) if head < 2 else '')
                        wire += word(4) + word(6) + bits(6, count)
                        for i in range(count):
                            short = present(i)
                            wire += bits(1, short) + (word(1, i) if short else word(12, i) * 2)
                        wire += word(7) + opt(4, inverted=True)
                        wire += bits(1, present())
                        if present():
                            wire += word(1) + opt(12, 1) + word(1, 1)
                        wire += bits(3, mask)
                        if mask & 3 == 3:
                            wire += word(2) + word(5)
                            if mask & 4:
                                wire += word(8) * 3
                        wire += bits(1, present(1))
                        if present(1):
                            wire += word(3) + word(32) + opt(32) + word(14)
                        add(f'low-frequency/{pattern}/{head}/{count}/{mask}',
                            'object-low-frequency-component', wire)

        for count in range(33, 64):
            add(f'damage/reject/{count}', 'object-damage-sections-component',
                bits(6, count), complete=False)
        add('scale/default', 'object-scale-component', '1')
        # Isolate every axis gate, including all set, without a Cartesian
        # product between count and every possible flag combination.
        for count in [0, 1, 2, 32, 64]:
            for mask in [0, 511, 0x155] + [1 << i for i in range(9)]:
                wire = bits(1, count != 0)
                if count:
                    wire += bits(32, 0x12345678) + bits(6, count - 1)
                    wire += ''.join(bits(1, i % 2) for i in range(count))
                for i in range(9):
                    yes = bool(mask & (1 << i))
                    width = 6 if i % 3 == 0 else 12
                    wire += bits(1, not yes if width == 6 else yes)
                    if yes:
                        wire += bits(width, (1 << width) - 1 - i)
                add(f'frame/{count}/{mask}', 'object-frame-configuration-component', wire)

        for archetype in [1, 35, 40]:
            for dead in [False, True]:
                for flags in ([0] if archetype == 1 else [0, 0x10, 255]):
                    for mask in ([0] if archetype == 1 else [0, 255, 0x55] + [1 << i for i in range(8)]):
                        def opt(width, i, inverted=False):
                            yes = bool(mask & (1 << i))
                            return bits(1, not yes if inverted else yes) + (bits(width, (1 << width) - 1) if yes else '')
                        wire = bits(1, dead)
                        if archetype in [35, 40]:
                            wire += opt(32, 0) + bits(8, flags) + opt(5, 1, True) + opt(5, 2, True)
                            wire += bits(4, 9) + bits(3, 5) + bits(1, bool(mask & 8))
                            if mask & 8:
                                wire += opt(32, 4) + bits(3, 6) + opt(6, 5, True)
                            wire += bits(4, 10) + bits(4, 4) + opt(10, 6, True) + opt(10, 7)
                            if flags & 0x10:
                                wire += bits(2, 3) + bits(14, 0x1357) * 3
                            wire += bits(5, 17) + bits(5, 9) + opt(32, 3)
                            if archetype == 35:
                                wire += bits(1, dead)
                        add(f'dead/{archetype}/{dead}/{flags}/{mask}', 'object-dead-state-component',
                            wire, archetype=archetype)
