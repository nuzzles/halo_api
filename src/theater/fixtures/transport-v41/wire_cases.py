"""Independent Python/RFC 1950 streams for bounded film decompression tests."""
from pathlib import Path
import zlib

root = Path(__file__).parent
payload = b'abcde0123456789' * 2048
for name, strategy, expected_block in [('fixed', zlib.Z_FIXED, 1), ('dynamic', zlib.Z_DEFAULT_STRATEGY, 2)]:
    compressor = zlib.compressobj(level=6, wbits=15, strategy=strategy)
    wire = compressor.compress(payload) + compressor.flush()
    assert ((wire[2] >> 1) & 3) == expected_block
    assert zlib.decompress(wire) == payload
    (root / f'{name}.zlib').write_bytes(wire)
    print(name, len(wire), 'bytes ->', len(payload))
