use std::time::Duration;

use super::raw::RawCommand;
use thiserror::Error;

#[derive(Debug)]
pub enum Command {
    Ping(Option<String>),
    Echo(String),
    Get(String),
    Set {
        key: String,
        value: String,
        expiry: Option<Duration>,
    },
}

#[derive(Debug, Error)]
pub(crate) enum CommandValidationError {
    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("ECHO command requires a message")]
    EchoRequiresMessage,

    #[error("GET command requires a key to retrieve value")]
    GetRequiresKey,

    #[error("SET command requires a key and a value to set")]
    SetRequiresKeyValue,

    #[error("Unknown SET command flag: {0}")]
    UnknownSetFlag(String),

    #[error("Missing value for flag: {0}")]
    MissingFlagValue(String),

    #[error("Invalid flag value: {0}. Expected {1}")]
    InvalidFlagValue(String, &'static str),

    #[error("Unexpected trailling arguments")]
    UnexpectedTrailingArguments,
}

impl TryFrom<RawCommand> for Command {
    type Error = CommandValidationError;
    fn try_from(value: RawCommand) -> Result<Self, Self::Error> {
        match value.name.as_str() {
            "PING" => Ok(Command::Ping(value.args.into_iter().next())),
            "ECHO" => {
                let msg = value
                    .args
                    .into_iter()
                    .next()
                    .ok_or(CommandValidationError::EchoRequiresMessage)?;
                Ok(Command::Echo(msg))
            }
            "GET" => {
                let key = value
                    .args
                    .into_iter()
                    .next()
                    .ok_or(CommandValidationError::GetRequiresKey)?;
                Ok(Command::Get(key))
            }
            "SET" => parse_set(value.args),
            _ => Err(CommandValidationError::UnknownCommand(value.name)),
        }
    }
}

fn parse_set(args: Vec<String>) -> Result<Command, CommandValidationError> {
    let mut iter = args.into_iter();
    let key = iter
        .next()
        .ok_or(CommandValidationError::SetRequiresKeyValue)?;
    let value = iter
        .next()
        .ok_or(CommandValidationError::SetRequiresKeyValue)?;

    let expiry = iter
        .next()
        .map(|flag| {
            let to_duration = if flag.eq_ignore_ascii_case("EX") {
                Duration::from_secs
            } else if flag.eq_ignore_ascii_case("PX") {
                Duration::from_millis
            } else {
                return Err(CommandValidationError::UnknownSetFlag(flag));
            };
            let duration = iter
                .next()
                .ok_or_else(|| CommandValidationError::MissingFlagValue(flag.to_owned()))?
                .parse::<u64>()
                .map(to_duration)
                .map_err(|_| CommandValidationError::InvalidFlagValue(flag, "integer"))?;
            Ok(duration)
        })
        .transpose()?;

    if iter.next().is_some() {
        return Err(CommandValidationError::UnexpectedTrailingArguments);
    }

    Ok(Command::Set { key, value, expiry })
}
