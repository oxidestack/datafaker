# datafaker

<!-- languages:begin -->
**English** · [Русский](README.ru.md) · [中文](README.zh-CN.md) · [Español](README.es.md)
<!-- languages:end -->

A port of Datafaker 2.7.0 (Java, `net.datafaker:datafaker`) to Rust: fake names, e-mail addresses, lorem ipsum and
more from Datafaker's own locale files, with the same output as Datafaker for the same `java.util.Random` seed. It is
an independent port, not made, endorsed or supported by the [Datafaker](https://www.datafaker.net/) project, which is
licensed under the Apache License 2.0; [NOTICE](NOTICE) says what this crate takes from it and from other projects.

`upstream/` is the original source, pinned to the ported release and read-only. `resources/` is a verbatim copy of
its locale files (`scripts/sync-resources.sh`), embedded into the crate at build time, so the crate needs neither
the submodule nor any file at run time.

It is built for [mock-server](https://github.com/oxidestack/mock-server), a Rust drop-in for MockServer, which must
behave exactly as MockServer does. MockServer uses Datafaker for OpenAPI example values (`SampleDataGenerator`) and
for the `faker` helper of its response templates.

Status: pre-release. The crate is not published on crates.io yet.

## Example

```rust
let faker = datafaker::Faker::with_seed(9999); // new Faker(new Random(9999))
assert_eq!(faker.name().first_name().unwrap(), "Kermit");
assert_eq!(faker.internet().email_address().unwrap(), "betty.vandervort@hotmail.com");
```

Every public method that can throw in Java returns `Result<_, Error>`, where `Error` carries the Java exception's
simple class name and message.

## What it covers

- **`java.util.Random`**, bit for bit, `SplittableRandom` and the `RandomGenerator` interface, with `RandomService`
  over any of them.
- **The locale data** as SnakeYAML reads it, the locale chain (`en_US` → `en_US`, `_US`, `en`), custom data
  (`addPath`/`addUrl`), and expression resolution (`#{Name.firstName}`, `numerify`, `bothify`, `regexify` on a port of
  RgxGen 3.1, `templatify`, the `csv`/`json` functions) with Java's overload choice.
- **The providers**: the base providers (`Name`, `Internet`, `Address`, `Finance`, …), the date providers on a port
  of the part of `java.time` they use, the 23 national identity number generators, phone numbers through the
  [libphonenumber](https://github.com/oxidestack/libphonenumber) crate, and the 132 themed providers (`StarWars`,
  `Pokemon`, `Beer`, …).
- **Transformations and sequences**: CSV, JSON, YAML, XML, TOML and SQL output of a schema, fake collections and
  streams.
- **Images**: `Image` and `Fingerprint`, byte for byte with the JDK's `ImageIO` writers.
- **Custom providers** reached by expressions, and the provider white list.

Where the port differs from Datafaker, the difference is deliberate and listed: identity hash codes in messages,
iteration orders Java leaves to `HashSet`, the default zone `GMT` and locale `en_US`, the JVM release whose data is
used. [docs/port.md](docs/port.md) says in detail what is ported, every difference and how the tests check the rest
against Datafaker's own output; [docs/test-coverage.md](docs/test-coverage.md) maps every upstream test class.

## Install

Not published yet. The crate depends on the `libphonenumber` crate by path, as a sibling checkout
(`../libphonenumber-rs`), so for now it is built from the two repositories side by side:

```sh
git clone https://github.com/oxidestack/datafaker
git clone https://github.com/oxidestack/libphonenumber libphonenumber-rs
```

and depended on by path: `datafaker = { path = "../datafaker" }`. The crate needs Rust 1.85 or later.

## Documentation

- The guides, in English, Russian, Chinese and Spanish: <https://oxidestack.github.io/datafaker/>, with
  [upgrading](docs/guide/upgrading.md) between releases.
- The API documentation: on docs.rs once the crate is published; until then, `cargo doc --open`.
- What changed in each release: [CHANGELOG.md](CHANGELOG.md).

## Build and test

```sh
git submodule update --init upstream   # the ported Datafaker source, read-only reference
git clone https://github.com/oxidestack/libphonenumber ../libphonenumber-rs   # the sibling crate the phone numbers use
cargo test
make lint        # clippy, warnings are errors
make deny        # licences, advisories, sources (cargo install cargo-deny --locked)
make public-check
make hooks       # run the publication gate before every push
make site        # the documentation site
```

`tests/vectors/` holds outputs recorded from the real Datafaker; `scripts/record.sh` regenerates them (a JDK is needed).

## Getting help

A question about how the crate works can go to [DeepWiki](https://deepwiki.com/oxidestack/datafaker) first: it
answers from this repository's code and documentation, and where it disagrees with the documentation, the
documentation is right. Questions it does not settle go to
[Discussions](https://github.com/oxidestack/datafaker/discussions), bugs to
[issues](https://github.com/oxidestack/datafaker/issues).

## Contributing, security, licence

[CONTRIBUTING.md](CONTRIBUTING.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [GOVERNANCE.md](GOVERNANCE.md),
[SECURITY.md](SECURITY.md). Code: Apache-2.0 ([LICENSE](LICENSE)). Documentation: CC-BY-4.0
([LICENSE-docs](LICENSE-docs)).
