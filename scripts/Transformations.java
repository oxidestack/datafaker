// Records differential vectors of the transformers and fake sequences (T5) from Datafaker 2.7.0. Reads
// `seed TAB case` lines from stdin and prints each with the case's result appended (see
// tests/vectors_transformations.rs, which builds the same cases). Each line gets a fresh `new Faker(new Random(seed))`.
// A thrown exception prints as `ERR <simple class name>: <message>`; `\` TAB and newline are escaped as `\\` `\t` `\n`.
// `#` lines and blank lines pass through. Run by `scripts/record.sh tests/vectors/transformations.tsv Transformations`.
import java.io.*;
import java.math.BigDecimal;
import java.math.BigInteger;
import java.nio.charset.StandardCharsets;
import java.util.*;
import java.util.function.Function;
import java.util.function.Supplier;
import java.util.stream.Collectors;
import net.datafaker.Faker;
import net.datafaker.providers.base.Name;
import net.datafaker.sequence.FakeSequence;
import net.datafaker.transformations.*;
import net.datafaker.transformations.sql.SqlDialect;
import net.datafaker.transformations.sql.SqlTransformer;

@SuppressWarnings({"unchecked", "rawtypes"})
public class Transformations {
  public static void main(String[] a) throws Exception {
    BufferedReader in = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
    PrintStream out = new PrintStream(new FileOutputStream(FileDescriptor.out), true, StandardCharsets.UTF_8);
    String line;
    while ((line = in.readLine()) != null) {
      if (line.startsWith("#") || line.isBlank()) { out.println(line); continue; }
      String[] cols = line.split("\t");
      long seed = Long.parseLong(cols[0]);
      Function<Faker, Object> c = CASES.get(cols[1]);
      if (c == null) throw new IllegalStateException("unknown case " + cols[1]);
      String result;
      try {
        result = String.valueOf(c.apply(new Faker(new Random(seed))));
      } catch (Throwable e) {
        result = "ERR " + e.getClass().getSimpleName() + ": " + e.getMessage();
      }
      out.println(cols[0] + '\t' + cols[1] + '\t' + result.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n"));
    }
  }

  static Field f(String name, Supplier<?> s) {
    return Field.field(name, s);
  }

  static Field fn(String name, Function<?, ?> s) {
    return Field.field(name, s);
  }

  static Field c(String name, Field... fields) {
    return Field.compositeField(name, fields);
  }

  static Schema s(Field... fields) {
    return Schema.of(fields);
  }

  static Map<Object, Object> map(Object... kv) {
    Map<Object, Object> m = new LinkedHashMap<>();
    for (int i = 0; i < kv.length; i += 2) m.put(kv[i], kv[i + 1]);
    return m;
  }

  // Scalars, collections and maps of every runtime type the transformers tell apart.
  static Schema values(Faker faker) {
    return s(
        f("null", () -> null),
        f("int", () -> faker.number().randomDigit()),
        f("long", () -> 5L),
        f("short", () -> (short) 3),
        f("byte", () -> (byte) 4),
        f("float", () -> 2.5f),
        f("double", () -> 1.0E-5),
        f("bigInteger", () -> new BigInteger("123456789012345678901234567890")),
        f("bigDecimal", () -> new BigDecimal("1.50")),
        f("bool", () -> faker.bool().bool()),
        f("char", () -> 'q'),
        f("string", () -> " a \"b\" 'c' \\ / \t\n\u0001\u001f " + faker.name().firstName() + " "),
        f("json", () -> "#{json 'a','b'}"),
        f("list", () -> List.of("x", 1, false)),
        f("map", () -> map("k", "v", "n", 2)));
  }

  static Schema nameSchema(Faker faker) {
    return s(f("first", () -> faker.name().firstName()), f("digit", () -> faker.number().randomDigit()));
  }

  static String join(java.util.stream.Stream<?> stream) {
    return stream.map(String::valueOf).collect(Collectors.joining("\n"));
  }

  static final Map<String, Function<Faker, Object>> CASES = new LinkedHashMap<>();

  static void add(String name, Function<Faker, Object> f) {
    CASES.put(name, f);
  }

  static {
    // CSV
    add("csv.values", f -> CsvTransformer.builder().build().generate(values(f), 2));
    add("csv.options", f -> CsvTransformer.builder().separator(" | ").quote('\'').header(false).build()
        .generate(s(f("it's", () -> "it's " + f.name().lastName()), f("n", () -> f.number().randomDigit())), 3));
    add("csv.stream", f -> join(CsvTransformer.builder().build().generateStream(nameSchema(f), 3)));
    add("csv.streamNoHeader", f -> join(CsvTransformer.builder().header(false).build().generateStream(nameSchema(f), 2)));
    add("csv.nullName", f -> CsvTransformer.builder().build().generate(s(f(null, () -> "v")), 1));
    add("csv.composite", f -> CsvTransformer.builder().build().generate(s(c("c", f("a", () -> "b"))), 1));
    add("csv.list", f -> CsvTransformer.builder().build().generate(List.of("a", "b'c"), s(fn("v", x -> x), f("n", () -> f.name().firstName()))));
    add("csv.empty", f -> CsvTransformer.builder().build().generate(List.of(), nameSchema(f)));
    add("csv.limit0", f -> CsvTransformer.builder().build().generate(nameSchema(f), 0));
    // JSON
    add("json.values", f -> JsonTransformer.builder().build().generate(values(f), 2));
    add("json.objects", f -> JsonTransformer.builder().build().generate(s(
        f("objects", () -> new Object[]{"s", 1, null, List.of(2), new String[]{"n"}, c(null, f("in", () -> "x"))}),
        f("field", () -> List.of(f("simple", () -> "x"))),
        c(null, f("a", () -> f.name().firstName()), c("b", f(null, () -> null)))), 1));
    add("json.ints", f -> JsonTransformer.builder().build().generate(s(f("ints", () -> new int[]{1, 2})), 1));
    add("json.doubles", f -> JsonTransformer.builder().build().generate(s(f("d", () -> new double[]{1})), 1));
    add("json.stream", f -> join(JsonTransformer.builder().build().generateStream(nameSchema(f), 3)));
    add("json.stream1", f -> join(JsonTransformer.builder().withCommaBetweenObjects(false).build().generateStream(nameSchema(f), 1)));
    add("json.streamNegative", f -> join(JsonTransformer.builder().build().generateStream(nameSchema(f), -1)));
    add("json.limit1", f -> JsonTransformer.builder().build().generate(nameSchema(f), 1));
    add("json.limit0", f -> JsonTransformer.builder().build().generate(nameSchema(f), 0));
    add("json.emptyInput", f -> JsonTransformer.builder().build().generate(List.of(), nameSchema(f)));
    add("json.oneInput", f -> JsonTransformer.builder().build().generate(List.of("x"), s(fn("v", x -> x))));
    add("json.nullRows", f -> JsonTransformer.builder().build().generate((Iterable) 
        f.collection(() -> f.name().firstName()).nullRate(0.5).len(6).build(), s(f("n", () -> f.number().randomDigit()))));
    add("json.nullRowsFunction", f -> JsonTransformer.builder().build().generate((Iterable) 
        f.collection(() -> f.name().firstName()).nullRate(0.5).len(6).build(), s(fn("n", x -> x))));
    add("json.streamRows", f -> JsonTransformer.builder().withCommaBetweenObjects(false).build().generate((Iterable) 
        f.<Name>stream().suppliers(f::name).len(1, 4).build(), s(fn("first", n -> ((Name) n).firstName()), f("d", () -> f.number().randomDigit()))));
    // YAML
    add("yaml.values", f -> new YamlTransformer<>().generate(values(f), 2));
    add("yaml.nested", f -> new YamlTransformer<>().generate(s(
        f(" key ", () -> "v"), f("key", () -> "dup"),
        f("nested", () -> s(f("a", () -> f.name().firstName()), f("b", () -> List.of(1, s(f("x", () -> "y")))))),
        f("composite", () -> c("c", f("in", () -> "z"))),
        c("compositeField", f("in", () -> "z")),
        f("objects", () -> new Object[]{"s", null, List.of("t")})), 1));
    add("yaml.ints", f -> new YamlTransformer<>().generate(s(f("ints", () -> new int[]{1})), 1));
    add("yaml.nullName", f -> new YamlTransformer<>().generate(s(f(null, () -> "v")), 1));
    add("yaml.rows", f -> new YamlTransformer<>().generate((Iterable) f.collection(() -> f.name().firstName()).len(3).build(), s(fn("v", x -> x), f("d", () -> f.number().randomDigit()))));
    // XML
    add("xml.mixed", f -> new XmlTransformer.XmlTransformerBuilder<>().build().generate(s(
        f("text", () -> "<a & 'b'> \"" + f.name().firstName() + "\""),
        f(" spaced ", () -> null),
        f("empty", List::of),
        f("number", () -> 5),
        c("el", f("x", () -> "1 < 2"), f(null, () -> List.of(f("child", () -> "c"), c("sub", f("k", () -> "v"))))),
        c("onlyText", f(null, () -> "t"), f(null, () -> "ignored"))), 2));
    add("xml.pretty", f -> new XmlTransformer.XmlTransformerBuilder<>().pretty(true).build().generate(s(
        f("root", () -> List.of(f("a", () -> f.name().firstName()), f("b", () -> List.of(f("c", () -> "d"), f("e", () -> null)))))), 2));
    add("xml.attrInt", f -> new XmlTransformer.XmlTransformerBuilder<>().build().generate(s(c("el", f("x", () -> 1))), 1));
    add("xml.attrNull", f -> new XmlTransformer.XmlTransformerBuilder<>().build().generate(s(c("el", f("x", () -> null))), 1));
    add("xml.childString", f -> new XmlTransformer.XmlTransformerBuilder<>().build().generate(s(f("el", () -> List.of("x"))), 1));
    add("xml.nullName", f -> new XmlTransformer.XmlTransformerBuilder<>().build().generate(s(f(null, () -> "v")), 1));
    add("xml.rows", f -> new XmlTransformer.XmlTransformerBuilder<>().build().generate((Iterable) f.collection(() -> f.name().firstName()).len(2).build(), s(fn("v", x -> x))));
    // TOML
    add("toml.values", f -> new TomlTransformer<>().generate(values(f), 2));
    add("toml.arrays", f -> new TomlTransformer<>().generate(s(
        f("ints", () -> new int[]{1, 2}), f("chars", () -> new char[]{'a', '"'}), f("bools", () -> new boolean[]{true}),
        f("doubles", () -> new double[]{0.1, 1e20}), f("floats", () -> new float[]{}), f("objects", () -> new Object[]{"s", null, List.of(2), 'c'}),
        f(null, () -> "nullName"), f(null, () -> "dup"), f("table", () -> c("t", f("in", () -> f.name().firstName()))),
        c("compositeField", f("in", () -> "z")),
        f("tables", () -> List.of(s(f("id", () -> 1), f("sub", () -> s(f("x", () -> "y")))), c("c", f("id", () -> 2)))),
        f("after", () -> "a\\b")), 1));
    add("toml.badTables", f -> new TomlTransformer<>().generate(s(f("tables", () -> List.of(s(f("id", () -> 1)), 2))), 1));
    add("toml.rows", f -> new TomlTransformer<>().generate((Iterable) f.<String>stream().suppliers(() -> f.name().firstName()).len(2).build(), s(fn("v", x -> x), f("d", () -> f.number().randomDigit()))));
    // SQL
    add("sql.values", f -> SqlTransformer.builder().dialect(SqlDialect.CALCITE).build().generate(values(f), 2));
    add("sql.valuesNoDialect", f -> SqlTransformer.builder().build().generate(s(f("a", () -> "it's"), f("b", () -> 'q'), f("c", () -> map("k", null)), f("d", () -> List.of())), 1));
    add("sql.arrays", f -> SqlTransformer.builder().dialect(SqlDialect.SPARKSQL).build().generate(s(
        f("chars", () -> new char[]{'a'}), f("floats", () -> new float[]{0.1f}), f("objects", () -> new Object[]{"it's", null, new long[]{1}, List.of(1), map(1, true)}),
        c("st", f("x", () -> 1), f("y", () -> null), c("in", f("z", () -> "w")))), 1));
    add("sql.arrayNoDialect", f -> SqlTransformer.builder().build().generate(s(f("ints", () -> new int[]{1})), 1));
    add("sql.compositeNoDialect", f -> SqlTransformer.builder().build().generate(s(c("c", f("a", () -> 1))), 1));
    add("sql.nullName", f -> SqlTransformer.builder().build().generate(s(f(null, () -> 1)), 1));
    add("sql.nullNameForced", f -> SqlTransformer.builder().forceUseSqlQuoteIdentifier().build().generate(s(f(null, () -> 1)), 1));
    add("sql.sparkNullName", f -> SqlTransformer.builder().dialect(SqlDialect.SPARKSQL).build().generate(s(c("c", f(null, () -> 1))), 1));
    add("sql.emptyIdentifier", f -> SqlTransformer.builder().sqlQuoteIdentifier("").build().generate(nameSchema(f), 1));
    add("sql.identifiers", f -> {
      StringBuilder sb = new StringBuilder();
      Schema schema = s(f("lower", () -> 1), f("UPPER", () -> 2), f("Mixed.Dot", () -> 3), f("quo\"te", () -> 4), f("back`tick", () -> 5),
          f("br[ack]et", () -> 6), f("ÄÖ", () -> 7), f("äö", () -> 8));
      for (SqlDialect d : SqlDialect.values()) {
        sb.append(d).append(": ").append(SqlTransformer.builder().dialect(d).tableName("my_table").schemaName("Sch.ema").build().generate(schema, 1)).append('\n');
      }
      sb.append(SqlTransformer.builder().casing(net.datafaker.transformations.sql.Casing.TO_LOWER).sqlQuoteIdentifier("[]").tableName(null).build().generate(schema, 1));
      return sb.toString();
    });
    add("sql.dialects", f -> {
      StringBuilder sb = new StringBuilder();
      SqlTransformer.Case[] cases = SqlTransformer.Case.values();
      int i = 0;
      for (SqlDialect d : SqlDialect.values()) {
        SqlTransformer.Case k = cases[i++ % cases.length];
        sb.append(d).append(' ').append(k).append(":\n");
        sb.append(SqlTransformer.builder().dialect(d).batch(2).keywordCase(k).build().generate(nameSchema(f), 3)).append('\n');
        sb.append(SqlTransformer.builder().dialect(d).keywordCase(k).build().generate(s(f("n", () -> null), c("c", f("a", () -> f.name().firstName()), f("b", () -> new int[]{1}))), 1)).append('\n');
      }
      return sb.toString();
    });
    add("sql.batchStream", f -> join(SqlTransformer.builder().batch(2).dialect(SqlDialect.ORACLE).build().generateStream(nameSchema(f), 4)));
    add("sql.batchAll", f -> SqlTransformer.builder().batch().build().generate(nameSchema(f), 3));
    add("sql.batchAllStream", f -> join(SqlTransformer.builder().batch().build().generateStream(nameSchema(f), 3).limit(2)));
    add("sql.stream", f -> join(SqlTransformer.builder().keywordCase(SqlTransformer.Case.LOWERCASE).build().generateStream(nameSchema(f), 2)));
    add("sql.streamNegative", f -> join(SqlTransformer.builder().build().generateStream(nameSchema(f), -1)));
    add("sql.startStream", f -> SqlTransformer.builder().build().getStartStream(nameSchema(f)));
    add("sql.rows", f -> SqlTransformer.builder().batch(2).build().generate((Iterable) f.<String>stream().suppliers(() -> f.name().firstName()).len(3).build(), s(fn("v", x -> x), f("d", () -> f.number().randomDigit()))));
    add("sql.quote", f -> SqlTransformer.builder().quote('"').build().generate(s(f("q", () -> "say \"hi\" it's"), f("n", () -> f.name().lastName())), 2));
    // Sequences
    add("seq.collection", f -> f.collection(() -> f.name().firstName(), () -> String.valueOf(f.number().randomDigit())).nullRate(0.3).len(2, 8).build().get());
    add("seq.stream", f -> ((java.util.stream.Stream<Object>) f.stream(() -> f.name().firstName()).nullRate(0.5).len(4).build().get()).collect(Collectors.toList()));
    add("seq.streamString", f -> f.stream(() -> 1).nullRate(0.25).len(2, 5).build());
    add("seq.default", f -> f.collection(() -> f.number().randomDigit()).maxLen(-3).build().get());
    add("seq.minOnly", f -> f.collection(() -> f.number().randomDigit()).minLen(12).build().get());
    add("seq.singletons", f -> {
      FakeSequence<String> seq = f.collection(() -> "a", () -> "b", () -> "c").nullRate(0.4).build();
      List<String> l = new ArrayList<>();
      for (int i = 0; i < 6; i++) l.add(seq.singleton());
      return l;
    });
    add("seq.noSuppliers", f -> f.collection().len(1).build().get());
    add("seq.nullRateNaN", f -> f.collection(() -> 1).nullRate(Double.NaN).len(3).build().get());
    add("seq.streamMinOnly", f -> f.stream(() -> 1).minLen(3).build());
  }
}
