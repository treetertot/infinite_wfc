use std::{marker::PhantomData, hash::Hash};

use ahash::AHashMap;


#[derive(Debug, Clone)]
pub struct UniqueSlices<T> {
    map: AHashMap<Box<[T]>, ID<T>>,
    concat_possibilities: Vec<T>,
    starts: Vec<u32>,
    lens: Vec<u16>,
}
impl<T> UniqueSlices<T>
where T: Clone + Eq + Hash {
    pub fn new() -> UniqueSlices<T> {
        Self::default()
    }
    pub fn try_identify(&self, possibilities: &[T]) -> Option<ID<T>> {
        self.map.get(possibilities).map(|a| a.clone())
    }
    pub fn identify(&mut self, possibilities: &[T]) -> ID<T> {
        if let Some(id) = self.try_identify(possibilities) {
            return id;
        }
        let id = ID::new(self.starts.len() as u32);
        self.map.insert(possibilities.to_owned().into_boxed_slice(), id.clone());

        let start = self.concat_possibilities.len() as u32;
        let len = possibilities.len() as u16;

        self.starts.push(start);
        self.lens.push(len);

        self.concat_possibilities.extend_from_slice(possibilities);

        id
    }
    pub fn get(&self, id: ID<T>) -> &[T] {
        let index = id.n as usize;
        let start = self.starts[index] as usize;
        let end = self.lens[index] as usize + start;
        &self.concat_possibilities[start..end]
    }
}
impl<T> Default for UniqueSlices<T> {
    fn default() -> Self {
        UniqueSlices {
            map: AHashMap::new(),
            concat_possibilities: Vec::new(),
            starts: Vec::new(),
            lens: Vec::new()
        }
    }
}

#[test]
fn identifying() {
    let mut ider = UniqueSlices::new();
    let a = [3, 6, 4];
    let a_id = ider.identify(&a);
    let b = [5];
    let b_id = ider.identify(&b);
    assert_eq!(a_id, ider.identify(&a));
    assert_ne!(a_id, b_id);
    assert_eq!(&b, ider.get(b_id));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Possibility (pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ID<T> {
    n: u32,
    _x: PhantomData<T>
}
impl<T> ID<T> {
    pub const ZERO: Self = Self::new(0);
    pub const ONE: Self = Self::new(1);
    const fn new(n: u32) -> Self {
        ID {
            n,
            _x: PhantomData
        }
    }
}