//! Python support for FoxGarden, as an extension.
//!
//! Contributes the language (`.py`, `.pyi`, `.pyw`) and its tree-sitter grammar
//! with a highlight query. Nothing else yet: no language server, build tool
//! or run markers.

use fg_extension::{
    CURRENT_SCHEMA_VERSION, Contributions, Extension, ExtensionManifest, FileIcon, GrammarContribution, GrammarSource, LanguageContribution,
    LanguageId, NodeKinds,
};

pub const PYTHON: &str = "python";

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

    fn contributions(&self) -> Contributions {
        Contributions {
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
            // The Nerd Fonts devicons Python glyph.
            file_icons: vec![FileIcon { language_id: PYTHON.to_string(), glyph: '\u{e73c}' }],
            ..Contributions::default()
        }
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
