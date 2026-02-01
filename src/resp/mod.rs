mod error;
mod parser;
mod value;

use parser::Parser;

pub use error::ParseError;
pub use value::Value;

pub fn parse_resp(input: &[u8]) -> Result<(Value, usize), ParseError> {
    let mut parser = Parser::new(input);
    let parsed = parser.parse()?;
    Ok((parsed, parser.bytes_consumed()))
}

impl From<ParseError> for Value {
    fn from(value: ParseError) -> Self {
        Value::SimpleError(value.to_string())
    }
}
