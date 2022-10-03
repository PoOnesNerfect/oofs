use oofs::*;

#[oofs]
fn main_fn() -> Result<(), Oof> {
    let my_struct = MyStruct {
        field0: 123,
        field1: "hello world".to_owned(),
    };

    my_struct
        .some_method(321, "watermelon sugar")?
        .checked_add(1)?;

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

#[test]
fn implements_basic_error() {
    let res = main_fn();

    assert!(res.is_err());

    println!("{:?}", res.unwrap_err());
}
