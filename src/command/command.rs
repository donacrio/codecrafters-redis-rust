use super::raw::RawCommand;
use thiserror::Error;

#[derive(Debug)]
pub enum Command {
    Ping(Option<String>),
    Echo(String),
    Get(String),
    Set(String, String),
}

#[derive(Debug, Error)]
pub enum CommandValidationError {
    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("ECHO command requires a message")]
    EchoRequiresMessage,

    #[error("GET command requires a key to retrieve value")]
    GetRequiresKey,

    #[error("SET command requires a key and a value to set")]
    SetRequiresKeyValue,
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
            "SET" => {
                let mut iter = value.args.into_iter();
                let key = iter
                    .next()
                    .ok_or(CommandValidationError::SetRequiresKeyValue)?;
                let value = iter
                    .next()
                    .ok_or(CommandValidationError::SetRequiresKeyValue)?;
                Ok(Command::Set(key, value))
            }
            name => Err(CommandValidationError::UnknownCommand(name.to_string())),
        }
    }
}
