use builder::OofBuilder;
use context::*;
use core::fmt::{self, Debug, Display, Write};
use std::{
    any::TypeId,
    collections::HashSet,
    error::{self, Error},
};

// FEATURES:
// - attribute macros
//   - #[oofs(tag(MyTag))]
//   - #[oofs(debug_strategy(owned))]
// - oof_eq!(actual, expected);
// - suggestions?
// display_owned:
//   default: debug on and release off

mod builder;
mod chain;
mod context;
mod ext;
mod tsa;
mod util;

pub use ext::OofExt;
pub use oofs_derive::*;

pub mod used_by_attribute {
    pub use crate::{builder::*, context::*, ext::*, tsa::*, util::*};

    pub const DISPLAY_OWNED: bool = cfg!(any(
        all(debug_assertions, not(feature = "display_owned_disabled")),
        all(not(debug_assertions), feature = "display_owned_release")
    ));
}

#[macro_export]
macro_rules! oof {
    ($($arg:tt)*) => {
        oofs::Oof::from_message(format!($($arg)*))
    };
}

#[track_caller]
pub fn err<T, E: From<Oof>>(e: impl 'static + Send + Sync + Error) -> Result<T, E> {
    Err(Oof::builder("Error encountered")
        .with_source(e)
        .build()
        .into())
}

pub struct Oof {
    source: Option<Box<dyn 'static + Send + Sync + Error>>,
    message: OofMessage,
    context: Option<String>,
    location: Option<Location>,
    tags: HashSet<TypeId>,
}

impl Display for Oof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#}", &self.message)?;

            let mut indented = Indented {
                inner: f,
                number: None,
                started: false,
            };

            if let Some(context) = &self.context {
                write!(indented, "\nwith context `{context}`")?;
            }

            if let Some(location) = &self.location {
                write!(indented, "\nat `{location}`")?;
            }

            if let OofMessage::Context(c) = &self.message {
                c.fmt_args(f)?;
            }
        } else {
            write!(f, "{}", &self.message)?;

            if let Some(context) = &self.context {
                write!(f, " with context `{context}`")?;
            }

            if let Some(location) = &self.location {
                write!(f, " at `{location}`")?;
            }
        }

        Ok(())
    }
}

impl Debug for Oof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return f
                .debug_struct("Oof")
                .field("message", &self.message)
                .field("source", &self.source)
                .field("context", &self.context)
                .field("location", &self.location)
                .field("tags", &self.tags)
                .finish();
        }

        write!(f, "{self:#}")?;

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

                writeln!(indented, "{error:#}")?;
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
    #[track_caller]
    pub fn from_message<T: Into<OofMessage>>(message: T) -> Oof {
        Self::builder(message).build()
    }

    #[track_caller]
    pub fn builder<T: Into<OofMessage>>(message: T) -> OofBuilder {
        OofBuilder::new(message.into())
    }

    pub fn tag<T: 'static>(&mut self) {
        self.tags.insert(TypeId::of::<T>());
    }

    pub fn tags(&self) -> impl Iterator<Item = &TypeId> {
        self.tags.iter()
    }

    pub fn tagged<T: 'static>(&self) -> bool {
        self.tags.contains(&TypeId::of::<T>())
    }

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
}
