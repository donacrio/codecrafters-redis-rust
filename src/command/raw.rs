use thiserror::Error;

use crate::resp::Value;

pub(super) struct RawCommand {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Error)]
pub(crate) enum RawCommandError {
    #[error("Expected string argument")]
    ExpectedString,

    #[error("Empty command array")]
    EmptyCommandArray,

    #[error("Cannot create command from value: {0}")]
    CannotCreateCommand(String),
}

impl TryFrom<Value> for RawCommand {
    type Error = RawCommandError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(values) => {
                let mut values = values.into_iter().map(|value| match value {
                    Value::SimpleString(s) | Value::BulkString(s) => Ok(s),
                    _ => Err(RawCommandError::ExpectedString),
                });
                let cmd_name = values.next().ok_or(RawCommandError::EmptyCommandArray)??;
                let args = values.collect::<Result<Vec<_>, _>>()?;
                Ok(RawCommand {
                    name: cmd_name.to_uppercase(),
                    args,
                })
            }
            Value::SimpleString(s) | Value::BulkString(s) => {
                let mut parts = s.split_ascii_whitespace();
                let name = parts
                    .next()
                    .ok_or(RawCommandError::EmptyCommandArray)?
                    .to_uppercase();
                let args = parts.map(|s| s.to_string()).collect();
                Ok(RawCommand { name, args })
            }
            value => Err(RawCommandError::CannotCreateCommand(value.to_string())),
        }
    }
}
