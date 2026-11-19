use std::fmt;

/// A Java exception that Datafaker would have thrown: its simple class name and its message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    class: &'static str,
    message: String,
    cause: Option<Box<Error>>,
    /// `getMessage() == null` (`new NullPointerException()`); [`Error::message`] is then empty.
    no_message: bool,
}

impl Error {
    /// An exception of the simple class name `class` with `message`.
    pub fn new(class: &'static str, message: impl Into<String>) -> Self {
        Error { class, message: message.into(), cause: None, no_message: false }
    }

    /// An exception thrown without a message (`new SimpleDateFormat(null)`'s bare `NullPointerException`).
    pub fn without_message(class: &'static str) -> Self {
        Error { class, message: String::new(), cause: None, no_message: true }
    }

    /// Whether `getMessage()` is not null.
    pub fn has_message(&self) -> bool {
        !self.no_message
    }

    /// The same exception with `cause` as its `getCause()`.
    pub fn with_cause(mut self, cause: Error) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// `getCause()`.
    pub fn cause(&self) -> Option<&Error> {
        self.cause.as_deref()
    }

    /// The innermost cause (AssertJ's `getRootCause()`), or this exception without one.
    pub fn root_cause(&self) -> &Error {
        let mut e = self;
        while let Some(c) = e.cause() {
            e = c;
        }
        e
    }

    /// An `IllegalArgumentException` with `message`.
    pub fn illegal_argument(message: impl Into<String>) -> Self {
        Error::new("IllegalArgumentException", message)
    }

    /// A `RuntimeException` with `message`.
    pub fn runtime(message: impl Into<String>) -> Self {
        Error::new("RuntimeException", message)
    }

    /// The exception's simple class name, e.g. `IllegalArgumentException`.
    pub fn simple_class(&self) -> &'static str {
        self.class
    }

    /// `getMessage()`: empty when the exception has none ([`Error::has_message`]).
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

/// A result whose error is the Java exception Datafaker would have thrown.
pub type Result<T> = std::result::Result<T, Error>;
