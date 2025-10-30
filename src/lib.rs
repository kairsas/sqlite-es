#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![warn(rust_2018_idioms)]
#![doc = include_str!("../README.md")]
//! # sqlite-es
//!
//! > A Sqlite implementation of the `EventStore` trait in [cqrs-es](https://crates.io/crates/cqrs-es).
//!
pub use crate::cqrs::*;
pub use crate::event_repository::*;
pub use crate::init::*;
pub use crate::testing::*;
pub use crate::types::*;
pub use crate::view_repository::*;

mod cqrs;
mod error;
mod event_repository;
mod init;
pub(crate) mod sql_query;
pub(crate) mod testing;
mod types;
mod view_repository;
