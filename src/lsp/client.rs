use std::path::PathBuf;

use super::data::{initialize_params, Body};
use crate::Result;
use tokio::io::AsyncReadExt;
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::mpsc;
pub struct LSPClient {
    sender: mpsc::Sender<Body>,
    reader: tokio::io::BufReader<tokio::io::Stdin>,
    server_stdin: ChildStdin,
    server_stdout: ChildStdout,
}

impl LSPClient {
    pub async fn new(
        channel: mpsc::Sender<Body>,
        ext: FileType,
        file_path: &PathBuf,
    ) -> Result<Self> {
        let (server_stdin, server_stdout) = Self::start_lsp_server(ext, file_path).await?;

        let stdin = tokio::io::stdin();
        let reader = tokio::io::BufReader::new(stdin);
        let client = Self {
            reader,
            sender: channel,
            server_stdin,
            server_stdout,
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
            self.sender.send(content.body).await.unwrap();
        }
    }

    async fn start_lsp_server(
        ext: FileType,
        file_path: &PathBuf,
    ) -> Result<(ChildStdin, ChildStdout)> {
        match ext {
            FileType::Rust => start_rust_lsp(file_path).await,
            _ => start_rust_lsp(file_path).await,
        }
    }
}

pub enum FileType {
    Rust,
    Python,
    Unknown,
}

impl From<&str> for FileType {
    fn from(value: &str) -> Self {
        match value {
            "rs" => FileType::Rust,
            "py" => FileType::Python,
            _ => FileType::Unknown,
        }
    }
}

async fn start_rust_lsp(file_path: &PathBuf) -> Result<(ChildStdin, ChildStdout)> {
    let mut ra = tokio::process::Command::new("rust-analyzer")
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .arg(file_path)
        .spawn()
        .expect("Failed starting Rust Analyzer");

    let writer = ra.stdin.take().expect("Failed to get stdin");
    let reader = ra.stdout.take().expect("Failed to get stdout");
    Ok((writer, reader))
}
