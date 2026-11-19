# datafaker

<!-- languages:begin -->
[English](README.md) · [Русский](README.ru.md) · [中文](README.zh-CN.md) · **Español**
<!-- languages:end -->
<!-- translation of README.md: 67b90ac5e502 -->

Un port a Rust de Datafaker 2.7.0 (Java, `net.datafaker:datafaker`): nombres, direcciones de correo, lorem ipsum y
mucho más, falsos, a partir de los propios archivos de locale de Datafaker, con la misma salida que Datafaker para la
misma semilla de `java.util.Random`. Es un port independiente, no hecho, respaldado ni mantenido por el proyecto
[Datafaker](https://www.datafaker.net/), que se distribuye bajo la Apache License 2.0; [NOTICE](NOTICE) dice qué toma
este crate de él y de otros proyectos.

`upstream/` es el código fuente original, fijado en la versión portada y de solo lectura. `resources/` es una copia
exacta de sus archivos de locale (`scripts/sync-resources.sh`), incrustada en el crate al compilar, de modo que en
ejecución el crate no necesita ni el submódulo ni ningún archivo.

Está hecho para [mock-server](https://github.com/oxidestack/mock-server), un sustituto directo de MockServer en Rust
que debe comportarse exactamente como MockServer. MockServer usa Datafaker para los valores de ejemplo de OpenAPI
(`SampleDataGenerator`) y para el helper `faker` de sus plantillas de respuesta.

Estado: previo a la primera versión. El crate todavía no está publicado en crates.io.

## Ejemplo

```rust
let faker = datafaker::Faker::with_seed(9999); // new Faker(new Random(9999))
assert_eq!(faker.name().first_name().unwrap(), "Kermit");
assert_eq!(faker.internet().email_address().unwrap(), "betty.vandervort@hotmail.com");
```

Todo método público que en Java puede lanzar una excepción devuelve `Result<_, Error>`, donde `Error` lleva el nombre
simple de la clase de la excepción de Java y su mensaje.

## Qué cubre

- **`java.util.Random`**, bit a bit, `SplittableRandom` y la interfaz `RandomGenerator`, con `RandomService` sobre
  cualquiera de ellos.
- **Los datos de locale** tal como los lee SnakeYAML, la cadena de locales (`en_US` → `en_US`, `_US`, `en`), los datos
  personalizados (`addPath`/`addUrl`) y la resolución de expresiones (`#{Name.firstName}`, `numerify`, `bothify`,
  `regexify` sobre un port de RgxGen 3.1, `templatify`, las funciones `csv`/`json`) con la elección de sobrecarga de
  Java.
- **Los proveedores**: los básicos (`Name`, `Internet`, `Address`, `Finance`, …), los de fechas sobre un port de la
  parte de `java.time` que usan, los 23 generadores de números de identidad nacionales, los números de teléfono a través
  del crate [libphonenumber](https://github.com/oxidestack/libphonenumber) y los 132 proveedores temáticos
  (`StarWars`, `Pokemon`, `Beer`, …).
- **Transformaciones y secuencias**: salida de un esquema en CSV, JSON, YAML, XML, TOML y SQL, colecciones y flujos
  falsos.
- **Imágenes**: `Image` y `Fingerprint`, byte a byte con los escritores `ImageIO` del JDK.
- **Proveedores personalizados** alcanzables desde expresiones, y la lista blanca de proveedores.

Donde el port difiere de Datafaker, la diferencia es deliberada y está documentada: los códigos hash de identidad en
los mensajes, los órdenes de iteración que Java deja a `HashSet`, la zona por defecto `GMT` y la locale `en_US`, la
versión de la JVM cuyos datos se usan. [docs/port.md](docs/port.md) explica en detalle qué se porta, cada diferencia y
cómo las pruebas comparan el resto con la propia salida de Datafaker; [docs/test-coverage.md](docs/test-coverage.md)
relaciona cada clase de prueba del original.

## Instalación

Todavía no publicado. El crate depende del crate `libphonenumber` por ruta, como un clon vecino
(`../libphonenumber-rs`), así que por ahora se compila con los dos repositorios uno al lado del otro:

```sh
git clone https://github.com/oxidestack/datafaker
git clone https://github.com/oxidestack/libphonenumber libphonenumber-rs
```

y se usa como dependencia por ruta: `datafaker = { path = "../datafaker" }`. El crate necesita Rust 1.85 o posterior.

## Documentación

- Las guías, en inglés, ruso, chino y español: <https://oxidestack.github.io/datafaker/>, con la
  [actualización](docs/i18n/es/guide/upgrading.md) entre versiones.
- La documentación de la API: en docs.rs cuando el crate se publique; hasta entonces, `cargo doc --open`.
- Qué cambió en cada versión: [CHANGELOG.md](CHANGELOG.md).

## Compilar y probar

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

`tests/vectors/` contiene salidas grabadas del Datafaker real; `scripts/record.sh` las regenera (hace falta un JDK).

## Ayuda

Una pregunta sobre cómo funciona el crate puede ir primero a [DeepWiki](https://deepwiki.com/oxidestack/datafaker):
responde a partir del código y la documentación de este repositorio, y donde no coincide con la documentación, la
documentación tiene razón. Las preguntas que no resuelva van a
[Discussions](https://github.com/oxidestack/datafaker/discussions), y los errores a
[issues](https://github.com/oxidestack/datafaker/issues).

## Contribuir, seguridad, licencia

[CONTRIBUTING.md](CONTRIBUTING.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [GOVERNANCE.md](GOVERNANCE.md),
[SECURITY.md](SECURITY.md). Código: Apache-2.0 ([LICENSE](LICENSE)). Documentación: CC-BY-4.0
([LICENSE-docs](LICENSE-docs)).
