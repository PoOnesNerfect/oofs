# oofs

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/oofs.svg
[crates-url]: https://crates.io/crates/oofs
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/PoOnesNerfect/oofs/blob/main/LICENSE

**Error handling library that generates and injects context for you.**

This library provides three main features:

- `#[oofs]` attribute that generates and injects context to function calls with `?` operators.
- Tagging an error for categorized error handling.
- Attaching custom contexts.

**Table of Content:**

- [oofs](#oofs)
  - [Basic Example 1](#basic-example-1)
  - [Basic Example 2](#basic-example-2)
  - [Tagging Errors](#tagging-errors)
  - [Attaching Custom Contexts](#attaching-custom-contexts)
  - [Returning Custom Errors](#returning-custom-errors)
  - [Features](#features)
  - [Notes/Limitations About the Library](#noteslimitations-about-the-library)
    - [About `#[oofs]` Attribute](#about-oofs-attribute)
    - [About `Oof` Error Struct](#about-oof-error-struct)
    - [About Underscore Methods like `._tag()` and `._attach(_)`](#about-underscore-methods-like-_tag-and-_attach_)
    - [Debugging Non-Copyable Arguments](#debugging-non-copyable-arguments)
    - [Compatibility with `#[async_trait]`](#compatibility-with-async_trait)
  - [Future Plans](#future-plans)

## Basic Example 1

Below showcases the context injection.

```rust
use oofs::{oofs, Oof};

#[oofs]
fn outer_fn() -> Result<(), Oof> {
    let x = 123;
    let y = "hello world";

    inner_fn(x, y)?;

    Ok(())
}

#[oofs]
fn inner_fn(x: usize, y: &str) -> Result<(), Oof> {
    let _ = y.parse::<usize>()?;

    Ok(())
}
```

Printing error from `outer_fn()` outputs:

```
inner_fn($0, $1) failed at `oofs/tests/basic.rs:6:5`

Parameters:
    $0: usize = 123
    $1: &str = "hello world"

Caused by:
    0: y.parse() failed at `oofs/tests/basic.rs:17:13`

    1: invalid digit found in string
```

## Basic Example 2

Now, let's look at a slightly longer example.
Below is an example from `oofs/tests/basic.rs`.

The example showcases context-generation, tagging, and attaching custom contexts.

```rust
use oofs::{oofs, Oof, OofExt};

// Marker type used for tagging.
struct RetryTag;

#[oofs]
fn application() -> Result<(), Oof> {
    if let Err(e) = middlelayer("hello world") {
        // Check if any of internal errors is tagged as `RetryTag`; if so, try again.
        if e.tagged_nested::<RetryTag>() {
            println!("Retrying middlelayer!\n");

            // If the call fails again, return it.
            // Since `?` operator is used, context is generated and injected to the call.
            middlelayer("hello world")?;
        } else {
            return Err(e);
        }
    }

    Ok(())
}

#[oofs]
fn middlelayer(text: &str) -> Result<u64, Oof> {
    let my_struct = MyStruct {
        field: text.to_owned(),
    };

    // Passing an expression as arg is also fine.
    // All args are evaluated before being debugged in the error.
    // Context is generated and injected to both `?`s in this statement.
    let ret = my_struct.failing_method(get_value()?)?;

    Ok(ret)
}

fn get_value() -> Result<usize, std::io::Error> {
    Ok(123)
}

#[derive(Debug)]
struct MyStruct {
    field: String,
}

// #[oofs] can also be used to `impl` blocks.
// Context will be injected to all methods that return a `Result`.
#[oofs]
impl MyStruct {
    fn failing_method(&self, x: usize) -> Result<u64, Oof> {
        let ret = self
            .field
            .parse::<u64>()
            ._tag::<RetryTag>()                 // tags the error with the type `RetryTag`.
            ._attach(x)                         // attach anything that implements `Debug` as custom context.
            ._attach(&self.field)               // attach the receiver as attachment to debug.
            ._attach_lazy(|| "extra context")?; // lazily evaluate context; useful for something like `|| serde_json::to_string(&x)`.

        Ok(ret)
    }
}
```

If we print the error from `application()`, it would output:

```
Retrying middlelayer!

middlelayer($0) failed at `oofs/tests/basic.rs:11:13`

Parameters:
    $0: &str = "hello world"

Caused by:
    0: my_struct.failing_method($0) failed at `oofs/tests/basic.rs:26:15`

       Parameters:
           $0: usize = 123

    1: self.field.parse() failed at `oofs/tests/basic.rs:46:14`

       Attachments:
           0: 123
           1: "hello world"
           2: "extra context"

    2: invalid digit found in string
```

The error displays chain of methods that fail, their locations in code, the parameters' types and their debug values.
Same will be displayed when you format the error as Debug (i.e. `{:?}`).

Nice looking error is not all; we also get categorized error handling with tags.

At the source method `failing_method`, we tag the `parse` method with `RetryTag` type.
At the very top level function `application`, we call `e.tagged_nested::<RetryTag>` to check any interal calls were tagged with `RetryTag`.
When the tag is found, we handle the case by calling `middlelayer` again.

With tagging, we no longer have to go through every error variant at every level.
We just look for the tag we want to handle for, and we handle the tagged error accordingly.
In the above example, we retry calling `middlelayer` again if `RetryTag` tag is found.

## Tagging Errors

As shown in the example above, you can tag an error with `_tag` and detect the tag with `tagged` and `tagged_nested`.

```rust
struct MyTag;

#[oofs]
fn application_level() -> Result<(), Oof> {
    if let Err(e) = source() {
        if e.tagged_nested::<MyTag>() {
            ...handle for this tag
        } else if e.tagged_nested::<OtherTag>() {
            ...handle for this tag
        } else {
            ...
        }
    }
}

...

#[oofs]
fn source() -> Result<(), Oof> {
    some_fn()._tag::<MyTag>()?;

    Ok(())
}
```

This allows you to categorize errors into different tag groups, and handle for them accordingly.
This gives a much better experience when handling errors compared to matching every enum variant in every nested function calls.

Note that you can also tag an error with multiple different tags.

I chose type as tag because types are small, readable and unique. `String` or `usize` can lead to having duplicate values by accident.

## Attaching Custom Contexts

At some point, you may find the generated context is not enough.
After all, it just shows the call that failed, and parameters that were passed to it.
It will not capture all the other possibe context information.

You can attach your own context information to the error with `_attach` and `_attach_lazy` methods.

```rust
#[oofs]
fn outer_fn() -> Result<(), Oof> {
    let x = 123usize;
    let y = std::time::Instant::now();

    "hello world"
        .parse::<usize>()
        ._attach(&x)
        ._attach(&y)?;

    Ok(())
}
```

Above will print the following error:

```
$0.parse() failed at `oofs/tests/basic.rs:10:10`

Parameters:
    $0: &str = "hello world"

Attachments:
    0: 123
    1: Instant { t: 11234993365176 }

Caused by:
    invalid digit found in string
```

`_attach` takes any type that implements `std::fmt::Debug`.

`_attach_lazy`, on the other hand, takes any closure that returns a type that implements `ToString`.

It can be something `&str` like `._attach_lazy(|| "some context")`, `String` like `._attach_lazy(|| format!("some context {:?}", x))`,
or some function that requires some work to display like `._attach_lazy(|| serde_json::to_string(&x))`.

## Returning Custom Errors

At some point, you also want to return your custom error.

For these cases, you have some options: `oof!(...)`, `wrap_err(_)`, `ensure!(...)` and `ensure_eq!(...)`.

- `oof!(...)`: this is a lot like `anyhow!` or `eyre!`; you input to macro like you do for `println!`.
  This returns `Oof` struct, and you can call methods on the returned `Oof` like

  ```rust
  return oof!("my custom error").tag::<MyTag>().attach(&x).into_res();
  ```

  `into_res()` wraps `Oof` into `Result::Err(_)`.

- `wrap_err(_)`: function that wraps a custom error with `Oof`.

  ```rust
  return wrap_err(std::io::Error::new(std::io::ErrorKind::Other, "Some Error")).tag::<MyTag>().into_res();
  ```

  `into_res()` wraps `Oof` into `Result::Err(_)`.

- `ensure!(...)`: this is similar to a lot of other libraries with slight differences.

  First parameter is an expression that evaluates to `bool`.
  If the expression evaluates to `false`, the macro will return `Err(Oof)`.

  Second parameter is `context(...)`, and is an optional second parameter.
  You can use this if you want to display your own context message, instead of the default `assertion failed: EXPRESSION at LOCATION`.
  Inside `context(...)`, you can write as you do for `println!`.

  Other optional parameters are for tagging, attach, and attach_lazy.

  ```rust
  ensure!(
    false,
    context("custom context with value {:?}", x),
    tag: [MyTag, OtherTag],
    attach: [&y, "attachment", Instant::now()],
    attach_lazy: [|| serde_json::to_string(&y), || format!("lazy attachment {}", &z)]
  );
  ```

- `ensure_eq!(...)`: this is similar to a lot of other libraries with slight differences.

  First two parameters are parameters to be compared.
  If the parameters are not same, the macro will return `Err(Oof)`.

  Third parameter is `context(...)`, and is an optional third parameter.
  You can use this if you want to display your own context message, instead of the default `assertion failed: (left == right) at LOCATION`.
  Inside `context(...)`, you can write as you do for `println!`.

  Other optional parameters are for tagging, attach, and attach_lazy.

  ```rust
  ensure_eq!(
    123, 456,
    context("custom context with value {:?}", x),
    tag: [MyTag, OtherTag],
    attach: [&y, "attachment", Instant::now()],
    attach_lazy: [|| serde_json::to_string(&y), || format!("lazy attachment {}", &z)]
  );
  ```

  Note that all other parameters than the first one are optional.

## Features

- `location` (default: `true`): enables printing location of code that fails.
- `debug_strategy_disabled` (default: `false`): Disables debugging non-copy-able parameters.

  Default behavior is to debug non-copyable parameters for debug mode, but not debugging them for release mode.

- `debug_strategy_full` (default: `false`): Enables debugging non-copy-able parameters even for release mode.

  Default behavior is to debug non-copyable parameters for debug mode, but not debugging them for release mode.

## Notes/Limitations About the Library

### About `#[oofs]` Attribute

- `#[oofs]` generates and injects contexts into all statements and expressions that have `?` operator.

- `return Err(...)` or last expression without semicolon do not get injected with context.

- If the receiver of a method is a variable (i.e. `x.some_method()`), or a field of a variable (i.e. `x.field.some_method()`),
  values of `x` or `x.field` are not displayed. This is because there is no way to determine in the macro whether this receiver
  is a reference, mutable reference, or an owned variable.
  - For these cases, you can attach the variable like `x.some_method()._attach(&x)` to display the value of `x` in the error.

### About `Oof` Error Struct

- `Oof` does not implement `From<E> where E: std::error::Error`, and so must be built by attribute macro.
  So, if you don't include `#[oofs]`, it will throw a comiler error;
  this is intentional because it will catch the user's eyes and force them to include the attribute.

- Unlike `anyhow::Error` or `eyre::Report`, `Oof` does implement `std::error::Error`.
  This is nice because it makes it compatible with these boxed error types.
  For example, this works:

  ```rust
  #[oofs]
  fn outer_fn() -> Result<(), anyhow::Error> {
      inner_fn()?;
      Ok(())
  }
  ```

  It works since `?` operator will implicitly convert `Oof` into `anyhow::Error`.

### About Underscore Methods like `._tag()` and `._attach(_)`

In the basic examples above, you may have noticed that all the methods used for oof starts with an underscore;
you could call them 'meta-methods' as they do not affect the logic, but only the result that is returned.

The reason for this is that there has to be a way for the macro to differentiate between functional methods and meta methods.
This is because macro would also try to include these meta methods as part of the displayed method chain, and things like
`_attach(x)` would be displayed twice in `Parameters` section and `Attachments` section.

This may seem disturbing and unnatural at first; it was for me, too.
But after trying it out, I got used to it; and now I think I like it because I can easily differentiate between functional methods and meta methods.

I apologize for the inconvenience, and please let me know if there was a better way to do this.

### Debugging Non-Copyable Arguments

One of the pain points while creating the library was to lazy-load values of copyable arguments and instantly load values of non-copyable arguments at compile time.
I figured out how to do this with a cool rust hack.

Now, should the default behavior be to always instantly load values of non-copyable arguments? this could incur unwanted performance costs, as it would load them for non-error cases.

As a compromise, I made it so that, for debug mode, it will instantly load values of non-copyable arguments; and, for release mode, it will not load values of non-copyable arguments.

You can change this behavior with features `debug_strategy_disabled` and `debug_strategy_full`.

`debug_strategy_disabled` will disable loading values of non-copyable arguments even for debug mode.
`debug_strategy_full` will enable loading values of non-copyable arguments even for releaes mode.

### Compatibility with `#[async_trait]`

`#[async_trait]` parses and converts `async fn` in traits into `fn -> Box<Future<Output = Result<...>>>`.
Since `#[oofs]` by default only applies context injection to methods that returns `Result<_, _>`,
it will not apply injection once `#[async_trait]` is applied.

There are two ways to deal with this:

- Place `#[oofs]` above `#[async_trait]`, so that oofs is applied first, then `#[async_trait]`.

  ```rust
  #[oofs]
  #[async_trait]
  impl Trait for Struct {
    ...
  }
  ```

- In the impl block, place `#[oofs]` above `fn ...`, and this will tell the macro to apply injection regardless.
  ```rust
  #[async_trait]
  #[oofs]
  impl Trait for Struct {
    #[oofs]
    async fn do_something() -> Result<(), _> {
        ...
    }
  }
  ```

## Future Plans

This library is still very much WIP.

I plan to test the error handling for performance, optimize memory footprints of errors,
and implement attribute arguments like `#[oofs(tag(MyTag))]`, `#[oofs(skip)]`, etc.

Also, it does not inject context into closures and async blocks. I plan to add attribute args like `#[oofs(closures)]` and `#[oofs(async_blocks)]` to enable injecting context to closures and async blocks.
