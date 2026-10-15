//! NextUs Language Server
//! 
//! Provides IDE features for .nxt files via the Language Server Protocol.
//! 
//! Run with: nextus-lsp
//! 
//! The server communicates over stdin/stdout using JSON-RPC.

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use std::collections::HashMap;
use std::sync::RwLock;

mod capabilities;
mod diagnostics;
mod completion;
mod hover;

/// Document state stored by the server
struct Document {
    content: String,
    #[allow(dead_code)]
    version: i32,
}

/// The NextUs Language Server
struct NextUsLanguageServer {
    /// Client handle for sending notifications (like diagnostics)
    client: Client,
    /// Open documents (URI -> content)
    documents: RwLock<HashMap<String, Document>>,
}

impl NextUsLanguageServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(HashMap::new()),
        }
    }

    /// Validate a document and publish diagnostics
    async fn validate_document(&self, uri: &Url, text: &str) {
        let diagnostics = diagnostics::get_diagnostics(text);
        
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for NextUsLanguageServer {
    /// Called when client connects - negotiate capabilities
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // Sync full document on change
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                // We provide completions
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        "{".to_string(),
                        " ".to_string(),
                        ":".to_string(),
                    ]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                // We provide hover info
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "NextUs Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    /// Called after initialize completes
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "NextUs LSP initialized!")
            .await;
    }

    /// Called when client disconnects
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    /// Document opened - store it and validate
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        // Store document
        {
            let mut docs = self.documents.write().unwrap();
            docs.insert(uri.to_string(), Document {
                content: text.clone(),
                version,
            });
        }

        // Validate and publish diagnostics
        self.validate_document(&uri, &text).await;
    }

    /// Document changed - update and revalidate
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        
        // Get the full new content (we use FULL sync)
        if let Some(change) = params.content_changes.into_iter().next() {
            let text = change.text;

            // Update stored document
            {
                let mut docs = self.documents.write().unwrap();
                docs.insert(uri.to_string(), Document {
                    content: text.clone(),
                    version,
                });
            }

            // Revalidate
            self.validate_document(&uri, &text).await;
        }
    }

    /// Document closed - remove from storage
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        
        // Clear diagnostics
        self.client
            .publish_diagnostics(uri.clone(), vec![], None)
            .await;

        // Remove from storage
        let mut docs = self.documents.write().unwrap();
        docs.remove(&uri.to_string());
    }

    /// Provide completions at cursor position
    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let docs = self.documents.read().unwrap();
        let completions = if let Some(doc) = docs.get(&uri.to_string()) {
            completion::get_completions(&doc.content, position)
        } else {
            vec![]
        };

        Ok(Some(CompletionResponse::Array(completions)))
    }

    /// Provide hover information
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.read().unwrap();
        if let Some(doc) = docs.get(&uri.to_string()) {
            Ok(hover::get_hover(&doc.content, position))
        } else {
            Ok(None)
        }
    }
}

#[tokio::main]
async fn main() {
    // Create the LSP service
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| NextUsLanguageServer::new(client));

    // Run the server (communicates via stdin/stdout)
    Server::new(stdin, stdout, socket).serve(service).await;
}
