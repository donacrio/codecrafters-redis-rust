mod error;
mod parser;
mod value;

use parser::Parser;

pub use error::ParseError;
pub use value::Value;

pub fn parse_resp(input: &[u8]) -> Result<Value, ParseError> {
    Parser::new(input).parse()
}

impl From<ParseError> for Value {
    fn from(value: ParseError) -> Self {
        Value::SimpleError(value.to_string())
    }
}
