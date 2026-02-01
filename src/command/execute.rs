use std::time::Duration;

use crate::command::Command;
use crate::{resp::Value, store::Store};

pub(super) fn execute(command: Command, store: &Store) -> Value {
    match command {
        Command::Ping(message) => execute_ping(message),
        Command::Echo(message) => execute_echo(message),
        Command::Get(key) => execute_get(&key, store),
        Command::Set { key, value, expiry } => execute_set(key, value, expiry, store),
    }
}

fn execute_ping(message: Option<String>) -> Value {
    message.map_or_else(|| Value::SimpleString("PONG".into()), Value::BulkString)
}

fn execute_echo(message: String) -> Value {
    Value::BulkString(message)
}

fn execute_get(key: &str, store: &Store) -> Value {
    store.get(key).map(Value::BulkString).unwrap_or(Value::Null)
}

fn execute_set(key: String, value: String, expiry: Option<Duration>, store: &Store) -> Value {
    store.set(key, value, expiry);
    Value::SimpleString("OK".into())
}
