use std::collections::HashMap;

use crate::{Error, Result};
use serde::{Deserialize, Serialize};

const CRLF: &str = r"\r\n";
const CRLF_BYTE_LEN: usize = CRLF.len();

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Body {
    Request(Request),
    Response(Response),
}

impl Default for Body {
    fn default() -> Self {
        Self::Request(Request::default())
    }
}

// {
// "jsonrpc": "2.0",
// "id": 1,
// "method": "textDocument/completion",
// "params": {
//     "textDocument": {
//         "uri": "file:///path/to/file.rs"
//     },
//     "position": {
//         "line": 10,
//         "character": 15
//     }
// }
// }"#;
impl Body {
    fn is_response(&self) -> bool {
        matches!(self, Body::Response(_))
    }
    fn is_request(&self) -> bool {
        matches!(self, Body::Request(_))
    }
    fn get_response(self) -> Result<Response> {
        match self {
            Self::Response(r) => Ok(r),
            _ => Err(Error::ParsingError(
                "Tried getting response from body that is not a response body.".to_string(),
            )),
        }
    }
    fn get_request(self) -> Result<Request> {
        match self {
            Self::Request(r) => Ok(r),
            _ => Err(Error::ParsingError(
                "Tried getting request from body that is not a request body.".to_string(),
            )),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    jsonrpc: String,
    id: Option<usize>,
    method: String,
    // Only Object or Array Param is allowed
    params: Params,
}
impl Default for Request {
    fn default() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(1),
            method: "textDocument/completion".to_string(),
            params: Params::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    jsonrpc: String,
    id: Option<usize>,
    result: String,
    error: Option<String>,
}

type LSPObject = HashMap<String, LSPAny>;

type LSPArray = Vec<usize>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LSPAny {
    Object(LSPObject),
    Array(LSPArray),
    String(String),
    Integer(i32),
    UInteger(u32),
    // Decimal is interpreted as a str but parsable as a float,
    // to avoid Eq issues
    Decimal(String),
    Boolean(bool),
    None,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum Params {
    Named(LSPObject),
    Positional(LSPArray),
}
impl Default for Params {
    fn default() -> Self {
        let mut params: LSPObject = HashMap::new();

        //     "textDocument": {
        //         "uri": "file:///path/to/file.rs"
        //     },
        let mut text_document: LSPObject = HashMap::new();
        text_document.insert(
            "uri".to_string(),
            LSPAny::String("file:///path/to/file.rs".to_string()),
        );

        //     "position": {
        //         "line": 10,
        //         "character": 15
        //     }
        let mut position: LSPObject = HashMap::new();
        position.insert("line".to_string(), LSPAny::Integer(10));
        position.insert("character".to_string(), LSPAny::Integer(15));

        params.insert("textDocument".to_string(), LSPAny::Object(text_document));
        params.insert("position".to_string(), LSPAny::Object(position));

        Params::Named(params)
    }
}

struct ContentBuilder<'pl> {
    header: Option<Header<'pl>>,
    body: Option<Body>,
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
    pub fn add_body(mut self, body: Body) -> Self {
        self.body = Some(body);
        self
    }
    pub fn build(self) -> Content<'pl> {
        Content {
            header: self
                .header
                .expect("Called build on a builder without a header"),
            body: self.body.expect("Called build on  abuilder without a body"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content<'pl> {
    pub header: Header<'pl>,
    pub body: Body,
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
        content = self.parse_body(content)?;
        Ok(content.build())
    }
    fn parse_body(&mut self, content: ContentBuilder<'pl>) -> Result<ContentBuilder<'pl>> {
        let length = content.header.clone().unwrap().content_length;

        let body_str = &self.payload[self.start_pointer..self.start_pointer + length as usize];
        let body: Body = serde_json::from_str(body_str).map_err(|e| {
            Error::ParsingError(format!("Deserializing body with serde failed: {e}"))
        })?;
        Ok(content.add_body(body))
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

        self.start_pointer += CRLF_BYTE_LEN;
        self.end_pointer += CRLF_BYTE_LEN;
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
        let mut content_builder = ContentBuilder::new();
        let mut parser = LspParser::new(&bytes);
        content_builder = parser.parse_header(content_builder).unwrap();
        let header = content_builder.header.unwrap();
        assert_eq!(header.content_type.unwrap(), "something");
        assert_eq!(header.content_length, 40);
    }

    #[test]
    fn parse_buffer_header_length_only() {
        let bytes = create_test_bytes("Content-Length:40\r\n\r\nDontparse\n");
        let mut content_builder = ContentBuilder::new();
        let mut parser = LspParser::new(&bytes);
        content_builder = parser.parse_header(content_builder).unwrap();
        let header = content_builder.header.unwrap();
        assert!(header.content_type.is_none());
        assert_eq!(header.content_length, 40);
    }

    #[test]
    fn parse_buffer_header_invalid_no_content_length() {
        let bytes = create_test_bytes("Content-Type:something\r\n\r\nDontparse\n");
        let content_builder = ContentBuilder::new();
        let mut parser = LspParser::new(&bytes);
        let result = parser.parse_header(content_builder);
        assert!(result.is_err());
    }

    #[test]
    fn parse_buffer_body() {
        let header = "Content-Length:157\r\nContent-Type:something\r\n\r\n";
        let body = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"textDocument/completion\",\"params\":{\"textDocument\":{\"uri\":\"file:///path/to/file.rs\"},\"position\":{\"line\":10,\"character\":15}}}".trim();

        let payload = format!("{}{}", header, body);
        let bytes = create_test_bytes(&payload);
        let mut content_builder = ContentBuilder::new();
        let mut parser = LspParser::new(&bytes);
        content_builder = parser.parse_header(content_builder).unwrap();
        content_builder = parser.parse_body(content_builder).unwrap();
        let body = content_builder.body.unwrap();
        assert!(body.is_request());
        assert_eq!(Body::default(), body)
    }
}
