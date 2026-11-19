# 升级

<!-- languages:begin -->
[English](../../../guide/upgrading.md) · [Русский](../../ru/guide/upgrading.md) · **中文** · [Español](../../es/guide/upgrading.md)
<!-- languages:end -->
<!-- translation of docs/guide/upgrading.md: b8b2b9b10c16 -->

如何把项目从它所依赖的 datafaker 版本升级到更新的版本。找到比你当前版本新的每个版本的章节，并按从旧到新的顺序逐一应用：从
0.1.0 升级到更高版本，意味着要应用中间每个版本的章节，再应用目标版本的章节。无需任何改动的版本也会明确说明。然后一步
[升级](#如何升级)到最新版本：从来不需要先安装某个中间版本。

每个章节列出该版本对你的代码的要求：**Action required** 下是需要在升级之前或同时进行的修改；**Deprecated**
下是目前仍然可用、但将在以后的版本中停止工作的内容；**Removed** 下是已经停止工作的内容。每个版本的其他所有变更见
[变更日志](https://github.com/oxidestack/datafaker/blob/main/CHANGELOG.md)。只有正式版本才有章节：候选版本（`-rc.N`）在其
GitHub release 页面上说明，它带来的变更写在发布它的正式版本的章节下。

你所使用版本的文档也在本站上：每页顶部的版本选择器可在各版本之间切换，旧版本的页面会链接到最新版本中的同一页面。

## 如何升级

版本号遵循[语义化版本](https://semver.org/lang/zh-CN/spec/v2.0.0.html)。在 1.0 之前，新的次版本（从 0.1 到 0.2）可能更改
API；新的修订版本（从 0.1.0 到 0.1.1）不会。

1. 阅读下面比你当前版本新的每个版本的章节，并对你的代码做出相应修改。
2. 修改你的 `Cargo.toml` 中的版本（`datafaker = "0.2"`），或者对于修订版本，运行 `cargo update -p datafaker`。
3. 运行 `cargo build` 和你的测试。**Action required** 中编译器能够发现的修改会在这里暴露出来；其余的是行为上的变化，
   章节中会说明如何检查。
4. 生成的值是契约的一部分：对于相同的种子，某个版本产生的结果与其所移植的 Datafaker 版本相同。移植了更新的 Datafaker 或更新的 locale 文件的版本会在 **Action required** 中说明，因为固定了生成值的测试届时可能需要更新为新值。

## 升级到 X.Y.Z（首个版本，版本号和日期尚未确定）

<!-- Placeholder: replace "X.Y.Z" in this heading with the first release's
     number when it is cut (expected 0.1.0). Every later release gets its own section above this one. -->

datafaker 的首个版本：Datafaker 2.7.0 的移植，对于相同的 `java.util.Random` 种子产生与 Datafaker 相同的输出。没有可以升级的旧版本。

### Action required

无。

### Deprecated

无。

### Removed

无。
