"""Authored manager/player/stat layouts and source-conflict cases.

No halo_api output is read. Source-only forms follow the pinned LevelUp readers;
the main suite contains only cases also understood by original libfilmshell.
"""
import json
from pathlib import Path


def append_cases(case, bits, handle, filters):
    source = []

    def leb(value):
        out = ''
        while value >= 128:
            out += bits(8, (value & 127) | 128)
            value >>= 7
        return out + bits(8, value)

    def argument(tag, flag=False):
        wire = bits(3, tag)
        if tag == 1:
            wire += bits(1, flag) + ('' if flag else bits(5, 21))
        elif tag == 2:
            wire += bits(1, flag) + bits(24 if flag else 32, 0xABCDEF)
        elif tag:
            wire += bits(32, 0x12345678 + tag)
        return wire

    def text(tags):
        if tags is None:
            return '0'
        return '1' + bits(32, 0xA153789B) + bits(3, len(tags)) + ''.join(
            argument(tag, i % 2 == 0) for i, tag in enumerate(tags))

    for snapshot in [False, True]:
        for version in [0, 1, 2, 4]:
            def add(label, name, wire, **kwargs):
                return case(f'managed/{snapshot}/{version}/{label}', name, wire,
                            snapshot=snapshot, version=version, **kwargs)

            def sourced(label, name, wire):
                padded = wire.ljust((len(wire) + 7) // 8 * 8, '0')
                row = dict(name=f'managed-source/{snapshot}/{version}/{label}',
                           component=name, snapshot=snapshot, version=version,
                           complete=True, bit_len=len(wire),
                           hex=int(padded, 2).to_bytes(len(padded) // 8, 'big').hex())
                source.append(row)
                return row

            for pattern in range(3):
                def word(width, offset=0):
                    return bits(width, ([0, (1 << width) - 1, ((1 << width) - 1) // 3][pattern] + offset) % (1 << width))

                for name, widths in {
                    'flock-relevancy-component': [8],
                    'game-engine-screen-sequence-component': [4, 8],
                    'managed-navpoint-docking-order-component': [8],
                    'managed-navpoint-flags-component': [8],
                    'managed-navpoint-radial-progress': [8],
                    'managed-navpoint-timers-component': [7, 7],
                    'managed-objective-outro-phase-duration-component': [8],
                    'managed-objective-priority-component': [8],
                }.items():
                    add(f'{name}/{pattern}', name, ''.join(word(w, i) for i, w in enumerate(widths)))
                for absent in [False, True]:
                    wire = bits(1, absent) + ('' if absent else word(5))
                    add(f'round/{pattern}/{absent}', 'game-engine-current-round-component', wire)
                    add(f'respawn-block/{pattern}/{absent}', 'managed-object-participant-respawn-block-component', word(32) + wire)
                for mask in [0, 1, 128, 0x55, 0xFF]:
                    wire = word(8) + word(9) + word(9, 1) + bits(8, mask) + word(8, 2) + word(8, 3)
                    wire += ''.join(word(4, i) for i in range(mask.bit_count()))
                    add(f'team-map/{pattern}/{mask}', 'game-engine-team-mapping-component', wire)
                for mask in [0, 1, 1 << 31, 0x55555555, 0xFFFFFFFF]:
                    # The pinned source reads one R32 mask; libfilmshell splits it.
                    sourced(f'alliance/{pattern}/{mask}', 'game-engine-alliance-component',
                        bits(32, mask) + ''.join(word(64, i) for i in range(mask.bit_count())))
                    wire = ''
                    for i in range(32):
                        yes = bool(mask & (1 << i))
                        wire += bits(1, not yes)
                        if yes:
                            wire += bits(2, i % 4) + word(32, i) + word(32, i + 1)
                    add(f'music-variables/{pattern}/{mask}', 'music-variables-component', wire)
                for tag in range(8):
                    add(f'danger/{pattern}/{tag}', 'flock-remembered-danger-component',
                        bits(3, tag) + word(8) + (word(19) if tag else ''))
                for identity in [0, 1, 0xFFFFFFFF]:
                    add(f'rtpc/{pattern}/{identity}', 'managed-object-rtpc-component',
                        bits(32, identity) + (word(22) if identity else ''))
                    for present in [False, True]:
                        wire = bits(32, identity)
                        if identity != 0xFFFFFFFF:
                            wire += bits(1, present) + (word(12) if present else '')
                        add(f'personal-ai/{pattern}/{identity}/{present}', 'personal-ai-data-component', wire)
                for mask in [0, 1, 32, 0x15, 63]:
                    wire = word(3)
                    for i in range(6):
                        yes = bool(mask & (1 << i))
                        wire += bits(1, yes) + (word(12, i) if yes else '')
                    add(f'player-malleable/{pattern}/{mask}', 'player-malleable-properties-simulation-component', wire + word(9))
                for selector in range(4):
                    def stat(index):
                        sel = (selector + index) % 4
                        return bits(2, sel) + word(8 << sel, index)
                    for mask in range(4):
                        wire = word(5) + word(5, 1) + stat(0) + stat(1) + bits(1, mask & 1 != 0) + bits(1, mask & 2 != 0)
                        wire += (stat(2) if mask & 1 else '') + (stat(3) if mask & 2 else '')
                        add(f'stat-current/{pattern}/{selector}/{mask}', 'statborg-current-round-value-stat-component', wire)
                    for mask in [0, 1, 1 << 31, 0x55555555, 0xFFFFFFFF]:
                        wire = bits(32, mask)
                        for i in range(2 * mask.bit_count()):
                            yes = pattern == 1 or pattern == 2 and i % 2 == 0
                            wire += bits(1, not yes) + (stat(i) if yes else '')
                        add(f'stat-final/{pattern}/{selector}/{mask}', 'statborg-finalized-rounds-values-stat-component', wire)

            for unit in [False, True]:
                for generation in range(4):
                    for flag in [False, True]:
                        wire = '1' + bits(1, unit) + handle(9 if unit else 13, 0 if generation == 0 else (511 if unit else 8191), generation) + bits(1, flag)
                        add(f'aim-assist/{unit}/{generation}/{flag}', 'player-aim-assist-component', wire)
            add('aim-assist/absent', 'player-aim-assist-component', '01')
            for count in [0, 1, 64]:
                for gates in range(8):
                    wire = bits(1, count != 0)
                    if count:
                        wire += bits(32, 0x12345678) + bits(6, count - 1) + ''.join(bits(1, i % 2) for i in range(count))
                    for i in range(3):
                        index, near, far = (bool(gates & (1 << j)) for j in range(3))
                        wire += bits(1, not index) + (bits(6, 63 - i) if index else '')
                        wire += bits(1, near) + (bits(12, 0xA35 + i) if near else '')
                        wire += bits(1, far) + (bits(12, 0x531 + i) if far else '')
                    add(f'player-frame/{count}/{gates}', 'player-desired-frame-configuration-component', wire)

            text_forms = [None, [], *[[i] for i in range(8)], list(range(7)), [1, 1, 2, 2, 7, 0, 3]]
            for i, tags in enumerate(text_forms):
                for name in ['managed-objective-formatted-text-component', 'managed-objective-secondary-formatted-text-component']:
                    add(f'{name}/{i}', name, text(tags))
                for kind in range(4):
                    add(f'prompt/{i}/{kind}', 'managed-player-custom-input-prompt-widget', '1' + bits(2, kind) + text(tags))
            add('prompt/absent', 'managed-player-custom-input-prompt-widget', '0')
            for count in [0, 1, 2, 255]:
                wire = bits(8, count)
                for i in range(count):
                    wire += bits(32, 0xA1234567 + i) + text(None if count == 255 else text_forms[i % len(text_forms)])
                add(f'navpoint-text/{count}', 'managed-navpoint-formatted-text-component', wire)
            for name in ['managed-navpoint-docking-filter-component', 'managed-navpoint-visibility-filter-component']:
                for tag in range(12):
                    add(f'{name}/{tag}', name, filters(version > 1, [tag]))
                add(f'{name}/full-mask', name, filters(version > 1, [0, 1, 5, 6]))
                add(f'{name}/unsupported', name, filters(version > 1, [15]), complete=False)

            for present in [False, True]:
                add(f'command-tick/{present}', 'unit-command-tick-component',
                    bits(1, present) + (bits(8, 0xA5) if present else '') + '0')

            for pair in range(4):
                for tail in range(4):
                    wire = '1' + bits(32, 0x12345678)
                    wire += bits(32, 0x89ABCDEF) + bits(1, pair & 1 != 0)
                    wire += bits(32, 0x53A17EF2) + bits(1, pair & 2 != 0)
                    wire += bits(1, tail & 1 != 0)
                    if tail & 1:
                        wire += '1' + bits(16, 0x1357) + bits(16, 0xABCD)
                    wire += bits(1, tail & 2 != 0)
                    if tail & 2:
                        wire += ''.join(bits(16, 0x1357 + i) for i in range(6))
                    sourced(f'music/{pair}/{tail}', 'music-state-component', wire)
                    if pair == 0:
                        add(f'music/shared/{tail}', 'music-state-component', wire)

            for mask in [0, 1, 1 << 63, (1 << 63) | 3, 0x5555555555555555, 0xFFFFFFFFFFFFFFFF]:
                for offset in range(4):
                    wire, values = bits(64, mask), []
                    for slot in range(64):
                        if not mask & (1 << slot):
                            continue
                        tag = (slot + offset) % 4
                        wire += bits(2, tag)
                        values.append([slot, tag])
                        if tag:
                            a, b, state, c = slot + 17, slot + 231, (slot + 11) % 32, slot + 513
                            wire += bits(16, a) + bits(16, b) + bits(5, state)
                            if tag == 1:
                                wire += bits(16, c)
                            values[-1] += [a, b, state] + ([c] if tag == 1 else [])
                    row = sourced(f'timers/{mask}/{offset}', 'managed-engine-timers-component', wire)
                    row['timers'] = values

            for present in [False, True]:
                for count in [0, 1, 7]:
                    for titles in range(4):
                        for tag in range(8):
                            # LevelUp has no leading kind4; its tail tag3 selects the word.
                            wire = text([tag] if present else None) + bits(24, 0xA35BC1) + bits(1, present)
                            if present:
                                wire += bits(3, count)
                                for i in range(count):
                                    wire += bits(16, 0xABCD + i) + bits(8, i) + bits(8, 0xA5) + '1' + bits(32, 0xABCDEF01 + i)
                                wire += bits(2, titles)
                                for i in range(titles):
                                    wire += bits(32, 0xAB123456 + i) + bits(16, 0x3571) + bits(1, i % 2 == 0)
                                    if i % 2 == 0:
                                        wire += argument(tag, i == 0)
                                if titles:
                                    wire += bits(8, 0xA5)
                                wire += bits(3, tag) + (bits(32, 0x12345678) if tag else '')
                                wire += '1' + bits(32, 0x3571A5BC) + '1' + bits(3, 5) + bits(6, 35) + bits(32, 0x7FA12345)
                            sourced(f'splash/{present}/{count}/{titles}/{tag}', 'managed-object-networked-splash-message-static-component', wire)
                            if not present and count == titles == tag == 0:
                                add('splash/shared/absent', 'managed-object-networked-splash-message-static-component', wire)

            add('multiplayer/shared/absent', 'object-multiplayer-properties-component', '1')
            for variant in [0, 1, 5, 31]:
                for end in [0, 1]:
                    for wire_type in range(32):
                        body = ''
                        if wire_type in [2, 3, 14]:
                            body = bits(8, 0xA5)
                        elif wire_type == 7:
                            body = bits(32, 0x89ABCDEF)
                        elif wire_type == 8:
                            body = bits(64, 0x0123456789ABCDEF)
                        elif wire_type in [4, 5, 6, 15, 16, 17]:
                            body = leb(0x123456789)
                        elif wire_type in [9, 10, 18]:
                            body = leb(3) + bits(48 if wire_type == 18 else 24, 0x123456)
                        elif wire_type in [11, 12]:
                            body = bits(8, (4 << 5) | 2) + bits(24, 0x123456)
                        elif wire_type == 13:
                            body = bits(8, 2) + bits(8, 4) + leb(2) + bits(8, 0xA5) + leb(129) + bits(8, 0x53) + leb(0)
                        wire = '0' + bits(5, variant) + leb(1) + bits(8, 0xC0 | wire_type) + bits(8, 0x35) + body
                        if wire_type > 1:
                            wire += bits(8, 0xE0 | end) + bits(16, 0x1234)
                        sourced(f'tlv/{variant}/{end}/{wire_type}', 'object-multiplayer-properties-component', wire)

    assert len({row['name'] for row in source}) == len(source)
    Path(__file__).with_name('managed-source.json').write_text(json.dumps(source, indent=2) + '\n')
