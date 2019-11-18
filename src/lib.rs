use std::{any::Any, marker::PhantomData};

use generational_arena as ga;

pub struct AnyArena<T> {
    arena: ga::Arena<ArenaCell<T>>,
}

pub struct ArenaCell<T> {
    uniform: T,
    any: Box<dyn Any>,
}

pub struct Index<T, U: ?Sized + Any> {
    index: ga::Index,
    _marker: PhantomData<*const (T, U)>,
}

impl<T, U: ?Sized + Any> Index<T, U> {
    fn new(index: ga::Index) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }
}

pub trait IndexCast<T, U: ?Sized + Any> {
    fn cast(self) -> Index<T, U>;
}

#[doc(hidden)]
pub trait CastFromIndex<T: ?Sized> {}

impl<T, U, V> IndexCast<T, U> for Index<T, V>
where
    U: 'static,
    V: CastFromIndex<U> + 'static,
{
    fn cast(self) -> Index<T, U> {
        Index::new(self.index)
    }
}

impl<T> AnyArena<T> {
    pub fn new() -> Self {
        Self {
            arena: ga::Arena::new(),
        }
    }

    pub fn insert<U: Any>(&mut self, uniform: T, any: U) -> Index<T, U> {
        Index::new(self.arena.insert(ArenaCell {
            uniform,
            any: Box::new(any),
        }))
    }

    pub fn get<U, I>(&self, index: Index<T, U>) -> Option<(&T, &U)>
    where
        U: Any + Into<I>,
    {
        self.arena
            .get(index.index)
            .map(|c| (&c.uniform, c.any.downcast_ref().unwrap()))
    }
}

#[macro_export]
macro_rules! any_trait {
    ($trait:path) => {
        impl<U> $crate::CastFromIndex<U> for dyn $trait where U: $trait + Sized + 'static {}
    };
}

#[macro_export]
macro_rules! any_super {
    ($sub:path : $super:path) => {
        impl $crate::CastFromIndex<dyn $sub> for dyn $super {}

        #[allow(dead_code)]
        fn upcast_test<T: $sub>(t: T) -> impl $super {
            t
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    trait Super {}
    trait Sub: Super {}

    any_trait!(Super);
    any_trait!(Sub);
    any_super!(Sub: Super);
}
