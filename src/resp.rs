use thiserror::Error;

#[derive(Debug)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    SimpleError(String),
    Array(Vec<Self>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SimpleString(s) => write!(f, "+{}\r\n", s),
            Self::BulkString(s) => write!(f, "${}\r\n{}\r\n", s.len(), s),
            Self::SimpleError(s) => write!(f, "-{}\r\n", s),
            Self::Array(values) => {
                write!(f, "*{}\r\n", values.len())?;
                for value in values {
                    write!(f, "{}", value)?
                }
                Ok(())
            }
        }
    }
}

impl From<CommandError> for Value {
    fn from(value: CommandError) -> Self {
        Value::SimpleError(value.to_string())
    }
}

#[derive(Debug)]
pub enum Command {
    Ping(Option<String>),
    Echo(String),
}

impl TryFrom<Value> for Command {
    type Error = CommandError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let (cmd_name, args) = match value {
            Value::SimpleString(s) | Value::BulkString(s) => match s.split_once(' ') {
                Some((cmd_name, args)) => Ok((cmd_name.to_owned(), vec![args.to_owned()])),
                None => Ok((s, vec![])),
            },
            Value::Array(values) => {
                let mut strings = values
                    .into_iter()
                    .map(|value| match value {
                        Value::SimpleString(s) | Value::BulkString(s) => Ok(s),
                        value => Err(CommandError::ExpectedString(value)),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let cmd_name = strings.get(0).ok_or(CommandError::EmptyCommand)?.to_owned();
                let args = strings.split_off(1);
                Ok((cmd_name, args))
            }
            Value::SimpleError(_) => Err(CommandError::CannotExecuteError),
        }?;

        match cmd_name.to_uppercase().as_str() {
            "PING" => Ok(Command::Ping(args.into_iter().next())),
            "ECHO" => {
                let msg = args
                    .into_iter()
                    .next()
                    .ok_or(CommandError::EchoRequiresMessage)?;
                Ok(Command::Echo(msg))
            }
            _ => Err(CommandError::UnknownCommand(cmd_name)),
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected byte {0}")]
    UnexpectedByte(u8),

    #[error("Expected byte {expected}, got {actual}")]
    ExpectedByte { actual: u8, expected: u8 },

    #[error("Unexpected EOF")]
    UnexpectedEOF,

    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),

    #[error("Error parsing integer: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Error parsing command: {0}")]
    ParseError(#[from] ParseError),

    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("ECHO command requires a message")]
    EchoRequiresMessage,

    #[error("Empty command array")]
    EmptyCommand,

    #[error("Expected string argument, got {0:?}")]
    ExpectedString(Value),

    #[error("Cannot execute error value")]
    CannotExecuteError,
}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    fn consume(&mut self) -> Result<u8, ParseError> {
        let current = self.input.get(self.pos).ok_or(ParseError::UnexpectedEOF)?;
        self.pos += 1;
        Ok(*current)
    }

    fn expect_crlf(&mut self) -> Result<(), ParseError> {
        match self.consume()? {
            b'\r' => match self.consume()? {
                b'\n' => Ok(()),
                byte => Err(ParseError::ExpectedByte {
                    actual: byte,
                    expected: b'\n',
                }),
            },
            byte => Err(ParseError::ExpectedByte {
                actual: byte,
                expected: b'\r',
            }),
        }
    }

    fn read_line(&mut self) -> Result<&'a [u8], ParseError> {
        let remaining = &self.input[self.pos..];
        let crlf_pos = remaining
            .windows(2)
            .position(|t| t == b"\r\n")
            .ok_or(ParseError::UnexpectedEOF)?;
        let line = &remaining[..crlf_pos];
        self.pos += crlf_pos + 2;
        Ok(line)
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], ParseError> {
        if self.input.len() - self.pos < n {
            return Err(ParseError::UnexpectedEOF);
        }
        let bytes = &self.input[self.pos..self.pos + n];
        self.pos += n;
        Ok(bytes)
    }

    fn read_usize(&mut self) -> Result<usize, ParseError> {
        let line = self.read_line()?;
        let s = std::str::from_utf8(line).map_err(|e| ParseError::InvalidUtf8(e))?;
        Ok(s.parse::<usize>()?)
    }

    fn parse_simple_string(&mut self) -> Result<Value, ParseError> {
        let line = self.read_line()?;
        let s = std::str::from_utf8(line).map_err(|e| ParseError::InvalidUtf8(e))?;
        self.expect_crlf()?;
        Ok(Value::SimpleString(s.to_string()))
    }

    fn parse_bulk_string(&mut self) -> Result<Value, ParseError> {
        let n = self.read_usize()?;
        let bytes = self.read_bytes(n)?;
        let s = std::str::from_utf8(bytes).map_err(|e| ParseError::InvalidUtf8(e))?;
        self.expect_crlf()?;
        Ok(Value::BulkString(s.to_string()))
    }

    fn parse_array(&mut self) -> Result<Value, ParseError> {
        let n = self.read_usize()?;
        let mut values = Vec::with_capacity(n);
        for _ in 0..n {
            let value = self.parse()?;
            values.push(value)
        }
        Ok(Value::Array(values))
    }

    fn parse_error(&mut self) -> Result<Value, ParseError> {
        let line = self.read_line()?;
        let s = std::str::from_utf8(line).map_err(|e| ParseError::InvalidUtf8(e))?;
        Ok(Value::SimpleError(s.to_string()))
    }

    pub fn parse(&mut self) -> Result<Value, ParseError> {
        match self.consume()? {
            b'+' => self.parse_simple_string(),
            b'$' => self.parse_bulk_string(),
            b'*' => self.parse_array(),
            b'-' => self.parse_error(),
            byte => Err(ParseError::UnexpectedByte(byte)),
        }
    }
}

pub fn parse_command(input: &'_ [u8]) -> Result<Command, CommandError> {
    let mut parser = Parser::new(input);
    let parsed = parser.parse()?;
    println!("{:}", parsed);
    parsed.try_into()
}
