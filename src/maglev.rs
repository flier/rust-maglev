use std::collections::hash_map::DefaultHasher;
use std::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};
use std::iter;
use std::ops::Index;

use primal::Sieve;

use crate::conshash::ConsistentHasher;

/// Maglev lookup table
#[derive(Clone, Debug)]
pub struct Maglev<N, S = BuildHasherDefault<DefaultHasher>> {
    nodes: Vec<N>,
    lookup: Option<Vec<isize>>,
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
        let lookup = Self::populate(&nodes, capacity, &hash_builder);

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

    fn populate(nodes: &[N], mut capacity: usize, hash_builder: &S) -> Option<Vec<isize>> {
        if nodes.is_empty() {
            return None;
        }
        if capacity == 0 {
            capacity = nodes.len() * 100
        }
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

        let mut next: Vec<usize> = vec![0; n];
        let mut entry: Vec<isize> = vec![-1; m];

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

        Some(entry)
    }
}

impl<N: Hash + Eq> iter::FromIterator<N> for Maglev<N, BuildHasherDefault<DefaultHasher>> {
    fn from_iter<T: IntoIterator<Item = N>>(iter: T) -> Self {
        Maglev::new(iter)
    }
}

impl<N, S> ConsistentHasher<N> for Maglev<N, S>
where
    N: Hash + Eq,
    S: BuildHasher,
{
    #[inline]
    fn nodes(&self) -> &[N] {
        self.nodes.as_slice()
    }

    #[inline]
    fn capacity(&self) -> usize {
        self.lookup.as_ref().map(|m| m.len()).unwrap_or_default()
    }

    #[inline]
    fn get<Q: ?Sized>(&self, key: &Q) -> Option<&N>
    where
        Q: Hash + Eq,
    {
        self.lookup.as_ref().map(|lookup| {
            let key = Self::hash_with_seed(key, 0xdead_babe, &self.hash_builder);

            &self.nodes[lookup[key % lookup.len()] as usize]
        })
    }
}

impl<N, S, Q> Index<&Q> for Maglev<N, S>
where
    N: Hash + Eq,
    S: BuildHasher,
    Q: Hash + Eq + ?Sized,
{
    type Output = N;

    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index).unwrap()
    }
}

#[cfg(test)]
pub mod tests {
    use fasthash::spooky::Hash128;

    use super::*;
    use crate::conshash::ConsistentHasher;

    include!(concat!(env!("OUT_DIR"), "/skeptic-tests.rs"));

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
        assert_eq!(m.lookup.as_ref().unwrap().len(), 701);
        assert!(m
            .lookup
            .as_ref()
            .unwrap()
            .iter()
            .all(|&n| n < m.nodes.len() as isize));

        assert_eq!(m["alice"], "Friday");
        assert_eq!(m["bob"], "Wednesday");

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
        assert_eq!(m.lookup.as_ref().unwrap().len(), 701);
        assert!(m
            .lookup
            .as_ref()
            .unwrap()
            .iter()
            .all(|&n| n < m.nodes.len() as isize));

        assert_eq!(m["alice"], "Friday");
        assert_eq!(m["bob"], "Wednesday");

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
        assert_eq!(m.lookup.as_ref().unwrap().len(), 701);
        assert!(m
            .lookup
            .as_ref()
            .unwrap()
            .iter()
            .all(|&n| n < m.nodes.len() as isize));

        assert_eq!(m["alice"], "Friday");
        assert_eq!(m["bob"], "Wednesday");

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
        assert_eq!(m.lookup.as_ref().unwrap().len(), 701);
        assert!(m
            .lookup
            .as_ref()
            .unwrap()
            .iter()
            .all(|&n| n < m.nodes.len() as isize));

        assert_eq!(m["alice"], "Saturday");
        assert_eq!(m["bob"], "Wednesday");
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
        assert_eq!(m.lookup.as_ref().unwrap().len(), 701);
        assert!(m
            .lookup
            .as_ref()
            .unwrap()
            .iter()
            .all(|&n| n < m.nodes.len() as isize));

        assert_eq!(m["alice"], "Monday");
        assert_eq!(m["bob"], "Wednesday");

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
        assert_eq!(m.lookup.as_ref().unwrap().len(), 701);
        assert!(m
            .lookup
            .as_ref()
            .unwrap()
            .iter()
            .all(|&n| n < m.nodes.len() as isize));

        assert_eq!(m["alice"], "Monday");
        assert_eq!(m["bob"], "Sunday");
    }

    #[test]
    fn test_maglev_with_empty_list() {
        let m = Maglev::<&str, _>::new(None);

        assert_eq!(m.nodes.len(), 0);
        assert!(m.lookup.is_none());

        assert_eq!(m.get("alice"), None);
    }
}
