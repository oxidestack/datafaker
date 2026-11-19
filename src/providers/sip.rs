//! `net.datafaker.providers.base.Sip`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{JValue, Method};
use crate::{Error, Faker, Result};

/// SIP: methods, ports, response codes and phrases, message bodies and name addresses, from `sip.*`.
#[derive(Clone, Copy)]
pub struct Sip<'a> {
    faker: &'a Faker,
}

impl<'a> Sip<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Sip { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Sip, key)
    }

    /// `sip.methods`.
    pub fn method(&self) -> Result<String> {
        self.resolve("sip.methods")
    }

    /// `sip.content.types`.
    pub fn content_type(&self) -> Result<String> {
        self.resolve("sip.content.types")
    }

    /// A port from 1000 to 9999.
    pub fn messaging_port(&self) -> Result<i32> {
        self.faker.random().next_int_between(1000, 9999)
    }

    /// An even port from 40000 to 50000 (`options().option(portPool)`).
    pub fn rtp_port(&self) -> Result<i32> {
        Ok(40000 + 2 * self.faker.random().next_int_bounded(5001)?)
    }

    /// `Integer.parseInt(faker.resolve(key))`.
    fn code(&self, key: &str) -> Result<i32> {
        let value = self.faker.resolve(key)?;
        crate::dispatch::java_parse_long(&value)
            .and_then(|v| i32::try_from(v).ok())
            .ok_or_else(|| Error::new("NumberFormatException", format!("For input string: \"{value}\"")))
    }

    /// A 1xx code.
    pub fn provisional_response_code(&self) -> Result<i32> {
        self.code("sip.response.codes.provisional")
    }

    /// A 2xx code.
    pub fn success_response_code(&self) -> Result<i32> {
        self.code("sip.response.codes.success")
    }

    /// A 3xx code.
    pub fn redirect_response_code(&self) -> Result<i32> {
        self.code("sip.response.codes.redirection")
    }

    /// A 4xx code.
    pub fn client_error_response_code(&self) -> Result<i32> {
        self.code("sip.response.codes.clientError")
    }

    /// A 5xx code.
    pub fn server_error_response_code(&self) -> Result<i32> {
        self.code("sip.response.codes.serverError")
    }

    /// A 6xx code.
    pub fn global_error_response_code(&self) -> Result<i32> {
        self.code("sip.response.codes.globalError")
    }

    /// `sip.response.phrases.provisional`.
    pub fn provisional_response_phrase(&self) -> Result<String> {
        self.resolve("sip.response.phrases.provisional")
    }

    /// `sip.response.phrases.success`.
    pub fn success_response_phrase(&self) -> Result<String> {
        self.resolve("sip.response.phrases.success")
    }

    /// `sip.response.phrases.redirection`.
    pub fn redirect_response_phrase(&self) -> Result<String> {
        self.resolve("sip.response.phrases.redirection")
    }

    /// `sip.response.phrases.clientError`.
    pub fn client_error_response_phrase(&self) -> Result<String> {
        self.resolve("sip.response.phrases.clientError")
    }

    /// `sip.response.phrases.serverError`.
    pub fn server_error_response_phrase(&self) -> Result<String> {
        self.resolve("sip.response.phrases.serverError")
    }

    /// `sip.response.phrases.globalError`.
    pub fn global_error_response_phrase(&self) -> Result<String> {
        self.resolve("sip.response.phrases.globalError")
    }

    /// An SDP body.
    pub fn body_string(&self) -> Result<String> {
        let first_name = self.faker.name().first_name()?;
        let uuid = self.faker.internet().uuid();
        let domain = self.faker.internet().domain_name()?;
        let ip = self.faker.internet().ip_v4_address();
        let port = self.rtp_port()?;
        Ok(format!(
            "v=0\no={first_name} {uuid} IN IP4 {domain}\ns=-\nc=IN IP4 {ip}\nt=0 0\nm=audio {port} RTP/AVP 0\na=rtpmap:0 PCMU/8000"
        ))
    }

    /// The SDP body in UTF-8.
    pub fn body_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.body_string()?.into_bytes())
    }

    /// `<sip:name@ip:port>`.
    pub fn name_address(&self) -> Result<String> {
        let first_name = self.faker.name().first_name()?;
        let ip = self.faker.internet().ip_v4_address();
        Ok(format!("<sip:{first_name}@{ip}:{}>", self.messaging_port()?))
    }
}

fn i(r: Result<i32>) -> Result<JValue> {
    r.map(JValue::Int)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("method", &[], |f, _| s(Sip::new(f).method())),
    Method::new("contentType", &[], |f, _| s(Sip::new(f).content_type())),
    Method::new("messagingPort", &[], |f, _| i(Sip::new(f).messaging_port())),
    Method::new("rtpPort", &[], |f, _| i(Sip::new(f).rtp_port())),
    Method::new("provisionalResponseCode", &[], |f, _| i(Sip::new(f).provisional_response_code())),
    Method::new("successResponseCode", &[], |f, _| i(Sip::new(f).success_response_code())),
    Method::new("redirectResponseCode", &[], |f, _| i(Sip::new(f).redirect_response_code())),
    Method::new("clientErrorResponseCode", &[], |f, _| i(Sip::new(f).client_error_response_code())),
    Method::new("serverErrorResponseCode", &[], |f, _| i(Sip::new(f).server_error_response_code())),
    Method::new("globalErrorResponseCode", &[], |f, _| i(Sip::new(f).global_error_response_code())),
    Method::new("provisionalResponsePhrase", &[], |f, _| s(Sip::new(f).provisional_response_phrase())),
    Method::new("successResponsePhrase", &[], |f, _| s(Sip::new(f).success_response_phrase())),
    Method::new("redirectResponsePhrase", &[], |f, _| s(Sip::new(f).redirect_response_phrase())),
    Method::new("clientErrorResponsePhrase", &[], |f, _| s(Sip::new(f).client_error_response_phrase())),
    Method::new("serverErrorResponsePhrase", &[], |f, _| s(Sip::new(f).server_error_response_phrase())),
    Method::new("globalErrorResponsePhrase", &[], |f, _| s(Sip::new(f).global_error_response_phrase())),
    Method::new("bodyString", &[], |f, _| s(Sip::new(f).body_string())),
    Method::new("bodyBytes", &[], |f, _| {
        Sip::new(f).body_bytes().map(|b| JValue::List(b.into_iter().map(|x| JValue::Int(x as i8 as i32)).collect()))
    }),
    Method::new("nameAddress", &[], |f, _| s(Sip::new(f).name_address())),
];
