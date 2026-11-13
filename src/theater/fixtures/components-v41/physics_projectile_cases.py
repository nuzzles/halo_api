"""Independent rigid-body, vector and projectile envelopes from pinned readers.

Rigid-body calls FUN_14076e494 at precision 0x10, as documented by pinned
LevelUp lecteur_position_exceptions.go; the older traversal-only helper there
is a dated delta compatibility exception, not the game grammar.
"""


def append_cases(case, bits):
    for snapshot in [False, True]:
        def add(label, name, wire, version=4):
            case(f'physics-projectile/{snapshot}/{label}', name, wire,
                 version=version, snapshot=snapshot)

        for pattern in range(3):
            def word(width, offset=0):
                return bits(width, [0, (1 << width) - 1,
                                   (((1 << width) - 1) // 3 + offset) % (1 << width)][pattern])

            for version in [0, 1, 4]:
                for form in (['raw'] if snapshot else ['default', 'indexed', 'mixed']):
                    for mask in [0, 255, 0x55, 0xAA] + [1 << i for i in range(8)]:
                        wire = bits(8, mask)
                        for i in range(8):
                            if not mask & (1 << i):
                                continue
                            default_direction = pattern == 0 or (pattern == 2 and i % 2 == 0)
                            wire += bits(1, default_direction)
                            if not default_direction:
                                wire += word(19, i)
                            wire += word(8, i)
                            if snapshot:
                                # Little-endian float bytes in MSB-first wire words:
                                # 0, -0 and a NaN payload remain exact source bits.
                                wire += ''.join(bits(32, v) for v in [0, 0x80, 0x4523C17F])
                            elif form == 'default' or (form == 'mixed' and i % 2 == 0):
                                wire += '1' + word(22, i) * 3
                            else:
                                wire += '00' + word(15, i) * 2 + word(17, i)
                        add(f'rigid/{version}/{form}/{pattern}/{mask}',
                            'generic-rigid-body-transforms-component', wire, version)

            for default in [False, True]:
                orientation = bits(1, default) + ('' if default else word(19)) + word(8)
                add(f'orientation/{pattern}/{default}', 'object-forward-and-up-component', orientation)
                for name, magnitude in [('object-angular-velocity-component', 8),
                                        ('object-translational-velocity-component', 10)]:
                    wire = bits(1, default) + ('' if default else word(19) + word(magnitude))
                    add(f'{name}/{pattern}/{default}', name, wire)
                    dynamic = name.replace('-component', '-dynamic-precision-component')
                    add(f'{dynamic}/compressed/{pattern}/{default}', dynamic, '0' + wire)

            for name in ['object-angular-velocity-dynamic-precision-component',
                         'object-translational-velocity-dynamic-precision-component']:
                raw = [0, 0x80, 0x4523C17F] if pattern == 2 else [int(word(32), 2)] * 3
                add(f'{name}/raw/{pattern}', name, '1' + ''.join(bits(32, v) for v in raw))

            for present in [False, True]:
                add(f'physics-state/{pattern}/{present}', 'physics-state-component',
                    word(32) + bits(1, present) + (word(32, 7) if present else ''))
                add(f'projectile-tick/{pattern}/{present}', 'projectile-command_tick',
                    bits(1, present) + (word(8) if present else ''))
                add(f'ignore-player/{pattern}/{present}', 'item-ignore-player-component',
                    bits(1, not present) + (word(5) if present else ''))
                add(f'playback/{pattern}/{present}', 'simulation-state-playback-component',
                    bits(1, present) + (word(4) + word(32) * 2 if present else ''))
                for rest in [False, True]:
                    add(f'projectile-rest/{pattern}/{rest}/{present}', 'projectile-at-rest-state',
                        bits(1, rest) + bits(1, present) + (word(19) if present else ''))
            add(f'animated-mesh/{pattern}', 'animated-mesh-dynamic-state-component',
                word(8) + word(1) + word(16))
        for name in ['projectile-tether-state', 'projectile-deceleration-disabled-state',
                     'item-at-rest-component']:
            for value in [0, 1]:
                add(f'{name}/{value}', name, bits(1, value))
