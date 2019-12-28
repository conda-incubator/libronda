//! A Rust library to handle performance-critical parts of conda
//!
//! ## Features
//!
//! ## Examples
//!

//#![feature(async_await)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate cpython;
#[cfg(test)]
#[macro_use] extern crate rstest;

mod version;
mod repodata;
// mod graph;
mod python_interface;
// mod resolve;

// Reexports
pub use crate::version::CompOp;
pub use crate::version::Version;
pub use crate::version::VersionPart;
pub use crate::version::conda_parser;
pub use crate::version::matching::{untreeify, treeify, ConstraintTree, StringOrConstraintTree, Combinator};
pub use crate::repodata::repodata::{Repodata, RepodataInfo, Record};