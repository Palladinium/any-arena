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

impl<T, U: Any + ?Sized> Index<T, U> {
    pub fn new(index: ga::Index) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
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

// FIXME redo as derive macro
#[macro_export]
macro_rules! any_trait {
    ($super:path) => {
        impl<T, U> From<Index<T, U>> for Index<T, dyn $super>
        where
            U: Superr + Sized + 'static,
        {
            fn from(idx: Index<T, U>) -> Index<T, dyn $super> {
                Index::new(idx.index)
            }
        }
    };
}

#[macro_export]
macro_rules! any_super {
    ($sub:path, $super:path) => {
        #[allow(dead_code)]
        fn upcast_test<T: $sub>(t: T) -> impl $super {
            t
        }
    };
}
