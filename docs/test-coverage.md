# Test coverage

Datafaker 2.7.0's own tests (`upstream/datafaker/src/test/`), mapped to tranches and, where ported, to their ports.

Counted on 2026-10-06 from the tree: 364 files (363 Java, 1 Kotlin) with 1,129 test annotations (`@Test`,
`@ParameterizedTest`, `@RepeatedTest`, `@TestFactory`) and 922 `TestSpec.of` provider-list cases, which
`AbstractProviderListTest` turns into parameterized runs (each checks that a provider method returns a value from
its YAML list). A parameterized test counts once here, not per argument.

Rechecked 2026-10-11: 1,118 of the 1,129 are ported and pass; the other 11 are listed below with their reason
(`JavaNamesTest.nullString`, `AviationTest.flight_nullCodeNotAllowed`, `RelationshipTest`'s two Mockito cases and
the 7 T6 tests).

Every test class ends in one of three states: ported and passing, adapted (JVM-only surface such as reflection,
`ServiceLoader`, `java.util.Locale` objects or Mockito, which Rust has no counterpart for), or listed here with a reason.

## Runtime dependencies of `src/main`

Read from `pom.xml` and the imports of `src/main/java` (2026-10-06). Everything else in `pom.xml` is test scope.

| Dependency | Used by | What the port needs |
|---|---|---|
| SnakeYAML 2.6 | `FakeValues` only: loads the 391 locale files | A YAML parser (the `yaml-rust2` crate, event API) plus SnakeYAML's YAML 1.1 scalar resolution (booleans `yes`/`on`, octal and sexagesimal ints, floats, nulls), done in `src/yaml.rs`. The files are embedded at build time. |
| RgxGen 3.1 | `FakeValuesService.regexify`, so `regexify`, slash-delimited YAML values `/…/` and every provider that calls them (Code, Finance, Vehicle, Passport, Aviation, Medical, Credentials.userId, IdNumbers, …) | Its parser and `GenerationVisitor`, draw for draw, with its Unicode category tables: ported in `src/rgxgen/`. Not ported: a negative lookaround containing an alternation (RgxGen checks it with `java.util.regex`), `RgxGenProperties`, `iterateUnique`, `generateNotMatching`; Datafaker reaches none of them with default use. A separate crate is possible later; it has no other consumer today. |
| libphonenumber 9.0.33 (MockServer 9.2.0 ships 9.0.41) | `PhoneNumber` and `PhoneNumberGenerator` (T3): valid numbers per region | The `libphonenumber` crate (a path dependency on a sibling checkout): libphonenumber 9.0.41's metadata, `parse`, `isValidNumber`, number types and `format`, the version MockServer runs Datafaker with. `getMetadataForRegion`, which upstream calls by reflection, is public there. `scripts/record.sh` puts 9.0.41 on the classpath. |
| JDK: `java.util.Random`, `java.util.random.RandomGenerator`, `SplittableRandom` | `RandomService` (over any `RandomGenerator`), RgxGen, `Locality.localeStringWithRandom` | Bit-exact ports, including `RandomGenerator`'s bounded defaults (`src/java/random.rs`); `nextGaussian` only on `Random`. |
| JDK: `java.net.IDN` | `FakerIDN` (Internet domains and emails) | `src/idn.rs`: Punycode and IDNA2003 nameprep on Unicode 3.2 tables (`src/idn_tables.rs`) read back from the JDK's `IDN.toASCII` by `scripts/idn-tables.sh`. |
| JDK: `java.math.BigDecimal` | `Number.randomDouble`, `decimalBetween` | `src/java/big_decimal.rs` over `num-bigint` (as java-compat-rs uses it): `valueOf`, `add`, `subtract`, `multiply`, `setScale(HALF_DOWN)`, `doubleValue`, `longValue`. |
| JDK: `java.text.Normalizer`, `java.util.regex` | `Internet.stripAccents` and others | `unicode-normalization` crate; each pattern a hand-written matcher. |
| JDK: `MessageDigest` (MD5; MD2, SHA-1, SHA-256, SHA-384, SHA-512) | `Internet.uuidv3` (`UUID.nameUUIDFromBytes`), `Hashing` | `src/md5.rs`; `src/digests.rs` (RFC 1319 and FIPS 180-4, MD2's table read from the JDK), checked against known answers and JVM vectors. |
| JDK: `Currency.getAvailableCurrencies()`, `Locale.getDisplayLanguage`/`getDisplayCountry`, `DecimalFormatSymbols` | `Money` (and `Country.currency`), `Locality.displayName`, `Commerce.price` (`DecimalFormat`), `Address.latitude`/`longitude` (`String.format(locale, "%.8f")`) | `src/providers/jdk_tables.rs`, written by `scripts/jdk-tables.sh` (JDK 25.0.2): currencies in code order (upstream's order is a `HashSet` of identity hashes), display names of `Locality`'s tags, each locale's zero digit, decimal separator and negative prefix. `DecimalFormat` (`HALF_EVEN` on the exact value) and `Formatter` (`HALF_UP` on the shortest digits) in `src/providers/mod.rs`. |
| JDK: `java.time.Duration` | `FakeDuration`, the date providers | `src/java/duration.rs` (`toString`, `Duration.of(amount, unit)`). |
| JDK: `java.time` (`Instant`, `LocalDate`, `LocalTime`, `LocalDateTime`, `Period`, `ChronoUnit`, `DateTimeFormatter.ofPattern`), `java.util.Date`, `java.sql.Timestamp`, `java.text.SimpleDateFormat`, `TimeUnit`, the system clock and default zone | `TimeAndDate`, `DateAndTime`, `Time`, `Business.creditCardExpiry`, `Twitter` (and the T2 generators' birthdays) | `src/java/time/`: values, arithmetic and `toString()`s; `DateTimeFormatter` formatting (every pattern letter, `en_US` texts and Sunday-first weeks, optional sections, padding, the JDK's error texts); `SimpleDateFormat` formatting on the Julian/Gregorian calendar; `LocalDate.parse` as strict `ISO_LOCAL_DATE`, `Instant.parse` as `ISO_INSTANT`; `DateTimeFormatter` parsing (`parse.rs`: every printer-parser's strict `parse`, adjacent value parsing, `Parsed.resolve` with the SMART and STRICT resolvers on the ISO chronology, week fields, day periods, offsets and fixed zone ids) and `java.util.Date.parse` (the lenient parser of `new Date(String)`, which expressions use to reach `DateAndTime`'s `Date` overloads). Zones (`zone.rs`): `ZoneId`/`ZoneOffset`/`ZoneRules` over tzdb 2026b and en_US names as MockServer 8.0.0's image (Temurin 26.0.2) has them (`tzdb.bin`, `scripts/tz-tables.sh mockserver/mockserver:8.0.0`), `java.util.TimeZone`'s reading of them for `Date`/`Timestamp`/`SimpleDateFormat`, region ids and zone names in parsing. The default zone is `GMT` (as in MockServer 8.0.0's image) unless set (`set_default_zone`, `ZoneId::host()`), the FORMAT locale `en_US` (README, "Differences"); `scripts/record.sh` pins both (the zone from a file's `# zone` line). |
| JDK: reflection (`getMethods`, `Method.invoke`, `getReturnType`) | `FakeValuesService` expression resolution, `ObjectMethods`, `FakeResolver` | Per-provider method tables (`src/dispatch.rs`, each provider's `METHODS`) with Java's overload choice and argument coercion; resolution order in `src/expr.rs`. `JavaObjectTransformer` and `FakeResolver` over user classes: the `JavaObject` and `FakeForSchema` traits (T5). Custom providers and other user objects reached by expressions (`AbstractProvider` subclasses, the current object of `resolve`): the `dispatch::JavaClass` trait, a method table per class; `ObjectMethods` over them (`object_methods`, return types and getters of user objects). |
| JDK: `Locale.getISOLanguages()`, `getISOCountries()` | `Nation.isoLanguage`, `isoCountry` | `src/providers/iso_codes.rs`, the JDK's arrays in its order, written by `scripts/iso-codes.sh` (JDK 25.0.2). |
| JDK: `javax.imageio` (`ImageIO.write` of BMP, GIF, JPEG, PNG, TIFF), `java.awt.image.BufferedImage`, `Graphics2D.fillRect`, `java.util.zip.Deflater`/`CRC32`, `Base64` | `Image`, `Fingerprint` | `src/java/imageio/`: each writer as it runs with default parameters and metadata (GIF with `PaletteBuilder` and `LZWCompressor`; JPEG on a port of the IJG libjpeg 6b the JDK bundles, as `imageioJPEG.c` sets it up), `src/java/deflater.rs` (zlib 1.3.1's `deflate.c`/`trees.c`, the JDK's bundled zlib, byte for byte; levels 1 to 9). |
| JDK: `ServiceLoader` | `AbstractProvider.loadGenerators` (IdNumber generators) | `idnumbers::generators()`, a static registry in the order of `META-INF/services` (T2). |

## Tranches

| Tranche | Scope | Classes | Test annotations | Provider-list cases |
|---|---|---|---|---|
| T0 | Core: services, helpers, the `Faker` root, and the providers MockServer reaches (Name, Internet, Lorem, Credentials, Text, Number, Options, `regexify`) | 26 | 308 | 2 |
| T1 | The other base providers (`providers/base`), YAML-driven or algorithmic (Address, Finance, Code, Date, IdNumber facade, …) | 133 | 557 | 356 |
| T2 | National identity numbers (`idnumbers`) | 24 | 84 | 0 |
| T3 | Phone numbers (`PhoneNumber`, `PhoneNumberGenerator`) on the `libphonenumber` crate | 2 | 23 | 0 |
| T4 | Themed providers: entertainment, video games, food, sport, healthcare | 133 | 22 | 564 |
| T5 | Transformations and formats (JSON, CSV, SQL, YAML, XML, Java objects), sequences, `@FakeForSchema` annotations, integration tests | 18 | 128 | 0 |
| T6 | Documentation and release scripts of the upstream repository | 2 | 7 | 0 |
| support | Test support code (base classes, DTOs, assertion helpers, log handlers): no tests of their own | 26 | 0 | 0 |

## Differential vectors

`tests/vectors/*.tsv` (all but `idn.tsv`) hold seeded outputs recorded from Datafaker 2.7.0 itself (`scripts/record.sh` runs
`scripts/Vectors.java` against `datafaker-2.7.0.jar` in `~/.m2`); `tests/vectors.rs` replays them. They are recorded on JDK 25.0.2,
except `dates.tsv` and the `dates_<zone>.tsv` files, which their `# image` lines record on the JVM of MockServer
8.0.0's image (Temurin 26.0.2), where `Instant.parse`, the order of `DateAndTime`'s overloads and the tzdb (2026b)
differ (README, "Differences"). Replayed on that JVM
(2026-10-11), every other file gives the same results apart from non-English number formats (`providers_t1b.tsv`,
70 results, and 10 in `providers.tsv`: the image's trimmed JDK has no `jdk.localedata`) and `idnumbers.tsv`'s
recording date. Each row is a
fresh `new Random(seed)`, a sequence of calls on it (through `Random`, `RandomService` or a `Faker` of some locale),
and what each call returned, exceptions included.

| File | Rows | Results | Covers |
|---|---|---|---|
| `random.tsv` | 63 | 898 | `java.util.Random` (every method, bounded variants, `nextGaussian`), `java.util.SplittableRandom` (`RandomGenerator`'s defaults, `split`), `RandomService` over each |
| `faker.tsv` | 180 | 1605 | Name, Internet, Lorem, Credentials, Text, Number, Options, `numerify`/`letterify`/`bothify`/`examplify`, `expression`/`resolve` with method calls, arguments and failures |
| `locales.tsv` | 45 | 495 | de, en_US, el_GR, ru, fa, pt_BR, zh_CN, ja, tr: names, e-mails, domains, URLs (IDN), lorem, titles |
| `regexify.tsv` | 56 | 411 | RgxGen: about 70 patterns, Datafaker's own among them, about 20 of them failing |
| `number.tsv` | 20 | 300 | `Number.randomDouble` (scales -2 to 17, int and long bounds, equal bounds) and `numberBetween` over ranges wider than `Long.MAX_VALUE` |
| `functions.tsv` | 12 | 66 | `csv`, `json`, `jsona` through `expression`: quoting, separators, nested objects, arrays, failures |
| `providers.tsv` | 1715 | 8057 | Every no-argument method of the T1, T3 and T4 providers (generated and hand-written, except those whose result a JVM does not reproduce: `Money`, `Country.currency`, and the methods that read the clock), on seeds 0, 1, 42, 9999, -7 in English and seed 7 in el_GR and de; rows written by `scripts/gen-providers.py`, results by `record.sh` |
| `providers_t1b.tsv` | 117 | 1131 | T1b methods with arguments, other locales (number formats in de, fa, ru, ja, pt_BR, tr, zh_CN, en_US) and expressions: ISBN/EAN/IMEI/IBAN/Luhn/CPF/CNPJ check digits, `Commerce.price`, latitudes and longitudes on 41 more seeds, `Locality`, `Compass` state, `Unique`, failures |
| `providers_t4.tsv` | 44 | 1288 | T4: every constant of the themed enums through the enum overloads (`Dune.quote(Quote)`, `Tron.character(Character)`, `Coffee.region(Country)`, …), draws that pick a constant or a list first (`standardRank`, `tileItemName`, `StarWars.quotes`, `OscarMovie`, `Tea.variety`, `anyDisease`, ICD-10 codes, `Cheese.wedge`) on 10 seeds, `Dota2.heroQuote` with failures, expressions with enum coercion and failures, food/coffee/beer/football lists in hy, pt_BR, hu, uz, he, ja, ko, fr, de |
| `dates.tsv` | 137 | 1059 | `TimeAndDate`, `DateAndTime`, `Time` and `Twitter` on reference dates (`future`/`past`/`between` with a reference, `duration`, `period`, `createdTime`, `text`, `getLink`), failures, clock-free expressions, and the formatters through the pattern overloads (`between(x, x, pattern)` draws nothing): `DateTimeFormatter` over 30 patterns on 7 instants (years 45 BC to AD 12345) and 9 on 5 times, `SimpleDateFormat` over 12 patterns on 7 dates (Julian ones too), `Timestamp`/`Date` `toString`; `LocalDate.parse` (strict `ISO_LOCAL_DATE`: year widths and signs, field widths, resolve failures, error indexes in UTF-16 units, the 64-character abbreviation) through `value.of(LocalDate(…))` |; `DateTimeFormatter.ofPattern(p).parse(text)` through `value.of(Parse(…))` (237 cases: adjacent value parsing, every pattern letter, SMART resolution and its conflicts, offsets, fixed zone ids, day periods, week fields, MJD, numbers past 18 digits; `Parsed` printed with its fields sorted), `new Date(String)` through `value.of(DateText(…))` (71 texts), `DateAndTime`'s `Date` overloads through expressions (14 on 3 seeds), and `Instant.parse` (`ISO_INSTANT`: case, fractions, offsets, `24:00`, leap second, years beyond 9999 and to the instant range, failures with their indexes) through `value.of(Instant(…))` (52 texts), region ids and en_US zone names through `Parse` (30) |
| `dates_new_york.tsv`, `dates_lord_howe.tsv`, `dates_dublin.tsv`, `dates_vancouver.tsv` | 35 | 205 | The default zone America/New_York, Australia/Lord_Howe (half-hour DST), Europe/Dublin (negative DST) and America/Vancouver (tzdb 2026b's permanent -07:00 from 2026-03-08, still named PST/Pacific Standard Time), each recorded with that `-Duser.timezone` (`# zone`): `Date`/`Timestamp` `toString`, `DateTimeFormatter` zone texts, ids and offsets and `SimpleDateFormat` `z`/`Z`/`X` on instants around DST transitions, before 1900 and after 2037, `new Date(String)`, `DateTimeFormatter` parsing with the default zone and `Date` expressions on local times in gaps and overlaps |
| `idnumbers.tsv` | 206 | 1460 | Every country's generator through `IdNumber` (`valid()`, `invalid()`, `valid(IdNumberRequest)` for four age ranges and both genders, edge ages that fail), the facade's per-country methods, `peselNumber(LocalDate, Gender)`, locales without a generator, expressions. Birthdays read the clock: the file carries its recording date (`# today`) and `tests/vectors.rs` replays it on that date (`java::time::with_fixed_clock`) |
| `providers_args.tsv` | 5 | 100 | T1a methods with arguments (`Weather` ranges, `Tire.code` overloads) and expressions: enum coercion (`FinancialTerms`), records (`EmergencyCase`), YAML references, failures |
| `transformations.tsv` | 134 | 134 | T5: 67 named cases on seeds 7 and 42, built by `scripts/Transformations.java` and again by `tests/vectors_transformations.rs` (`scripts/record.sh tests/vectors/transformations.tsv Transformations`): every transformer over values of each runtime type (boxed numbers, `Character`, `BigInteger`/`BigDecimal`, escapes, collections, maps, object and primitive arrays, nested schemas and composite fields), streams and their start/end, rows from lists and fake sequences with `null`s, the 23 SQL dialects (identifier quoting, batches, keyword cases, rows, arrays), and the exceptions (`ClassCastException`s, `NullPointerException`s of `null` names, unknown dialect, infinite sequences, negative stream limits); `FakeCollection`/`FakeStream` draws, null rates, defaults, `toString` and failures |
| `phone_numbers.tsv` | 612 | 2448 | T3: `cellPhone`, `cellPhoneInternational`, `phoneNumberNational`, `phoneNumberInternational` on seeds 1, 42, -7 for every locale of `Locality.allSupportedLocales()`, `PhoneNumberTest`'s locales, the language-only locales of `detectsCountryByLanguage`, the `CAT`/`NEP`/`PAK`/`BORK` country mappings and countries libphonenumber does not know (`NullPointerException`); rows written by `scripts/phone-vectors.py` |
| `images.tsv` | 169 | 169 | `Image` and `Fingerprint` (`scripts/Images.java`, `scripts/record.sh tests/vectors/images.tsv Images`, replayed by `tests/vectors_images.rs`): every no-argument method on 4 seeds, each format at 15 sizes from 1 × 1 to 1000 × 1000 (BMP 2000 × 1000), sizes below 1 (`BufferedImage`'s errors, SVG's empty grid), the builder's checks, fingerprints of each pattern at 4 sizes and their errors; a result over 2000 characters compared by length and SHA-256, then the random's next `nextInt()` |
| `idn.tsv` | 8298 | 8298 | `IDN.toASCII` from the JDK itself (`scripts/idn-tables.sh`, not `record.sh`): crafted labels, every 37th code point up to U+2FFFF, 3,000 random mixed labels |

## Classes

Status: `ported` (with the port's file and how many of the class's tests), a follow-up with what it needs, `Tn` (to do in that tranche), or `support`.

### T0

307 of the 308 tests are ported (2026-10-06; the four that waited for `Address`, `Aviation` and `Finance` came
with T1b, the two date expressions of `FakeValuesServiceTest` with T1c, `RandomServiceTest.customRandomGenerator`
with `SplittableRandom` on 2026-10-07, and with the `JavaClass` user providers on 2026-10-07 the three
`FakeValuesServiceTest` cases with user objects as the current object of `resolve` and `FakerTest.whitelistTest`).
`JavaNamesTest.nullString` has no Rust form. There is no
`GenerationUtilsTest` upstream; `GenerationUtils` is covered through `numerify`.

| Upstream class | Tests | Cases | Status |
|---|---|---|---|
| FakerConcurrencyModificationExceptionTest.java | 2 | 0 | ported: `tests/upstream_service.rs` (2/2) |
| FakerTest.java | 31 | 0 | 31/31 in `tests/upstream_faker.rs` (`issue883Test` on the `ancient.god` key, the `doWith` exception cases as returned errors, `testDeterministicAndNonDeterministicProvidersReturnValues` with the `@Deterministic` methods listed, `whitelistTest` on `Faker::with_white_list` over binary class names) |
| Issue1178Test.java | 8 | 0 | ported: `tests/upstream_faker.rs` (8/8) |
| Issue759Test.java | 1 | 0 | ported: `tests/upstream_faker.rs` (1/1, five threads on one faker, ten repetitions) |
| NoDuplicatesInYmlTest.java | 1 | 0 | ported: `tests/upstream_yml.rs` (1/1, over `resources/` and `tests/resources/`) |
| internal/helper/FakerIDNTest.java | 2 | 0 | ported: `tests/upstream_helpers.rs` (2/2) |
| internal/helper/JavaNamesTest.java | 13 | 0 | ported: `tests/upstream_helpers.rs` (12/13; `nullString` has no Rust form) |
| internal/helper/LazyEvaluatedTest.java | 2 | 0 | ported: `tests/upstream_service.rs` (2/2) |
| internal/helper/WordUtilsTest.java | 2 | 0 | ported: `tests/upstream_helpers.rs` (2/2, null half dropped) |
| providers/base/CredentialsTest.java | 12 | 0 | ported: `tests/upstream_credentials.rs` (12/12; Mockito case adapted, `userId(null)` has no Rust form) |
| providers/base/InternetTest.java | 48 | 0 | ported: `tests/upstream_internet.rs` (48/48; Mockito and `EmailValidator` cases adapted) |
| providers/base/LoremTest.java | 25 | 2 | ported: `tests/upstream_lorem.rs` (25/25 and both list cases) |
| providers/base/NameTest.java | 18 | 0 | ported: `tests/upstream_name.rs` (18/18) |
| providers/base/NumberTest.java | 29 | 0 | ported: `tests/upstream_number.rs` (29/29; the custom generator is `EvenlyDistributedRandomLongGenerator` in `tests/common`) |
| providers/base/OptionsTest.java | 9 | 0 | ported: `tests/upstream_options.rs` (9/9) |
| providers/base/TextTest.java | 19 | 0 | ported: `tests/upstream_text.rs` (19/19) |
| service/EvenlyDistributedRandomLongGeneratorTest.java | 3 | 0 | ported: `tests/upstream_service.rs` (3/3) |
| service/FakeValuesGroupingTest.java | 2 | 0 | ported: `tests/upstream_values.rs` (2/2) |
| service/FakeValuesServiceTest.java | 32 | 0 | 32/32 in `tests/upstream_fake_values_service.rs` (`test.yml` added with `addPath`; Mockito `verify` dropped; `FakerContext` cases on the faker; `DummyService` and the local `Property` provider as `JavaClass`es, `PropertyFaker`'s accessor declared with `declare_provider`); `futureDateExpression`/`pastDateExpression` parse the `Timestamp` text by hand |
| service/FakeValuesTest.java | 13 | 0 | ported: `tests/upstream_values.rs` (13/13) |
| service/FakerContextTest.java | 2 | 0 | ported: `tests/upstream_helpers.rs` (2/2; supported locales read from the embedded file names until `Locality` lands) |
| service/RandomNumbersTest.java | 6 | 0 | ported: `tests/upstream_service.rs` (6/6) |
| service/RandomServiceTest.java | 16 | 0 | ported: `tests/upstream_random_service.rs` (16/16) |
| service/RangeTest.java | 4 | 0 | ported: `tests/upstream_service.rs` (4/4) |
| service/WeightedRandomSelectorTest.java | 7 | 0 | ported: `tests/upstream_service.rs` (7/7) |
| service/files/EnFileTest.java | 1 | 0 | ported: `tests/upstream_service.rs` (1/1) |

### T1

T1a (2026-10-06): 84 provider classes ported, 68 of them generated by `scripts/gen-providers.py` from the Java
sources (every public method resolves a YAML key) and 16 written by hand (a few lines of logic each: Animal, App,
Brand, Computer, Educator, Emergency, FinancialTerms, Mbti, Music, Nation, Relationship, Restaurant, Shakespeare,
Space, Tire, Weather). Their test classes: 85 of 133 rows done (83 classes and the two abstract bases), 75 of the
77 tests and all 288 of their list cases ported; the other two are `RelationshipTest`'s Mockito cases.

T1b (2026-10-06): 31 more provider classes, all written by hand (Address, Aviation, Aws, Azure, Barcode, Bool, CNPJ, CPF, Code, Color, Commerce, Company,
Compass, Country, Domain, DrivingLicense, Drone, FakeDuration, File, Finance, Hashing, Http, LanguageCode,
Locality, Medical, Money, Passport, Planet, Sip, Unique, Vehicle); state upstream keeps on the cached provider
(`Compass`'s chosen point, `Locality`'s shuffle, `Unique`'s remaining values, `Company`'s buzzwords) lives on the
faker. 36 more rows done: 332 of their 333 tests and all 67 list cases (`AviationTest.flight_nullCodeNotAllowed`
has no Rust form; `LocalityTest`'s `SplittableRandom` case came with the `SplittableRandom` port).

T1c (2026-10-07): `TimeAndDate`, `DateAndTime`, `Time`, `Business` and `Twitter`, by hand, on the
`java.time` subset in `src/java/time/`. 5 more rows done, all 70 of their tests and the list case
(`tests/upstream_dates.rs`; `BusinessTest`'s other cases generated). Their `assertValidDate`/`assertValidTime`
parse the text back with `DateTimeFormatter`, as upstream (2026-10-07). The `IdNumber` facade came with T2 (below). `Image` and `Fingerprint` (2026-10-07): 2 more
rows, all 24 tests (`tests/upstream_image.rs`), with every image byte for byte against the JVM
(`tests/vectors/images.tsv`). `CustomFakerTest` (2026-10-07): 10/10 on `JavaClass` user providers.
`ObjectMethodsTest` (2026-10-07): 3/3 on `object_methods` over `JavaClass`es, which declare return
types (`Method::returning`) and getters of user objects (`JavaClass::OBJECTS`).

`tests/upstream_generated.rs` is written by the same script: the provider-list cases of every ported provider's
test class and the tests that only assert that a provider call matches a regex (or is not empty or blank). The
script skips test classes whose hand-written port names them in its first line (`` `NameTest` ``), and reports
what it cannot read (`--report`): nested classes for other locales, cases with arguments, other assertions,
`@ParameterizedTest`s. Those are in `tests/upstream_base_providers.rs` (T1a) and `tests/upstream_algorithmic.rs`
(T1b).

| Upstream class | Tests | Cases | Status |
|---|---|---|---|
| providers/base/AddressTest.java | 46 | 0 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (46/46; `@ValueSource` cases as loops, the `@NullSource` case of `testCountyForWrongZipCode` as the string `null`) |
| providers/base/AncientTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs`; nested `AncientInGreekTest` in `tests/upstream_base_providers.rs` (0/0, 4 list cases) |
| providers/base/AnimalTest.java | 3 | 1 | ported (hand-written provider): `tests/upstream_generated.rs` (3/3, 1 list cases) |
| providers/base/AppTest.java | 2 | 1 | ported (hand-written provider): `tests/upstream_generated.rs` (2/2, 1 list cases) |
| providers/base/ApplianceTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/base/ArtistTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/AustraliaTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/base/AviationTest.java | 9 | 18 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (8/9, 18 list cases; `flight_nullCodeNotAllowed` has no Rust form) |
| providers/base/AwsTest.java | 9 | 2 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (9/9, 2 list cases) |
| providers/base/AzureTest.java | 32 | 0 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (32/32) |
| providers/base/BarcodeTest.java | 11 | 0 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (11/11) |
| providers/base/BloodTypeTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/base/BookTest.java | 1 | 3 | ported (generated provider): `tests/upstream_generated.rs` (1/1, 3 list cases) |
| providers/base/BoolTest.java | 1 | 0 | ported: `tests/upstream_algorithmic.rs` (1/1) |
| providers/base/BrandTest.java | 0 | 3 | ported (hand-written provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/base/BusinessTest.java | 3 | 1 | ported: `tests/upstream_generated.rs` and `tests/upstream_dates.rs` (3/3, 1 list case) |
| providers/base/CNPJTest.java | 6 | 0 | ported: `tests/upstream_algorithmic.rs` (6/6) |
| providers/base/CPFTest.java | 3 | 0 | ported: `tests/upstream_algorithmic.rs` (3/3) |
| providers/base/CameraTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/base/CannabisTest.java | 0 | 10 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 10 list cases) |
| providers/base/CatTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/base/ChiquitoTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/base/CodeTest.java | 14 | 0 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (14/14; commons-validator's ISBN, EAN-13 and Luhn checks written out) |
| providers/base/CoinTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/ColorLocaleTest.java | 0 | 1 | ported: `tests/upstream_algorithmic.rs` (0/0, 1 list case over `internationalFakers()`) |
| providers/base/ColorTest.java | 2 | 1 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (2/2, 1 list case) |
| providers/base/CommerceLocaleTest.java | 0 | 3 | ported: `tests/upstream_algorithmic.rs` (0/0, 3 list cases over `internationalFakers()`) |
| providers/base/CommerceTest.java | 7 | 3 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (7/7, 3 list cases) |
| providers/base/CommunityTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/base/CompanyTest.java | 6 | 3 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (6/6, 3 list cases) |
| providers/base/CompassTest.java | 6 | 0 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (6/6) |
| providers/base/ComputerTest.java | 1 | 6 | ported (hand-written provider): `tests/upstream_generated.rs` (1/1, 6 list cases) |
| providers/base/ConstructionTest.java | 0 | 6 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 6 list cases) |
| providers/base/CosmereTest.java | 0 | 10 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 10 list cases) |
| providers/base/CountryTest.java | 3 | 4 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (3/3, 4 list cases) |
| providers/base/CryptoCoinTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/CultureSeriesTest.java | 0 | 6 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 6 list cases) |
| providers/base/CurrencyTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/base/CustomFakerTest.java | 10 | 0 | ported: `tests/upstream_custom_faker.rs` (10/10; `MyCustomFaker`'s accessors declared with `Faker::declare_provider`, `getProvider(Class, factory)` as `Faker::get_provider`; `addPath(locale, null)` as an empty path) |
| providers/base/DateAndTimeTest.java | 24 | 0 | ported: `tests/upstream_dates.rs` (24/24; parameterized cases as loops) |
| providers/base/DcComicsTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/base/DemographicTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/base/DeviceTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/base/DogTest.java | 0 | 8 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 8 list cases) |
| providers/base/DomainTest.java | 8 | 0 | ported: `tests/upstream_algorithmic.rs` (8/8) |
| providers/base/DrivingLicenseTest.java | 1 | 0 | ported: `tests/upstream_algorithmic.rs` (1/1, `@EnumSource` patterns read from the test source) |
| providers/base/DroneTest.java | 17 | 8 | ported: `tests/upstream_generated.rs` (17/17, 8 list cases) |
| providers/base/DungeonsAndDragonsTest.java | 0 | 9 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 9 list cases) |
| providers/base/DurationTest.java | 4 | 0 | ported: `tests/upstream_algorithmic.rs` (4/4) |
| providers/base/EducatorTest.java | 5 | 0 | ported (hand-written provider): `tests/upstream_generated.rs` (5/5) |
| providers/base/ElectricalComponentsTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/base/EmergencyTest.java | 2 | 3 | ported (hand-written provider): `tests/upstream_generated.rs`; both tests in `tests/upstream_base_providers.rs` (2/2, 3 list cases) |
| providers/base/EmojiTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/base/FamousLastWordsTest.java | 1 | 0 | ported (generated provider): `tests/upstream_generated.rs` (1/1) |
| providers/base/FileTest.java | 7 | 0 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (7/7) |
| providers/base/FinanceTest.java | 14 | 1 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (14/14, 1 list case; iban4j's `IbanRegistry` lengths replaced by the mod-97 check) |
| providers/base/FinancialTermsTest.java | 2 | 0 | ported (hand-written provider): `tests/upstream_generated.rs`; `category` (`@EnumSource`) in `tests/upstream_base_providers.rs` (2/2) |
| providers/base/FingerprintTest.java | 7 | 0 | ported: `tests/upstream_image.rs` (7/7; the image size read from the PNG header where upstream calls `ImageIO.read`; the debug copy to `target/surefire-reports` dropped) |
| providers/base/FunnyNameTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/GarmentSizeTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/GenderTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/base/GreekPhilosopherTest.java | 2 | 2 | ported (generated provider): `tests/upstream_generated.rs`; nested `GreekPhilosopherInGreekTest` in `tests/upstream_base_providers.rs` (2/2, 2 list cases) |
| providers/base/HackerTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/base/HashingTest.java | 6 | 0 | ported: `tests/upstream_generated.rs` (6/6) |
| providers/base/HipsterTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/HobbyTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/HololiveTest.java | 1 | 1 | ported (generated provider): `tests/upstream_generated.rs`; `talent_jaLocale` in `tests/upstream_base_providers.rs` (1/1, 1 list cases) |
| providers/base/HorseTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/base/HouseTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/base/HttpTest.java | 19 | 15 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (19/19, 15 list cases) |
| providers/base/IdNumberTest.java | 38 | 0 | ported (hand-written provider, T2): 7 in `tests/upstream_generated.rs`, 31 in `tests/upstream_idnumbers.rs` (38/38) |
| providers/base/ImageTest.java | 17 | 0 | ported: `tests/upstream_image.rs` (17/17; `ImageAssert` reads sizes from each format's header, the SVG checked by pattern) |
| providers/base/IndustrySegmentsTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/base/InternetPasswordTest.java | 3 | 0 | ported: `tests/upstream_algorithmic.rs` (3/3) |
| providers/base/JobTest.java | 0 | 8 | ported (generated provider): `tests/upstream_generated.rs`; nested `JobInGreekTest` (its 4 cases) in `tests/upstream_base_providers.rs` (0/0, 8 list cases) |
| providers/base/KpopTest.java | 0 | 6 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 6 list cases) |
| providers/base/LanguageCodeTest.java | 1 | 0 | ported: `tests/upstream_algorithmic.rs` (1/1) |
| providers/base/LargeLanguageModelTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/base/LocalityTest.java | 8 | 0 | ported: `tests/upstream_algorithmic.rs` (8/8) |
| providers/base/LocationTest.java | 0 | 6 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 6 list cases) |
| providers/base/MarketingTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/MatzTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/MbtiTest.java | 6 | 0 | ported (hand-written provider): `tests/upstream_generated.rs`; all 6 in `tests/upstream_base_providers.rs` (one test) (6/6) |
| providers/base/MeasurementTest.java | 0 | 8 | ported (generated provider): `tests/upstream_generated.rs`; nested `MeasurementInGreekTest` in `tests/upstream_base_providers.rs` (0/0, 8 list cases) |
| providers/base/MedicalTest.java | 4 | 5 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (4/4, 5 list cases) |
| providers/base/MilitaryTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/base/MoneyTest.java | 4 | 0 | ported: `tests/upstream_generated.rs` (4/4) |
| providers/base/MoodTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/base/MountainTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/base/MountaineeringTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/MusicTest.java | 3 | 1 | ported (hand-written provider): `tests/upstream_generated.rs` (3/3, 1 list cases) |
| providers/base/NationTest.java | 6 | 3 | ported (hand-written provider): `tests/upstream_generated.rs`; `flag` in `tests/upstream_base_providers.rs` (6/6, 3 list cases) |
| providers/base/NatoPhoneticAlphabetTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/NigeriaTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/base/ObjectMethodsTest.java | 3 | 0 | ported: `tests/upstream_object_methods.rs` (3/3; the test's classes as `JavaClass`es with return types, `Person`'s getters of user objects as `OBJECTS`) |
| providers/base/OlympicSportTest.java | 0 | 6 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 6 list cases) |
| providers/base/PassportTest.java | 13 | 0 | ported: `tests/upstream_algorithmic.rs` (13/13) |
| providers/base/PhotographyTest.java | 0 | 9 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 9 list cases) |
| providers/base/PlanetTest.java | 10 | 1 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (10/10, 1 list case; expected values read from the test source) |
| providers/base/ProgrammingLanguageTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/base/PronounsTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/base/ProviderListLocaleTest.java | 1 | 0 | support: abstract base of the `*LocaleTest` classes; its parameterized test runs through them |
| providers/base/ProviderListTest.java | 1 | 0 | support: abstract base of the provider tests; its parameterized test is each class's `providerListTest` row |
| providers/base/RandomFakerTest.java | 4 | 0 | ported: `tests/upstream_algorithmic.rs` (4/4) |
| providers/base/RelationshipTest.java | 3 | 6 | 1/3 ported (hand-written provider): list cases and `anyTest` in `tests/upstream_generated.rs`/`tests/upstream_base_providers.rs`; `anyWithIllegalArgumentExceptionThrown` and `anyWithInvocationTargetExceptionThrown` mock `FakeValuesService` (Mockito), which has no Rust form |
| providers/base/RestaurantTest.java | 2 | 4 | ported (hand-written provider): `tests/upstream_generated.rs`; both tests in `tests/upstream_base_providers.rs` (2/2, 4 list cases) |
| providers/base/RobinTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/RockBandTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/ScienceTest.java | 7 | 8 | ported (generated provider): `tests/upstream_generated.rs` (7/7, 8 list cases) |
| providers/base/ShakespeareTest.java | 4 | 0 | ported (hand-written provider): `tests/upstream_generated.rs` (4/4) |
| providers/base/SipTest.java | 19 | 0 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (19/19) |
| providers/base/SizeTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/SlackEmojiTest.java | 9 | 0 | ported (generated provider): `tests/upstream_generated.rs` (9/9) |
| providers/base/SpaceTest.java | 1 | 12 | ported (hand-written provider): `tests/upstream_generated.rs` (1/1, 12 list cases) |
| providers/base/StockTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/base/SubscriptionTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/base/SuperheroTest.java | 1 | 4 | ported (generated provider): `tests/upstream_generated.rs` (1/1, 4 list cases) |
| providers/base/TeamTest.java | 3 | 3 | ported (generated provider): `tests/upstream_generated.rs`; `testStateWithZaLocale` in `tests/upstream_base_providers.rs` (3/3, 3 list cases) |
| providers/base/TimeAndDateTest.java | 21 | 0 | ported: `tests/upstream_dates.rs` (21/21; parameterized cases as loops) |
| providers/base/TimeTest.java | 10 | 0 | ported: `tests/upstream_dates.rs` (10/10) |
| providers/base/TireTest.java | 2 | 8 | ported (hand-written provider): `tests/upstream_generated.rs`; both `code` tests in `tests/upstream_base_providers.rs` (2/2, 8 list cases) |
| providers/base/TransportTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/TwitterTest.java | 12 | 0 | ported: `tests/upstream_dates.rs` (12/12) |
| providers/base/UniqueTest.java | 10 | 0 | ported: `tests/upstream_algorithmic.rs` (10/10; the Mockito `verify` of `nextInt` calls compared with a generator of the same seed) |
| providers/base/UniversityLocaleTest.java | 0 | 1 | ported: `tests/upstream_base_providers.rs` (uk_UA, uz_UZ, en_NZ) |
| providers/base/UniversityTest.java | 1 | 4 | ported (generated provider): `tests/upstream_generated.rs` (1/1, 4 list cases) |
| providers/base/VehicleLocaleTest.java | 2 | 2 | ported: `tests/upstream_algorithmic.rs` (2/2, 2 list cases over `internationalFakers()`) |
| providers/base/VehicleTest.java | 24 | 0 | ported: `tests/upstream_generated.rs` and `tests/upstream_algorithmic.rs` (24/24) |
| providers/base/VerbTest.java | 5 | 0 | ported (generated provider): `tests/upstream_generated.rs` (5/5) |
| providers/base/WeatherTest.java | 4 | 1 | ported (hand-written provider): `tests/upstream_generated.rs`; the two range tests in `tests/upstream_base_providers.rs` (4/4, 1 list cases) |
| providers/base/WordTest.java | 0 | 7 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 7 list cases) |
| providers/base/YodaTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |
| providers/base/ZodiacTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list cases) |

### T2

T2 (2026-10-07): the 23 generators of `net.datafaker.idnumbers` (`src/idnumbers/`, one module per
country, `Utils` and `LatinLetters` in `mod.rs`) behind the `IdNumber` facade (`src/providers/id_number.rs`), with a
static registry in place of `ServiceLoader`. All 24 rows done, 84/84 tests; `IdNumberTest` (T1 table) 38/38. Adapted:
upstream parses dates back with `LocalDate.parse(…, ofPattern(…))` (not ported); the tests check the same fields.
`FrenchIdNumberTest.controlKey` draws from an unseeded `RandomService` in place of `new Random()`.

| Upstream class | Tests | Cases | Status |
|---|---|---|---|
| idnumbers/AlbanianIdNumberTest.java | 7 | 0 | ported: `tests/upstream_idnumbers.rs` (7/7) |
| idnumbers/BulgarianIdNumberTest.java | 1 | 0 | ported: `tests/upstream_idnumbers.rs` (1/1) |
| idnumbers/ChineseIdNumberTest.java | 3 | 0 | ported: `tests/upstream_idnumbers.rs` (3/3) |
| idnumbers/EstonianIdNumberTest.java | 5 | 0 | ported: `tests/upstream_idnumbers.rs` (5/5) |
| idnumbers/FrenchIdNumberTest.java | 4 | 0 | ported: `tests/upstream_idnumbers.rs` (4/4) |
| idnumbers/HungarianIdNumberTest.java | 2 | 0 | ported: `tests/upstream_idnumbers.rs` (2/2) |
| idnumbers/IrishIdNumberTest.java | 2 | 0 | ported: `tests/upstream_idnumbers.rs` (2/2) |
| idnumbers/ItalianIdNumberTest.java | 4 | 0 | ported: `tests/upstream_idnumbers.rs` (4/4) |
| idnumbers/LatinLettersTest.java | 3 | 0 | ported: `tests/upstream_idnumbers.rs` (3/3) |
| idnumbers/LatvianIdNumberTest.java | 4 | 0 | ported: `tests/upstream_idnumbers.rs` (4/4) |
| idnumbers/MacedonianIdNumberTest.java | 1 | 0 | ported: `tests/upstream_idnumbers.rs` (1/1) |
| idnumbers/MexicanIdNumberTest.java | 4 | 0 | ported: `tests/upstream_idnumbers.rs` (4/4) |
| idnumbers/MoldovanIdNumberTest.java | 2 | 0 | ported: `tests/upstream_idnumbers.rs` (2/2) |
| idnumbers/NorwegianIdNumberTest.java | 4 | 0 | ported: `tests/upstream_idnumbers.rs` (4/4) |
| idnumbers/PolishIdNumberTest.java | 6 | 0 | ported: `tests/upstream_idnumbers.rs` (6/6) |
| idnumbers/PortugueseIdNumberTest.java | 2 | 0 | ported: `tests/upstream_idnumbers.rs` (2/2) |
| idnumbers/RomanianIdNumberTest.java | 8 | 0 | ported: `tests/upstream_idnumbers.rs` (8/8) |
| idnumbers/SingaporeIdNumberTest.java | 4 | 0 | ported: `tests/upstream_idnumbers.rs` (4/4) |
| idnumbers/SouthAfricanIdNumberTest.java | 5 | 0 | ported: `tests/upstream_idnumbers.rs` (5/5) |
| idnumbers/SouthKoreanIdNumberTest.java | 1 | 0 | ported: `tests/upstream_idnumbers.rs` (1/1) |
| idnumbers/SwedishIdNumberTest.java | 6 | 0 | ported: `tests/upstream_idnumbers.rs` (6/6) |
| idnumbers/UkrainianIdNumberTest.java | 2 | 0 | ported: `tests/upstream_idnumbers.rs` (2/2) |
| idnumbers/UtilsTest.java | 3 | 0 | ported: `tests/upstream_idnumbers.rs` (3/3) |
| idnumbers/pt/br/IdNumberGeneratorPtBrUtilTest.java | 1 | 0 | ported: `tests/upstream_algorithmic.rs` (1/1, with `CPFTest`; `CPF`/`CNPJ` reach `src/idnumbers/pt_br.rs` in T1b) |

### T3

23 of 23 tests are ported (2026-10-07), on `src/providers/phone_number.rs` and the `libphonenumber` crate, a port of
libphonenumber 9.0.41 with its own tests and 13,624 differential rows (see that repository's
docs/test-coverage.md). The tests check validity with that crate's `PhoneNumberUtil`, as upstream does with
libphonenumber's. Nothing is adapted; repeated tests loop, parameterized ones run each argument.

| Upstream class | Tests | Cases | Status |
|---|---|---|---|
| providers/base/PhoneNumberTest.java | 18 | 0 | ported: `tests/upstream_phone_number.rs` (18/18) |
| providers/base/PhoneNumberValidityFinderTest.java | 5 | 0 | ported: `tests/upstream_phone_number.rs` (5/5) |

### T4

T4 (2026-10-07): all 132 themed provider classes registered by `EntertainmentProviders` (78),
`FoodProviders` (8), `HealthcareProviders` (5), `SportProviders` (9) and `VideoGameProviders` (32) are ported, 118 of
them generated by `scripts/gen-providers.py` and 14 written by hand: Avatar, Dune (+`Quote`, `Saying`), OscarMovie,
StarWars, Tron (+`Character`, `Quote`, `AlternateCharacterSpelling`), Cheese (+`Wedge`), Coffee (+`Country`), Food,
Tea, Disease (+`DiseaseType`), MedicalProcedure, Dota2 (`heroQuote(String)`), Hearthstone, Minecraft. The nested
enums are `provider_enum!` (`src/providers/mod.rs`). All 133 rows done: 22/22 tests and 564/564 list cases (490 in
`tests/upstream_generated.rs`, 74 in `tests/upstream_themed.rs`, which also reruns `FootballTest`'s 5 in el_GR). The themed fakers upstream (`EntertainmentFaker`,
`FoodFaker`, …) are views of `Faker` with fewer providers; here every test uses `Faker`, as upstream's `Faker`
implements every registry. The generator now also reads chained AssertJ checks (`isNotEmpty().matches(…)`,
`doesNotContain`, `startsWith`, `contains`); before, it kept only the first check of such a chain, so the
`DroneTest` and `HttpTest` rows of T1 now check what upstream checks.

| Upstream class | Tests | Cases | Status |
|---|---|---|---|
| providers/entertainment/AquaTeenHungerForceTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list case) |
| providers/entertainment/AvatarTest.java | 1 | 0 | ported: `tests/upstream_themed.rs` (1/1) |
| providers/entertainment/Babylon5Test.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/BackToTheFutureTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/BigBangTheoryTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/BlueyTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/BoardgameTest.java | 0 | 7 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 7 list cases) |
| providers/entertainment/BojackHorsemanTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/BossaNovaTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/BreakingBadTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/BrooklynNineNineTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/BuffyTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/entertainment/ChuckNorrisTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list case) |
| providers/entertainment/CowboyBebopTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/entertainment/DeathNoteTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/entertainment/DepartedTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/DetectiveConanTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/DoctorWhoTest.java | 0 | 7 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 7 list cases) |
| providers/entertainment/DoraemonTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/DragonBallTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list case) |
| providers/entertainment/DumbAndDumberTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/DuneTest.java | 0 | 26 | ported: `tests/upstream_generated.rs` (3 list cases) and `tests/upstream_themed.rs` (23 `Quote`/`Saying` list cases) |
| providers/entertainment/EntertainmentTest.java | 0 | 3 | ported (generated provider `Show`): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/FamilyGuyTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/FinalSpaceTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/FreshPrinceOfBelAirTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/FriendsTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/FullmetalAlchemistTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/FuturamaTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/entertainment/GameOfThronesTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/entertainment/GhostbustersTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/GratefulDeadTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/GravityFallsTest.java | 0 | 7 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 7 list cases) |
| providers/entertainment/HarryPotterTest.java | 0 | 6 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 6 list cases) |
| providers/entertainment/HeyArnoldTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/HitchhikersGuideToTheGalaxyTest.java | 0 | 7 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 7 list cases) |
| providers/entertainment/HobbitTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/entertainment/HowIMetYourMotherTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/entertainment/HowToTrainYourDragonTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/JokeTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/KaamelottTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/LebowskiTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/LordOfTheRingsTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/MoneyHeistTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/MovieTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/NarutoTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/entertainment/NewGirlTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/OnePieceTest.java | 0 | 6 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 6 list cases) |
| providers/entertainment/OscarMovieTest.java | 5 | 0 | ported: `tests/upstream_generated.rs` (`actor`, `character`) and `tests/upstream_themed.rs` (`movieName`, `quote`) (5/5) |
| providers/entertainment/PokemonTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/entertainment/PrincessBrideTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/ResidentEvilTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/entertainment/RickAndMortyTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/RuPaulDragRaceTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/SeinfeldTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/SeveranceTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list case) |
| providers/entertainment/SiliconValleyTest.java | 0 | 8 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 8 list cases) |
| providers/entertainment/SimpsonsTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/SouthParkTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/SpongebobTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/StarTrekTest.java | 0 | 6 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 6 list cases) |
| providers/entertainment/StarWarsTest.java | 2 | 6 | ported: `tests/upstream_generated.rs` (2/2, 6 list cases) |
| providers/entertainment/StargateTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/StrangerThingsTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/StudioGhibliTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/SuitsTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/entertainment/SupernaturalTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/SwordArtOnlineTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/entertainment/TheExpanseTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/entertainment/TheItCrowdTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/entertainment/TheKingkillerChronicleTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/entertainment/TheRoomTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/entertainment/TheThickOfItTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/TheVentureBrosTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/entertainment/TronTest.java | 0 | 30 | ported: `tests/upstream_generated.rs` (4 list cases) and `tests/upstream_themed.rs` (26 enum list cases) |
| providers/entertainment/TwinPeaksTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/VForVendettaTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/entertainment/WitcherTest.java | 0 | 9 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 9 list cases) |
| providers/foods/AppleTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/foods/BeerTest.java | 0 | 6 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 6 list cases) |
| providers/foods/CheeseTest.java | 1 | 9 | ported: `tests/upstream_generated.rs` (9 list cases) and `tests/upstream_themed.rs` (`shouldReturnCheeseWedge`, 1/1) |
| providers/foods/CoffeeTest.java | 2 | 25 | ported: `tests/upstream_generated.rs` (2/2, 7 list cases) and `tests/upstream_themed.rs` (18 `region(Coffee.Country)` list cases) |
| providers/foods/DessertTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/foods/FoodTest.java | 1 | 7 | ported: `tests/upstream_generated.rs` (1/1, 7 list cases) |
| providers/foods/HebrewFoodTest.java | 1 | 6 | ported: `tests/upstream_generated.rs` (1/1, 6 list cases; an English `FoodFaker` upstream too) |
| providers/foods/IceCreamTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/foods/TeaTest.java | 1 | 1 | ported: `tests/upstream_generated.rs` (1/1, 1 list case) |
| providers/healthcare/CareProviderTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/healthcare/DiseaseTest.java | 2 | 7 | ported: `tests/upstream_themed.rs` (2/2, 7 list cases keyed by `DiseaseType.yamlKey`) |
| providers/healthcare/MedicalProcedureTest.java | 1 | 0 | ported: `tests/upstream_themed.rs` (1/1) |
| providers/healthcare/MedicationTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list case) |
| providers/healthcare/ObservationTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list case) |
| providers/sport/BaseballTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/sport/BasketballTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/sport/ChessTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/sport/CricketTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/sport/EnglandFootBallTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/sport/FootballTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs`; nested `FootballInGreekTest` in `tests/upstream_themed.rs` (0/0, 5 list cases) |
| providers/sport/Formula1Test.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/sport/MartialArtTest.java | 0 | 1 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 1 list case) |
| providers/sport/VolleyballTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/videogame/Battlefield1Test.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/videogame/ClashOfClansTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/videogame/ControlTest.java | 0 | 8 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 8 list cases) |
| providers/videogame/DarkSoulsTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/videogame/Dota2Test.java | 1 | 11 | ported: `tests/upstream_generated.rs` (11 list cases) and `tests/upstream_themed.rs` (`dota2HeroQuote`, 1/1) |
| providers/videogame/EldenRingTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/videogame/ElderScrollsTest.java | 0 | 8 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 8 list cases) |
| providers/videogame/EsportsTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/videogame/FalloutTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/videogame/FinalFantasyXIVTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/videogame/HalfLifeTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/videogame/HearthstoneTest.java | 3 | 3 | ported: `tests/upstream_generated.rs` (3 list cases) and `tests/upstream_themed.rs` (3/3) |
| providers/videogame/HeroesOfTheStormTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/videogame/LeagueOfLegendsTest.java | 0 | 6 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 6 list cases) |
| providers/videogame/MarvelSnapTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/videogame/MassEffectTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/videogame/MinecraftTest.java | 1 | 5 | ported: `tests/upstream_generated.rs` (1/1, 5 list cases) |
| providers/videogame/MystTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/videogame/OverwatchTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/videogame/RedDeadRedemption2Test.java | 0 | 9 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 9 list cases) |
| providers/videogame/SonicTheHedgehogTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/videogame/SoulKnightTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/videogame/StarCraftTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/videogame/StreetFighterTest.java | 0 | 4 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 4 list cases) |
| providers/videogame/SuperMarioTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/videogame/SuperSmashBrosTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/videogame/TormentaTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/videogame/TouhouTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/videogame/VideoGameTest.java | 0 | 3 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 3 list cases) |
| providers/videogame/WarhammerFantasyTest.java | 0 | 5 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 5 list cases) |
| providers/videogame/WorldOfWarcraftTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |
| providers/videogame/ZeldaTest.java | 0 | 2 | ported (generated provider): `tests/upstream_generated.rs` (0/0, 2 list cases) |

### T5

All 128 tests ported (2026-10-07): `src/transformations/` (`Schema`, `Field`, `Value`, the CSV, JSON,
YAML, XML, TOML and SQL transformers, `JavaObjectTransformer`), `src/sequence.rs` (`FakeCollection`,
`FakeStream`) and `src/annotations.rs` (`FakeForSchema`, `FakeResolver`, `Faker::populate`). Adaptations:

- Java's user classes (`JavaObjectTransformer`'s and `@FakeForSchema`'s reflection) are Rust types
  implementing `JavaObject`, which says what reflection finds: the record or constructor the values build, or
  the parameterless constructor and the fields set by name; the annotation is the `FakeForSchema` trait.
- `FakeStreamTest.generateInfiniteStream` checks an unbounded `size_hint` (`estimateSize() == Long.MAX_VALUE`),
  `testCsvFilesCreated` writes into a buffer, `generateCollectionWithDifferentObjects` uses an enum of the two
  types; `SqlTest.testWordTruncated` adds `test.yml` with `addPath`.
- The integration tests walk the dispatch tables (`ProviderKind::ALL`, each provider's methods) where Java uses
  reflection; a `String` method is one that returns a string. `new Locale("pt", "Br", "x2")` has no variant here
  (it reads as `pt_BR` upstream too). `FakerRepeatabilityIntegrationTest` fixes the clock for both reports instead
  of skipping the methods that return an `Instant`.

| Upstream class | Tests | Cases | Status |
|---|---|---|---|
| kotlin: example/SchemaExampleTest.kt | 1 | 0 | ported: `tests/upstream_transformations.rs` (1/1, the data class as a `JavaObject` built by its constructor) |
| annotations/FakeAnnotationTest.java | 10 | 0 | ported: `tests/upstream_transformations.rs` (10/10, the `dto` classes and records as `JavaObject`/`FakeForSchema` types) |
| formats/CsvTest.java | 14 | 0 | ported: `tests/upstream_formats.rs` (14/14) |
| formats/JsonTest.java | 12 | 0 | ported: `tests/upstream_formats.rs` (12/12) |
| formats/SqlTest.java | 24 | 0 | ported: `tests/upstream_formats.rs` (24/24) |
| formats/TomlTest.java | 3 | 0 | ported: `tests/upstream_formats.rs` (3/3) |
| formats/XmlTest.java | 5 | 0 | ported: `tests/upstream_formats.rs` (5/5; nested fake streams shared through `Arc`) |
| formats/YamlTest.java | 3 | 0 | ported: `tests/upstream_formats.rs` (3/3) |
| integration/FakerIntegrationTest.java | 2 | 0 | ported: `tests/upstream_integration.rs` (2/2, over the dispatch tables) |
| integration/FakerRepeatabilityIntegrationTest.java | 2 | 0 | ported: `tests/upstream_integration.rs` (2/2, fixed clock) |
| integration/Issue194SlashFormatRegexTest.java | 3 | 0 | ported: `tests/upstream_integration.rs` (3/3) |
| integration/MostSpecificLocaleTest.java | 1 | 0 | ported: `tests/upstream_integration.rs` (1/1) |
| integration/UkLocalDirectivesTest.java | 1 | 0 | ported: `tests/upstream_integration.rs` (1/1) |
| sequence/FakeCollectionTest.java | 17 | 0 | ported: `tests/upstream_sequence.rs` (17/17) |
| sequence/FakeStreamTest.java | 20 | 0 | ported: `tests/upstream_sequence.rs` (20/20) |
| transformations/JavaObjectTransformerConstructorTest.java | 3 | 0 | ported: `tests/upstream_transformations.rs` (3/3) |
| transformations/JavaObjectTransformerTest.java | 5 | 0 | ported: `tests/upstream_transformations.rs` (5/5) |
| transformations/JsonTransformerTest.java | 2 | 0 | ported: `tests/upstream_transformations.rs` (2/2) |

### T6

| Upstream class | Tests | Cases | Status |
|---|---|---|---|
| script/MkdocsReleaseNavUpdaterTest.java | 3 | 0 | T6 |
| script/ReleaseNotesGeneratorTest.java | 4 | 0 | T6 |

### support

| Upstream class | Tests | Cases | Status |
|---|---|---|---|
| LogFormatter.java | 0 | 0 | support |
| TestSetupExtension.java | 0 | 0 | support |
| ThreadLocalLogHandler.java | 0 | 0 | support |
| annotations/dto/ComplexPersonJavaRecord.java | 0 | 0 | support (in `tests/upstream_transformations.rs`) |
| annotations/dto/Person.java | 0 | 0 | support (in `tests/upstream_transformations.rs`) |
| annotations/dto/PersonJavaRecord.java | 0 | 0 | support (in `tests/upstream_transformations.rs`) |
| assertions/ImageAssert.java | 0 | 0 | support |
| assertions/ImageAssertions.java | 0 | 0 | support |
| assertions/ImageData.java | 0 | 0 | support |
| helpers/IdNumberPatterns.java | 0 | 0 | support |
| providers/base/AbstractProviderListTest.java | 0 | 0 | support |
| providers/base/BaseFakerTest.java | 0 | 0 | support |
| providers/base/LocalityExample.java | 0 | 0 | support |
| providers/entertainment/EntertainmentFakerTest.java | 0 | 0 | support |
| providers/foods/FoodFakerTest.java | 0 | 0 | support |
| providers/healthcare/HealthcareFakerTest.java | 0 | 0 | support |
| providers/sport/SportFakerTest.java | 0 | 0 | support |
| providers/videogame/VideoGameFakerTest.java | 0 | 0 | support |
| script/MkdocsReleaseNavUpdater.java | 0 | 0 | support |
| script/ProviderGenerator.java | 0 | 0 | support |
| script/ProvidersDocsGenerator.java | 0 | 0 | support |
| script/RandomImageGenerator.java | 0 | 0 | support |
| script/ReleaseNotesGenerator.java | 0 | 0 | support |
| sequence/Data.java | 0 | 0 | support (in `tests/upstream_sequence.rs`) |
| service/EvenlyDistributedRandomDoubleGenerator.java | 0 | 0 | support |
| service/EvenlyDistributedRandomLongGenerator.java | 0 | 0 | support |

