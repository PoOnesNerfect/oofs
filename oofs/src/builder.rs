use crate::{
    context::{Context, OofGeneratedContext},
    tags::Tags,
    Oof, OofExt,
};
use core::fmt;
use std::{convert::Infallible, error::Error};

#[cfg(feature = "location")]
use crate::Location;

#[derive(Debug)]
pub struct OofBuilder<E: 'static + Send + Sync + Error = Infallible> {
    context: Context,
    source: Option<E>,
    tags: Tags,
    attachments: Vec<String>,
    #[cfg(feature = "location")]
    location: Location,
}

impl OofBuilder {
    #[cfg_attr(feature = "location", track_caller)]
    pub(crate) fn new() -> Self {
        Self {
            context: Context::default(),
            source: None,
            #[cfg(feature = "location")]
            location: Location::caller(),
            tags: Tags::new(),
            attachments: Vec::new(),
        }
    }

    pub(crate) fn with_source<E>(self, source: E) -> OofBuilder<E>
    where
        E: 'static + Send + Sync + Error,
    {
        let Self {
            context,
            tags,
            attachments,
            location,
            ..
        } = self;

        OofBuilder {
            source: Some(source),
            context,
            tags,
            attachments,
            #[cfg(feature = "location")]
            location,
        }
    }
}

impl<E> OofBuilder<E>
where
    E: 'static + Send + Sync + Error,
{
    pub(crate) fn with_generated(mut self, context: OofGeneratedContext) -> Self {
        self.context = context.into();
        self
    }

    pub(crate) fn with_custom<D: ToString>(mut self, custom: D) -> Self {
        self.context = Context::Custom(custom.to_string());
        self
    }

    pub(crate) fn with_tag<T: 'static>(mut self) -> Self {
        self.tags.tag::<T>();
        self
    }

    pub(crate) fn with_tag_if<T, F>(self, f: F) -> Self
    where
        T: 'static,
        F: FnOnce(&E) -> bool,
    {
        if let Some(source) = &self.source {
            if f(&source) {
                return self.with_tag::<T>();
            }
        }

        self
    }

    pub(crate) fn with_tag_manually<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&E, &mut Tags),
    {
        if let Some(source) = &self.source {
            f(&source, &mut self.tags);
        }

        self
    }

    pub(crate) fn with_attachment<D: fmt::Debug>(mut self, debuggable: D) -> Self {
        self.attachments.push(format!("{debuggable:?}"));
        self
    }

    pub(crate) fn with_attachment_lazy<D: ToString, F: FnOnce() -> D>(mut self, f: F) -> Self {
        self.attachments.push(f().to_string());
        self
    }

    pub(crate) fn build(self) -> Oof {
        Oof {
            source: self.source.map(Into::into),
            context: Box::new(self.context),
            #[cfg(feature = "location")]
            location: self.location,
            tags: self.tags,
            attachments: self.attachments,
        }
    }
}

impl<T, E> OofExt for Result<T, OofBuilder<E>>
where
    E: 'static + Send + Sync + Error,
{
    type Return = T;
    type Error = E;

    fn _context<D: ToString>(self, context: D) -> Result<T, OofBuilder<E>> {
        self.map_err(|b| b.with_custom(context))
    }

    fn _tag<Tag: 'static>(self) -> Result<T, OofBuilder<E>> {
        self.map_err(|b| b.with_tag::<Tag>())
    }

    fn _tag_if<Tag, F>(self, f: F) -> Result<T, OofBuilder<E>>
    where
        Tag: 'static,
        F: FnOnce(&Self::Error) -> bool,
    {
        self.map_err(|b| b.with_tag_if::<Tag, _>(f))
    }

    fn _tag_manually<F: FnOnce(&Self::Error, &mut Tags)>(
        self,
        f: F,
    ) -> Result<Self::Return, OofBuilder<Self::Error>> {
        self.map_err(|b| b.with_tag_manually(f))
    }

    fn _attach<D: fmt::Debug>(self, debuggable: D) -> Result<T, OofBuilder<E>> {
        self.map_err(|b| b.with_attachment(debuggable))
    }

    fn _attach_lazy<D: ToString, F: FnOnce() -> D>(self, f: F) -> Result<T, OofBuilder<E>> {
        self.map_err(|b| b.with_attachment_lazy(f))
    }
}

pub trait OofGenerator<T> {
    fn build_oof<F: FnOnce() -> OofGeneratedContext>(this: Self, f: F) -> Result<T, Oof>;
}

impl<T, E> OofGenerator<T> for Result<T, OofBuilder<E>>
where
    E: 'static + Send + Sync + Error,
{
    fn build_oof<F: FnOnce() -> OofGeneratedContext>(this: Self, f: F) -> Result<T, Oof> {
        match this {
            Ok(t) => Ok(t),
            Err(mut b) => {
                if b.context.is_none() {
                    b = b.with_generated(f());
                }

                Err(b.build())
            }
        }
    }
}

impl<T, E> OofGenerator<T> for Result<T, E>
where
    E: 'static + Send + Sync + Error,
{
    #[cfg_attr(feature = "location", track_caller)]
    fn build_oof<F: FnOnce() -> OofGeneratedContext>(this: Self, f: F) -> Result<T, Oof> {
        match this {
            Ok(t) => Ok(t),
            Err(e) => {
                let mut b = OofBuilder::new().with_source(e);

                if b.context.is_none() {
                    b = b.with_generated(f());
                }

                Err(b.build())
            }
        }
    }
}

impl<T> OofGenerator<T> for Option<T> {
    #[cfg_attr(feature = "location", track_caller)]
    fn build_oof<F: FnOnce() -> OofGeneratedContext>(this: Self, f: F) -> Result<T, Oof> {
        match this {
            Some(t) => Ok(t),
            None => {
                let mut b = OofBuilder::new();

                if b.context.is_none() {
                    b = b.with_generated(f());
                }

                Err(b.build())
            }
        }
    }
}
