use core::fmt;

pub trait __VarCheck {
    type Target;

    fn target(&self) -> &Self::Target;

    fn impls_copy(&self) -> bool {
        false
    }

    fn try_debug_fmt(&self) -> Option<String> {
        None
    }

    fn try_lazy<F>(&self, should_exec: bool, f: F) -> __InstantExecute
    where
        F: FnOnce(&Self) -> Option<String>,
    {
        __InstantExecute(should_exec.then(|| f(self)).flatten())
    }
}
impl<T> __VarCheck for __VarWrapper<T> {
    type Target = T;

    fn target(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct __VarWrapper<T>(pub T);
impl<T> __VarWrapper<T> {
    pub fn unload(self) -> T {
        self.0
    }
}
impl<T: fmt::Debug> __VarWrapper<T> {
    pub fn try_debug_fmt(&self) -> Option<String> {
        Some(format!("{:?}", self.0))
    }
}
impl<T: Copy> __VarWrapper<T> {
    pub fn target(self) -> T {
        self.0
    }

    pub fn impls_copy(&self) -> bool {
        true
    }

    pub fn try_lazy<F>(&self, should_exec: bool, f: F) -> __LazyExecute<Self, F>
    where
        F: FnOnce(Self) -> Option<String>,
    {
        __LazyExecute(*self, should_exec.then_some(f))
    }
}

#[derive(Debug, Clone)]
pub struct __InstantExecute(Option<String>);
impl __InstantExecute {
    pub fn exec(self) -> Option<String> {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct __LazyExecute<T, F>(T, Option<F>);
impl<T, F> __LazyExecute<T, F>
where
    F: FnOnce(T) -> Option<String>,
{
    pub fn exec(self) -> Option<String> {
        let Self(arg, f) = self;
        f.map(|f| f(arg)).flatten()
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
        let w = __VarWrapper(&x);
        let w_fn = w.try_lazy(true, |val| {
            fn_called.store(true, Ordering::Relaxed);

            val.try_debug_fmt()
        });

        // before `exec()`, fn should not be called for ref value.
        assert!(!fn_called.load(Ordering::Relaxed));
        let w_val = w_fn.exec();
        // Since `&String` implements `fmt::Debug`, it should output `Some(...)`.
        assert!(w_val.is_some());
        // after `exec()`, fn should have been executed.
        assert!(fn_called.load(Ordering::Relaxed));

        // reset `fn_called` to `false`.
        fn_called.store(false, Ordering::Relaxed);

        // we load an owned value to the bin.
        let z = __VarWrapper(x);
        let z_fn = z.try_lazy(true, |val| {
            fn_called.store(true, Ordering::Relaxed);

            val.try_debug_fmt()
        });
        // since fn is instantly called, `fn_called` should be set to `true`.
        assert!(fn_called.load(Ordering::Relaxed));
        let z_val = z_fn.exec();
        // Since `String` implements `fmt::Debug`, it should output `Some(...)`.
        assert!(z_val.is_some());

        // unload should unwrap the `__VarWrapper` wrapper.
        let z = z.unload();

        // Load the struct that does not implement `fmt::Debug`.
        let y = __VarWrapper(NoDebug(z));
        let y_fn = y.try_lazy(true, |val| val.try_debug_fmt());

        // since the value does not implement `fmt::Debug`, it should return `None`.
        assert!(y_fn.exec().is_none());
    }

    #[test]
    fn test_generic_fn() {
        fn generic_debug<T: fmt::Debug>(t: T) {
            let bin = __VarWrapper(t);
            let val = bin.try_lazy(true, |v| v.try_debug_fmt()).exec();

            assert!(val.is_some());
        }

        fn generic_no_debug<T>(t: T) {
            let bin = __VarWrapper(t);
            let val = bin.try_lazy(true, |v| v.try_debug_fmt()).exec();

            assert!(val.is_none());
        }

        generic_debug(5u64);
        generic_debug(&5u64);
        generic_no_debug(5u64);
        generic_no_debug(&5u64);
    }
}
