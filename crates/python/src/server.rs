//! The language server: `jedi-language-server`, Jedi behind the Language Server
//! Protocol — completion, hover, go to definition, references, rename and
//! symbols, and the syntax errors Jedi's parser finds.
//!
//! Installed by the user (`pip install jedi-language-server`), not bundled: it
//! is a Python package, and the project's own environment is where the user
//! keeps their tools. It is found in the project's virtual environment first,
//! then on `PATH`, and a path configured in Settings wins over both.

use std::path::Path;

use fg_extension::{LanguageServerContribution, ResolvedServerStart, ServerStartContext};

use crate::PYTHON;
use crate::run::{environment_program, environment_python};

pub const JEDI: &str = "jedi";
pub const JEDI_BINARY: &str = "jedi-language-server";

pub(crate) fn contribution() -> LanguageServerContribution {
    LanguageServerContribution {
        id: JEDI.to_string(),
        display_name: "Jedi Language Server".to_string(),
        language_ids: vec![PYTHON.to_string()],
        binary_name: JEDI_BINARY.to_string(),
        args: Vec::new(),
        initialization_options: None,
        install: None,
        virtual_documents: None,
        host_runtime: None,
    }
}

/// How to start the server for the project in `context`. Jedi resolves imports
/// against the environment it runs in unless told otherwise, so a project that
/// has a virtual environment hands Jedi its interpreter: completions then see
/// the packages the project installed, not just the server's own.
pub(crate) fn resolve(context: &ServerStartContext) -> ResolvedServerStart {
    let root = context.project_root.as_deref();
    let configured = context.configured_binary.trim();
    let binary = if !configured.is_empty() {
        configured.into()
    } else {
        root.and_then(|root| environment_program(root, JEDI_BINARY)).unwrap_or_else(|| JEDI_BINARY.into())
    };
    let environment = root.and_then(environment_python);
    ResolvedServerStart {
        binary,
        args: Vec::new(),
        initialization_options: environment.as_deref().map(initialization_options),
        // A session started before the project grew an environment is replaced
        // by one that knows about it.
        restart_key: environment.map(|python| python.display().to_string()).unwrap_or_default(),
    }
}

fn initialization_options(python: &Path) -> String {
    serde_json::json!({ "workspace": { "environmentPath": python.display().to_string() } }).to_string()
}

#[cfg(test)]
#[path = "server_test.rs"]
mod server_test;
