use std::{any::Any, marker::PhantomData};

use generational_arena as ga;
use traitcast::Traitcast;

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

// Cannot derive Clone or Copy since the generic params may not be Clone/Copy

impl<T, U: ?Sized + Any> Clone for Index<T, U> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            _marker: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.index = source.index;
    }
}

impl<T, U: ?Sized + Any> Copy for Index<T, U> {}

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
    U: CastFromIndex<V> + ?Sized + 'static,
    V: ?Sized + 'static,
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

    pub fn get<U>(&self, index: Index<T, U>) -> Option<(&T, &U)>
    where
        U: Any + ?Sized + 'static,
    {
        self.arena
            .get(index.index)
            .map(|c| (&c.uniform, c.any.cast_ref().unwrap()))
    }
}

#[macro_export]
macro_rules! any_arena {
    (struct $struct:path) => {
        traitcast!(struct $struct);
    };

    (struct $struct:ty : $($trait:ident),+ $(,)?) => {
        $(
            impl $crate::CastFromIndex<$struct> for dyn $trait {}
        )+

        traitcast!(struct $struct : $($trait),+);
    };

    (impl $trait:ident for $type:path) => {
        impl $crate::CastFromIndex<$type> for dyn $trait {}
        impl $crate::CastFromIndex<dyn $trait> for $type {}

        traitcast!(impl $trait for $type);
    };

    (trait $sub:path : $($super:path),+ $(,)?) => {
        $(
            impl $crate::CastFromIndex<dyn $sub> for dyn $super {}
        )+
    };
}

#[cfg(test)]
mod test {
    use super::*;

    use traitcast::traitcast;

    trait Super {}
    trait Sub: Super {}

    struct Foo;

    impl Super for Foo {}
    impl Sub for Foo {}

    struct Bar;

    impl Super for Bar {}
    impl Sub for Bar {}

    any_arena!(struct Foo);
    any_arena!(impl Sub for Foo);
    any_arena!(impl Super for Foo);
    any_arena!(struct Bar: Sub, Super);
    any_arena!(trait Sub: Super);

    #[test]
    fn test() {
        let mut arena: AnyArena<()> = AnyArena::new();

        let foo1: Index<(), Foo> = arena.insert((), Foo);
        let foo1_sub: Index<(), dyn Sub> = foo1.cast();
        let foo1_sub_super: Index<(), dyn Super> = foo1_sub.cast();
        let foo1_super: Index<(), dyn Super> = foo1.cast();
    }
}
