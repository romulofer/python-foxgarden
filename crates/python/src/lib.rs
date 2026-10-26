//! Python support for FoxGarden, as an extension.
//!
//! Contributes the language (`.py`, `.pyi`, `.pyw`) and its tree-sitter grammar
//! with a highlight query, a build tool that runs a file with the project's
//! interpreter, a run marker on a file's main guard, and Jedi as its language
//! server.

use std::path::Path;

use fg_extension::{
    BuildToolContribution, ResolvedServerStart, ServerStartContext, CURRENT_SCHEMA_VERSION, Capability, CommandSpec, Contributions, Extension, ExtensionManifest, FileIcon,
    GrammarContribution, GrammarSource, LanguageContribution, LanguageId, NodeKinds, RunSpec, RunTarget,
};

mod main_guard;
mod run;
mod server;

pub const PYTHON: &str = "python";
/// The build tool that runs a Python project: there is nothing to build, so it
/// only launches files.
pub const PYTHON_TOOL: &str = "python";

pub struct PythonExtension;

impl Extension for PythonExtension {
    fn manifest(&self) -> ExtensionManifest {
        ExtensionManifest {
            id: "python".to_string(),
            name: "Python".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            schema_version: CURRENT_SCHEMA_VERSION,
        }
    }

    /// The interpreter is a program this extension starts.
    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability::ProcessExec]
    }

    fn contributions(&self) -> Contributions {
        Contributions {
            build_tools: vec![BuildToolContribution {
                id: PYTHON_TOOL.to_string(),
                display_name: "Python".to_string(),
                marker_files: ["pyproject.toml", "requirements.txt", "setup.py", "setup.cfg", "Pipfile", "main.py"]
                    .map(str::to_string)
                    .to_vec(),
            }],
            languages: vec![LanguageContribution {
                id: LanguageId::from(PYTHON),
                display_name: "Python".to_string(),
                file_extensions: vec!["py".to_string(), "pyi".to_string(), "pyw".to_string()],
                filename_patterns: Vec::new(),
            }],
            grammars: vec![GrammarContribution {
                language_id: PYTHON.to_string(),
                source: GrammarSource::Builtin(tree_sitter_python::LANGUAGE),
                highlight_query: Some(include_str!("../queries/highlights_python.scm").to_string()),
                node_kinds: node_kinds(),
            }],
            language_servers: vec![server::contribution()],
            // The Nerd Fonts devicons Python glyph.
            file_icons: vec![FileIcon { language_id: PYTHON.to_string(), glyph: '\u{e73c}' }],
            ..Contributions::default()
        }
    }

    fn run_targets(&self, language_id: &str, tree: &tree_sitter::Tree, source: &str, file: &Path) -> Vec<RunTarget> {
        if language_id == PYTHON { main_guard::main_guard(tree, source, file) } else { Vec::new() }
    }

    fn resolve_server_start(&self, server_id: &str, context: &ServerStartContext) -> Option<Result<ResolvedServerStart, String>> {
        (server_id == server::JEDI).then(|| Ok(server::resolve(context)))
    }

    fn run_command(&self, tool_id: &str, project_root: &Path, run: &RunSpec) -> Option<Result<CommandSpec, String>> {
        (tool_id == PYTHON_TOOL).then(|| Ok(run::command(project_root, run)))
    }
}

/// Checked against `tree-sitter-python` 0.25.0's real parse output: a `def` or
/// `class` body is a `block`, and a decorated one is wrapped in a
/// `decorated_definition` whose header line is the decorator, so only the
/// definitions themselves are pinned.
fn node_kinds() -> NodeKinds {
    NodeKinds {
        scopes: vec!["class_definition".to_string(), "function_definition".to_string()],
        foldable: vec!["block".to_string()],
        import: Some("import_statement".to_string()),
    }
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod lib_test;
