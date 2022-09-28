# oofs

This library is WIP.

It compiles with basic attribute `#[oofs]`, and it will generate `Oof`s, but error display needs a lot more work.

Also, attribute args need to be implemented as well as well as features.

Below is a description of the finished product, and not the current state of the library.

You can still check it out, play with it, and expand the macro to see how things work.

Stay tuned for a next minor release!

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/oofs.svg
[crates-url]: https://crates.io/crates/oofs
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/PoOnesNerfect/oofs/blob/main/LICENSE

This library provides `#[oofs]`, an attribute that generates and injects an error context into all instances of try operator `?` in `fn` or `impl` methods.

Below is an example from `oofs/tests/basic.rs`

```rust
use oofs::{oofs, Oof};

#[oofs]
fn main_fn() -> Result<(), Oof> {
    let my_struct = MyStruct {
        field0: 123,
        field1: "hello world".to_owned(),
    };

    my_struct.some_method(321, "watermelon sugar")?;

    Ok(())
}

#[derive(Debug)]
struct MyStruct {
    field0: usize,
    field1: String,
}

#[oofs]
impl MyStruct {
    fn some_method(&self, x: usize, y: &str) -> Result<usize, Oof> {
        some_fn(y)?;

        Ok(x)
    }
}

#[oofs]
fn some_fn(text: &str) -> Result<(), Oof> {
    let _ = text.parse::<u64>()?;

    Ok(())
}
```

In the above example, if we were to print out an error from `main_fn`, it would look something like this:

```
$my_struct.some_method($x, $y) failed
    at `oofs/tests/basic.rs:10:15`

Paramters:
    $my_struct: MyStruct = MyStruct { field0: 123, field1: "hello world" }
    $x: usize = 321,
    $y: &str = "watermelon sugar"

Caused by:
    0: some_fn($y) failed
           at `oofs/tests/basic.rs:21:1`

       parameters:
           $y: &str = "watermelon sugar"

    1: $text.parse::<u64>() failed
           at `oofs/tests/basic.rs:32:18`

       Paramters:
           $text: &str = "watermelon sugar"

    2: invalid digit found in string
```

I don't know about you, but, this looks really nice for something you get for free!

You get the information about the chains of methods that fail and their locations in code, the parameters' names, types and their debug info!

## Handling Error Cases

Yes, those error messages look wonderful, but how can we handle for different error cases in code?
Since `Oof` does wrap previous errors in a `Box`, it is still hard to handle different errors, or is it?

That is why I introduced tagging to `Oof`.

Take the same example from above, but I change this one line in `some_fn(...)`:

```rust
struct NoRetry;

#[oofs]
fn some_fn(text: &str) -> Result<(), Oof> {
    let _ = text.parse::<u64>().tag::<NoRetry>()?;

    Ok(())
}
```

Now, in any functions or methods that use `some_fn(...)`, you can search for the tag!

```rust
struct NoRetry;

#[oofs]
fn main_fn() -> Result<(), Oof> {
    let my_struct = MyStruct {
        field0: 123,
        field1: "hello world".to_owned(),
    };

    if let Err(err) = my_struct.some_method(321, "watermelon sugar") {
        if err.tagged_nested::<NoRetry>() {
            ...some action
        } else {
            ...some other action
        }
    }

    Ok(())
}

...

#[oofs]
fn some_fn(text: &str) -> Result<(), Oof> {
    let _ = text.parse::<u64>().tag::<NoRetry>()?;

    Ok(())
}
```

With tagging, you do not need to handle every different error cases; instead, you can group them by tags, and just handle for different tags!

You can also tag multiple times to the same error. Underneath, it's just a `HashSet<TypeId>`.

I think this is especially useful when categorizing different errors is necessary, like returning HTTP status codes, or retry or no_retry, etc.

If you want to tag all try operator instances in a given function, simply add a tag attribute:

```rust
struct NoRetry;

#[oofs]
#[oofs(tag(NoRetry))]
fn some_fn(text: &str) -> Result<(), Oof> {
    let _ = text.parse::<u64>()?;

    some_other_fn(...)?;

    Ok(())
}
```

This will tag `NoRetry` to both `parse()` and `some_other_fn()` and any other instances of try operators.
