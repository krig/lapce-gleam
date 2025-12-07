// Deny usage of print and eprint as it won't have same result
// in WASI as if doing in standard program, you must really know
// what you are doing to disable that lint (and you don't know)
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]

use anyhow::{bail, Result};
use lapce_plugin::{
    psp_types::{
        lsp_types::{
            request::Initialize, DocumentFilter, DocumentSelector, InitializeParams, MessageType,
            Url,
        },
        Request,
    },
    register_plugin, LapcePlugin, VoltEnvironment, PLUGIN_RPC,
};
use serde_json::Value;

#[derive(Default)]
struct State {}

register_plugin!(State);

fn initialize(params: InitializeParams) -> Result<()> {
    PLUGIN_RPC.stderr("lapce-gleam");
    let document_selector: DocumentSelector = vec![DocumentFilter {
        language: Some(string!("gleam")),
        pattern:  Some(string!("**/*.gleam")),
        scheme:   None,
    }];
    let mut server_args = vec![string!("lsp")];
    let mut initialization_options = None;

    if let Some(options) = params.initialization_options.as_ref() {
        if let Some(gleam) = options.get("gleam") {
            initialization_options = Some(gleam.to_owned());
        }

        if let Some(lsp) = options.get("lsp") {
            if let Some(args) = volt.get("serverArgs") {
                if let Some(args) = args.as_array() {
                    if !args.is_empty() {
                        server_args = vec![];
                    }
                    for arg in args {
                        if let Some(arg) = arg.as_str() {
                            server_args.push(arg.to_string());
                        }
                    }
                }
            }

            if let Some(server_path) = lsp.get("serverPath") {
                if let Some(server_path) = server_path.as_str() {
                    if !server_path.is_empty() {
                        let server_uri = Url::parse(&format!("urn:{}", server_path))?;
                        PLUGIN_RPC.start_lsp(
                            server_uri,
                            server_args,
                            document_selector,
                            initialization_options,
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}

impl LapcePlugin for State {
    fn handle_request(&mut self, _id: u64, method: String, params: Value) {
        #[allow(clippy::single_match)]
        match method.as_str() {
            Initialize::METHOD => {
                let params: InitializeParams = serde_json::from_value(params).unwrap();
                if let Err(e) = initialize(params) {
                    let _ = PLUGIN_RPC.window_log_message(MessageType::ERROR, e.to_string());
                    let _ = PLUGIN_RPC.window_show_message(MessageType::ERROR, e.to_string());
                }
            }
            _ => {}
        }
    }
}
