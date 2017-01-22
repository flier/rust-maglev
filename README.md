# rust-maglev [![travis build](https://travis-ci.org/flier/rust-maglev.svg?branch=master)](https://travis-ci.org/flier/rust-maglev) [![crate](https://img.shields.io/crates/v/maglev.svg)](https://crates.io/crates/maglev)
Google's consistent hashing algorithm

[API Document](https://docs.rs/maglev)

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