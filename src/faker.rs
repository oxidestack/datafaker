//! `net.datafaker.Faker` (with `BaseFaker` and `FakerContext`).

use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex, RwLock};

use crate::dispatch::{AnyObject, JValue, JavaClass, Method, P};
use crate::expr::Resolver;
use crate::locale::Locale;
use crate::random_service::RandomService;
use crate::service::FakeValuesService;
use crate::values::{FakeValues, FakeValuesContext, FakeValuesGrouping, FakeValuesInterface};
use crate::{Error, Result};

/// A faker: a locale (with its fallback chain) and a [`RandomService`].
///
/// `Faker::with_seed(seed)` is `new Faker(new Random(seed))`: the same calls in the same order give Datafaker's
/// values.
pub struct Faker {
    locale: RwLock<(Locale, Vec<Locale>)>,
    random: RwLock<RandomService>,
    /// `FakeValuesService.fakeValuesInterfaceMap`: the data of each locale this faker's chains have reached, and
    /// what `addPath`/`addUrl` added.
    fake_values: RwLock<HashMap<Locale, FakeValuesInterface>>,
    /// `FakeValuesService.regexp2SupplierMap`: how each `#{…}` expression was resolved the first time.
    pub(crate) resolvers: Mutex<HashMap<String, Resolver>>,
    /// State that upstream keeps on the provider object `BaseFaker.getProvider` caches per faker (`Compass`'s
    /// chosen point, `Locality`'s shuffled locales), one value per provider class.
    provider_state: Mutex<HashMap<crate::providers::ProviderKind, Box<dyn std::any::Any + Send>>>,
    /// `BaseFaker.whiteListPredicate` over provider class names; `None` allows every provider.
    white_list: Option<Arc<WhiteList>>,
    /// `BaseFaker.providersCache` for the user's provider classes.
    providers: Mutex<HashMap<std::any::TypeId, Arc<dyn AnyObject>>>,
    /// The provider accessors a faker subclass would declare (`public Insect insect()`), in declaration order.
    declared: RwLock<Vec<Declared>>,
    /// `getClass().getSimpleName()` of the faker: `Faker`, or a subclass's name ([`Faker::subclass`]).
    simple_name: &'static str,
}

/// A white-list predicate: whether the provider class of that binary name may be created.
type WhiteList = dyn Fn(&str) -> bool + Send + Sync;

/// How a declared accessor gets its (cached) provider.
type GetProvider = dyn Fn(&Faker) -> Result<Arc<dyn AnyObject>> + Send + Sync;

/// A declared provider accessor: its Java name, the class's simple name and how it gets the provider.
#[derive(Clone)]
struct Declared {
    accessor: &'static str,
    simple_name: &'static str,
    get: Arc<GetProvider>,
}

impl Default for Faker {
    fn default() -> Self {
        Faker::new()
    }
}

impl Faker {
    /// `new Faker()`: English, unseeded.
    pub fn new() -> Faker {
        Faker::with_locale_and_random(Locale::english(), RandomService::default())
    }

    /// `new Faker(new Random(seed))`.
    pub fn with_seed(seed: i64) -> Faker {
        Faker::with_locale_and_random(Locale::english(), RandomService::with_seed(seed))
    }

    /// `new Faker(locale)`, unseeded.
    pub fn with_locale(locale: Locale) -> Faker {
        Faker::with_locale_and_random(locale, RandomService::default())
    }

    /// `new Faker(locale, new Random(seed))`.
    pub fn with_locale_and_seed(locale: Locale, seed: i64) -> Faker {
        Faker::with_locale_and_random(locale, RandomService::with_seed(seed))
    }

    /// `new Faker(locale, randomService)`.
    pub fn with_locale_and_random(locale: Locale, random: RandomService) -> Faker {
        Faker::build(locale, random, None)
    }

    /// `new Faker(new FakeValuesService(), new FakerContext(locale, randomService), whiteListPredicate)`: a provider
    /// whose class's binary name (`net.datafaker.providers.base.Name`) the predicate rejects cannot be created, so
    /// an expression reaching it fails with `RuntimeException: Provider '<name>' is not in white list` as the root
    /// cause. Upstream checks when the faker first creates a provider, through any path; here the check is made by
    /// expressions (and method tables) each time they reach a provider, and by [`Faker::get_provider`]. The Rust
    /// accessors (`faker.name()`) are views that cannot fail and are not checked.
    pub fn with_white_list(locale: Locale, random: RandomService, white_list: impl Fn(&str) -> bool + Send + Sync + 'static) -> Faker {
        Faker::build(locale, random, Some(Arc::new(white_list)))
    }

    fn build(locale: Locale, random: RandomService, white_list: Option<Arc<WhiteList>>) -> Faker {
        let locale = locale.normalize();
        let chain = locale.chain();
        let faker = Faker {
            locale: RwLock::new((locale, chain)),
            random: RwLock::new(random),
            fake_values: RwLock::new(HashMap::new()),
            resolvers: Mutex::new(HashMap::new()),
            provider_state: Mutex::new(HashMap::new()),
            white_list,
            providers: Mutex::new(HashMap::new()),
            declared: RwLock::new(Vec::new()),
            simple_name: "Faker",
        };
        faker.update_fake_values_interface_map();
        faker
    }

    /// This faker as an instance of a `Faker` subclass of that simple name (MockServer's `TemplateFaker`): only
    /// `toString()` (`TemplateFaker.en`, also inside a provider's `DateAndTime(TemplateFaker.en)@…`) shows it.
    pub fn subclass(mut self, simple_name: &'static str) -> Faker {
        self.simple_name = simple_name;
        self
    }

    /// The white-list check `BaseFaker.getProvider` makes before creating a provider of class `class_name`.
    pub(crate) fn check_white_list(&self, class_name: &str) -> Result<()> {
        match &self.white_list {
            Some(allowed) if !allowed(class_name) => Err(Error::runtime(format!("Provider '{class_name}' is not in white list"))),
            _ => Ok(()),
        }
    }

    /// `getProvider(clazz, valueSupplier)`: this faker's one provider of class `T`, created by `factory` on first
    /// use (after the white list allows it) and cached per faker. `factory` may add data to the faker
    /// (`addPath`); it must not ask for the same provider again.
    pub fn get_provider<T: JavaClass>(&self, factory: impl FnOnce(&Faker) -> T) -> Result<Arc<T>> {
        let id = std::any::TypeId::of::<T>();
        let cached = self.providers.lock().expect("providers").get(&id).cloned();
        let object = match cached {
            Some(object) => object,
            None => {
                self.check_white_list(T::NAME)?;
                let created: Arc<dyn AnyObject> = Arc::new(factory(self));
                self.providers.lock().expect("providers").entry(id).or_insert(created).clone()
            }
        };
        Ok(object.into_any().downcast::<T>().unwrap_or_else(|_| unreachable!("cached by its TypeId")))
    }

    /// Declares a provider accessor, as a `BaseFaker` subclass does with `public Insect insect() { return
    /// getProvider(Insect.class, Insect::new); }`: expressions then reach the provider by its class's simple name
    /// (`#{Insect.nextInsectName}`, `getProvider("Insect")`) and by the accessor's name (`#{insect.next_insect_name}`),
    /// creating it through [`Faker::get_provider`] on first use.
    pub fn declare_provider<T: JavaClass>(&self, accessor: &'static str, factory: fn(&Faker) -> T) {
        let get = Arc::new(move |faker: &Faker| faker.get_provider(factory).map(|p| p as Arc<dyn AnyObject>));
        let simple_name = crate::dispatch::simple_name_of(T::NAME);
        self.declared.write().expect("declared providers").push(Declared { accessor, simple_name, get });
    }

    /// `getProvider(simpleClassName)` for a declared provider (see [`Faker::declare_provider`]): `None` when no
    /// declared accessor returns that class; downcast the result to the provider's type.
    pub fn custom_provider(&self, simple_class_name: &str) -> Result<Option<Arc<dyn std::any::Any + Send + Sync>>> {
        Ok(self.declared_provider(|d| d.simple_name == simple_class_name)?.map(|p| p.into_any()))
    }

    /// The provider of the first declared accessor that `matches`, created on first use.
    fn declared_provider(&self, matches: impl Fn(&Declared) -> bool) -> Result<Option<Arc<dyn AnyObject>>> {
        let found = self.declared.read().expect("declared providers").iter().find(|d| matches(d)).cloned();
        match found {
            Some(d) => (d.get)(self).map(Some),
            None => Ok(None),
        }
    }

    /// `ProviderRegistration.getProvider(simpleClassName)` over the declared accessors: the faker's class name and
    /// the accessor's error, as `ObjectMethods.executeMethodByReturnType` wraps it.
    pub(crate) fn declared_provider_by_class(&self, simple_class_name: &str) -> Result<Option<Arc<dyn AnyObject>>> {
        self.declared_provider(|d| d.simple_name == simple_class_name)
            .map_err(|e| Error::runtime(format!("Failed to call method net.datafaker.Faker.{simple_class_name}()")).with_cause(e))
    }

    /// The declared accessor whose lower-cased name is `name` (`FakeValuesService.accessor` on the faker class).
    pub(crate) fn declared_provider_by_accessor(&self, name: &str) -> Result<Option<Arc<dyn AnyObject>>> {
        self.declared_provider(|d| d.accessor.to_ascii_lowercase() == name)
    }

    /// Runs `f` on the state of this faker's cached `kind` provider (created with `T::default()` on first use). `f`
    /// must not reach the same provider's state again.
    pub(crate) fn with_provider_state<T: Default + Send + 'static, R>(&self, kind: crate::providers::ProviderKind, f: impl FnOnce(&mut T) -> R) -> R {
        let mut states = self.provider_state.lock().expect("provider state");
        let state = states.entry(kind).or_insert_with(|| Box::new(T::default()));
        f(state.downcast_mut::<T>().expect("one state type per provider"))
    }

    /// `FakeValuesService.updateFakeValuesInterfaceMap(chain)`: the bundled data of each locale of the current
    /// chain that has no entry yet.
    fn update_fake_values_interface_map(&self) {
        let chain = self.locale_chain();
        let mut map = self.fake_values.write().expect("fake values");
        for locale in chain {
            map.entry(locale.clone()).or_insert_with(|| FakeValuesInterface::bundled(&locale));
        }
    }

    /// This faker's data for `locale`, if its service has an entry for it.
    pub(crate) fn fake_values_interface(&self, locale: &Locale) -> Option<FakeValuesInterface> {
        self.fake_values.read().expect("fake values").get(locale).cloned()
    }

    /// `addPath(locale, path)`: custom data for `locale` from a YAML file, as `addUrl` with the file's URL.
    pub fn add_path(&self, locale: &Locale, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let readable = path.is_file() && std::fs::File::open(path).is_ok();
        if !readable {
            return Err(Error::illegal_argument(format!("Path should be an existing readable file: \"{}\"", path.display())));
        }
        let absolute = std::path::absolute(path).map_err(|_| Error::illegal_argument(format!("Failed to read \"{}\"", path.display())))?;
        self.add_url(locale, &format!("file:{}", absolute.display()))
    }

    /// `addUrl(locale, url)`: custom data for `locale` from the YAML document at `url` (a `file:` URL). With data
    /// for `locale` already present (the locale is in a chain this faker has used), both are grouped; otherwise the
    /// document is the locale's only data.
    pub fn add_url(&self, locale: &Locale, url: &str) -> Result<()> {
        let fake_values = FakeValues::of(FakeValuesContext::of_url(locale, url));
        let mut map = self.fake_values.write().expect("fake values");
        let merged = match map.get(locale) {
            None => FakeValuesInterface::Values(fake_values),
            Some(previous) => {
                let mut grouping = FakeValuesGrouping::new();
                grouping.add(previous.clone())?;
                grouping.add(FakeValuesInterface::Values(fake_values))?;
                FakeValuesInterface::Grouping(Arc::new(grouping))
            }
        };
        map.insert(locale.clone(), merged);
        Ok(())
    }

    /// `getContext().getLocale()`.
    pub fn locale(&self) -> Locale {
        self.locale.read().expect("locale").0.clone()
    }

    /// `getContext().getLocaleChain()`.
    pub fn locale_chain(&self) -> Vec<Locale> {
        self.locale.read().expect("locale").1.clone()
    }

    /// `random()`.
    pub fn random(&self) -> RandomService {
        self.random.read().expect("random").clone()
    }

    fn set_locale(&self, locale: Locale) {
        let locale = locale.normalize();
        let chain = locale.chain();
        *self.locale.write().expect("locale") = (locale, chain);
    }

    /// `doWith(callable, locale)`: runs `f` with another locale, then restores the current one.
    pub fn do_with_locale<T>(&self, locale: Locale, f: impl FnOnce(&Faker) -> T) -> T {
        let current = self.locale();
        self.set_locale(locale);
        self.update_fake_values_interface_map();
        let result = f(self);
        self.set_locale(current);
        self.update_fake_values_interface_map();
        result
    }

    /// `doWith(callable, seed)`: runs `f` with `new RandomService(new Random(seed))`, then restores the current one.
    pub fn do_with_seed<T>(&self, seed: i64, f: impl FnOnce(&Faker) -> T) -> T {
        let current = std::mem::replace(&mut *self.random.write().expect("random"), RandomService::with_seed(seed));
        let result = f(self);
        *self.random.write().expect("random") = current;
        result
    }

    /// `getContext().setRandomService(randomService)` for the duration of `f`, then the current one again.
    pub fn do_with_random<T>(&self, random: RandomService, f: impl FnOnce(&Faker) -> T) -> T {
        let current = std::mem::replace(&mut *self.random.write().expect("random"), random);
        let result = f(self);
        *self.random.write().expect("random") = current;
        result
    }

    /// `doWith(callable, locale, seed)`: runs `f` with another locale and `new RandomService(new Random(seed))`,
    /// then restores both. Unlike `doWith(callable, locale)` it does not load the other locale's data into this
    /// faker's service, so a locale it has not used before reads as having none (upstream's behaviour).
    pub fn do_with_locale_and_seed<T>(&self, locale: Locale, seed: i64, f: impl FnOnce(&Faker) -> T) -> T {
        let current_locale = self.locale();
        let current_random = std::mem::replace(&mut *self.random.write().expect("random"), RandomService::with_seed(seed));
        self.set_locale(locale);
        let result = f(self);
        *self.random.write().expect("random") = current_random;
        self.set_locale(current_locale);
        result
    }

    /// `fakeValuesService()`, bound to this faker's context.
    pub fn fake_values_service(&self) -> FakeValuesService<'_> {
        FakeValuesService::new(self)
    }

    /// `numerify(numberString)`: each `#` becomes a digit, each `Ø` a non-zero digit.
    pub fn numerify(&self, number_string: &str) -> String {
        self.fake_values_service().numerify(number_string)
    }

    /// `letterify(letterString)`: each `?` becomes a lower-case letter.
    pub fn letterify(&self, letter_string: &str) -> String {
        self.fake_values_service().letterify(letter_string, false)
    }

    /// `letterify(letterString, isUpper)`.
    pub fn letterify_with(&self, letter_string: &str, is_upper: bool) -> String {
        self.fake_values_service().letterify(letter_string, is_upper)
    }

    /// `bothify(string)`: `numerify` and `letterify` in one pass.
    pub fn bothify(&self, string: &str) -> String {
        self.fake_values_service().bothify(string, false)
    }

    /// `bothify(string, isUpper)`.
    pub fn bothify_with(&self, string: &str, is_upper: bool) -> String {
        self.fake_values_service().bothify(string, is_upper)
    }

    /// `regexify(regex)`: a string matching `regex`, generated as RgxGen 3.1 does.
    pub fn regexify(&self, regex: &str) -> Result<String> {
        self.fake_values_service().regexify(regex)
    }

    /// `examplify(example)`: letters, upper-case letters and digits replaced by random ones of the same kind.
    pub fn examplify(&self, example: &str) -> String {
        self.fake_values_service().examplify(example)
    }

    /// `templatify(string, char2replace, options...)`.
    pub fn templatify(&self, string: &str, char2replace: char, options: &[&str]) -> Result<String> {
        self.fake_values_service().templatify(string, &[(char2replace, options)])
    }

    /// `csv(limit, columnExpressions...)`.
    pub fn csv(&self, limit: i32, column_expressions: &[&str]) -> Result<String> {
        self.fake_values_service().csv(limit, column_expressions)
    }

    /// `csv(separator, quote, withHeader, limit, columnExpressions...)`.
    pub fn csv_with(&self, separator: &str, quote: char, with_header: bool, limit: i32, column_expressions: &[&str]) -> Result<String> {
        self.fake_values_service().csv_with(separator, quote, with_header, limit, column_expressions)
    }

    /// `json(fieldExpressions...)`.
    pub fn json(&self, field_expressions: &[&str]) -> Result<String> {
        self.fake_values_service().json(field_expressions)
    }

    /// `jsona(fieldExpressions...)`.
    pub fn jsona(&self, field_expressions: &[&str]) -> Result<String> {
        self.fake_values_service().jsona(field_expressions)
    }

    /// `expression(expression)`: resolves every `#{…}` in `expression`.
    pub fn expression(&self, expression: &str) -> Result<String> {
        crate::expr::resolve_expression(self, expression, None)
    }

    /// `resolve(key)`: the YAML value at `key`, with its expressions resolved.
    pub fn resolve(&self, key: &str) -> Result<String> {
        crate::expr::resolve_key(self, key, Some(crate::dispatch::Target::Faker.into()))
    }
}

impl fmt::Display for Faker {
    /// `BaseFaker.toString()`: `Faker.en`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.simple_name, self.locale())
    }
}

impl fmt::Debug for Faker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Faker({})", self.locale())
    }
}

fn s(args: &[JValue], i: usize) -> &str {
    args[i].as_str()
}

/// The first argument, a string the Java body dereferences (with Java's helpful `NullPointerException` text).
fn nn<'a>(args: &'a [JValue], message: &str) -> Result<&'a str> {
    Ok(crate::dispatch::non_null(args, 0, message)?.as_str())
}

/// `bothify`'s `input`, the string numerify, letterify and bothify turn into a `char[]`.
fn input(args: &[JValue]) -> Result<&str> {
    nn(args, "Cannot invoke \"String.toCharArray()\" because \"input\" is null")
}

/// Varargs argument `i`, which Velocity passes as a null array for a single `null` (`array.length` throws).
fn array<'a>(args: &'a [JValue], i: usize, name: &str) -> Result<&'a JValue> {
    crate::dispatch::non_null(args, i, &format!("Cannot read the array length because \"{name}\" is null"))
}

fn strs(list: &JValue) -> Vec<&str> {
    list.as_list().iter().map(JValue::as_str).collect()
}

/// A `String[]` argument whose elements may be `null` (`None`).
fn opt_strs(list: &JValue) -> Vec<Option<&str>> {
    list.as_list().iter().map(crate::dispatch::opt_str).collect()
}

/// The faker's public methods that expressions reach (`#{regexify '…'}`, `#{Name.first_name}`): the provider
/// accessors of the registry, then these.
pub(crate) static FAKER_METHODS: LazyLock<Vec<Method>> = LazyLock::new(|| crate::providers::generated::ACCESSORS.iter().chain(OWN_METHODS).copied().collect());

static OWN_METHODS: &[Method] = &[
    Method::new("random", &[], |_, _| Ok(JValue::RandomService)),
    Method::new("numerify", &[P::String], |f, a| Ok(f.numerify(input(a)?).into())).accepting_null(),
    Method::new("letterify", &[P::String], |f, a| Ok(f.letterify(input(a)?).into())).accepting_null(),
    Method::new("letterify", &[P::String, P::Boolean], |f, a| Ok(f.letterify_with(input(a)?, a[1].as_bool()).into())).accepting_null(),
    Method::new("bothify", &[P::String], |f, a| Ok(f.bothify(input(a)?).into())).accepting_null(),
    Method::new("bothify", &[P::String, P::Boolean], |f, a| Ok(f.bothify_with(input(a)?, a[1].as_bool()).into())).accepting_null(),
    // `RgxGen.parse(null)`.
    Method::new("regexify", &[P::String], |f, a| Ok(f.regexify(nn(a, "Cannot invoke \"String.length()\" because \"this.aValue\" is null")?)?.into()))
        .accepting_null(),
    // `examplify(null)` returns `null`.
    Method::new("examplify", &[P::String], |f, a| Ok(if a[0] == JValue::Null { JValue::Null } else { f.examplify(s(a, 0)).into() })).accepting_null(),
    Method::varargs("templatify", &[P::String, P::Char, P::String], |f, a| {
        let options: Vec<&str> = a[2].as_list().iter().map(JValue::as_str).collect();
        let c = char::from_u32(a[1].as_char() as u32).unwrap_or('\u{FFFD}');
        Ok(f.templatify(s(a, 0), c, &options)?.into())
    }),
    // `optionsMap` is a `Map<Character, String[]>` ([`JValue::Map`]; a `String[]` value is a [`JValue::List`]): a
    // letter that is a key becomes one of its options, any other is kept.
    Method::new("templatify", &[P::String, P::Opaque("Map")], |f, a| {
        let letters = nn(a, "Cannot invoke \"String.length()\" because \"letterString\" is null")?;
        if letters.is_empty() {
            return Ok(JValue::str(""));
        }
        let JValue::Map(entries) = &a[1] else {
            return Err(crate::dispatch::npe("Cannot invoke \"java.util.Map.containsKey(Object)\" because \"optionsMap\" is null"));
        };
        let mut sb: Vec<u16> = Vec::new();
        for key in letters.encode_utf16() {
            let Some((_, value)) = entries.iter().find(|(k, _)| *k == JValue::Char(key)) else {
                sb.push(key);
                continue;
            };
            let options = match value {
                JValue::List(options) => options,
                JValue::Null => return Err(crate::dispatch::npe("Array with available options should be non null")),
                JValue::Foreign { class, .. } => return Err(crate::dispatch::class_cast(class, "[Ljava.lang.String;")),
                other => return Err(crate::dispatch::class_cast(&format!("{other:?}"), "[Ljava.lang.String;")),
            };
            let option = &options[f.random().next_int_bounded(options.len() as i32)? as usize];
            sb.extend(option.to_java_string(f).encode_utf16());
        }
        Ok(JValue::Str(String::from_utf16_lossy(&sb)))
    })
    .accepting_null(),
    Method::varargs("csv", &[P::Int, P::String], |f, a| {
        let columns = opt_strs(array(a, 1, "columnExpressions")?);
        Ok(f.fake_values_service().csv_with_nullable(crate::transformations::DEFAULT_SEPARATOR, crate::transformations::DEFAULT_QUOTE, true, a[0].as_int(), &columns)?.into())
    })
    .accepting_null_items(),
    Method::varargs("csv", &[P::String, P::Char, P::Boolean, P::Int, P::String], |f, a| {
        let quote = char::from_u32(a[1].as_char() as u32).unwrap_or('\u{FFFD}');
        Ok(f.csv_with(s(a, 0), quote, a[2].as_bool(), a[3].as_int(), &strs(&a[4]))?.into())
    }),
    Method::varargs("json", &[P::String], |f, a| Ok(f.fake_values_service().json_nullable(&opt_strs(array(a, 0, "fieldExpressions")?))?.into()))
        .accepting_null_items(),
    Method::varargs("jsona", &[P::String], |f, a| Ok(f.fake_values_service().jsona_nullable(&opt_strs(array(a, 0, "fieldExpressions")?))?.into()))
        .accepting_null_items(),
    Method::new("expression", &[P::String], |f, a| {
        Ok(f.expression(nn(a, "Cannot invoke \"String.contains(java.lang.CharSequence)\" because \"expression\" is null")?)?.into())
    })
    .accepting_null(),
    Method::new("resolve", &[P::String], |f, a| Ok(f.resolve(nn(a, "Cannot invoke \"String.length()\" because \"string\" is null")?)?.into()))
        .accepting_null(),
    // Reached only with a null supplier: a missing key asks it for the message.
    Method::new("resolve", &[P::String, P::Opaque("Supplier")], |f, a| {
        let key = nn(a, "Cannot invoke \"String.length()\" because \"string\" is null")?;
        let Some(expression) = f.fake_values_service().safe_fetch(key, None)? else {
            return Err(crate::dispatch::npe("Cannot invoke \"java.util.function.Supplier.get()\" because \"exceptionMessage\" is null"));
        };
        Ok(crate::expr::resolve_expression(f, &expression, Some(crate::dispatch::Target::Faker.into()))?.into())
    })
    .accepting_null(),
];
