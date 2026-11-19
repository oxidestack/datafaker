// Records differential vectors of `Image` and `Fingerprint` from Datafaker 2.7.0 on the recording JDK's
// `javax.imageio` encoders. Reads `seed TAB case` lines from stdin and prints each with the case's result appended
// (see tests/vectors_images.rs, which builds the same cases). Each line gets a fresh `new Faker(new Random(seed))`.
// A result of at most 2000 characters prints as is; a longer one as `#<length> <SHA-256 of its UTF-8 bytes>` (byte
// arrays print as their Base64 first). A thrown exception prints as `ERR <simple class name>: <message>`.
// `#` lines and blank lines pass through. Run by `scripts/record.sh tests/vectors/images.tsv Images`.
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.util.*;
import java.util.function.Function;
import net.datafaker.Faker;
import net.datafaker.providers.base.Fingerprint.PatternType;
import net.datafaker.providers.base.Image;
import net.datafaker.providers.base.Image.ImageType;

public class Images {
  public static void main(String[] a) throws Exception {
    BufferedReader in = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
    PrintStream out = new PrintStream(new FileOutputStream(FileDescriptor.out), true, StandardCharsets.UTF_8);
    String line;
    while ((line = in.readLine()) != null) {
      if (line.startsWith("#") || line.isBlank()) { out.println(line); continue; }
      String[] cols = line.split("\t");
      long seed = Long.parseLong(cols[0]);
      String result;
      try {
        Faker faker = new Faker(new Random(seed));
        Object r = run(cols[1], faker);
        // What the faker's random draws next shows how many draws the case made.
        result = show(r) + " | next " + faker.random().nextInt();
      } catch (Throwable e) {
        result = "ERR " + e.getClass().getSimpleName() + ": " + e.getMessage();
      }
      out.println(cols[0] + '\t' + cols[1] + '\t' + result);
    }
  }

  static String show(Object o) throws Exception {
    String s = o instanceof byte[] b ? Base64.getEncoder().encodeToString(b) : String.valueOf(o);
    if (s.length() <= 2000) return s;
    byte[] d = MessageDigest.getInstance("SHA-256").digest(s.getBytes(StandardCharsets.UTF_8));
    StringBuilder hex = new StringBuilder();
    for (byte b : d) hex.append(String.format("%02x", b));
    return "#" + s.length() + " " + hex;
  }

  // A case: `image.base64PNG`, ... (no-argument methods), `image.<TYPE>.<width>x<height>` (`base64(new
  // Base64ImageRuleConfig(type, width, height))`), `builder.<TYPE|null>.<width>.<height>` (the `ImageBuilder` with
  // those settings, `-` keeping a default), `fingerprint.png`, `fingerprint.base64`, `fingerprint.<PATTERN>.<width>x<height>`
  // (`base64(width, height, pattern)`).
  static Object run(String c, Faker faker) {
    String[] p = c.split("\\.");
    switch (p[0]) {
      case "image":
        if (p.length == 2) {
          return switch (p[1]) {
            case "base64BMP" -> faker.image().base64BMP();
            case "base64GIF" -> faker.image().base64GIF();
            case "base64JPG" -> faker.image().base64JPG();
            case "base64JPEG" -> faker.image().base64JPEG();
            case "base64PNG" -> faker.image().base64PNG();
            case "base64SVG" -> faker.image().base64SVG();
            case "base64TIFF" -> faker.image().base64TIFF();
            default -> throw new IllegalStateException("unknown case " + c);
          };
        }
        String[] wh = p[2].split("x");
        return faker.image().base64(new Image.Base64ImageRuleConfig(ImageType.valueOf(p[1]), Integer.parseInt(wh[0]), Integer.parseInt(wh[1])));
      case "builder": {
        Image.ImageBuilder b = Image.ImageBuilder.builder();
        if (!p[1].equals("-")) b.type(p[1].equals("null") ? null : ImageType.valueOf(p[1]));
        if (!p[2].equals("-")) b.width(Integer.parseInt(p[2]));
        if (!p[3].equals("-")) b.height(Integer.parseInt(p[3]));
        return faker.image().base64(b.build());
      }
      case "fingerprint":
        if (p[1].equals("png")) return faker.fingerprint().png();
        if (p[1].equals("base64")) return faker.fingerprint().base64();
        String[] fwh = p[2].split("x");
        return faker.fingerprint().base64(Integer.parseInt(fwh[0]), Integer.parseInt(fwh[1]), PatternType.valueOf(p[1]));
      default:
        throw new IllegalStateException("unknown case " + c);
    }
  }
}
