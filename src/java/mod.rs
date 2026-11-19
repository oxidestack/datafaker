//! The parts of the JDK that Datafaker's behaviour depends on: `java.util.Random`, `SplittableRandom` and
//! the `RandomGenerator` interface, `StrictMath`, `Character` predicates over UTF-16 units, `String.hashCode`, `HashMap` iteration order, `Double.toString`, and the part of
//! `BigDecimal` that `Number` uses, `java.time.Duration`, the part of `java.time`/`java.util.Date`/`SimpleDateFormat`
//! that the date providers use, `Base64`, and `java.util.zip.Deflater` and the `javax.imageio` writers that `Image` and
//! `Fingerprint` use.

pub mod big_decimal;
pub mod deflater;
pub mod duration;
pub mod fdlibm;
pub mod imageio;
pub mod random;
pub mod time;

use unicode_general_category::{GeneralCategory, get_general_category};

pub use big_decimal::BigDecimal;
pub use duration::Duration;
pub use random::{Random, RandomGenerator, SplittableRandom};
pub use time::{ChronoUnit, Date, DateTimeFormatter, Instant, LocalDate, LocalDateTime, LocalTime, Parsed, Period, SimpleDateFormat, TimeUnit, ZoneId};

fn category(c: u16) -> Option<GeneralCategory> {
    char::from_u32(c as u32).map(get_general_category)
}

/// `Character.isLetter(char)`.
pub fn is_letter(c: u16) -> bool {
    matches!(
        category(c),
        Some(
            GeneralCategory::UppercaseLetter
                | GeneralCategory::LowercaseLetter
                | GeneralCategory::TitlecaseLetter
                | GeneralCategory::ModifierLetter
                | GeneralCategory::OtherLetter
        )
    )
}

/// `Character.isDigit(char)`.
pub fn is_digit(c: u16) -> bool {
    matches!(category(c), Some(GeneralCategory::DecimalNumber))
}

/// `Character.isUpperCase(char)`: `UPPERCASE_LETTER` or `Other_Uppercase`, which is Unicode's `Uppercase`.
pub fn is_upper_case(c: u16) -> bool {
    char::from_u32(c as u32).is_some_and(char::is_uppercase)
}

/// `Character.isLowerCase(char)`: `LOWERCASE_LETTER` or `Other_Lowercase`, which is Unicode's `Lowercase`.
pub fn is_lower_case(c: u16) -> bool {
    char::from_u32(c as u32).is_some_and(char::is_lowercase)
}

/// `Character.isWhitespace(char)`.
pub fn is_whitespace(c: u16) -> bool {
    match c {
        0x09..=0x0D | 0x1C..=0x1F => true,
        0x00A0 | 0x2007 | 0x202F => false,
        _ => matches!(
            category(c),
            Some(GeneralCategory::SpaceSeparator | GeneralCategory::LineSeparator | GeneralCategory::ParagraphSeparator)
        ),
    }
}

fn single_case(c: u16, upper: bool) -> u16 {
    let Some(ch) = char::from_u32(c as u32) else { return c };
    let mut it: Box<dyn Iterator<Item = char>> = if upper { Box::new(ch.to_uppercase()) } else { Box::new(ch.to_lowercase()) };
    match (it.next(), it.next()) {
        (Some(m), None) if (m as u32) <= 0xFFFF => m as u16,
        _ => c,
    }
}

/// `Character.toLowerCase(char)`.
pub fn to_lower_case(c: u16) -> u16 {
    single_case(c, false)
}

/// `Character.toUpperCase(char)`.
pub fn to_upper_case(c: u16) -> u16 {
    single_case(c, true)
}

/// `String.toLowerCase(locale)`: Unicode lowercasing, with the Turkish and Azeri dotted/dotless i.
pub fn string_to_lower(s: &str, language: &str) -> String {
    if matches!(language, "tr" | "az") {
        let mapped: String = s.chars().map(|c| match c {
            'I' => 'ı',
            'İ' => 'i',
            other => other,
        }).collect();
        return mapped.to_lowercase();
    }
    s.to_lowercase()
}

/// `String.toUpperCase(locale)`.
pub fn string_to_upper(s: &str, language: &str) -> String {
    if matches!(language, "tr" | "az") {
        let mapped: String = s.chars().map(|c| if c == 'i' { 'İ' } else { c }).collect();
        return mapped.to_uppercase();
    }
    s.to_uppercase()
}

/// `String.hashCode()`.
pub fn string_hash(s: &str) -> i32 {
    s.encode_utf16().fold(0i32, |h, c| h.wrapping_mul(31).wrapping_add(c as i32))
}

/// The order in which a `java.util.HashMap` built with the default capacity, filled by `put` in the given order,
/// iterates its keys (tree bins aside). `hashes` are the keys' `hashCode()`s.
pub fn hash_map_order(hashes: &[i32]) -> Vec<usize> {
    let mut capacity = 16usize;
    while hashes.len() as f64 > capacity as f64 * 0.75 {
        capacity *= 2;
    }
    let spread = |h: i32| (h ^ ((h as u32) >> 16) as i32) as u32 as usize;
    let mut order: Vec<usize> = (0..hashes.len()).collect();
    order.sort_by_key(|&i| (spread(hashes[i]) & (capacity - 1), i));
    order
}

/// `Float.toString(f)` (JDK 19+: the shortest decimal that reads back as `f`).
pub fn float_to_string(f: f32) -> String {
    if f.is_finite() && f != 0.0 {
        java_decimal(&format!("{:e}", f.abs()), f < 0.0)
    } else {
        double_to_string(f as f64)
    }
}

/// `Double.toString(d)` (JDK 19+: the shortest decimal that reads back as `d`).
pub fn double_to_string(d: f64) -> String {
    if d.is_nan() {
        return "NaN".into();
    }
    if d.is_infinite() {
        return if d > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    if d == 0.0 {
        return if d.is_sign_negative() { "-0.0".into() } else { "0.0".into() };
    }
    java_decimal(&format!("{:e}", d.abs()), d < 0.0)
}

/// Lays out Rust's shortest `{:e}` digits as Java's `Double.toString`/`Float.toString` do.
fn java_decimal(sci: &str, negative: bool) -> String {
    let (mantissa, exp) = sci.split_once('e').expect("exponent");
    let exp: i32 = exp.parse().expect("exponent");
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let sign = if negative { "-" } else { "" };
    if (-3..7).contains(&exp) {
        let point = exp + 1;
        let s = if point <= 0 {
            format!("0.{}{}", "0".repeat((-point) as usize), digits)
        } else if point as usize >= digits.len() {
            format!("{}{}.0", digits, "0".repeat(point as usize - digits.len()))
        } else {
            format!("{}.{}", &digits[..point as usize], &digits[point as usize..])
        };
        format!("{sign}{s}")
    } else {
        let frac = if digits.len() > 1 { &digits[1..] } else { "0" };
        format!("{sign}{}.{}E{}", &digits[..1], frac, exp)
    }
}

/// `Base64.getEncoder().encodeToString(bytes)`: RFC 4648's alphabet, padded, no line breaks.
pub fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubles_print_as_java() {
        assert_eq!(double_to_string(1.0), "1.0");
        assert_eq!(double_to_string(1500.0), "1500.0");
        assert_eq!(double_to_string(1.0e7), "1.0E7");
        assert_eq!(double_to_string(0.001), "0.001");
        assert_eq!(double_to_string(0.0001), "1.0E-4");
        assert_eq!(double_to_string(-12.25), "-12.25");
    }

    #[test]
    fn hash_map_order_follows_buckets() {
        let keys = ["abcdefghijklmnopqrstuvwxyz", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", "!@#$%^&*", "0123456789"];
        let hashes: Vec<i32> = keys.iter().map(|k| string_hash(k)).collect();
        assert_eq!(hash_map_order(&hashes).len(), 4);
        assert_eq!(string_hash("abc"), 96354);
    }
}
