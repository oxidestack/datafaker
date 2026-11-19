//! Tests of Datafaker 2.7.0's themed provider classes (tranche T4) that scripts/gen-providers.py does not read:
//! provider-list cases with enum arguments or non-literal keys, the nested class for another locale, and assertions
//! other than a regex. Unseeded fakers as upstream (`EntertainmentFaker`, `FoodFaker`, … are the one `Faker` here);
//! repeated tests run their body as often.

mod common;

use common::{assert_matches, provider_list};
use datafaker::providers::{CoffeeCountry, DiseaseType, DuneQuote, DuneSaying, TronAlternateCharacterSpelling, TronCharacter, TronQuote};
use datafaker::{Faker, Locale};

/// The strings of a YAML list (`AbstractProviderListTest.getBaseList`).
fn base_list(faker: &Faker, key: &str) -> Vec<String> {
    match faker.fake_values_service().fetch_object(key) {
        Some(datafaker::yaml::Value::List(l)) => l.iter().map(|v| v.to_java_string()).collect(),
        other => panic!("{key} is not a list: {other:?}"),
    }
}

/// `DuneTest.providerListTest`: the cases with a `Quote` or a `Saying` (the others are generated).
#[test]
fn dune_enum_provider_list() {
    let f = Faker::new();
    let dune = f.dune();
    assert_eq!(DuneQuote::VALUES.len(), 18);
    for &quote in DuneQuote::VALUES {
        provider_list(&f, &format!("dune.quotes.{}", quote.yaml_key()), &dune.quote_of(quote).unwrap(), "");
    }
    assert_eq!(DuneSaying::VALUES.len(), 5);
    for &saying in DuneSaying::VALUES {
        provider_list(&f, &format!("dune.sayings.{}", saying.yaml_key()), &dune.saying_of(saying).unwrap(), "");
    }
}

/// `TronTest.providerListTest`: the cases with an `AlternateCharacterSpelling`, a `Character` or a `Quote`.
#[test]
fn tron_enum_provider_list() {
    let f = Faker::new();
    let tron = f.tron();
    assert_eq!(TronAlternateCharacterSpelling::VALUES.len(), 8);
    for &spelling in TronAlternateCharacterSpelling::VALUES {
        let key = format!("tron.alternate_character_spellings.{}", spelling.yaml_key());
        provider_list(&f, &key, &tron.alternate_character_spelling_of(spelling).unwrap(), "");
    }
    for (character, key) in [
        (TronCharacter::Other, "tron.characters.other"),
        (TronCharacter::Program, "tron.characters.programs"),
        (TronCharacter::User, "tron.characters.users"),
    ] {
        provider_list(&f, key, &tron.character_of(character).unwrap(), "");
    }
    assert_eq!(TronQuote::VALUES.len(), 15);
    for &quote in TronQuote::VALUES {
        provider_list(&f, &format!("tron.quotes.{}", quote.yaml_key()), &tron.quote_of(quote).unwrap(), "");
    }
}

/// `CoffeeTest.providerListTest`: the `region(Coffee.Country)` cases.
#[test]
fn coffee_region_provider_list() {
    let f = Faker::new();
    assert_eq!(CoffeeCountry::VALUES.len(), 18);
    for &country in CoffeeCountry::VALUES {
        let key = format!("coffee.regions.{}", country.constant().to_lowercase());
        provider_list(&f, &key, &f.coffee().region_of(country).unwrap(), "");
    }
}

/// The no-argument overloads draw an enum constant and resolve its list.
#[test]
fn enum_drawing_overloads() {
    let f = Faker::new();
    for _ in 0..20 {
        assert!(!f.dune().quote().unwrap().is_empty());
        assert!(!f.dune().saying().unwrap().is_empty());
        assert!(!f.tron().character().unwrap().is_empty());
        assert!(!f.tron().quote().unwrap().is_empty());
        assert!(!f.tron().alternate_character_spelling().unwrap().is_empty());
        assert!(!f.coffee().region().unwrap().is_empty());
        assert!(!f.star_wars().quotes().unwrap().is_empty());
    }
}

/// `DiseaseTest.providerListTest`: keys from `DiseaseType.yamlKey`.
#[test]
fn disease_provider_list() {
    let f = Faker::new();
    let disease = f.disease();
    provider_list(&f, DiseaseType::InternalDiseaseKey.yaml_key(), &disease.internal_disease().unwrap(), "");
    provider_list(&f, DiseaseType::NeurologicalDiseaseKey.yaml_key(), &disease.neurology().unwrap(), "");
    provider_list(&f, DiseaseType::SurgicalDiseaseKey.yaml_key(), &disease.surgery().unwrap(), "");
    provider_list(&f, DiseaseType::PaediatricDiseaseKey.yaml_key(), &disease.paediatrics().unwrap(), "");
    let key = DiseaseType::GynecologyAndObstetricsDiseaseKey.yaml_key();
    provider_list(&f, key, &disease.gynecology_and_obstetrics().unwrap(), "");
    let key = DiseaseType::OphthalmologyAndOtorhinolaryngologyDiseaseKey.yaml_key();
    provider_list(&f, key, &disease.ophthalmology_and_otorhinolaryngology().unwrap(), "");
    provider_list(&f, DiseaseType::DermatologyDiseaseKey.yaml_key(), &disease.dermatology().unwrap(), "");
}

/// `DiseaseTest.testAnyDisease`: one of the seven lists.
#[test]
fn disease_test_any_disease() {
    let f = Faker::new();
    let all: Vec<String> = DiseaseType::VALUES.iter().flat_map(|t| base_list(&f, t.yaml_key())).collect();
    for _ in 0..10 {
        let any = f.disease().any_disease().unwrap();
        assert!(!any.trim().is_empty());
        assert!(all.contains(&any), "{any}");
    }
}

/// `DiseaseTest.testDiseaseCodes`.
#[test]
fn disease_test_disease_codes() {
    let f = Faker::new();
    for _ in 0..10 {
        assert_matches(r"^[A-Z][0-9]{1,2}(\.[0-9])?$", &f.disease().icd10().unwrap());
    }
}

/// `MedicalProcedureTest.testProcedureCodes`.
#[test]
fn medical_procedure_test_procedure_codes() {
    let f = Faker::new();
    for _ in 0..100 {
        assert_matches(r"^[a-hj-np-zA-HJ-NP-Z0-9]{7}$", &f.medical_procedure().icd10().unwrap());
    }
}

/// `AvatarTest.testAvatar`.
#[test]
fn avatar_test_avatar() {
    for _ in 0..10 {
        let avatar = Faker::new().avatar().image().unwrap();
        assert_matches(r"^https://robohash.org/[a-z]{8}.png$", &avatar);
    }
}

/// `OscarMovieTest.movieName` and `quote`: `isNullOrEmpty` is false.
#[test]
fn oscar_movie_movie_name_and_quote() {
    let oscar_movie = Faker::new();
    let oscar_movie = oscar_movie.oscar_movie();
    for _ in 0..10 {
        assert!(!oscar_movie.movie_name().unwrap().is_empty());
        assert!(!oscar_movie.quote().unwrap().is_empty());
    }
}

/// `CheeseTest.shouldReturnCheeseWedge`.
#[test]
fn cheese_should_return_cheese_wedge() {
    let f = Faker::new();
    let wedge = f.cheese().wedge().unwrap();
    for field in [
        &wedge.name,
        &wedge.producer,
        &wedge.r#type,
        &wedge.texture,
        &wedge.color,
        &wedge.milk,
        &wedge.rind,
        &wedge.rind_edibility,
        &wedge.packaging,
    ] {
        assert!(!field.is_empty());
    }
    assert!(wedge.to_string().starts_with(&format!("Wedge[name={}, type={}, ", wedge.name, wedge.r#type)));
}

/// `Dota2Test.dota2HeroQuote`: every hero's snake_case name has quotes.
#[test]
fn dota2_hero_quote() {
    let f = Faker::new();
    let dota2 = f.dota2();
    for _ in 0..5 {
        let hero = dota2.hero().unwrap().replace(' ', "_").to_lowercase();
        assert!(!dota2.hero_quote(&hero).unwrap().is_empty());
    }
}

/// `HearthstoneTest.battlegroundsScoreTest`, `standardRankTest`, `wildRankTest`.
#[test]
fn hearthstone_scores_and_ranks() {
    let f = Faker::new();
    let score = f.hearthstone().battlegrounds_score().unwrap();
    assert!((0..=16000).contains(&score), "{score}");
    assert_matches("[ A-Za-z0-9]+", &f.hearthstone().standard_rank().unwrap());
    assert_matches("[ A-Za-z0-9]+", &f.hearthstone().wild_rank().unwrap());
}

/// `FootballTest.FootballInGreekTest`.
#[test]
fn football_in_greek_provider_list() {
    let f = Faker::with_locale(Locale::new("el", "GR"));
    let football = f.football();
    provider_list(&f, "football.coaches", &football.coaches().unwrap(), "");
    provider_list(&f, "football.competitions", &football.competitions().unwrap(), "");
    provider_list(&f, "football.players", &football.players().unwrap(), "");
    provider_list(&f, "football.positions", &football.positions().unwrap(), "");
    provider_list(&f, "football.teams", &football.teams().unwrap(), "");
}
