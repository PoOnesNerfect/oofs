use proc_macro_error::proc_macro_error;
use quote::ToTokens;
use syn::parse_macro_input;

mod implementation;

/// Place above `fn` or `impl` to generate and inject context to `?` operators.
///
/// ## Attribute arguments
/// - [skip](#skip)
/// - [closures](#closures)
/// - [async_blocks](#async_blocks)
/// - [tags](#tags)
/// - [attach](#attach)
/// - [attach_lazy](#attach_lazy)
///
/// ### Notes
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
///   // inject context to all closures in every method returning `Result<_, _>`.
///   #[oofs(closures)]
///   impl Foo {
///       // tag `RetryTag` to all `?` operators in this method.
///       #[oofs(tags(RetryTag))]
///       fn method(&self) -> Result<usize, Oof> {
///           // ...
///           # Ok(0)
///       }
///   }
///   ```
/// - Multiple arguments can be grouped together in a single attribute, separated by comma.
///
///   ```rust
///   use oofs::{oofs, Oof, OofExt};
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
///   #[oofs(closures, tags(RetryTag))]
///   impl Foo {
///       fn method(&self) -> Result<usize, Oof> {
///           // ...
///           # Ok(0)
///       }
///   }
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
///   #[oofs(tags(RetryTag))]
///   #[oofs(attach(123, x, "hello world"))]
///   impl Foo {
///       fn method(&self) -> Result<usize, Oof> {
///           // ...
///           # Ok(0)
///       }
///   }
///   ```
///
/// ## Default Behaviors
/// There are some **default behaviors** this attribute chooses to make:
/// 1. for `impl` blocks, methods that return `Result<_, _>` will have context injected.
/// 2. for `impl` blocks, methods that do not return `Result<_, _>` will be skipped.
/// 3. `?` operators inside closures (i.e. `|| { ... }`) will not have context injected.
/// 4. `?` operators inside async blocks (i.e. `async { ... }`) will not have context injected.
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
/// Behaviors 1 - 4 can be changed by specifying arguments to the attribute like `#[oofs(...)]`.
///
/// ## skip
///
/// `#[oofs(skip)]`
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
///
/// #[oofs]
/// impl Foo {
///     // context is not injected, as `skip` argument is passed.
///     #[oofs(skip)]
///     fn method(&self) -> Result<usize, Oof> {
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
/// `#[oofs(closures)]`
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
/// `#[oofs(async_blocks)]`
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
///             // context is not injected here
///             some_async_fn().await?;
///
///             Ok(())
///         }
///     }
/// }
/// ```
///
/// ## tags
///
/// `#[oofs(tags(ThisType, ThatType))]`
///
/// This argument tags specified types into all `?` operators.
///
/// Ex)
/// ```rust
/// use oofs::{oofs, Oof, OofExt};
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
/// #[oofs]
/// impl Foo {
///     #[oofs(tags(ThisType, ThatType))]
///     fn method(&self) -> Result<usize, Oof> {
///         // `ThisType` and `ThatType` are tagged
///         some_fn()?;
///
///         // `ThisType` and `ThatType` are tagged
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
/// use oofs::{oofs, Oof, OofExt};
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
///     #[oofs(attach(123, x, "hello world"))]
///     fn method(&self) -> Result<usize, Oof> {
///         let x = "some context";
///
///         // 123, x, and "hello world" are attached
///         some_fn()?;
///
///         // 123, x, and "hello world" are attached
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
/// use oofs::{oofs, Oof, OofExt};
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
