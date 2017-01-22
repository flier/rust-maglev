//! Maglev - Google's consistent hashing algorithm
//!
//! https://static.googleusercontent.com/media/research.google.com/zh-CN//pubs/archive/44824.pdf
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
//! ```
extern crate primal;

#[cfg(test)]
extern crate fasthash;

mod conshash;
mod maglev;

pub use conshash::ConsistentHasher;
pub use maglev::Maglev;
