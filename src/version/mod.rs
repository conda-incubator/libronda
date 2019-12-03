// Applies only to the test module - not to below.  Load test module first because its macros
//   need to be defined before the other code gets compiled.
#[cfg(test)]
#[macro_use] pub mod test;

pub mod parsers;
pub mod custom_parts;
pub mod comp_op;
pub mod errors;
pub mod version;
pub mod version_part;
pub mod matching;

pub use self::comp_op::CompOp;
pub use self::version::Version;
pub use self::version_part::VersionPart;
pub use self::parsers::conda::conda_parser;