//! `java.net.IDN.toASCII(input)` (flag 0) and Datafaker's `FakerIDN.toASCII`, with RFC 3492 Punycode.
//!
//! ASCII labels pass as they are. A non-ASCII label goes through IDNA2003 nameprep (RFC 3491) as the JDK runs it,
//! on Unicode 3.2: unassigned code points (table A.1) are refused, each code point is mapped (B.1 to nothing, B.2
//! case folding, NFKC), prohibited outputs (C.1.2, C.2.2, C.3-C.9) are refused, and the bidi rule of RFC 3454
//! section 6 applies. The tables (`idn_tables.rs`) are read back from the JDK's own `IDN.toASCII` by
//! `scripts/idn-tables.sh`, one code point at a time; the composition step of NFKC runs on current Unicode data,
//! which composes the code points Unicode 3.2 assigns as Unicode 3.2 did.

use unicode_normalization::UnicodeNormalization;

use crate::idn_tables::{L, MAP, PROHIBITED, RAND_AL, UNASSIGNED};
use crate::{Error, Result};

const ACE_PREFIX: &str = "xn--";
const MAX_LABEL_LENGTH: usize = 63;

fn is_dot(c: char) -> bool {
    matches!(c, '.' | '\u{3002}' | '\u{FF0E}' | '\u{FF61}')
}

/// `IDN.toASCII(input)`.
pub fn to_ascii(input: &str) -> Result<String> {
    if input.chars().count() == 1 && input.chars().all(is_dot) {
        return Ok(".".into());
    }
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::new();
    let mut p = 0;
    while p < chars.len() {
        let q = chars[p..].iter().position(|c| is_dot(*c)).map_or(chars.len(), |i| p + i);
        let label: String = chars[p..q].iter().collect();
        out.push_str(&to_ascii_label(&label)?);
        if q != chars.len() {
            out.push('.');
        }
        p = q + 1;
    }
    Ok(out)
}

fn to_ascii_label(label: &str) -> Result<String> {
    let is_ascii = label.is_ascii();
    let dest = if is_ascii { label.to_string() } else { nameprep(label)? };
    if dest.is_empty() {
        return Err(Error::illegal_argument("Empty label is not a legal name"));
    }
    let dest = if !is_ascii && !dest.is_ascii() {
        if dest.to_ascii_lowercase().starts_with(ACE_PREFIX) {
            return Err(Error::illegal_argument("The input starts with the ACE Prefix"));
        }
        format!("{ACE_PREFIX}{}", punycode_encode(&dest)?.to_ascii_lowercase())
    } else {
        dest
    };
    if dest.encode_utf16().count() > MAX_LABEL_LENGTH {
        return Err(Error::illegal_argument("The label in the input is too long"));
    }
    Ok(dest)
}

fn in_ranges(table: &[(u32, u32)], c: char) -> bool {
    let u = c as u32;
    table
        .binary_search_by(|(lo, hi)| {
            if *hi < u {
                std::cmp::Ordering::Less
            } else if *lo > u {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

fn parse_error(message: &str, label: &str) -> Error {
    Error::illegal_argument(format!("java.text.ParseException: {message}{label}"))
}

/// `StringPrep.prepare` with the nameprep profile: the unassigned check and the mapping per code point, NFKC over
/// the label, then the prohibited and bidi checks. The unassigned error ends with the label as given, the others
/// with the prepared label.
fn nameprep(label: &str) -> Result<String> {
    if label.chars().any(|c| in_ranges(UNASSIGNED, c)) {
        return Err(parse_error("An unassigned code point was found in the input ", label));
    }
    let mut mapped = String::with_capacity(label.len());
    for c in label.chars() {
        if c.is_ascii() {
            mapped.push(c.to_ascii_lowercase());
        } else {
            match MAP.binary_search_by_key(&(c as u32), |(k, _)| *k) {
                Ok(i) => mapped.push_str(MAP[i].1),
                Err(_) => mapped.push(c),
            }
        }
    }
    // Each mapped code point is already NFKC on its own; composing the label as a whole finishes NFKC.
    let prepared: String = mapped.nfkc().collect();
    // A prohibited code point maps to itself, and nothing composes with it.
    if label.chars().any(|c| in_ranges(PROHIBITED, c)) {
        return Err(parse_error("A prohibited code point was found in the input", &prepared));
    }
    let rand_al = |c: char| in_ranges(RAND_AL, c);
    if prepared.chars().any(rand_al) {
        let first = prepared.chars().next().is_some_and(rand_al);
        let last = prepared.chars().last().is_some_and(rand_al);
        if prepared.chars().any(|c| in_ranges(L, c) || c.is_ascii_alphabetic()) || !first || !last {
            return Err(parse_error("The input does not conform to the rules for BiDi code points.", &prepared));
        }
    }
    Ok(prepared)
}

/// RFC 3492 Punycode encoding.
fn punycode_encode(input: &str) -> Result<String> {
    const BASE: u32 = 36;
    const TMIN: u32 = 1;
    const TMAX: u32 = 26;
    const SKEW: u32 = 38;
    const DAMP: u32 = 700;
    let adapt = |mut delta: u32, num_points: u32, first: bool| {
        delta = if first { delta / DAMP } else { delta / 2 };
        delta += delta / num_points;
        let mut k = 0;
        while delta > ((BASE - TMIN) * TMAX) / 2 {
            delta /= BASE - TMIN;
            k += BASE;
        }
        k + (BASE - TMIN + 1) * delta / (delta + SKEW)
    };
    let digit = |d: u32| (if d < 26 { b'a' + d as u8 } else { b'0' + (d - 26) as u8 }) as char;
    let code_points: Vec<u32> = input.chars().map(|c| c as u32).collect();
    let mut output: String = code_points.iter().filter(|c| **c < 0x80).map(|c| char::from_u32(*c).expect("ascii")).collect();
    let basic = output.len() as u32;
    let mut handled = basic;
    if basic > 0 {
        output.push('-');
    }
    let mut n: u32 = 0x80;
    let mut delta: u32 = 0;
    let mut bias: u32 = 72;
    let overflow = || Error::illegal_argument("java.text.ParseException: The input point is out of range");
    while (handled as usize) < code_points.len() {
        let m = code_points.iter().copied().filter(|c| *c >= n).min().expect("unhandled point");
        delta = delta.checked_add((m - n).checked_mul(handled + 1).ok_or_else(overflow)?).ok_or_else(overflow)?;
        n = m;
        for &c in &code_points {
            if c < n {
                delta = delta.checked_add(1).ok_or_else(overflow)?;
            }
            if c == n {
                let mut q = delta;
                let mut k = BASE;
                loop {
                    let t = if k <= bias { TMIN } else if k >= bias + TMAX { TMAX } else { k - bias };
                    if q < t {
                        break;
                    }
                    output.push(digit(t + (q - t) % (BASE - t)));
                    q = (q - t) / (BASE - t);
                    k += BASE;
                }
                output.push(digit(q));
                bias = adapt(delta, handled + 1, handled == basic);
                delta = 0;
                handled += 1;
            }
        }
        delta += 1;
        n += 1;
    }
    Ok(output)
}

/// `FakerIDN.toASCII(in)`: the whole string if `IDN.toASCII` takes it, else each character that it takes.
pub fn faker_to_ascii(input: &str) -> Result<String> {
    if let Ok(s) = to_ascii(input) {
        return Ok(s);
    }
    let units: Vec<u16> = input.encode_utf16().collect();
    let mut result = String::new();
    for i in 0..units.len() {
        let one = String::from_utf16_lossy(&units[i..i + 1]);
        if let Ok(s) = to_ascii(&one) {
            result.push_str(&s);
        }
    }
    if result.is_empty() {
        return Err(Error::runtime(format!("Unable to convert \"{input}\" to ASCII")));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punycode() {
        assert_eq!(to_ascii("bücher").unwrap(), "xn--bcher-kva");
        assert_eq!(to_ascii("münchen.de").unwrap(), "xn--mnchen-3ya.de");
        assert_eq!(to_ascii("abc").unwrap(), "abc");
        assert!(to_ascii("a..b").is_err());
        assert_eq!(faker_to_ascii("a..b").unwrap(), "a..b");
    }
}
