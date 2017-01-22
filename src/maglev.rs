use std::mem;
use std::iter;
use std::hash::{Hash, Hasher, BuildHasher, BuildHasherDefault};
use std::collections::hash_map::DefaultHasher;

use primal::Sieve;

use conshash::ConsistentHasher;

#[derive(Clone)]
pub struct Maglev<N, S> {
    nodes: Vec<N>,
    lookup: Vec<usize>,
    hash_builder: S,
}

impl<'a, N: 'a + Hash + Clone> Maglev<N, BuildHasherDefault<DefaultHasher>> {
    pub fn new<I: Into<&'a [N]>>(nodes: I) -> Self {
        Maglev::with_capacity_and_hasher(nodes, 0, Default::default())
    }

    pub fn with_capacity<I: Into<&'a [N]>>(nodes: I, capacity: usize) -> Self {
        Maglev::with_capacity_and_hasher(nodes, capacity, Default::default())
    }
}

impl<'a, N: 'a + Hash + Clone, S: BuildHasher> Maglev<N, S> {
    pub fn with_hasher<I: Into<&'a [N]>>(nodes: I, hash_builder: S) -> Self {
        Maglev::with_capacity_and_hasher(nodes, 0, hash_builder)
    }

    pub fn with_capacity_and_hasher<I: Into<&'a [N]>>(nodes: I,
                                                      capacity: usize,
                                                      hash_builder: S)
                                                      -> Self {
        let nodes = Vec::from(nodes.into());
        let lookup = Self::populate(&nodes,
                                    if capacity > 0 {
                                        capacity
                                    } else {
                                        nodes.len() * 100
                                    },
                                    &hash_builder);

        Maglev {
            nodes: nodes,
            lookup: lookup,
            hash_builder: hash_builder,
        }
    }
}

impl<'a, N: Hash, S: BuildHasher> ConsistentHasher<N> for Maglev<N, S> {
    #[inline]
    fn capacity(&self) -> usize {
        self.lookup.len()
    }

    #[inline]
    fn get<Q: Hash>(&self, key: &Q) -> &N {
        let key = Self::hash_with_seed(key, 0xdeadbabe, &self.hash_builder);

        &self.nodes[self.lookup[key % self.lookup.len()]]
    }
}

impl<'a, N: 'a + Hash, S: BuildHasher> Maglev<N, S> {
    #[inline]
    fn hash_with_seed<Q: Hash>(key: &Q, seed: u32, hash_builder: &S) -> usize {
        let mut hasher = hash_builder.build_hasher();
        hasher.write_u32(seed);
        key.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn populate(nodes: &Vec<N>, capacity: usize, hash_builder: &S) -> Vec<usize> {
        let m = Sieve::new(capacity * 2).primes_from(capacity).next().unwrap();
        let n = nodes.len();

        let permutation: Vec<Vec<usize>> = nodes.iter()
            .map(|node| {
                let offset = Self::hash_with_seed(&node, 0xdeadbabe, &hash_builder) % m;
                let skip = (Self::hash_with_seed(&node, 0xdeadbeef, &hash_builder) % (m - 1)) + 1;

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

        unsafe { mem::transmute(entry) }
    }
}

#[cfg(test)]
pub mod tests {
    use fasthash::spooky::SpookyHash128;

    use super::*;
    use conshash::ConsistentHasher;

    #[test]
    fn test_maglev() {
        let m = Maglev::new(&["Monday",
                              "Tuesday",
                              "Wednesday",
                              "Thursday",
                              "Friday",
                              "Saturday",
                              "Sunday"][..]);

        assert_eq!(m.nodes.len(), 7);
        assert_eq!(m.lookup.len(), 701);
        assert!(m.lookup.iter().all(|&n| n < m.nodes.len()));

        assert_eq!(*m.get(&"alice"), "Friday");
        assert_eq!(*m.get(&"bob"), "Wednesday");

        let m = Maglev::with_capacity(&["Monday",
                                        "Tuesday",
                                        "Wednesday",
                                        // "Thursday",
                                        "Friday",
                                        "Saturday",
                                        "Sunday"][..],
                                      m.capacity());

        assert_eq!(m.nodes.len(), 6);
        assert_eq!(m.lookup.len(), 701);
        assert!(m.lookup.iter().all(|&n| n < m.nodes.len()));

        assert_eq!(*m.get(&"alice"), "Friday");
        assert_eq!(*m.get(&"bob"), "Wednesday");

        let m = Maglev::with_capacity(&["Monday",
                                        // "Tuesday",
                                        "Wednesday",
                                        // "Thursday",
                                        "Friday",
                                        "Saturday",
                                        "Sunday"][..],
                                      m.capacity());

        assert_eq!(m.nodes.len(), 5);
        assert_eq!(m.lookup.len(), 701);
        assert!(m.lookup.iter().all(|&n| n < m.nodes.len()));

        assert_eq!(*m.get(&"alice"), "Friday");
        assert_eq!(*m.get(&"bob"), "Wednesday");

        let m = Maglev::with_capacity(&["Monday",
                                        "Tuesday",
                                        "Wednesday",
                                        // "Thursday",
                                        // "Friday",
                                        "Saturday",
                                        "Sunday"][..],
                                      m.capacity());

        assert_eq!(m.nodes.len(), 5);
        assert_eq!(m.lookup.len(), 701);
        assert!(m.lookup.iter().all(|&n| n < m.nodes.len()));

        assert_eq!(*m.get(&"alice"), "Saturday");
        assert_eq!(*m.get(&"bob"), "Wednesday");
    }

    #[test]
    fn test_maglev_with_custom_hasher() {
        let m = Maglev::with_hasher(&["Monday",
                                      "Tuesday",
                                      "Wednesday",
                                      "Thursday",
                                      "Friday",
                                      "Saturday",
                                      "Sunday"][..],
                                    SpookyHash128 {});

        assert_eq!(m.nodes.len(), 7);
        assert_eq!(m.lookup.len(), 701);
        assert!(m.lookup.iter().all(|&n| n < m.nodes.len()));

        assert_eq!(*m.get(&"alice"), "Monday");
        assert_eq!(*m.get(&"bob"), "Wednesday");

        let m = Maglev::with_capacity_and_hasher(&["Monday",
                                                   "Tuesday",
                                                   // "Wednesday",
                                                   "Thursday",
                                                   // "Friday",
                                                   "Saturday",
                                                   "Sunday"][..],
                                                 m.capacity(),
                                                 SpookyHash128 {});

        assert_eq!(m.nodes.len(), 5);
        assert_eq!(m.lookup.len(), 701);
        assert!(m.lookup.iter().all(|&n| n < m.nodes.len()));

        assert_eq!(*m.get(&"alice"), "Monday");
        assert_eq!(*m.get(&"bob"), "Sunday");
    }
}
