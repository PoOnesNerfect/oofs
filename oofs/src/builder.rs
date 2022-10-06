use crate::{
    context::{Context, OofGeneratedContext},
    Oof, OofExt,
};
use core::{any::TypeId, fmt};
use std::{collections::HashSet, error::Error};

#[cfg(feature = "location")]
use crate::Location;

#[derive(Debug)]
pub struct OofBuilder {
    context: Context,
    source: Option<Box<dyn 'static + Send + Sync + Error>>,
    #[cfg(feature = "location")]
    location: Location,
    tags: HashSet<TypeId>,
    attachments: Vec<String>,
}

impl OofBuilder {
    #[cfg_attr(feature = "location", track_caller)]
    pub(crate) fn new() -> OofBuilder {
        Self {
            context: Context::default(),
            source: None,
            #[cfg(feature = "location")]
            location: Location::caller(),
            tags: HashSet::new(),
            attachments: Vec::new(),
        }
    }

    pub(crate) fn with_generated(mut self, context: OofGeneratedContext) -> OofBuilder {
        self.context = context.into();
        self
    }

    pub(crate) fn with_custom(mut self, custom: String) -> OofBuilder {
        self.context = Context::Custom(custom);
        self
    }

    pub(crate) fn with_source<E: 'static + Send + Sync + Error>(mut self, source: E) -> Self {
        self.source.replace(Box::new(source));
        self
    }

    pub(crate) fn with_tag<T: 'static>(mut self) -> Self {
        self.tags.insert(TypeId::of::<T>());
        self
    }

    pub(crate) fn with_tag_if<T, F>(self, f: F) -> Self
    where
        T: 'static,
        F: FnOnce(&Box<dyn 'static + Send + Sync + Error>) -> bool,
    {
        if let Some(source) = &self.source {
            if f(&source) {
                return self.with_tag::<T>();
            }
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
            source: self.source,
            context: Box::new(self.context),
            #[cfg(feature = "location")]
            location: self.location,
            tags: self.tags,
            attachments: self.attachments,
        }
    }
}

impl<T> OofExt<T> for Result<T, OofBuilder> {
    fn _tag<Tag: 'static>(self) -> Result<T, OofBuilder> {
        self.map_err(|b| b.with_tag::<Tag>())
    }

    fn _tag_if<Tag, F>(self, f: F) -> Result<T, OofBuilder>
    where
        Tag: 'static,
        F: FnOnce(&Box<dyn 'static + Send + Sync + Error>) -> bool,
    {
        self.map_err(|b| b.with_tag_if::<Tag, _>(f))
    }

    fn _attach<D: fmt::Debug>(self, debuggable: D) -> Result<T, OofBuilder> {
        self.map_err(|b| b.with_attachment(debuggable))
    }

    fn _attach_lazy<D: ToString, F: FnOnce() -> D>(self, f: F) -> Result<T, OofBuilder> {
        self.map_err(|b| b.with_attachment_lazy(f))
    }
}

pub trait OofGenerator<T> {
    fn build_oof<F: FnOnce() -> OofGeneratedContext>(this: Self, f: F) -> Result<T, Oof>;
}

impl<T> OofGenerator<T> for Result<T, OofBuilder> {
    fn build_oof<F: FnOnce() -> OofGeneratedContext>(this: Self, f: F) -> Result<T, Oof> {
        match this {
            Ok(t) => Ok(t),
            Err(b) => Err(b.with_generated(f()).build()),
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
            Err(e) => Err(OofBuilder::new().with_source(e).with_generated(f()).build()),
        }
    }
}

impl<T> OofGenerator<T> for Option<T> {
    #[cfg_attr(feature = "location", track_caller)]
    fn build_oof<F: FnOnce() -> OofGeneratedContext>(this: Self, f: F) -> Result<T, Oof> {
        match this {
            Some(t) => Ok(t),
            None => Err(OofBuilder::new().with_generated(f()).build()),
        }
    }
}
