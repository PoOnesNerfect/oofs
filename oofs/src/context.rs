use core::fmt::{self, Debug, Display, Write};

#[derive(Debug, Clone)]
pub(crate) enum Context {
    Generated(OofGeneratedContext),
    Custom(String),
    None,
}

impl Default for Context {
    fn default() -> Self {
        Context::None
    }
}

impl From<OofGeneratedContext> for Context {
    fn from(c: OofGeneratedContext) -> Self {
        Context::Generated(c)
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generated(c) => Display::fmt(c, f),
            Self::Custom(m) => Display::fmt(m, f),
            Self::None => write!(f, "Error encountered"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OofGeneratedContext {
    receiver: OofReceiver,
    chain: Vec<OofMethod>,
    returns_option: bool,
}

impl Display for OofGeneratedContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.receiver)?;

        let is_multiline = self.chain.len() > 2;
        if is_multiline && !f.alternate() {
            let mut indented = Indented {
                inner: f,
                number: None,
                started: false,
            };

            for method in &self.chain {
                write!(indented, "\n.{method}")?;
            }
        } else {
            for method in &self.chain {
                write!(f, ".{method}")?;
            }
        }

        if self.returns_option {
            write!(f, " returned `None`")?;
        } else {
            write!(f, " failed")?;
        }

        Ok(())
    }
}

impl OofGeneratedContext {
    pub fn new(receiver: OofReceiver) -> Self {
        Self {
            returns_option: false,
            receiver,
            chain: Vec::new(),
        }
    }

    pub fn with_capacity(receiver: OofReceiver, capacity: usize) -> Self {
        Self {
            returns_option: false,
            receiver,
            chain: Vec::with_capacity(capacity),
        }
    }

    pub fn with_method(mut self, method: OofMethod) -> Self {
        self.chain.push(method);
        self
    }

    pub fn returns_option(&mut self) {
        self.returns_option = true;
    }
}

impl OofGeneratedContext {
    pub fn fmt_args(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.receiver.args_exists() || self.chain.iter().any(|m| !m.args.is_empty()) {
            writeln!(f, "\nParameters:")?;

            let mut indented = Indented {
                inner: f,
                number: None,
                started: false,
            };

            self.receiver.fmt_args(&mut indented)?;

            for method in &self.chain {
                method.fmt_args(&mut indented)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum OofReceiver {
    Ident(OofIdent),
    Method(OofMethod),
    Arg(OofArg),
}

impl From<OofIdent> for OofReceiver {
    fn from(i: OofIdent) -> Self {
        Self::Ident(i)
    }
}

impl From<OofMethod> for OofReceiver {
    fn from(m: OofMethod) -> Self {
        Self::Method(m)
    }
}

impl From<OofArg> for OofReceiver {
    fn from(m: OofArg) -> Self {
        Self::Arg(m)
    }
}

impl Display for OofReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ident(i) => Display::fmt(i, f),
            Self::Method(m) => Display::fmt(m, f),
            Self::Arg(a) => Display::fmt(a, f),
        }
    }
}

impl OofReceiver {
    pub fn args_exists(&self) -> bool {
        match self {
            Self::Arg(_) => true,
            Self::Method(m) => !m.args.is_empty(),
            Self::Ident(_) => false,
        }
    }

    pub fn fmt_args(&self, f: &mut impl Write) -> fmt::Result {
        match self {
            Self::Arg(a) => writeln!(f, "{a:#}"),
            Self::Method(m) => m.fmt_args(f),
            Self::Ident(_) => Ok(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OofMethod {
    is_async: bool,
    name: &'static str,
    args: Vec<OofArg>,
}

impl Display for OofMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        write!(f, "(")?;
        match self.args.as_slice() {
            [rest @ .., last] => {
                for arg in rest {
                    write!(f, "{}, ", arg)?;
                }
                write!(f, "{}", last)?;
            }
            [] => {}
        }
        write!(f, ")")?;

        if self.is_async {
            write!(f, ".await")?;
        }

        Ok(())
    }
}

impl OofMethod {
    fn fmt_args(&self, f: &mut impl Write) -> fmt::Result {
        for arg in &self.args {
            writeln!(f, "{arg:#}")?;
        }

        Ok(())
    }
}

impl OofMethod {
    pub fn new(is_async: bool, name: &'static str, args: Vec<OofArg>) -> OofMethod {
        Self {
            is_async,
            name,
            args,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OofIdent {
    name: &'static str,
    is_async: bool,
}

impl Display for OofIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if self.is_async {
            write!(f, ".await")?;
        }

        Ok(())
    }
}

impl OofIdent {
    pub fn new(is_async: bool, name: &'static str) -> OofIdent {
        Self { name, is_async }
    }
}

#[derive(Debug, Clone)]
pub struct OofArg {
    index: usize,
    ty: &'static str,
    display: Option<String>,
}

impl Display for OofArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${}", self.index)?;

        if f.alternate() {
            write!(f, ": {}", self.ty)?;

            if let Some(display) = &self.display {
                write!(f, " = {display}")?;
            }
        }

        Ok(())
    }
}

impl OofArg {
    pub fn new(index: usize, ty: &'static str, display: Option<String>) -> Self {
        Self { index, ty, display }
    }
}

#[cfg(feature = "location")]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub struct Location {
    /// The file where the error was reported
    file: &'static str,
    /// The line where the error was reported
    line: u32,
    /// The column where the error was reported
    column: u32,
}

#[cfg(feature = "location")]
impl Default for Location {
    #[inline]
    #[cfg_attr(feature = "location", track_caller)]
    fn default() -> Self {
        Self::caller()
    }
}

#[cfg(feature = "location")]
impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{file}:{line}:{column}",
            file = self.file,
            line = self.line,
            column = self.column,
        )
    }
}

#[cfg(feature = "location")]
impl Location {
    /// Constructs a `Location` using the given information
    pub fn new(file: &'static str, line: u32, column: u32) -> Self {
        Self { file, line, column }
    }

    #[inline]
    #[cfg_attr(feature = "location", track_caller)]
    pub fn caller() -> Self {
        let loc = core::panic::Location::caller();
        Self::new(loc.file(), loc.line(), loc.column())
    }
}

pub(crate) struct Indented<'a, D> {
    pub(crate) inner: &'a mut D,
    pub(crate) number: Option<usize>,
    pub(crate) started: bool,
}

impl<T> Write for Indented<'_, T>
where
    T: Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for (i, line) in s.split('\n').enumerate() {
            if !self.started {
                self.started = true;
                match self.number {
                    Some(number) => write!(self.inner, "{: >5}: ", number)?,
                    None => self.inner.write_str("    ")?,
                }
            } else if i > 0 {
                self.inner.write_char('\n')?;
                if self.number.is_some() {
                    self.inner.write_str("       ")?;
                } else {
                    self.inner.write_str("    ")?;
                }
            }

            self.inner.write_str(line)?;
        }

        Ok(())
    }
}
