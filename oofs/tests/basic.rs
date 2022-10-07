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

    // Passing an expression as parameter is also fine.
    // All parameters are evaluated before being debugged in the error.
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

// #[oofs] can also be used to `impl`.
// Context will be injected to all methods that return a `Result`.
#[oofs]
impl MyStruct {
    fn failing_method(&self, x: usize) -> Result<u64, Oof> {
        let ret = self
            .field
            .parse::<u64>()
            ._tag::<RetryTag>() // tags the error with the type `RetryTag`.
            ._attach(x) // attach anything that implements `Debug` as custom context.
            ._attach(&self.field) // attach the receiver as attachment to debug.
            ._attach_lazy(|| "extra context")?; // lazily evaluate context; useful for something like `|| serde_json::to_string(&x)`.

        Ok(ret)
    }
}

#[test]
fn implements_basic_error() {
    let res = application();

    assert!(res.is_err());

    let err = res.unwrap_err();
    println!("{:?}", err);
}
