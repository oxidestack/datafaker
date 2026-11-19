//! `TimeAndDateTest`, `DateAndTimeTest`, `TimeTest`, `TwitterTest` and BusinessTest's `creditCardExpiry` (its other
//! cases are generated), Datafaker 2.7.0, `providers/base`. Unseeded fakers as upstream; repeated tests run
//! their body as often; parameterized tests loop over their arguments.
//!
//! `assertValidDate`/`assertValidDateTime`/`assertValidTime` parse the text back with `DateTimeFormatter.ofPattern`
//! as upstream does.

mod common;

use common::assert_matches;
use datafaker::Faker;
use datafaker::java::{ChronoUnit, Date, DateTimeFormatter, Duration, Instant, LocalDate, LocalDateTime, LocalTime, Period, TimeUnit};

/// `TimeAndDateTest.assertValidDate`: `DateTimeFormatter.ofPattern(pattern).parse(date).query(LocalDate::from)`
/// does not throw.
fn assert_valid_date(text: &str, pattern: &str) {
    let parsed = DateTimeFormatter::of_pattern(pattern).unwrap().parse(text).unwrap_or_else(|e| panic!("{e}"));
    LocalDate::from_parsed(&parsed).unwrap_or_else(|e| panic!("{e}"));
}

/// `DateAndTimeTest.assertValidDate`/`assertValidDateTime`: `formatter.parse(date)` is not null.
fn assert_parses(text: &str, pattern: &str) {
    DateTimeFormatter::of_pattern(pattern).unwrap().parse(text).unwrap_or_else(|e| panic!("{e}"));
}

/// `TimeTest.assertValidTime`: `DateTimeFormatter.ofPattern(pattern).parse(date).query(LocalTime::from)` does not
/// throw.
fn assert_valid_time(text: &str, pattern: &str) {
    let parsed = DateTimeFormatter::of_pattern(pattern).unwrap().parse(text).unwrap_or_else(|e| panic!("{e}"));
    LocalTime::from_parsed(&parsed).unwrap_or_else(|e| panic!("{e}"));
}

// ---------------------------------------------------------------------------------------------------------------
// TimeAndDateTest

fn instant_millis(i: Instant) -> i64 {
    i.to_epoch_milli().unwrap()
}

/// `testFutureDate` (repeated 100 times).
#[test]
fn time_and_date_future_date() {
    let t = Faker::new();
    for _ in 0..100 {
        assert!(t.time_and_date().future().unwrap().is_after(&Instant::now()));
    }
}

/// `testFutureDateWithBounds` (repeated 100 times).
#[test]
fn time_and_date_future_date_with_bounds() {
    let f = Faker::new();
    for _ in 0..100 {
        let now = Instant::now();
        let future = f.time_and_date().future_within(1, TimeUnit::Seconds).unwrap();
        assert!(future >= now && future <= now.plus_seconds(1).unwrap(), "{future} {now}");
    }
}

/// `testFutureDateWithBoundsFromGivenMoment` (repeated 100 times).
#[test]
fn time_and_date_future_date_with_bounds_from_given_moment() {
    let f = Faker::new();
    for _ in 0..100 {
        let moment = Instant::now().minus(10, ChronoUnit::Days).unwrap();
        let future = f.time_and_date().future_from(15, TimeUnit::Minutes, moment).unwrap();
        assert!(future >= moment && future <= moment.plus(15, ChronoUnit::Minutes).unwrap());
    }
}

/// `testFutureDateWithMinimum` (repeated 100 times).
#[test]
fn time_and_date_future_date_with_minimum() {
    let f = Faker::new();
    for _ in 0..100 {
        let now = Instant::now();
        let future = f.time_and_date().future_between(5, 4, TimeUnit::Seconds).unwrap();
        assert!(future >= now.plus_millis(3500).unwrap() && future <= now.plus_millis(5500).unwrap());
    }
}

/// `testPastDateWithMinimum` (repeated 100 times).
#[test]
fn time_and_date_past_date_with_minimum() {
    let f = Faker::new();
    for _ in 0..100 {
        let now = instant_millis(Instant::now());
        let past = instant_millis(f.time_and_date().past_between(5, 4, TimeUnit::Seconds).unwrap());
        assert!(past < now && past > now - 5500 && past < now - 3500);
    }
}

/// `testPastDateWithReferenceDate` (repeated 100 times).
#[test]
fn time_and_date_past_date_with_reference_date() {
    let f = Faker::new();
    for _ in 0..100 {
        let now = Instant::now();
        let past = instant_millis(f.time_and_date().past_from(1, TimeUnit::Seconds, now).unwrap());
        assert!(past < instant_millis(now) && past > instant_millis(now) - 1000);
    }
}

/// `testPastDate` (repeated 100 times).
#[test]
fn time_and_date_past_date() {
    let f = Faker::new();
    for _ in 0..100 {
        assert!(f.time_and_date().past().unwrap().is_before(&Instant::now()));
    }
}

/// `testPastDateWithBounds` (repeated 100 times).
#[test]
fn time_and_date_past_date_with_bounds() {
    let f = Faker::new();
    for _ in 0..100 {
        let now = Instant::now();
        let past = f.time_and_date().past_within(100, TimeUnit::Seconds).unwrap();
        assert!(instant_millis(past) < instant_millis(now));
    }
}

/// `testBetween` (repeated 100 times).
#[test]
fn time_and_date_between() {
    let f = Faker::new();
    for _ in 0..100 {
        let now = Instant::now();
        let then = Instant::now().plus_millis(1000).unwrap();
        let date = instant_millis(f.time_and_date().between(now, then).unwrap());
        assert!(date < instant_millis(then) && date >= instant_millis(now));
    }
}

/// `testBetweenWithMaskReturningString`.
#[test]
fn time_and_date_between_with_mask_returning_string() {
    let now = Instant::now();
    let then = Instant::now().plus_millis(1000).unwrap();
    let pattern = "yyyy MM.dd mm:hh:ss";
    assert_valid_date(&Faker::new().time_and_date().between_formatted(now, then, pattern).unwrap(), pattern);
}

/// `testBetweenThenLargerThanNow`.
#[test]
fn time_and_date_between_then_larger_than_now() {
    let now = Instant::now();
    let then = Instant::now().plus_millis(1000).unwrap();
    let e = Faker::new().time_and_date().between(then, now).unwrap_err();
    assert_eq!(e.simple_class(), "IllegalArgumentException");
    assert_eq!(e.message(), format!("Invalid date range: the upper bound date ({now}) is before the lower bound ({then})"));
}

/// `testBirthday` (repeated 100 times).
#[test]
fn time_and_date_birthday() {
    let f = Faker::new();
    for _ in 0..100 {
        let now = LocalDateTime::now().date();
        let from = now.minus_years(18).unwrap();
        let to = now.minus_years(65).unwrap();
        let birthday = f.time_and_date().birthday().unwrap();
        assert!(birthday >= to && birthday <= from);
    }
}

/// `testBirthdayWithAges` (repeated 100 times).
#[test]
fn time_and_date_birthday_with_ages() {
    let f = Faker::new();
    for _ in 0..100 {
        let nw = LocalDate::now();
        let min_age = f.number().number_between(1, 99);
        let max_age = f.number().number_between(min_age, 100);
        let from = nw.minus_years(max_age as i64).unwrap();
        let to = nw.minus_years(min_age as i64).unwrap();
        let birthday = f.time_and_date().birthday_between(min_age, max_age).unwrap();
        assert!(birthday >= from && birthday <= to);
    }
}

/// `birthdayWithMask`.
#[test]
fn time_and_date_birthday_with_mask() {
    let pattern = "yyyy MM.dd";
    assert_valid_date(&Faker::new().time_and_date().birthday_between_formatted(1, 50, pattern).unwrap(), pattern);
}

/// `futureWithMask`.
#[test]
fn time_and_date_future_with_mask() {
    let t = Faker::new();
    let t = t.time_and_date();
    let pattern = "yyyy MM.dd mm:hh:ss";
    assert_valid_date(&t.future_within_formatted(1, TimeUnit::Hours, pattern).unwrap(), pattern);
    assert_valid_date(&t.future_between_formatted(20, 1, TimeUnit::Hours, pattern).unwrap(), pattern);
    assert_valid_date(&t.future_from_formatted(20, TimeUnit::Hours, Instant::now(), pattern).unwrap(), pattern);
    assert_valid_date(&t.future_from_formatted(900, TimeUnit::Days, Instant::now(), pattern).unwrap(), pattern);
}

/// `pastWithMask`.
#[test]
fn time_and_date_past_with_mask() {
    let t = Faker::new();
    let t = t.time_and_date();
    let pattern = "yyyy MM.dd mm:hh:ss";
    assert_valid_date(&t.past_within_formatted(1, TimeUnit::Days, pattern).unwrap(), pattern);
    assert_valid_date(&t.past_between_formatted(20, 1, TimeUnit::Days, pattern).unwrap(), pattern);
    assert_valid_date(&t.past_from_formatted(1, TimeUnit::Days, Instant::now(), pattern).unwrap(), pattern);
}

fn period_days(p: Period) -> i32 {
    (p.years() * 12 + p.months()) * 30 + p.days()
}

/// `periodTest`.
#[test]
fn time_and_date_period() {
    let max = Period::of(3, 2, 1);
    let min = Period::of(2, 1, 0);
    let period = Faker::new().time_and_date().period(min, max).unwrap();
    assert!((period_days(min)..=period_days(max)).contains(&period_days(period)));
}

/// `durationTest(long, long, ChronoUnit)` over `generateDurationsWithMinMax`.
#[test]
fn time_and_date_duration_min_max() {
    let f = Faker::new();
    let cases = [
        (123, 123, ChronoUnit::Days),
        (12, 123, ChronoUnit::Hours),
        (15, 400, ChronoUnit::Minutes),
        (65, 98, ChronoUnit::Seconds),
        (76, 100, ChronoUnit::Millis),
        (879, 1030, ChronoUnit::Micros),
        (879, 1030, ChronoUnit::Nanos),
        (0, i64::MAX, ChronoUnit::Nanos),
        (i64::MIN, 0, ChronoUnit::Nanos),
        (i64::MAX - 1, i64::MAX, ChronoUnit::Nanos),
        (i64::MIN, i64::MAX, ChronoUnit::Nanos),
    ];
    for (min_value, max_value, unit) in cases {
        let generated = f.time_and_date().duration_between(min_value, max_value, unit).unwrap();
        let min = Duration::of(min_value, unit).unwrap();
        let max = Duration::of(max_value, unit).unwrap();
        assert!(generated >= min);
        if min_value >= max_value {
            assert_eq!(generated, max, "Duration ({generated}) must be equal to max value if min ({min}) == max ({max})");
        } else {
            assert!(generated < max, "Duration ({generated}) must be lower than max value ({max}) (min value: {min})");
        }
    }
}

/// `durationTest(long, ChronoUnit)` over `generateDurationsWithMaxOnly`.
#[test]
fn time_and_date_duration_max_only() {
    let f = Faker::new();
    let cases = [
        (100, ChronoUnit::Days),
        (456, ChronoUnit::Hours),
        (43, ChronoUnit::Minutes),
        (78, ChronoUnit::Seconds),
        (786, ChronoUnit::Millis),
        (786, ChronoUnit::Micros),
        (8729, ChronoUnit::Nanos),
    ];
    for (max_value, unit) in cases {
        let generated = f.time_and_date().duration(max_value, unit).unwrap();
        let max = Duration::of(max_value, unit).unwrap();
        assert!(generated < max, "Duration ({generated}) must be lower than max value ({max_value} {unit})");
    }
}

/// `durationIsZero_ifMaxIsZero`.
#[test]
fn time_and_date_duration_is_zero_if_max_is_zero() {
    assert_eq!(Faker::new().time_and_date().duration(0, ChronoUnit::Days).unwrap(), Duration::of(0, ChronoUnit::Days).unwrap());
}

fn bad_periods() -> [(Period, Period); 3] {
    [
        (Period::of(1, 1, 1), Period::of(0, 1, 1)),
        (Period::of(1, 1, 3), Period::of(1, 1, 2)),
        (Period::of(1, 2, 1), Period::of(1, 1, 1)),
    ]
}

/// `maxLessThanMinPeriod` over `generatePeriod`.
#[test]
fn time_and_date_max_less_than_min_period() {
    for (min, max) in bad_periods() {
        assert_eq!(Faker::new().time_and_date().period(min, max).unwrap_err().simple_class(), "IllegalArgumentException");
    }
}

// ---------------------------------------------------------------------------------------------------------------
// DateAndTimeTest

const DATE_PATTERN: &str = "yyyy MM.dd";
const DATE_TIME_PATTERN: &str = "yyyy MM.dd mm:hh:ss";

fn now_millis() -> i64 {
    instant_millis(Instant::now())
}

/// `testFutureDate`.
#[test]
fn date_and_time_future_date() {
    let now = Date::timestamp(now_millis());
    let f = Faker::new();
    for _ in 0..1000 {
        let future = f.date().future_from(1, TimeUnit::Seconds, now).unwrap();
        assert!(future.get_time() > now.get_time() && future.get_time() < now.get_time() + 1000);
    }
}

/// `testFutureDateWithMinimum`.
#[test]
fn date_and_time_future_date_with_minimum() {
    let now = Date::now();
    let f = Faker::new();
    for _ in 0..1000 {
        let future = f.date().future_between(5, 4, TimeUnit::Seconds).unwrap().get_time();
        assert!(future > now.get_time() && future < now.get_time() + 5500 && future > now.get_time() + 3500);
    }
}

/// `testPastDateWithMinimum`.
#[test]
fn date_and_time_past_date_with_minimum() {
    let f = Faker::new();
    for _ in 0..1000 {
        let now = now_millis();
        let past = f.date().past_between(5, 4, TimeUnit::Seconds).unwrap().get_time();
        assert!(past < now && past > now - 5500 && past < now - 3500);
    }
}

/// `testPastDateWithReferenceDate`.
#[test]
fn date_and_time_past_date_with_reference_date() {
    let now = Date::now();
    let f = Faker::new();
    for _ in 0..1000 {
        let past = f.date().past_from(1, TimeUnit::Seconds, now).unwrap().get_time();
        assert!(past < now.get_time() && past > now.get_time() - 1000);
    }
}

/// `testPastDate`.
#[test]
fn date_and_time_past_date() {
    let now = Date::now();
    let past = Faker::new().date().past(100, TimeUnit::Seconds).unwrap();
    assert!(past.get_time() < now.get_time());
}

/// `testBetween`.
#[test]
fn date_and_time_between() {
    let now = Date::timestamp(now_millis());
    let then = Date::timestamp(now_millis() + 1000);
    let f = Faker::new();
    for _ in 0..1000 {
        let date = f.date().between(now, then).unwrap().get_time();
        assert!(date < then.get_time() && date >= now.get_time());
    }
}

/// `testBetweenWithMaskReturningString`.
#[test]
fn date_and_time_between_with_mask_returning_string() {
    let now = Date::timestamp(now_millis());
    let then = Date::timestamp(now_millis() + 1000);
    assert_parses(&Faker::new().date().between_formatted(now, then, DATE_TIME_PATTERN).unwrap(), DATE_TIME_PATTERN);
}

/// `testBetweenDateAsArgument`: two `new Date()`s (equal, or one millisecond apart).
#[test]
fn date_and_time_between_date_as_argument() {
    let (a, b) = (Date::now(), Date::now());
    Faker::new().date().between(a, b).unwrap();
}

/// `testBetweenThenLargerThanNow`.
#[test]
fn date_and_time_between_then_larger_than_now() {
    let now = Date::timestamp(now_millis());
    let then = Date::timestamp(now_millis() + 1000);
    let e = Faker::new().date().between(then, now).unwrap_err();
    assert_eq!(e.simple_class(), "IllegalArgumentException");
    assert_eq!(e.message(), format!("Invalid date range: the upper bound date ({now}) is before the lower bound ({then})"));
}

/// Midnight of `date`, in milliseconds (the default zone is UTC).
fn midnight_millis(date: LocalDate) -> i64 {
    date.to_epoch_day() * 86_400_000
}

/// `testBirthday`.
#[test]
fn date_and_time_birthday() {
    let today = LocalDateTime::now().date();
    let to = midnight_millis(today.minus_years(18).unwrap());
    let from = midnight_millis(today.minus_years(65).unwrap());
    let f = Faker::new();
    for _ in 0..5000 {
        let birthday = f.date().birthday().unwrap().get_time();
        assert!(birthday < to && birthday >= from);
    }
}

/// `testBirthdayWithAges`.
#[test]
fn date_and_time_birthday_with_ages() {
    let nw = LocalDate::now();
    let f = Faker::new();
    for _ in 0..20 {
        let min_age = f.number().number_between(1, 99);
        let max_age = f.number().number_between(min_age, 100);
        let from = midnight_millis(nw.minus_years(max_age as i64).unwrap());
        let to = midnight_millis(nw.minus_years(min_age as i64).unwrap());
        let birthday = f.date().birthday_between(min_age, max_age).unwrap().get_time();
        assert!(birthday >= from && birthday <= to);
    }
}

/// `birthdayWithMask`.
#[test]
fn date_and_time_birthday_with_mask() {
    assert_parses(&Faker::new().date().birthday_between_formatted(1, 50, DATE_PATTERN).unwrap(), DATE_PATTERN);
}

/// `futureWithMask`.
#[test]
fn date_and_time_future_with_mask() {
    let f = Faker::new();
    let d = f.date();
    assert_parses(&d.future_formatted(1, TimeUnit::Hours, DATE_TIME_PATTERN).unwrap(), DATE_TIME_PATTERN);
    assert_parses(&d.future_between_formatted(20, 1, TimeUnit::Hours, DATE_TIME_PATTERN).unwrap(), DATE_TIME_PATTERN);
    assert_parses(&d.future_from_formatted(20, TimeUnit::Hours, Date::now(), DATE_TIME_PATTERN).unwrap(), DATE_TIME_PATTERN);
}

/// `pastWithMask`.
#[test]
fn date_and_time_past_with_mask() {
    let f = Faker::new();
    let d = f.date();
    assert_parses(&d.past_formatted(1, TimeUnit::Days, DATE_TIME_PATTERN).unwrap(), DATE_TIME_PATTERN);
    assert_parses(&d.past_between_formatted(20, 1, TimeUnit::Days, DATE_TIME_PATTERN).unwrap(), DATE_TIME_PATTERN);
    assert_parses(&d.past_from_formatted(1, TimeUnit::Days, Date::now(), DATE_TIME_PATTERN).unwrap(), DATE_TIME_PATTERN);
}

/// `periodTest`.
#[test]
fn date_and_time_period() {
    let max = Period::of(3, 2, 1);
    let min = Period::of(2, 1, 0);
    let period = Faker::new().date().period(min, max).unwrap();
    assert!((period_days(min)..=period_days(max)).contains(&period_days(period)));
}

/// `invalidDuration` over `"null"`, `""`, `"month"`, `"year"`, `"week"`.
#[test]
fn date_and_time_invalid_duration() {
    let f = Faker::new();
    for invalid in ["null", "", "month", "year", "week"] {
        let e = f.date().duration_named(f.random().next_long(), Some(invalid)).unwrap_err();
        assert_eq!(e.simple_class(), "IllegalArgumentException");
    }
}

const DURATION_UNITS: [ChronoUnit; 7] = [
    ChronoUnit::Nanos,
    ChronoUnit::Micros,
    ChronoUnit::Millis,
    ChronoUnit::Seconds,
    ChronoUnit::Minutes,
    ChronoUnit::Hours,
    ChronoUnit::Days,
];

/// `durationTest(long, long, ChronoUnit)` over `generateDurationsWithMinMax`.
#[test]
fn date_and_time_duration_min_max() {
    let f = Faker::new();
    let cases = [
        (12, 123, ChronoUnit::Hours),
        (15, 400, ChronoUnit::Minutes),
        (65, 98, ChronoUnit::Seconds),
        (76, 100, ChronoUnit::Millis),
        (879, 1030, ChronoUnit::Micros),
        (879, 1030, ChronoUnit::Nanos),
    ];
    for (min_value, max_value, unit) in cases {
        let generated = f.date().duration_between(min_value, max_value, unit).unwrap();
        assert!(generated >= Duration::of(min_value, unit).unwrap(), "Duration must be equal or greater than min value");
        assert!(generated < Duration::of(max_value, unit).unwrap(), "Duration must be lower than max value");
    }
}

/// `durationTest_equalMinAndMax` over `durationUnits`.
#[test]
fn date_and_time_duration_equal_min_and_max() {
    let f = Faker::new();
    for unit in DURATION_UNITS {
        assert_eq!(f.date().duration_between(123, 123, unit).unwrap(), Duration::of(123, unit).unwrap());
    }
}

/// `durationTest(long, ChronoUnit)` over `generateDurationsWithMaxOnly`.
#[test]
fn date_and_time_duration_max_only() {
    let f = Faker::new();
    let cases = [
        (100, ChronoUnit::Days),
        (456, ChronoUnit::Hours),
        (43, ChronoUnit::Minutes),
        (78, ChronoUnit::Seconds),
        (786, ChronoUnit::Millis),
        (786, ChronoUnit::Micros),
        (8729, ChronoUnit::Nanos),
    ];
    for (max_value, unit) in cases {
        assert!(f.date().duration(max_value, unit).unwrap() < Duration::of(max_value, unit).unwrap());
    }
}

/// `durationTest_zero` over `durationUnits`.
#[test]
fn date_and_time_duration_zero() {
    let f = Faker::new();
    for unit in DURATION_UNITS {
        assert_eq!(f.date().duration(0, unit).unwrap(), Duration::ZERO);
        assert_eq!(f.date().duration_between(0, 0, unit).unwrap(), Duration::ZERO);
        assert_eq!(f.date().duration(1, unit).unwrap(), Duration::ZERO);
    }
}

/// `durationTestUnitAsString` over `generateDurationsFromStringWithMinMax`.
#[test]
fn date_and_time_duration_unit_as_string() {
    let f = Faker::new();
    let cases = [
        (12, 123, "days"),
        (21, 32, "DAY"),
        (45, 100, "HOUR"),
        (23, 100, "hours"),
        (15, 400, "minutes"),
        (14, 500, "minute"),
        (32, 54, "seconds"),
        (65, 98, "second"),
        (76, 100, "millis"),
        (87, 100, "milli"),
        (76, 100, "micros"),
        (87, 100, "micro"),
        (874, 1300, "nano"),
        (879, 1030, "nanos"),
    ];
    for (min_value, max_value, unit) in cases {
        let generated = f.date().duration_between_named(min_value, max_value, Some(unit)).unwrap();
        let u = datafaker::providers::str2duration_unit(Some(unit)).unwrap();
        assert!(generated >= Duration::of(min_value, u).unwrap(), "Duration must be >= min value");
        assert!(generated < Duration::of(max_value, u).unwrap(), "Duration must be < max value");
    }
}

/// `durationTestUnitAsString_equalMinAndMax`.
#[test]
fn date_and_time_duration_unit_as_string_equal_min_and_max() {
    assert_eq!(Faker::new().date().duration_between_named(123, 123, Some("days")).unwrap(), Duration::of(123, ChronoUnit::Days).unwrap());
}

/// `durationTest(long, String)` over `generateDurationsFromStringWithMaxOnly`.
#[test]
fn date_and_time_duration_unit_as_string_max_only() {
    let f = Faker::new();
    let cases = [
        (0, "days"),
        (100, "days"),
        (123, "DAY"),
        (456, "HOUR"),
        (1234, "hours"),
        (43, "minutes"),
        (78, "minute"),
        (56, "seconds"),
        (34, "second"),
        (786, "millis"),
        (879, "milli"),
        (8729, "nano"),
        (8739, "nanos"),
    ];
    for (max_value, unit) in cases {
        let generated = f.date().duration_named(max_value, Some(unit)).unwrap();
        let max = Duration::of(max_value, datafaker::providers::str2duration_unit(Some(unit)).unwrap()).unwrap();
        if max_value == 0 {
            assert_eq!(generated, Duration::of(0, ChronoUnit::Millis).unwrap());
        } else {
            assert!(generated < max);
        }
    }
}

/// `maxLessThanMinPeriod` over `generatePeriod`.
#[test]
fn date_and_time_max_less_than_min_period() {
    for (min, max) in bad_periods() {
        assert_eq!(Faker::new().date().period(min, max).unwrap_err().simple_class(), "IllegalArgumentException");
    }
}

// ---------------------------------------------------------------------------------------------------------------
// TimeTest

fn time_of(nanos: i64) -> LocalTime {
    LocalTime::of_nano_of_day(nanos).unwrap()
}

/// `testFutureTime`.
#[test]
fn time_future_time() {
    let now = LocalTime::now();
    let f = Faker::new();
    for _ in 0..1000 {
        let future = time_of(f.time().future(1, ChronoUnit::Seconds).unwrap());
        assert!(future.is_after(&now) && future.is_before(&now.plus_seconds(10)));
    }
}

/// `testFutureTimeWithMinimum`.
#[test]
fn time_future_time_with_minimum() {
    let now = LocalTime::now();
    let f = Faker::new();
    for _ in 0..1000 {
        let future = time_of(f.time().future_between(5, 4, ChronoUnit::Seconds).unwrap());
        assert!(future.is_after(&now) && future.is_before(&now.plus_seconds(10)) && future.is_after(&now.plus_seconds(1)));
    }
}

/// `testPastTimeWithMinimum`.
#[test]
fn time_past_time_with_minimum() {
    let now = LocalTime::now();
    let f = Faker::new();
    for _ in 0..1000 {
        let past = time_of(f.time().past_between(5, 4, ChronoUnit::Seconds).unwrap());
        assert!(past.is_before(&now) && past.is_after(&now.minus_seconds(6)) && past.is_before(&now.minus_seconds(2)));
    }
}

/// `testPastTime`.
#[test]
fn time_past_time() {
    let now = LocalTime::now();
    let past = time_of(Faker::new().time().past(100, ChronoUnit::Seconds).unwrap());
    assert!(past.is_before(&now));
}

/// `testBetween`.
#[test]
fn time_between() {
    let now = LocalTime::now();
    let then = now.plus_seconds(1);
    let f = Faker::new();
    for _ in 0..1000 {
        let time = time_of(f.time().between(now, then).unwrap());
        assert!(time.is_before(&then) && !time.is_before(&now));
    }
}

/// `testBetweenWithSameLocalTime`.
#[test]
fn time_between_with_same_local_time() {
    let now = LocalTime::now();
    assert_eq!(time_of(Faker::new().time().between(now, now).unwrap()), now);
}

/// `testBetweenThenLargerThanNow`.
#[test]
fn time_between_then_larger_than_now() {
    let now = LocalTime::now();
    let then = now.plus_seconds(1);
    let e = Faker::new().time().between(then, now).unwrap_err();
    assert_eq!(e.simple_class(), "IllegalArgumentException");
    assert_eq!(e.message(), format!("Invalid time range: the upper bound time ({now}) is before the lower bound ({then})"));
}

/// `testBetweenWithMask` (repeated 10 times).
#[test]
fn time_between_with_mask() {
    let f = Faker::new();
    for _ in 0..10 {
        let pattern = "HH:mm:ss";
        let moment = f.random().next_long_between(0, 86_400 - 60).unwrap();
        let now = LocalTime::of_second_of_day(moment).unwrap();
        let then = now.plus_minutes(1);
        let result = f.time().between_formatted(now, then, pattern).unwrap();
        assert_matches("[0-2][0-9]:[0-5][0-9]:[0-5][0-9]", &result);
        let parsed = DateTimeFormatter::of_pattern(pattern).unwrap().parse(&result).unwrap();
        let parsed = LocalTime::from_parsed(&parsed).unwrap();
        assert!(!parsed.is_before(&now) && !parsed.is_after(&then));
    }
}

/// `futureWithMask`.
#[test]
fn time_future_with_mask() {
    let pattern = "mm:HH:ss";
    let f = Faker::new();
    assert_valid_time(&f.time().future_formatted(1, ChronoUnit::Hours, pattern).unwrap(), pattern);
    assert_valid_time(&f.time().future_between_formatted(20, 2, ChronoUnit::Hours, pattern).unwrap(), pattern);
}

/// `pastWithMask`.
#[test]
fn time_past_with_mask() {
    let pattern = "mm:HH:ss";
    let f = Faker::new();
    assert_valid_time(&f.time().past_formatted(1, ChronoUnit::Minutes, pattern).unwrap(), pattern);
    assert_valid_time(&f.time().past_between_formatted(20, 1, ChronoUnit::Millis, pattern).unwrap(), pattern);
}

// ---------------------------------------------------------------------------------------------------------------
// BusinessTest

/// `creditCardExpiry`.
#[test]
fn business_credit_card_expiry() {
    let date = Faker::new().business().credit_card_expiry().unwrap();
    assert!(!date.is_empty());
    assert_matches("\\d{4}-\\d{2}-\\d{2}", &date);
}

// ---------------------------------------------------------------------------------------------------------------
// TwitterTest

/// `testCreatedDateForward`.
#[test]
fn twitter_created_date_forward() {
    let test_date = Date::now();
    let constrain_date = Date::new(test_date.get_time() + 3_000_000);
    let generated = Faker::new().twitter().created_time(true, test_date, constrain_date);
    assert!(generated.after(&test_date) && generated.before(&constrain_date));
}

/// `testCreatedDateBackward`.
#[test]
fn twitter_created_date_backward() {
    let test_date = Date::now();
    let constrain_date = Date::new(test_date.get_time() - 3_000_000);
    let generated = Faker::new().twitter().created_time(false, test_date, constrain_date);
    assert!(generated.before(&test_date) && generated.after(&constrain_date));
}

/// `testShortTwitterIdLength`.
#[test]
fn twitter_short_twitter_id_length() {
    assert_eq!(Faker::new().twitter().twitter_id(6).unwrap().len(), 6);
}

/// `testLongTwitterIdLength` (repeated 10 times).
#[test]
fn twitter_long_twitter_id_length() {
    let f = Faker::new();
    for _ in 0..10 {
        assert_eq!(f.twitter().twitter_id(25).unwrap().len(), 25);
    }
}

/// `testTwitterIdLength`.
#[test]
fn twitter_twitter_id_length() {
    assert_eq!(Faker::new().twitter().twitter_id(15).unwrap().len(), 15);
}

/// `testTwitterIdUnique`.
#[test]
fn twitter_twitter_id_unique() {
    let f = Faker::new();
    assert_ne!(f.twitter().twitter_id(15).unwrap(), f.twitter().twitter_id(15).unwrap());
}

/// `testTextLength`.
#[test]
fn twitter_text_length() {
    let text = Faker::new().twitter().text(None, 15, 5).unwrap();
    assert!(text.split(' ').count() <= 15);
}

/// `testTextKeyWords`.
#[test]
fn twitter_text_key_words() {
    let keywords = ["buy".to_string(), "see".to_string()];
    let text = Faker::new().twitter().text(Some(&keywords), 15, 5).unwrap();
    let words: Vec<&str> = text.split(' ').collect();
    assert!(keywords.iter().all(|k| words.contains(&k.as_str())));
}

/// `username`.
#[test]
fn twitter_username() {
    let f = Faker::new();
    for _ in 0..10 {
        assert_matches("[a-zA-Z\\d_\\-\u{4e00}-\u{9fa5}]+", &f.twitter().user_name().unwrap());
    }
}

/// `userId`.
#[test]
fn twitter_user_id() {
    let f = Faker::new();
    for _ in 0..10 {
        assert_matches("\\d+", &f.twitter().user_id().unwrap());
    }
}

/// `linkTestRules`.
#[test]
fn twitter_link_test_rules() {
    let f = Faker::new();
    for _ in 0..10 {
        assert_matches("[A-Za-z\\d.:/]+", &f.twitter().get_link("John", 6).unwrap());
    }
}

/// `linkTestKeyWords`.
#[test]
fn twitter_link_test_key_words() {
    let f = Faker::new();
    for _ in 0..10 {
        assert!(f.twitter().get_link("John", 6).unwrap().contains("John"));
    }
}

/// The `twitterId` algorithm on a fixed UUID and clock (the upstream tests only check lengths).
#[test]
fn twitter_id_with_fixed_inputs() {
    let f = Faker::with_seed(1);
    let id = f.twitter().twitter_id_with(25, "8f2c1a8e-6c39-4f1e-9a54-2e8b8f6f3b7d", Instant::of_epoch_milli(1_700_000_000_123)).unwrap();
    assert_eq!(id.len(), 25);
    assert!(id.chars().all(|c| c.is_ascii_digit()));
}
