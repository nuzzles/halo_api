# Live Fire region 3 position boundary

Source match: `9a87edc3-1758-46b2-ae6a-d248726dbcdf` (KOTH Live Fire).
`false-team-packet.bin` is the complete 237-byte packet payload in chunk 15 at
byte 717055, replay time 291.926900 seconds. `registry.json` retains component
names and precision levels for the schemas exercised by this packet.

The vehicle-schema object in slot 769 selects absolute-position region 3.
Applying Live Fire region 1's `[12, 12, 11]` widths consumes 35 coordinate bits.
Its following velocity and command-tick fields consequently start at the wrong
bits, and the next apparent record starts at bit 1237. Those bits happen to decode
as managed-player ID 1073747189, a mask containing out-of-schema bit 32, and team 6.
The managed-player schema association is correct; the record boundary is not.

Region 3 has a fixture-verified **54-bit aggregate position payload**:

- Position payload: bits [1145, 1199), followed by the two-bit finite field.
- Velocity: bits [1201, 1232).
- Command tick: bits [1232, 1242), value 39, matching the packet's counter.
- Four subsequent object deltas begin at 1242, 1385, 1489 and 1593.
- Controls end at bit 1891; the remaining five bits are zero padding.

An audit of every decoded update of this object in the match found 2,296 records.
All 2,250 carrying a command tick match the packet counter with the 54-bit payload;
the old layout matched only 100. The remaining 46 records have mask 4096 (scale)
and contain no command tick. `samples.json` identifies six independent packet
captures exercising distinct position/velocity/orientation component masks.

This verifies the aggregate width, **not** the per-axis split, world bounds, or
XYZ position. The contextual map calibration therefore retains those 54 bits as
`position.opaque_region_bits`, never as guessed `position[0..2]` fields. The
runtime cannot publish a position from this opaque field. The canonical reference
profile remains unchanged. This calibration is scoped to Live Fire.
