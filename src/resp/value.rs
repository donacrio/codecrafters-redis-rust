#[derive(Debug)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    SimpleError(String),
    Array(Vec<Self>),
    Null,
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
            Self::Null => write!(f, "$-1\r\n"),
        }
    }
}
