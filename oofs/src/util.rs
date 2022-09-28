use std::any::TypeId;

pub trait __TypeInfo {
    fn __type_name(&self) -> &'static str {
        core::any::type_name::<Self>()
    }

    fn __type_id(&self) -> TypeId
    where
        Self: 'static,
    {
        TypeId::of::<Self>()
    }
}
impl<T> __TypeInfo for T {}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct __Struct<T>(T);

    #[test]
    fn test_type_name() {
        let x = __Struct("hello world".to_owned());

        assert_eq!(
            x.__type_name(),
            "oof::util::tests::__Struct<alloc::string::String>"
        );
    }

    pub struct Generic<T>(T);

    pub struct CGeneric<const N: usize>([u8; N]);

    #[test]
    fn test_type_id() {
        assert_ne!(
            TypeId::of::<Generic<usize>>(),
            TypeId::of::<Generic<String>>()
        );
        assert_ne!(TypeId::of::<CGeneric<1>>(), TypeId::of::<CGeneric<5>>());
    }
}
