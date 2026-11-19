//! The call syntax of `scripts/Vectors.java`, parsed the same way, and the dispatch of `faker.…` calls.
#![allow(dead_code)]

use datafaker::Faker;
use datafaker::java::double_to_string;

#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Int(i32),
    Long(i64),
    Double(f64),
    Bool(bool),
    Char(char),
    Str(String),
    Null,
    /// An enum constant, `TimeUnit.SECONDS`: the class's simple name and the constant.
    Enum(String, String),
    /// An object from a factory of `Vectors.java`, `Instant("…")`: the factory's name and its arguments.
    Obj(String, Vec<Arg>),
}

impl Arg {
    pub fn int(&self) -> i32 {
        match self {
            Arg::Int(i) => *i,
            other => panic!("not an int: {other:?}"),
        }
    }

    pub fn long(&self) -> i64 {
        match self {
            Arg::Long(i) => *i,
            Arg::Int(i) => *i as i64,
            other => panic!("not a long: {other:?}"),
        }
    }

    pub fn double(&self) -> f64 {
        match self {
            Arg::Double(d) => *d,
            Arg::Int(i) => *i as f64,
            other => panic!("not a double: {other:?}"),
        }
    }

    pub fn bool(&self) -> bool {
        match self {
            Arg::Bool(b) => *b,
            other => panic!("not a boolean: {other:?}"),
        }
    }

    pub fn str(&self) -> &str {
        match self {
            Arg::Str(s) => s,
            other => panic!("not a string: {other:?}"),
        }
    }

    pub fn time_unit(&self) -> datafaker::java::TimeUnit {
        match self {
            Arg::Enum(c, k) if c == "TimeUnit" => datafaker::java::TimeUnit::value_of(k).expect("a TimeUnit"),
            other => panic!("not a TimeUnit: {other:?}"),
        }
    }

    pub fn chrono_unit(&self) -> datafaker::java::ChronoUnit {
        match self {
            Arg::Enum(c, k) if c == "ChronoUnit" => datafaker::java::ChronoUnit::value_of(k).expect("a ChronoUnit"),
            other => panic!("not a ChronoUnit: {other:?}"),
        }
    }

    fn obj(&self, factory: &str) -> &[Arg] {
        match self {
            Arg::Obj(f, args) if f == factory => args,
            other => panic!("not a {factory}: {other:?}"),
        }
    }

    pub fn instant(&self) -> datafaker::java::Instant {
        datafaker::java::Instant::parse(self.obj("Instant")[0].str()).expect("an instant")
    }

    pub fn local_time(&self) -> datafaker::java::LocalTime {
        datafaker::java::LocalTime::parse(self.obj("LocalTime")[0].str()).expect("a local time")
    }

    pub fn period(&self) -> datafaker::java::Period {
        datafaker::java::Period::parse(self.obj("Period")[0].str()).expect("a period")
    }

    /// `Date(…)` or `Timestamp(…)`.
    pub fn date(&self) -> datafaker::java::Date {
        match self {
            Arg::Obj(f, args) if f == "Date" => datafaker::java::Date::new(args[0].long()),
            Arg::Obj(f, args) if f == "Timestamp" => datafaker::java::Date::timestamp(args[0].long()),
            other => panic!("not a Date: {other:?}"),
        }
    }

    pub fn local_date(&self) -> datafaker::java::LocalDate {
        datafaker::java::LocalDate::parse(self.obj("LocalDate")[0].str()).expect("a local date")
    }

    /// `IdNumberRequest(minAge, maxAge, GenderRequest.X)`.
    pub fn id_number_request(&self) -> datafaker::idnumbers::IdNumberRequest {
        use datafaker::idnumbers::{GenderRequest, IdNumberRequest};
        let args = self.obj("IdNumberRequest");
        let gender = match &args[2] {
            Arg::Enum(c, k) if c == "GenderRequest" && k == "FEMALE" => GenderRequest::Female,
            Arg::Enum(c, k) if c == "GenderRequest" && k == "MALE" => GenderRequest::Male,
            Arg::Enum(c, k) if c == "GenderRequest" && k == "ANY" => GenderRequest::Any,
            other => panic!("not a GenderRequest: {other:?}"),
        };
        IdNumberRequest::new(args[0].int(), args[1].int(), gender)
    }

    /// `PolishGender.X` (`PolishIdNumber.Gender`) or `null`.
    pub fn polish_gender(&self) -> Option<datafaker::idnumbers::PolishGender> {
        use datafaker::idnumbers::PolishGender;
        match self {
            Arg::Null => None,
            Arg::Enum(c, k) if c == "PolishGender" && k == "MALE" => Some(PolishGender::Male),
            Arg::Enum(c, k) if c == "PolishGender" && k == "FEMALE" => Some(PolishGender::Female),
            Arg::Enum(c, k) if c == "PolishGender" && k == "ANY" => Some(PolishGender::Any),
            other => panic!("not a PolishGender: {other:?}"),
        }
    }

    /// `Strings(…)` or `null`.
    pub fn strings(&self) -> Option<Vec<String>> {
        match self {
            Arg::Null => None,
            other => Some(other.obj("Strings").iter().map(|a| a.str().to_string()).collect()),
        }
    }
}

/// `root.a().b(args)`: `chain` holds the method names, `args` the last call's arguments (earlier calls in a chain
/// take none in the vectors).
#[derive(Debug)]
pub struct Call {
    pub root: String,
    pub chain: Vec<String>,
    pub args: Vec<Arg>,
}

impl Call {
    pub fn parse(text: &str) -> Call {
        let chars: Vec<char> = text.trim().chars().collect();
        let mut pos = 0;
        let ident = |pos: &mut usize| {
            let start = *pos;
            while *pos < chars.len() && (chars[*pos].is_alphanumeric() || chars[*pos] == '_') {
                *pos += 1;
            }
            chars[start..*pos].iter().collect::<String>()
        };
        let root = ident(&mut pos);
        let mut chain = Vec::new();
        let mut args = Vec::new();
        while pos < chars.len() {
            assert_eq!(chars[pos], '.', "{text}");
            pos += 1;
            chain.push(ident(&mut pos));
            assert_eq!(chars[pos], '(', "{text}");
            pos += 1;
            args = Vec::new();
            while chars[pos] != ')' {
                args.push(literal(&chars, &mut pos));
                if chars[pos] == ',' {
                    pos += 1;
                }
            }
            pos += 1;
        }
        Call { root, chain, args }
    }
}

fn literal(chars: &[char], pos: &mut usize) -> Arg {
    if chars[*pos] == '"' {
        let mut s = String::new();
        *pos += 1;
        while chars[*pos] != '"' {
            let d = chars[*pos];
            *pos += 1;
            if d == '\\' {
                let e = chars[*pos];
                *pos += 1;
                s.push(match e {
                    'n' => '\n',
                    't' => '\t',
                    other => other,
                });
            } else {
                s.push(d);
            }
        }
        *pos += 1;
        return Arg::Str(s);
    }
    if chars[*pos] == '\'' {
        let c = chars[*pos + 1];
        *pos += 3;
        return Arg::Char(c);
    }
    if chars[*pos].is_uppercase() {
        let ident = |pos: &mut usize| {
            let start = *pos;
            while *pos < chars.len() && (chars[*pos].is_alphanumeric() || chars[*pos] == '_') {
                *pos += 1;
            }
            chars[start..*pos].iter().collect::<String>()
        };
        let name = ident(pos);
        if chars[*pos] == '(' {
            *pos += 1;
            let mut args = Vec::new();
            while chars[*pos] != ')' {
                args.push(literal(chars, pos));
                if chars[*pos] == ',' {
                    *pos += 1;
                }
            }
            *pos += 1;
            return Arg::Obj(name, args);
        }
        assert_eq!(chars[*pos], '.');
        *pos += 1;
        return Arg::Enum(name, ident(pos));
    }
    let start = *pos;
    while *pos < chars.len() && chars[*pos] != ',' && chars[*pos] != ')' {
        *pos += 1;
    }
    let t: String = chars[start..*pos].iter().collect::<String>().trim().to_string();
    match t.as_str() {
        "true" => Arg::Bool(true),
        "false" => Arg::Bool(false),
        "null" => Arg::Null,
        _ if t.ends_with('L') => Arg::Long(t[..t.len() - 1].parse().expect("long")),
        _ if t.contains('.') || t.contains('E') => Arg::Double(t.parse().expect("double")),
        _ => Arg::Int(t.parse().expect("int")),
    }
}

/// Splits on `;` outside string literals.
pub fn split_calls(s: &str) -> Vec<String> {
    let mut res = Vec::new();
    let mut cur = String::new();
    let mut in_str = false;
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if in_str && c == '\\' {
            cur.push(c);
            if let Some(n) = it.next() {
                cur.push(n);
            }
            continue;
        }
        if c == '"' {
            in_str = !in_str;
        }
        if c == ';' && !in_str {
            res.push(std::mem::take(&mut cur));
            continue;
        }
        cur.push(c);
    }
    res.push(cur);
    res
}

/// Reverses the output escaping of `Vectors.java`.
pub fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some(o) => out.push(o),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn show<T: ToString>(r: datafaker::Result<T>) -> String {
    match r {
        Ok(v) => v.to_string(),
        Err(e) => format!("ERR {}: {}", e.simple_class(), e.message()),
    }
}

fn list(r: datafaker::Result<Vec<String>>) -> String {
    show(r.map(|v| format!("[{}]", v.join(", "))))
}

fn opt(r: datafaker::Result<Option<String>>) -> String {
    show(r.map(|v| v.unwrap_or_else(|| "null".into())))
}

/// The locale of a faker root of `Vectors.java` (`faker`, `fakerDe`, `fakerEnUs`, …).
pub fn faker_root_locale(root: &str) -> Option<datafaker::Locale> {
    use datafaker::Locale;
    Some(match root {
        "faker" => Locale::english(),
        "fakerDe" => Locale::language_only("de"),
        "fakerEnUs" => Locale::new("en", "US"),
        "fakerElGr" => Locale::new("el", "GR"),
        "fakerRu" => Locale::language_only("ru"),
        "fakerFa" => Locale::language_only("fa"),
        "fakerPtBr" => Locale::new("pt", "BR"),
        "fakerZhCn" => Locale::new("zh", "CN"),
        "fakerJa" => Locale::language_only("ja"),
        "fakerTr" => Locale::language_only("tr"),
        // `faker_<language>[_<COUNTRY>]`: any other locale.
        other => {
            let mut parts = other.strip_prefix("faker_")?.split('_');
            match (parts.next(), parts.next()) {
                (Some(language), Some(country)) => Locale::new(language, country),
                (Some(language), None) => Locale::language_only(language),
                _ => return None,
            }
        }
    })
}

/// Runs one `faker.…` call of a vector row on the Rust API.
pub fn faker_call(f: &Faker, call: &Call) -> String {
    let a = &call.args;
    let chain: Vec<&str> = call.chain.iter().map(String::as_str).collect();
    match (chain.as_slice(), a.as_slice()) {
        (["numerify"], [x]) => f.numerify(x.str()),
        (["letterify"], [x]) => f.letterify(x.str()),
        (["letterify"], [x, u]) => f.letterify_with(x.str(), u.bool()),
        (["bothify"], [x]) => f.bothify(x.str()),
        (["bothify"], [x, u]) => f.bothify_with(x.str(), u.bool()),
        (["examplify"], [x]) => f.examplify(x.str()),
        (["regexify"], [x]) => show(f.regexify(x.str())),
        (["expression"], [x]) => show(f.expression(x.str())),
        (["resolve"], [x]) => show(f.resolve(x.str())),
        (["name", "name"], []) => show(f.name().name()),
        (["name", "nameWithMiddle"], []) => show(f.name().name_with_middle()),
        (["name", "fullName"], []) => show(f.name().full_name()),
        (["name", "firstName"], []) => show(f.name().first_name()),
        (["name", "femaleFirstName"], []) => show(f.name().female_first_name()),
        (["name", "maleFirstName"], []) => show(f.name().male_first_name()),
        (["name", "lastName"], []) => show(f.name().last_name()),
        (["name", "prefix"], []) => show(f.name().prefix()),
        (["name", "suffix"], []) => show(f.name().suffix()),
        (["name", "title"], []) => show(f.name().title()),
        (["name", "username"], []) => show(f.name().username()),
        (["credentials", "username"], []) => show(f.credentials().username()),
        (["credentials", "password"], []) => show(f.credentials().password()),
        (["credentials", "password"], [a, b, c, d, e]) => show(f.credentials().password_with(a.int(), b.int(), c.bool(), d.bool(), e.bool())),
        (["credentials", "weakPassword"], []) => show(f.credentials().weak_password()),
        (["credentials", "userId"], []) => opt(f.credentials().user_id()),
        (["text", "text"], []) => show(f.text().text()),
        (["text", "text"], [n]) => show(f.text().text_of_length(n.int())),
        (["text", "text"], [a, b]) => show(f.text().text_between(a.int(), b.int())),
        (["text", "text"], [a, b, c, d, e]) => show(f.text().text_with(a.int(), b.int(), c.bool(), d.bool(), e.bool())),
        (["text", "character"], []) => show(f.text().character()),
        (["text", "uppercaseCharacter"], []) => show(f.text().uppercase_character()),
        (["text", "lowercaseCharacter"], []) => show(f.text().lowercase_character()),
        (["number", "numberBetween"], [Arg::Int(x), Arg::Int(y)]) => f.number().number_between(*x, *y).to_string(),
        (["number", "numberBetween"], [x @ (Arg::Int(_) | Arg::Long(_)), y @ (Arg::Int(_) | Arg::Long(_))]) => {
            show(f.number().number_between_long(x.long(), y.long()))
        }
        (["number", "randomDouble"], [n, Arg::Int(x), Arg::Int(y)]) => double_to_string(f.number().random_double_int(n.int(), *x, *y)),
        (["number", "randomDouble"], [n, x, y]) => double_to_string(f.number().random_double(n.int(), x.long(), y.long())),
        (["number", "randomDigit"], []) => f.number().random_digit().to_string(),
        (["number", "randomDigitNotZero"], []) => f.number().random_digit_not_zero().to_string(),
        (["number", "randomNumber"], []) => f.number().random_number().to_string(),
        (["number", "randomNumber"], [n]) => show(f.number().random_number_digits(n.int())),
        (["number", "positive"], []) => f.number().positive().to_string(),
        (["number", "negative"], []) => f.number().negative().to_string(),
        (["number", "digits"], [n]) => f.number().digits(n.int() as usize),
        (["number", "digit"], []) => f.number().digit(),
        (["lorem", "word"], []) => show(f.lorem().word()),
        (["lorem", "words"], []) => list(f.lorem().words(3)),
        (["lorem", "words"], [n]) => list(f.lorem().words(n.int())),
        (["lorem", "supplemental"], []) => show(f.lorem().supplemental()),
        (["lorem", "sentence"], []) => show(f.lorem().sentence()),
        (["lorem", "sentence"], [n]) => show(f.lorem().sentence_of(n.int())),
        (["lorem", "sentence"], [n, m]) => show(f.lorem().sentence_with(n.int(), m.int())),
        (["lorem", "sentences"], [n]) => list(f.lorem().sentences(n.int())),
        (["lorem", "paragraph"], []) => show(f.lorem().paragraph()),
        (["lorem", "paragraph"], [n]) => show(f.lorem().paragraph_of(n.int())),
        (["lorem", "paragraphs"], [n]) => list(f.lorem().paragraphs(n.int())),
        (["lorem", "character"], []) => show(f.lorem().character()),
        (["lorem", "character"], [u]) => show(f.lorem().character_with(u.bool())),
        (["lorem", "characters"], []) => show(f.lorem().characters()),
        (["lorem", "characters"], [a, b]) => show(f.lorem().characters_between(a.int(), b.int())),
        (["lorem", "characters"], [a, b, c, d, e]) => show(f.lorem().characters_with(a.int(), b.int(), c.bool(), d.bool(), e.bool())),
        (["lorem", "fixedString"], [n]) => show(f.lorem().fixed_string(n.int())),
        (["lorem", "maxLengthSentence"], [n]) => show(f.lorem().max_length_sentence(n.int())),
        (["internet", "emailAddress"], []) => show(f.internet().email_address()),
        (["internet", "emailAddress"], [n]) => show(f.internet().email_address_for(n.str())),
        (["internet", "safeEmailAddress"], []) => show(f.internet().safe_email_address()),
        (["internet", "emailSubject"], []) => show(f.internet().email_subject()),
        (["internet", "domainName"], []) => show(f.internet().domain_name()),
        (["internet", "domainWord"], []) => show(f.internet().domain_word()),
        (["internet", "domainSuffix"], []) => show(f.internet().domain_suffix()),
        (["internet", "url"], []) => show(f.internet().url()),
        (["internet", "webdomain"], []) => show(f.internet().webdomain()),
        (["internet", "image"], []) => show(f.internet().image()),
        (["internet", "httpMethod"], []) => show(f.internet().http_method()),
        (["internet", "password"], []) => show(f.internet().password()),
        (["internet", "password"], [a, b, c, d, e]) => show(f.internet().password_with(a.int(), b.int(), c.bool(), d.bool(), e.bool())),
        (["internet", "port"], []) => f.internet().port().to_string(),
        (["internet", "macAddress"], []) => f.internet().mac_address(),
        (["internet", "macAddress"], [p]) => f.internet().mac_address_with(p.str()),
        (["internet", "ipV4Address"], []) => f.internet().ip_v4_address(),
        (["internet", "privateIpV4Address"], []) => f.internet().private_ip_v4_address(),
        (["internet", "publicIpV4Address"], []) => f.internet().public_ip_v4_address(),
        (["internet", "ipV4Cidr"], []) => f.internet().ip_v4_cidr(),
        (["internet", "ipV6Address"], []) => f.internet().ip_v6_address(),
        (["internet", "ipV6Cidr"], []) => f.internet().ip_v6_cidr(),
        (["internet", "slug"], []) => show(f.internet().slug()),
        (["internet", "uuid"], []) => f.internet().uuid(),
        (["internet", "uuidv3"], []) => f.internet().uuidv3(),
        (["internet", "uuidv7"], []) => f.internet().uuidv7(),
        (["internet", "userAgent"], []) => show(f.internet().user_agent()),
        (["internet", "botUserAgentAny"], []) => show(f.internet().bot_user_agent_any()),
        (["internet", "username"], []) => show(f.internet().username()),
        ([provider, method], args) => date_call(f, provider, method, args)
            .or_else(|| reflect(f, provider, method, args))
            .unwrap_or_else(|| panic!("no dispatch for {call:?}")),
        _ => panic!("no dispatch for {call:?}"),
    }
}

/// The overloads taking objects (`Instant`, `Date`, `LocalTime`, `Period`, `String[]`, `IdNumberRequest`,
/// `LocalDate`), which the method tables leave out, on the Rust API.
fn date_call(f: &Faker, provider: &str, method: &str, a: &[Arg]) -> Option<String> {
    let has_obj = a.iter().any(|x| matches!(x, Arg::Obj(..))) || provider == "twitter" && method == "text";
    if !has_obj {
        return None;
    }
    Some(match (provider, method, a) {
        ("timeAndDate", "future", [n, u, r]) => show(f.time_and_date().future_from(n.long(), u.time_unit(), r.instant())),
        ("timeAndDate", "future", [n, u, r, p]) => show(f.time_and_date().future_from_formatted(n.long(), u.time_unit(), r.instant(), p.str())),
        ("timeAndDate", "past", [n, u, r]) => show(f.time_and_date().past_from(n.long(), u.time_unit(), r.instant())),
        ("timeAndDate", "past", [n, u, r, p]) => show(f.time_and_date().past_from_formatted(n.long(), u.time_unit(), r.instant(), p.str())),
        ("timeAndDate", "between", [x, y]) => show(f.time_and_date().between(x.instant(), y.instant())),
        ("timeAndDate", "between", [x, y, p]) => show(f.time_and_date().between_formatted(x.instant(), y.instant(), p.str())),
        ("timeAndDate", "period", [x, y]) => show(f.time_and_date().period(x.period(), y.period())),
        ("date", "future", [n, u, r]) => show(f.date().future_from(n.int(), u.time_unit(), r.date())),
        ("date", "future", [n, u, r, p]) => show(f.date().future_from_formatted(n.int(), u.time_unit(), r.date(), p.str())),
        ("date", "past", [n, u, r]) => show(f.date().past_from(n.int(), u.time_unit(), r.date())),
        ("date", "past", [n, u, r, p]) => show(f.date().past_from_formatted(n.int(), u.time_unit(), r.date(), p.str())),
        ("date", "between", [x, y]) => show(f.date().between(x.date(), y.date())),
        ("date", "between", [x, y, p]) => show(f.date().between_formatted(x.date(), y.date(), p.str())),
        ("date", "period", [x, y]) => show(f.date().period(x.period(), y.period())),
        ("time", "between", [x, y]) => show(f.time().between(x.local_time(), y.local_time())),
        ("time", "between", [x, y, p]) => show(f.time().between_formatted(x.local_time(), y.local_time(), p.str())),
        ("twitter", "createdTime", [d, b, c]) => f.twitter().created_time(d.bool(), b.date(), c.date()).to_string(),
        ("twitter", "text", [k, s, w]) => show(f.twitter().text(k.strings().as_deref(), s.int(), w.int())),
        ("idNumber", "valid", [r]) => show(f.id_number().valid_for(&r.id_number_request())),
        ("idNumber", "peselNumber", [d, g]) => show(f.id_number().pesel_number_for(d.local_date(), g.polish_gender())),
        _ => return None,
    })
}

/// `faker.<provider>().<method>(args)` through the method tables expressions use (each entry calls the Rust method
/// of that name), choosing the overload by the literals' Java types as `Vectors.java` does.
fn reflect(f: &Faker, provider: &str, method: &str, args: &[Arg]) -> Option<String> {
    use datafaker::JValue;
    use datafaker::dispatch::{P, Target};
    let getter = Target::Faker.methods().iter().find(|m| m.name == provider && m.params.is_empty())?;
    let target = Target::of(&(getter.invoke)(f, &[]).ok()?)?;
    let convert = |p: &P, a: &Arg| -> Option<JValue> {
        Some(match (p, a) {
            (P::Int, Arg::Int(i)) => JValue::Int(*i),
            (P::Long, Arg::Int(i)) => JValue::Long(*i as i64),
            (P::Long, Arg::Long(l)) => JValue::Long(*l),
            (P::Double, Arg::Double(d)) => JValue::Double(*d),
            (P::Boolean, Arg::Bool(b)) => JValue::Bool(*b),
            (P::String | P::CharSequence | P::Object, Arg::Str(s)) => JValue::Str(s.clone()),
            (P::Enum(class, constants), Arg::Enum(c, k)) if class == c => JValue::Enum(constants.iter().find(|x| *x == k)?),
            _ => return None,
        })
    };
    let (m, values) = target.methods().iter().filter(|m| m.name == method && m.params.len() == args.len() && !m.varargs).find_map(|m| {
        let values: Option<Vec<JValue>> = m.params.iter().zip(args).map(|(p, a)| convert(p, a)).collect();
        values.map(|v| (m, v))
    })?;
    Some(show((m.invoke)(f, &values).map(|v| v.to_java_string(f))))
}

/// `String.matches(javaRegex)`: the whole string, with Java's ASCII-only `\w`, `\d` and `\s`. A leading `(?U)` is
/// Java's `UNICODE_CHARACTER_CLASS`, under which those classes are the Unicode ones the `regex` crate has by default
/// (in the `regex` crate `(?U)` would mean "swap greed", so it is dropped).
pub fn matches(java_regex: &str, s: &str) -> bool {
    let translated = match java_regex.strip_prefix("(?U)") {
        Some(unicode) => unicode.to_string(),
        None => java_regex.replace("\\w", "[A-Za-z0-9_]").replace("\\d", "[0-9]").replace("\\s", "[ \\t\\n\\x0B\\f\\r]"),
    };
    let re = regex::Regex::new(&format!("^(?:{translated})$")).unwrap_or_else(|e| panic!("{java_regex}: {e}"));
    re.is_match(s)
}

/// `assertThat(s).matches(javaRegex)`.
#[track_caller]
pub fn assert_matches(java_regex: &str, s: &str) {
    assert!(matches(java_regex, s), "{s:?} does not match {java_regex}");
}

/// Stands in for commons-validator's `EmailValidator.getInstance().isValid` on the addresses Datafaker makes: a
/// dot-atom local part and a domain of dot-separated labels ending in a letters-only TLD.
pub fn is_valid_email(s: &str) -> bool {
    matches(r"[A-Za-z0-9!#$%&'*+/=?^_`{|}~-]+(\.[A-Za-z0-9!#$%&'*+/=?^_`{|}~-]+)*@([A-Za-z0-9]([A-Za-z0-9-]*[A-Za-z0-9])?\.)+[A-Za-z]{2,}", s)
}

/// `AbstractProviderListTest.testProviderList`: `actual` is non-empty and one of the strings listed at `key`, none
/// of which is blank; it matches `regex` when one is given.
#[track_caller]
pub fn provider_list(faker: &Faker, key: &str, actual: &str, regex: &str) {
    let expected: Vec<String> = match faker.fake_values_service().fetch_object(key) {
        Some(datafaker::yaml::Value::List(l)) => l.iter().map(|v| v.to_java_string()).collect(),
        other => panic!("{key} is not a list: {other:?}"),
    };
    assert!(!actual.is_empty(), "{key}: empty value");
    assert!(!expected.is_empty() && expected.iter().any(|e| e == actual), "{key}: {actual:?} not listed");
    assert!(expected.iter().all(|e| !e.trim().is_empty()), "{key}: blank entry");
    if !regex.is_empty() {
        assert_matches(regex, actual);
    }
}

/// `EvenlyDistributedRandomLongGenerator` (upstream test support, `service/EvenlyDistributedRandomLongGenerator.java`):
/// a `java.util.Random` subclass that returns every value of `[lower, upper)` once, in a shuffled order, then
/// starts over.
#[derive(Debug)]
pub struct EvenlyDistributedRandomLongGenerator {
    values: Vec<i64>,
    lower_inclusive: i64,
    upper_exclusive: i64,
    index: std::sync::Mutex<usize>,
}

impl EvenlyDistributedRandomLongGenerator {
    /// The generator as a `Random` (`new EvenlyDistributedRandomLongGenerator(lower, upper)`).
    // Upstream's constructor, kept as `new`: upstream's class is a `Random` subclass, so here the constructor returns
    // the `Random` it builds.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(lower_inclusive: i64, upper_exclusive: i64) -> datafaker::java::Random {
        assert!(lower_inclusive < upper_exclusive, "Expected {lower_inclusive} < {upper_exclusive}");
        let count = usize::try_from(upper_exclusive - lower_inclusive).expect("Math.toIntExact");
        let mut values: Vec<i64> = (0..count as i64).map(|i| lower_inclusive + i).collect();
        // `Collections.shuffle(values)`.
        let random = datafaker::java::Random::from_entropy();
        for i in (2..=values.len()).rev() {
            let j = random.next_int_bounded(i as i32).expect("positive") as usize;
            values.swap(i - 1, j);
        }
        let generator = EvenlyDistributedRandomLongGenerator { values, lower_inclusive, upper_exclusive, index: std::sync::Mutex::new(0) };
        datafaker::java::Random::from_entropy().with_overrides(std::sync::Arc::new(generator))
    }

    fn next(&self) -> i64 {
        let mut index = self.index.lock().expect("index");
        let i = *index;
        *index = (i + 1) % self.values.len();
        self.values[i]
    }
}

impl datafaker::java::random::RandomOverrides for EvenlyDistributedRandomLongGenerator {
    fn next_double(&self, _: &datafaker::java::Random) -> Option<f64> {
        panic!("UnsupportedOperationException")
    }

    fn next_int_bounded(&self, _: &datafaker::java::Random, _: i32) -> Option<datafaker::Result<i32>> {
        Some(Err(datafaker::Error::new("UnsupportedOperationException", "")))
    }

    fn next_long(&self, _: &datafaker::java::Random) -> Option<i64> {
        Some(self.next())
    }

    fn next_long_range(&self, _: &datafaker::java::Random, origin: i64, bound: i64) -> Option<datafaker::Result<i64>> {
        if origin != self.lower_inclusive || bound != self.upper_exclusive {
            return Some(Err(datafaker::Error::illegal_argument(format!(
                "Only numbers within [{}, {}) allowed, but requested: [{origin}, {bound}]",
                self.lower_inclusive, self.upper_exclusive
            ))));
        }
        Some(Ok(self.next()))
    }

    fn next_long_bounded(&self, random: &datafaker::java::Random, bound: i64) -> Option<datafaker::Result<i64>> {
        self.next_long_range(random, 0, bound)
    }
}

/// `AbstractProviderListTest.internationalFakers()`: a faker per tag of `Locality.allSupportedLocales()`, each
/// `new BaseFaker(new Locale(tag))`.
pub fn international_fakers() -> Vec<Faker> {
    Faker::new().locality().all_supported_locales().iter().map(|tag| Faker::with_locale(datafaker::Locale::language_only(tag))).collect()
}

/// The `(constant, string)` pairs `regex` captures in an upstream test source (`src/test/java/net/datafaker/<path>`),
/// for enums of test data too long to copy by hand. Java's `\\` escapes are undone.
pub fn java_source_enum(path: &str, regex: &str) -> Vec<(String, String)> {
    let file = format!("{}/upstream/datafaker/src/test/java/net/datafaker/{path}", env!("CARGO_MANIFEST_DIR"));
    let src = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{file}: {e}"));
    regex::Regex::new(regex).expect("regex").captures_iter(&src).map(|c| (c[1].to_string(), c[2].replace("\\\\", "\\"))).collect()
}

/// `new BaseFaker(new Locale("test"))`, whose data upstream finds on the test class path: `tests/resources/test.yml`
/// added with `addPath`.
pub fn test_locale_faker() -> Faker {
    let locale = datafaker::Locale::language_only("test");
    let faker = Faker::with_locale(locale.clone());
    faker.add_path(&locale, concat!(env!("CARGO_MANIFEST_DIR"), "/tests/resources/test.yml")).expect("test.yml");
    faker
}

/// `LuhnCheckDigit.isValid`.
pub fn is_luhn_valid(code: &str) -> bool {
    let sum: u32 = code
        .chars()
        .rev()
        .enumerate()
        .map(|(i, c)| {
            let d = c.to_digit(10).unwrap();
            if i % 2 == 1 { if d * 2 > 9 { d * 2 - 9 } else { d * 2 } } else { d }
        })
        .sum();
    sum.is_multiple_of(10)
}
