"""Bounded, schema-free Bond Compact Binary v2 reader for companion assets.

This is an experiment utility, not a film packet parser. Field IDs and omitted
defaults need a schema before they acquire gameplay meanings. Spans are absolute
byte offsets, end-exclusive; field headers have their own header_offset.
Protocol: microsoft/bond, e27170b0f768037087e5d3103b9e8035c0968626,
cpp/inc/bond/protocol/compact_binary.h.
"""
import math
import struct


class DecodeError(ValueError):
    pass


class Reader:
    def __init__(self, data, *, origin=0, max_depth=64, max_values=1_000_000):
        self.data = data
        self.pos = 0
        self.origin = origin
        self.max_depth = max_depth
        self.remaining_values = max_values

    def fail(self, reason):
        raise DecodeError(f"byte {self.origin + self.pos}: {reason}")

    def take(self, count, end):
        if count < 0 or count > end - self.pos:
            self.fail("value exceeds its containing structure")
        result = self.data[self.pos:self.pos + count]
        self.pos += count
        return result

    def byte(self, end):
        return self.take(1, end)[0]

    def varuint(self, bits, end):
        result = 0
        for shift in range(0, bits, 7):
            byte = self.byte(end)
            result |= (byte & 127) << shift
            if result >= 1 << bits:
                self.fail(f"integer exceeds uint{bits}")
            if byte < 128:
                return result
        self.fail(f"unterminated uint{bits}")

    def budget(self, count=1):
        if count > self.remaining_values:
            self.fail("value-count limit exceeded")
        self.remaining_values -= count

    def check_count(self, count, end, multiplier=1):
        # Every encoded value consumes at least one byte and one budget unit.
        if count * multiplier > min(end - self.pos, self.remaining_values):
            self.fail("container count exceeds available bytes or value limit")

    def value(self, kind, end, depth=0):
        if depth > self.max_depth:
            self.fail("nesting limit exceeded")
        if not 2 <= kind <= 18:
            self.fail(f"unsupported value type {kind}")
        self.budget()
        start = self.pos
        if kind in (2, 3, 14):
            value = self.byte(end)
            if kind == 2:
                if value > 1:
                    self.fail("invalid boolean")
                value = bool(value)
            elif kind == 14 and value >= 128:
                value -= 256
        elif kind in (4, 5, 6, 15, 16, 17):
            bits = {4: 16, 5: 32, 6: 64, 15: 16, 16: 32, 17: 64}[kind]
            value = self.varuint(bits, end)
            if kind >= 15:
                value = (value >> 1) ^ -(value & 1)
        elif kind in (7, 8):
            value = struct.unpack('<f' if kind == 7 else '<d',
                                  self.take(4 if kind == 7 else 8, end))[0]
            if not math.isfinite(value):
                # Keep the IEEE bytes even though JSON has no nonfinite numbers.
                value = {"nonfinite": str(value), "hex": self.data[start:self.pos].hex()}
        elif kind in (9, 18):
            count = self.varuint(32, end)
            raw = self.take(count * (2 if kind == 18 else 1), end)
            try:
                value = raw.decode('utf-16-le' if kind == 18 else 'utf-8')
            except UnicodeDecodeError as error:
                self.fail(f"invalid string encoding: {error.reason}")
        elif kind == 10:
            count = self.varuint(32, end)
            if count < 1 or count > end - self.pos:
                self.fail("invalid structure length")
            boundary = self.pos + count
            value = []
            while self.pos < boundary:
                header_offset = self.pos
                header = self.byte(boundary)
                child_type, field_id = header & 31, header >> 5
                if child_type in (0, 1):
                    if header != child_type:
                        self.fail("invalid STOP/STOP_BASE header")
                    self.budget()
                    value.append(dict(type=child_type, offset=self.origin + header_offset,
                                      end=self.origin + self.pos, value=None))
                    if child_type == 0:
                        if self.pos != boundary:
                            self.fail("STOP precedes structure boundary")
                        break
                    continue
                if field_id == 6:
                    field_id = self.byte(boundary)
                elif field_id == 7:
                    field_id = int.from_bytes(self.take(2, boundary), 'little')
                child = self.value(child_type, boundary, depth + 1)
                child.update(id=field_id, header_offset=self.origin + header_offset)
                value.append(child)
            if not value or value[-1]['type'] != 0:
                self.fail("structure has no terminating STOP")
        elif kind in (11, 12):
            header = self.byte(end)
            child_type = header & 31
            if not 2 <= child_type <= 18:
                self.fail("invalid element type")
            count = (header >> 5) - 1 if header >> 5 else self.varuint(32, end)
            self.check_count(count, end)
            value = dict(element_type=child_type,
                         items=[self.value(child_type, end, depth + 1) for _ in range(count)])
        else:  # map
            key_type, value_type = self.byte(end), self.byte(end)
            if not (2 <= key_type <= 18 and 2 <= value_type <= 18):
                self.fail("invalid map types")
            count = self.varuint(32, end)
            self.check_count(count, end, 2)
            value = dict(key_type=key_type, value_type=value_type,
                         pairs=[[self.value(key_type, end, depth + 1),
                                 self.value(value_type, end, depth + 1)] for _ in range(count)])
        return dict(type=kind, offset=self.origin + start, end=self.origin + self.pos, value=value)


def parse(data, *, origin=0, allow_zero_padding=False, **limits):
    """Read one root struct. Trailing bytes are errors unless zero padding is opted in.

    Padding is reported separately and is never included in the root's span.
    There is no automatic protocol detection or recovery past malformed fields.
    """
    reader = Reader(data, origin=origin, **limits)
    root = reader.value(10, len(data))
    trailing = data[reader.pos:]
    if trailing and (not allow_zero_padding or any(trailing)):
        reader.fail("trailing bytes after root structure")
    return dict(root=root, padding_bytes=len(trailing))


def field(node, field_id, kind=None):
    """Get a unique field of the expected type; no schema defaults are supplied."""
    if node is None or node['type'] != 10:
        return None
    matches = [f for f in node['value'] if f.get('id') == field_id]
    if len(matches) != 1 or (kind is not None and matches[0]['type'] != kind):
        return None
    return matches[0]
