use crate::command::Command;
use crate::{resp::Value, store::Store};

pub fn execute(command: Command, store: &Store) -> Value {
    match command {
        Command::Ping(message) => execute_ping(message),
        Command::Echo(message) => execute_echo(message),
        Command::Get(key) => execute_get(&key, store),
        Command::Set(key, value) => execute_set(key, value, store),
    }
}

fn execute_ping(message: Option<String>) -> Value {
    match message {
        Some(content) => Value::BulkString(content),
        None => Value::SimpleString("PONG".to_string()),
    }
}

fn execute_echo(message: String) -> Value {
    Value::BulkString(message)
}

fn execute_get(key: &str, store: &Store) -> Value {
    match store.get(key) {
        Some(value) => Value::BulkString(value.to_owned()),
        None => Value::Null,
    }
}

fn execute_set(key: String, value: String, store: &Store) -> Value {
    store.set(key, value);
    Value::SimpleString("OK".to_string())
}
