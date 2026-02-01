use super::{error::ParseError, value::Value};

pub(super) struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    fn consume(&mut self) -> Result<u8, ParseError> {
        let current = self
            .input
            .get(self.pos)
            .copied()
            .ok_or(ParseError::UnexpectedEOF)?;
        self.pos += 1;
        Ok(current)
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), ParseError> {
        let actual = self.consume()?;
        if actual != expected {
            return Err(ParseError::ExpectedByte { actual, expected });
        }
        Ok(())
    }

    fn expect_crlf(&mut self) -> Result<(), ParseError> {
        self.expect_byte(b'\r')?;
        self.expect_byte(b'\n')
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
        if self.pos + n > self.input.len() {
            return Err(ParseError::UnexpectedEOF);
        }
        let bytes = &self.input[self.pos..self.pos + n];
        self.pos += n;
        Ok(bytes)
    }

    fn read_usize(&mut self) -> Result<usize, ParseError> {
        let line = self.read_line()?;
        let s = std::str::from_utf8(line).map_err(ParseError::InvalidUtf8)?;
        Ok(s.parse::<usize>()?)
    }

    fn parse_simple_string(&mut self) -> Result<Value, ParseError> {
        let line = self.read_line()?;
        let s = std::str::from_utf8(line).map_err(ParseError::InvalidUtf8)?;
        Ok(Value::SimpleString(s.to_string()))
    }

    fn parse_bulk_string(&mut self) -> Result<Value, ParseError> {
        let n = self.read_usize()?;
        let bytes = self.read_bytes(n)?;
        let s = std::str::from_utf8(bytes).map_err(ParseError::InvalidUtf8)?;
        self.expect_crlf()?;
        Ok(Value::BulkString(s.to_string()))
    }

    fn parse_array(&mut self) -> Result<Value, ParseError> {
        let n = self.read_usize()?;
        // Protect from user attempting to allocate huge values
        let remaining = self.input.len() - self.pos;
        if n > remaining {
            return Err(ParseError::UnexpectedEOF);
        }
        let mut values = Vec::with_capacity(n);
        for _ in 0..n {
            let value = self.parse()?;
            values.push(value)
        }
        Ok(Value::Array(values))
    }

    fn parse_error(&mut self) -> Result<Value, ParseError> {
        let line = self.read_line()?;
        let s = std::str::from_utf8(line).map_err(ParseError::InvalidUtf8)?;
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

    pub fn bytes_consumed(&self) -> usize {
        self.pos
    }
}
