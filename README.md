# rust-maglev [![travis build](https://api.travis-ci.org/flier/rust-maglev.svg)](https://travis-ci.org/flier/rust-maglev) [![crate](https://img.shields.io/crates/v/maglev.svg)](https://crates.io/crates/maglev) [![docs](https://docs.rs/maglev/badge.svg)](https://docs.rs/maglev/)
Google's consistent hashing algorithm

## Usage

To use `maglev`, first add this to your `Cargo.toml`:

```toml
[dependencies]
maglev = "0.2"
```

And then, use `Maglev` with `ConsistentHasher` trait

```rust
use maglev::{ConsistentHasher, Maglev};

fn main() {
    let m = Maglev::new(vec!["Monday",
                            "Tuesday",
                            "Wednesday",
                            "Thursday",
                            "Friday",
                            "Saturday",
                            "Sunday"]);

    assert_eq!(m["alice"], "Friday");
    assert_eq!(m["bob"], "Wednesday");

    // When the node list changed, ensure to use same `capacity` to rebuild

    let m = Maglev::with_capacity(vec!["Monday",
                                  // "Tuesday",
                                    "Wednesday",
                                  // "Thursday",
                                    "Friday",
                                    "Saturday",
                                    "Sunday"],
                                m.capacity());

    assert_eq!(m["alice"], "Friday");
    assert_eq!(m["bob"], "Wednesday");
}
```

Maglev use `std::collections::hash_map::DefaultHasher` by default, we could use the given hash builder to hash keys.

```rust
use fasthash::spooky::Hash128;
use maglev::Maglev;

fn main() {
    let m = Maglev::with_hasher(vec!["Monday",
                                     "Tuesday",
                                     "Wednesday",
                                     "Thursday",
                                     "Friday",
                                     "Saturday",
                                     "Sunday"],
                                Hash128 {});

    assert_eq!(m["alice"], "Monday");
    assert_eq!(m["bob"], "Wednesday");
}
```