use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const LOCALE: &str = "eng_todo";
const CLIENT_NAME: &str = "client";
const CLIENT_VERSION: &str = "v1.0.0";

/// Inserts a key-value pair into a HashMap, converting the key to a String and the value to LSPAny.
///
/// # Arguments
///
/// * `$hm` - The HashMap to insert into
/// * `$key` - The key to insert, will be converted to a String
/// * `$value` - The value to insert, will be converted to LSPAny
macro_rules! insert {
    ($hm:ident,$key:expr,$value: expr) => {
        $hm.insert($key.to_string(), $value.into())
    };
}

/// Implements From<T> for LSPAny for various types.
///
/// # Arguments
///
/// * `$ident` - The LSPAny variant to use
/// * `$ty` - The type to convert from
///
/// # Example
///
/// ```
/// from_any!(String, String);
/// from_any!(Integer, i32);
/// ```
///
/// Use `#` before the type to serialize it to a JSON string:
///
/// ```
/// from_any!(String, #CustomType);
/// ```
macro_rules! from_any {
    ($ident:ident, $ty:ty) => {
        impl From<$ty> for LSPAny {
            fn from(value: $ty) -> Self {
                LSPAny::$ident(value.into())
            }
        }
    };

    ($ident:ident, #$ty:ty) => {
        impl From<$ty> for LSPAny {
            fn from(value: $ty) -> Self {
                LSPAny::$ident(serde_json::to_string(&value).unwrap().into())
            }
        }
    };
}

from_any!(String, String);
from_any!(String, &str);
from_any!(Integer, i32);
from_any!(Integer, i16);
from_any!(Integer, i8);
from_any!(UInteger, u32);
from_any!(UInteger, u16);
from_any!(UInteger, u8);
from_any!(Decimal, f32);
from_any!(Boolean, bool);
from_any!(Object, LSPObject);
from_any!(Array, LSPArray);
from_any!(String, #ClientCapabilities);

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
    Notification(Notification),
}

impl Default for Body {
    fn default() -> Self {
        Self::Request(Request::default())
    }
}

impl Body {
    fn is_response(&self) -> bool {
        matches!(self, Body::Response(_))
    }
    pub fn is_request(&self) -> bool {
        matches!(self, Body::Request(_))
    }
    fn is_notification(&self) -> bool {
        matches!(self, Body::Request(_))
    }
    pub fn get_response(self) -> Result<Response> {
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
pub struct Response {
    jsonrpc: String,
    id: Option<usize>,
    result: String,
    error: Option<String>,
}

type LSPObject = HashMap<String, LSPAny>;
type LSPArray = Vec<usize>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LSPAny {
    Object(LSPObject),
    Array(LSPArray),
    String(String),
    Integer(i32),
    UInteger(u32),
    // Decimal is interpreted as a str but parsable as a float,
    // to avoid Eq issues
    Decimal(f32),
    Boolean(bool),
    None,
}
impl PartialEq for LSPAny {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LSPAny::Object(a), LSPAny::Object(b)) => a == b,
            (LSPAny::Array(a), LSPAny::Array(b)) => a == b,
            (LSPAny::String(a), LSPAny::String(b)) => a == b,
            (LSPAny::Integer(a), LSPAny::Integer(b)) => a == b,
            (LSPAny::UInteger(a), LSPAny::UInteger(b)) => a == b,
            (LSPAny::Decimal(a), LSPAny::Decimal(b)) => {
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            }
            (LSPAny::Boolean(a), LSPAny::Boolean(b)) => a == b,
            (LSPAny::None, LSPAny::None) => true,
            _ => false,
        }
    }
}

impl Eq for LSPAny {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Params {
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
        insert!(text_document, "uri", "file:///path/to/file.rs");

        //     "position": {
        //         "line": 10,
        //         "character": 15
        //     }
        let mut position: LSPObject = HashMap::new();
        insert!(position, "line", 10);
        insert!(position, "character", 15);

        insert!(params, "textDocument", text_document);
        insert!(params, "position", position);

        Params::Named(params)
    }
}

enum HeaderType {
    ContentLength,
    ContentType,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    method: String,
    // Only Object or Array Param is allowed
    params: Params,
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

impl Request {
    pub fn initialization_req(initializer_params: Params) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(1),
            method: "initialize".to_string(),
            params: initializer_params,
        }
    }
}

pub fn initialize_params(process_id: u32, capabilities: ClientCapabilities) -> Params {
    let mut params = HashMap::new();
    let mut client_info: HashMap<String, LSPAny> = HashMap::new();
    // The name of the client as defined by the client
    insert!(client_info, "name", CLIENT_NAME);
    // The version of the client as defined by the client
    insert!(client_info, "version", CLIENT_VERSION);

    // The process Id of the parent process that started the server. Is null if
    // the process has not been started by another process. If the parent
    // process is not alive then the server should exit (see exit notification)
    // its process.
    insert!(params, "processId", process_id);
    insert!(params, "clientInfo", client_info);
    insert!(params, "locale", LOCALE);
    // The rootPath of the workspace. Is null if no folder is open
    insert!(params, "rootPath", "TODO");
    // The rootUri of the workspace. Is null if no folder is open. If both rootUri and rootPath are
    // set rootUrl has priority.
    insert!(params, "rootUri", "TODO");
    insert!(params, "initializationOptions", capabilities);
    insert!(params, "capabilities", "TODO");

    Params::Named(params)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Workspace specific client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceClientCapabilities>,

    /// Text document specific client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document: Option<TextDocumentClientCapabilities>,

    /// Capabilities specific to the notebook document support.
    ///
    /// @since 3.17.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notebook_document: Option<NotebookDocumentClientCapabilities>,

    /// Window specific client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<WindowClientCapabilities>,

    /// General client capabilities.
    ///
    /// @since 3.16.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub general: Option<GeneralClientCapabilities>,

    /// Experimental client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceClientCapabilities {
    /// The client supports applying batch edits to the workspace by supporting the request
    /// 'workspace/applyEdit'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_edit: Option<bool>,

    /// Capabilities specific to `WorkspaceEdit`s
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_edit: Option<WorkspaceEditClientCapabilities>,

    /// Capabilities specific to the `workspace/didChangeConfiguration` notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_change_configuration: Option<DidChangeConfigurationClientCapabilities>,

    /// Capabilities specific to the `workspace/didChangeWatchedFiles` notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_change_watched_files: Option<DidChangeWatchedFilesClientCapabilities>,

    /// Capabilities specific to the `workspace/symbol` request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<WorkspaceSymbolClientCapabilities>,

    /// Capabilities specific to the `workspace/executeCommand` request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execute_command: Option<ExecuteCommandClientCapabilities>,

    /// The client has support for workspace folders.
    ///
    /// @since 3.6.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<bool>,

    /// The client supports `workspace/configuration` requests.
    ///
    /// @since 3.6.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<bool>,

    /// Capabilities specific to the semantic token requests scoped to the workspace.
    ///
    /// @since 3.16.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_tokens: Option<SemanticTokensWorkspaceClientCapabilities>,

    /// Capabilities specific to the code lens requests scoped to the workspace.
    ///
    /// @since 3.16.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_lens: Option<CodeLensWorkspaceClientCapabilities>,

    /// The client has support for file requests/notifications.
    ///
    /// @since 3.16.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_operations: Option<FileOperationsClientCapabilities>,

    /// Client workspace capabilities specific to inline values.
    ///
    /// @since 3.17.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_value: Option<InlineValueWorkspaceClientCapabilities>,

    /// Client workspace capabilities specific to inlay hints.
    ///
    /// @since 3.17.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inlay_hint: Option<InlayHintWorkspaceClientCapabilities>,

    /// Client workspace capabilities specific to diagnostics.
    ///
    /// @since 3.17.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<DiagnosticWorkspaceClientCapabilities>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowClientCapabilities {
    /// It indicates whether the client supports server initiated progress using the
    /// `window/workDoneProgress/create` request.
    ///
    /// The capability also controls Whether client supports handling of progress notifications.
    /// If set servers are allowed to report a `workDoneProgress` property in the request specific
    /// server capabilities.
    ///
    /// @since 3.15.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_done_progress: Option<bool>,

    /// Capabilities specific to the showMessage request
    ///
    /// @since 3.16.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_message: Option<ShowMessageRequestClientCapabilities>,

    /// Client capabilities for the show document request.
    ///
    /// @since 3.16.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_document: Option<ShowDocumentClientCapabilities>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralClientCapabilities {
    /// Client capability that signals how the client handles stale requests (e.g. a request
    /// for which the client will not process the response anymore since the information is outdated).
    ///
    /// @since 3.17.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_request_support: Option<StaleRequestSupportCapability>,

    /// Client capabilities specific to regular expressions.
    ///
    /// @since 3.16.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_expressions: Option<RegularExpressionsClientCapabilities>,

    /// Client capabilities specific to the client's markdown parser.
    ///
    /// @since 3.16.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<MarkdownClientCapabilities>,

    /// The position encodings supported by the client. Client and server have to agree on the same
    /// position encoding to ensure that offsets (e.g. character position in a line) are interpreted
    /// the same on both side.
    ///
    /// To keep the protocol backwards compatible the following applies: if the value 'utf-16' is
    /// missing from the array of position encodings servers can assume that the client supports
    /// UTF-16. UTF-16 is therefore a mandatory encoding.
    ///
    /// If omitted it defaults to ['utf-16'].
    ///
    /// Implementation considerations: since the conversion from one encoding into another requires
    /// the content of the file / line the conversion is best done where the file is read which is
    /// usually on the server side.
    ///
    /// @since 3.17.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_encodings: Option<Vec<PositionEncodingKind>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StaleRequestSupportCapability {
    /// The client will actively cancel the request.
    pub cancel: bool,

    /// The list of requests for which the client will retry the request if it receives a
    /// response with error code `ContentModified`
    pub retry_on_content_modified: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileOperationsClientCapabilities {
    /// Whether the client supports dynamic registration for file requests/notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// The client has support for sending didCreateFiles notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_create: Option<bool>,

    /// The client has support for sending willCreateFiles requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_create: Option<bool>,

    /// The client has support for sending didRenameFiles notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_rename: Option<bool>,

    /// The client has support for sending willRenameFiles requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_rename: Option<bool>,

    /// The client has support for sending didDeleteFiles notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_delete: Option<bool>,

    /// The client has support for sending willDeleteFiles requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_delete: Option<bool>,
}

// Note: The following types are not fully defined in the provided interface.
// You'll need to implement them based on their actual definitions.
type WorkspaceEditClientCapabilities = serde_json::Value;
type DidChangeConfigurationClientCapabilities = serde_json::Value;
type DidChangeWatchedFilesClientCapabilities = serde_json::Value;
type WorkspaceSymbolClientCapabilities = serde_json::Value;
type ExecuteCommandClientCapabilities = serde_json::Value;
type SemanticTokensWorkspaceClientCapabilities = serde_json::Value;
type CodeLensWorkspaceClientCapabilities = serde_json::Value;
type InlineValueWorkspaceClientCapabilities = serde_json::Value;
type InlayHintWorkspaceClientCapabilities = serde_json::Value;
type DiagnosticWorkspaceClientCapabilities = serde_json::Value;
type TextDocumentClientCapabilities = serde_json::Value;
type NotebookDocumentClientCapabilities = serde_json::Value;
type ShowMessageRequestClientCapabilities = serde_json::Value;
type ShowDocumentClientCapabilities = serde_json::Value;
type RegularExpressionsClientCapabilities = serde_json::Value;
type MarkdownClientCapabilities = serde_json::Value;
type PositionEncodingKind = String;
