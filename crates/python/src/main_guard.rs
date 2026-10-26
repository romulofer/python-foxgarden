//! Finding where a Python file can be run from: its `if __name__ == "__main__":`
//! block, the line a ▶ goes beside.
//!
//! Syntactic, from the tree the editor already parsed, so the marker is there
//! the instant the file opens.

use std::path::Path;

use fg_extension::{RunKind, RunTarget};
use tree_sitter::{Node, Tree};

/// The marker for `file`, when it has a main guard at module level. A guard
/// nested in a function or class never runs on its own.
pub(crate) fn main_guard(tree: &Tree, source: &str, file: &Path) -> Vec<RunTarget> {
    let root = tree.root_node();
    let mut cursor = root.walk();
    root.named_children(&mut cursor)
        .filter(|node| node.kind() == "if_statement")
        .filter(|node| node.child_by_field_name("condition").is_some_and(|c| is_main_comparison(c, source)))
        .map(|node| RunTarget {
            line: node.start_position().row,
            kind: RunKind::Main,
            entry_point: file.display().to_string(),
            label: file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        })
        .take(1)
        .collect()
}

/// `__name__ == "__main__"`, in either order and with either quote, with or
/// without parentheses around it.
fn is_main_comparison(condition: Node, source: &str) -> bool {
    let text: String = source[condition.byte_range()].chars().filter(|c| !c.is_whitespace()).collect();
    let text = text.trim_start_matches('(').trim_end_matches(')');
    ["__name__==\"__main__\"", "__name__=='__main__'", "\"__main__\"==__name__", "'__main__'==__name__"].contains(&text)
}

#[cfg(test)]
#[path = "main_guard_test.rs"]
mod main_guard_test;
