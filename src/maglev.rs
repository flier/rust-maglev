use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};
use std::iter;

use primal::Sieve;

use crate::conshash::ConsistentHasher;

/// Maglev lookup table
#[derive(Clone)]
pub struct Maglev<N, S> {
    nodes: Vec<N>,
    lookup: Vec<isize>,
    hash_builder: S,
}

impl<N: Hash + Eq> Maglev<N, BuildHasherDefault<DefaultHasher>> {
    /// Creates a `Maglev` lookup table.
    pub fn new<I: IntoIterator<Item = N>>(nodes: I) -> Self {
        Maglev::with_capacity_and_hasher(nodes, 0, Default::default())
    }

    /// Creates a `Maglev` lookup table with the specified capacity.
    pub fn with_capacity<I: IntoIterator<Item = N>>(nodes: I, capacity: usize) -> Self {
        Maglev::with_capacity_and_hasher(nodes, capacity, Default::default())
    }
}

impl<N: Hash + Eq, S: BuildHasher> Maglev<N, S> {
    /// Creates a `Maglev` lookup table which will use the given hash builder to hash keys.
    pub fn with_hasher<I: IntoIterator<Item = N>>(nodes: I, hash_builder: S) -> Self {
        Maglev::with_capacity_and_hasher(nodes, 0, hash_builder)
    }

    /// Creates a `Maglev` lookup table with the specified capacity, using hasher to hash the keys.
    pub fn with_capacity_and_hasher<I: IntoIterator<Item = N>>(
        nodes: I,
        capacity: usize,
        hash_builder: S,
    ) -> Self {
        let nodes = nodes.into_iter().collect::<Vec<_>>();
        let lookup = Self::populate(
            &nodes,
            if capacity > 0 {
                capacity
            } else {
                nodes.len() * 100
            },
            &hash_builder,
        );

        Maglev {
            nodes,
            lookup,
            hash_builder,
        }
    }

    #[inline]
    fn hash_with_seed<Q: Hash + Eq + ?Sized>(key: &Q, seed: u32, hash_builder: &S) -> usize {
        let mut hasher = hash_builder.build_hasher();
        hasher.write_u32(seed);
        key.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn populate(nodes: &[N], capacity: usize, hash_builder: &S) -> Vec<isize> {
        let m = Sieve::new(capacity * 2)
            .primes_from(capacity)
            .next()
            .unwrap();
        let n = nodes.len();

        let permutation: Vec<Vec<usize>> = nodes
            .iter()
            .map(|node| {
                let offset = Self::hash_with_seed(&node, 0xdead_babe, &hash_builder) % m;
                let skip = (Self::hash_with_seed(&node, 0xdead_beef, &hash_builder) % (m - 1)) + 1;

                (0..m).map(|i| (offset + i * skip) % m).collect()
            })
            .collect();

        let mut next: Vec<usize> = iter::repeat(0).take(n).collect();
        let mut entry: Vec<isize> = iter::repeat(-1).take(m).collect();

        let mut j = 0;

        while j < m {
            for i in 0..n {
                let mut c = permutation[i][next[i]];

                while entry[c] >= 0 {
                    next[i] += 1;
                    c = permutation[i][next[i]];
                }

                entry[c] = i as isize;
                next[i] += 1;
                j += 1;

                if j == m {
                    break;
                }
            }
        }

        entry
    }
}

impl<N: Hash + Eq> iter::FromIterator<N> for Maglev<N, BuildHasherDefault<DefaultHasher>> {
    fn from_iter<T: IntoIterator<Item = N>>(iter: T) -> Self {
        Maglev::new(iter)
    }
}

impl<N: Hash + Eq, S: BuildHasher> ConsistentHasher<N> for Maglev<N, S> {
    #[inline]
    fn nodes(&self) -> &[N] {
        self.nodes.as_slice()
    }

    #[inline]
    fn capacity(&self) -> usize {
        self.lookup.len()
    }

    #[inline]
    fn get<Q: ?Sized>(&self, key: &Q) -> &N
    where
        Q: Hash + Eq,
        N: Borrow<Q>,
    {
        let key = Self::hash_with_seed(key, 0xdead_babe, &self.hash_builder);

        &self.nodes[self.lookup[key % self.lookup.len()] as usize]
    }
}

#[cfg(test)]
pub mod tests {
    use fasthash::spooky::Hash128;

    use super::*;
    use crate::conshash::ConsistentHasher;

    #[test]
    fn test_maglev() {
        let m = Maglev::new(vec![
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ]);

        assert_eq!(m.nodes.len(), 7);
        assert_eq!(m.lookup.len(), 701);
        assert!(m.lookup.iter().all(|&n| n < m.nodes.len() as isize));

        assert_eq!(*m.get("alice"), "Friday");
        assert_eq!(*m.get("bob"), "Wednesday");

        let m = Maglev::with_capacity(
            vec![
                "Monday",
                "Tuesday",
                "Wednesday",
                // "Thursday",
                "Friday",
                "Saturday",
                "Sunday",
            ],
            m.capacity(),
        );

        assert_eq!(m.nodes.len(), 6);
        assert_eq!(m.lookup.len(), 701);
        assert!(m.lookup.iter().all(|&n| n < m.nodes.len() as isize));

        assert_eq!(*m.get("alice"), "Friday");
        assert_eq!(*m.get("bob"), "Wednesday");

        let m = Maglev::with_capacity(
            vec![
                "Monday",
                // "Tuesday",
                "Wednesday",
                // "Thursday",
                "Friday",
                "Saturday",
                "Sunday",
            ],
            m.capacity(),
        );

        assert_eq!(m.nodes.len(), 5);
        assert_eq!(m.lookup.len(), 701);
        assert!(m.lookup.iter().all(|&n| n < m.nodes.len() as isize));

        assert_eq!(*m.get("alice"), "Friday");
        assert_eq!(*m.get("bob"), "Wednesday");

        let m = Maglev::with_capacity(
            vec![
                "Monday",
                "Tuesday",
                "Wednesday",
                // "Thursday",
                // "Friday",
                "Saturday",
                "Sunday",
            ],
            m.capacity(),
        );

        assert_eq!(m.nodes.len(), 5);
        assert_eq!(m.lookup.len(), 701);
        assert!(m.lookup.iter().all(|&n| n < m.nodes.len() as isize));

        assert_eq!(*m.get("alice"), "Saturday");
        assert_eq!(*m.get("bob"), "Wednesday");
    }

    #[test]
    fn test_maglev_with_custom_hasher() {
        let m = Maglev::with_hasher(
            vec![
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
                "Sunday",
            ],
            Hash128 {},
        );

        assert_eq!(m.nodes.len(), 7);
        assert_eq!(m.lookup.len(), 701);
        assert!(m.lookup.iter().all(|&n| n < m.nodes.len() as isize));

        assert_eq!(*m.get("alice"), "Monday");
        assert_eq!(*m.get("bob"), "Wednesday");

        let m = Maglev::with_capacity_and_hasher(
            vec![
                "Monday", "Tuesday",  // "Wednesday",
                "Thursday", // "Friday",
                "Saturday", "Sunday",
            ],
            m.capacity(),
            Hash128 {},
        );

        assert_eq!(m.nodes.len(), 5);
        assert_eq!(m.lookup.len(), 701);
        assert!(m.lookup.iter().all(|&n| n < m.nodes.len() as isize));

        assert_eq!(*m.get("alice"), "Monday");
        assert_eq!(*m.get("bob"), "Sunday");
    }
}
