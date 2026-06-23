#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeaderName {
    value: String,
}

impl HeaderName {
    pub fn new(value: &str) -> Result<Self, String> {
        if value.is_empty() || !value.bytes().all(is_header_name_byte) {
            return Err(format!("invalid HTTP header name: {value}"));
        }
        Ok(Self {
            value: value.to_ascii_lowercase(),
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

fn is_header_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}
