"""Synthetic action bodies shared by the component and event fixture authors.

No parser output is read. Each enclosing exporter runs its original upstream
entry point independently; common bytes exercise the shared wire grammar.
"""

def bits(width, value=0):
    return format(value, f'0{width}b')


def optional(width, present):
    return bits(1, present) + (bits(width, (1 << width) - 3) if present else '')


def vector(present):
    return bits(1, not present) + (bits(19, 0x5ABCD) + bits(10, 771) if present else '')


CASES = []
for variant in range(26):
    for flags in range(4):
        a, b = bool(flags & 1), bool(flags & 2)
        if variant == 0:
            wire = bits(1, a) + (bits(13, 7001) + bits(2, flags) if a else '') + optional(8, b)
        elif variant == 1:
            wire = optional(8, a) + bits(8, 213) + bits(8, 197) + bits(32, 0xFEDCBA98) + bits(15, 29003)
        elif variant == 2:
            wire = bits(8, 231) + optional(8, a) + bits(16, 55003)
        elif variant == 3:
            wire = bits(2, flags) + bits(15, 29003)
        elif variant == 4:
            wire = bits(32, 0xDEADBEEF) + bits(8, 213) + bits(15, 29003) + vector(a) + optional(8, b) + bits(16, 55003)
        elif variant == 5:
            wire = bits(32, 0xDEADBEEF) + optional(8, a) + bits(16, 55003) + bits(8, 213)
        elif variant == 6:
            wire = optional(8, a) + bits(32, 0xDEADBEEF)
        elif variant == 7:
            wire = optional(8, a) + bits(3, 5) + bits(15, 29003)
        elif variant in [8, 9, 10]:
            wire = optional(19, a) + bits(32, 0xDEADBEEF) + bits(4, 13)
            if variant == 8:
                wire += bits(4, 11) + bits(1, b)
        elif variant == 11:
            wire = optional(8, a) + bits(19, 0x5ABCD) + bits(2, flags)
        elif variant == 12:
            wire = bits(32, 0xDEADBEEF) + bits(1, a)
        elif variant == 13:
            wire = bits(32, 0x00000080) + bits(32, 0x4523C17F) + bits(32, 0xFEDCBA98)
            wire += bits(19, 0x5ABCD) + optional(8, a)
        elif variant == 14:
            wire = bits(8, 213)
        elif variant in [15, 22]:
            wire = optional(8, a)
        elif variant == 16:
            wire = vector(a)
        elif variant == 17:
            wire = bits(15, 29003) + bits(15, 17001) + bits(1, a) + optional(8, b)
        elif variant == 18:
            wire = '0' + bits(2, flags) + bits(32, 0xDEADBEEF) + bits(1, a)
            wire += bits(16, 55003) + bits(8, 213) + optional(8, b)
        elif variant == 19:
            wire = bits(32, 0xDEADBEEF) + bits(8, 213) + optional(8, a)
        elif variant == 20:
            wire = bits(15, 29003)
        elif variant in [21, 23, 24]:
            width = {21: 4, 23: 1, 24: 2}[variant]
            wire = optional(8, a) + bits(32, 0xDEADBEEF) + bits(width, (1 << width) - 1)
        elif variant == 25:
            wire = bits(32, 0xDEADBEEF) + bits(16, 55003) + bits(32, 0xFEDCBA98) + bits(1, a) + bits(8, 213) + optional(8, b)
        CASES.append(dict(name=f'{variant}/{flags}', variant=variant, wire=wire, complete=True))
for generation in [0, 2]:
    CASES.append(dict(name=f'0/generation-{generation}', variant=0,
                      wire='1' + bits(13, 7001) + bits(2, generation) + optional(8, True), complete=True))
CASES.append(dict(name='18/refused', variant=18, wire='1', complete=False))
for variant in range(26, 32):
    CASES.append(dict(name=f'{variant}/unknown', variant=variant, wire='', complete=False))
