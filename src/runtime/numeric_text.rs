// SPDX-License-Identifier: Apache-2.0
// Float formatting derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Rust translation and safe parser changes: TeamDman.
// See NOTICE and licenses/Apache-2.0.txt.
//! Safe, locale-independent text conversion for the native execution lane.
//! The accepted spelling follows upstream's C-locale `strtof` contract, including
//! hexadecimal numbers and the original byte length versus first-NUL distinction.

pub(super) fn show(value: f32) -> String {
    if value.is_nan() {
        return "nan".into();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-inf"
        } else {
            "inf"
        }
        .into();
    }
    let double = f64::from(value);
    for precision in 0..=8 {
        let scientific = format!("{double:.precision$e}");
        if scientific.parse::<f32>().ok() != Some(value) {
            continue;
        }
        let (mantissa, exponent) = scientific
            .split_once('e')
            .expect("scientific formatter has an exponent");
        let exponent = exponent.parse::<i32>().expect("binary32 exponent fits i32");
        if !(-6..21).contains(&exponent) {
            return format!("{mantissa}e{exponent:+}");
        }
        let precision = i32::try_from(precision).expect("precision is at most eight");
        if exponent > precision {
            let mut result = mantissa.replace('.', "");
            let zeros = usize::try_from(exponent - precision).expect("positive decimal padding");
            result.extend(std::iter::repeat_n('0', zeros));
            return result;
        }
        let decimals = precision - exponent;
        let decimals = usize::try_from(decimals).expect("nonnegative decimal precision");
        return format!("{double:.decimals$}");
    }
    unreachable!("nine significant decimal digits round-trip every finite binary32 value")
}

pub(super) fn read(text: &str) -> Option<u32> {
    if text.is_empty() {
        return None;
    }
    let prefix = text
        .split('\0')
        .next()
        .expect("split always yields a prefix");
    // Upstream checks the original byte length and *end == 0, but does not
    // require strtof to consume a character. A leading NUL consequently reads 0.
    if prefix.is_empty() {
        return Some(0);
    }
    let number = prefix.trim_start_matches([' ', '\t', '\n', '\r', '\u{b}', '\u{c}']);
    let (negative, unsigned) = match number.as_bytes().first() {
        Some(b'-') => (true, &number[1..]),
        Some(b'+') => (false, &number[1..]),
        _ => (false, number),
    };
    let sign = if negative { 0x8000_0000 } else { 0 };
    if unsigned.eq_ignore_ascii_case("inf") || unsigned.eq_ignore_ascii_case("infinity") {
        return Some(sign | 0x7f80_0000);
    }
    if let Some(payload) = nan_payload(unsigned) {
        return Some(sign | 0x7fc0_0000 | payload);
    }
    if unsigned.starts_with("0x") || unsigned.starts_with("0X") {
        return hexadecimal(&unsigned[2..]).map(|bits| sign | bits);
    }
    // Rust's binary32 parser is correctly rounded. Validate the C decimal
    // grammar first so spellings such as Rust's NaN/inf cannot bypass the rules.
    decimal(unsigned)
        .then(|| number.parse::<f32>().ok().map(f32::to_bits))
        .flatten()
}

fn decimal(text: &str) -> bool {
    let mut bytes = text.as_bytes().iter().copied().peekable();
    let mut digits = 0;
    while bytes.peek().is_some_and(u8::is_ascii_digit) {
        bytes.next();
        digits += 1;
    }
    if bytes.peek() == Some(&b'.') {
        bytes.next();
        while bytes.peek().is_some_and(u8::is_ascii_digit) {
            bytes.next();
            digits += 1;
        }
    }
    if digits == 0 {
        return false;
    }
    if matches!(bytes.peek(), Some(b'e' | b'E')) {
        bytes.next();
        if matches!(bytes.peek(), Some(b'+' | b'-')) {
            bytes.next();
        }
        let mut exponent_digits = 0;
        while bytes.peek().is_some_and(u8::is_ascii_digit) {
            bytes.next();
            exponent_digits += 1;
        }
        if exponent_digits == 0 {
            return false;
        }
    }
    bytes.next().is_none()
}

fn nan_payload(text: &str) -> Option<u32> {
    if text.eq_ignore_ascii_case("nan") {
        return Some(if cfg!(target_env = "msvc") {
            0x003f_ffff
        } else {
            0
        });
    }
    let prefix = text.get(..4)?;
    if !prefix.eq_ignore_ascii_case("nan(") || !text.ends_with(')') {
        return None;
    }
    let payload = &text[4..text.len() - 1];
    if !payload
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return None;
    }
    // C leaves the payload implementation-defined. The verified Microsoft CRT
    // uses all-one payload bits even for numeric payload strings. Other targets
    // use the conventional numeric-payload interpretation; CRT parity there is
    // not established by the Windows oracle.
    if cfg!(target_env = "msvc") {
        return Some(0x003f_ffff);
    }
    let (radix, digits) = if payload.starts_with("0x") || payload.starts_with("0X") {
        (16, &payload[2..])
    } else if payload.starts_with('0') {
        (8, payload)
    } else {
        (10, payload)
    };
    let bits = u64::from_str_radix(digits, radix).unwrap_or(0);
    Some(u32::try_from(bits & 0x003f_ffff).expect("masked binary32 payload fits u32"))
}

fn hexadecimal(text: &str) -> Option<u32> {
    let mut prefix = 0_u32;
    let mut bit_count = 0_i64;
    let mut sticky = false;
    let mut after_dot = false;
    let mut fractional_digits = 0_i64;
    let mut digits = 0_usize;
    let mut exponent_start = text.len();
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'.' && !after_dot {
            after_dot = true;
            continue;
        }
        if matches!(byte, b'p' | b'P') {
            exponent_start = index;
            break;
        }
        let digit = char::from(byte).to_digit(16)?;
        digits += 1;
        fractional_digits += i64::from(after_dot);
        for shift in (0..4).rev() {
            let bit = (digit >> shift) & 1;
            if bit_count == 0 && bit == 0 {
                continue;
            }
            bit_count += 1;
            if bit_count <= 32 {
                prefix = (prefix << 1) | bit;
            } else {
                sticky |= bit != 0;
            }
        }
    }
    if digits == 0 {
        return None;
    }
    let exponent = if exponent_start == text.len() {
        0
    } else {
        binary_exponent(&text[exponent_start + 1..])?
    };
    if bit_count == 0 {
        return Some(0);
    }
    let mut highest = exponent - 4 * fractional_digits + bit_count - 1;
    if highest > 127 {
        return Some(0x7f80_0000);
    }
    if highest < -150 {
        return Some(0);
    }
    let kept = if highest >= -126 { 24 } else { highest + 150 };
    let mut significand = if bit_count <= kept {
        prefix << (kept - bit_count)
    } else {
        let discarded = bit_count.min(32) - kept;
        let integer = if discarded == 32 {
            0
        } else {
            prefix >> discarded
        };
        let guard = (prefix >> (discarded - 1)) & 1;
        let lower = (prefix & ((1 << (discarded - 1)) - 1)) != 0 || sticky;
        integer + u32::from(guard != 0 && (lower || integer & 1 != 0))
    };
    if highest < -126 {
        return Some(significand);
    }
    if significand == 1 << 24 {
        significand >>= 1;
        highest += 1;
    }
    if highest > 127 {
        return Some(0x7f80_0000);
    }
    let exponent_bits = u32::try_from(highest + 127).expect("normal binary32 exponent is positive");
    Some((exponent_bits << 23) | (significand & 0x007f_ffff))
}

fn binary_exponent(text: &str) -> Option<i64> {
    let (negative, digits) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    if digits.is_empty() {
        return None;
    }
    let mut exponent = 0_i64;
    for byte in digits.bytes() {
        if !byte.is_ascii_digit() {
            return None;
        }
        // The input is bounded to 8 MiB. Beyond this exponent magnitude no
        // mantissa of that size can change overflow/underflow classification.
        exponent = (exponent * 10 + i64::from(byte - b'0')).min(1_000_000_000);
    }
    Some(if negative { -exponent } else { exponent })
}
