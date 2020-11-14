//! Maglev hashing - A consistent hashing algorithm from Google
//!
//! [Maglev: A Fast and Reliable Software Network Load Balancer](https://static.googleusercontent.com/media/research.google.com/zh-CN//pubs/archive/44824.pdf)
//!
//! # Example
//!
//! ```rust
//! use maglev::*;
//!
//! let m = Maglev::new(vec!["Monday",
//!                          "Tuesday",
//!                          "Wednesday",
//!                          "Thursday",
//!                          "Friday",
//!                          "Saturday",
//!                          "Sunday"]);
//!
//! assert_eq!(m["alice"], "Friday");
//! assert_eq!(m["bob"], "Wednesday");
//!
//! // When the node list changed, ensure to use same `capacity` to rebuild the lookup table.
//!
//! let m = Maglev::with_capacity(vec!["Monday",
//!                                 // "Tuesday",
//!                                    "Wednesday",
//!                                 // "Thursday",
//!                                    "Friday",
//!                                    "Saturday",
//!                                    "Sunday"],
//!                               m.capacity());
//!
//! assert_eq!(m["alice"], "Friday");
//! assert_eq!(m["bob"], "Wednesday");
//! ```
//!
//! Maglev use `std::collections::hash_map::DefaultHasher` by default,
//! we could use the given hash builder to hash keys.
//!
//! ```rust
//! extern crate fasthash;
//! extern crate maglev;
//!
//! use fasthash::spooky::Hash128;
//!
//! use maglev::*;
//!
//! fn main() {
//!     let m = Maglev::with_hasher(vec!["Monday",
//!                                      "Tuesday",
//!                                      "Wednesday",
//!                                      "Thursday",
//!                                      "Friday",
//!                                      "Saturday",
//!                                      "Sunday"],
//!                                 Hash128 {});
//!
//!     assert_eq!(m["alice"], "Monday");
//!     assert_eq!(m["bob"], "Wednesday");
//! }
//! ```
mod conshash;
mod maglev;

pub use crate::conshash::ConsistentHasher;
pub use crate::maglev::Maglev;
