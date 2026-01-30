use thiserror::Error;

use crate::resp::Value;

pub(super) struct RawCommand {
    pub(super) name: String,
    pub(super) args: Vec<String>,
}

#[derive(Debug, Error)]
pub enum RawCommandError {
    #[error("Expected string argument, got {0}")]
    ExpectedString(String),

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
                    value => Err(RawCommandError::ExpectedString(value.to_string())),
                });
                let cmd_name = values.next().ok_or(RawCommandError::EmptyCommandArray)??;
                let args = values.collect::<Result<Vec<_>, _>>()?;
                Ok(RawCommand {
                    name: cmd_name.to_uppercase(),
                    args,
                })
            }
            Value::SimpleString(s) | Value::BulkString(s) => match s.split_once(' ') {
                Some((cmd_name, args)) => Ok(RawCommand {
                    name: cmd_name.to_string(),
                    args: args.split(' ').map(|s| s.to_string()).collect(),
                }),
                None => Ok(RawCommand {
                    name: s.to_uppercase(),
                    args: vec![],
                }),
            },
            value => Err(RawCommandError::CannotCreateCommand(value.to_string())),
        }
    }
}
