use core::{
    convert::Infallible,
    ops::{Deref, DerefMut},
    slice,
};
use std::{fmt::Debug, marker::PhantomData};

use serde::Serialize;

pub trait CollectionFamily: Serialize + Debug + Copy + Clone {
    type Member<T, const N: usize>: PostcardVecMut<T>
    where
        T: Serialize + Debug + Clone;

    fn new<T: Serialize + Debug + Clone, const N: usize>(&self) -> Self::Member<T, N>;
}

#[derive(Copy, Clone, PartialEq, Serialize, Debug)]
pub struct VecFamily;

#[cfg(feature = "use-std")]
impl CollectionFamily for VecFamily {
    type Member<T, const N:usize> = Vec<T> where T: Serialize + Debug + Clone;

    fn new<T: Serialize + Debug + Clone, const N: usize>(&self) -> Self::Member<T, N> {
        Self::Member::new()
    }
}

#[derive(Copy, Clone, PartialEq, Serialize, Debug)]
pub struct HVecFamily;

impl CollectionFamily for HVecFamily {
    type Member<T, const N: usize> = heapless::Vec<T, N> where T: Serialize + Debug + Clone;

    fn new<T: Serialize + Debug + Clone, const N: usize>(&self) -> Self::Member<T, N> {
        Self::Member::new()
    }
}

pub trait Collection<T> {
    type Iter<'iter>: Iterator<Item = &'iter T>
    where
        Self: 'iter,
        T: 'iter;
    fn iterate<'iter>(&'iter self) -> Self::Iter<'iter>;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
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

    fn len(&self) -> usize {
        unimplemented!()
    }

    fn is_empty(&self) -> bool {
        unimplemented!()
    }
}

#[cfg(feature = "use-std")]
impl<T> Collection<T> for Vec<T> {
    type Iter<'iter> = slice::Iter<'iter, T> where T: 'iter;
    fn iterate<'iter>(&'iter self) -> slice::Iter<'iter, T> {
        self.iter()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<T, const N: usize> Collection<T> for heapless::Vec<T, N> {
    fn iterate<'iter>(&'iter self) -> slice::Iter<'iter, T> {
        self.iter()
    }
    type Iter<'iter> = slice::Iter<'iter, T> where T: 'iter;

    fn len(&self) -> usize {
        N
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
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
    fn insert(&mut self, index: usize, element: T);
    fn remove(&mut self, index: usize) -> T;
    fn clear(&mut self);
}

#[cfg(feature = "use-std")]
impl<T> CollectionMut<T> for Vec<T> {
    type Err = Infallible;

    fn push(&mut self, value: T) -> Result<(), Self::Err> {
        self.push(value);

        Ok(())
    }

    fn insert(&mut self, index: usize, element: T) {
        self.insert(index, element)
    }

    fn remove(&mut self, index: usize) -> T {
        self.remove(index)
    }

    fn clear(&mut self) {
        self.clear()
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

    fn insert(&mut self, _index: usize, _element: T) {
        unimplemented!()
    }

    fn remove(&mut self, _index: usize) -> T {
        unimplemented!()
    }

    fn clear(&mut self) {
        self.clear()
    }
}

/// a serializable vector-like
pub trait PostcardVec<T>: Collection<T> + Serialize + Deref + AsRef<[T]> + Clone + Debug {}
impl<T, C: Collection<T> + Serialize + Deref + AsRef<[T]> + Clone + Debug> PostcardVec<T> for C {}

/// a serializable and mutable vector-like
pub trait PostcardVecMut<T>: PostcardVec<T> + CollectionMut<T> + DerefMut + AsMut<[T]> {}
impl<T, C: PostcardVec<T> + CollectionMut<T> + DerefMut + AsMut<[T]>> PostcardVecMut<T> for C {}

#[cfg(all(test, feature = "use-std"))]
mod tests {

    use serde::{de::DeserializeOwned, Deserialize};

    use super::*;

    #[derive(Serialize, Deserialize)]
    struct Outer<T, C: PostcardVecMut<T>> {
        content: C,
        _phantom: T,
    }
    #[test]
    fn ser_de() {
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

    type C<CF, T, const N: usize> = <CF as CollectionFamily>::Member<T, N>;

    #[derive(Serialize, Debug, Clone)]
    struct Inner2<CF: CollectionFamily> {
        data: C<CF, u32, 2>,
    }

    #[derive(Serialize)]
    struct Outer2<CF: CollectionFamily> {
        inners: C<CF, Inner2<CF>, 2>, // <- cannot #[derive(Deserialize)]
        simple: C<CF, u32, 1>,
    }

    impl<CF: CollectionFamily> Outer2<CF> {
        fn new(f: CF) -> Self {
            Self {
                simple: f.new(),
                inners: f.new(),
            }
        }
    }

    #[test]
    fn nested() {
        let factory = VecFamily;

        let mut outer = Outer2::new(factory);

        outer.inners.push(Inner2 {
            data: factory.new::<_, 0>(),
        });
    }

    #[test]
    fn heapless() {
        let factory = HVecFamily;

        let mut outer = Outer2::new(factory);

        outer.simple.push(1).unwrap();
        assert!(outer.simple.push(2).is_err());

        outer
            .inners
            .push(Inner2 {
                data: factory.new(),
            })
            .unwrap();

        outer
            .inners
            .push(Inner2 {
                data: factory.new(),
            })
            .unwrap();

        assert!(outer
            .inners
            .push(Inner2 {
                data: factory.new(),
            })
            .is_err(),);
    }
}
