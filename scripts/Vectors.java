// Records differential vectors from Datafaker 2.7.0. Reads `seed TAB call[;call...]` lines from stdin and prints each
// with what every call returned appended, one TAB-separated column per call (see tests/vectors.rs). `#` lines and
// blank lines pass through. Each line gets one fresh `new Random(seed)`, shared by:
//   random   the java.util.Random itself
//   rs       new RandomService(random)
// and one fresh `new SplittableRandom(seed)`, shared by:
//   splittable  the SplittableRandom itself
//   srs         new RandomService(splittable)
//   value    Identity: `value.of(x)` returns its argument, so a row can show what a factory builds or throws
//   faker    new Faker(random)
//   fakerDe, fakerEnUs, fakerElGr, fakerRu, fakerFa, fakerPtBr, fakerZhCn, fakerJa, fakerTr
//            new Faker(<that locale>, random)
//   faker_<language>[_<COUNTRY>]  (faker_sv_SE) new Faker(new Locale(language, country), random)
// A `# today` line is rewritten as `# today <LocalDate.now()>`: the date the clock-reading calls of the file (birthdays)
// were recorded on, which tests/vectors.rs replays them on. The run fails if the date changes while it records.
// A row uses one faker root only.
// A call is a chain such as `faker.internet().password(8,16,true,true,true)`; arguments are int, long (`5L`),
// boolean, double (`1.5`), char (`'x'`), "string" or `null` literals, enum constants (`TimeUnit.SECONDS`, see
// ENUMS) and objects built by a factory from literals (`Instant("2023-11-14T22:13:20Z")`, see FACTORIES). A result prints with String.valueOf (arrays with
// Arrays.toString); a thrown exception prints as `ERR <simple class name>: <message>`. In output, `\` `TAB` and
// newline are escaped as `\\` `\t` `\n`.
import java.io.*;
import java.lang.reflect.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import java.util.function.Function;
import net.datafaker.Faker;
import net.datafaker.service.RandomService;

public class Vectors {
  public static void main(String[] a) throws Exception {
    BufferedReader in = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
    PrintStream out = new PrintStream(new FileOutputStream(FileDescriptor.out), true, StandardCharsets.UTF_8);
    String line;
    java.time.LocalDate today = java.time.LocalDate.now();
    while ((line = in.readLine()) != null) {
      if (line.startsWith("# today")) { out.println("# today " + today); continue; }
      if (line.startsWith("#") || line.isBlank()) { out.println(line); continue; }
      String[] cols = line.split("\t");
      long seed = Long.parseLong(cols[0]);
      Random random = new Random(seed);
      Map<String, Object> roots = new HashMap<>();
      roots.put("random", random);
      roots.put("rs", new RandomService(random));
      SplittableRandom splittable = new SplittableRandom(seed);
      roots.put("splittable", splittable);
      roots.put("srs", new RandomService(splittable));
      roots.put("value", new Identity());
      roots.put("faker", new Faker(random));
      roots.put("fakerDe", new Faker(new Locale("de"), random));
      roots.put("fakerEnUs", new Faker(new Locale("en", "US"), random));
      roots.put("fakerElGr", new Faker(new Locale("el", "GR"), random));
      roots.put("fakerRu", new Faker(new Locale("ru"), random));
      roots.put("fakerFa", new Faker(new Locale("fa"), random));
      roots.put("fakerPtBr", new Faker(new Locale("pt", "BR"), random));
      roots.put("fakerZhCn", new Faker(new Locale("zh", "CN"), random));
      roots.put("fakerJa", new Faker(new Locale("ja"), random));
      roots.put("fakerTr", new Faker(new Locale("tr"), random));
      StringBuilder sb = new StringBuilder(cols[0]).append('\t').append(cols[1]);
      for (String call : splitCalls(cols[1])) {
        String result;
        try {
          result = show(eval(call.trim(), roots));
        } catch (Throwable e) {
          Throwable t = e instanceof InvocationTargetException ? e.getCause() : e;
          result = "ERR " + t.getClass().getSimpleName() + ": " + t.getMessage();
        }
        sb.append('\t').append(result.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n"));
      }
      out.println(sb);
    }
    if (!java.time.LocalDate.now().equals(today)) {
      System.err.println("the date changed while recording; record again");
      System.exit(1);
    }
  }

  static String show(Object o) {
    if (o instanceof byte[] b) return Arrays.toString(b);
    if (o instanceof int[] b) return Arrays.toString(b);
    return String.valueOf(o);
  }

  // Splits on `;` outside string literals.
  static List<String> splitCalls(String s) {
    List<String> res = new ArrayList<>();
    StringBuilder cur = new StringBuilder();
    boolean str = false;
    for (int i = 0; i < s.length(); i++) {
      char c = s.charAt(i);
      if (str && c == '\\') { cur.append(c).append(s.charAt(++i)); continue; }
      if (c == '"') str = !str;
      if (c == ';' && !str) { res.add(cur.toString()); cur.setLength(0); continue; }
      cur.append(c);
    }
    res.add(cur.toString());
    return res;
  }

  static int pos;
  static String src;

  public static class Identity {
    public Object of(Object o) {
      return o;
    }
  }

  // Enum classes by simple name.
  static final Map<String, Class<?>> ENUMS = Map.ofEntries(
      Map.entry("TimeUnit", java.util.concurrent.TimeUnit.class),
      Map.entry("ChronoUnit", java.time.temporal.ChronoUnit.class),
      Map.entry("GenderRequest", net.datafaker.providers.base.IdNumber.GenderRequest.class),
      Map.entry("PolishGender", net.datafaker.idnumbers.PolishIdNumber.Gender.class),
      // Themed providers' nested enums, by provider and enum name (two are called `Quote`).
      Map.entry("DuneQuote", net.datafaker.providers.entertainment.Dune.Quote.class),
      Map.entry("DuneSaying", net.datafaker.providers.entertainment.Dune.Saying.class),
      Map.entry("TronQuote", net.datafaker.providers.entertainment.Tron.Quote.class),
      Map.entry("TronCharacter", net.datafaker.providers.entertainment.Tron.Character.class),
      Map.entry("TronAlternateCharacterSpelling", net.datafaker.providers.entertainment.Tron.AlternateCharacterSpelling.class),
      Map.entry("CoffeeCountry", net.datafaker.providers.food.Coffee.Country.class));

  // Object factories by name, from their literal arguments. `Parse("pattern", "text")` is
  // `DateTimeFormatter.ofPattern(pattern).parse(text)` shown as `Parsed.toString()` with its fields sorted by name
  // (upstream's order follows identity hash codes); `ParseZ` the same with `withZone(ZoneId.systemDefault())`.
  static final Map<String, Function<List<Object>, Object>> FACTORIES = Map.ofEntries(
      Map.entry("Instant", a -> java.time.Instant.parse((String) a.get(0))),
      Map.entry("LocalDate", a -> java.time.LocalDate.parse((String) a.get(0))),
      Map.entry("LocalTime", a -> java.time.LocalTime.parse((String) a.get(0))),
      Map.entry("Period", a -> java.time.Period.parse((String) a.get(0))),
      Map.entry("Date", a -> new Date(((Number) a.get(0)).longValue())),
      Map.entry("DateText", a -> new Date((String) a.get(0))),
      Map.entry("Timestamp", a -> new java.sql.Timestamp(((Number) a.get(0)).longValue())),
      Map.entry("Parse", a -> sortedParsed(java.time.format.DateTimeFormatter.ofPattern((String) a.get(0)).parse((String) a.get(1)))),
      Map.entry("ParseZ", a -> sortedParsed(java.time.format.DateTimeFormatter.ofPattern((String) a.get(0))
          .withZone(java.time.ZoneId.systemDefault()).parse((String) a.get(1)))),
      Map.entry("Strings", a -> a.stream().map(String.class::cast).toArray(String[]::new)),
      Map.entry("IdNumberRequest", a -> new net.datafaker.providers.base.IdNumber.IdNumberRequest(
          (Integer) a.get(0), (Integer) a.get(1), (net.datafaker.providers.base.IdNumber.GenderRequest) a.get(2))));

  // `Parsed.toString()` with the `{field=value, ...}` map sorted by field name.
  static String sortedParsed(Object parsed) {
    String s = parsed.toString();
    int end = s.indexOf('}');
    String map = s.substring(1, end);
    List<String> entries = map.isEmpty() ? new ArrayList<>() : new ArrayList<>(Arrays.asList(map.split(", ")));
    entries.sort(Comparator.comparing(e -> e.substring(0, e.indexOf('='))));
    return "{" + String.join(", ", entries) + s.substring(end);
  }

  static Object eval(String call, Map<String, Object> roots) throws Exception {
    src = call;
    pos = 0;
    String root = ident();
    Object target = roots.get(root);
    if (target == null && root.startsWith("faker_")) {
      String[] parts = root.substring(6).split("_");
      Locale locale = parts.length > 1 ? new Locale(parts[0], parts[1]) : new Locale(parts[0]);
      target = new Faker(locale, (Random) roots.get("random"));
      roots.put(root, target);
    }
    if (target == null) throw new IllegalStateException("unknown root " + root);
    while (pos < src.length()) {
      expect('.');
      String name = ident();
      expect('(');
      List<Object> args = new ArrayList<>();
      while (src.charAt(pos) != ')') {
        args.add(literal());
        if (src.charAt(pos) == ',') pos++;
      }
      expect(')');
      target = invoke(target, name, args);
    }
    return target;
  }

  static void expect(char c) {
    if (src.charAt(pos) != c) throw new IllegalStateException("expected " + c + " at " + pos + " in " + src);
    pos++;
  }

  static String ident() {
    int start = pos;
    while (pos < src.length() && Character.isJavaIdentifierPart(src.charAt(pos))) pos++;
    return src.substring(start, pos);
  }

  static Object literal() {
    char c = src.charAt(pos);
    if (c == '"') {
      StringBuilder sb = new StringBuilder();
      pos++;
      while (src.charAt(pos) != '"') {
        char d = src.charAt(pos++);
        if (d == '\\') {
          char e = src.charAt(pos++);
          sb.append(switch (e) { case 'n' -> '\n'; case 't' -> '\t'; default -> e; });
        } else sb.append(d);
      }
      pos++;
      return sb.toString();
    }
    if (c == '\'') {
      char v = src.charAt(pos + 1);
      pos += 3;
      return v;
    }
    if (Character.isUpperCase(c)) {
      String name = ident();
      if (pos < src.length() && src.charAt(pos) == '(') {
        pos++;
        List<Object> args = new ArrayList<>();
        while (src.charAt(pos) != ')') {
          args.add(literal());
          if (src.charAt(pos) == ',') pos++;
        }
        pos++;
        Function<List<Object>, Object> f = FACTORIES.get(name);
        if (f == null) throw new IllegalStateException("unknown factory " + name);
        return f.apply(args);
      }
      expect('.');
      String constant = ident();
      Class<?> e = ENUMS.get(name);
      if (e == null) throw new IllegalStateException("unknown enum " + name);
      for (Object k : e.getEnumConstants()) if (((Enum<?>) k).name().equals(constant)) return k;
      throw new IllegalStateException("unknown constant " + name + "." + constant);
    }
    int start = pos;
    while (pos < src.length() && ",)".indexOf(src.charAt(pos)) < 0) pos++;
    String t = src.substring(start, pos).trim();
    if (t.equals("true") || t.equals("false")) return Boolean.parseBoolean(t);
    if (t.equals("null")) return null;
    if (t.endsWith("L")) return Long.parseLong(t.substring(0, t.length() - 1));
    if (t.contains(".") || t.contains("E")) return Double.parseDouble(t);
    return Integer.parseInt(t);
  }

  static Object invoke(Object target, String name, List<Object> args) throws Exception {
    Method best = null;
    for (Method m : target.getClass().getMethods()) {
      if (!m.getName().equals(name) || m.getParameterCount() != args.size()) continue;
      Class<?>[] ps = m.getParameterTypes();
      boolean ok = true;
      for (int i = 0; i < ps.length && ok; i++) ok = accepts(ps[i], args.get(i));
      if (ok && (best == null || exact(m, args) && !exact(best, args))) best = m;
    }
    if (best == null) throw new NoSuchMethodException(target.getClass().getSimpleName() + "." + name + args);
    best.setAccessible(true);
    Object[] conv = new Object[args.size()];
    Class<?>[] ps = best.getParameterTypes();
    for (int i = 0; i < conv.length; i++) conv[i] = convert(ps[i], args.get(i));
    return best.invoke(target, conv);
  }

  static boolean exact(Method m, List<Object> args) {
    Class<?>[] ps = m.getParameterTypes();
    for (int i = 0; i < ps.length; i++) {
      Object a = args.get(i);
      if (a instanceof Integer && ps[i] != int.class) return false;
      if (a instanceof Long && ps[i] != long.class) return false;
      if (a instanceof Double && ps[i] != double.class) return false;
    }
    return true;
  }

  static boolean accepts(Class<?> p, Object a) {
    if (a == null) return !p.isPrimitive();
    if (a instanceof Integer) return p == int.class || p == long.class || p == Integer.class || p == double.class;
    if (a instanceof Long) return p == long.class || p == Long.class;
    if (a instanceof Double) return p == double.class || p == Double.class || p == float.class;
    if (a instanceof Boolean) return p == boolean.class || p == Boolean.class;
    if (a instanceof Character) return p == char.class || p == Character.class;
    if (a instanceof String) return p == String.class || p == CharSequence.class || p == Object.class;
    return p.isInstance(a);
  }

  static Object convert(Class<?> p, Object a) {
    if (a instanceof Integer i) {
      if (p == long.class) return i.longValue();
      if (p == double.class) return i.doubleValue();
    }
    if (a instanceof Double d && p == float.class) return d.floatValue();
    return a;
  }
}
