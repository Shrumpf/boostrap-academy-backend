use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Debug, Default)]
pub struct TypeMap(HashMap<TypeId, Box<dyn Any>>);

impl TypeMap {
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.0
            .get(&TypeId::of::<T>())
            .map(|x| x.downcast_ref().unwrap())
    }

    pub fn insert<T: 'static>(&mut self, x: T) {
        self.0.insert(TypeId::of::<T>(), Box::new(x));
    }
}
