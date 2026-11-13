"""Device branch envelopes authored from pinned libfilmshell components.rs.

No local reader output is used. The original-reader exporter checks all body
endpoints and records the primitive boundaries independently.
"""


def append_cases(case, bits, handle):
    fixed = {
        'device-position-component': [14, 1],
        'device-position-animation-name-component': [32, 10],
        'device-position-animation-control-component': [64] * 4,
        'device-power-component': [14],
        'device-power-group-component': [14],
        'device-interaction-hold-time-component': [8],
        'device-interaction-start-time-override-component': [8],
        'device-control-action-string-override-component': [32, 32],
        'device-health-station-charges-component': [1, 6],
        'device-health-station-in-use-component': [1],
        'device-in-primary-mode-component': [1],
        'device-dispenser-require-los-component': [1],
        'device-dispenser-state-flags-component': [5],
        'device-position-transition-velocity-component': [18],
        'device-machine-flags-component': [9],
    }

    def optional_user(mode, generation=0):
        if mode == 0:
            return '0'
        width = 9 if mode == 1 else 13
        index = [0, (1 << width) - 1, 0x135, 1][generation]
        return '1' + bits(1, mode == 1) + handle(width, index, generation)

    for snapshot in [False, True]:
        def add(label, component, wire, version=1, complete=True):
            case(f'device/{snapshot}/{label}', component, wire,
                 version=version, snapshot=snapshot, complete=complete)

        for pattern in range(3):
            def word(width, offset=0):
                value = [0, (1 << width) - 1, ((1 << width) - 1) // 3 + offset][pattern]
                return bits(width, value % (1 << width))

            for name, widths in fixed.items():
                add(f'{name}/{pattern}', name,
                    ''.join(word(width, i) for i, width in enumerate(widths)))
            for present in [False, True]:
                add(f'group/{pattern}/{present}', 'device-position-group-component',
                    word(32) + bits(1, present) + (word(8) if present else ''))

            for count in range(9):
                # Empty, all present (alternating widths), mixed presence.
                wire = bits(8, count)
                for i in range(count):
                    mode = 0 if pattern == 0 else (1 + i % 2 if pattern == 1 else i % 3)
                    wire += optional_user(mode, i % 4)
                add(f'monitors/{pattern}/{count}', 'device-dispenser-monitors-changed-component', wire)

            masks = [0, 255, 0x55, 0xAA] + [1 << i for i in range(8)]
            for mask in masks:
                settings = state = '1'
                for i in range(8):
                    present = bool(mask & (1 << i))
                    settings += bits(1, present)
                    state += bits(1, present)
                    if present:
                        settings += ''.join(word(w, i) for w in [32, 14, 14, 10, 14, 2])
                        weight = [0, 1023, i % 2][pattern]
                        state += bits(10, weight) + (word(14, i) if weight else '')
                add(f'layer-settings/{pattern}/{mask}', 'device-animation-layer-settings-component', settings)
                add(f'layer-state/{pattern}/{mask}', 'device-animation-layer-state-component', state)

            for mask in range(4):
                wire = ''
                for i in range(2):
                    present = bool(mask & (1 << i))
                    wire += bits(1, present)
                    if present:
                        wire += ''.join(word(w, i) for w in [10, 10, 5, 10])
                add(f'timer/{pattern}/{mask}', 'device-object-dispenser-timer-component', wire)

        for name in ['device-interaction-in-progress-component', 'device-exclusive-user-component']:
            add(f'{name}/absent', name, '0')
            for mode in [1, 2]:
                for generation in range(4):
                    add(f'{name}/{mode}/{generation}', name, optional_user(mode, generation))
        for name in ['device-animation-layer-settings-component', 'device-animation-layer-state-component']:
            add(f'{name}/absent', name, '0')
        for count in [9, 255]:
            add(f'monitors/reject/{count}', 'device-dispenser-monitors-changed-component',
                bits(8, count), complete=False)

        # Each slot: absent, present with absent object, present with object.
        # State3 is unconditional, including when either presence gate is clear.
        for first in range(3):
            for second in range(3):
                for generation in range(4):
                    wire = ''
                    for i, mode in enumerate([first, second]):
                        wire += bits(1, mode != 0)
                        if mode:
                            wire += bits(1, mode == 2)
                            if mode == 2:
                                wire += handle(13, [0, 8191, 0x1357, 1][generation], generation)
                        wire += bits(3, (generation + 3 * i) % 8)
                    add(f'dispenser/{first}/{second}/{generation}', 'device-dispenser-state-component', wire)

        # These readers have no row-version branches in the pinned dispatch.
        # Exercise additional recorded version values at conditional boundaries.
        for version in [0, 2, 4]:
            add(f'group/version/{version}', 'device-position-group-component',
                bits(32, 0x12345678) + '1' + bits(8, 0xA5), version)
            add(f'state/version/{version}', 'device-animation-layer-state-component',
                '11' + bits(10, 0) + '1' + bits(10, 1) + bits(14, 0x1234) + '0' * 6, version)
