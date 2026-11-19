# The port: scope, differences and tests

What of Datafaker 2.7.0 this crate ports, where it differs from the Java library, and how the tests hold it to
Datafaker's own output. The [README](../README.md) is the short version.

## Scope

Tranches T0 (the core), T1a (the base providers that read YAML lists), most of T1b (the algorithmic base
providers), T1c (the date providers), T2 (national identity numbers), T3 (phone numbers, on the `libphonenumber`
crate), T4 (the themed providers) and T5 (the transformers and fake sequences) are done, and so are `Image` and
`Fingerprint` (see `docs/test-coverage.md`).

- **`java.util.Random`**, bit for bit (`java::Random`), including the `RandomGenerator` defaults it inherits on
  JDK 17+ (`nextInt(origin, bound)` and `nextLong(origin, bound)` draw through `RandomSupport`'s bounded
  algorithms) and `nextGaussian` with fdlibm's `log`; `java.util.SplittableRandom` (`java::SplittableRandom`,
  `split` included) and the `RandomGenerator` interface (`java::RandomGenerator`, without `nextGaussian` and the
  streams).
- **`RandomService`** over any `RandomGenerator` (`RandomService::new(SplittableRandom::new(seed))`) and `Range`;
  subclasses of `java.util.Random` (`Random::with_overrides`, as in `new Faker(new MyRandom())`), and
  `WeightedRandomSelector`.
- **Locale data.** SnakeYAML's reading of the files (YAML 1.1 scalar types, aliases, insertion-ordered maps),
  `FakeValues` per locale, the English grouping by `EnFile` path, the load-time rewriting of
  `enrichMapWithJavaNames` (`#{first_name}` → `#{Name.firstName}`, `firstName` aliases), and `FakerContext`'s
  locale chain (`en_US` → `en_US`, `_US`, `en`).
- **`FakeValuesService`**: `fetch`, `fetchString`, `fetchObject`, `safeFetch` (with slash-delimited regexes),
  `numerify`, `letterify`, `bothify`, `examplify`, `templatify`, `regexify`, and expression resolution: `#{…}`
  splitting and quoted arguments, upstream's search order (methods on the current provider, YAML keys, methods on
  the faker, `Provider.method`, the YAML spelling of the reference), its per-expression resolver cache, and
  reflection replaced by method tables that keep Java's overload choice (`rightIsMostRestrictive`) and argument
  coercion.
- **Custom data**: `FakeValuesContext`, `FakeValues` and `FakeValuesGrouping` as public types, each faker's own
  map of locale data, and `addPath`/`addUrl` (`file:` URLs), grouped with the locale's data as upstream groups them.
- **Expression functions** `csv`, `json` and `jsona` (`JsonTransformer`/`CsvTransformer` output for string fields),
  `fileExpression`, and `doWith` with a locale, a seed or both.
- **Providers**: `Name`, `Internet`, `Lorem`, `Credentials`, `Text`, `Number` (its `BigDecimal` methods on exact
  decimals, `java::BigDecimal`), `Options`. `Internet` brings `java.net.IDN.toASCII` with Punycode and IDNA2003
  nameprep on the JDK's Unicode 3.2 tables, `stripAccents`, `Inet4Address`/`Inet6Address` formatting and
  `UUID.nameUUIDFromBytes` (MD5).
- **YAML-list providers** (T1a): 84 base providers whose methods read the locale files, such as `Ancient`,
  `Animal`, `App`, `BloodType`, `Book`, `Dog`, `Job`, `Military`, `Mood`, `Space`, `Weather`. The 68 whose methods
  only resolve a key are generated from Datafaker's sources by `scripts/gen-providers.py` (`src/providers/
  generated.rs`, which also holds the registry of every ported provider: `ProviderKind`, the `Faker` accessors and
  the faker's method table); the 16 with a few lines of logic are written by hand, one module each.
- **Algorithmic base providers** (T1b): `Address`, `Aviation`, `Aws`, `Azure`, `Barcode`, `Bool`, `CNPJ`, `CPF`,
  `Code`, `Color`, `Commerce`, `Company`, `Compass`, `Country`, `Domain`, `DrivingLicense`, `Drone`, `FakeDuration`
  (`java::Duration`), `File`, `Finance`, `Hashing`, `Http`, `LanguageCode`, `Locality`, `Medical`, `Money`,
  `Passport`, `Planet`, `Sip`, `Unique`, `Vehicle`: their check digits (ISBN, EAN/GTIN, IMEI, Luhn, IBAN mod 97,
  ABA routing, CPF/CNPJ mod 11), MD2 and SHA digests, `DecimalFormat`/`String.format` number output in the faker's
  locale, and the state upstream keeps on a faker's cached provider (`Compass.compassPoint`, `Locality`'s shuffled
  locales, `Unique`'s remaining values).
- **Date providers** (T1c): `TimeAndDate`, `DateAndTime` (`faker.date()`), `Time`, `Business` and `Twitter`, on
  the part of `java.time` they use (`java::time`: `Instant`, `LocalDate`, `LocalTime`, `LocalDateTime`, `Period`,
  `ChronoUnit`, `TimeUnit`, `java.util.Date`/`java.sql.Timestamp` as `java::Date`), with `toString()`s as the JDK
  prints them, `DateTimeFormatter.ofPattern` and `SimpleDateFormat` formatting (every pattern letter, `en_US`
  texts, the Julian calendar before 1582 for `Date`), `DateTimeFormatter` parsing (`parse`, strict, with adjacent
  value parsing and the SMART resolver; `Parsed`, `LocalDate::from_parsed` and the like) and `java.util.Date`'s
  lenient `Date.parse` (`Date::parse_text`), which expressions use to reach `DateAndTime`'s `Date` overloads
  (`#{date.between 'Sat, 12 Aug 1995 13:30:00 GMT','…'}`). Methods that read the clock have
  `…_at`/`…_on`/`…_from` variants that take the reference date.
- **National identity numbers** (T2): the 23 generators of `net.datafaker.idnumbers` (`idnumbers`, plus the
  unregistered `IrishIdNumber`) behind `faker.id_number()`, found through a static registry in the order of
  upstream's `META-INF/services` (no `ServiceLoader`). `java::time::with_fixed_clock` fixes the clock of the
  current thread (as `Clock.fixed`), for code that must reproduce birthdays drawn relative to today.
- **Themed providers** (T4): the 132 providers of `EntertainmentProviders`, `FoodProviders`,
  `HealthcareProviders`, `SportProviders` and `VideoGameProviders` (`Dune`, `StarWars`, `Pokemon`, `Beer`,
  `Coffee`, `Disease`, `Football`, `Minecraft`, …), all reached from `Faker` as upstream's `Faker` reaches them
  (the narrower `EntertainmentFaker`, `FoodFaker`, … are not separate types). 118 are generated; the 14 with logic
  or nested enums (`Dune.Quote`, `Tron.Character`, `Coffee.Country`, `Disease.DiseaseType`, the `Cheese.Wedge`
  record) are written by hand, the enums through `provider_enum!` with `_of` overloads (`dune().quote_of(…)`).
- **Phone numbers** (T3): `PhoneNumber` (`cellPhone`, `phoneNumber`, their national and international forms,
  `extension`, `subscriberNumber`) and `PhoneNumberGenerator`, which draws a region's national-number pattern with
  `regexify` until libphonenumber accepts the number. libphonenumber is the `libphonenumber` crate
  (`../libphonenumber-rs`, a path dependency on a sibling checkout), a port of libphonenumber 9.0.41, the version MockServer 9.2.0 runs
  Datafaker with (Datafaker itself names 9.0.33).
- **Transformations and sequences** (T5): `transformations::Schema` of `Field`s (`field`, `field_of`,
  `composite_field`) rendered by `CsvTransformer`, `JsonTransformer`, `YamlTransformer`, `XmlTransformer`,
  `TomlTransformer` and `SqlTransformer` (its 23 `SqlDialect`s, batches, keyword cases, arrays, maps, rows), each
  with `generate`, `generate_from` (a list or a fake sequence), `generate_stream` and `write_to_output_stream`;
  `sequence::FakeCollection`/`FakeStream` (`faker.collection()`, `faker.stream()`, null rates, infinite streams).
  A Java field function returns `Object`; here it returns anything that becomes a `transformations::Value`, one
  variant per runtime type the transformers tell apart (boxed numbers, `Character`, object and primitive arrays,
  collections, maps, nested schemas, other objects with their `toString()`). Reflection over user classes is
  adapted (Rust has no reflection over a class, so the type says how it is built): `JavaObjectTransformer` builds types implementing `JavaObject` (a record
  or constructor path, or a parameterless constructor and fields set by name), and `@FakeForSchema` is the
  `annotations::FakeForSchema` trait behind `Faker::populate`.
- **`regexify`**: RgxGen 3.1's parser and generator (`rgxgen`), see the module documentation for what it covers.
- **Custom providers and white lists**: user classes reached by expressions (`#{Insect.nextInsectName}`, a bare
  `#{hello}` on the current object, `getProvider("Insect")`) through `dispatch::JavaClass`, `Faker::get_provider`,
  `Faker::declare_provider`, `FakeValuesService::resolve_on`, and the provider white list of
  `new Faker(fakeValuesService, context, whiteListPredicate)` (`Faker::with_white_list`).
- **Images.** `Image` and `Fingerprint` write their images through ports of the JDK's `javax.imageio` writers as
  `ImageIO.write` runs them with default parameters (`java::imageio`): BMP, GIF (with `PaletteBuilder`'s octree and
  the writer's LZW), JPEG (IJG libjpeg 6b as the JDK bundles it), PNG (over `java::deflater`, zlib 1.3.1's deflate
  as the JDK bundles it) and TIFF, byte for byte with JDK 25.0.2 (`tests/vectors/images.tsv`).

Every public method that can throw in Java returns `Result<_, Error>`, where `Error` carries the Java exception's
simple class name and message.

## Differences

- **IDN.** The nameprep tables are read back from the JDK (`scripts/idn-tables.sh`), one code point at a time;
  only the composition step of NFKC runs on current Unicode data, which composes Unicode 3.2's code points as
  Unicode 3.2 did. `tests/vectors/idn.tsv` holds 8,298 JDK results, all equal.
- **Custom data (`addPath`/`addUrl`).** Upstream keeps the `FakeValues` of a path in a `HashSet` (identity-hash
  order) and, when it groups custom data with the English files, adds to the very sets of the shared English
  grouping, so a custom English file of one faker reaches every faker. Here the files of a path merge in the order
  they were added, and custom data stays with the faker it was added to. Only `file:` URLs (and plain paths) are
  read.
- **Grouped English files.** Upstream merges the files of one `EnFile` path in `HashSet` order, which changes
  between JVM runs; here they merge in `EnFile` order. No bundled key is defined twice with different values, so the
  result is the same (a unit test checks it).
- **Aliases in YAML** are copies here, where SnakeYAML shares one object; the load-time rewriting of an aliased
  list therefore applies per occurrence. No bundled file aliases a list across providers.
- **Identity hash codes** in exception messages (`java.util.Random@1b2c3d`, `[Ljava.lang.String;@5e2de80c`) are
  stand-ins; so is the `toString()` of an array or a `FakeCollection` in transformer output (`[I@0`). A
  `Value` keeps no Java class for collections, maps and other objects, so a `ClassCastException` names them
  `java.util.ArrayList`, `java.util.LinkedHashMap` and `java.lang.Object`; a `BigDecimal` value prints the text
  it was given (`java::BigDecimal` converts with `toPlainString()`).
- **Transformers' `Field`s are invariant in their lifetime**: a fake sequence used inside another one's
  fields is shared through an `Arc` (see `tests/upstream_formats.rs`).
- **Custom providers** implement `dispatch::JavaClass` (binary name, method table) where upstream reflects on
  the class; a faker "subclass" is a faker with declared accessors (`Faker::declare_provider`). A user method
  returns a `JValue`, so it cannot hand another user object to an expression.
- **White lists** (`Faker::with_white_list`) are checked by expressions and method tables each time they reach a
  provider, and by `Faker::get_provider`; the Rust accessors (`faker.name()`) are views that cannot fail and are
  not checked. Upstream checks once, when the faker first creates the provider.
- **`Fingerprint`'s ridges** use the platform's `sin`, `exp` and `atan2` where upstream calls `Math`'s (HotSpot's
  intrinsics on some CPUs): a last-bit difference changes a pixel only when a value falls exactly on a ridge or
  rounding boundary. `Fingerprint.png()` through an expression returns the bytes as a list of `byte` values
  (upstream's `byte[]` prints as an identity hash).
- **`Options.subset` for non-strings** returns elements in draw order; upstream returns a `HashSet`.
- **`Credentials.userId(regex)`** checks the pattern with RgxGen's parser, not `java.util.regex.Pattern`.
- **`Nation.isoLanguage`/`isoCountry`** draw from the ISO code lists of JDK 25.0.2 (`src/providers/iso_codes.rs`);
  a JVM whose lists differ draws other codes for the same seed.

- **`Money`** draws from the JDK's currencies in code order (`src/providers/jdk_tables.rs`); upstream copies a
  `HashSet` of `Currency` objects, whose order follows identity hash codes and changes between JVM runs, so no
  order reproduces it. Names and symbols are those of the default locale `en_US`.
- **JDK locale data** (`scripts/jdk-tables.sh`, JDK 25.0.2): `Locality.displayName`'s names and each locale's
  digits, decimal separator and negative prefix for `Commerce.price` and `Address.latitude`/`longitude`.
- **Default zone and locale.** `ZoneId.systemDefault()` and `TimeZone.getDefault()` are `GMT` (not `UTC`: the
  ids, names and `Date.toString()` differ), as in MockServer 8.0.0's image, unless `java::time::set_default_zone`
  sets another zone (`ZoneId::of("Europe/Berlin")`, or `ZoneId::host()`: `TZ`, then `/etc/localtime`, as the JDK
  reads them; `with_default_zone` sets one for the current thread). Zones come from tzdb 2026b with its en_US names,
  as MockServer 8.0.0's image (Temurin 26.0.2) has them (`src/java/time/tzdb.bin`,
  `scripts/tz-tables.sh mockserver/mockserver:8.0.0`), and `java.util.Date`, `Timestamp` and `SimpleDateFormat` follow `java.util.TimeZone`'s reading of them (no transitions
  before 1900; a wall time in an overlap takes the later offset). The FORMAT locale is `en_US`.
  `scripts/record.sh` runs the JVM with `-Duser.language=en -Duser.country=US` and `-Duser.timezone=GMT` (or the
  zone a file's `# zone` line names).
- **Date parsing.** `SimpleDateFormat` formats only (Datafaker never parses with it). `Parsed` keeps its unresolved
  fields in the order they were parsed and prints them sorted by name; upstream's `HashMap` orders them by identity
  hash codes, which change between JVM runs, so when several fields conflict the one an error names can differ.
  `Instant`, `LocalTime`, `Period` and `String[]` have no `(String)` constructor, so expressions cannot reach the
  overloads that take them (upstream neither).
- **Which JVM.** The reference is Datafaker 2.7.0 on JDK 25.0.2, except where MockServer 8.0.0, this crate's first
  user, runs it differently: its image runs Temurin 26.0.2, whose tzdb is 2026b (zone rules and names; British
  Columbia stays on -07:00 after 2026-03-08), where `Instant.parse` takes `+HH`, `+HHmm` and
  `+HHmmss` offsets, and where `getMethods()` orders `DateAndTime.future(int, TimeUnit, String)` before
  `future(int, TimeUnit, Date)` and `Aviation.flight(AviationCodeType)` before `flight(String)`. When an
  expression's arguments coerce to both overloads, Datafaker takes the first, so
  `#{date.future '10','DAYS','12 Aug 1995'}` reaches the pattern overload and fails, as on MockServer's oracle (JDK 25
  reaches the `Date` one). The other tied overloads (`past`, `duration`, `Name.malefirstName`) are ordered the same
  on both JVMs, and the image gave all of them the same order on each of six runs. One difference of that image is
  not followed: its trimmed JDK has no `jdk.localedata` (non-English locales format numbers as `en` does there).
- **Clock resolution.** `Instant.now()` reads the system clock at microsecond resolution, as the JDK does on
  macOS and Linux.
- **`Twitter.twitterId`** hashes a random UUID from the operating system (`/dev/urandom`), as upstream's
  `UUID.randomUUID()` reads `SecureRandom`; neither is reproducible.
- **`File.fileName`** uses `/` as the separator (`FileSystems.getDefault().getSeparator()` on Unix); `Aws`
  lower-cases app names as the JVM's default locale `en_US` does.

## Tests

`cargo test`:

- **Differential vectors.** `tests/vectors/*.tsv` were recorded from Datafaker 2.7.0 itself: `scripts/record.sh`
  compiles `scripts/Vectors.java` against `datafaker-2.7.0.jar` in `~/.m2` and runs each row's calls on
  `new Faker(new Random(seed))`. `tests/vectors.rs` replays them on the Rust API, exceptions included.
  `tests/vectors/transformations.tsv` holds named transformer and sequence cases built by
  `scripts/Transformations.java` (`scripts/record.sh tests/vectors/transformations.tsv Transformations`) and again
  by `tests/vectors_transformations.rs`. The classpath has libphonenumber 9.0.41 (MockServer 9.2.0's); the rows of
  `tests/vectors/phone_numbers.tsv` are written by `scripts/phone-vectors.py`. A file whose
  calls read the clock carries the date it was recorded on (`# today`) and is replayed on that date.
- **MockServer anchors.** `tests/anchors.rs` holds values MockServer 8.0.0's oracle produced.
- **Upstream's unit tests**, ported in `tests/upstream_*.rs`; `docs/test-coverage.md` maps every upstream test
  class to its port or tranche. `tests/upstream_generated.rs` is generated with the providers: the provider-list
  cases and regex-only tests of their test classes.

`scripts/gen-providers.py` regenerates the providers, their tests and the rows of `tests/vectors/providers.tsv`
(keeping recorded results); `--check` fails when a generated file is out of date, `--report` lists what it left
to hand-written code. New vector rows get their results from `scripts/record.sh tests/vectors/providers.tsv`.
- **Resources.** `tests/resources.rs` checks that `resources/` (and `tests/resources/`, upstream's test YAML
  files) equal the upstream files and that all of them load.

