// Writes src/java/time/tzdb.bin (run by scripts/tz-tables.sh): the running JDK's time-zone rules and their en_US
// names, in a format of this crate's own (read by src/java/time/zone.rs; all numbers big-endian, a string is a u16
// length and UTF-8 bytes, a string reference is a u16 index into the string table, 0xFFFF for null):
//
//   "DFTZ" u8 1, string tzdb version
//   u16 rule count; each ZoneRules: u32 n, n x i64 standardTransitions, n+1 x i32 standardOffsets; u32 m,
//       m x i64 savingsInstantTransitions, m+1 x i32 wallOffsets; u8 k, k last rules (ZoneOffsetTransitionRule: u8
//       month, i8 dayOfMonthIndicator, u8 dayOfWeek or 0, i32 second of day, u8 midnightEndOfDay, u8 timeDefinition
//       UTC 0 WALL 1 STANDARD 2, i32 standard, before and after offsets)
//   u16 string count, the strings
//   u16 region count; each ZoneId.getAvailableZoneIds() (sorted): ref id, u16 rule, 6 refs of the names
//       DateTimeFormatter prints (TimeZoneNameUtility.retrieveDisplayNames long and short standard and daylight, and
//       retrieveGenericDisplayName long and short), 4 refs of TimeZone.getDisplayName(daylight, style, Locale.US)
//       (standard long, short, daylight long, short)
//   the parse tree of ZoneTextPrinterParser as getTree builds it for en_US: u16 row count, each
//       TimeZoneNameUtility.getZoneStrings row whose id is a region: ref id, ref ZoneName.toZid(id, locale), 6 refs
//       names[1..6]; u16 count of the other region ids (getAvailableZoneIds order, without Etc and GMT ones), each:
//       ref id, 6 refs retrieveDisplayNames(id)[1..6]
import java.io.*;
import java.lang.reflect.*;
import java.nio.charset.StandardCharsets;
import java.time.*;
import java.time.zone.*;
import java.util.*;

public class TzTables {
  static final List<String> STRINGS = new ArrayList<>();
  static final Map<String, Integer> INDEX = new HashMap<>();

  static int ref(String s) {
    if (s == null) return 0xFFFF;
    return INDEX.computeIfAbsent(s, k -> { STRINGS.add(k); return STRINGS.size() - 1; });
  }

  static void str(DataOutputStream out, String s) throws IOException {
    byte[] b = s.getBytes(StandardCharsets.UTF_8);
    out.writeShort(b.length);
    out.write(b);
  }

  public static void main(String[] a) throws Exception {
    Locale locale = Locale.US;
    Class<?> util = Class.forName("sun.util.locale.provider.TimeZoneNameUtility");
    Method displayNames = util.getMethod("retrieveDisplayNames", String.class, Locale.class);
    Method genericName = util.getMethod("retrieveGenericDisplayName", String.class, int.class, Locale.class);
    Method zoneStrings = util.getMethod("getZoneStrings", Locale.class);
    Method toZid = Class.forName("java.time.format.ZoneName").getDeclaredMethod("toZid", String.class, Locale.class);
    toZid.setAccessible(true);
    Field[] f = new Field[5];
    String[] names = {"standardTransitions", "standardOffsets", "savingsInstantTransitions", "wallOffsets", "lastRules"};
    for (int i = 0; i < 5; i++) {
      f[i] = ZoneRules.class.getDeclaredField(names[i]);
      f[i].setAccessible(true);
    }

    Set<String> available = ZoneId.getAvailableZoneIds();
    List<String> ids = new ArrayList<>(new TreeSet<>(available));
    List<ZoneRules> rules = new ArrayList<>();
    Map<ZoneRules, Integer> ruleIndex = new HashMap<>();
    ByteArrayOutputStream regionBytes = new ByteArrayOutputStream();
    DataOutputStream regions = new DataOutputStream(regionBytes);
    regions.writeShort(ids.size());
    for (String id : ids) {
      ZoneRules r = ZoneId.of(id).getRules();
      int index = ruleIndex.computeIfAbsent(r, k -> { rules.add(k); return rules.size() - 1; });
      regions.writeShort(ref(id));
      regions.writeShort(index);
      String[] dn = (String[]) displayNames.invoke(null, id, locale);
      for (int i = 1; i <= 4; i++) regions.writeShort(ref(dn == null ? null : dn[i]));
      regions.writeShort(ref((String) genericName.invoke(null, id, TimeZone.LONG, locale)));
      regions.writeShort(ref((String) genericName.invoke(null, id, TimeZone.SHORT, locale)));
      TimeZone tz = TimeZone.getTimeZone(id);
      regions.writeShort(ref(tz.getDisplayName(false, TimeZone.LONG, locale)));
      regions.writeShort(ref(tz.getDisplayName(false, TimeZone.SHORT, locale)));
      regions.writeShort(ref(tz.getDisplayName(true, TimeZone.LONG, locale)));
      regions.writeShort(ref(tz.getDisplayName(true, TimeZone.SHORT, locale)));
    }
    // ZoneTextPrinterParser.getTree
    String[][] rows = (String[][]) zoneStrings.invoke(null, locale);
    Set<String> regionIds = new HashSet<>(available);
    List<String[]> treeRows = new ArrayList<>();
    for (String[] row : rows) {
      if (!regionIds.remove(row[0])) continue;
      treeRows.add(row);
    }
    regions.writeShort(treeRows.size());
    for (String[] row : treeRows) {
      regions.writeShort(ref(row[0]));
      regions.writeShort(ref((String) toZid.invoke(null, row[0], locale)));
      for (int i = 1; i <= 6; i++) regions.writeShort(ref(i < row.length ? row[i] : null));
    }
    List<String> custom = regionIds.stream().filter(z -> !z.startsWith("Etc") && !z.startsWith("GMT")).toList();
    regions.writeShort(custom.size());
    for (String cid : custom) {
      String[] cn = (String[]) displayNames.invoke(null, cid, locale);
      regions.writeShort(ref(cid));
      for (int i = 1; i <= 6; i++) regions.writeShort(ref(i < cn.length ? cn[i] : null));
    }

    DataOutputStream out = new DataOutputStream(new BufferedOutputStream(new FileOutputStream(a[0])));
    out.writeBytes("DFTZ");
    out.writeByte(1);
    str(out, ZoneRulesProvider.getVersions("UTC").firstKey());
    out.writeShort(rules.size());
    for (ZoneRules r : rules) {
      long[] st = (long[]) f[0].get(r);
      ZoneOffset[] so = (ZoneOffset[]) f[1].get(r);
      long[] sv = (long[]) f[2].get(r);
      ZoneOffset[] wo = (ZoneOffset[]) f[3].get(r);
      ZoneOffsetTransitionRule[] lr = (ZoneOffsetTransitionRule[]) f[4].get(r);
      out.writeInt(st.length);
      for (long t : st) out.writeLong(t);
      for (ZoneOffset o : so) out.writeInt(o.getTotalSeconds());
      out.writeInt(sv.length);
      for (long t : sv) out.writeLong(t);
      for (ZoneOffset o : wo) out.writeInt(o.getTotalSeconds());
      out.writeByte(lr.length);
      for (ZoneOffsetTransitionRule t : lr) {
        out.writeByte(t.getMonth().getValue());
        out.writeByte(t.getDayOfMonthIndicator());
        out.writeByte(t.getDayOfWeek() == null ? 0 : t.getDayOfWeek().getValue());
        out.writeInt(t.isMidnightEndOfDay() ? 86400 : t.getLocalTime().toSecondOfDay());
        out.writeByte(t.isMidnightEndOfDay() ? 1 : 0);
        out.writeByte(t.getTimeDefinition().ordinal());
        out.writeInt(t.getStandardOffset().getTotalSeconds());
        out.writeInt(t.getOffsetBefore().getTotalSeconds());
        out.writeInt(t.getOffsetAfter().getTotalSeconds());
      }
    }
    out.writeShort(STRINGS.size());
    for (String s : STRINGS) str(out, s);
    out.write(regionBytes.toByteArray());
    out.close();
  }
}
