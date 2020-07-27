use std::error::Error;
use std::fmt::{Display, Formatter, Result};

use serde::de;

#[derive(Clone, Debug, PartialEq)]
pub enum VersionParsingError {
    // One or more variants that can be created by data structures through the
    // `ser::Error` and `de::Error` traits. For example the Serialize impl for
    // Mutex<T> might return an error because the mutex is poisoned, or the
    // Deserialize impl for a struct may return an error because a required
    // field is missing.
    Message(String),

    // Zero or more variants that can be created directly by the Serializer and
    // Deserializer without going through `ser::Error` and `de::Error`. These
    // are specific to the format, in this case JSON.
    DisallowedCharacter,
    DuplicatedEpochCharacter,
    DuplicatedLocalSeparatorCharacter,
    UnknownParseError,
}

impl de::Error for VersionParsingError {
    fn custom<T: Display>(msg: T) -> Self {
        VersionParsingError::Message(msg.to_string())
    }
}

impl Display for VersionParsingError {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        formatter.write_str(Error::description(self))
    }
}

// This is important for other errors to wrap this one.
impl Error for VersionParsingError {
    fn description(&self) -> &str {
        match *self {
            VersionParsingError::Message(ref msg) => msg,
            VersionParsingError::DisallowedCharacter => "Disallowed character in string",
            VersionParsingError::DuplicatedEpochCharacter => "Duplicate epoch character (!)",
            VersionParsingError::DuplicatedLocalSeparatorCharacter => {
                "duplicated local version separator (+)"
            }
            VersionParsingError::UnknownParseError => "Unknown parse error",
        }
    }
}
