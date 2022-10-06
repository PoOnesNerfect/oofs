use builder::*;
use context::*;
use core::fmt::{self, Debug, Display, Write};
use std::{
    any::TypeId,
    collections::HashSet,
    error::{self, Error},
};

#[cfg(all(feature = "debug_strategy_disabled", feature = "debug_strategy_full"))]
compile_error!(
    "features `debug_strategy_disabled` and `debug_strategy_full` are mutually exclusive"
);

pub use ext::OofExt;

/// Place above `fn` or `impl` to generate and inject context to `?` operators.
pub use oofs_derive::oofs;

/// Create a custom error `Oof` similar to `anyhow!`
///
/// You can format the error just like you do for `println!` and `anyhow!`.
///
/// Ex)
/// ```rust
/// # use oofs::{Oof, oofs, oof};
/// # #[oofs]
/// # fn _ex() -> Result<(), Oof> {
/// return oof!("custom error {}", "failure").into_res();
/// # }
/// ```
///
/// [Oof::into_res()](struct.Oof.html#method.into_res) wraps `Oof` in `Result::Err(_)`, so you can return it directly.
///
/// Since the macro returns `Oof`, you can chain methods like `tag` and `attach`.
///
/// Ex)
/// ```rust
/// # use oofs::{Oof, oofs, oof};
/// # #[oofs]
/// # fn _ex() -> Result<(), Oof> {
/// struct MyTag;
///
/// let x = 123usize;
///
/// return oof!("custom error {}", "failure").tag::<MyTag>().attach(x).into_res();
/// # }
/// ```
#[macro_export]
macro_rules! oof {
    ($($arg:tt)*) => {
        $crate::Oof::custom(format!($($arg)*))
    };
}

/// Check that a given expression evaluates to `true`, else return an error.
///
/// Ex)
/// ```rust
/// # use oofs::*;
/// # use std::time::Instant;
/// # #[oofs]
/// # fn _ex() -> Result<(), Oof> {
/// ensure!(false);
/// # Ok(())
/// # }
/// ```
///
/// First parameter is an expression that evaluates to `bool`.
/// If the expression evaluates to `false`, the macro will return `Err(Oof)`.
///
/// Second parameter is `context(...)`, and is an optional second parameter.
/// You can use this if you want to display your own context message, instead of the default `assertion failed: EXPRESSION at LOCATION`.
/// Inside `context(...)`, you can write as you do for `println!`.
///
/// Other optional parameters are for tagging, attach, and attach_lazy.
///
/// Ex)
/// ```rust
/// # use oofs::*;
/// # use std::time::Instant;
/// # #[oofs]
/// # fn _ex() -> Result<(), Oof> {
/// struct MyTag;
/// struct OtherTag;
///
/// let x = 123usize;
/// let y = "some value";
/// let z = "lazy attachment";
///
/// ensure!(
///   false,
///   context("custom context with value {:?}", x),
///   tag: [MyTag, OtherTag],
///   attach: [&y, "attachment", Instant::now()],
///   attach_lazy: [|| format!("context {}", &z)]
/// );
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! ensure {
    ($cond:expr, context($($arg:tt)*) $(, $($t:tt)*)?) => {
        $crate::ensure!(@internal $cond, $crate::oof!($($arg)*), $($($t)*)?);
    };
    ($cond:expr $(, $($t:tt)*)?) => {
        $crate::ensure!($cond, context("assertion failed: `{}`", stringify!($cond)), $($($t)*)?);
    };
    (@internal $cond:expr, $ret:expr, tag: [$($tag:ty),* $(,)?] $(, $($t:tt)*)?) => {
        $crate::ensure!(@internal $cond, $ret $(.tag::<$tag>())*, $($($t)*)?);
    };
    (@internal $cond:expr, $ret:expr, attach: [$($a:expr),* $(,)?] $(, $($t:tt)*)?) => {
        $crate::ensure!(@internal $cond, $ret $(.attach($a))*, $($($t)*)?);
    };
    (@internal $cond:expr, $ret:expr, attach_lazy: [$($l:expr),* $(,)?] $(, $($t:tt)*)?) => {
        $crate::ensure!(@internal $cond, $ret $(.attach_lazy($l))*, $($($t)*)?);
    };
    (@internal $cond:expr, $ret:expr, ) => {
        if !$cond {
            return $ret.into_res();
        }
    };
}

/// Check that two given expressions are same, else return an error.
///
/// Ex)
/// ```rust
/// # use oofs::*;
/// # use std::time::Instant;
/// # #[oofs]
/// # fn _ex() -> Result<(), Oof> {
/// ensure_eq!(1u8, 2u8);
/// # Ok(())
/// # }
/// ```
///
/// First two parameters are parameters to be compared.
/// If the parameters are not same, the macro will return `Err(Oof)`.
///
/// Third parameter is `context(...)`, and is an optional third parameter.
/// You can use this if you want to display your own context message, instead of the default `assertion failed: (left == right) at LOCATION`.
/// Inside `context(...)`, you can write as you do for `println!`.
///
/// Other optional parameters are for tagging, attach, and attach_lazy.
///
/// Ex)
/// ```rust
/// # use oofs::*;
/// # use std::time::Instant;
/// # #[oofs]
/// # fn _ex() -> Result<(), Oof> {
/// struct MyTag;
/// struct OtherTag;
///
/// let x = 123usize;
/// let y = "some value";
/// let z = "lazy attachment";
///
/// ensure_eq!(
///   1u8, 2u8,
///   context("custom context with value {:?}", x),
///   tag: [MyTag, OtherTag],
///   attach: [&y, "attachment", Instant::now()],
///   attach_lazy: [|| format!("context {}", &z)]
/// );
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! ensure_eq {
    ($l:expr, $r:expr, context($($c:tt)*) $(, $($t:tt)*)?) => {
        match (&$l, &$r) {
            (left, right) => {
                $crate::ensure!(*left == *right, context($($c)*), $($($t)*)?);
            }
        }
    };
    ($l:expr, $r:expr $(, $($t:tt)*)?) => {
        match (&$l, &$r) {
            (left, right) => {
                $crate::ensure!(
                    *left == *right,
                    context("assertion failed: `(left == right)`"),
                    attach_lazy: [
                        || format!(" left: {:?}", &*left),
                        || format!("right: {:?}", &*right)
                    ],
                    $($($t)*)?
                );
            }
        }
    };
}

/// Wraps a custom error with `Oof`
///
/// Ex)
/// ```rust
/// # use oofs::*;
/// # use std::time::Instant;
/// # #[oofs]
/// # fn _ex() -> Result<(), Oof> {
/// return wrap_err(std::io::Error::new(std::io::ErrorKind::Other, "Some Error")).into_res();
/// # Ok(())
/// # }
/// ```
///
/// Since `wrap_err(_)` returns `Oof`, you can chain methods like `tag` and `attach`.
///
/// Ex)
/// ```rust
/// # use oofs::*;
/// # use std::time::Instant;
/// # #[oofs]
/// # fn _ex() -> Result<(), Oof> {
/// struct MyTag;
/// let x = 123u8;
///
/// return wrap_err(std::io::Error::new(std::io::ErrorKind::Other, "Some Error"))
///     .tag::<MyTag>()
///     .attach(x)
///     .into_res();
/// # Ok(())
/// # }
/// ```
#[cfg_attr(feature = "location", track_caller)]
pub fn wrap_err(e: impl 'static + Send + Sync + Error) -> Oof {
    Oof::builder().with_source(e).build()
}

/// Error type for oofs.
///
/// `Oof` implements `std::error::Error`.
pub struct Oof {
    source: Option<Box<dyn 'static + Send + Sync + Error>>,
    context: Box<Context>,
    tags: HashSet<TypeId>,
    attachments: Vec<String>,
    #[cfg(feature = "location")]
    location: Location,
}

impl Display for Oof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let context = self.context.as_ref();

        write!(f, "{context}")?;

        #[cfg(feature = "location")]
        write!(f, " at `{}`", self.location)?;

        if matches!(context, Context::Generated(_)) || !self.attachments.is_empty() {
            writeln!(f)?;
        }

        if let Context::Generated(c) = context {
            c.fmt_args(f)?;
        }

        if !self.attachments.is_empty() {
            writeln!(f, "\nAttachments:")?;
            for (i, a) in self.attachments.iter().enumerate() {
                let mut indented = Indented {
                    inner: f,
                    number: Some(i),
                    started: false,
                };

                write!(indented, "{}", a)?;
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

impl Debug for Oof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            #[cfg(not(feature = "location"))]
            let debug = f
                .debug_struct("Oof")
                .field("context", &self.context)
                .field("source", &self.source)
                .field("tags", &self.tags)
                .field("attachments", &self.attachments)
                .finish();

            #[cfg(feature = "location")]
            let debug = f
                .debug_struct("Oof")
                .field("context", &self.context)
                .field("source", &self.source)
                .field("location", &self.location)
                .field("tags", &self.tags)
                .field("attachments", &self.attachments)
                .finish();

            return debug;
        }

        write!(f, "{self}")?;

        if let Some(cause) = self.source() {
            write!(f, "\nCaused by:")?;

            let multiple = cause.source().is_some();
            for (n, error) in chain::Chain::new(cause).enumerate() {
                writeln!(f)?;

                let mut indented = Indented {
                    inner: f,
                    number: if multiple { Some(n) } else { None },
                    started: false,
                };

                write!(indented, "{error}")?;
            }
        }

        Ok(())
    }
}

impl error::Error for Oof {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        if let Some(e) = &self.source {
            Some(e.as_ref())
        } else {
            None
        }
    }
}

impl Oof {
    /// Create a new `Oof` with custom context message.
    ///
    /// You can also use [oof!(...)](oof).
    #[cfg_attr(feature = "location", track_caller)]
    pub fn custom(message: String) -> Oof {
        Self::builder().with_custom(message).build()
    }

    #[cfg_attr(feature = "location", track_caller)]
    fn builder() -> OofBuilder {
        OofBuilder::new()
    }

    /// Lists all tags as [TypeId](https://doc.rust-lang.org/1.64.0/core/any/struct.TypeId.html).
    ///
    /// You can also use [oof!(...)](oof).
    pub fn tags(&self) -> impl Iterator<Item = &TypeId> {
        self.tags.iter()
    }

    /// Check if this `Oof` is tagged as given type.
    ///
    /// This method only checks one level deep.
    /// To check all nested errors, use [Oof::tagged_nested](struct.Oof.html#method.tagged_nested).
    pub fn tagged<T: 'static>(&self) -> bool {
        self.tags.contains(&TypeId::of::<T>())
    }

    /// Check if this `Oof` is tagged in all nested errors.
    ///
    /// This method checks all levels.
    pub fn tagged_nested<T: 'static>(&self) -> bool {
        if self.tagged::<T>() {
            return true;
        }

        for cause in chain::Chain::new(self).skip(1) {
            if let Some(e) = cause.downcast_ref::<Oof>() {
                if e.tagged::<T>() {
                    return true;
                }
            }
        }

        false
    }

    /// Check if this `Oof` is tagged in all nested errors in reverse order.
    ///
    /// This method checks all levels.
    pub fn tagged_nested_rev<T: 'static>(&self) -> bool {
        for cause in chain::Chain::new(self).skip(1).rev() {
            if let Some(e) = cause.downcast_ref::<Oof>() {
                if e.tagged::<T>() {
                    return true;
                }
            }
        }

        if self.tagged::<T>() {
            return true;
        }

        false
    }

    /// Tag `Oof` with type and return Self.
    pub fn tag<T: 'static>(mut self) -> Self {
        self.tags.insert(TypeId::of::<T>());
        self
    }

    /// Tag `Oof` if given closure returns `true` and return Self.
    pub fn tag_if<Tag, F>(self, f: F) -> Self
    where
        Tag: 'static,
        F: FnOnce(&Box<dyn 'static + Send + Sync + Error>) -> bool,
    {
        if let Some(source) = self.source.as_ref() {
            if f(&source) {
                return self.tag::<Tag>();
            }
        }

        self
    }

    /// Attach any value that implements `std::fmt::Debug`.
    ///
    /// This attached value will be listed as attachments in the displayed error.
    ///
    /// Ex)
    /// ```rust
    /// use oofs::{oof, oofs};
    /// # use oofs::Oof;
    ///
    /// # #[oofs]
    /// # fn _ex() -> Result<(), Oof> {
    /// let x = 123u8;
    ///
    /// return oof!("custom error")
    ///     .attach(x)
    ///     .attach("some attachment")
    ///     .into_res();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Above example will output:
    ///
    /// ```text
    /// custom error at `oofs/tests/basic.rs:9:13`
    ///
    /// Attachments:
    ///    0: 123
    ///    1: "some attachment"
    /// ```
    pub fn attach<D: fmt::Debug>(mut self, debuggable: D) -> Self {
        self.attachments.push(format!("{debuggable:?}"));
        self
    }

    /// Lazily load and attach any value that implements `ToString`.
    ///
    /// This attached value will be listed as attachments in the displayed error.
    ///
    /// Ex)
    /// ```rust
    /// use oofs::{oof, oofs};
    /// # use oofs::Oof;
    ///
    /// # #[oofs]
    /// # fn _ex() -> Result<(), Oof> {
    ///
    /// return oof!("custom error")
    ///     .attach_lazy(|| "some attachment")
    ///     .into_res();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Above example will output:
    ///
    /// ```text
    /// custom error at `oofs/tests/basic.rs:9:13`
    ///
    /// Attachments:
    ///    0: "some attachment"
    /// ```
    pub fn attach_lazy<D: ToString, F: FnOnce() -> D>(mut self, f: F) -> Self {
        self.attachments.push(f().to_string());
        self
    }

    /// Wraps `Oof` in `Result::Err(_)`.
    ///
    /// Use it to easily return an `Oof` instead of manually wrapping it in `Err(_)`.
    pub fn into_res<T, E>(self) -> Result<T, E>
    where
        E: From<Self>,
    {
        Err(self.into())
    }
}

mod builder;
mod chain;
mod context;
mod ext;
mod tsa;

/// Module by attribute `#[oofs]`
pub mod __used_by_attribute {
    pub use crate::{builder::*, context::*, tsa::*};

    pub const DEBUG_OWNED: bool = cfg!(all(
        not(feature = "debug_strategy_disabled"),
        any(debug_assertions, feature = "debug_strategy_full")
    ));
}
