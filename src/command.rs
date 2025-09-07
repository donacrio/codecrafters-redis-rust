use crate::resp::{Command, CommandError, Value};

pub fn execute(command: Command) -> Result<Value, CommandError> {
    match command {
        Command::Ping(message) => execute_ping(message),
        Command::Echo(message) => execute_echo(message),
    }
}

fn execute_ping(message: Option<String>) -> Result<Value, CommandError> {
    match message {
        Some(content) => Ok(Value::BulkString(content)),
        None => Ok(Value::SimpleString("PONG".to_string())),
    }
}

fn execute_echo(message: String) -> Result<Value, CommandError> {
    Ok(Value::BulkString(message))
}
