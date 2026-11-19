//! `dispatch::accessor_nullable`: a `null` argument of a call made by reflection (a Velocity template's
//! `$faker.dune().quote(null)`), as MockServer 8.0.0's shaded Datafaker 2.7.0 answers it.

use datafaker::dispatch::{accessor_nullable, Target};
use datafaker::providers::ProviderKind;
use datafaker::{Faker, JValue};

/// A call's outcome: `None` the reference rendered as written, else the value or the exception's class and message.
type Outcome = Option<Result<JValue, (String, String)>>;

fn call(faker: &Faker, target: Target, name: &str, args: &[Option<&str>]) -> Outcome {
    let args: Vec<Option<String>> = args.iter().map(|a| a.map(str::to_string)).collect();
    let found = accessor_nullable(target, &name.to_ascii_lowercase(), &args).unwrap()?;
    Some((found.method.invoke)(faker, &found.args).map_err(|e| {
        assert_eq!(e.has_message(), !e.message().is_empty(), "{e:?}");
        (e.simple_class().to_string(), e.message().to_string())
    }))
}

fn npe(message: &str) -> Outcome {
    Some(Err(("NullPointerException".into(), message.into())))
}

#[test]
fn a_null_enum_argument_throws_the_yaml_key_dereference() {
    let faker = Faker::with_seed(1);
    let dune = Target::Provider(ProviderKind::Dune);
    assert_eq!(call(&faker, dune, "quote", &[None]), npe("Cannot read field \"yamlKey\" because \"quote\" is null"));
    assert_eq!(call(&faker, dune, "saying", &[None]), npe("Cannot read field \"yamlKey\" because \"saying\" is null"));
    let tron = Target::Provider(ProviderKind::Tron);
    assert_eq!(call(&faker, tron, "quote", &[None]), npe("Cannot read field \"yamlKey\" because \"quote\" is null"));
    assert_eq!(call(&faker, tron, "character", &[None]), npe("Cannot read field \"yamlKey\" because \"character\" is null"));
    assert_eq!(
        call(&faker, tron, "alternateCharacterSpelling", &[None]),
        npe("Cannot read field \"yamlKey\" because \"alternateCharacterSpelling\" is null")
    );
}

#[test]
fn a_null_string_reaches_the_faker_methods() {
    let faker = Faker::with_seed(1);
    let input = "Cannot invoke \"String.toCharArray()\" because \"input\" is null";
    for name in ["numerify", "letterify", "bothify"] {
        assert_eq!(call(&faker, Target::Faker, name, &[None]), npe(input), "{name}");
    }
    assert_eq!(call(&faker, Target::Faker, "letterify", &[None, Some("true")]), npe(input));
    assert_eq!(call(&faker, Target::Faker, "bothify", &[None, Some("false")]), npe(input));
    assert_eq!(call(&faker, Target::Faker, "regexify", &[None]), npe("Cannot invoke \"String.length()\" because \"this.aValue\" is null"));
    assert_eq!(call(&faker, Target::Faker, "resolve", &[None]), npe("Cannot invoke \"String.length()\" because \"string\" is null"));
    assert_eq!(
        call(&faker, Target::Faker, "expression", &[None]),
        npe("Cannot invoke \"String.contains(java.lang.CharSequence)\" because \"expression\" is null")
    );
    assert_eq!(call(&faker, Target::Faker, "examplify", &[None]), Some(Ok(JValue::Null)));
    assert_eq!(
        call(&faker, Target::Provider(ProviderKind::Internet), "emailAddress", &[None]),
        npe("Cannot invoke \"java.lang.CharSequence.toString()\" because \"<parameter1>\" is null")
    );
}

#[test]
fn a_null_argument_draws_nothing_before_it_throws() {
    let (a, b) = (Faker::with_seed(7), Faker::with_seed(7));
    assert!(call(&a, Target::Faker, "bothify", &[None]).unwrap().is_err());
    assert!(call(&a, Target::Provider(ProviderKind::Dune), "quote", &[None]).unwrap().is_err());
    assert_eq!(a.numerify("####"), b.numerify("####"));
}

#[test]
fn a_null_compass_point_clears_the_choice() {
    let faker = Faker::with_seed(3);
    let compass = Target::Provider(ProviderKind::Compass);
    assert_eq!(call(&faker, compass, "compassPoint", &[Some("CARDINAL")]), Some(Ok(JValue::Provider(ProviderKind::Compass))));
    let cardinal = ["north", "east", "south", "west"];
    for _ in 0..20 {
        assert!(cardinal.contains(&faker.compass().word().unwrap().to_lowercase().as_str()));
    }
    assert_eq!(call(&faker, compass, "compassPoint", &[None]), Some(Ok(JValue::Provider(ProviderKind::Compass))));
    assert!((0..200).any(|_| !cardinal.contains(&faker.compass().word().unwrap().to_lowercase().as_str())));
}

#[test]
fn null_binds_to_no_primitive_parameter() {
    let faker = Faker::with_seed(1);
    assert_eq!(call(&faker, Target::Provider(ProviderKind::Number), "numberBetween", &[None, Some("5")]), None);
    assert_eq!(call(&faker, Target::Provider(ProviderKind::Lorem), "sentence", &[None]), None);
}

#[test]
fn a_call_velocity_finds_ambiguous_is_not_found() {
    let faker = Faker::with_seed(1);
    let date = Target::Provider(ProviderKind::DateAndTime);
    // `future(int, TimeUnit, Date)` against `future(int, TimeUnit, String)`; `flight(String)` against
    // `flight(AviationCodeType)`; `option(Object...)` against `option(String...)`.
    assert_eq!(call(&faker, date, "future", &[Some("1"), None, None]), None);
    assert_eq!(call(&faker, Target::Provider(ProviderKind::Aviation), "flight", &[None]), None);
    assert_eq!(call(&faker, Target::Provider(ProviderKind::Options), "option", &[None, None]), None);
    // A single null: `option(Class)` beats the overloads ending in an array, `nextElement(List)` `nextElement(Object[])`.
    assert_eq!(
        call(&faker, Target::Provider(ProviderKind::Options), "option", &[None]),
        npe("Cannot invoke \"java.lang.Class.getEnumConstants()\" because \"klass\" is null")
    );
    // Velocity converts no string to an enum constant.
    assert_eq!(call(&faker, Target::Provider(ProviderKind::DateAndTime), "future", &[Some("1"), Some("2"), Some("DAYS"), None]), None);
    // Without a null argument the same accessor finds what `accessor` finds.
    assert!(matches!(call(&faker, Target::Faker, "bothify", &[Some("##")]), Some(Ok(JValue::Str(s))) if s.len() == 2));
}

fn err(class: &str, message: &str) -> Outcome {
    Some(Err((class.into(), message.into())))
}

/// A `NullPointerException` without a message (`new SimpleDateFormat(null)`).
fn bare_npe() -> Outcome {
    err("NullPointerException", "")
}

/// Velocity renders the reference as written: no method found, or `null` returned.
const LITERAL: Outcome = None;

/// Every null call MockServer 8.0.0 answered with a `NullPointerException` (Datafaker's own class names, not the
/// jar's `shaded_package.` ones), another exception, or the reference as written.
#[test]
fn the_null_calls_probed_on_the_oracle() {
    let cases: &[(Target, &str, &[Option<&str>], Outcome)] = &[
        (Target::Faker, "templatify", &[None, Some("x"), None], LITERAL),
        (Target::Faker, "templatify", &[None, None], npe("Cannot invoke \"String.length()\" because \"letterString\" is null")),
        (Target::Faker, "csv", &[Some("1"), None], npe("Cannot read the array length because \"columnExpressions\" is null")),
        (Target::Faker, "csv", &[None, Some("x"), Some("true"), Some("1"), None], LITERAL),
        (Target::Faker, "json", &[None], npe("Cannot read the array length because \"fieldExpressions\" is null")),
        (Target::Faker, "jsona", &[None], npe("Cannot read the array length because \"fieldExpressions\" is null")),
        (Target::Faker, "resolve", &[None, None], npe("Cannot invoke \"String.length()\" because \"string\" is null")),
        (Target::Provider(ProviderKind::Address), "zipCodeByState", &[None], err("RuntimeException", "address.postcode_by_state.null resulted in null expression")),
        (Target::Provider(ProviderKind::Address), "countyByZipCode", &[None], err("RuntimeException", "County is not configured for postcode null")),
        (Target::Provider(ProviderKind::Aviation), "flight", &[None], LITERAL),
        (Target::Provider(ProviderKind::Coffee), "region", &[None], npe("Cannot invoke \"net.datafaker.providers.food.Coffee$Country.name()\" because \"country\" is null")),
        (Target::Provider(ProviderKind::Credentials), "userId", &[None], LITERAL),
        (Target::Provider(ProviderKind::DateAndTime), "future", &[Some("1"), None, None, None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::DateAndTime), "future", &[Some("1"), None, None], LITERAL),
        (Target::Provider(ProviderKind::DateAndTime), "future", &[Some("1"), Some("2"), None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::DateAndTime), "future", &[Some("1"), Some("2"), None, None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::DateAndTime), "future", &[Some("1"), None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::DateAndTime), "past", &[Some("1"), None, None], LITERAL),
        (Target::Provider(ProviderKind::DateAndTime), "past", &[Some("1"), Some("2"), None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::DateAndTime), "past", &[Some("1"), None, None, None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::DateAndTime), "past", &[Some("1"), Some("2"), None, None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::DateAndTime), "past", &[Some("1"), None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::DateAndTime), "birthday", &[None], bare_npe()),
        (Target::Provider(ProviderKind::DateAndTime), "birthday", &[Some("1"), Some("2"), None], bare_npe()),
        (Target::Provider(ProviderKind::DateAndTime), "duration", &[Some("1"), None], LITERAL),
        (Target::Provider(ProviderKind::DateAndTime), "duration", &[Some("1"), Some("2"), None], LITERAL),
        (Target::Provider(ProviderKind::DateAndTime), "between", &[None, None], npe("Cannot invoke \"java.util.Date.before(java.util.Date)\" because \"to\" is null")),
        (Target::Provider(ProviderKind::DateAndTime), "between", &[None, None, None], npe("Cannot invoke \"java.util.Date.before(java.util.Date)\" because \"to\" is null")),
        (Target::Provider(ProviderKind::Dota2), "heroQuote", &[None], err("RuntimeException", "games.dota2.null.quote resulted in null expression")),
        (Target::Provider(ProviderKind::DrivingLicense), "drivingLicense", &[None], err("RuntimeException", "driving_license.usa.null resulted in null expression")),
        (Target::Provider(ProviderKind::Finance), "iban", &[None], npe("Cannot invoke \"String.length()\" because \"this.aValue\" is null")),
        (Target::Provider(ProviderKind::Fingerprint), "png", &[Some("1"), Some("1"), None], npe("Cannot invoke \"net.datafaker.providers.base.Fingerprint$PatternType.ordinal()\" because \"patternType\" is null")),
        (Target::Provider(ProviderKind::Fingerprint), "base64", &[Some("1"), Some("1"), None], npe("Cannot invoke \"net.datafaker.providers.base.Fingerprint$PatternType.ordinal()\" because \"patternType\" is null")),
        (Target::Provider(ProviderKind::Http), "userAgent", &[None], npe("Cannot read field \"key\" because \"browser\" is null")),
        (Target::Provider(ProviderKind::Http), "responseBody", &[None], npe("Cannot invoke \"String.contains(java.lang.CharSequence)\" because \"contentType\" is null")),
        (Target::Provider(ProviderKind::Image), "base64", &[None], npe("Cannot read field \"imageType\" because \"config\" is null")),
        (Target::Provider(ProviderKind::Internet), "safeEmailAddress", &[None], npe("Cannot invoke \"java.lang.CharSequence.toString()\" because \"<parameter1>\" is null")),
        (Target::Provider(ProviderKind::Locality), "localeStringWithRandom", &[None], npe("Cannot invoke \"java.util.random.RandomGenerator.nextInt(int)\" because \"random\" is null")),
        (Target::Provider(ProviderKind::Options), "option", &[None], npe("Cannot invoke \"java.lang.Class.getEnumConstants()\" because \"klass\" is null")),
        (Target::Provider(ProviderKind::Options), "option", &[None, None], LITERAL),
        (Target::Provider(ProviderKind::Options), "subset", &[Some("1"), None], npe("Cannot read the array length because \"<parameter1>\" is null")),
        (Target::Provider(ProviderKind::Options), "nextElement", &[None], npe("Cannot invoke \"java.util.List.size()\" because \"list\" is null")),
        (Target::Provider(ProviderKind::Text), "text", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Text$TextRuleConfig.getFixedNumberOfCharacters()\" because \"textRuleConfig\" is null")),
        (Target::Provider(ProviderKind::Time), "future", &[Some("1"), None], npe("Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")),
        (Target::Provider(ProviderKind::Time), "future", &[Some("1"), None, None], npe("Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")),
        (Target::Provider(ProviderKind::Time), "future", &[Some("1"), Some("2"), None], npe("Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")),
        (Target::Provider(ProviderKind::Time), "future", &[Some("1"), Some("2"), None, None], npe("Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")),
        (Target::Provider(ProviderKind::Time), "past", &[Some("1"), None], npe("Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")),
        (Target::Provider(ProviderKind::Time), "past", &[Some("1"), None, None], npe("Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")),
        (Target::Provider(ProviderKind::Time), "past", &[Some("1"), Some("2"), None], npe("Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")),
        (Target::Provider(ProviderKind::Time), "past", &[Some("1"), Some("2"), None, None], npe("Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "future", &[Some("1"), None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "future", &[Some("1"), None, None], LITERAL),
        (Target::Provider(ProviderKind::TimeAndDate), "future", &[Some("1"), Some("2"), None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toChronoUnit()\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "future", &[Some("1"), Some("2"), None, None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toChronoUnit()\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "past", &[Some("1"), None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "past", &[Some("1"), None, None], LITERAL),
        (Target::Provider(ProviderKind::TimeAndDate), "past", &[Some("1"), Some("2"), None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "past", &[Some("1"), Some("2"), None, None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "birthday", &[None], npe("pattern")),
        (Target::Provider(ProviderKind::TimeAndDate), "birthday", &[Some("1"), Some("2"), None], npe("pattern")),
        (Target::Provider(ProviderKind::TimeAndDate), "duration", &[Some("1"), None], npe("unit")),
        (Target::Provider(ProviderKind::TimeAndDate), "duration", &[Some("1"), Some("2"), None], npe("unit")),
        (Target::Provider(ProviderKind::Unique), "fetchFromYaml", &[None], npe("Cannot invoke \"String.length()\" because \"string\" is null")),
        (Target::Provider(ProviderKind::Vehicle), "model", &[None], err("RuntimeException", "vehicle.models_by_make.null resulted in null expression")),
        (Target::Provider(ProviderKind::Vehicle), "licensePlate", &[None], err("RuntimeException", "vehicle.license_plate_by_state.null resulted in null expression")),
        (Target::RandomService, "nextInt", &[None], npe("Cannot invoke \"net.datafaker.service.Range.cast(java.util.function.Function)\" because \"range\" is null")),
        (Target::RandomService, "nextLong", &[None], npe("Cannot invoke \"net.datafaker.service.Range.from()\" because \"range\" is null")),
        (Target::Provider(ProviderKind::FinancialTerms), "noun", &[None], npe("Cannot invoke \"net.datafaker.providers.base.FinancialTerms$Category.getName()\" because \"category\" is null")),
        (Target::Provider(ProviderKind::FinancialTerms), "verb", &[None], npe("Cannot invoke \"net.datafaker.providers.base.FinancialTerms$Category.getName()\" because \"category\" is null")),
        (Target::Provider(ProviderKind::FinancialTerms), "adjective", &[None], npe("Cannot invoke \"net.datafaker.providers.base.FinancialTerms$Category.getName()\" because \"category\" is null")),
        (Target::Provider(ProviderKind::Planet), "symbol", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "classification", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "discoveryYear", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "meanDistanceFromTheSun", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "lengthOfDay", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "equatorialRadius", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "surfaceArea", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "volume", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "mass", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "gravitationParameters", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "density", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "equatorialGravity", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "escapeVelocity", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "rotationPeriod", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "orbitalPeriod", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "meanOrbitalSpeed", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "eccentricity", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "inclination", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "axialTilt", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "meanSurfaceTemperature", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "meanAirTemperature", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "atmosphericComposition", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "numberOfKnownMoons", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "rings", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Planet), "planetaryDiscriminant", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null")),
        (Target::Provider(ProviderKind::Aviation), "airport", &[None], npe("Cannot invoke \"net.datafaker.providers.base.AviationCodeType.ordinal()\" because \"codeType\" is null")),
        (Target::Provider(ProviderKind::Finance), "creditCard", &[None], npe("Cannot invoke \"net.datafaker.providers.base.Finance$CreditCardType.toString()\" because \"creditCardType\" is null")),
        (Target::Provider(ProviderKind::Internet), "port", &[None], npe("Cannot read field \"from\" because \"range\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "between", &[None, None], npe("Cannot invoke \"java.time.Instant.isBefore(java.time.Instant)\" because \"to\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "between", &[None, None, None], npe("Cannot invoke \"java.time.Instant.isBefore(java.time.Instant)\" because \"to\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "period", &[None, None], npe("Cannot invoke \"java.time.Period.minus(java.time.temporal.TemporalAmount)\" because \"max\" is null")),
        (Target::Provider(ProviderKind::DateAndTime), "period", &[None, None], npe("Cannot invoke \"java.time.Period.minus(java.time.temporal.TemporalAmount)\" because \"max\" is null")),
        (Target::Provider(ProviderKind::Time), "between", &[None, None], npe("Cannot invoke \"java.time.LocalTime.isBefore(java.time.LocalTime)\" because \"to\" is null")),
        (Target::Provider(ProviderKind::Time), "between", &[None, None, None], npe("Cannot invoke \"java.time.LocalTime.isBefore(java.time.LocalTime)\" because \"to\" is null")),
        (Target::Provider(ProviderKind::Twitter), "createdTime", &[Some("true"), None, None], npe("Cannot invoke \"java.util.Date.getTime()\" because \"base\" is null")),
        (Target::Provider(ProviderKind::Twitter), "text", &[None, Some("1"), Some("1")], err("IllegalArgumentException", "Min (3) > Max (1)")),
        (Target::Provider(ProviderKind::IdNumber), "valid", &[None], err("IllegalArgumentException", "ID Number generation not supported for country ''")),
        (Target::Provider(ProviderKind::IdNumber), "peselNumber", &[None, None], npe("Cannot invoke \"java.time.LocalDate.getYear()\" because \"birthDate\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "future", &[Some("1"), None, Some("yyyy")], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::DateAndTime), "future", &[Some("1"), None, Some("yyyy")], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::DateAndTime), "future", &[Some("1"), Some("DAYS"), None], LITERAL),
        (Target::Provider(ProviderKind::TimeAndDate), "future", &[Some("1"), Some("DAYS"), None], LITERAL),
        (Target::Provider(ProviderKind::DateAndTime), "past", &[Some("1"), Some("DAYS"), None], LITERAL),
        (Target::Faker, "csv", &[None, Some("x"), Some("true"), Some("1"), Some("a"), Some("b")], LITERAL),
        (Target::Faker, "templatify", &[Some("abc"), None], npe("Cannot invoke \"java.util.Map.containsKey(Object)\" because \"optionsMap\" is null")),
        (Target::Faker, "resolve", &[Some("no.such"), None], npe("Cannot invoke \"java.util.function.Supplier.get()\" because \"exceptionMessage\" is null")),
        (Target::Provider(ProviderKind::Options), "option", &[None, None, None], LITERAL),
        (Target::Provider(ProviderKind::DateAndTime), "future", &[None, None], LITERAL),
        (Target::Provider(ProviderKind::IdNumber), "peselNumber", &[None, Some("ANY")], LITERAL),
        (Target::Provider(ProviderKind::Internet), "image", &[None, Some("1"), Some("x")], LITERAL),
        (Target::Provider(ProviderKind::Http), "responseBody", &[None, Some("x")], LITERAL),
        (Target::Provider(ProviderKind::DateAndTime), "future", &[Some("1"), Some("2"), Some("DAYS"), None], LITERAL),
        (Target::Provider(ProviderKind::DateAndTime), "past", &[Some("1"), Some("2"), Some("DAYS"), None], LITERAL),
        (Target::Provider(ProviderKind::DateAndTime), "between", &[None, None, Some("yyyy")], npe("Cannot invoke \"java.util.Date.before(java.util.Date)\" because \"to\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "future", &[Some("1"), Some("2"), Some("DAYS"), None], LITERAL),
        (Target::Provider(ProviderKind::TimeAndDate), "past", &[Some("1"), Some("2"), Some("DAYS"), None], LITERAL),
        (Target::Provider(ProviderKind::TimeAndDate), "between", &[None, None, Some("yyyy")], npe("Cannot invoke \"java.time.Instant.isBefore(java.time.Instant)\" because \"to\" is null")),
        (Target::Provider(ProviderKind::Time), "future", &[Some("1"), Some("DAYS"), None], LITERAL),
        (Target::Provider(ProviderKind::Time), "future", &[Some("1"), Some("2"), Some("DAYS"), None], LITERAL),
        (Target::Provider(ProviderKind::Time), "past", &[Some("1"), Some("DAYS"), None], LITERAL),
        (Target::Provider(ProviderKind::Time), "between", &[None, None, Some("HH")], npe("Cannot invoke \"java.time.LocalTime.isBefore(java.time.LocalTime)\" because \"to\" is null")),
        (Target::Provider(ProviderKind::Twitter), "createdTime", &[Some("false"), None, None], npe("Cannot invoke \"java.util.Date.getTime()\" because \"base\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "future", &[Some("1"), None, None, None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "past", &[Some("1"), None, None, None], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
        (Target::Provider(ProviderKind::TimeAndDate), "past", &[Some("1"), None, Some("yyyy")], npe("Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")),
    ];
    for (target, name, args, expected) in cases {
        let faker = Faker::with_seed(5);
        let actual = match call(&faker, *target, name, args) {
            Some(Ok(JValue::Null)) => None,
            other => other,
        };
        assert_eq!(&actual, expected, "{target:?}.{name}({args:?})");
    }
}

#[test]
fn a_null_argument_concatenates_as_null_or_reads_as_the_default() {
    let faker = Faker::with_seed(2);
    let text = |target: Target, name: &str, args: &[Option<&str>]| match call(&faker, target, name, args) {
        Some(Ok(JValue::Str(s))) => s,
        other => panic!("{name}: {other:?}"),
    };
    let domain = Target::Provider(ProviderKind::Domain);
    assert!(text(domain, "firstLevelDomain", &[None]).starts_with("null."));
    assert!(text(domain, "fullDomain", &[None]).contains(".null."));
    let internet = Target::Provider(ProviderKind::Internet);
    assert_eq!(text(internet, "image", &[Some("1"), Some("2"), None]), "https://picsum.photos/seed/null/1/2");
    assert_eq!(text(internet, "macAddress", &[None]).len(), 17);
    assert!(text(internet, "slug", &[None, None]).contains('_'));
    assert!(text(internet, "slug", &[None, Some("-")]).contains('-'));
    assert!(!text(internet, "userAgent", &[None]).is_empty());
    assert!(text(Target::Provider(ProviderKind::Address), "latLon", &[None]).contains("null"));
    assert!(text(Target::Provider(ProviderKind::Tire), "code", &[None]).starts_with("null"));
    assert!(text(Target::Provider(ProviderKind::Twitter), "getLink", &[None, Some("5")]).starts_with("https://twitter.com/null/"));
    assert!(text(Target::Provider(ProviderKind::File), "fileName", &[None, None, None, None]).contains('/'));
    let options = Target::Provider(ProviderKind::Options);
    assert_eq!(call(&faker, options, "subset", &[Some("1"), None, None]), Some(Ok(JValue::List(vec![JValue::Null]))));
    assert!(matches!(call(&faker, options, "subset", &[Some("2"), Some("a"), None, Some("b"), None]), Some(Ok(JValue::List(l))) if l.len() == 2));
    assert!(matches!(call(&faker, Target::Provider(ProviderKind::Locality), "allSupportedLocales", &[None]), Some(Ok(JValue::List(l))) if !l.is_empty()));
    assert_eq!(call(&faker, Target::Provider(ProviderKind::Credentials), "userId", &[None]), Some(Ok(JValue::Null)));
}

#[test]
fn a_null_random_leaves_the_locales_unshuffled() {
    let faker = Faker::with_seed(4);
    let locality = Target::Provider(ProviderKind::Locality);
    assert_eq!(
        call(&faker, locality, "localeStringWithoutReplacement", &[None]),
        npe("Cannot invoke \"java.util.random.RandomGenerator.nextInt(int)\" because \"rnd\" is null")
    );
    // The shuffle started over and threw at its first draw: the next calls walk the locales in order.
    let first = faker.locality().all_supported_locales()[0];
    assert_eq!(call(&faker, locality, "localeStringWithoutReplacement", &[]), Some(Ok(JValue::str(first))));
}

#[test]
fn null_elements_of_the_csv_json_and_jsona_expressions_print_as_null() {
    let faker = Faker::with_seed(3);
    let text = |name: &str, args: &[Option<&str>]| match call(&faker, Target::Faker, name, args) {
        Some(Ok(JValue::Str(s))) => s,
        other => panic!("{name}({args:?}): {other:?}"),
    };
    assert_eq!(text("json", &[None, None]), "{null: null}");
    assert_eq!(text("json", &[None, Some("a")]), "{null: \"a\"}");
    assert_eq!(text("json", &[Some("a"), None, None, Some("b")]), "{\"a\": null, null: \"b\"}");
    assert_eq!(text("jsona", &[None, None, None]), "{null: null}");
    assert_eq!(text("jsona", &[None, Some("a"), None]), "{\"a\": null}");
    assert_eq!(text("jsona", &[Some("1"), None, None]), "{null: [null]}");
    assert_eq!(text("jsona", &[Some("2"), None, Some("a")]), "{null: [\"a\", \"a\"]}");
    assert_eq!(text("jsona", &[Some("1"), Some("a"), None]), "{\"a\": [null]}");
    assert_eq!(text("csv", &[Some("1"), None, None]), "null\nnull");
    assert_eq!(text("csv", &[Some("1"), Some("a"), None]), "\"a\"\nnull");
    assert_eq!(text("csv", &[Some("2"), None, None, None, None]), "null;null\nnull;null\nnull;null");
    assert_eq!(
        call(&faker, Target::Faker, "jsona", &[None, None]),
        Some(Err(("IllegalArgumentException".into(), "Total number of field names and field values should be dividable by 3 (received 2 field expressions: [null, null])".into())))
    );
    assert_eq!(
        call(&faker, Target::Faker, "json", &[None, None, None]),
        Some(Err(("IllegalArgumentException".into(), "Total number of field names and field values should be even (received 3 fields: [null, null, null])".into())))
    );
}
