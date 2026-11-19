#!/usr/bin/env python3
"""Generates the YAML-only providers of Datafaker 2.7.0 from its Java sources.

A provider class is generated when every public method it declares takes no argument, returns `String` and only
resolves a YAML key: `return resolve("a.b");` or `return faker.resolve("a.b");` (string constants of the class and
`+` concatenations of them count as a key). Every other provider class is written by hand as
`src/providers/<module>.rs`; such a module is found by its `pub struct <Class><'a>` and takes precedence.

Writes, from the repository root (`scripts/gen-providers.py` with no arguments does all three):
  src/providers/generated.rs        the generated providers (`yaml_provider!`) and the `registry!` of every ported
                                    provider, hand-written ones included, in the order of the registry interfaces
  tests/upstream_generated.rs       the provider-list cases (`TestSpec.of`) of upstream's test classes, top-level
                                    contexts only, for every ported provider method; nested classes (other locales)
                                    and cases with arguments are reported and ported by hand
  tests/vectors/providers.tsv       vector rows: every no-argument method of each generated provider (and of the
                                    hand-written ones not covered elsewhere) on five seeds in English and one seed
                                    in el_GR and de; rows already recorded keep their results, new rows get theirs
                                    from `scripts/record.sh tests/vectors/providers.tsv`

Options: --registry <BaseProviders.java> (repeatable; later tranches add their registry interfaces), --check (exit 1
when a written file would change), --report (print what was skipped and why).
"""

import argparse
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
JAVA = ROOT / "upstream/datafaker/src/main/java/net/datafaker"
TESTS = ROOT / "upstream/datafaker/src/test/java/net/datafaker"
DEFAULT_REGISTRIES = [
    "providers/base/BaseProviders.java", "providers/base/ProviderRegistration.java",
    "providers/entertainment/EntertainmentProviders.java", "providers/food/FoodProviders.java",
    "providers/healthcare/HealthcareProviders.java", "providers/sport/SportProviders.java",
    "providers/videogame/VideoGameProviders.java",
]
# Test packages named otherwise than their main package.
TEST_PACKAGES = {"providers/food": "providers/foods"}
# Upstream's abstract test bases whose `faker` is an English faker (`new Faker()`, `new EntertainmentFaker()`, ...).
ENGLISH_BASES = ["BaseFakerTest", "EntertainmentFakerTest", "FoodFakerTest", "HealthcareFakerTest", "SportFakerTest",
                 "VideoGameFakerTest"]
FAKERS = r"(?:Base|Entertainment|Food|Healthcare|Sport|VideoGame)?Faker"

RUST_KEYWORDS = {
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
    "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static", "struct", "super",
    "trait", "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box",
    "do", "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
}

SEEDS = [0, 1, 42, 9999, -7]
# Hand-written providers whose vectors are elsewhere (tests/vectors/faker.tsv, number.tsv, images.tsv).
VECTORS_ELSEWHERE = {"Credentials", "Fingerprint", "Image", "Internet", "Lorem", "Name", "Number", "Options", "Text"}
# Methods whose results a JVM does not reproduce for a seed, so no vector row calls them (`*`: every method):
# Money draws from `Currency.getAvailableCurrencies()`, a HashSet of objects with identity hash codes; methods that
# read the clock (`LocalDate.now()`, `Instant.now()`) give other values on every run.
NOT_REPRODUCIBLE = {
    "Money": {"*"},
    "Country": {"currency", "currencyCode"},
    # The clock: the date providers' reference-date overloads are in tests/vectors/dates.tsv.
    "Business": {"creditCardExpiry"},
    "DateAndTime": {"*"},
    "Time": {"*"},
    "TimeAndDate": {"*"},
    # The id number generators' birthdays: in tests/vectors/idnumbers.tsv, replayed on the date it was recorded.
    "IdNumber": {"*"},
}
LOCALE_ROWS = [("fakerElGr", 7), ("fakerDe", 7)]


def snake(name):
    """`keySkills` → `key_skills`, `CNPJ` → `cnpj`, `ipV4Address` → `ip_v4_address`."""
    s = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", name)
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s)
    return s.lower()


def rust_ident(name):
    s = snake(name)
    return "r#" + s if s in RUST_KEYWORDS else s


def strip_comments(src):
    """Drops comments, keeping string, text-block and char literals intact."""
    out = []
    i = 0
    n = len(src)
    while i < n:
        if src.startswith('"""', i):
            j = src.index('"""', i + 3) + 3
            out.append(src[i:j])
            i = j
        elif src[i] in "\"'":
            q = src[i]
            j = i + 1
            while src[j] != q:
                j += 2 if src[j] == "\\" else 1
            out.append(src[i:j + 1])
            i = j + 1
        elif src.startswith("//", i):
            i = src.find("\n", i)
            i = n if i < 0 else i
        elif src.startswith("/*", i):
            i = src.index("*/", i + 2) + 2
            out.append(" ")
        else:
            out.append(src[i])
            i += 1
    return "".join(out)


def block_end(src, i):
    """Index just past the `}` closing the block whose `{` precedes `i` (comments already stripped)."""
    depth = 1
    while depth:
        c = src[i]
        if src.startswith('"""', i):
            i = src.index('"""', i + 3) + 3
            continue
        if c in "\"'":
            j = i + 1
            while src[j] != c:
                j += 2 if src[j] == "\\" else 1
            i = j + 1
            continue
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
        i += 1
    return i


def java_string(literal):
    """The value of a Java string literal's body."""
    out = []
    i = 0
    while i < len(literal):
        c = literal[i]
        if c == "\\":
            e = literal[i + 1]
            i += 2
            if e == "u":
                out.append(chr(int(literal[i:i + 4], 16)))
                i += 4
            else:
                out.append({"n": "\n", "t": "\t", "r": "\r", "b": "\b", "f": "\f"}.get(e, e))
            continue
        out.append(c)
        i += 1
    return "".join(out)


def string_expr(expr, constants):
    """A key expression of string literals and constants joined by `+`; None for anything else."""
    parts = []
    rest = expr.strip()
    while True:
        m = re.match(r'"((?:[^"\\]|\\.)*)"|([A-Za-z_]\w*)', rest)
        if not m:
            return None
        if m.group(2) is not None:
            if m.group(2) not in constants:
                return None
            parts.append(constants[m.group(2)])
        else:
            parts.append(java_string(m.group(1)))
        rest = rest[m.end():].strip()
        if not rest:
            return "".join(parts)
        if not rest.startswith("+"):
            return None
        rest = rest[1:].strip()


class Provider:
    def __init__(self, cls, accessor, path):
        self.cls = cls
        self.accessor = accessor
        self.path = path
        self.package = ".".join(path.relative_to(JAVA.parent.parent).with_suffix("").parts[:-1])
        self.methods = []  # (java name, via, key)
        self.reasons = []
        self.module = None  # hand-written module, if any

    @property
    def generated(self):
        return self.module is None and not self.reasons

    def parse(self):
        src = strip_comments(self.path.read_text(encoding="utf-8"))
        constants = {}
        for m in re.finditer(r'(?:static\s+final|final\s+static)\s+String\s+(\w+)\s*=\s*([^;]+);', src):
            value = string_expr(m.group(2), constants)
            if value is not None:
                constants[m.group(1)] = value
        if not re.search(r"\bextends\s+AbstractProvider\b", src):
            self.reasons.append("does not extend AbstractProvider")
        body_start = src.index("{", re.search(r"\bclass\s+" + self.cls + r"\b", src).start()) + 1
        body = src[body_start:block_end(src, body_start) - 1]
        depth_zero = re.finditer(r"public\s+([\w<>\[\], ?.]+?)\s+(\w+)\s*\(([^)]*)\)\s*(?:throws\s+[\w., ]+)?\{", body)
        for m in depth_zero:
            ret, name, params = m.group(1).strip(), m.group(2), m.group(3).strip()
            end = block_end(body, m.end())
            if body[:m.start()].count("{") - body[:m.start()].count("}") != 0:
                continue  # a member of a nested type
            text = body[m.end():end - 1].strip()
            k = re.fullmatch(r"return\s+(faker\.)?resolve\((.*)\)\s*;", text, flags=re.S)
            key = string_expr(k.group(2), constants) if k else None
            if ret != "String" or params or key is None:
                self.reasons.append(f"{name}({params})")
                continue
            self.methods.append((name, "faker" if k.group(1) else "provider", key))
        if re.search(r"\bpublic\s+(?:static\s+)?(?:final\s+)?(?:class|enum|record|interface)\b", body):
            self.reasons.append("declares a public nested type")
        rust = [rust_ident(n) for n, _, _ in self.methods]
        if len(set(rust)) != len(rust):
            self.reasons.append("two methods with one snake_case name")


def registry_providers(registries):
    providers = []
    for reg in registries:
        path = JAVA / reg
        src = strip_comments(path.read_text(encoding="utf-8"))
        for m in re.finditer(r"default\s+(\w+)\s+(\w+)\(\)\s*\{\s*return\s+getProvider\(\s*\1\.class,\s*\1::new\)", src):
            cls, accessor = m.group(1), m.group(2)
            providers.append(Provider(cls, accessor, path.parent / f"{cls}.java"))
    return providers


def hand_written():
    """Class name → module of each hand-written provider module."""
    modules = {}
    for f in sorted((ROOT / "src/providers").glob("*.rs")):
        if f.stem in ("mod", "generated"):
            continue
        for m in re.finditer(r"pub struct (\w+)<'a>", f.read_text(encoding="utf-8")):
            modules.setdefault(m.group(1), f.stem)
    return modules


def generate_source(providers):
    out = [
        "//! Generated by `scripts/gen-providers.py` from Datafaker 2.7.0's sources: the providers whose public methods",
        "//! only resolve YAML keys, and the registry of every ported provider. Do not edit; run the script.",
        "",
    ]
    for p in providers:
        if not p.generated:
            continue
        mod = rust_ident(p.cls)
        out.append(f"/// The [`{p.cls}`] provider, port of `{p.package}.{p.cls}`: each method resolves one YAML key.")
        out.append(f"pub mod {mod} {{")
        out.append("    yaml_provider! {")
        out.append(f"        /// `{p.package}.{p.cls}`.")
        out.append(f"        {p.cls} {{")
        for name, via, key in p.methods:
            out.append(f"            /// `{key}`{' (resolved on the faker)' if via == 'faker' else ''}.")
            out.append(f"            {rust_ident(name)}, \"{name}\", {via} \"{key}\";")
        out.append("        }")
        out.append("    }")
        out.append("}")
        out.append(f"pub use {mod}::{p.cls};")
        out.append("")
    out.append("registry! {")
    for p in providers:
        if p.module:
            out.append(f"    {p.cls} => super::{p.module}, \"{p.accessor}\", {rust_ident(p.accessor)}, \"{p.package}\";")
        elif p.generated:
            out.append(f"    {p.cls} => {rust_ident(p.cls)}, \"{p.accessor}\", {rust_ident(p.accessor)}, \"{p.package}\";")
    out.append("}")
    return "\n".join(out) + "\n"


def rust_methods(p):
    """The Rust method names a ported provider has (generated ones, or the `pub fn x(&self)` of its module), each
    with whether it returns a `Result` (generated ones all do)."""
    if p.generated:
        return {rust_ident(n): True for n, _, _ in p.methods}
    src = (ROOT / "src/providers" / f"{p.module}.rs").read_text(encoding="utf-8")
    return {m.group(1): m.group(2).startswith("Result") for m in re.finditer(r"pub fn (r#\w+|\w+)\(&self\) -> ([\w:]+)", src)}


def call(prov, method):
    """`f.<provider>().<method>()`, unwrapped when the method returns a `Result`."""
    unwrap = ".unwrap()" if rust_methods(prov).get(method, True) else ""
    return f"f.{rust_ident(prov.accessor)}().{method}(){unwrap}"


def raw(s):
    if not s:
        return '""'
    if '"' not in s and "\\" not in s:
        return f'"{s}"'
    hashes = "#" if '"' in s else ""
    while '"' + hashes in s:
        hashes += "#"
    return f"r{hashes}\"{s}\"{hashes}"


def receiver(src, var, by_class):
    """The provider a test's variable holds, when it comes from the test's default (English) faker."""
    decl = re.search(r"\b([A-Z]\w*)\s+" + var + r"\b\s*([=;,)])", src)
    if not decl or decl.group(1) not in by_class:
        return None
    if decl.group(2) == "=":
        init = src[decl.end():src.index(";", decl.end())].strip()
        if not re.fullmatch(r"(?:faker|getFaker\(\)|new " + FAKERS + r"\(\))\.\w+\(\)", init):
            return None
    return by_class[decl.group(1)]


ASSERTION = re.compile(r"assertThat\(((?:faker|getFaker\(\)|new " + FAKERS + r"\(\))\.\w+\(\)|\w+)\.(\w+)\(\)\)((?:\.\w+\([^;]*?\))+)")
# The AssertJ checks a generated test reproduces: those without an argument, and those taking a string.
CHECK = re.compile(r"\.(?:(isNotEmpty|isNotBlank)\(\)|(matches|doesNotContain|startsWith|contains)\(\s*((?:\"(?:[^\"\\]|\\.)*\"|[A-Za-z_]\w*)(?:\s*\+\s*(?:\"(?:[^\"\\]|\\.)*\"|[A-Za-z_]\w*))*)\s*\))")


def default_faker(src):
    """Whether the test's `faker` (and `getFaker()`) is an English faker: an `ENGLISH_BASES` subclass's, or a field
    `new Faker()` (or a themed faker)."""
    bases = "|".join(ENGLISH_BASES)
    return bool(re.search(r"extends\s+(?:" + bases + r")\b|\bfaker\s*=\s*new\s+" + FAKERS + r"\(\)\s*;", src))


def assertion_tests(src, top, constants, by_class, by_accessor, test, report):
    """`@Test`/`@RepeatedTest(n)` methods whose body only asserts on provider calls with `CHECK`s (match a regex, are
    not empty or blank, contain a string, …): (name, repetitions, [(provider, rust method, [(check, argument)])])."""
    tests = []
    for m in re.finditer(r"@(Test|RepeatedTest\((\d+)\))\s+(?:@\w+(?:\([^)]*\))?\s+)*(?:public\s+|protected\s+)?void\s+(\w+)\(\)\s*\{", top):
        name, reps = m.group(3), int(m.group(2) or 1)
        body = top[m.end():block_end(top, m.end()) - 1]
        statements = [st.strip() for st in body.split(";") if st.strip()]
        checks = []
        for st in statements:
            a = ASSERTION.fullmatch(st)
            if not a:
                break
            recv, method, chain = a.groups()
            r = re.fullmatch(r"(faker|getFaker\(\)|new " + FAKERS + r"\(\))\.(\w+)\(\)", recv)
            if r:
                prov = by_accessor.get(r.group(2)) if r.group(1).startswith("new") or default_faker(src) else None
            else:
                prov = receiver(src, recv, by_class)
            asserts, pos = [], 0
            while pos < len(chain):
                c = CHECK.match(chain, pos)
                arg = c and (string_expr(c.group(3), constants) if c.group(2) else "")
                if arg is None:
                    break
                asserts.append((c.group(1) or c.group(2), arg))
                pos = c.end()
            if prov is None or pos < len(chain) or rust_ident(method) not in rust_methods(prov):
                break
            checks.append((prov, rust_ident(method), asserts))
        else:
            if checks:
                tests.append((name, reps, checks))
                continue
        report.append(f"{test.name}: {name} (by hand)")
    for m in re.finditer(r"@ParameterizedTest\b.*?\bvoid\s+(\w+)\(", top, flags=re.S):
        report.append(f"{test.name}: {m.group(1)} (parameterized, by hand)")
    return tests


def ported_by_hand():
    """Test classes a hand-written port names in its first line (`` `NameTest` ``): the generator leaves them alone."""
    names = set()
    for f in (ROOT / "tests").glob("upstream_*.rs"):
        if f.name != "upstream_generated.rs":
            first = f.read_text(encoding="utf-8").split("\n", 1)[0]
            names.update(re.findall(r"`(\w+Test)`", first))
    return names


def test_package(path):
    """The test directory of a main source directory (`providers/food` -> `providers/foods`)."""
    rel = path.relative_to(JAVA).as_posix()
    return TESTS / TEST_PACKAGES.get(rel, rel)


def test_files(providers):
    """The test classes to read: `<Class>Test.java` of each ported provider, in registry order, then the other test
    classes of the providers' test packages (`EntertainmentTest` tests `Show`), by name; abstract bases left out."""
    files = []
    for p in providers:
        test = test_package(p.path.parent) / f"{p.cls}Test.java"
        if (p.module or p.generated) and test.exists():
            files.append(test)
    seen = set(files)
    for d in dict.fromkeys(test_package(p.path.parent) for p in providers):
        for test in sorted(d.glob("*Test.java")):
            text = test.read_text(encoding="utf-8")
            if test not in seen and not re.search(r"\babstract\s+class\b", text):
                files.append(test)
                seen.add(test)
    return files


def test_cases(providers, report):
    """Per upstream test class: the provider-list cases it has for ported provider methods."""
    by_hand = ported_by_hand()
    by_class = {p.cls: p for p in providers if p.module or p.generated}
    by_accessor = {p.accessor: p for p in by_class.values()}
    classes = {}
    for test in test_files(providers):
        if test in classes or test.stem in by_hand:
            continue
        src = strip_comments(test.read_text(encoding="utf-8"))
        top = src.split("@Nested")[0]
        if "arguments(TestSpec" in src:
            report.append(f"{test.name}: specs over several fakers (by hand)")
            continue
        cases = []
        constants = {}
        for c in re.finditer(r"(?:static\s+final|final\s+static)\s+String\s+(\w+)\s*=\s*([^;]+);", src):
            value = string_expr(c.group(2), constants)
            if value is not None:
                constants[c.group(1)] = value
        arg = r"(\"(?:[^\"\\]|\\.)*\"|[A-Z][A-Z0-9_]*)"
        specs = [m for m in re.finditer(r"TestSpec\.of\(\s*([^,]*?)\s*,\s*" + arg + r"\s*(?:,\s*" + arg + r"\s*)?\)", top, flags=re.S)
                 if all(g is None or string_expr(g, constants) is not None for g in m.groups()[1:])]
        unparsed = top.count("TestSpec.of(") - len(specs)
        if unparsed:
            report.append(f"{test.name}: {unparsed} TestSpec.of with a non-literal key or regex (by hand)")
        for m in specs:
            supplier, key, regex = m.group(1), string_expr(m.group(2), constants), string_expr(m.group(3) or '""', constants)
            target = None
            r = re.fullmatch(r"(\w+)::(\w+)", supplier)
            if r:
                var, method = r.groups()
                decl = re.search(r"\b([A-Z]\w*)\s+" + var + r"\b\s*[=;,)]", src)
                target = (by_class.get(decl.group(1)) if decl else None, method)
            r = re.fullmatch(r"\(\)\s*->\s*faker\.(\w+)\(\)\.(\w+)\(\)", supplier)
            if r:
                target = (by_accessor.get(r.group(1)), r.group(2))
            r = re.fullmatch(r"\(\)\s*->\s*(\w+)\.(\w+)\(\)", supplier)
            if r and r.group(1) != "faker":
                decl = re.search(r"\b([A-Z]\w*)\s+" + r.group(1) + r"\b\s*[=;,)]", src)
                target = (by_class.get(decl.group(1)) if decl else None, r.group(2))
            if target is None:
                report.append(f"{test.name}: `{supplier}` for {key} (by hand)")
                continue
            prov, method = target
            if prov is None or rust_ident(method) not in rust_methods(prov):
                report.append(f"{test.name}: {method} for {key}: provider method not ported")
                continue
            cases.append((prov, rust_ident(method), key, regex))
        if "@Nested" in src:
            report.append(f"{test.name}: nested classes (by hand)")
        asserts = assertion_tests(src, top, constants, by_class, by_accessor, test, report)
        if cases or asserts:
            classes[test] = (cases, asserts)
    return classes


def generate_tests(classes):
    out = [
        "//! Generated by `scripts/gen-providers.py` from upstream's test classes (Datafaker 2.7.0,",
        "//! `providers/**/<Class>Test.java`), for the ported providers: the provider-list cases (`TestSpec.of`), checked",
        "//! as `AbstractProviderListTest.testProviderList` does (one test per class, as the parameterized test counts",
        "//! once), and the tests that only assert that provider calls match a regex. Unseeded English fakers, as",
        "//! upstream; repeated tests run their body as often. Nested classes for other locales and every other test",
        "//! are ported by hand in `tests/upstream_*.rs`. Do not edit; run the script.",
        "",
        "mod common;",
        "",
        "use common::{assert_matches, provider_list};",
        "use datafaker::Faker;",
    ]
    for test, (cases, asserts) in sorted(classes.items(), key=lambda kv: kv[0].name):
        prefix = snake(test.stem.removesuffix("Test"))
        if cases:
            out.append("")
            out.append(f"/// `{test.relative_to(TESTS)}`: `providerListTest`.")
            out.append("#[test]")
            out.append(f"fn {prefix}_provider_list() {{")
            out.append("    let f = Faker::new();")
            for prov, method, key, regex in cases:
                out.append(f"    provider_list(&f, \"{key}\", &{call(prov, method)}, {raw(regex)});")
            out.append("}")
        for name, reps, checks in asserts:
            out.append("")
            out.append(f"/// `{test.relative_to(TESTS)}`: `{name}`.")
            out.append("#[test]")
            out.append(f"fn {prefix}_{snake(name)}() {{")
            out.append("    let f = Faker::new();")
            indent = "    "
            if reps > 1:
                out.append(f"    for _ in 0..{reps} {{")
                indent = "        "
            for prov, method, asserts in checks:
                c = call(prov, method)
                if len(asserts) > 1:
                    out.append(f"{indent}let v = {c};")
                    c = "v"
                for check, arg in asserts:
                    out.append(indent + {
                        "matches": f"assert_matches({raw(arg)}, &{c});",
                        "isNotEmpty": f"assert!(!{c}.is_empty());",
                        "isNotBlank": f"assert!(!{c}.trim().is_empty());",
                        "doesNotContain": f"assert!(!{c}.contains({raw(arg)}), \"{{}}\", {c});",
                        "startsWith": f"assert!({c}.starts_with({raw(arg)}), \"{{}}\", {c});",
                        "contains": f"assert!({c}.contains({raw(arg)}), \"{{}}\", {c});",
                    }[check])
            if reps > 1:
                out.append("    }")
            out.append("}")
    return "\n".join(out) + "\n"


def vector_methods(p):
    """The no-argument methods a vector row calls: all of a generated provider's, and those of a hand-written one
    that its method table lists."""
    if p.generated:
        return [n for n, _, _ in p.methods]
    skip = NOT_REPRODUCIBLE.get(p.cls, set())
    if p.cls in VECTORS_ELSEWHERE or "*" in skip:
        return []
    src = (ROOT / "src/providers" / f"{p.module}.rs").read_text(encoding="utf-8")
    return [n for n in re.findall(r'Method::new\("(\w+)", &\[\]', src) if n not in skip]


def generate_vectors(providers):
    out = ["# Generated by scripts/gen-providers.py: every no-argument method of the ported providers on new Faker(new",
           "# Random(seed)), and on el_GR and de fakers; recorded from Datafaker 2.7.0 by scripts/record.sh"]
    for p in providers:
        methods = vector_methods(p) if p.generated or p.module else []
        if not methods:
            continue
        for root, seeds in [("faker", SEEDS)] + [(r, [s]) for r, s in LOCALE_ROWS]:
            for seed in seeds:
                calls = ";".join(f"{root}.{p.accessor}().{n}()" for n in methods)
                out.append(f"{seed}\t{calls}")
    return "\n".join(out) + "\n"


def merge_vectors(path, generated):
    """Keeps recorded results of rows whose seed and calls are unchanged."""
    recorded = {}
    if path.exists():
        for line in path.read_text(encoding="utf-8").splitlines():
            cols = line.split("\t")
            if len(cols) > 2:
                recorded[(cols[0], cols[1])] = line
    return "\n".join(recorded.get(tuple(l.split("\t")[:2]), l) for l in generated.splitlines()) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--registry", action="append")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--report", action="store_true")
    ap.add_argument("--hand-only", action="store_true", help="register the hand-written providers only")
    args = ap.parse_args()
    providers = registry_providers(args.registry or DEFAULT_REGISTRIES)
    modules = hand_written()
    for p in providers:
        p.module = modules.get(p.cls)
        p.parse()
        if args.hand_only and not p.module:
            p.reasons.append("--hand-only")
    report = []
    outputs = {
        ROOT / "src/providers/generated.rs": generate_source(providers),
        ROOT / "tests/upstream_generated.rs": generate_tests(test_cases(providers, report)),
    }
    vectors = ROOT / "tests/vectors/providers.tsv"
    outputs[vectors] = merge_vectors(vectors, generate_vectors(providers))
    changed = []
    for path, text in outputs.items():
        if not path.exists() or path.read_text(encoding="utf-8") != text:
            changed.append(path.relative_to(ROOT))
            if not args.check:
                path.write_text(text, encoding="utf-8")
    if args.report:
        gen = [p.cls for p in providers if p.generated]
        hand = [p.cls for p in providers if p.module]
        print(f"generated {len(gen)}: {' '.join(gen)}")
        print(f"hand-written {len(hand)}: {' '.join(hand)}")
        for p in providers:
            if not p.generated and not p.module:
                print(f"not ported: {p.cls}: {', '.join(p.reasons)}")
        for line in report:
            print(f"tests: {line}")
    for path in changed:
        print(f"{'differs' if args.check else 'wrote'}: {path}", file=sys.stderr)
    return 1 if args.check and changed else 0


if __name__ == "__main__":
    sys.exit(main())
