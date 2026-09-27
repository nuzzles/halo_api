//! strconv.ParseFloat(s, 32) acceptance used by geometry.parseF32.
fn digit(c: u8, base: u8) -> bool {
    c.is_ascii_digit() || (base == 16 && c.is_ascii_hexdigit())
}
fn digits(s: &[u8], p: &mut usize, base: u8) -> Option<usize> {
    let mut count = 0;
    while *p < s.len() {
        if digit(s[*p], base) {
            count += 1;
            *p += 1;
        } else if s[*p] == b'_' {
            if count == 0 || !s.get(*p + 1).is_some_and(|&c| digit(c, base)) {
                return None;
            }
            *p += 1;
        } else {
            break;
        }
    }
    Some(count)
}
/// Invalid syntax and overflow become positive zero, as in native parseF32.
pub(super) fn geometry_float(s: &[u8]) -> f32 {
    if s.eq_ignore_ascii_case(b"nan") {
        return f32::NAN;
    }
    let (negative, text) = match s.first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, s),
    };
    if text.eq_ignore_ascii_case(b"inf") || text.eq_ignore_ascii_case(b"infinity") {
        return if negative {
            f32::NEG_INFINITY
        } else {
            f32::INFINITY
        };
    }
    let hex = text.starts_with(b"0x") || text.starts_with(b"0X");
    let base = if hex { 16 } else { 10 };
    let mut p = if hex { 2 } else { 0 };
    if hex && text.get(p) == Some(&b'_') {
        p += 1;
        if !text.get(p).is_some_and(|&c| digit(c, 16)) {
            return 0.;
        }
    }
    let Some(mut n) = digits(text, &mut p, base) else {
        return 0.;
    };
    if text.get(p) == Some(&b'.') {
        p += 1;
        let Some(after) = digits(text, &mut p, base) else {
            return 0.;
        };
        n += after;
    }
    if n == 0 {
        return 0.;
    }
    let mantissa_end = p;
    let exponent_marker = if hex { b'p' } else { b'e' };
    if text
        .get(p)
        .is_some_and(|c| c.to_ascii_lowercase() == exponent_marker)
    {
        p += 1;
        if matches!(text.get(p), Some(b'+' | b'-')) {
            p += 1;
        }
        if !matches!(digits(text, &mut p, 10), Some(1..)) {
            return 0.;
        }
    } else if hex {
        return 0.;
    }
    if p != text.len() {
        return 0.;
    }
    if !hex {
        let clean: Vec<_> = s.iter().copied().filter(|&c| c != b'_').collect();
        return std::str::from_utf8(&clean)
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .unwrap_or(0.);
    }
    // Convert hexadecimal directly to IEEE f32, avoiding a float64 intermediate
    // (which double-rounds halfway values). Keep the full tail for sticky bits.
    let mantissa = &text[2..mantissa_end];
    let mut fraction_digits = 0i64;
    let mut after_dot = false;
    let mut bits = Vec::new();
    for &c in mantissa {
        if c == b'.' {
            after_dot = true;
            continue;
        }
        if c == b'_' {
            continue;
        }
        if after_dot {
            fraction_digits += 1;
        }
        let d = if c.is_ascii_digit() {
            c - b'0'
        } else {
            c.to_ascii_lowercase() - b'a' + 10
        };
        for shift in (0..4).rev() {
            bits.push((d >> shift) & 1);
        }
    }
    let sign = u32::from(negative) << 31;
    let Some(first) = bits.iter().position(|&b| b != 0) else {
        return f32::from_bits(sign);
    };
    let exponent_text = &text[mantissa_end + 1..];
    let exponent_negative = exponent_text.first() == Some(&b'-');
    let mut exponent = 0i64;
    for &c in exponent_text {
        if c.is_ascii_digit() {
            exponent = (exponent * 10 + i64::from(c - b'0')).min(1_000_000_000);
        }
    }
    if exponent_negative {
        exponent = -exponent;
    }
    let bits = &bits[first..];
    let high = exponent - 4 * fraction_digits + bits.len() as i64 - 1;
    if high > 127 {
        return 0.;
    }
    if high < -150 {
        return f32::from_bits(sign);
    }
    let unit = (high - 23).max(-149);
    let retained = high - unit + 1;
    let mut value = 0u32;
    for i in 0..retained.max(0) as usize {
        value = (value << 1) | u32::from(*bits.get(i).unwrap_or(&0));
    }
    let round_index = retained.max(0) as usize;
    let halfway = bits.get(round_index) == Some(&1);
    let sticky = bits
        .get(round_index + 1..)
        .is_some_and(|tail| tail.contains(&1));
    if halfway && (sticky || value & 1 != 0) {
        value += 1;
    }
    let encoded = if high < -126 {
        value
    } else {
        (((high + 126) as u32) << 23) + value
    };
    if encoded >= 0x7f800000 {
        return 0.;
    }
    f32::from_bits(sign | encoded)
}
