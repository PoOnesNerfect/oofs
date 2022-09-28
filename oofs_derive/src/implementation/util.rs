pub struct OpWrapper<T> {
    pub op: OpLevel,
    pub item: T,
}

pub enum OpLevel {
    Disabled,
    Debug,
    Release,
}
