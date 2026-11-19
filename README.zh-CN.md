# datafaker

<!-- languages:begin -->
[English](README.md) · [Русский](README.ru.md) · **中文** · [Español](README.es.md)
<!-- languages:end -->
<!-- translation of README.md: 67b90ac5e502 -->

Datafaker 2.7.0（Java，`net.datafaker:datafaker`）的 Rust 移植：根据 Datafaker 自己的 locale 文件生成假的姓名、电子邮件地址、
lorem ipsum 等数据，对于相同的 `java.util.Random` 种子，输出与 Datafaker 相同。这是一个独立的移植，并非由
[Datafaker](https://www.datafaker.net/) 项目制作、认可或支持；Datafaker 采用 Apache License 2.0 许可，[NOTICE](NOTICE)
说明了本 crate 从它以及其他项目中采用了哪些内容。

`upstream/` 是原始源代码，固定在所移植的版本，只读。`resources/` 是其 locale 文件的原样副本（`scripts/sync-resources.sh`），
在构建时嵌入 crate，因此运行时既不需要子模块，也不需要任何文件。

它是为 [mock-server](https://github.com/oxidestack/mock-server) 构建的。mock-server 是 MockServer 的 Rust 直接替代品，
其行为必须与 MockServer 完全一致。MockServer 使用 Datafaker 生成 OpenAPI 示例值（`SampleDataGenerator`），并作为响应模板中的
`faker` 助手。

状态：尚未发布。该 crate 还没有在 crates.io 上发布。

## 示例

```rust
let faker = datafaker::Faker::with_seed(9999); // new Faker(new Random(9999))
assert_eq!(faker.name().first_name().unwrap(), "Kermit");
assert_eq!(faker.internet().email_address().unwrap(), "betty.vandervort@hotmail.com");
```

每个在 Java 中可能抛出异常的公共方法都返回 `Result<_, Error>`，其中 `Error` 带有 Java 异常的简单类名和消息。

## 涵盖范围

- **`java.util.Random`**（逐位一致）、`SplittableRandom` 和 `RandomGenerator` 接口，以及基于其中任意一个的 `RandomService`。
- **locale 数据**（按 SnakeYAML 的读取方式）、locale 链（`en_US` → `en_US`、`_US`、`en`）、自定义数据（`addPath`/`addUrl`），
  以及表达式解析（`#{Name.firstName}`、`numerify`、`bothify`、基于 RgxGen 3.1 移植的 `regexify`、`templatify`、
  `csv`/`json` 函数），并保持 Java 的重载选择。
- **provider**：基础 provider（`Name`、`Internet`、`Address`、`Finance` 等）、基于它们所用的 `java.time` 部分的移植实现的日期
  provider、23 个国民身份号码生成器、通过 [libphonenumber](https://github.com/oxidestack/libphonenumber) crate 生成的电话号码，
  以及 132 个主题 provider（`StarWars`、`Pokemon`、`Beer` 等）。
- **转换与序列**：将 schema 输出为 CSV、JSON、YAML、XML、TOML 和 SQL，以及假数据集合和流。
- **图像**：`Image` 和 `Fingerprint`，与 JDK 的 `ImageIO` 写入器逐字节一致。
- 表达式可以调用的**自定义 provider**，以及 provider 白名单。

移植版与 Datafaker 不同之处都是有意为之并已列出：消息中的身份哈希码、Java 交给 `HashSet` 决定的迭代顺序、默认时区 `GMT`
和默认 locale `en_US`、所采用数据的 JVM 版本。[docs/port.md](docs/port.md) 详细说明了移植了什么、每一处差异，以及测试如何将其余部分与
Datafaker 自身的输出进行比对；[docs/test-coverage.md](docs/test-coverage.md) 列出了每个上游测试类的对应关系。

## 安装

尚未发布。本 crate 通过路径依赖 `libphonenumber` crate，即相邻的检出目录（`../libphonenumber-rs`），因此目前需要把两个仓库并排放置来构建：

```sh
git clone https://github.com/oxidestack/datafaker
git clone https://github.com/oxidestack/libphonenumber libphonenumber-rs
```

然后通过路径依赖：`datafaker = { path = "../datafaker" }`。该 crate 需要 Rust 1.85 或更高版本。

## 文档

- 指南（英文、俄文、中文和西班牙文）：<https://oxidestack.github.io/datafaker/>，包括版本之间的
  [升级](docs/i18n/zh/guide/upgrading.md)。
- API 文档：crate 发布后位于 docs.rs；在此之前请使用 `cargo doc --open`。
- 每个版本的变更：[CHANGELOG.md](CHANGELOG.md)。

## 构建与测试

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

`tests/vectors/` 中保存了从真实 Datafaker 记录的输出；`scripts/record.sh` 可以重新生成它们（需要 JDK）。

## 获取帮助

关于本 crate 如何工作的问题，可以先问 [DeepWiki](https://deepwiki.com/oxidestack/datafaker)：它根据本仓库的代码和文档作答，
当它与文档不一致时，以文档为准。它解决不了的问题请到 [Discussions](https://github.com/oxidestack/datafaker/discussions)，
缺陷请提交到 [issues](https://github.com/oxidestack/datafaker/issues)。

## 贡献、安全、许可证

[CONTRIBUTING.md](CONTRIBUTING.md)、[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)、[GOVERNANCE.md](GOVERNANCE.md)、
[SECURITY.md](SECURITY.md)。代码：Apache-2.0（[LICENSE](LICENSE)）。文档：CC-BY-4.0（[LICENSE-docs](LICENSE-docs)）。
