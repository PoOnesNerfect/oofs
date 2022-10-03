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
pub fn err<T>(e: impl 'static + Send + Sync + Error) -> Result<T, Oof> {
    Err(Oof::builder("Error encountered").with_source(e).build())
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

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::used_by_attribute::*;

    #[test]
    fn test_debug_ref() {
        let mut x = "hello world".to_owned();
        let y = Duration::from_secs(1);
        let z = 8usize;
        let a = Instant::now();
        struct ExStruct {
            field0: usize,
        }
        let b = ExStruct { field0: 0 };

        let __display_owned = false || DISPLAY_OWNED;

        let __0 = &x;
        let __0_name = "x";
        let __0_type = __0.__type_name();
        let __0_bin = __TsaBin(__0);
        let __0_ref_type = __0_bin.__ref_type();
        let __0_display_fn = __0_bin.__try_lazy_fn(__display_owned, |v| v.__try_debug());
        let __0 = __0_bin.__tsa_unload();

        let __1_async = false;
        let __1_name = "parse::<u64>";

        let __2_async = true;
        let __2_name = "some_other";

        let __2_0 = y;
        let __2_0_name = "y";
        let __2_0_type = __2_0.__type_name();
        let __2_0_bin = __TsaBin(__2_0);
        let __2_0_ref_type = __2_0_bin.__ref_type();
        let __2_0_display_fn = __2_0_bin.__try_lazy_fn(__display_owned, |v| v.__try_debug());
        let __2_0 = __2_0_bin.__tsa_unload();

        let __3_async = false;
        let __3_name = "some_another";

        let __3_0 = z;
        let __3_0_name = "z";
        let __3_0_type = __3_0.__type_name();
        let __3_0_bin = __TsaBin(__3_0);
        let __3_0_ref_type = __3_0_bin.__ref_type();
        let __3_0_display_fn = __3_0_bin.__try_lazy_fn(__display_owned, |v| v.__try_debug());
        let __3_0 = __3_0_bin.__tsa_unload();

        let __3_1 = a;
        let __3_1_name = "a";
        let __3_1_type = __3_1.__type_name();
        let __3_1_bin = __TsaBin(__3_1);
        let __3_1_ref_type = __3_1_bin.__ref_type();
        let __3_1_display_fn = __3_1_bin.__try_lazy_fn(__display_owned, |v| v.__try_debug());
        let __3_1 = __3_1_bin.__tsa_unload();

        let __3_2 = &b;
        let __3_2_name = "b";
        let __3_2_type = __3_2.__type_name();
        let __3_2_bin = __TsaBin(__3_2);
        let __3_2_ref_type = __3_2_bin.__ref_type();
        let __3_2_display_fn = __3_2_bin.__try_lazy_fn(__display_owned, |v| v.__try_debug());
        let __3_2 = __3_2_bin.__tsa_unload();

        let err = __0
            .parse::<u64>()
            .map_err(|x| x)
            .with_oof_builder(|| {
                let mut context = Context::new(
                    Arg::new(__0_name, __0_ref_type, __0_type, __0_display_fn.call()).into(),
                );

                let __1_args = vec![];
                let __1 = Method::new(__1_async, __1_name, __1_args);
                context.add_method(__1);

                let __2_args = vec![Arg::new(
                    __2_0_name,
                    __2_0_ref_type,
                    __2_0_type,
                    __2_0_display_fn.call(),
                )];
                let __2 = Method::new(__2_async, __2_name, __2_args);
                context.add_method(__2);

                let __3_args = vec![
                    Arg::new(
                        __3_0_name,
                        __3_0_ref_type,
                        __3_0_type,
                        __3_0_display_fn.call(),
                    ),
                    Arg::new(
                        __3_1_name,
                        __3_1_ref_type,
                        __3_1_type,
                        __3_1_display_fn.call(),
                    ),
                    Arg::new(
                        __3_2_name,
                        __3_2_ref_type,
                        __3_2_type,
                        __3_2_display_fn.call(),
                    ),
                ];
                let __3 = Method::new(__3_async, __3_name, __3_args);
                context.add_method(__3);

                OofBuilder::new(context.into())
            })
            .map_err(|b| b.build())
            .unwrap_err();

        println!("[ERROR] {err:?}");
    }

    #[test]
    fn test_debug_owned() {
        let x = "hello world".to_owned();

        let __display_owned = false || DISPLAY_OWNED;

        let __0 = x;
        let __0_name = "x";
        let __0_type = __0.__type_name();
        let __0_bin = __TsaBin(__0);
        let __0_ref_type = __0_bin.__ref_type();
        let __0_display_fn = __0_bin.__try_lazy_fn(__display_owned, |v| v.__try_debug());
        let __0 = __0_bin.__tsa_unload();

        let __1_async = false;
        let __1_name = "parse::<u64>";

        let err = __0
            .parse::<u64>()
            .with_oof_builder(|| {
                let __0 = Arg::new(__0_name, __0_ref_type, __0_type, __0_display_fn.call());

                let mut context = Context::new(__0.into());

                let __1_args = vec![];
                let __1 = Method::new(__1_async, __1_name, __1_args);
                context.add_method(__1);

                OofBuilder::new(context.into())
            })
            .map_err(|b| b.build())
            .unwrap_err();

        println!("[ERROR] {err:?}");
    }

    #[test]
    fn test_return_from_tag() {
        pub struct MyTag;

        let x = "hello world".to_owned();

        let __display_owned = false || DISPLAY_OWNED;

        let __0 = &x;
        let __0_name = "x";
        let __0_type = __0.__type_name();
        let __0_bin = __TsaBin(__0);
        let __0_ref_type = __0_bin.__ref_type();
        let __0_display_fn = __0_bin.__try_lazy_fn(__display_owned, |v| v.__try_debug());
        let __0 = __0_bin.__tsa_unload();

        let __1_async = false;
        let __1_name = "parse::<u64>";

        let err = __0
            .parse::<u64>()
            .with_oof_builder(|| {
                let __0 = Arg::new(__0_name, __0_ref_type, __0_type, __0_display_fn.call());

                let mut context = Context::new(__0.into());

                let __1_args = vec![];
                let __1 = Method::new(__1_async, __1_name, __1_args);
                context.add_method(__1);

                OofBuilder::new(context.into())
            })
            .tag::<MyTag>()
            .map_err(|b| b.build())
            .unwrap_err();

        println!("[ERROR] {err:?}");
    }
}
