use super::data::{initialize_params, Body};
use crate::Result;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
pub struct LSPClient {
    sender: mpsc::Sender<Body>,
    reader: tokio::io::BufReader<tokio::io::Stdin>,
}

impl LSPClient {
    pub async fn new(channel: mpsc::Sender<Body>) -> Result<Self> {
        let stdin = tokio::io::stdin();
        let reader = tokio::io::BufReader::new(stdin);
        let client = Self {
            reader,
            sender: channel,
        };

        client.initialize().await?;
        client.announce_capabilities().await?;

        Ok(client)
    }
    // Not every language server can support all features defined by the protocol.
    // LSP therefore provides ‘capabilities’. A capability groups a set of language features.
    // A development tool and the language server announce their supported features using capabilities.
    // As an example, a server announces that it can handle the textDocument/hover request,
    // but it might not handle the workspace/symbol request.
    // Similarly, a development tool announces its ability to provide about to save notifications before a document is saved,
    // so that a server can compute textual edits to format the edited document before it is saved.
    async fn announce_capabilities(&self) -> Result<()> {
        todo!()
    }
    // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#initialize
    async fn initialize(&self) -> Result<()> {
        todo!()
    }
    async fn send_request(&self) -> Result<()> {
        todo!()
    }
    async fn send_notification(&self) -> Result<()> {
        todo!()
    }
    pub async fn listen_and_serve(&mut self) -> ! {
        let mut msg = vec![];

        loop {
            let _bytes_read = self.reader.read_to_end(&mut msg).await.unwrap();
            let mut parser = super::parser::LspParser::new(&msg);
            let content = parser.parse();
            let content = content.unwrap();
            self.sender.send(content.body);
        }
    }
}
