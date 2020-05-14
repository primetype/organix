use std::fmt::{self, Display};
use syn::{Ident, Path};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Symbol(&'static str);

macro_rules! symbol {
    ($name:ident, $symbol:expr) => {
        pub const $name: Symbol = Symbol($symbol);
    };
}

symbol!(RUNTIME, "runtime");
symbol!(SHARED, "shared");
symbol!(SKIP, "skip");
symbol!(IO_DRIVER, "io");
symbol!(TIME_DRIVER, "time");

impl PartialEq<Symbol> for Ident {
    fn eq(&self, other: &Symbol) -> bool {
        self == other.0
    }
}

impl<'a> PartialEq<Symbol> for &'a Ident {
    fn eq(&self, other: &Symbol) -> bool {
        *self == other.0
    }
}

impl PartialEq<Symbol> for Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl<'a> PartialEq<Symbol> for &'a Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl Display for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.0)
    }
}
