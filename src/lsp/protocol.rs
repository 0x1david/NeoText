use crate::{Error, Result};

struct LspParser<'pl> {
    payload: &'pl str,
    start_pointer: usize,
    end_pointer: usize,
}

struct Header<'pl> {
    content_length: u16,
    content_type: Option<&'pl str>,
}

struct ContentBuilder<'pl> {
    header: Option<Header<'pl>>,
}
impl<'pl> ContentBuilder<'pl> {
    pub fn new() -> Self {
        Self { header: None }
    }
    pub fn add_header(mut self, content_length: u16, content_type: Option<&'pl str>) -> Self {
        self.header = Some(Header {
            content_length,
            content_type,
        });
        return self;
    }
    pub fn build(self) -> Content<'pl> {
        Content {
            header: self
                .header
                .expect("Called build on a builder without a header"),
        }
    }
}

struct Content<'pl> {
    header: Header<'pl>,
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
    fn parse(&self) -> Content {
        let content = ContentBuilder::new();
        let content = self.parse_header(content);
    }
    fn parse_header(&mut self, content: ContentBuilder<'pl>) -> Result<ContentBuilder<'pl>> {
        let mut content_length = 0;
        let mut content_type = None;

        while !self.payload[self.end_pointer + 1..].starts_with("\r\n") {
            self.end_pointer = self.payload.find(':').ok_or(Error::ParsingError(
                "Couldn't find `:` between name and value in header of the payload.".to_string(),
            ))?;
            let name = &self.payload[self.start_pointer..self.end_pointer];
            self.start_pointer = self.end_pointer;

            self.end_pointer = self.payload.find("\r\n").ok_or(Error::ParsingError(
                "Couldn't find `\r\n` delimiter after a header section of the payload.".to_string(),
            ))?;
            let value = &self.payload[self.start_pointer..self.end_pointer];
            self.start_pointer = self.end_pointer;

            match name {
                "content-length" => {
                    content_length = value.parse::<u16>().map_err(|e| {
                        Error::ParsingError(format!(
                            "Failed parsing the content-length value as a u16: {e}"
                        ))
                    })?
                }
                "content-type" => content_type = Some(value),
                _ => Err(Error::ParsingError("Unknown header type".to_string()))?,
            };
        }
        if content_length == 0 {
            return Err(Error::ParsingError(
                "Content-length specified as zero isn't allowed".to_string(),
            ));
        };

        Ok(content.add_header(content_length, content_type))
    }
}

enum HeaderType {
    ContentLength,
    ContentType,
}
