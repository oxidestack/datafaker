//! Values MockServer 8.0.0's oracle produced with Datafaker 2.7.0 (`templateFakerSeed`): a fresh
//! `new Faker(new Random(seed))`, then `name().firstName()` and `internet().emailAddress()` on that same faker, in
//! that order (one template rendering both).

use datafaker::Faker;

#[test]
fn seed_9999() {
    let faker = Faker::with_seed(9999);
    assert_eq!(faker.name().first_name().unwrap(), "Kermit");
    assert_eq!(faker.internet().email_address().unwrap(), "betty.vandervort@hotmail.com");
}

#[test]
fn seed_4242() {
    let faker = Faker::with_seed(4242);
    assert_eq!(faker.name().first_name().unwrap(), "Elinor");
    assert_eq!(faker.internet().email_address().unwrap(), "shaun.pagac@gmail.com");
}
