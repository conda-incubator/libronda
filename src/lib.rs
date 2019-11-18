//! A Rust library to handle performance-critical parts of conda
//!
//! ## Features
//!
//! ## Examples
//!
#[macro_use] extern crate lazy_static;
#[cfg(test)]
#[macro_use] extern crate rstest;

mod version;
mod repodata;

// Reexports
pub use crate::version::CompOp;
pub use crate::version::Version;
pub use crate::version::VersionPart;
pub use crate::version::conda_parser;
