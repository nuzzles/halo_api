// Copyright 2025 The Go Authors. All rights reserved.
// Ported from Go 1.26.5 internal/strconv/ftoadbox.go and math.go.
// BSD license: reference/GO_LICENSE.txt.
//! Native float32 shortest decimals for cache JSON, including Dragonbox boundaries.
fn pow(k: i32, e: i32) -> (u64, i32) {
    let mut phi = POWERS[(k + 31) as usize];
    if !(0..=27).contains(&k) {
        phi += 1;
    }
    (phi, e + ((k * 108853) >> 15))
}
fn trim(mut mant: u32, mut exponent: i32) -> (u32, i32) {
    while mant != 0 && mant.is_multiple_of(10) {
        mant /= 10;
        exponent += 1;
    }
    (mant, exponent)
}
fn parity(mant: u32, phi: u64, beta: i32) -> (bool, bool) {
    let r = (mant as u64).wrapping_mul(phi);
    (
        (r.checked_shr((64 - beta) as u32).unwrap_or(0) & 1) != 0,
        (r >> ((32 - beta) as u32)) as u32 == 0,
    )
}
/// Decimal mantissa and power of ten. Input must be finite; sign is handled by caller.
pub(super) fn shortest(value: f32) -> (u32, i32) {
    let bits = value.to_bits() & 0x7fffffff;
    let raw_exp = (bits >> 23) as i32;
    let mut mant = bits & 0x7fffff;
    if raw_exp != 0 {
        mant |= 1 << 23;
    }
    if mant == 0 {
        return (0, 0);
    }
    let exp = raw_exp.max(1) - 127 - 23;
    if mant == 1 << 23 && raw_exp != 0 {
        let k0 = -((exp * 631305 - 261663) >> 21);
        let (phi, beta) = pow(k0, exp);
        let mut xi = ((phi - (phi >> 25)) >> (40 - beta)) as u32;
        let zi = (phi.wrapping_add(phi >> 24) >> (40 - beta)) as u32;
        if exp != 2 && exp != 3 {
            xi += 1;
        }
        let q = zi / 10;
        if xi <= q * 10 {
            return trim(q, -k0 + 1);
        }
        let mut rounded = ((phi >> (39 - beta)).wrapping_add(1) as u32) / 2;
        if exp == -77 && !rounded.is_multiple_of(2) {
            rounded -= 1;
        } else if rounded < xi {
            rounded += 1;
        }
        return (rounded, -k0);
    }
    let k0 = -((exp * 78913) >> 18);
    let (phi, beta) = pow(1 + k0, exp);
    let u = (mant * 2 + 1) << beta;
    let upper = (((u as u128) * (phi as u128)) >> 32) as u64;
    let zi = (upper >> 32) as u32;
    let exact = upper as u32 == 0;
    let (mut s, mut r) = (zi / 100, zi % 100);
    let delta = (phi >> (63 - beta)) as u32;
    if r < delta {
        if r != 0 || !exact || mant.is_multiple_of(2) {
            return trim(s, -k0 + 1);
        }
        s -= 1;
        r = 100;
    } else if r == delta {
        let (p, exact) = parity(mant * 2 - 1, phi, beta);
        if p || (exact && mant.is_multiple_of(2)) {
            return trim(s, -k0 + 1);
        }
    }
    let d = (r + 5).wrapping_sub(delta / 2);
    let mut rounded = 10 * s + d / 10;
    if d.is_multiple_of(10) {
        let (p, exact) = parity(mant * 2, phi, beta);
        if p != (!d.wrapping_sub(5).is_multiple_of(2)) || (exact && !rounded.is_multiple_of(2)) {
            rounded -= 1;
        }
    }
    (rounded, -k0)
}

// High halves of Go pow10tab.go for the full finite float32 exponent domain.
const POWERS: [u64; 78] = [
    0x81ceb32c4b43fcf4, // 1e-31
    0xa2425ff75e14fc31, // 1e-30
    0xcad2f7f5359a3b3e, // 1e-29
    0xfd87b5f28300ca0d, // 1e-28
    0x9e74d1b791e07e48, // 1e-27
    0xc612062576589dda, // 1e-26
    0xf79687aed3eec551, // 1e-25
    0x9abe14cd44753b52, // 1e-24
    0xc16d9a0095928a27, // 1e-23
    0xf1c90080baf72cb1, // 1e-22
    0x971da05074da7bee, // 1e-21
    0xbce5086492111aea, // 1e-20
    0xec1e4a7db69561a5, // 1e-19
    0x9392ee8e921d5d07, // 1e-18
    0xb877aa3236a4b449, // 1e-17
    0xe69594bec44de15b, // 1e-16
    0x901d7cf73ab0acd9, // 1e-15
    0xb424dc35095cd80f, // 1e-14
    0xe12e13424bb40e13, // 1e-13
    0x8cbccc096f5088cb, // 1e-12
    0xafebff0bcb24aafe, // 1e-11
    0xdbe6fecebdedd5be, // 1e-10
    0x89705f4136b4a597, // 1e-9
    0xabcc77118461cefc, // 1e-8
    0xd6bf94d5e57a42bc, // 1e-7
    0x8637bd05af6c69b5, // 1e-6
    0xa7c5ac471b478423, // 1e-5
    0xd1b71758e219652b, // 1e-4
    0x83126e978d4fdf3b, // 1e-3
    0xa3d70a3d70a3d70a, // 1e-2
    0xcccccccccccccccc, // 1e-1
    0x8000000000000000, // 1e0
    0xa000000000000000, // 1e1
    0xc800000000000000, // 1e2
    0xfa00000000000000, // 1e3
    0x9c40000000000000, // 1e4
    0xc350000000000000, // 1e5
    0xf424000000000000, // 1e6
    0x9896800000000000, // 1e7
    0xbebc200000000000, // 1e8
    0xee6b280000000000, // 1e9
    0x9502f90000000000, // 1e10
    0xba43b74000000000, // 1e11
    0xe8d4a51000000000, // 1e12
    0x9184e72a00000000, // 1e13
    0xb5e620f480000000, // 1e14
    0xe35fa931a0000000, // 1e15
    0x8e1bc9bf04000000, // 1e16
    0xb1a2bc2ec5000000, // 1e17
    0xde0b6b3a76400000, // 1e18
    0x8ac7230489e80000, // 1e19
    0xad78ebc5ac620000, // 1e20
    0xd8d726b7177a8000, // 1e21
    0x878678326eac9000, // 1e22
    0xa968163f0a57b400, // 1e23
    0xd3c21bcecceda100, // 1e24
    0x84595161401484a0, // 1e25
    0xa56fa5b99019a5c8, // 1e26
    0xcecb8f27f4200f3a, // 1e27
    0x813f3978f8940984, // 1e28
    0xa18f07d736b90be5, // 1e29
    0xc9f2c9cd04674ede, // 1e30
    0xfc6f7c4045812296, // 1e31
    0x9dc5ada82b70b59d, // 1e32
    0xc5371912364ce305, // 1e33
    0xf684df56c3e01bc6, // 1e34
    0x9a130b963a6c115c, // 1e35
    0xc097ce7bc90715b3, // 1e36
    0xf0bdc21abb48db20, // 1e37
    0x96769950b50d88f4, // 1e38
    0xbc143fa4e250eb31, // 1e39
    0xeb194f8e1ae525fd, // 1e40
    0x92efd1b8d0cf37be, // 1e41
    0xb7abc627050305ad, // 1e42
    0xe596b7b0c643c719, // 1e43
    0x8f7e32ce7bea5c6f, // 1e44
    0xb35dbf821ae4f38b, // 1e45
    0xe0352f62a19e306e, // 1e46
];
