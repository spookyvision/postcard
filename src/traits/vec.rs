use core::{
    convert::Infallible,
    ops::{Deref, DerefMut},
    slice,
};
use std::marker::PhantomData;

use serde::Serialize;

pub trait Collection<T> {
    type Iter<'iter>: Iterator<Item = &'iter T>
    where
        Self: 'iter,
        T: 'iter;
    fn iterate<'iter>(&'iter self) -> Self::Iter<'iter>;
}

pub struct NothingIterator<'it> {
    _phantom: PhantomData<&'it ()>,
}

impl<'it> NothingIterator<'it> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData::default(),
        }
    }
}
impl<'it> Iterator for NothingIterator<'it> {
    type Item = &'it ();

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
impl Collection<()> for () {
    type Iter<'iter> = NothingIterator<'iter>;

    fn iterate<'iter>(&'iter self) -> Self::Iter<'iter> {
        NothingIterator::new()
    }
}

#[cfg(feature = "use-std")]
impl<T> Collection<T> for Vec<T> {
    type Iter<'iter> = slice::Iter<'iter, T> where T: 'iter;
    fn iterate<'iter>(&'iter self) -> slice::Iter<'iter, T> {
        self.iter()
    }
}

impl<T> Collection<T> for &[T] {
    type Iter<'iter> = slice::Iter<'iter, T> where T: 'iter, Self: 'iter;
    fn iterate<'iter>(&'iter self) -> slice::Iter<'iter, T> {
        self.iter()
    }
}

impl<T, const N: usize> Collection<T> for heapless::Vec<T, N> {
    fn iterate<'iter>(&'iter self) -> slice::Iter<'iter, T> {
        self.iter()
    }
    type Iter<'iter> = slice::Iter<'iter, T> where T: 'iter;
}

pub trait IterMut<T> {
    type IterMut<'iter>: Iterator<Item = &'iter mut T>
    where
        Self: 'iter,
        T: 'iter;
    fn iterate_mut<'iter>(&'iter mut self) -> Self::IterMut<'iter>;
}

pub trait CollectionMut<T>: IterMut<T> {
    type Err;
    fn push(&mut self, value: T) -> Result<(), Self::Err>;
}

#[cfg(feature = "use-std")]
impl<T> CollectionMut<T> for Vec<T> {
    type Err = Infallible;

    fn push(&mut self, value: T) -> Result<(), Self::Err> {
        self.push(value);

        Ok(())
    }
}

#[cfg(feature = "use-std")]
impl<T> IterMut<T> for Vec<T> {
    type IterMut<'iter> = slice::IterMut<'iter, T> where T: 'iter;

    fn iterate_mut<'iter>(&'iter mut self) -> slice::IterMut<'iter, T> {
        self.iter_mut()
    }
}

impl<T> IterMut<T> for &mut [T] {
    type IterMut<'iter> = slice::IterMut<'iter, T> where T: 'iter, Self: 'iter;
    fn iterate_mut<'iter>(&'iter mut self) -> slice::IterMut<'iter, T> {
        self.iter_mut()
    }
}

impl<T, const N: usize> IterMut<T> for heapless::Vec<T, N> {
    type IterMut<'iter> = slice::IterMut<'iter, T> where T: 'iter;

    fn iterate_mut<'iter>(&'iter mut self) -> slice::IterMut<'iter, T> {
        self.iter_mut()
    }
}

impl<T, const N: usize> CollectionMut<T> for heapless::Vec<T, N> {
    type Err = T;

    fn push(&mut self, value: T) -> Result<(), Self::Err> {
        self.push(value)
    }
}

/// a serializable vector-like
pub trait PostcardVec<T>: Collection<T> + Serialize + Deref + AsRef<[T]> {}
impl<T, C: Collection<T> + Serialize + Deref + AsRef<[T]>> PostcardVec<T> for C {}

/// a serializable and mutable vector-like
pub trait PostcardVecMut<T>: PostcardVec<T> + CollectionMut<T> + DerefMut + AsMut<[T]> {}
impl<T, C: PostcardVec<T> + CollectionMut<T> + DerefMut + AsMut<[T]>> PostcardVecMut<T> for C {}

#[cfg(all(test, feature = "use-std"))]
mod tests {

    use serde::Deserialize;

    use super::*;

    #[derive(Serialize, Deserialize)]
    struct Outer<T, C: PostcardVecMut<T>> {
        content: C,
        _phantom: T,
    }
    #[test]
    fn it_works() {
        type HV32<T> = heapless::Vec<T, 32>;

        let mut std = Outer {
            content: vec![1, 2, 3],
            _phantom: Default::default(),
        };
        std.content.push(5);

        let _data: &[u8] = &std.content;

        let mut heapless = Outer {
            content: HV32::from_slice(&[1, 2, 3]).unwrap(),
            _phantom: Default::default(),
        };
        heapless.content.push(5).unwrap();

        let ser_std = serde_json::to_string(&std).unwrap();
        let ser_heapless = serde_json::to_string(&heapless).unwrap();

        assert_eq!(ser_std, ser_heapless);

        // the LHS/RHS swap is intentional
        let _de_std: Outer<u8, std::vec::Vec<u8>> = serde_json::from_str(&ser_heapless).unwrap();
        let _de_heapless: Outer<u8, HV32<_>> = serde_json::from_str(&ser_std).unwrap();
    }
}
