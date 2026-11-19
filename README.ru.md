# datafaker

<!-- languages:begin -->
[English](README.md) · **Русский** · [中文](README.zh-CN.md) · [Español](README.es.md)
<!-- languages:end -->
<!-- translation of README.md: 67b90ac5e502 -->

Порт на Rust библиотеки Datafaker 2.7.0 (Java, `net.datafaker:datafaker`): фальшивые имена, адреса электронной почты,
lorem ipsum и многое другое из собственных файлов локалей Datafaker, с тем же результатом, что у Datafaker, при том же
зерне `java.util.Random`. Это независимый порт, не созданный, не одобренный и не поддерживаемый проектом
[Datafaker](https://www.datafaker.net/), который распространяется по лицензии Apache License 2.0; в [NOTICE](NOTICE)
сказано, что этот крейт берёт из него и из других проектов.

`upstream/` — исходный код оригинала, закреплённый на портированном выпуске и доступный только для чтения.
`resources/` — точная копия его файлов локалей (`scripts/sync-resources.sh`), встраиваемая в крейт при сборке, так что
во время выполнения крейту не нужны ни подмодуль, ни какие-либо файлы.

Он создан для [mock-server](https://github.com/oxidestack/mock-server) — замены MockServer на Rust, которая должна вести
себя в точности как MockServer. MockServer использует Datafaker для значений-примеров OpenAPI (`SampleDataGenerator`)
и для помощника `faker` в шаблонах ответов.

Статус: до первого выпуска. Крейт ещё не опубликован на crates.io.

## Пример

```rust
let faker = datafaker::Faker::with_seed(9999); // new Faker(new Random(9999))
assert_eq!(faker.name().first_name().unwrap(), "Kermit");
assert_eq!(faker.internet().email_address().unwrap(), "betty.vandervort@hotmail.com");
```

Каждый публичный метод, который в Java может бросить исключение, возвращает `Result<_, Error>`, где `Error` содержит
простое имя класса исключения Java и его сообщение.

## Что входит

- **`java.util.Random`** бит в бит, `SplittableRandom` и интерфейс `RandomGenerator`, а над любым из них —
  `RandomService`.
- **Данные локалей** в том виде, в каком их читает SnakeYAML, цепочка локалей (`en_US` → `en_US`, `_US`, `en`),
  пользовательские данные (`addPath`/`addUrl`) и разрешение выражений (`#{Name.firstName}`, `numerify`, `bothify`,
  `regexify` на порте RgxGen 3.1, `templatify`, функции `csv`/`json`) с выбором перегрузки, как в Java.
- **Провайдеры**: базовые (`Name`, `Internet`, `Address`, `Finance`, …), провайдеры дат на порте той части `java.time`,
  которой они пользуются, 23 генератора национальных идентификационных номеров, телефонные номера через крейт
  [libphonenumber](https://github.com/oxidestack/libphonenumber) и 132 тематических провайдера (`StarWars`,
  `Pokemon`, `Beer`, …).
- **Преобразования и последовательности**: вывод схемы в CSV, JSON, YAML, XML, TOML и SQL, фальшивые коллекции и
  потоки.
- **Изображения**: `Image` и `Fingerprint`, байт в байт с writer-ами `ImageIO` из JDK.
- **Пользовательские провайдеры**, доступные из выражений, и белый список провайдеров.

Там, где порт отличается от Datafaker, отличие намеренное и перечислено: хеш-коды идентичности в сообщениях, порядок
обхода, который Java оставляет `HashSet`, зона по умолчанию `GMT` и локаль `en_US`, выпуск JVM, чьи данные
используются. В [docs/port.md](docs/port.md) подробно описано, что портировано, каждое отличие и как тесты сверяют
остальное с собственным выводом Datafaker; [docs/test-coverage.md](docs/test-coverage.md) сопоставляет каждый тестовый
класс оригинала.

## Установка

Ещё не опубликован. Крейт зависит от крейта `libphonenumber` по пути, как от соседнего клона (`../libphonenumber-rs`),
поэтому пока он собирается из двух репозиториев, лежащих рядом:

```sh
git clone https://github.com/oxidestack/datafaker
git clone https://github.com/oxidestack/libphonenumber libphonenumber-rs
```

и подключается по пути: `datafaker = { path = "../datafaker" }`. Крейту нужен Rust 1.85 или новее.

## Документация

- Руководства на английском, русском, китайском и испанском: <https://oxidestack.github.io/datafaker/>, включая
  [обновление](docs/i18n/ru/guide/upgrading.md) между выпусками.
- Документация API: на docs.rs, когда крейт будет опубликован; до тех пор — `cargo doc --open`.
- Что изменилось в каждом выпуске: [CHANGELOG.md](CHANGELOG.md).

## Сборка и тесты

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

В `tests/vectors/` лежат результаты, записанные с настоящего Datafaker; `scripts/record.sh` пересоздаёт их (нужен JDK).

## Помощь

Вопрос о том, как работает крейт, можно сначала задать [DeepWiki](https://deepwiki.com/oxidestack/datafaker): он
отвечает по коду и документации этого репозитория, а где он расходится с документацией, права документация. Вопросы,
которые он не решил, — в [Discussions](https://github.com/oxidestack/datafaker/discussions), ошибки — в
[issues](https://github.com/oxidestack/datafaker/issues).

## Участие, безопасность, лицензия

[CONTRIBUTING.md](CONTRIBUTING.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [GOVERNANCE.md](GOVERNANCE.md),
[SECURITY.md](SECURITY.md). Код: Apache-2.0 ([LICENSE](LICENSE)). Документация: CC-BY-4.0
([LICENSE-docs](LICENSE-docs)).
