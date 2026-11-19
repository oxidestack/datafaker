//! `new java.text.SimpleDateFormat(pattern).format(date)` in the default locale `en_US` and the default zone,
//! on `GregorianCalendar` (Julian before 1582-10-15; Sunday-first weeks, one minimal day in the first week).

use super::{Date, HybridDate, MONTHS, WEEKDAYS};
use crate::{Error, Result};

/// `DateFormatSymbols.patternChars`.
const PATTERN_CHARS: &str = "GyMdkHmsSEDFwWahKzZYuXL";

#[derive(Debug, Clone, PartialEq, Eq)]
enum Item {
    Literal(String),
    Field(char, usize),
}

/// A compiled `SimpleDateFormat`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleDateFormat {
    items: Vec<Item>,
}

impl SimpleDateFormat {
    /// `new SimpleDateFormat(pattern)`.
    pub fn new(pattern: &str) -> Result<SimpleDateFormat> {
        let chars: Vec<char> = pattern.chars().collect();
        let mut items = Vec::new();
        let mut in_quote = false;
        let mut quoted = String::new();
        let mut last: Option<(char, usize)> = None;
        let flush = |items: &mut Vec<Item>, last: &mut Option<(char, usize)>| {
            if let Some((c, n)) = last.take() {
                items.push(Item::Field(c, n));
            }
        };
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '\'' {
                if i + 1 < chars.len() && chars[i + 1] == '\'' {
                    i += 2;
                    flush(&mut items, &mut last);
                    if in_quote {
                        quoted.push('\'');
                    } else {
                        items.push(Item::Literal("'".into()));
                    }
                    continue;
                }
                if in_quote {
                    items.push(Item::Literal(std::mem::take(&mut quoted)));
                } else {
                    flush(&mut items, &mut last);
                }
                in_quote = !in_quote;
                i += 1;
                continue;
            }
            if in_quote {
                quoted.push(c);
                i += 1;
                continue;
            }
            if !c.is_ascii_alphabetic() {
                flush(&mut items, &mut last);
                items.push(Item::Literal(c.to_string()));
                i += 1;
                continue;
            }
            if !PATTERN_CHARS.contains(c) {
                return Err(Error::illegal_argument(format!("Illegal pattern character '{c}'")));
            }
            match last {
                Some((l, n)) if l == c => last = Some((c, n + 1)),
                _ => {
                    flush(&mut items, &mut last);
                    last = Some((c, 1));
                }
            }
            i += 1;
        }
        if in_quote {
            return Err(Error::illegal_argument("Unterminated quote"));
        }
        flush(&mut items, &mut last);
        Ok(SimpleDateFormat { items })
    }

    /// `format(date)`.
    pub fn format(&self, date: &Date) -> String {
        let (day, d, ms) = date.fields();
        let hour = ms / 3_600_000;
        // `ZONE_OFFSET + DST_OFFSET` of the default zone, in seconds.
        let offset = super::default_zone().util_offset_at(date.get_time().div_euclid(1000)).0 as i64;
        let dow = super::day_of_week(day) as i64; // ISO, Monday 1
        let mut out = String::new();
        for item in &self.items {
            let (c, count) = match item {
                Item::Literal(s) => {
                    out.push_str(s);
                    continue;
                }
                Item::Field(c, n) => (*c, *n),
            };
            let number = |out: &mut String, value: i64| out.push_str(&padded(value, count, usize::MAX));
            match c {
                'G' => out.push_str(if d.era == 1 { "AD" } else { "BC" }),
                'y' | 'Y' => {
                    let year = if c == 'y' { d.year_of_era } else { week_year(day, &d) };
                    out.push_str(&if count == 2 { padded(year, 2, 2) } else { padded(year, count, usize::MAX) });
                }
                'M' | 'L' => match count {
                    4.. => out.push_str(MONTHS[d.month as usize - 1]),
                    3 => out.push_str(&MONTHS[d.month as usize - 1][..3]),
                    _ => number(&mut out, d.month as i64),
                },
                'E' => {
                    let name = WEEKDAYS[dow as usize - 1];
                    out.push_str(if count >= 4 { name } else { &name[..3] });
                }
                'a' => out.push_str(if hour < 12 { "AM" } else { "PM" }),
                'k' => number(&mut out, if hour == 0 { 24 } else { hour }),
                'h' => number(&mut out, if hour % 12 == 0 { 12 } else { hour % 12 }),
                'K' => number(&mut out, hour % 12),
                'H' => number(&mut out, hour),
                'm' => number(&mut out, ms / 60_000 % 60),
                's' => number(&mut out, ms / 1000 % 60),
                'S' => number(&mut out, ms % 1000),
                'd' => number(&mut out, d.day as i64),
                'D' => number(&mut out, d.day_of_year(day)),
                'F' => number(&mut out, (d.day as i64 - 1) / 7 + 1),
                'w' => number(&mut out, week_of_year(day, &d)),
                'W' => {
                    let dom = d.day as i64;
                    number(&mut out, compute_week(start_of_week_offset(dom, sunday_dow(dow)), dom))
                }
                'u' => number(&mut out, dow),
                'z' => out.push_str(&date.zone_name(count >= 4)),
                'Z' => {
                    let minutes = offset / 60;
                    out.push(if minutes < 0 { '-' } else { '+' });
                    let m = minutes.abs();
                    out.push_str(&format!("{:04}", m / 60 * 100 + m % 60));
                }
                // 'X'
                _ => {
                    let minutes = offset / 60;
                    if minutes == 0 {
                        out.push('Z');
                    } else {
                        out.push(if minutes < 0 { '-' } else { '+' });
                        let m = minutes.abs();
                        out.push_str(&format!("{:02}", m / 60));
                        if count >= 2 {
                            out.push_str(&format!("{}{:02}", if count == 3 { ":" } else { "" }, m % 60));
                        }
                    }
                }
            }
        }
        out
    }
}

/// `zeroPaddingNumber(value, min, max)`: `DecimalFormat` without grouping, keeping the last `max` digits.
fn padded(value: i64, min: usize, max: usize) -> String {
    let digits = value.unsigned_abs().to_string();
    let digits = if digits.len() > max { &digits[digits.len() - max..] } else { &digits };
    let sign = if value < 0 { "-" } else { "" };
    format!("{sign}{digits:0>min$}")
}

/// Day of week counted from Sunday (1) to Saturday (7).
fn sunday_dow(iso_dow: i64) -> i64 {
    iso_dow % 7 + 1
}

fn start_of_week_offset(day: i64, dow: i64) -> i64 {
    let week_start = (day - dow).rem_euclid(7);
    if week_start + 1 > 1 { 7 - week_start } else { -week_start }
}

fn compute_week(offset: i64, day: i64) -> i64 {
    (7 + offset + (day - 1)) / 7
}

fn year_length(d: &HybridDate) -> i64 {
    365 + i64::from(d.is_leap())
}

/// `WEEK_OF_YEAR`: the week containing January 1st is week 1, also for the last days of December.
fn week_of_year(day: i64, d: &HybridDate) -> i64 {
    let doy = d.day_of_year(day);
    let offset = start_of_week_offset(doy, sunday_dow(super::day_of_week(day) as i64));
    let week = compute_week(offset, doy);
    let next_year_week = compute_week(offset, year_length(d) + 1);
    if week >= next_year_week { week - next_year_week + 1 } else { week }
}

/// `getWeekYear()`: the proleptic year (`1 - year` of era before Christ), plus one in the first week of the next.
fn week_year(day: i64, d: &HybridDate) -> i64 {
    let doy = d.day_of_year(day);
    let offset = start_of_week_offset(doy, sunday_dow(super::day_of_week(day) as i64));
    let week = compute_week(offset, doy);
    let year = d.year;
    if week >= compute_week(offset, year_length(d) + 1) { year + 1 } else { year }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(pattern: &str, millis: i64) -> String {
        match SimpleDateFormat::new(pattern) {
            Ok(f) => f.format(&Date::new(millis)),
            Err(e) => format!("ERR {}: {}", e.simple_class(), e.message()),
        }
    }

    #[test]
    fn patterns() {
        let t = 1_700_000_000_123;
        assert_eq!(fmt("yyyy MM.dd mm:hh:ss", t), "2023 11.14 13:10:20");
        assert_eq!(fmt("yy DDD hh:mm:ss", t), "23 318 10:13:20");
        assert_eq!(fmt("EEEE MMMM d G a k K S z Z X", t), "Tuesday November 14 AD PM 22 10 123 GMT +0000 Z");
        assert_eq!(fmt("'o''clock' '' yyyyy", t), "o'clock ' 02023");
        assert_eq!(fmt("w W F u Y", t), "46 3 2 2 2023");
        assert_eq!(fmt("b", t), "ERR IllegalArgumentException: Illegal pattern character 'b'");
        assert_eq!(fmt("'abc", t), "ERR IllegalArgumentException: Unterminated quote");
    }
}
