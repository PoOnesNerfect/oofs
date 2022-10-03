use crate::RefType;
use core::fmt;

pub trait __TsaCheck {
    fn __ref_type(&self) -> RefType {
        RefType::Owned
    }

    fn __try_debug(&self) -> Option<String> {
        None
    }

    fn __try_lazy_fn<F>(&self, display_owned: bool, f: F) -> __InstantDisplayFn
    where
        F: FnOnce(&Self) -> Option<String>,
    {
        __InstantDisplayFn(display_owned.then(|| f(self)).flatten())
    }
}
impl<T> __TsaCheck for __TsaBin<T> {}

#[derive(Debug, Clone, Copy)]
pub struct __TsaBin<T>(pub T);
impl<T> __TsaBin<T> {
    pub fn __tsa_unload(self) -> T {
        self.0
    }
}
impl<T: fmt::Debug> __TsaBin<T> {
    pub fn __try_debug(&self) -> Option<String> {
        Some(format!("{:?}", self.0))
    }
}
impl<T> __TsaBin<&mut T> {
    pub fn __ref_type(&self) -> RefType {
        RefType::RefMut
    }
}
impl<T> __TsaBin<&T> {
    pub fn __ref_type(&self) -> RefType {
        RefType::Ref
    }
}
impl<T: Copy> __TsaBin<T> {
    pub fn __try_lazy_fn<F>(&self, _display_owned: bool, f: F) -> __LazyDisplayFn<Self, F>
    where
        F: FnOnce(Self) -> Option<String>,
    {
        __LazyDisplayFn(*self, Some(f))
    }
}

#[derive(Debug, Clone)]
pub struct __InstantDisplayFn(Option<String>);
impl __InstantDisplayFn {
    pub fn call(self) -> Option<String> {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct __LazyDisplayFn<T, F>(T, Option<F>);
impl<T, F> __LazyDisplayFn<T, F>
where
    F: FnOnce(T) -> Option<String>,
{
    pub fn call(mut self) -> Option<String> {
        let f = self.1.take().expect("Fn should exist");
        f(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    pub struct NoDebug(String);

    #[test]
    fn test_debug_and_ref() {
        let x = "hello world".to_owned();
        let fn_called = AtomicBool::from(false);

        // we load a reference to the bin.
        let w = __TsaBin(&x);
        let w_fn = w.__try_lazy_fn(true, |val| {
            fn_called.store(true, Ordering::Relaxed);

            val.__try_debug()
        });

        // before `call()`, fn should not be called for ref value.
        assert!(!fn_called.load(Ordering::Relaxed));
        let w_val = w_fn.call();
        // Since `&String` implements `fmt::Debug`, it should output `Some(...)`.
        assert!(w_val.is_some());
        // after `call()`, fn should have been executed.
        assert!(fn_called.load(Ordering::Relaxed));

        // reset `fn_called` to `false`.
        fn_called.store(false, Ordering::Relaxed);

        // we load an owned value to the bin.
        let z = __TsaBin(x);
        let z_fn = z.__try_lazy_fn(true, |val| {
            fn_called.store(true, Ordering::Relaxed);

            val.__try_debug()
        });
        // since fn is instantly called, `fn_called` should be set to `true`.
        assert!(fn_called.load(Ordering::Relaxed));
        let z_val = z_fn.call();
        // Since `String` implements `fmt::Debug`, it should output `Some(...)`.
        assert!(z_val.is_some());

        // unload should unwrap the `__TsaBin` wrapper.
        let z = z.__tsa_unload();

        // Load the struct that does not implement `fmt::Debug`.
        let y = __TsaBin(NoDebug(z));
        let y_fn = y.__try_lazy_fn(true, |val| val.__try_debug());

        // since the value does not implement `fmt::Debug`, it should return `None`.
        assert!(y_fn.call().is_none());
    }

    #[test]
    fn test_generic_fn() {
        fn generic_debug<T: fmt::Debug>(t: T) {
            let bin = __TsaBin(t);
            let val = bin.__try_lazy_fn(true, |v| v.__try_debug()).call();

            assert!(val.is_some());
        }

        fn generic_no_debug<T>(t: T) {
            let bin = __TsaBin(t);
            let val = bin.__try_lazy_fn(true, |v| v.__try_debug()).call();

            assert!(val.is_none());
        }

        generic_debug(5u64);
        generic_debug(&5u64);
        generic_no_debug(5u64);
        generic_no_debug(&5u64);
    }
}
