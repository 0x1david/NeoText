use crate::{Error, Result};

const CRLF: &str = r"\r\n";
const CRLF_BYTE_LEN: usize = CRLF.len();

type LSPObject<'a> = &'a [(&'a str, LSPAny<'a>)];
type LSPArray<'a> = &'a [usize];

struct LspParser<'pl> {
    payload: &'pl str,
    start_pointer: usize,
    end_pointer: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header<'pl> {
    pub content_length: u16,
    pub content_type: Option<&'pl str>,
}
pub enum Body<'pl> {
    Request(Request<'pl>),
    Response(Response<'pl>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request<'pl> {
    jsonrpc: &'pl str,
    id: Option<usize>,
    method: &'pl str,
    // Only Object or Array
    params: Params<'pl>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response<'pl> {
    jsonrpc: &'pl str,
    id: Option<usize>,
    result: &'pl str,
    error: Option<&'pl str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LSPAny<'pl> {
    Object(&'pl [(&'pl str, LSPAny<'pl>)]),
    Array(&'pl [usize]),
    String(&'pl str),
    Integer(&'pl i64),
    UInteger(&'pl u64),
    // Decimal is interpreted as a str but parsable as a float,
    // to avoid Eq issues
    Decimal(&'pl str),
    Boolean(bool),
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Params<'pl> {
    Named(LSPObject<'pl>),
    Positional(LSPArray<'pl>),
}

struct ContentBuilder<'pl> {
    header: Option<Header<'pl>>,
    body: Option<Body<'pl>>,
}
impl<'pl> ContentBuilder<'pl> {
    pub fn new() -> Self {
        Self {
            header: None,
            body: None,
        }
    }
    pub fn add_header(mut self, content_length: u16, content_type: Option<&'pl str>) -> Self {
        self.header = Some(Header {
            content_length,
            content_type,
        });
        self
    }
    pub fn build(self) -> Content<'pl> {
        Content {
            header: self
                .header
                .expect("Called build on a builder without a header"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content<'pl> {
    pub header: Header<'pl>,
}

impl<'pl> LspParser<'pl> {
    fn new(payload: &'pl [u8]) -> LspParser<'pl> {
        let str_payload = &std::str::from_utf8(payload)
            .expect("According to spec LSP should be always utf-8 encoded.");
        LspParser {
            payload: str_payload,
            start_pointer: 0,
            end_pointer: 0,
        }
    }
    fn parse(&mut self) -> Result<Content> {
        let mut content = ContentBuilder::new();
        content = self.parse_header(content)?;
        Ok(content.build())
    }
    fn parse_header(&mut self, content: ContentBuilder<'pl>) -> Result<ContentBuilder<'pl>> {
        let mut content_length = 0;
        let mut content_type = None;

        while !self.payload[self.end_pointer..].starts_with(CRLF) {
            self.end_pointer = self.start_pointer
                + self.payload[self.start_pointer..]
                    .find(':')
                    .ok_or(Error::ParsingError(
                        "Couldn't find `:` between name and value in header of the payload."
                            .to_string(),
                    ))?;
            let name = &self.payload[self.start_pointer..self.end_pointer];
            self.end_pointer += 1;
            self.start_pointer = self.end_pointer;

            self.end_pointer = self.start_pointer
                + self.payload[self.start_pointer..]
                    .find(CRLF)
                    .ok_or(Error::ParsingError(
                        "Couldn't find `\r\n` delimiter after a header section of the payload."
                            .to_string(),
                    ))?;

            let value = &self.payload[self.start_pointer..self.end_pointer];
            self.start_pointer = self.end_pointer;

            match name {
                "Content-Length" => {
                    content_length = value.parse::<u16>().map_err(|e| {
                        Error::ParsingError(format!(
                            "Failed parsing the content-length value: `{value}` as a u16: {e}"
                        ))
                    })?
                }
                "Content-Type" => content_type = Some(value),
                _ => Err(Error::ParsingError(format!("Unknown header type: {name}")))?,
            };
            self.start_pointer += CRLF_BYTE_LEN;
            self.end_pointer += CRLF_BYTE_LEN;
        }
        if content_length == 0 {
            return Err(Error::ParsingError(
                "Content-length must be specified and higher than zero.".to_string(),
            ));
        };

        Ok(content.add_header(content_length, content_type))
    }
}

enum HeaderType {
    ContentLength,
    ContentType,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn create_test_bytes(text: &str) -> Vec<u8> {
        text.chars()
            .flat_map(|c| match c {
                '\r' => vec![b'\\', b'r'],
                '\n' => vec![b'\\', b'n'],
                _ => vec![c as u8],
            })
            .collect()
    }
    #[test]
    fn parse_buffer_header() {
        let bytes =
            create_test_bytes("Content-Length:40\r\nContent-Type:something\r\n\r\nDontparse\n");
        let mut parser = LspParser::new(&bytes);
        let content = parser.parse().unwrap();
        assert_eq!(content.header.content_type.unwrap(), "something");
        assert_eq!(content.header.content_length, 40);
    }

    #[test]
    fn parse_buffer_header_length_only() {
        let bytes = create_test_bytes("Content-Length:40\r\n\r\nDontparse\n");
        let mut parser = LspParser::new(&bytes);
        let content = parser.parse().unwrap();
        assert!(content.header.content_type.is_none());
        assert_eq!(content.header.content_length, 40);
    }

    #[test]
    fn parse_buffer_header_invalid_no_content_length() {
        let bytes = create_test_bytes("Content-Type:something\r\n\r\nDontparse\n");
        let mut parser = LspParser::new(&bytes);
        let content = parser.parse();
        assert!(content.is_err());
    }
}
