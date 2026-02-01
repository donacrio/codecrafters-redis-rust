mod command;
mod execute;
mod raw;

use crate::{resp::Value, store::Store};
pub use command::Command;
use raw::RawCommand;

use execute::execute;

fn to_error_value(e: impl std::fmt::Display) -> Value {
    Value::SimpleError(e.to_string())
}

fn interpret_command(value: Value) -> Result<Command, Value> {
    let raw: RawCommand = value.try_into().map_err(to_error_value)?;
    raw.try_into().map_err(to_error_value)
}

pub(crate) fn handle_value(value: Value, store: &Store) -> Value {
    interpret_command(value)
        .map(|command| execute(command, store))
        .unwrap_or_else(|e| e)
}
