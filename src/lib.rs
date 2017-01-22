//! Maglev hashing - A consistent hashing algorithm from Google
//!
//! [Maglev: A Fast and Reliable Software Network Load Balancer]
//! (https://static.googleusercontent.com/media/research.google.com/zh-CN//pubs/archive/44824.pdf)
//!
//! # Example
//!
//! ```rust
//! use maglev::*;
//!
//! let m = Maglev::new(&["Monday",
//!                       "Tuesday",
//!                       "Wednesday",
//!                       "Thursday",
//!                       "Friday",
//!                       "Saturday",
//!                       "Sunday"][..]);
//!
//! assert_eq!(*m.get(&"alice"), "Friday");
//! assert_eq!(*m.get(&"bob"), "Wednesday");
//!
//! // When the node list changed, ensure to use same `capacity` to rebuild the lookup table.
//!
//! let m = Maglev::with_capacity(&["Monday",
//!                                 // "Tuesday",
//!                                 "Wednesday",
//!                                 // "Thursday",
//!                                 "Friday",
//!                                 "Saturday",
//!                                 "Sunday"][..],
//!                               m.capacity());
//!
//! assert_eq!(*m.get(&"alice"), "Friday");
//! assert_eq!(*m.get(&"bob"), "Wednesday");
//! ```
extern crate primal;

mod conshash;
mod maglev;

pub use conshash::ConsistentHasher;
pub use maglev::Maglev;
