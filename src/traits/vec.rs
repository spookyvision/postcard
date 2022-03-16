use core::{
    convert::Infallible,
    ops::{Deref, DerefMut},
    slice,
};

use serde::Serialize;

trait Collection<T> {
    type Iter<'iter>: Iterator<Item = &'iter T>
    where
        Self: 'iter,
        T: 'iter;
    fn iterate<'iter>(&'iter self) -> Self::Iter<'iter>;
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

trait IterMut<T> {
    type IterMut<'iter>: Iterator<Item = &'iter mut T>
    where
        Self: 'iter,
        T: 'iter;
    fn iterate_mut<'iter>(&'iter mut self) -> Self::IterMut<'iter>;
}

trait CollectionMut<T>: IterMut<T> {
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

trait PostcardVec<T>: Collection<T> + Serialize + Deref + AsRef<[u8]> {}
impl<T, C: Collection<T> + Serialize + Deref + AsRef<[u8]>> PostcardVec<T> for C {}

trait PostcardVecMut<T>: PostcardVec<T> + CollectionMut<T> + DerefMut + AsMut<[u8]> {}
impl<T, C: PostcardVec<T> + CollectionMut<T> + DerefMut + AsMut<[u8]>> PostcardVecMut<T> for C {}

#[cfg(all(test, feature = "use-std"))]
mod tests {

    use serde::Deserialize;

    use super::*;

    #[derive(Serialize, Deserialize)]
    struct Clip<T, C: PostcardVecMut<T>> {
        beats: C,
        _phantom: T,
    }
    #[test]
    fn it_works() {
        type HV32<T> = heapless::Vec<T, 32>;

        let mut std = Clip {
            beats: vec![1, 2, 3],
            _phantom: Default::default(),
        };
        std.beats.push(5);

        let _data: &[u8] = &std.beats;

        let mut heapless = Clip {
            beats: HV32::from_slice(&[1, 2, 3]).unwrap(),
            _phantom: Default::default(),
        };
        heapless.beats.push(5).unwrap();

        let ser_std = serde_json::to_string(&std).unwrap();
        let ser_heapless = serde_json::to_string(&heapless).unwrap();

        assert_eq!(ser_std, ser_heapless);

        // the LHS/RHS swap is intentional
        let _de_std: Clip<u8, std::vec::Vec<u8>> = serde_json::from_str(&ser_heapless).unwrap();
        let _de_heapless: Clip<u8, HV32<_>> = serde_json::from_str(&ser_std).unwrap();
    }
}
