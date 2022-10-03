use core::fmt::{self, Debug, Display, Write};

#[derive(Debug, Clone)]
pub enum OofMessage {
    Context(Context),
    Custom(String),
}

impl From<Context> for OofMessage {
    fn from(c: Context) -> Self {
        OofMessage::Context(c)
    }
}

impl From<String> for OofMessage {
    fn from(m: String) -> Self {
        OofMessage::Custom(m)
    }
}

impl From<&str> for OofMessage {
    fn from(m: &str) -> Self {
        OofMessage::Custom(m.to_owned())
    }
}

impl Display for OofMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Context(c) => Display::fmt(c, f),
            Self::Custom(m) => Display::fmt(m, f),
        }
    }
}

impl OofMessage {
    pub fn returns_option(&mut self) {
        if let Self::Context(fn_context) = self {
            fn_context.returns_option();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    returns_option: bool,
    receiver: Receiver,
    chain: Vec<Method>,
}

impl Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.receiver)?;

        let is_multiline = self.chain.len() > 2;
        if is_multiline && f.alternate() {
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

impl Context {
    pub fn new(receiver: Receiver) -> Self {
        Self {
            returns_option: false,
            receiver,
            chain: Vec::new(),
        }
    }

    pub fn with_capacity(receiver: Receiver, capacity: usize) -> Self {
        Self {
            returns_option: false,
            receiver,
            chain: Vec::with_capacity(capacity),
        }
    }

    pub fn add_method(&mut self, method: Method) {
        self.chain.push(method);
    }

    pub fn returns_option(&mut self) {
        self.returns_option = true;
    }
}

impl Context {
    pub fn fmt_args(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.receiver.args_exists() || self.chain.iter().any(|m| !m.args.is_empty()) {
            writeln!(f, "\n\nParameters:")?;

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
pub enum Receiver {
    Ident(Ident),
    Method(Method),
    Arg(Arg),
}

impl From<Ident> for Receiver {
    fn from(i: Ident) -> Self {
        Self::Ident(i)
    }
}

impl From<Method> for Receiver {
    fn from(m: Method) -> Self {
        Self::Method(m)
    }
}

impl From<Arg> for Receiver {
    fn from(m: Arg) -> Self {
        Self::Arg(m)
    }
}

impl Display for Receiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ident(i) => Display::fmt(i, f),
            Self::Method(m) => Display::fmt(m, f),
            Self::Arg(a) => Display::fmt(a, f),
        }
    }
}

impl Receiver {
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
pub struct Method {
    is_async: bool,
    name: &'static str,
    args: Vec<Arg>,
}

impl Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        write!(f, "(")?;
        match self.args.as_slice() {
            // [arg] => write!(f, "{}", arg)?,
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

impl Method {
    fn fmt_args(&self, f: &mut impl Write) -> fmt::Result {
        for arg in &self.args {
            writeln!(f, "{arg:#}")?;
        }

        Ok(())
    }
}

impl Method {
    pub fn new(is_async: bool, name: &'static str, args: Vec<Arg>) -> Method {
        Self {
            is_async,
            name,
            args,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ident {
    name: &'static str,
    is_async: bool,
}

impl Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if self.is_async {
            write!(f, ".await")?;
        }

        Ok(())
    }
}

impl Ident {
    pub fn new(is_async: bool, name: &'static str) -> Ident {
        Self { name, is_async }
    }
}

#[derive(Debug, Clone)]
pub struct Arg {
    name: &'static str,
    ref_ty: RefType,
    ty: &'static str,
    display: Option<String>,
}

impl Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${}", self.name)?;

        if f.alternate() {
            write!(f, ": {}{}", self.ref_ty, self.ty)?;

            if let Some(display) = &self.display {
                write!(f, " = {display}")?;
            }
        }

        Ok(())
    }
}

impl Arg {
    pub fn new(
        name: &'static str,
        ref_ty: RefType,
        ty: &'static str,
        display: Option<String>,
    ) -> Self {
        Self {
            ref_ty,
            name,
            ty,
            display,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RefType {
    Ref,
    RefMut,
    Owned,
}

impl Display for RefType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let x = match self {
            RefType::Ref => "&",
            RefType::RefMut => "",
            RefType::Owned => "",
        };

        write!(f, "{x}")
    }
}

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

impl Default for Location {
    #[inline]
    #[track_caller]
    fn default() -> Self {
        Self::caller()
    }
}

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

impl Location {
    /// Constructs a `Location` using the given information
    pub fn new(file: &'static str, line: u32, column: u32) -> Self {
        Self { file, line, column }
    }

    #[inline]
    #[track_caller]
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
