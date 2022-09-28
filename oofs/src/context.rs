use crate::{builder::OofBuilder, Oof};
use core::fmt::{self, Debug, Display, Write};
use std::{
    any::TypeId,
    collections::HashSet,
    error::{self, Error},
};

// This default trait impl ensures no-op compile-success for non-static types.
// and ensures that only static types are successfully tagged to error.
pub trait __TagIfStatic {
    fn tag_if_static<T>(&mut self) {}
}
impl __TagIfStatic for Oof {}
impl Oof {
    pub fn tag_if_static<T: 'static>(&mut self) {
        self.tag::<T>()
    }
}

#[derive(Debug, Clone)]
pub enum OofMessage {
    FnContext(FnContext),
    Message(String),
}

impl From<FnContext> for OofMessage {
    fn from(c: FnContext) -> Self {
        OofMessage::FnContext(c)
    }
}

impl From<String> for OofMessage {
    fn from(m: String) -> Self {
        OofMessage::Message(m)
    }
}

impl From<&str> for OofMessage {
    fn from(m: &str) -> Self {
        OofMessage::Message(m.to_owned())
    }
}

impl Display for OofMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FnContext(c) => Display::fmt(c, f),
            Self::Message(m) => write!(f, "{m}"),
        }
    }
}

impl OofMessage {
    pub fn set_as_returning_option(&mut self) {
        if let Self::FnContext(fn_context) = self {
            fn_context.set_as_returning_option();
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnContext {
    pub fn_name: &'static str,
    pub params: Vec<FnArg>,
    pub returns_option: bool,
    pub is_async: bool,
}

impl Display for FnContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_fn(f, !f.alternate())
    }
}

impl FnContext {
    #[track_caller]
    pub fn new(is_async: bool, fn_name: &'static str, params: Vec<FnArg>) -> Self {
        Self {
            fn_name,
            params,
            returns_option: false,
            is_async,
        }
    }

    pub fn set_as_returning_option(&mut self) {
        self.returns_option = true;
    }
}

impl FnContext {
    pub fn write_fn(&self, f: &mut fmt::Formatter<'_>, multiline: bool) -> fmt::Result {
        if self.is_async {
            write!(f, "async ")?;
        }

        write!(f, "fn {}(", self.fn_name)?;

        let multiline = multiline && self.params.len() > 1;

        let mut params_iter = self.params.iter();

        let param = params_iter.next().expect("len checked above");
        if multiline {
            let mut indented = Indented {
                inner: f,
                number: None,
                started: false,
            };

            write!(indented, "\n{},", param.type_pair())?;
        } else {
            write!(f, "{}", param.type_pair())?;
        }

        for param in params_iter {
            if multiline {
                let mut indented = Indented {
                    inner: f,
                    number: None,
                    started: false,
                };

                write!(indented, "\n{},", param.type_pair())?;
            } else {
                write!(f, ", {}", param.type_pair())?;
            }
        }

        if multiline {
            write!(f, "\n")?;
        }

        if self.returns_option {
            write!(f, ") returned a `None`")?;
        } else {
            write!(f, ") failed")?;
        }

        Ok(())
    }

    pub fn write_params(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params = self
            .params
            .iter()
            .filter_map(|param| param.value_pair())
            .collect::<Vec<_>>();

        if !params.is_empty() {
            write!(f, "\n\nParamters:")?;

            let mut indented = Indented {
                inner: f,
                number: None,
                started: false,
            };

            for param in params {
                write!(indented, "\n{}", param)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FnArg {
    pub var_type: VarType,
    pub ref_type: RefType,
    pub type_name: &'static str,
    pub value: Option<String>,
}

impl Display for RefType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let x = match self {
            RefType::Ref => "&",
            RefType::RefMut => "&mut",
            RefType::Owned => "",
        };

        write!(f, "{x}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VarType {
    Expr {
        param_num: &'static str,
        expr: &'static str,
    },
    Var {
        name: &'static str,
    },
}

impl VarType {
    pub fn expr(param_num: &'static str, expr: &'static str) -> Self {
        Self::Expr { param_num, expr }
    }

    pub fn var(name: &'static str) -> Self {
        Self::Var { name }
    }

    pub fn var_name(&self) -> &str {
        match self {
            VarType::Expr { param_num, .. } => param_num,
            VarType::Var { name } => name,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RefType {
    Ref,
    RefMut,
    Owned,
}

impl FnArg {
    pub fn new(
        var_type: VarType,
        ref_type: RefType,
        type_name: &'static str,
        value: Option<String>,
    ) -> FnArg {
        Self {
            var_type,
            ref_type,
            type_name,
            value,
        }
    }

    pub fn type_pair(&self) -> String {
        format!(
            "${}: {}{}",
            self.var_type.var_name(),
            self.ref_type,
            self.type_name
        )
    }

    pub fn value_pair(&self) -> Option<String> {
        if let Some(value) = &self.value {
            let pair = match self.var_type {
                VarType::Var { name } => format!("${}: {}", name, value),
                VarType::Expr { param_num, expr } => {
                    format!("${}:\n    expr: {}\n    value: {}", param_num, expr, value)
                }
            };

            Some(pair)
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Location {
    /// The file where the error was reported
    file: &'static str,
    /// The line where the error was reported
    line: u32,
    /// The column where the error was reported
    column: u32,

    // Use `#[non_exhaustive]` when we upgrade to Rust 1.40
    _other: (),
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
        Self {
            file,
            line,
            column,
            _other: (),
        }
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
