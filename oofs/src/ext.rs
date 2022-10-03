use crate::{OofBuilder, OofMessage};
use std::error::Error;

pub trait OofExt<T>: Sized {
    fn tag<Tag: 'static>(self) -> Result<T, OofBuilder>;
    fn tag_if<Tag: 'static, F: FnOnce(&Box<dyn 'static + Send + Sync + Error>) -> bool>(
        self,
        f: F,
    ) -> Result<T, OofBuilder>;
    fn display_owned(self) -> Result<T, OofBuilder>;
    fn add_context<F: FnOnce() -> String>(self, context_fn: F) -> Result<T, OofBuilder>;
}

impl<T, E> OofExt<T> for Result<T, E>
where
    E: 'static + Send + Sync + Error,
{
    fn tag<Tag: 'static>(self) -> Result<T, OofBuilder> {
        panic!(".tag() should not be called without attribute #[oofs::oof]")
    }

    fn tag_if<Tag: 'static, F: FnOnce(&Box<dyn 'static + Send + Sync + Error>) -> bool>(
        self,
        _f: F,
    ) -> Result<T, OofBuilder> {
        panic!(".tag_if(...) should not be called without attribute #[oofs::oof]")
    }

    fn display_owned(self) -> Result<T, OofBuilder> {
        panic!(".display_owned() should not be called without attribute #[oofs::oof]");
    }

    fn add_context<F: FnOnce() -> String>(self, _context_fn: F) -> Result<T, OofBuilder> {
        panic!(".add_context() should not be called without attribute #[oofs::oof]")
    }
}

impl<T> OofExt<T> for Option<T> {
    fn tag<Tag: 'static>(self) -> Result<T, OofBuilder> {
        panic!(".tag() should not be called without attribute #[oofs::oof]")
    }

    fn tag_if<Tag: 'static, F: FnOnce(&Box<dyn 'static + Send + Sync + Error>) -> bool>(
        self,
        _f: F,
    ) -> Result<T, OofBuilder> {
        panic!(".tag_if(...) should not be called without attribute #[oofs::oof]")
    }

    fn display_owned(self) -> Result<T, OofBuilder> {
        panic!(".display_owned() should not be called without attribute #[oofs::oof]")
    }

    fn add_context<F: FnOnce() -> String>(self, _context_fn: F) -> Result<T, OofBuilder> {
        panic!(".add_context() should not be called without attribute #[oofs::oof]")
    }
}
