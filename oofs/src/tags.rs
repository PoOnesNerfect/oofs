use std::{any::TypeId, collections::HashSet};

#[derive(Debug, Clone)]
pub struct Tags {
    set: HashSet<TypeId>,
}

impl Tags {
    pub fn new() -> Self {
        Tags {
            set: HashSet::new(),
        }
    }

    pub fn tag<T: 'static>(&mut self) {
        self.set.insert(TypeId::of::<T>());
    }

    pub fn untag<T: 'static>(&mut self) {
        self.set.remove(&TypeId::of::<T>());
    }

    pub fn tagged<T: 'static>(&self) -> bool {
        self.set.contains(&TypeId::of::<T>())
    }

    pub fn iter(&self) -> impl Iterator<Item = &TypeId> {
        self.set.iter()
    }
}
