//! `java.time.Duration`, as far as Datafaker builds and prints one.

use std::fmt;

use crate::{Error, Result};

/// A time-based amount: seconds and nanoseconds (`0..1_000_000_000`), as `java.time.Duration` stores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Duration {
    seconds: i64,
    nanos: i32,
}

impl Duration {
    /// `Duration.ZERO`.
    pub const ZERO: Duration = Duration { seconds: 0, nanos: 0 };

    fn of_scaled(amount: i64, seconds_per_unit: i64) -> Result<Duration> {
        let seconds = amount.checked_mul(seconds_per_unit).ok_or_else(|| Error::new("ArithmeticException", "long overflow"))?;
        Ok(Duration { seconds, nanos: 0 })
    }

    /// `Duration.ofSeconds(seconds)`.
    pub fn of_seconds(seconds: i64) -> Duration {
        Duration { seconds, nanos: 0 }
    }

    /// `Duration.ofMinutes(minutes)`.
    pub fn of_minutes(minutes: i64) -> Result<Duration> {
        Duration::of_scaled(minutes, 60)
    }

    /// `Duration.ofHours(hours)`.
    pub fn of_hours(hours: i64) -> Result<Duration> {
        Duration::of_scaled(hours, 3600)
    }

    /// `Duration.ofDays(days)`.
    pub fn of_days(days: i64) -> Result<Duration> {
        Duration::of_scaled(days, 86400)
    }

    /// `Duration.ofSeconds(seconds, nanoAdjustment)`.
    pub fn of_seconds_and_nanos(seconds: i64, nano_adjustment: i64) -> Result<Duration> {
        let overflow = || Error::new("ArithmeticException", "long overflow");
        let seconds = seconds.checked_add(nano_adjustment.div_euclid(1_000_000_000)).ok_or_else(overflow)?;
        Ok(Duration { seconds, nanos: nano_adjustment.rem_euclid(1_000_000_000) as i32 })
    }

    /// `Duration.getSeconds()`.
    pub fn seconds(&self) -> i64 {
        self.seconds
    }

    /// `Duration.getNano()`: the nanoseconds within the second.
    pub fn nanos(&self) -> i32 {
        self.nanos
    }
}

impl fmt::Display for Duration {
    /// `Duration.toString()`: ISO-8601 seconds-based, `PT8H6M12.345S`, `PT0S`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Duration::ZERO {
            return f.write_str("PT0S");
        }
        let mut total = self.seconds;
        if self.seconds < 0 && self.nanos > 0 {
            total += 1;
        }
        let hours = total / 3600;
        let minutes = (total % 3600) / 60;
        let secs = total % 60;
        let mut buf = String::from("PT");
        if hours != 0 {
            buf.push_str(&format!("{hours}H"));
        }
        if minutes != 0 {
            buf.push_str(&format!("{minutes}M"));
        }
        if secs == 0 && self.nanos == 0 && buf.len() > 2 {
            return f.write_str(&buf);
        }
        if self.seconds < 0 && self.nanos > 0 {
            buf.push_str(if secs == 0 { "-0" } else { "" });
            if secs != 0 {
                buf.push_str(&secs.to_string());
            }
        } else {
            buf.push_str(&secs.to_string());
        }
        if self.nanos > 0 {
            let fraction = if self.seconds < 0 { 2_000_000_000 - self.nanos as i64 } else { self.nanos as i64 + 1_000_000_000 };
            let digits = fraction.to_string();
            buf.push('.');
            buf.push_str(digits[1..].trim_end_matches('0'));
        }
        buf.push('S');
        f.write_str(&buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        assert_eq!(Duration::ZERO.to_string(), "PT0S");
        assert_eq!(Duration::of_seconds(55).to_string(), "PT55S");
        assert_eq!(Duration::of_minutes(45).unwrap().to_string(), "PT45M");
        assert_eq!(Duration::of_days(2).unwrap().to_string(), "PT48H");
        assert_eq!(Duration::of_seconds(3725).to_string(), "PT1H2M5S");
        assert_eq!(Duration::of_seconds_and_nanos(12, 345_000_000).unwrap().to_string(), "PT12.345S");
        assert_eq!(Duration::of_seconds_and_nanos(-1, 500_000_000).unwrap().to_string(), "PT-0.5S");
        assert_eq!(Duration::of_seconds(-61).to_string(), "PT-1M-1S");
        assert!(Duration::of_days(i64::MAX).is_err());
    }
}
