use std::hash::Hash;

pub trait ConsistentHasher<N: Sized> {
    fn capacity(&self) -> usize;

    fn get<Q: Hash>(&self, key: &Q) -> &N;
}
