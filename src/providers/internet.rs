//! `net.datafaker.providers.base.Internet`.

use unicode_general_category::{GeneralCategory, get_general_category};
use unicode_normalization::UnicodeNormalization;

use super::{ProviderKind, resolve};
use crate::dispatch::{java_str, non_null, opt_str, JValue, Method, P};
use crate::idn::faker_to_ascii;
use crate::java;
use crate::yaml::Value;
use crate::{Error, Faker, Result};

const HTTP_SCHEMES: [&str; 2] = ["http://", "https://"];
const MIN_PORT_NUMBER: i32 = 0;
const MAX_PORT_NUMBER: i32 = 65535;

/// `Internet.UserAgent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAgent {
    /// `AOL`.
    Aol,
    /// `CHROME`.
    Chrome,
    /// `FIREFOX`.
    Firefox,
    /// `INTERNET_EXPLORER`.
    InternetExplorer,
    /// `NETSCAPE`.
    Netscape,
    /// `OPERA`.
    Opera,
    /// `SAFARI`.
    Safari,
}

impl UserAgent {
    /// `values()`.
    pub const VALUES: [UserAgent; 7] = [
        UserAgent::Aol,
        UserAgent::Chrome,
        UserAgent::Firefox,
        UserAgent::InternetExplorer,
        UserAgent::Netscape,
        UserAgent::Opera,
        UserAgent::Safari,
    ];
    const NAMES: &'static [&'static str] = &["AOL", "CHROME", "FIREFOX", "INTERNET_EXPLORER", "NETSCAPE", "OPERA", "SAFARI"];

    /// `toString()`: the browser's key in `internet.user_agent`.
    pub fn browser_name(self) -> &'static str {
        match self {
            UserAgent::Aol => "aol",
            UserAgent::Chrome => "chrome",
            UserAgent::Firefox => "firefox",
            UserAgent::InternetExplorer => "internet_explorer",
            UserAgent::Netscape => "netscape",
            UserAgent::Opera => "opera",
            UserAgent::Safari => "safari",
        }
    }
}

/// `Internet.BotUserAgent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotUserAgent {
    /// `GOOGLEBOT`.
    Googlebot,
    /// `BINGBOT`.
    Bingbot,
    /// `DUCKDUCKBOT`.
    Duckduckbot,
    /// `BAIDUSPIDER`.
    Baiduspider,
    /// `YANDEXBOT`.
    Yandexbot,
}

impl BotUserAgent {
    /// `values()`.
    pub const VALUES: [BotUserAgent; 5] =
        [BotUserAgent::Googlebot, BotUserAgent::Bingbot, BotUserAgent::Duckduckbot, BotUserAgent::Baiduspider, BotUserAgent::Yandexbot];
    const NAMES: &'static [&'static str] = &["GOOGLEBOT", "BINGBOT", "DUCKDUCKBOT", "BAIDUSPIDER", "YANDEXBOT"];

    /// `toString()`: the bot's key in `internet.bot_user_agent`.
    pub fn browser_name(self) -> &'static str {
        match self {
            BotUserAgent::Googlebot => "googlebot",
            BotUserAgent::Bingbot => "bingbot",
            BotUserAgent::Duckduckbot => "duckduckbot",
            BotUserAgent::Baiduspider => "baiduspider",
            BotUserAgent::Yandexbot => "yandexbot",
        }
    }
}

/// `Internet.PortRange`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortRange {
    /// Ports 0 to 1023.
    WellKnownPorts,
    /// Ports 1024 to 49151.
    RegisteredPorts,
    /// Ports 49152 to 65535.
    DynamicPrivatePorts,
}

impl PortRange {
    const NAMES: &'static [&'static str] = &["WellKnownPorts", "RegisteredPorts", "DynamicPrivatePorts"];

    fn bounds(self) -> (i32, i32) {
        match self {
            PortRange::WellKnownPorts => (0, 1023),
            PortRange::RegisteredPorts => (1024, 49151),
            PortRange::DynamicPrivatePorts => (49152, 65535),
        }
    }
}

/// E-mail addresses, domains, URLs, IP and MAC addresses, user agents, UUIDs.
#[derive(Clone, Copy)]
pub struct Internet<'a> {
    faker: &'a Faker,
}

impl<'a> Internet<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Internet { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Internet, key)
    }

    fn lower(&self, s: &str) -> String {
        java::string_to_lower(s, self.faker.locale().language())
    }

    /// Deprecated upstream: `credentials().username()`.
    pub fn username(&self) -> Result<String> {
        self.faker.credentials().username()
    }

    /// `emailAddress()`: `first.last@free-mail-domain`.
    pub fn email_address(&self) -> Result<String> {
        let name = self.faker.credentials().username()?;
        self.email_address_for(&name)
    }

    /// `emailAddress(name)`.
    pub fn email_address_for(&self, name: &str) -> Result<String> {
        let local = self.to_local_part(name)?;
        let domain = faker_to_ascii(&self.faker.resolve("internet.free_email")?)?;
        Ok(format!("{}@{domain}", strip_accents(&local)))
    }

    /// `safeEmailAddress()`: on an `example.*` domain.
    pub fn safe_email_address(&self) -> Result<String> {
        let name = self.faker.credentials().username()?;
        self.safe_email_address_for(&name)
    }

    /// `safeEmailAddress(name)`: an address at a safe domain with a local part made from `name`.
    pub fn safe_email_address_for(&self, name: &str) -> Result<String> {
        let local = self.to_local_part(name)?;
        let domain = faker_to_ascii(&self.faker.resolve("internet.safe_email")?)?;
        Ok(format!("{}@{domain}", strip_accents(&local)))
    }

    /// `internet.email_subject`.
    pub fn email_subject(&self) -> Result<String> {
        self.resolve("internet.email_subject")
    }

    fn string_list(&self, key: &str) -> Vec<String> {
        match self.faker.fake_values_service().fetch_object(key) {
            Some(Value::List(l)) if l.iter().all(|v| matches!(v, Value::Str(_))) => {
                l.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()
            }
            _ => Vec::new(),
        }
    }

    /// `toLocalPart(name)`: the first and last words without a name prefix or suffix, lower-cased, keeping
    /// `[a-z0-9.]` only, joined with a dot.
    fn to_local_part(self, name: &str) -> Result<String> {
        let stripped = strip_accents(name);
        let mut parts: Vec<&str> = stripped.split(' ').collect();
        while parts.len() > 1 && parts.last() == Some(&"") {
            parts.pop();
        }
        if parts.len() == 1 && parts[0].is_empty() && !stripped.is_empty() {
            parts.clear();
        }
        let prefixes = self.string_list("name.prefix");
        let first = parts.first().ok_or_else(|| Error::new("ArrayIndexOutOfBoundsException", "Index 0 out of bounds for length 0"))?;
        if prefixes.iter().any(|p| p == first) {
            parts.remove(0);
        }
        let suffixes = self.string_list("name.suffix");
        let last = parts.last().ok_or_else(|| Error::new("ArrayIndexOutOfBoundsException", "Index -1 out of bounds for length 0"))?;
        if suffixes.iter().any(|s| s == last) {
            parts.pop();
        }
        let clean = |s: &str| self.lower(s).chars().filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '.').collect::<String>();
        Ok(match parts.len() {
            0 => clean(name),
            1 => clean(parts[0]),
            n => format!("{}.{}", clean(parts[0]), clean(parts[n - 1])),
        })
    }

    /// `domainName()`: a domain word and a suffix.
    pub fn domain_name(&self) -> Result<String> {
        let word = self.domain_word()?;
        Ok(format!("{word}.{}", self.domain_suffix()?))
    }

    /// `domainWord()`: a last name, lower-cased, apostrophes dropped, IDN-encoded.
    pub fn domain_word(&self) -> Result<String> {
        let last = self.faker.name().last_name()?;
        faker_to_ascii(&self.lower(&last).replace('\'', ""))
    }

    /// `internet.domain_suffix`.
    pub fn domain_suffix(&self) -> Result<String> {
        self.resolve("internet.domain_suffix")
    }

    /// `url()`: each part present or not by the parity of six random bytes.
    pub fn url(&self) -> Result<String> {
        let b = self.faker.random().next_random_bytes(6);
        self.url_with(b[0] % 2 == 0, b[1] % 2 == 0, b[2] % 2 == 0, b[3] % 2 == 0, b[4] % 2 == 0, b[5] % 2 == 0)
    }

    /// `url(schemeChoice, portChoice, pathChoice, fileChoice, paramsChoice, anchorChoice)`.
    pub fn url_with(
        &self,
        scheme_choice: bool,
        port_choice: bool,
        path_choice: bool,
        file_choice: bool,
        params_choice: bool,
        anchor_choice: bool,
    ) -> Result<String> {
        let lorem = self.faker.lorem();
        let scheme = if scheme_choice { self.faker.options().next_element(&HTTP_SCHEMES)? } else { "https://" };
        let port = if port_choice { format!(":{}", self.port()) } else { String::new() };
        let path = if path_choice { format!("/{}", slug(&lorem.words(2)?, "/")) } else { "/".into() };
        let file = if file_choice { lorem.words(1)?.remove(0) } else { String::new() };
        let params = if params_choice {
            let a = slug(&lorem.words(2)?, "=");
            format!("?{a}&{}", slug(&lorem.words(2)?, "="))
        } else {
            String::new()
        };
        let anchor = if anchor_choice { format!("#{}", lorem.words(1)?.remove(0)) } else { String::new() };
        Ok(format!("{scheme}{}{port}{path}{file}{params}{anchor}", self.webdomain()?))
    }

    /// `webdomain()`: `www.first-last.suffix`.
    pub fn webdomain(&self) -> Result<String> {
        let first = self.lower(&self.faker.name().first_name()?).replace('\'', "");
        let word = self.domain_word()?;
        let host = faker_to_ascii(&format!("{first}-{word}"))?;
        Ok(format!("www.{host}.{}", self.domain_suffix()?))
    }

    /// `image()`: a picsum.photos URL with a dimension from `internet.image_dimension`.
    pub fn image(&self) -> Result<String> {
        let dimension = self.resolve("internet.image_dimension")?;
        let parts: Vec<&str> = dimension.split('x').collect();
        let parse = |s: &str| s.trim().parse::<i32>().map_err(|_| Error::new("NumberFormatException", format!("For input string: \"{}\"", s.trim())));
        Ok(self.image_of(parse(parts[0])?, parse(parts.get(1).copied().unwrap_or(""))?))
    }

    /// `image(width, height)`: a picsum.photos URL of that size.
    pub fn image_of(&self, width: i32, height: i32) -> String {
        format!("https://picsum.photos/{width}/{height}")
    }

    /// `image(width, height, seed)`: a picsum.photos URL of that size, the same for the same seed.
    pub fn image_with_seed(&self, width: i32, height: i32, seed: &str) -> String {
        format!("https://picsum.photos/seed/{seed}/{width}/{height}")
    }

    /// `internet.http_method`.
    pub fn http_method(&self) -> Result<String> {
        self.resolve("internet.http_method")
    }

    /// Deprecated upstream: `password(8, 16)`, lower-case letters and at least one digit.
    pub fn password(&self) -> Result<String> {
        self.password_with(8, 16, false, false, true)
    }

    /// Deprecated upstream: `text().text(minimumLength, maximumLength, includeUppercase, includeSpecial,
    /// includeDigit)`.
    pub fn password_with(
        &self,
        minimum_length: i32,
        maximum_length: i32,
        include_uppercase: bool,
        include_special: bool,
        include_digit: bool,
    ) -> Result<String> {
        self.faker.text().text_with(minimum_length, maximum_length, include_uppercase, include_special, include_digit)
    }

    /// `port()`: 0 to 65535.
    pub fn port(&self) -> i32 {
        self.port_between(MIN_PORT_NUMBER, MAX_PORT_NUMBER).expect("valid range")
    }

    /// `port(range)`: a port within `range`.
    pub fn port_in(&self, range: PortRange) -> Result<i32> {
        let (from, to) = range.bounds();
        self.port_between(from, to)
    }

    /// `port(from, to)`, both inclusive.
    pub fn port_between(&self, from: i32, to: i32) -> Result<i32> {
        if from < MIN_PORT_NUMBER {
            return Err(Error::illegal_argument(format!("Port number {from} cannot be less than {MIN_PORT_NUMBER}")));
        }
        if to > MAX_PORT_NUMBER {
            return Err(Error::illegal_argument(format!("Port number {to} cannot be greater than {MAX_PORT_NUMBER}")));
        }
        self.faker.random().next_int_between(from, to)
    }

    /// `macAddress(prefix)`: the prefix, completed to six hex pairs.
    pub fn mac_address_with(&self, prefix: &str) -> String {
        let prefix_length = if prefix.trim().is_empty() {
            0
        } else {
            let mut parts: Vec<&str> = prefix.split(':').collect();
            while parts.last() == Some(&"") {
                parts.pop();
            }
            parts.len()
        };
        let mut out = prefix.to_string();
        let random = self.faker.random();
        for _ in 0..6usize.saturating_sub(prefix_length) {
            if !out.is_empty() {
                out.push(':');
            }
            out.push_str(&format!("{:x}", random.next_int_bounded(16).expect("16 > 0")));
            out.push_str(&format!("{:x}", random.next_int_bounded(16).expect("16 > 0")));
        }
        out
    }

    /// `macAddress()`.
    pub fn mac_address(&self) -> String {
        self.mac_address_with("")
    }

    /// `ipV4Address()`: four octets from 2 to 255.
    pub fn ip_v4_address(&self) -> String {
        let r = self.faker.random();
        let octet = || (r.next_int_bounded(254).expect("254 > 0") + 2) as u8;
        let (a, b, c, d) = (octet(), octet(), octet(), octet());
        format!("{a}.{b}.{c}.{d}")
    }

    /// `privateIpV4Address()`.
    pub fn private_ip_v4_address(&self) -> String {
        const FIRST: [u8; 5] = [10, 127, 169, 192, 172];
        const SECOND_172: [u8; 16] = [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
        let r = self.faker.random();
        let first = FIRST[r.next_int_bounded(5).expect("5 > 0") as usize];
        let mut second = r.next_int_bounded(256).expect("256 > 0") as u8;
        let third = r.next_int_bounded(256).expect("256 > 0") as u8;
        let fourth = r.next_int_bounded(256).expect("256 > 0") as u8;
        match first {
            172 => second = SECOND_172[r.next_int_bounded(16).expect("16 > 0") as usize],
            192 => second = 168,
            169 => second = 254,
            _ => {}
        }
        format!("{first}.{second}.{third}.{fourth}")
    }

    /// `publicIpV4Address()`: redraws the first octet while upstream's `Arrays.binarySearch` over its unsorted
    /// list of private first octets finds it at an index above 0.
    pub fn public_ip_v4_address(&self) -> String {
        const PRIVATE_FIRST: [i8; 5] = [10, 127, -87, -64, -84];
        let r = self.faker.random();
        let draw = || r.next_int_bounded(256).expect("256 > 0") as u8 as i8;
        let mut first = draw();
        let (second, third, fourth) = (draw(), draw(), draw());
        while binary_search(&PRIVATE_FIRST, first) > 0 {
            first = draw();
        }
        format!("{}.{}.{}.{}", first as u8, second as u8, third as u8, fourth as u8)
    }

    /// `ipV4Cidr()`: an IPv4 address with a prefix length from 1 to 31.
    pub fn ip_v4_cidr(&self) -> String {
        let ip = self.ip_v4_address();
        format!("{ip}/{}", self.faker.random().next_int_bounded(31).expect("31 > 0") + 1)
    }

    /// `ipV6Address()`: eight groups of four random hex digits, printed as `Inet6Address.getHostAddress()` does
    /// (leading zeros dropped, no `::` compression).
    pub fn ip_v6_address(&self) -> String {
        let random = self.faker.random();
        let groups: Vec<String> = (0..8)
            .map(|_| {
                let hex = random.hex_with(4, false);
                format!("{:x}", u16::from_str_radix(&hex, 16).expect("hex"))
            })
            .collect();
        groups.join(":")
    }

    /// `ipV6Cidr()`: an IPv6 address with a prefix length from 1 to 127.
    pub fn ip_v6_cidr(&self) -> String {
        let ip = self.ip_v6_address();
        format!("{ip}/{}", self.faker.random().next_int_bounded(127).expect("127 > 0") + 1)
    }

    /// `slug()`: two lorem words joined with `_`.
    pub fn slug(&self) -> Result<String> {
        Ok(slug(&self.faker.lorem().words(2)?, "_"))
    }

    /// `slug(words, glue)`: `None` stands for two lorem words and `_`.
    pub fn slug_with(&self, words: Option<&[String]>, glue: Option<&str>) -> Result<String> {
        let glue = glue.unwrap_or("_");
        match words {
            Some(w) => Ok(slug(w, glue)),
            None => Ok(slug(&self.faker.lorem().words(2)?, glue)),
        }
    }

    /// `uuidv3()`: `UUID.nameUUIDFromBytes` of 16 random bytes.
    pub fn uuidv3(&self) -> String {
        let bytes: Vec<u8> = self.faker.random().next_random_bytes(16).iter().map(|b| *b as u8).collect();
        let mut md5 = crate::md5::digest(&bytes);
        md5[6] &= 0x0f;
        md5[6] |= 0x30;
        md5[8] &= 0x3f;
        md5[8] |= 0x80;
        let msb = u64::from_be_bytes(md5[..8].try_into().expect("8 bytes"));
        let lsb = u64::from_be_bytes(md5[8..].try_into().expect("8 bytes"));
        uuid_string(msb, lsb)
    }

    /// `uuidv4()`: the same as [`Self::uuid`].
    pub fn uuidv4(&self) -> String {
        self.uuid()
    }

    /// `uuidv7()`.
    pub fn uuidv7(&self) -> String {
        let r = self.faker.random();
        let timestamp = r.next_long() as u64;
        let bits1 = r.next_long() as u64;
        let bits2 = r.next_long() as u64;
        let msb = (timestamp & !0xF000) | 0x7000;
        let lsb = ((bits1 ^ bits2) & !0xC000_0000_0000_0000) | 0x8000_0000_0000_0000;
        uuid_string(msb, lsb)
    }

    /// `uuid()`: `uuidv3()` with the version digit set to 4.
    pub fn uuid(&self) -> String {
        let v3 = self.uuidv3();
        format!("{}4{}", &v3[..14], &v3[15..])
    }

    /// `userAgent(userAgent)`: `None` picks a browser with `nextDouble()`.
    pub fn user_agent_of(&self, user_agent: Option<UserAgent>) -> Result<String> {
        let agent = match user_agent {
            Some(a) => a,
            None => UserAgent::VALUES[(self.faker.random().next_double() * 7.0) as usize],
        };
        self.resolve(&format!("internet.user_agent.{}", agent.browser_name()))
    }

    /// `userAgent()`: a user agent of a random browser.
    pub fn user_agent(&self) -> Result<String> {
        self.user_agent_of(None)
    }

    /// `botUserAgent(vendor)`.
    pub fn bot_user_agent(&self, vendor: Option<BotUserAgent>) -> Result<String> {
        let agent = match vendor {
            Some(a) => a,
            None => BotUserAgent::VALUES[(self.faker.random().next_double() * 5.0) as usize],
        };
        self.resolve(&format!("internet.bot_user_agent.{}", agent.browser_name()))
    }

    /// `botUserAgentAny()`: a user agent of a random bot.
    pub fn bot_user_agent_any(&self) -> Result<String> {
        self.bot_user_agent(None)
    }
}

/// `Internet.slug(words, glue)` without its defaults.
fn slug(words: &[String], glue: &str) -> String {
    words.join(glue)
}

pub(crate) fn uuid_string(msb: u64, lsb: u64) -> String {
    let h = format!("{msb:016x}{lsb:016x}");
    format!("{}-{}-{}-{}-{}", &h[..8], &h[8..12], &h[12..16], &h[16..20], &h[20..])
}

/// `Arrays.binarySearch(byte[], key)`, which assumes a sorted array.
fn binary_search(a: &[i8], key: i8) -> i32 {
    let mut low: i32 = 0;
    let mut high: i32 = a.len() as i32 - 1;
    while low <= high {
        let mid = ((low + high) as u32 >> 1) as i32;
        let v = a[mid as usize];
        if v < key {
            low = mid + 1;
        } else if v > key {
            high = mid - 1;
        } else {
            return mid;
        }
    }
    -(low + 1)
}

/// `Internet.stripAccents`: NFD, then runs of combining diacritical marks (U+0300..U+036F), modifier letters and
/// modifier symbols removed.
pub(crate) fn strip_accents(input: &str) -> String {
    input
        .nfd()
        .filter(|c| {
            !(('\u{0300}'..='\u{036F}').contains(c)
                || matches!(get_general_category(*c), GeneralCategory::ModifierLetter | GeneralCategory::ModifierSymbol))
        })
        .collect()
}

fn i(f: &Faker) -> Internet<'_> {
    f.internet()
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

fn ok(v: String) -> Result<JValue> {
    Ok(JValue::Str(v))
}

fn user_agent_arg(v: &JValue) -> Option<UserAgent> {
    match v {
        JValue::Enum(name) => UserAgent::NAMES.iter().position(|n| n == name).map(|i| UserAgent::VALUES[i]),
        _ => None,
    }
}

fn bot_arg(v: &JValue) -> Option<BotUserAgent> {
    match v {
        JValue::Enum(name) => BotUserAgent::NAMES.iter().position(|n| n == name).map(|i| BotUserAgent::VALUES[i]),
        _ => None,
    }
}

fn port_range_arg(v: &JValue) -> PortRange {
    match v {
        JValue::Enum("RegisteredPorts") => PortRange::RegisteredPorts,
        JValue::Enum("DynamicPrivatePorts") => PortRange::DynamicPrivatePorts,
        _ => PortRange::WellKnownPorts,
    }
}

const USER_AGENT: P = P::Enum("UserAgent", UserAgent::NAMES);
const BOT_USER_AGENT: P = P::Enum("BotUserAgent", BotUserAgent::NAMES);
const PORT_RANGE: P = P::Enum("PortRange", PortRange::NAMES);

pub(crate) static METHODS: &[Method] = &[
    Method::new("username", &[], |f, _| s(i(f).username())),
    Method::new("emailAddress", &[], |f, _| s(i(f).email_address())),
    // `toLocalPart(name)` starts with `StringUtils.stripAccents(name)`.
    Method::new("emailAddress", &[P::String], |f, a| {
        s(i(f).email_address_for(non_null(a, 0, "Cannot invoke \"java.lang.CharSequence.toString()\" because \"<parameter1>\" is null")?.as_str()))
    })
    .accepting_null(),
    Method::new("safeEmailAddress", &[], |f, _| s(i(f).safe_email_address())),
    Method::new("safeEmailAddress", &[P::String], |f, a| {
        s(i(f).safe_email_address_for(non_null(a, 0, "Cannot invoke \"java.lang.CharSequence.toString()\" because \"<parameter1>\" is null")?.as_str()))
    })
    .accepting_null(),
    Method::new("emailSubject", &[], |f, _| s(i(f).email_subject())),
    Method::new("domainName", &[], |f, _| s(i(f).domain_name())),
    Method::new("domainWord", &[], |f, _| s(i(f).domain_word())),
    Method::new("domainSuffix", &[], |f, _| s(i(f).domain_suffix())),
    Method::new("url", &[], |f, _| s(i(f).url())),
    Method::new("url", &[P::Boolean, P::Boolean, P::Boolean, P::Boolean, P::Boolean, P::Boolean], |f, a| {
        s(i(f).url_with(a[0].as_bool(), a[1].as_bool(), a[2].as_bool(), a[3].as_bool(), a[4].as_bool(), a[5].as_bool()))
    }),
    Method::new("webdomain", &[], |f, _| s(i(f).webdomain())),
    Method::new("image", &[], |f, _| s(i(f).image())),
    Method::new("image", &[P::Int, P::Int], |f, a| ok(i(f).image_of(a[0].as_int(), a[1].as_int()))),
    Method::new("image", &[P::Int, P::Int, P::String], |f, a| ok(i(f).image_with_seed(a[0].as_int(), a[1].as_int(), java_str(&a[2])))).accepting_null(),
    Method::new("httpMethod", &[], |f, _| s(i(f).http_method())),
    Method::new("password", &[], |f, _| s(i(f).password())),
    Method::new("password", &[P::Boolean], |f, a| s(i(f).password_with(8, 16, false, false, a[0].as_bool()))),
    Method::new("password", &[P::Int, P::Int], |f, a| s(i(f).password_with(a[0].as_int(), a[1].as_int(), false, false, true))),
    Method::new("password", &[P::Int, P::Int, P::Boolean], |f, a| {
        s(i(f).password_with(a[0].as_int(), a[1].as_int(), a[2].as_bool(), false, true))
    }),
    Method::new("password", &[P::Int, P::Int, P::Boolean, P::Boolean], |f, a| {
        s(i(f).password_with(a[0].as_int(), a[1].as_int(), a[2].as_bool(), a[3].as_bool(), true))
    }),
    Method::new("password", &[P::Int, P::Int, P::Boolean, P::Boolean, P::Boolean], |f, a| {
        s(i(f).password_with(a[0].as_int(), a[1].as_int(), a[2].as_bool(), a[3].as_bool(), a[4].as_bool()))
    }),
    Method::new("port", &[], |f, _| Ok(JValue::Int(i(f).port()))),
    Method::new("port", &[PORT_RANGE], |f, a| {
        non_null(a, 0, "Cannot read field \"from\" because \"range\" is null")?;
        Ok(JValue::Int(i(f).port_in(port_range_arg(&a[0]))?))
    })
    .accepting_null(),
    Method::new("port", &[P::Int, P::Int], |f, a| Ok(JValue::Int(i(f).port_between(a[0].as_int(), a[1].as_int())?))),
    // A null prefix is an empty one.
    Method::new("macAddress", &[P::String], |f, a| ok(i(f).mac_address_with(a[0].as_str()))).accepting_null(),
    Method::new("macAddress", &[], |f, _| ok(i(f).mac_address())),
    Method::new("ipV4Address", &[], |f, _| ok(i(f).ip_v4_address())),
    Method::new("getIpV4Address", &[], |f, _| ok(format!("/{}", i(f).ip_v4_address()))),
    Method::new("privateIpV4Address", &[], |f, _| ok(i(f).private_ip_v4_address())),
    Method::new("getPrivateIpV4Address", &[], |f, _| ok(format!("/{}", i(f).private_ip_v4_address()))),
    Method::new("publicIpV4Address", &[], |f, _| ok(i(f).public_ip_v4_address())),
    Method::new("getPublicIpV4Address", &[], |f, _| ok(format!("/{}", i(f).public_ip_v4_address()))),
    Method::new("ipV4Cidr", &[], |f, _| ok(i(f).ip_v4_cidr())),
    Method::new("ipV6Address", &[], |f, _| ok(i(f).ip_v6_address())),
    Method::new("getIpV6Address", &[], |f, _| ok(format!("/{}", i(f).ip_v6_address()))),
    Method::new("ipV6Cidr", &[], |f, _| ok(i(f).ip_v6_cidr())),
    Method::new("slug", &[], |f, _| s(i(f).slug())),
    // A null list is two lorem words; a null glue is `_`.
    Method::new("slug", &[P::Opaque("List"), P::String], |f, a| match &a[0] {
        JValue::Null => s(i(f).slug_with(None, opt_str(&a[1]))),
        words => s(i(f).slug_with(Some(&crate::dispatch::java_strings(words)?), opt_str(&a[1]))),
    })
    .accepting_null(),
    Method::new("uuidv3", &[], |f, _| ok(i(f).uuidv3())),
    Method::new("uuidv4", &[], |f, _| ok(i(f).uuidv4())),
    Method::new("uuidv7", &[], |f, _| ok(i(f).uuidv7())),
    Method::new("uuid", &[], |f, _| ok(i(f).uuid())),
    // A null agent is a random one.
    Method::new("userAgent", &[USER_AGENT], |f, a| s(i(f).user_agent_of(user_agent_arg(&a[0])))).accepting_null(),
    Method::new("userAgent", &[], |f, _| s(i(f).user_agent())),
    Method::new("botUserAgent", &[BOT_USER_AGENT], |f, a| s(i(f).bot_user_agent(bot_arg(&a[0])))).accepting_null(),
    Method::new("botUserAgentAny", &[], |f, _| s(i(f).bot_user_agent_any())),
];
