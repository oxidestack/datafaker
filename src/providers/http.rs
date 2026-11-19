//! `net.datafaker.providers.base.Http`, with `Http.Browser`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{JValue, Method, P};
use crate::{Error, Faker, Result};

const ALL_STATUS_CATEGORIES: [&str; 5] = ["informational", "successful", "redirect", "client_error", "server_error"];
const TEXT_BODY_TYPES: [&str; 9] = ["json", "xml", "html", "plain", "csv", "javascript", "css", "graphql", "markdown"];

/// `Http.Browser`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Browser {
    /// `CHROME`.
    Chrome,
    /// `FIREFOX`.
    Firefox,
    /// `SAFARI`.
    Safari,
    /// `EDGE`.
    Edge,
    /// `MOBILE`.
    Mobile,
}

impl Browser {
    /// The constants in declaration order.
    pub const ALL: [Browser; 5] = [Browser::Chrome, Browser::Firefox, Browser::Safari, Browser::Edge, Browser::Mobile];

    fn key(self) -> &'static str {
        ["chrome", "firefox", "safari", "edge", "mobile"][self as usize]
    }
}

const BROWSER_CONSTANTS: &[&str] = &["CHROME", "FIREFOX", "SAFARI", "EDGE", "MOBILE"];

/// HTTP: status codes and reasons, headers, content types, user agents, methods and response bodies, from `http.*`.
#[derive(Clone, Copy)]
pub struct Http<'a> {
    faker: &'a Faker,
}

impl<'a> Http<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Http { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Http, key)
    }

    fn category(&self, category: &str) -> Result<String> {
        self.resolve(&format!("http.status_code.{category}"))
    }

    fn random_status_entry(&self) -> Result<String> {
        let category = self.faker.options().next_element(&ALL_STATUS_CATEGORIES)?;
        self.category(category)
    }

    /// A status code of any category.
    pub fn status_code(&self) -> Result<i32> {
        parse_code(&self.random_status_entry()?)
    }

    /// A 1xx code.
    pub fn informational(&self) -> Result<i32> {
        parse_code(&self.category("informational")?)
    }

    /// A 2xx code.
    pub fn successful(&self) -> Result<i32> {
        parse_code(&self.category("successful")?)
    }

    /// A 3xx code.
    pub fn redirect(&self) -> Result<i32> {
        parse_code(&self.category("redirect")?)
    }

    /// A 4xx code.
    pub fn client_error(&self) -> Result<i32> {
        parse_code(&self.category("client_error")?)
    }

    /// A 5xx code.
    pub fn server_error(&self) -> Result<i32> {
        parse_code(&self.category("server_error")?)
    }

    /// A reason phrase of any category.
    pub fn status_message(&self) -> Result<String> {
        Ok(parse_reason(&self.random_status_entry()?))
    }

    /// A 1xx reason phrase.
    pub fn informational_message(&self) -> Result<String> {
        Ok(parse_reason(&self.category("informational")?))
    }

    /// A 2xx reason phrase.
    pub fn successful_message(&self) -> Result<String> {
        Ok(parse_reason(&self.category("successful")?))
    }

    /// A 3xx reason phrase.
    pub fn redirect_message(&self) -> Result<String> {
        Ok(parse_reason(&self.category("redirect")?))
    }

    /// A 4xx reason phrase.
    pub fn client_error_message(&self) -> Result<String> {
        Ok(parse_reason(&self.category("client_error")?))
    }

    /// A 5xx reason phrase.
    pub fn server_error_message(&self) -> Result<String> {
        Ok(parse_reason(&self.category("server_error")?))
    }

    /// A code and its reason phrase (`404 Not Found`).
    pub fn status_code_with_reason(&self) -> Result<String> {
        self.random_status_entry()
    }

    /// `http.request_header`.
    pub fn request_header(&self) -> Result<String> {
        self.resolve("http.request_header")
    }

    /// `http.response_header`.
    pub fn response_header(&self) -> Result<String> {
        self.resolve("http.response_header")
    }

    /// `http.content_type`.
    pub fn content_type(&self) -> Result<String> {
        self.resolve("http.content_type")
    }

    /// A user agent of a random browser.
    pub fn user_agent(&self) -> Result<String> {
        let browser = self.faker.options().next_element(&Browser::ALL)?;
        self.user_agent_of(browser)
    }

    /// `userAgent(browser)`: `http.user_agent.<browser>`.
    pub fn user_agent_of(&self, browser: Browser) -> Result<String> {
        self.resolve(&format!("http.user_agent.{}", browser.key()))
    }

    /// `http.user_agent.mobile`.
    pub fn mobile_user_agent(&self) -> Result<String> {
        self.resolve("http.user_agent.mobile")
    }

    /// `http.http_method`.
    pub fn http_method(&self) -> Result<String> {
        self.resolve("http.http_method")
    }

    /// `http.http_version`.
    pub fn http_version(&self) -> Result<String> {
        self.resolve("http.http_version")
    }

    /// `http.encoding`.
    pub fn encoding(&self) -> Result<String> {
        self.resolve("http.encoding")
    }

    /// A response body of a random text type.
    pub fn response_body(&self) -> Result<String> {
        let kind = self.faker.options().next_element(&TEXT_BODY_TYPES)?;
        self.resolve(&format!("http.response_body.{kind}"))
    }

    /// `responseBody(contentType)`: a body for the media type (parameters ignored; JSON for unknown types).
    pub fn response_body_for(&self, content_type: &str) -> Result<String> {
        self.resolve(&format!("http.response_body.{}", content_type_to_body_key(content_type)))
    }
}

fn content_type_to_body_key(content_type: &str) -> &'static str {
    let media = content_type.split(';').next().unwrap_or("").trim_matches(|c: char| c <= ' ').to_lowercase();
    match media.as_str() {
        "application/xml" | "text/xml" => "xml",
        "text/html" => "html",
        "text/plain" => "plain",
        "text/csv" => "csv",
        "application/javascript" | "text/javascript" => "javascript",
        "text/css" => "css",
        "application/graphql" => "graphql",
        "text/markdown" => "markdown",
        _ => "json",
    }
}

/// `Integer.parseInt(entry.substring(0, entry.indexOf(' ')))`.
fn parse_code(entry: &str) -> Result<i32> {
    let Some(space) = entry.find(' ') else {
        return Err(Error::new("StringIndexOutOfBoundsException", format!("begin 0, end -1, length {}", entry.encode_utf16().count())));
    };
    let code = &entry[..space];
    crate::dispatch::java_parse_long(code)
        .and_then(|v| i32::try_from(v).ok())
        .ok_or_else(|| Error::new("NumberFormatException", format!("For input string: \"{code}\"")))
}

/// `entry.substring(entry.indexOf(' ') + 1)`.
fn parse_reason(entry: &str) -> String {
    entry.find(' ').map_or(entry, |i| &entry[i + 1..]).to_string()
}

fn i(r: Result<i32>) -> Result<JValue> {
    r.map(JValue::Int)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("statusCode", &[], |f, _| i(Http::new(f).status_code())),
    Method::new("informational", &[], |f, _| i(Http::new(f).informational())),
    Method::new("successful", &[], |f, _| i(Http::new(f).successful())),
    Method::new("redirect", &[], |f, _| i(Http::new(f).redirect())),
    Method::new("clientError", &[], |f, _| i(Http::new(f).client_error())),
    Method::new("serverError", &[], |f, _| i(Http::new(f).server_error())),
    Method::new("statusMessage", &[], |f, _| s(Http::new(f).status_message())),
    Method::new("informationalMessage", &[], |f, _| s(Http::new(f).informational_message())),
    Method::new("successfulMessage", &[], |f, _| s(Http::new(f).successful_message())),
    Method::new("redirectMessage", &[], |f, _| s(Http::new(f).redirect_message())),
    Method::new("clientErrorMessage", &[], |f, _| s(Http::new(f).client_error_message())),
    Method::new("serverErrorMessage", &[], |f, _| s(Http::new(f).server_error_message())),
    Method::new("statusCodeWithReason", &[], |f, _| s(Http::new(f).status_code_with_reason())),
    Method::new("requestHeader", &[], |f, _| s(Http::new(f).request_header())),
    Method::new("responseHeader", &[], |f, _| s(Http::new(f).response_header())),
    Method::new("contentType", &[], |f, _| s(Http::new(f).content_type())),
    Method::new("userAgent", &[], |f, _| s(Http::new(f).user_agent())),
    Method::new("userAgent", &[P::Enum("Browser", BROWSER_CONSTANTS)], |f, a| {
        let browser = match &a[0] {
            JValue::Enum(name) => BROWSER_CONSTANTS.iter().position(|c| c == name).map(|i| Browser::ALL[i]),
            _ => None,
        };
        crate::dispatch::non_null(a, 0, "Cannot read field \"key\" because \"browser\" is null")?;
        s(Http::new(f).user_agent_of(browser.unwrap_or(Browser::Chrome)))
    })
    .accepting_null(),
    Method::new("mobileUserAgent", &[], |f, _| s(Http::new(f).mobile_user_agent())),
    Method::new("httpMethod", &[], |f, _| s(Http::new(f).http_method())),
    Method::new("httpVersion", &[], |f, _| s(Http::new(f).http_version())),
    Method::new("encoding", &[], |f, _| s(Http::new(f).encoding())),
    Method::new("responseBody", &[], |f, _| s(Http::new(f).response_body())),
    Method::new("responseBody", &[P::String], |f, a| {
        let content_type = crate::dispatch::non_null(a, 0, "Cannot invoke \"String.contains(java.lang.CharSequence)\" because \"contentType\" is null")?;
        s(Http::new(f).response_body_for(content_type.as_str()))
    })
    .accepting_null(),
];
