mod command;
mod execute;
mod raw;

use crate::resp::Value;
pub use command::Command;
use command::CommandValidationError;
use raw::{RawCommand, RawCommandError};

pub use execute::execute;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InterpretError {
    #[error("Error parsing input command: {0}")]
    RawCommandError(#[from] RawCommandError),
    #[error("Invalid command: {0}")]
    CommandValidationError(#[from] CommandValidationError),
}

impl From<InterpretError> for Value {
    fn from(value: InterpretError) -> Self {
        Value::SimpleError(value.to_string())
    }
}

pub fn interpret_command(value: Value) -> Result<Command, InterpretError> {
    let raw_command: RawCommand = value.try_into().map_err(InterpretError::RawCommandError)?;
    raw_command
        .try_into()
        .map_err(InterpretError::CommandValidationError)
}
