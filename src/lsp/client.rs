use super::data::initialize_params;
use crate::Result;
pub struct LSPClient {}

impl LSPClient {
    // Not every language server can support all features defined by the protocol.
    // LSP therefore provides ‘capabilities’. A capability groups a set of language features.
    // A development tool and the language server announce their supported features using capabilities.
    // As an example, a server announces that it can handle the textDocument/hover request,
    // but it might not handle the workspace/symbol request.
    // Similarly, a development tool announces its ability to provide about to save notifications before a document is saved,
    // so that a server can compute textual edits to format the edited document before it is saved.
    fn announce_capabilities() -> Result<()> {
        todo!()
    }
    // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#initialize
    fn initialize() {
        todo!()
    }
    fn send_request() -> Result<()> {
        todo!()
    }
    fn send_notification() -> Result<()> {
        todo!()
    }
}
enum LSPAction {}
