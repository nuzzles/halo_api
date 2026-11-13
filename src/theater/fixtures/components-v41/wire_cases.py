"""Independently authored component wire layouts; no halo_api output is read."""
import json
import runpy
from pathlib import Path

CASES = []


def bits(width, value=0):
    assert width >= 0 and 0 <= value < (1 << width)
    return format(value, f"0{width}b") if width else ""


def handle(width, value=5, generation=0):
    return bits(width, value) + bits(2, generation)


def case(name, component, wire, version=1, complete=True, snapshot=False, archetype=0):
    padded = wire.ljust((len(wire) + 7) // 8 * 8, "0")
    CASES.append(dict(name=name, component=component, version=version,
                      bit_len=len(wire), complete=complete, snapshot=snapshot,
                      hex=int(padded or "0", 2).to_bytes(len(padded) // 8, "big").hex()))

    if archetype:
        CASES[-1]["archetype"] = archetype
    return CASES[-1]

# Individual widths follow the reference's documented bit groups. Nonzero
# values distinguish adjacent fields even when two layouts share an endpoint.
FIXED = {
    "vehicle-auto-turret-triggers-component": [3],
    "vehicle-auto-turret-aiming-vector-component": [19],
    "vehicle-transformed-or-desired-open-state-changed-component": [1, 8],
    "vehicle-sentry-state-component": [3, 1],
    "vehicle-auto-turret-component": [2],
    "vehicle-seats-override-pitch-component": [8, 8],
    "vehicle-seats-override-yaw-component": [8, 8],
    "air-drop-flight-component": [2, 14, 8],
    "managed-radialmenu-enabled-component": [1],
    "managed-object-flags-component": [2],
    "managed-navpoint-override-flags": [5],
    "managed-navpoint-bottom-progress": [8],
    "managed-navpoint-top-progress": [8],
    "matchflow-isplaying-flags-component": [8],
    "managed-radialmenu-identifier-component": [32],
    "managed-radialmenu-item-count-component": [32],
    "managed-radialmenu-iconindex-component": [32],
    "managed-navpoint-object-marker": [32],
    "managed-object-navpoint-component": [32],
    "managed-object-networked-property-component": [32],
    "supply-lines-item-unlocked-component": [32],
    "game-engine-soft-ceilings-component": [128],
    "scenario-intro-component": [7, 1],
    "supply-lines-blocked-status-component": [2],
    "matchflow-sequence-data-component": [4, 32, 32, 32, 32],
    "matchflow-focus-data-component": [6, 4],
    "tacmap-backmenulocked-state": [16],
}
for name, widths in FIXED.items():
    for variant in [0, 1]:
        values = [0 if variant == 0 else ((1 << w) - 1) ^ ((i * 37) % (1 << w))
                  for i, w in enumerate(widths)]
        case(f"{name}/{variant}", name, "".join(bits(w, v) for w, v in zip(widths, values)))

for a in [0, 1]:
    for b in [0, 1]:
        case(f"warp/{a}{b}", "warp-component", bits(1, a) + bits(1, b)
             + (bits(8, 0xA5) if a else "") + (bits(8, 0x5A) if b else ""))
        case(f"weapon-set/{a}{b}", "vehicle-weapon-set-component", bits(3, 5)
             + bits(1, a) + ("" if a else bits(2, 2))
             + bits(1, b) + ("" if b else bits(2, 1)))
for name, width in [("vehicle-low-frequency-component", 5),
                    ("sound-placement-state-data-component", 20)]:
    for absent in [0, 1]:
        case(f"{name}/{absent}", name, bits(1, absent)
             + ("" if absent else bits(width, (1 << width) - 3))
             + ("1" if width == 20 else ""))
case("turret-target/absent", "vehicle-auto-turret-target-component", "0")
for unit in [0, 1]:
    case(f"turret-target/{unit}", "vehicle-auto-turret-target-component",
         "1" + bits(1, unit) + handle(9 if unit else 13, 301, 0))
for present in [0, 1]:
    case(f"turret-parent/{present}", "vehicle-equipment-turret-parent-component",
         bits(1, present) + (handle(13, 8001, 3) if present else ""))

# Both optional text forms, all parameter tags, and the short/long numeric form.
for component in ["managed-radialmenu-title-component", "managed-radialmenu-description-component"]:
    case(component + "/absent", component, "0")
    for tag in range(8):
        for narrow in ([0, 1] if tag in [1, 2] else [0]):
            argument = bits(3, tag)
            if tag == 1:
                argument += bits(1, narrow) + ("" if narrow else bits(5, 23))
            elif tag == 2:
                argument += bits(1, narrow) + bits(24 if narrow else 32, 0xAB1234)
            elif tag != 0:
                argument += bits(32, 0xDEADBEEF)
            case(f"{component}/tag{tag}-{narrow}", component,
                 "1" + bits(32, 0x12345678) + bits(3, 1) + argument)

for count, mask in [(0, 0xFFFFFFFF), (1, 1), (5, 0b10101), (32, 0x80000001), (33, 0xFFFFFFFF)]:
    wire = bits(32, count) + bits(32, mask) + bits(32, 0xFEED1234)
    if count <= 32:
        for i in range(count):
            if mask & (1 << i):
                absent = i % 2
                wire += bits(1, absent) + ("" if absent else bits(5, i)) + bits(1, i % 2)
    case(f"radial/{count}", "managed-radialmenu-visibility-component", wire, complete=count <= 32)
for count in [0, 1, 71, 8191]:
    case(f"kill-volumes/{count}", "game-engine-disabled-kill-volume-flags-component",
         bits(13, count) + ("10" * ((count + 1) // 2))[:count])
for count in [1, 17, 32]:
    case(f"supply-busy/{count}", "supply-lines-busy-state-component",
         bits(5, count - 1) + ("101" * count)[:count])
for count in [0, 2, 63]:
    wire = bits(6, count)
    for i in range(count):
        wire += bits(1, i % 2) + (bits(5, i % 32) if i % 2 == 0 else "")
        wire += bits(256, int.from_bytes(bytes((i + j) % 256 for j in range(32)), "big"))
    case(f"powerframe/{count}", "powerframe-player-selection-data-component", wire)
for variant in [0, 1]:
    wire = "1" + bits(16, 0xABCD)
    for i in range(4):
        absent = (i + variant) % 2
        wire += bits(1, absent) + ("" if absent else bits(7, 100 + i))
    for i in range(4):
        present = (i + variant) % 2
        wire += bits(1, present) + (bits(16, 50000 + i) if present else "")
    case(f"letterbox/{variant}", "GameEngineComposerLetterboxComponent", wire)
for component in ["tacmap-mapdismissallock", "tacmap-playercontrol", "tacmap-tooltip-state"]:
    for variant in [0, 1]:
        wire = ""
        for i in range(32):
            absent = (i + variant) % 2
            wire += bits(1, absent) + ("" if absent else bits(5, i))
            if component == "tacmap-tooltip-state":
                wire += bits(1, i % 2) + bits(32, 0x10203040 + i) + bits(32, 0x98765432 - i)
        case(f"{component}/{variant}", component, wire)


def condition(tag):
    """A condition independent of the library's filter reader."""
    wire = bits(4, tag)
    if tag == 0:
        return wire
    wire += "1"  # negate
    fixed = {1: (1, 1), 2: (32, 0xDEADBEEF), 3: (4, 9), 4: (9, 391),
             7: (4, 11), 8: (8, 231), 9: (32, 0xFEDCBA98)}
    if tag in fixed:
        wire += bits(*fixed[tag])
    elif tag == 5:
        wire += bits(3, 2) + "1" + "0" + bits(13, 7001)
    elif tag == 6:
        wire += bits(4, 2) + "0" + "1" + handle(13, 5003, 0)
    elif tag == 10:
        wire += bits(4, 9) + bits(4, 7) + bits(32, 0xDEADBEEF)
    elif tag == 11:
        wire += "0" + bits(5, 29)
    return wire


def filters(narrow, tags):
    mask = (1 << len(tags)) - 1
    return bits(4, mask) + bits(1 if narrow else 32, 1 if narrow else 0xABCDEF01) + "".join(condition(t) for t in tags)


for component in ["managed-object-interaction-filter-component", "managed-objective-interaction-filter-component"]:
    for version in [0, 1, 2, 3]:
        for tag in range(12):
            case(f"{component}/v{version}-tag{tag}", component, filters(version > 1, [tag]), version)
        case(f"{component}/v{version}-unknown", component, filters(version > 1, [15]), version, False)
for component in ["managed-navpoint-visibility-distance-filters-component",
                  "managed-navpoint-visible-offscreen-filters-component",
                  "managed-navpoint-can-be-occluded-filters-component"]:
    distance = "distance" in component
    for version in [1, 2, 3]:
        narrow = version > (2 if distance else 1)
        wire = filters(narrow, [0, 1, 10])
        wire += bits(16, 50001) + bits(16, 60003) if distance else "1"
        for i in range(3):
            wire += bits(16, 1001 + i) + bits(16, 2007 + i) if distance else bits(1, i % 2)
            if not narrow:
                wire += bits(4, 10 + i)
        wire += "".join(bits(3 if narrow else 2, i) for i in range(3))
        case(f"{component}/v{version}", component, wire, version)
for version in [0, 1, 2, 3]:
    wire = ""
    if version < 2:
        wire = bits(8, 0b10000001)
        for i in range(2):
            wire += bits(32, 0x10203040 + i) + filters(False, [2, 5]) + bits(32, 0xFEDCBA98 - i)
            for j in range(2):
                wire += bits(32, 0xABCDEF01 + j) + bits(4, 11 + j) + bits(2, j)
    case(f"visual-states/v{version}", "managed-navpoint-visual-states-component", wire, version)
for group in range(8):
    component = f"managed-navpoint-visual-state-groups-component-{group}"
    case(f"visual-group/{group}/absent", component, "0", 2)
    wire = "1" + bits(32, 0x10203040) + filters(True, [0, 6, 11]) + bits(32, 0xDEADBEEF)
    for i in range(3):
        wire += bits(32, 0xFFEEDD00 + i) + bits(3, i)
    case(f"visual-group/{group}/present", component, wire, 2)


# The enclosing snapshot selects three raw float words; it carries no point
# region gate. Delta points include the region gate and axis encodings.
for snapshot in [False, True]:
    for default_region in [False, True]:
        if snapshot:
            point = bits(32, 0x00000080) + bits(32, 0x4523C17F) + bits(32, 0x0000803F)
        elif default_region:
            point = "1" + bits(22, 0x3ABCDE) + bits(22, 1234) + bits(22, 0x210001)
        else:
            point = "00" + bits(15, 31001) + bits(15, 17003) + bits(17, 110005)
        label = f"{'snapshot' if snapshot else 'delta'}/{int(default_region)}"
        case("navpoint-offset/" + label, "managed-navpoint-position-offset", point, snapshot=snapshot)
        wire = point + "0" + bits(19, 0x5ABCD) + bits(8, 0xAD)
        wire += bits(16, 0x1234) + bits(8, 0xCD) + bits(2, 3) + bits(6, 4)
        for flags in [0, 1, 2, 7]:
            wire += bits(3, flags)
            if flags & 1:
                wire += point
            if flags & 2:
                wire += "1" + bits(8, 0xC7)
            wire += bits(16, 0xABCD) + bits(5, 29)
        case("low-frequency/" + label, "low-frequency", wire, snapshot=snapshot)
for state in range(4):
    case(f"vehicle-type-state/{state}", "vehicle-type-state-component",
         bits(2, state) + (bits(6, 53) if state in [1, 3] else ""))

# Shared snapshot and present-handle branches. Raw NaN/negative zero words
# ensure these are retained as bits rather than normalized floating values.
raw_point = bits(32, 0x00000080) + bits(32, 0x4523C17F) + bits(32, 0x0000803F)
for prefix in range(4):
    for flag in ([0, 1] if prefix == 0 else [0]):
        wire = bits(2, prefix)
        if prefix == 0:
            wire += bits(1, flag) + raw_point + bits(2, 3)
        case(f"dynamic-position/snapshot/{prefix}/{flag}",
             "object-position-dynamic-precision-component", wire,
             complete=prefix == 0, snapshot=True)
for attached in [0, 1]:
    for parent in ([0, 1] if attached else [0]):
        for region in ([0, 1] if attached else [0]):
            wire = bits(1, attached) + raw_point
            if attached:
                wire += bits(1, parent) + (handle(13, 7103, 0) if parent else "")
                wire += bits(1, region) + (bits(11, 1701) if region else "")
            wire += bits(2, 2)
            case(f"object-position/snapshot/{attached}{parent}{region}",
                 "object-position-component", wire, snapshot=True)
for name, width in [("crew-marked-objects-component", 13),
                    ("player-primary-respawn-object-component", 13),
                    ("player-desired-respawn-seat-component", 8)]:
    for present in [0, 1]:
        wire = bits(1, present) + (handle(width, (1 << width) - 5, 0) if present else "")
        if width == 8:
            wire += bits(7, 97)
        case(f"{name}/handle-{present}", name, wire)
for tag in [0, 1]:
    for first in [0, 1]:
        for second in [0, 1, 2]:
            wire = bits(1, tag) + (bits(32, 0xABCDEF12) if tag else "")
            wire += bits(1, first) + (handle(13, 6001, 3) if first else "") + bits(32, 0x1234ABCD)
            wire += bits(1, second != 0)
            if second:
                wire += bits(1, second == 2) + (handle(13, 7997, 0) if second == 2 else "")
                wire += bits(32, 0xDEADBEEF)
            case(f"effect-state/{tag}{first}{second}", "effect-state-data-component", wire)
for snapshot in [False, True]:
    case(f"cutscene/{snapshot}/absent", "nav-cutscene-flag-component", "0", snapshot=snapshot)
    for default_index in [0, 1]:
        for default_region in [0, 1]:
            point = raw_point if snapshot else (
                "1" + bits(22, 0x3ABCDE) + bits(22, 1234) + bits(22, 0x210001)
                if default_region else "00" + bits(15, 31001) + bits(15, 17003) + bits(17, 110005))
            wire = "1" + bits(1, default_index) + ("" if default_index else bits(16, 59003))
            wire += point + bits(32, 0xABCD1234) + bits(32, 0xDEADBEEF)
            wire += bits(16, 55007) + bits(8, 231) + bits(2, 3) + bits(32, 0xFEDCBA98)
            case(f"cutscene/{snapshot}/{default_index}{default_region}",
                 "nav-cutscene-flag-component", wire, snapshot=snapshot)

case("vehicle-physics/raw", "vehicle-type-physics-component", "1" + raw_point * 3)
for default_forward in [0, 1]:
    for stationary in [0, 1]:
        wire = "0" + bits(1, default_forward) + ("" if default_forward else bits(19, 0x5ABCD))
        wire += bits(8, 237) + bits(1, stationary)
        if not stationary:
            wire += bits(19, 0x6ABCD) + bits(8, 179)
        case(f"vehicle-physics/quantized/{default_forward}{stationary}",
             "vehicle-type-physics-component", wire)

for masked in [False, True]:
    component = "managed-object-player-masked-property-component" if masked else "managed-object-property-component"
    for tag in range(16):
        wire = bits(4, tag)
        if tag != 0 and (tag < 7) != masked:
            width = {1: 4, 11: 4, 2: 1, 10: 1, 3: 24, 7: 24}.get(tag, 32)
            wire += bits(width, (1 << width) - 3 if width > 1 else 1)
        case(f'property/{masked}/tag{tag}', component, wire)

# Shared map points must not use a width guessed from the registry row version.
for snapshot in [False, True]:
    for version in [0, 1, 2, 7]:
        for default_region in [False, True]:
            point = raw_point if snapshot else (
                "1" + bits(22, 0x3ABCDE) + bits(22, 1234) + bits(22, 0x210001)
                if default_region else "00" + bits(15, 31001) + bits(15, 17003) + bits(17, 110005))
            label = f'{snapshot}/v{version}/{default_region}'
            case('flock-position/' + label, 'flock-position-component', point, version, snapshot=snapshot)
            case('flock-destination/' + label, 'flock-destination-component',
                 '1' + point + (bits(2, 3) if version > 1 else ''), version, snapshot=snapshot)
            for present in [False, True]:
                case(f'crew-order/{label}/{present}', 'crew-order-component',
                     bits(3, 5) + bits(1, present) + (point if present else ''), version, snapshot=snapshot)
                case(f'respawn-location/{label}/{present}', 'player-desired-respawn-location-component',
                     bits(1, present) + (point + bits(19, 0x5ABCD) if present else ''), version, snapshot=snapshot)
                wire = bits(1, present)
                if present:
                    wire += '0' + bits(5, 23) + '1'
                    wire += ''.join(bits(16, 20003 + i) for i in range(4))
                    wire += bits(2, 3) + bits(2, 2)
                    wire += ''.join(bits(16, 50003 + i) for i in range(4))
                    wire += ('1' if default_region else '0' + bits(19, 0x6ABCD)) + bits(8, 213)
                    wire += point
                case(f'simulation/{label}/{present}', 'simulation-state-component', wire, version, snapshot=snapshot)

actions = runpy.run_path(str(Path(__file__).parent.parent / 'actions-v41/wire_cases.py'))['CASES']
for snapshot in [False, True]:
    for action in actions:
        # Only the first 73 mask bits contribute a sub-state. Upper bits remain
        # source fields and cannot turn into additional sub-state reads.
        wire = bits(32, 0x80000001) + bits(32, 0x80000002) + bits(32, 0xFFFFFE05)
        wire += bits(4, 1) + bits(7, 101) + bits(5, action['variant']) + action['wire']
        if action['complete']:
            wire += ''.join(bits(1, i % 2) + (bits(2, 3) if i % 2 else '') for i in range(6))
            wire += bits(32, 0xFEDCBA98) + bits(32, 0x76543210) + bits(32, 0xDEADBEEF)
        case(f'action/{snapshot}/{action["name"]}', 'biped-action-component', wire,
             snapshot=snapshot, complete=action['complete'])

# A maximum-length mixed list checks that variant tails cannot eat the next tag.
for snapshot in [False, True]:
    selected = [next(a for a in actions if a['name'] == f'{v}/3') for v in range(11, 26)]
    wire = bits(96) + bits(4, 15)
    for i, action in enumerate(selected):
        wire += bits(7, 100 + i) + bits(5, action['variant']) + action['wire']
    wire += bits(96, (1 << 96) - 1)
    case(f'action/{snapshot}/mixed-max', 'biped-action-component', wire, snapshot=snapshot)
    for rejected in ['18/refused', '31/unknown']:
        valid = next(a for a in actions if a['name'] == '13/3')
        invalid = next(a for a in actions if a['name'] == rejected)
        wire = bits(96) + bits(4, 2)
        wire += bits(7, 73) + bits(5, valid['variant']) + valid['wire']
        wire += bits(7, 74) + bits(5, invalid['variant']) + invalid['wire']
        case(f'action/{snapshot}/mixed-stop/{rejected}', 'biped-action-component', wire,
             snapshot=snapshot, complete=False)

# Parent-local object positions: each optional tail changes the component end.
# The first discriminant is set, so these never use map/world position bounds.
for version in [0, 1, 2, 4]:
    for generation in [None, 0, 1, 2, 3]:
        for region in [False, True]:
            for finite in range(4):
                wire = '1' + bits(14, 0) + bits(14, 16383) + bits(14, 8193)
                wire += bits(1, generation is not None)
                if generation is not None:
                    wire += handle(13, 8191, generation)
                wire += bits(1, region)
                if region:
                    wire += bits(11, 2047 if finite & 1 else 0)
                wire += bits(2, finite)
                case(f'object-position/attached/{version}/{generation}/{region}/{finite}',
                     'object-position-component', wire, version=version)

# Dynamic orientation, versions where the extra precision flag is present.
# Exclude the 16 paths where the original reader conflicts with newer
# instruction-addressed LevelUp disassembly; see orientation-source.json.
for version in [2, 4]:
    for snapshot in [False, True]:
        def orientation(name, wire):
            case(f'orientation/{version}/{snapshot}/{name}',
                 'object-forward-and-up-dynamic-precision-component',wire,
                 version=version,snapshot=snapshot)
        for high in [0]:
            raw = ''.join(bits(32, v) for v in [0, 0x4523C17F, 0x80, 0x803F, 0x80BF, 0xFFFF7F7F])
            orientation(f'raw/{high}', '1'+bits(1,high)+raw)
        for baseline in [0, 1]:
            for high in [0, 1]:
                if baseline and not high:
                    for keep, delta in [(1,0),(0,0),(0,1)]:
                        for short_roll in [1]:
                            wire='010'+bits(1,keep)
                            if not keep:
                                wire+=bits(1,delta)+(bits(8,0xE3) if delta else bits(19,0x54321))
                            wire+=bits(1,short_roll)+(bits(4,0xD) if short_roll else bits(8,0xDB))
                            orientation(f'baseline/{keep}/{delta}/{short_roll}',wire)
                else:
                    for default in [0,1]:
                        wire='0'+bits(1,baseline)+bits(1,high)+bits(1,default)
                        if not default:wire+=bits(30 if high else 19,0x3456789 if high else 0x54321)
                        wire+=bits(30 if high else 8,0x2345678 if high else 0xDB)
                        orientation(f'absolute/{baseline}/{high}/{default}',wire)

# Audit previously existing fixed-width aliases against the pinned dispatch.
# Bits8 has an archetype-specific snapshot form and is audited separately.
import re
inventory = json.loads((Path(__file__).parents[2] / 'docs/research/libfilmshell/component-inventory.json').read_text())
covered = {c['component'] for c in CASES}
for name, reader in inventory['upstream_aliases'].items():
    fixed = re.fullmatch(r'(?:Skip|Bits)([0-9]+)', reader)
    if name in covered or fixed is None or reader == 'Bits8':
        continue
    width = int(fixed.group(1))
    for pattern, value in [('zero',0),('ones',(1<<width)-1),('mixed',int('10'*(width//2)+'1'*(width%2),2))]:
        case(f'fixed-alias/{name}/{pattern}',name,bits(width,value),version=4)

# Only high-frequency is shared by two distinct descriptor tables. The
# recorded archetype selects its grammar in both delta and snapshot records.
for archetype, width in [(3,26),(4,8)]:
    for snapshot in [False,True]:
        for version in [0,1,4]:
            for pattern,value in [('zero',0),('ones',(1<<width)-1),('mixed',int('10'*(width//2),2))]:
                case(f'high-frequency/{archetype}/{snapshot}/{version}/{pattern}',
                     'high-frequency',bits(width,value),version=version,snapshot=snapshot,archetype=archetype)

# Equipment stack count is an actual length, not a last index. The original
# LevelUp FUN_140f72dec analysis confirms n iterations (advance before compare).
for snapshot in [False, True]:
    def equipment(label, component, wire, version=1):
        case(f'equipment/{snapshot}/{label}', component, wire,
             version=version, snapshot=snapshot)

    stack = 'equipment-tracked-object-handles-stack-component'
    for count in range(16):
        for pattern in ['absent', 'present', 'alternating']:
            wire = bits(4, count)
            for i in range(count):
                present = pattern == 'present' or (pattern == 'alternating' and i % 2 == 0)
                wire += bits(1, present)
                if present:
                    wire += handle(13, [0, 8191, 0x1357][i % 3], (i + count) % 4)
            equipment(f'stack/{count}/{pattern}', stack, wire)
    for version in [0, 1, 2, 4]:
        for generation in range(4):
            equipment(f'stack/single/{version}/{generation}', stack,
                      bits(4, 1) + '1' + handle(13, 8191, generation), version)

    activated = 'equipment-activated-component'
    for value in range(8):
        equipment(f'activated/value/{value}', activated, '0' + bits(3, value))
    equipment('activated/no-handle', activated, '10')
    for generation in range(4):
        for index in [0, 511, 0x135]:
            equipment(f'activated/handle/{generation}/{index}', activated,
                      '11' + handle(9, index, generation))

    signal = 'equipment-control-signal-component'
    for value in range(16):
        equipment(f'signal/{value}/absent', signal, bits(4, value) + '0')
        for generation in range(4):
            equipment(f'signal/{value}/{generation}', signal,
                      bits(4, value) + '1' + handle(9, [0, 511, 0x135, 1][generation], generation))

    command = 'equipment-command-tick-component'
    for single, count in [(0, 2), (1, 1)]:
        for mask in range(1 << count):
            for value in [0, 255, 0xA5]:
                wire = bits(1, single)
                for i in range(count):
                    present = bool(mask & (1 << i))
                    wire += bits(1, present)
                    if present:
                        wire += bits(8, value)
                equipment(f'command/{single}/{mask}/{value}', command, wire)

from unit_weapon_cases import append_cases
append_cases(case, bits, handle)

from tacmap_cases import append_cases as append_tacmap_cases
append_tacmap_cases(case, bits)

from device_cases import append_cases as append_device_cases
append_device_cases(case, bits, handle)

from physics_projectile_cases import append_cases as append_physics_projectile_cases
append_physics_projectile_cases(case, bits)

from object_state_cases import append_cases as append_object_state_cases
append_object_state_cases(case, bits, handle)

from ability_anchor_cases import append_cases as append_ability_anchor_cases
append_ability_anchor_cases(case, bits, handle)

from biped_cases import append_cases as append_biped_cases
append_biped_cases(case, bits, handle)

from managed_cases import append_cases as append_managed_cases
append_managed_cases(case, bits, handle, filters)

assert len({c['name'] for c in CASES}) == len(CASES)
Path(__file__).with_name("cases.json").write_text(json.dumps(CASES, indent=2) + "\n")
print(f"Authored {len(CASES)} cases for {len({c['component'] for c in CASES})} component names")
