use crate::{Location, Oof, OofExt, OofMessage};
use std::{any::TypeId, collections::HashSet, error::Error};

#[derive(Debug)]
pub struct OofBuilder {
    message: OofMessage,
    source: Option<Box<dyn 'static + Send + Sync + Error>>,
    context: Option<String>,
    location: Option<Location>,
    tags: Option<HashSet<TypeId>>,
}

impl OofBuilder {
    #[track_caller]
    pub fn new(message: OofMessage) -> OofBuilder {
        Self {
            message,
            source: None,
            context: None,
            location: Some(Location::caller()),
            tags: None,
        }
    }

    pub fn with_source<E: 'static + Send + Sync + Error>(mut self, source: E) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    pub fn build(self) -> Oof {
        Oof {
            source: self.source,
            message: self.message,
            context: self.context,
            location: self.location,
            tags: self.tags.unwrap_or_default(),
        }
    }
}

impl<T> OofExt<T> for Result<T, OofBuilder> {
    fn tag<Tag: 'static>(self) -> Result<T, OofBuilder> {
        match self {
            Ok(ret) => Ok(ret),
            Err(mut builder) => {
                if let Some(tags) = &mut builder.tags {
                    tags.insert(TypeId::of::<Tag>());
                } else {
                    let mut tags = HashSet::new();
                    tags.insert(TypeId::of::<Tag>());

                    builder.tags.replace(tags);
                }

                Err(builder)
            }
        }
    }

    fn tag_if<Tag: 'static, F: FnOnce(&Box<dyn 'static + Send + Sync + Error>) -> bool>(
        self,
        f: F,
    ) -> Result<T, OofBuilder> {
        match self {
            Ok(t) => Ok(t),
            Err(b) => {
                if let Some(source) = &b.source {
                    if f(&source) {
                        return Err(b).tag::<Tag>();
                    }
                }

                Err(b)
            }
        }
    }

    fn display_owned(self) -> Result<T, OofBuilder> {
        self
    }

    fn add_context<F: FnOnce() -> String>(self, context_fn: F) -> Result<T, OofBuilder> {
        match self {
            Ok(ret) => Ok(ret),
            Err(mut builder) => {
                builder.context.replace(context_fn());
                Err(builder)
            }
        }
    }
}

pub trait OofBuilderExt<T> {
    fn with_oof_builder<F: FnOnce() -> OofBuilder>(self, f: F) -> Result<T, OofBuilder>;
}
impl<T> OofBuilderExt<T> for Result<T, OofBuilder> {
    fn with_oof_builder<F: FnOnce() -> OofBuilder>(self, f: F) -> Result<T, OofBuilder> {
        match self {
            Ok(t) => Ok(t),
            Err(b) => {
                let mut builder = f();
                builder.source = b.source;
                Err(builder)
            }
        }
    }
}
impl<T, E> OofBuilderExt<T> for Result<T, E>
where
    E: 'static + Send + Sync + Error,
{
    fn with_oof_builder<F: FnOnce() -> OofBuilder>(self, f: F) -> Result<T, OofBuilder> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(f().with_source(e)),
        }
    }
}
impl<T> OofBuilderExt<T> for Option<T> {
    fn with_oof_builder<F: FnOnce() -> OofBuilder>(self, f: F) -> Result<T, OofBuilder> {
        if let Some(t) = self {
            Ok(t)
        } else {
            let mut b = f();
            b.message.returns_option();
            Err(b)
        }
    }
}

#[cfg(test)]
mod oof_builder_tests {
    use super::*;

    pub struct MyTag;

    #[test]
    fn test_oof_builder() {
        let err = valid_err_fn().unwrap_err();

        assert!(err.tagged::<MyTag>());
        assert!(err.to_string().starts_with(&format!("parsing failed")));
    }

    fn valid_err_fn() -> Result<(), Oof> {
        "hello world"
            .parse::<u8>()
            .with_oof_builder(|| OofBuilder::new("parsing failed".into()))
            .tag::<MyTag>()
            .map_err(|e| e.build())?;

        Ok(())
    }
}
