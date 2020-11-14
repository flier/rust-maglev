use std::borrow::Borrow;
use std::hash::Hash;

/// Consistent hasher is a special kind of hashing such that when a hash table is resized,
/// only `K/n` keys need to be remapped on average, where `K` is the number of keys,
/// and `n` is the number of slots.
pub trait ConsistentHasher<N: Sized> {
    /// Returns all nodes in arbitrary order.
    fn nodes(&self) -> &[N];

    /// Returns the number of slots in the lookup table.
    fn capacity(&self) -> usize;

    /// Returns a reference to the node corresponding to the key.
    fn get<Q: ?Sized>(&self, key: &Q) -> Option<&N>
    where
        Q: Hash + Eq,
        N: Borrow<Q>;
}
