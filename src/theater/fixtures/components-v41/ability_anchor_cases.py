"""Non-predicted ability bodies from the pinned reader/writer call analysis.

LevelUp components_biped_anchor.go describes FUN_142f25e90 and its writer;
libfilmshell's unchanged AbilityNonPredicted reader supplies primitive traces.
"""


def append_cases(case, bits, handle):
    component = 'biped-spartan-ability-non-predicted-state'
    for snapshot in [False, True]:
        for version in [0, 1, 2, 4]:
            def empty_observation(tag):
                return dict(tag=tag, body_walked=False, body_ok=False, inner=None,
                            reference=None, position=None, flags=None, source=None,
                            target=None, destination=None, value8=None, vectors=[None] * 3,
                            flag=None, packed=None, tail=None)

            def add(label, wire, observation):
                row = case(f'ability-anchor/{snapshot}/{version}/{label}', component, wire,
                           version=version, snapshot=snapshot)
                row['observation'] = observation

            for tag in range(3):
                add(f'inactive/{tag}', bits(2, tag) + (bits(3, 5) if version > 1 else ''), empty_observation(tag))
            for kind in range(1, 9):
                for form in (['raw', 'object', 'unit'] if snapshot else ['default', 'indexed', 'object', 'unit']):
                    for pattern in range(4):
                        def word(width, offset=0):
                            values = [0, (1 << width) - 1, ((1 << width) - 1) // 3, 1]
                            return bits(width, (values[pattern] + offset) % (1 << width))

                        def present(index=0):
                            return pattern == 1 or (pattern == 2 and index % 2 == 0) or (pattern == 3 and index % 2 != 0)

                        def opt(width, index=0):
                            yes = present(index)
                            return bits(1, yes) + (word(width, index) if yes else '')

                        def reference(width, index=0):
                            yes = present(index)
                            return bits(1, yes) + (handle(width, int(word(width, index), 2), pattern) if yes else '')

                        def point(index=0):
                            if snapshot:
                                return ''.join(bits(32, v) for v in [0, 0x80, 0x4523C17F])
                            if form == 'default' or form in ['object', 'unit'] and index % 2 and pattern < 2:
                                return '1' + word(22, index) * 3
                            return '00' + word(15, index) * 2 + word(17, index)

                        def vector(index=0):
                            constant = not present(index)
                            return bits(1, constant) + ('' if constant else word(24, index) + word(12, index))

                        wire = '11' + bits(3, kind - 1)
                        observation = empty_observation(3)
                        observation.update(body_walked=True, body_ok=True, inner=kind - 1)

                        def ref_observation(start, end, yes, value=0, probe=False):
                            return dict(kind='VariableWidth', start_bit=start, end_bit=end,
                                        present=yes, value=value if yes else 0,
                                        tail=pattern if yes else 0, probe=probe)

                        def append_reference(name, width, index=0):
                            nonlocal wire
                            start = len(wire)
                            wire += reference(width, index)
                            observation[name] = ref_observation(start, len(wire), present(index), int(word(width, index), 2))

                        def append_vector(index):
                            nonlocal wire
                            wire += vector(index)
                            if present(index):
                                observation['vectors'][index] = [int(word(24, index), 2), int(word(12, index), 2)]

                        attached = form in ['object', 'unit']
                        wire += bits(1, attached)
                        if attached:
                            width = 9 if form == 'unit' else 13
                            wire += bits(1, form == 'unit') + handle(width, int(word(width), 2), pattern)
                        observation['reference'] = ref_observation(5, len(wire), attached,
                                                                  int(word(width), 2) if attached else 0, True)
                        start = len(wire)
                        if attached:
                            wire += word(2) + word(13) * 3 + opt(16)
                        else:
                            wire += point()
                        observation['position'] = dict(start=start, end=len(wire))
                        wire += word(6)
                        observation['flags'] = int(word(6), 2)
                        if kind == 1:
                            wire += opt(8)
                        elif kind == 2:
                            append_reference('source', 8, 1)
                            wire += opt(8)
                        elif kind == 3:
                            append_reference('target', 13)
                            append_reference('source', 8, 1)
                            for i in range(3):
                                append_vector(i)
                            wire += word(24) + word(9)
                        elif kind in [4, 5]:
                            append_reference('source', 8, 1)
                            append_vector(0)
                            start = len(wire)
                            wire += point(1)
                            observation['destination'] = dict(start=start, end=len(wire))
                            wire += word(24) + word(9)
                        elif kind == 6:
                            wire += opt(8)
                            if not present():
                                append_reference('source', 8, 1)
                            append_reference('target', 13)
                            append_vector(0)
                            append_vector(1)
                            wire += word(1) + word(24)
                            observation['flag'] = bool(int(word(1), 2))
                        if kind in [1, 2, 6] and present():
                            observation['value8'] = int(word(8), 2)
                        if kind in [3, 4, 5, 6]:
                            observation['packed'] = int(word(24), 2)
                        if kind in [3, 4, 5]:
                            observation['tail'] = int(word(9), 2)
                        # Encoded labels 7/8 have no variant-specific tail.
                        wire += bits(3, 5) if version > 1 else ''
                        add(f'{kind}/{form}/{pattern}', wire, observation)
