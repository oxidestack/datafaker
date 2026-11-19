//! Differential vectors recorded from Datafaker 2.7.0 (`scripts/record.sh`, `scripts/Vectors.java`): each row is a
//! seed, a `;`-separated list of calls, and what each call returned on a fresh `new Random(seed)` shared by
//! `random`, `rs = new RandomService(random)` and `faker = new Faker(random)`, and on a fresh
//! `new SplittableRandom(seed)` shared by `splittable` and `srs = new RandomService(splittable)`; `value.of(x)` shows
//! a factory argument. A thrown exception reads `ERR <simple class name>: <message>`.

mod common;

use common::Call;
use datafaker::java::{RandomGenerator, SplittableRandom, double_to_string, float_to_string};
use datafaker::{Faker, RandomService, Result};

fn fmt_err(e: &datafaker::Error) -> String {
    format!("ERR {}: {}", e.simple_class(), e.message())
}

fn show<T: ToString>(r: Result<T>) -> String {
    match r {
        Ok(v) => v.to_string(),
        Err(e) => fmt_err(&e),
    }
}

fn bytes(b: &[i8]) -> String {
    format!("[{}]", b.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))
}

/// `random`/`splittable` calls on a generator as a `RandomGenerator`; `random` also has `nextGaussian`.
fn generator_call(random: &dyn RandomGenerator, method: &str, a: &[common::Arg]) -> Option<String> {
    Some(match (method, a) {
        ("nextInt", []) => random.next_int().to_string(),
        ("nextInt", [b]) => show(random.next_int_bounded(b.int())),
        ("nextInt", [o, b]) => show(random.next_int_range(o.int(), b.int())),
        ("nextLong", []) => random.next_long().to_string(),
        ("nextLong", [b]) => show(random.next_long_bounded(b.long())),
        ("nextLong", [o, b]) => show(random.next_long_range(o.long(), b.long())),
        ("nextBoolean", []) => random.next_boolean().to_string(),
        ("nextFloat", []) => float_to_string(random.next_float()),
        ("nextDouble", []) => double_to_string(random.next_double()),
        ("nextGaussian", []) => double_to_string(random.as_random()?.next_gaussian()),
        _ => return None,
    })
}

fn random_call(rs: &RandomService, splittable: &SplittableRandom, srs: &RandomService, c: &Call) -> Option<String> {
    let a = &c.args;
    let service = match c.root.as_str() {
        "random" => return generator_call(rs.random(), &c.chain[0], a),
        // `splittable.split().<call>`: one call on the generator split off.
        "splittable" if c.chain[0] == "split" => return generator_call(&splittable.split(), c.chain.get(1)?, a),
        "splittable" => return generator_call(splittable, &c.chain[0], a),
        "rs" => rs,
        "srs" => srs,
        "value" => return value_call(c),
        _ => return None,
    };
    Some(match (c.chain[0].as_str(), a.as_slice()) {
        ("nextInt", []) => service.next_int().to_string(),
        ("nextInt", [b]) => show(service.next_int_bounded(b.int())),
        ("nextInt", [o, b]) => show(service.next_int_between(o.int(), b.int())),
        ("nextLong", []) => service.next_long().to_string(),
        ("nextLong", [b]) => show(service.next_long_bounded(b.long())),
        ("nextLong", [o, b]) => show(service.next_long_between(o.long(), b.long())),
        ("nextFloat", []) => float_to_string(service.next_float()),
        ("nextDouble", []) => double_to_string(service.next_double()),
        ("nextDouble", [o, b]) => double_to_string(service.next_double_between(o.double(), b.double())),
        ("nextBoolean", []) => service.next_boolean().to_string(),
        ("hex", []) => service.hex(),
        ("hex", [n, u]) => service.hex_with(n.int(), u.bool()),
        ("nextRandomBytes", [n]) => bytes(&service.next_random_bytes(n.int() as usize)),
        _ => return None,
    })
}

/// `value.of(x)`: what a factory argument builds, or the exception it throws (before any method runs, as Java
/// evaluates arguments first).
fn value_call(c: &Call) -> Option<String> {
    match (c.chain[0].as_str(), c.args.as_slice()) {
        ("of", [common::Arg::Obj(f, args)]) if f == "LocalDate" => Some(show(datafaker::java::LocalDate::parse(args[0].str()))),
        ("of", [common::Arg::Obj(f, args)]) if f == "Instant" => Some(show(datafaker::java::Instant::parse(args[0].str()))),
        ("of", [a @ common::Arg::Obj(f, _)]) if f == "Date" || f == "Timestamp" => Some(a.date().to_string()),
        ("of", [common::Arg::Obj(f, args)]) if f == "DateText" => Some(show(datafaker::java::Date::parse_text(args[0].str()))),
        ("of", [common::Arg::Obj(f, args)]) if f == "Parse" || f == "ParseZ" => {
            let formatter = datafaker::java::DateTimeFormatter::of_pattern(args[0].str());
            let formatter = if f == "ParseZ" { formatter.map(|f| f.with_default_zone()) } else { formatter };
            Some(show(formatter.and_then(|f| f.parse(args[1].str()))))
        }
        _ => None,
    }
}

/// A row uses either `faker` or `random`/`rs`, never both, so each gets its own generator here.
fn run(seed: i64, calls: &str) -> Vec<String> {
    let rs = RandomService::with_seed(seed);
    let splittable = std::sync::Arc::new(SplittableRandom::new(seed));
    let srs = RandomService::with_generator(splittable.clone());
    let faker = std::cell::OnceCell::new();
    common::split_calls(calls)
        .iter()
        .map(|text| {
            let call = Call::parse(text);
            if let Some(locale) = common::faker_root_locale(&call.root) {
                common::faker_call(faker.get_or_init(|| Faker::with_locale_and_seed(locale, seed)), &call)
            } else {
                random_call(&rs, &splittable, &srs, &call).unwrap_or_else(|| panic!("no dispatch for {text}"))
            }
        })
        .collect()
}

/// Identity hash codes in exception messages (`java.util.Random@1b2c3d`, `[Ljava.lang.String;@5e2de80c`, a
/// provider's `Address(Faker.en)@7d4793a8`) vary between JVM runs.
fn normalize(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('@') {
        let before = &rest[..i];
        let end = i + 1;
        out.push_str(&rest[..end]);
        let array = before.len() >= 2 && before.as_bytes()[before.len() - 2] == b'[';
        let provider = before.ends_with(')') && before.contains("(Faker.");
        let hashed = before.ends_with("java.util.Random") || before.ends_with(';') || array || provider;
        if hashed {
            rest = rest[end..].trim_start_matches(|c: char| c.is_ascii_hexdigit());
            out.push('N');
        } else {
            rest = &rest[end..];
        }
    }
    out.push_str(rest);
    out
}

/// Replays `table`; a `# today <date>` line fixes the clock at that date (the day the file was recorded), for the
/// calls that read it, and a `# zone <id>` line sets the default zone of the file's JVM.
fn check(file: &str, table: &str) {
    let today = table.lines().find_map(|l| l.strip_prefix("# today ")).map(|d| datafaker::java::Instant::parse(&format!("{}T00:00:00Z", d.trim())).expect("a date"));
    let zone = table.lines().find_map(|l| l.strip_prefix("# zone ")).map(|z| datafaker::java::time::ZoneId::of(z.trim()).expect("a zone"));
    let rows = || match today {
        Some(instant) => datafaker::java::time::with_fixed_clock(instant, || check_rows(file, table)),
        None => check_rows(file, table),
    };
    match zone {
        Some(zone) => datafaker::java::time::with_default_zone(zone, rows),
        None => rows(),
    }
}

fn check_rows(file: &str, table: &str) {
    let mut failures = Vec::new();
    let mut count = 0;
    for line in table.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')) {
        let cols: Vec<&str> = line.split('\t').collect();
        let seed: i64 = cols[0].parse().expect("seed");
        let got = run(seed, cols[1]);
        let calls = common::split_calls(cols[1]);
        assert_eq!(cols.len() - 2, calls.len(), "{file}: row for seed {seed} has the wrong number of results");
        for (i, want) in cols[2..].iter().enumerate() {
            count += 1;
            let want = normalize(&common::unescape(want));
            if normalize(&got[i]) != want {
                failures.push(format!("seed {seed} call {} `{}`\n  got  {}\n  want {want}", i + 1, calls[i], got[i]));
            }
        }
    }
    eprintln!("{file}: {count} results");
    assert!(failures.is_empty(), "{file}: {} of {count} differ:\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn faker() {
    check("faker.tsv", include_str!("vectors/faker.tsv"));
}

#[test]
fn random() {
    check("random.tsv", include_str!("vectors/random.tsv"));
}

#[test]
fn locales() {
    check("locales.tsv", include_str!("vectors/locales.tsv"));
}

#[test]
fn functions() {
    check("functions.tsv", include_str!("vectors/functions.tsv"));
}

#[test]
fn number() {
    check("number.tsv", include_str!("vectors/number.tsv"));
}

/// Every method of the generated YAML-only providers (`scripts/gen-providers.py`), English, el_GR and de.
#[test]
fn providers() {
    check("providers.tsv", include_str!("vectors/providers.tsv"));
}

/// Hand-written rows for T1a provider methods with arguments, and expressions reaching T1a providers.
#[test]
fn providers_args() {
    check("providers_args.tsv", include_str!("vectors/providers_args.tsv"));
}

/// Hand-written rows for T1b provider methods with arguments, other locales, and expressions reaching T1b providers.
#[test]
fn providers_t1b() {
    check("providers_t1b.tsv", include_str!("vectors/providers_t1b.tsv"));
}

/// Hand-written rows (written once by a script, then recorded) for the themed providers of T4: enum overloads, draws
/// that read other lists, `heroQuote`, other locales' lists, and expressions reaching themed providers.
#[test]
fn providers_t4() {
    check("providers_t4.tsv", include_str!("vectors/providers_t4.tsv"));
}

/// The date providers on reference dates (no clock), and the JDK's `DateTimeFormatter`/`SimpleDateFormat` through
/// their pattern overloads.
#[test]
fn dates() {
    check("dates.tsv", include_str!("vectors/dates.tsv"));
}

/// Date output in other default zones: `toString`s, `DateTimeFormatter`/`SimpleDateFormat` zone texts and offsets,
/// DST gaps and overlaps, local parsing (`Date.parse`, `DateTimeFormatter` with the default zone), and tzdb 2026b's
/// permanent -07:00 in British Columbia.
#[test]
fn dates_in_zones() {
    check("dates_new_york.tsv", include_str!("vectors/dates_new_york.tsv"));
    check("dates_lord_howe.tsv", include_str!("vectors/dates_lord_howe.tsv"));
    check("dates_dublin.tsv", include_str!("vectors/dates_dublin.tsv"));
    check("dates_vancouver.tsv", include_str!("vectors/dates_vancouver.tsv"));
}

/// The id number generators of every country (valid, invalid, valid for a request), the `IdNumber` facade's
/// per-country methods and expressions, on the day the file was recorded (`# today`).
#[test]
fn idnumbers() {
    check("idnumbers.tsv", include_str!("vectors/idnumbers.tsv"));
}

/// The generating `PhoneNumber` methods (T3) on every supported locale, PhoneNumberTest's locales and the
/// language-only ones, recorded with libphonenumber 9.0.41 on the classpath (scripts/phone-vectors.py).
#[test]
fn phone_numbers() {
    check("phone_numbers.tsv", include_str!("vectors/phone_numbers.tsv"));
}

#[test]
fn regexify() {
    check("regexify.tsv", include_str!("vectors/regexify.tsv"));
}

/// `tests/vectors/idn.tsv`'s escapes: `\\`, `\t`, `\n`, `\"` and `\u{…}`.
fn unescape_rust(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('u') => {
                let hex: String = chars.by_ref().skip(1).take_while(|c| *c != '}').collect();
                out.push(char::from_u32(u32::from_str_radix(&hex, 16).expect("hex")).expect("a code point"));
            }
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

/// `IDN.toASCII(input)` against the JDK (`scripts/idn-tables.sh`).
#[test]
fn idn() {
    let mut failures = Vec::new();
    let mut count = 0;
    for line in include_str!("vectors/idn.tsv").lines().filter(|l| !l.starts_with('#')) {
        let (input, want) = line.split_once('\t').expect("two columns");
        let (input, want) = (unescape_rust(input), unescape_rust(want));
        let got = match datafaker::idn::to_ascii(&input) {
            Ok(s) => s,
            Err(e) => format!("ERR {}", e.message()),
        };
        count += 1;
        if got != want {
            failures.push(format!("{input:?}\n  got  {got:?}\n  want {want:?}"));
        }
    }
    eprintln!("idn.tsv: {count} results");
    let shown: Vec<String> = failures.iter().take(40).cloned().collect();
    assert!(failures.is_empty(), "idn.tsv: {} of {count} differ:\n{}", failures.len(), shown.join("\n"));
}
