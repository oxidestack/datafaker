//! `net.datafaker.internal.helper.LazyEvaluated`: a value computed on first use, once, even when several threads
//! ask at the same time.

use std::fmt;
use std::sync::OnceLock;

/// `LazyEvaluated`: a value its supplier computes on first use, once.
pub struct LazyEvaluated<T> {
    value: OnceLock<T>,
    supplier: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T> LazyEvaluated<T> {
    /// `new LazyEvaluated<>(supplier)`.
    pub fn new(supplier: impl Fn() -> T + Send + Sync + 'static) -> LazyEvaluated<T> {
        LazyEvaluated { value: OnceLock::new(), supplier: Box::new(supplier) }
    }

    /// `get()`: the supplier runs on the first call only.
    pub fn get(&self) -> &T {
        self.value.get_or_init(|| (self.supplier)())
    }
}

impl<T: fmt::Debug> fmt::Debug for LazyEvaluated<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LazyEvaluated({:?})", self.value.get())
    }
}
