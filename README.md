# rust-maglev [![travis build](https://api.travis-ci.org/flier/rust-maglev.svg)](https://travis-ci.org/flier/rust-maglev) [![crate](https://img.shields.io/crates/v/maglev.svg)](https://crates.io/crates/maglev) [![docs](https://docs.rs/maglev/badge.svg)](https://docs.rs/maglev/)
Google's consistent hashing algorithm

# Usage

To use `maglev`, first add this to your `Cargo.toml`:

```toml
[dependencies]
maglev = "0.1"
```

Then, add this to your crate root:

```rust
extern crate maglev;

use maglev::*;
```

And then, use `Maglev` with `ConsistentHasher` trait

```rust
let m = Maglev::new(&["Monday",
                      "Tuesday",
                      "Wednesday",
                      "Thursday",
                      "Friday",
                      "Saturday",
                      "Sunday"][..]);

assert_eq!(*m.get(&"alice"), "Friday");
assert_eq!(*m.get(&"bob"), "Wednesday");
```

When the node list changed, ensure to use same `capacity` to rebuild

```rust
let m = Maglev::with_capacity(&["Monday",
                                // "Tuesday",
                                "Wednesday",
                                // "Thursday",
                                "Friday",
                                "Saturday",
                                "Sunday"][..],
                              m.capacity());

assert_eq!(*m.get(&"alice"), "Friday");
assert_eq!(*m.get(&"bob"), "Wednesday");
```

Maglev will `std::collections::hash_map::DefaultHasher` by default, we could use the given hash builder to hash keys.

```rust
use fasthash::spooky::SpookyHash128;

let m = Maglev::with_hasher(&["Monday",
                              "Tuesday",
                              "Wednesday",
                              "Thursday",
                              "Friday",
                              "Saturday",
                              "Sunday"][..],
                            SpookyHash128 {});

assert_eq!(*m.get(&"alice"), "Monday");
assert_eq!(*m.get(&"bob"), "Wednesday");
```