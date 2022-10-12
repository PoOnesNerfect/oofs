use proc_macro_error::proc_macro_error;
use quote::ToTokens;
use syn::parse_macro_input;

mod implementation;

/// Place above `fn` or `impl` to generate and inject context to `?` operators.
///
/// ## Attribute arguments
///
/// These are the available arguments for the attribute; click to see details on each argument.
///
/// - [tag](#tag)
/// - [attach](#attach)
/// - [attach_lazy](#attach_lazy)
/// - [skip](#skip)
/// - [closures](#closures)
/// - [async_blocks](#async_blocks)
/// - [debug_skip](#debug_skip)
/// - [debug_with](#debug_with)
/// - [debug_non_copyable](#debug_non_copyable)
///
/// ## Default Behaviors
///
/// There are some **default behaviors** this attribute chooses to make:
/// 1. for `impl` blocks, all methods that return `Result` (i.e. `fn method(...) -> Result<_, _>`) will have context injected.
///     - override this behavior by specifying `#[oofs(skip)]` above `fn` to have that specific method skipped.
/// 2. for `impl` blocks, methods that do not return `Result<_, _>` will be skipped.
///     - override this behavior by specifying `#[oofs]` above `fn` to apply injection regardless.
/// 3. `?` operators inside closures (i.e. `|| { ... }`) will not have context injected.
///     - override this behavior by specifying `#[oofs(closures)]` above `fn` to apply injections to inside closures.
/// 4. `?` operators inside async blocks (i.e. `async { ... }`) will not have context injected.
///     - override this behavior by specifying `#[oofs(async_blocks)]` above `fn` to apply injections to inside async blocks.
/// 5. `return ...` statements and last expression without semicolon will not have context injected.
///
/// Below is an example showing each of listed default bahaviours.
///
/// Ex)
/// ```rust
/// use oofs::{oofs, Oof};
///
/// pub struct Foo {
///     field: usize
/// }
/// # async fn some_async_fn() -> Result<(), Oof> { todo!() }
///
/// #[oofs]
/// impl Foo {
///     // context is injected into this method.
///     fn method(&self) -> Result<usize, Oof> {
///         // context injected here
///         some_fn()?;
///
///         // context not injected inside the closure
///         let c = |x: &str| {
///             // context not injected here
///             x.parse::<usize>()?;
///
///             Ok::<_, std::num::ParseIntError>(())
///         };
///
///         // context is injected here
///         c("hello world")?;
///
///         // ...
///         # Ok(0)
///     }
///
///     // context is injected into async method as well.
///     async fn async_method(&self) -> Result<usize, Oof> {
///         // context injected here
///         some_async_fn().await?;
///
///         // context not injected inside the async block
///         let a = async {
///             // context not injected here
///             some_async_fn().await?;
///
///             Ok::<_, Oof>(())
///         }.await?; // <- context injected here
///
///         // ...
///         # Ok(0)
///     }
///
///     // context is not injected.
///     fn returns_option(&self) -> Option<usize> {
///         // ...
///         # None
///     }
///
///     // context is not injected.
///     fn another_method(&self) -> usize {
///         // ...
///         # 0
///     }
/// }
///
/// #[oofs]
/// fn some_fn() -> Result<usize, Oof> {
///     // ...
///     # Ok(0)
/// }
/// ```
///
/// ### Notes
///
/// - Attribute arguments can be applied to the entire `impl` block, or to individual `fn` methods.
///
///   ```rust
///   use oofs::{oofs, Oof};
///
///   struct RetryTag;
///
///   pub struct Foo {
///       field: usize
///   }
///   # async fn some_async_fn() -> Result<(), Oof> { todo!() }
///
///   // tag `RetryTag` to all `?` operators in every method returning `Result<_, _>`.
///   #[oofs(tag(RetryTag))]
///   impl Foo {
///       // inject context to inside closures.
///       #[oofs(closures)]
///       fn method(&self) -> Result<usize, Oof> {
///           // ...
///           # Ok(0)
///       }
///   }
///   ```
///
/// - Multiple arguments can be grouped together in a single attribute, separated by comma.
///
///   ```rust
///   use oofs::{oofs, Oof};
///
///   struct RetryTag;
///
///   pub struct Foo {
///       field: usize
///   }
///   # async fn some_async_fn() -> Result<(), Oof> { todo!() }
///
///   // inject context to inside closures in every method returning `Result<_, _>`.
///   // inject context to inside async blocks in every method returning `Result<_, _>`.
///   // also, tag `RetryTag` to all `?` in every method returning `Result<_, _>`.
///   #[oofs(closures, async_blocks, tag(RetryTag))]
///   impl Foo {
///       fn method(&self) -> Result<usize, Oof> {
///           // ...
///           # Ok(0)
///       }
///   }
///
/// - You can specify multiple arguments in separate lines.
///
///   ```rust
///   use oofs::{oofs, Oof};
///
///   struct RetryTag;
///
///   pub struct Foo {
///       field: usize
///   }
///   # async fn some_async_fn() -> Result<(), Oof> { todo!() }
///
///   // inject context to all closures in every method returning `Result<_, _>`.
///   // also, tag `RetryTag` to all `?` in every method returning `Result<_, _>`.
///   // also, attaches `123`, `x`, and `"hello world"` to all `?` in every method returning `Result<_, _>`.
///   #[oofs(closures)]
///   #[oofs(tag(RetryTag))]
///   #[oofs(attach(123, x, "hello world"))]
///   impl Foo {
///       fn method(&self) -> Result<usize, Oof> {
///           // ...
///           # Ok(0)
///       }
///   }
///   ```
///
/// ## tag
///
/// `#[oofs(tag(ThisType, ThatType))]`
///
/// This argument tags specified types into all `?` operators.
///
/// Ex)
/// ```rust
/// use oofs::{oofs, Oof};
/// use std::future::Future;
///
/// struct ThisType;
/// struct ThatType;
///
/// pub struct Foo {
///     field: usize
/// }
/// # fn some_fn() -> Result<(), Oof> { todo!() }
/// # fn another_fn() -> Result<(), Oof> { todo!() }
///
/// // tag `Foo` for all methods of `Foo`.
/// #[oofs(tag(Foo))]
/// impl Foo {
///     #[oofs(tag(ThisType, ThatType))]
///     fn method(&self) -> Result<usize, Oof> {
///         // `Foo`, `ThisType` and `ThatType` are tagged
///         some_fn()?;
///
///         // `Foo`, `ThisType` and `ThatType` are tagged
///         another_fn()?;
///
///         // ...
///         # Ok(0)
///     }
/// }
/// ```
///
/// ## attach
///
/// `#[oofs(attach(123, x, "hello world"))]`
///
/// This argument attaches specified contexts into all `?` operators.
///
/// Ex)
/// ```rust
/// use oofs::{oofs, Oof};
/// use std::future::Future;
///
/// pub struct Foo {
///     field: usize
/// }
/// # fn some_fn() -> Result<(), Oof> { todo!() }
/// # fn another_fn() -> Result<(), Oof> { todo!() }
///
/// #[oofs(attach("This is Foo"))]
/// impl Foo {
///     #[oofs(attach(123, x, "hello world"))]
///     fn method(&self) -> Result<usize, Oof> {
///         let x = "some context";
///
///         // "This is Foo", 123, x, and "hello world" are attached
///         some_fn()?;
///
///         // "This is Foo", 123, x, and "hello world" are attached
///         another_fn()?;
///
///         // ...
///         # Ok(0)
///     }
/// }
/// ```
///
/// ## attach_lazy
///
/// `#[oofs(attach_lazy(|| 123, || "hello world"))]`
///
/// This argument lazily loads and attaches specified contexts into all `?` operators.
///
/// Ex)
/// ```rust
/// use oofs::{oofs, Oof};
/// use std::future::Future;
///
/// pub struct Foo {
///     field: usize
/// }
/// # fn some_fn() -> Result<(), Oof> { todo!() }
/// # fn another_fn() -> Result<(), Oof> { todo!() }
///
/// #[oofs]
/// impl Foo {
///     #[oofs(attach_lazy(|| 123, || x, || "hello world"))]
///     fn method(&self) -> Result<usize, Oof> {
///         let x = "some context";
///
///         // 123, x, and "hello world" are lazily attached
///         some_fn()?;
///
///         // 123, x, and "hello world" are lazily attached
///         another_fn()?;
///
///         // ...
///         # Ok(0)
///     }
/// }
/// ```
///
/// ## skip
///
/// `#[oofs(skip)]` or `#[skip(true)]`
///
/// `#[oofs(skip(false))]` will unskip (apply) injection, if already enabled from outer scope.
///
/// This argument skips context injection for a method in an `impl` block.
///
/// Ex)
/// ```rust
/// use oofs::{oofs, Oof};
/// use std::future::Future;
///
/// pub struct Foo {
///     field: usize
/// }
///
/// # async fn some_async_fn() -> Result<(), Oof> { todo!() }
/// # struct CustomError;
///
/// #[oofs]
/// impl Foo {
///     // context is not injected, as `skip` argument is passed.
///     #[oofs(skip)]
///     fn method(&self) -> Result<usize, CustomError> {
///         // ...
///         # Ok(0)
///     }
///
///     // context will be injected, as `#[oofs]` argument is specified.
///     #[oofs]
///     fn another_method(&self) -> impl Future<Output = Result<(), Oof>> {
///         // ...
///         # async { Ok(()) }
///     }
/// }
/// ```
///
/// ## closures
///
/// `#[oofs(closures)]` or `#[oofs(closures(true))]`
///
/// `#[oofs(closures(false))]` will disable closures, if already enabled from outer scope.
///
/// This argument applies context injection to inside closures.
///
/// Ex)
/// ```rust
/// use oofs::{oofs, Oof};
/// use std::future::Future;
///
/// pub struct Foo {
///     field: usize
/// }
/// # async fn some_async_fn() -> Result<(), Oof> { todo!() }
///
/// #[oofs]
/// impl Foo {
///     #[oofs(closures)]
///     fn method(&self) -> Result<usize, Oof> {
///         // context is now inside the closure
///         let c = |x: &str| {
///             // context now injected here
///             x.parse::<usize>()?;
///
///             Ok::<_, Oof>(())
///         };
///
///         // context is injected here as well
///         c("hello world")?;
///         // ...
///         # Ok(0)
///     }
/// }
/// ```
///
/// ## async_blocks
///
/// `#[oofs(async_blocks)]` or `#[oofs(async_blocks(true))]`
///
/// `#[oofs(async_blocks(false))]` will disable injecting into async blocks, if already enabled from outer scope.
///
/// This argument applies context injection to inside async blocks.
///
/// Ex)
/// ```rust
/// use oofs::{oofs, Oof};
/// use std::future::Future;
///
/// pub struct Foo {
///     field: usize
/// }
/// # async fn some_async_fn() -> Result<(), Oof> { todo!() }
///
/// #[oofs]
/// impl Foo {
///     // context will be injected, as `#[oofs]` argument is specified.
///     // also, `?` operators in async blocks will have context injected.
///     #[oofs(async_blocks)]
///     fn another_method(&self) -> impl Future<Output = Result<(), Oof>> {
///         // ...
///         async {
///             // context is injected here
///             some_async_fn().await?;
///
///             Ok(())
///         }
///     }
/// }
/// ```
///
/// ## debug_skip
///
/// `#[oofs(debug_skip(&x))]`
///
/// Argument expressions supplied are skipped from debugging argument values.
///
/// You can supply multiple expressions separated by commas.
///
/// Note that the supplied expression must match exactly the one you want to skip debugging.
///
/// Ex)
/// ```rust
/// use oofs::{oofs, Oof};
/// use std::future::Future;
///
/// pub struct Foo {
///     field: usize
/// }
/// # fn some_fn(x: &str, n: usize, y: &str) -> Result<(), Oof> { todo!() }
///
/// #[oofs]
/// impl Foo {
///     // function arguments that match any of these expressions are ignored from being debugged.
///     #[oofs(debug_skip(&x, 123usize))]
///     fn another_method(&self) -> Result<(), Oof> {
///         let x = "hello world";
///
///         // `&x` and `123usize` are not debugged, but third arg `x` is still debugged.
///         some_fn(&x, 123usize, x)?;
///
///         // ...
///
///         Ok(())
///     }
/// }
/// ```
///
/// ## debug_with
///
/// `#[oofs(debug_with(&x -> serde_json::to_string($a).unwrap()))]`
///
/// Argument expressions supplied before `->` are debugged using the custom expression after `->`.
///
/// Expression before `->` must match exactly the argument you want to debug with custom method.
///
/// Expression after `->` must return an object/primitives that implements `ToString` (i.e. String, &str, usize, etc.).
/// If you want to supply the argument expression as argument to the custom debug expression, you must use `$a` to refer to it.
///
/// You can only supply ***one*** expression for better readability considerations.
///
/// Note that the supplied expression must match exactly the one you want to custom debug.
///
/// Ex)
/// ```rust
/// use oofs::{oofs, Oof};
/// use std::future::Future;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// pub struct Foo {
///     field: usize
/// }
/// # fn some_fn(x: &Foo, n: usize) -> Result<(), Oof> { todo!() }
///
/// #[oofs]
/// impl Foo {
///     #[oofs(debug_with(&x -> serde_json::to_string($a).unwrap()))]
///     fn another_method(&self) -> Result<(), Oof> {
///         let x = Foo {
///             field: 123
///         };
///
///         // `&x` will be displayed as `{ "field": 123 }`.
///         some_fn(&x, 123usize)?;
///
///         // ...
///
///         Ok(())
///     }
/// }
/// ```
///
/// ## debug_non_copyable
///
/// `#[oofs(debug_non_copyable(full))]`
///
/// `#[oofs(debug_non_copyable(disabled))]`
///
/// This argument takes either `full` or `disabled`.
///
/// Non-copyable arguments cannot have debug values lazily generated like references or copyable values like primitives.
///
/// Default behavior for debugging non-copyable values (String, custom objects, etc.) are:
/// - For debug mode, load debug formatted values before calling each function, incurring overhead at every call.
/// - For release mode, skip debugging non-copyable values.
///
/// You can use these arguments to change this default behavior:
/// - `full`: enable debugging copyable values for release mode. This will incur overhead of formatting debug values for every call.
/// - `disabled`: disable debugging non-copyable values even for debug mode.
///
/// If you want to set this setting for the entire library/binary, you can enable features either `debug_non_copyable_full` or `debug_non_copyable_disabled`.
///
/// Ex)
/// ```rust
/// use oofs::{oofs, Oof};
/// use std::future::Future;
///
/// pub struct Foo {
///     field: usize
/// }
/// # async fn some_async_fn() -> Result<(), Oof> { todo!() }
///
/// // for the entire `impl` block, disable debugging non-copyable values even fore debug mode.
/// #[oofs(debug_non_copyable(disabled))]
/// impl Foo {
///     // for this specific method, enable debugging non-copyable values even for release mode.
///     #[oofs(debug_non_copyable(full))]
///     fn another_method(&self) -> Result<(), Oof> {
///         // ...
///         # Ok(())
///     }
/// }
/// ```
///
#[proc_macro_attribute]
#[proc_macro_error]
pub fn oofs(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let oofs = parse_macro_input!(input as implementation::Oofs);
    let args = parse_macro_input!(args as implementation::PropArgs);
    oofs.with_args(args).to_token_stream().into()
}
