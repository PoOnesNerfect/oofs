use crate::{builder::OofBuilder, tags::Tags};
use core::fmt;
use std::{convert::Infallible, error::Error};

/// Helper trait for `Result` and `Option` to add tags and attach extra contexts.
///
/// Ex)
///
/// ```rust
/// use oofs::{oofs, Oof, OofExt};
///
/// struct MyTag;
///
/// #[oofs]
/// fn some_fn(x: usize) -> Result<u64, Oof> {
///     let ret = "hello world"
///         .parse::<u64>()
///         ._tag::<MyTag>()                    // tags the error with the type `RetryTag`.
///         ._attach(x)                         // attach anything that implements `Debug` as custom context.
///         ._attach_lazy(|| "extra context")?; // lazily evaluate context; useful for something like `|| serde_json::to_string(&x)`.
///
///     Ok(ret)
/// }
/// ```
pub trait OofExt: Sized {
    type Return;
    type Error: 'static + Send + Sync + Error;

    /// Build the error `Oof` with the given context, instead of using the generated context by attribute.
    fn _context<D: ToString>(self, context: D) -> Result<Self::Return, OofBuilder<Self::Error>>;

    /// Tag the given type that can be searched with `.tagged_nested::<T>()` in the higher level call.
    fn _tag<Tag: 'static>(self) -> Result<Self::Return, OofBuilder<Self::Error>>;

    /// Tag the given type if the closure evaluates to `true`.
    ///
    /// Closure provides the underlying source error, so that one can optionally use the source error to determine
    /// to tag the error or not.
    fn _tag_if<Tag: 'static, F: FnOnce(&Self::Error) -> bool>(
        self,
        f: F,
    ) -> Result<Self::Return, OofBuilder<Self::Error>>;

    /// Tag the given type if the closure evaluates to `true`.
    ///
    /// Closure provides the underlying source error, so that one can optionally use the source error to determine
    /// to tag the error or not.
    fn _tag_manually<F: FnOnce(&Self::Error, &mut Tags)>(
        self,
        f: F,
    ) -> Result<Self::Return, OofBuilder<Self::Error>>;

    /// Attach any value that implements `std::fmt::Debug`.
    ///
    /// This attached value will be listed as attachments in the displayed error.
    ///
    /// Ex)
    /// ```rust
    /// # use oofs::*;
    /// # use std::time::Instant;
    /// # #[oofs]
    /// # fn _ex() -> Result<(), Oof> {
    /// let x = 123u8;
    ///
    /// "hello world"
    ///     .parse::<usize>()
    ///     ._attach(x)
    ///     ._attach("some attachment")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Above example will output:
    /// ```text
    /// $0.parse() failed at `oofs/tests/basic.rs:11:11`
    ///
    /// Parameters:
    ///     $0: &str = "hello world"
    ///
    /// Attachments:
    ///     0: 123
    ///     1: "some attachment"
    ///
    /// Caused by:
    ///     invalid digit found in string
    /// ```
    fn _attach<D: fmt::Debug>(self, debuggable: D)
        -> Result<Self::Return, OofBuilder<Self::Error>>;

    /// Lazily load and attach any value that implements `ToString`.
    ///
    /// This attached value will be listed as attachments in the displayed error.
    ///
    /// Ex)
    /// ```rust
    /// # use oofs::*;
    /// # use std::time::Instant;
    /// # #[oofs]
    /// # fn _ex() -> Result<(), Oof> {
    ///
    /// "hello world"
    ///     .parse::<usize>()
    ///     ._attach_lazy(|| "some attachment")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Above example will output:
    /// ```text
    /// $0.parse() failed at `oofs/tests/basic.rs:11:11`
    ///
    /// Parameters:
    ///     $0: &str = "hello world"
    ///
    /// Attachments:
    ///     0: "some attachment"
    ///
    /// Caused by:
    ///     invalid digit found in string
    /// ```
    fn _attach_lazy<D: ToString, F: FnOnce() -> D>(
        self,
        f: F,
    ) -> Result<Self::Return, OofBuilder<Self::Error>>;
}

impl<T, E> OofExt for Result<T, E>
where
    E: 'static + Send + Sync + Error,
{
    type Return = T;
    type Error = E;

    #[cfg_attr(feature = "location", track_caller)]
    fn _context<D: ToString>(self, context: D) -> Result<Self::Return, OofBuilder<Self::Error>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(OofBuilder::new().with_custom(context).with_source(e)),
        }
    }

    #[cfg_attr(feature = "location", track_caller)]
    fn _tag<Tag: 'static>(self) -> Result<Self::Return, OofBuilder<Self::Error>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(OofBuilder::new().with_source(e).with_tag::<Tag>()),
        }
    }

    #[cfg_attr(feature = "location", track_caller)]
    fn _tag_if<Tag: 'static, F: FnOnce(&Self::Error) -> bool>(
        self,
        f: F,
    ) -> Result<Self::Return, OofBuilder<Self::Error>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(OofBuilder::new().with_source(e).with_tag_if::<Tag, _>(f)),
        }
    }

    #[cfg_attr(feature = "location", track_caller)]
    fn _tag_manually<F: FnOnce(&Self::Error, &mut Tags)>(
        self,
        f: F,
    ) -> Result<Self::Return, OofBuilder<Self::Error>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(OofBuilder::new().with_source(e).with_tag_manually(f)),
        }
    }

    #[cfg_attr(feature = "location", track_caller)]
    fn _attach<D: fmt::Debug>(
        self,
        debuggable: D,
    ) -> Result<Self::Return, OofBuilder<Self::Error>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(OofBuilder::new().with_source(e).with_attachment(debuggable)),
        }
    }

    #[cfg_attr(feature = "location", track_caller)]
    fn _attach_lazy<D: ToString, F: FnOnce() -> D>(
        self,
        f: F,
    ) -> Result<Self::Return, OofBuilder<Self::Error>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(OofBuilder::new().with_source(e).with_attachment_lazy(f)),
        }
    }
}

impl<T> OofExt for Option<T> {
    type Return = T;
    type Error = Infallible;

    #[cfg_attr(feature = "location", track_caller)]
    fn _context<D: ToString>(self, context: D) -> Result<Self::Return, OofBuilder<Self::Error>> {
        match self {
            Some(t) => Ok(t),
            None => Err(OofBuilder::new().with_custom(context)),
        }
    }

    #[cfg_attr(feature = "location", track_caller)]
    fn _tag<Tag: 'static>(self) -> Result<T, OofBuilder> {
        match self {
            Some(t) => Ok(t),
            None => Err(OofBuilder::new().with_tag::<Tag>()),
        }
    }

    #[cfg_attr(feature = "location", track_caller)]
    fn _tag_if<Tag: 'static, F: FnOnce(&Self::Error) -> bool>(self, f: F) -> Result<T, OofBuilder> {
        match self {
            Some(t) => Ok(t),
            None => Err(OofBuilder::new().with_tag_if::<Tag, _>(f)),
        }
    }

    #[cfg_attr(feature = "location", track_caller)]
    fn _tag_manually<F: FnOnce(&Self::Error, &mut Tags)>(
        self,
        f: F,
    ) -> Result<Self::Return, OofBuilder<Self::Error>> {
        match self {
            Some(t) => Ok(t),
            None => Err(OofBuilder::new().with_tag_manually(f)),
        }
    }

    #[cfg_attr(feature = "location", track_caller)]
    fn _attach<D: fmt::Debug>(self, debuggable: D) -> Result<T, OofBuilder> {
        match self {
            Some(t) => Ok(t),
            None => Err(OofBuilder::new().with_attachment(debuggable)),
        }
    }

    #[cfg_attr(feature = "location", track_caller)]
    fn _attach_lazy<D: ToString, F: FnOnce() -> D>(self, f: F) -> Result<T, OofBuilder> {
        match self {
            Some(t) => Ok(t),
            None => Err(OofBuilder::new().with_attachment_lazy(f)),
        }
    }
}
