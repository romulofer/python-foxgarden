//! Finding the tests in a file for the run gutter: pytest's `test_*` functions
//! and the `Test*` classes (or `unittest.TestCase` subclasses) holding `test_*`
//! methods.
//!
//! Syntactic, like [`crate::main_guard`], so the markers are there the instant
//! the file opens. The entry point is a pytest node id (`<file>::Class::name`),
//! which `pytest` takes as it is.

use std::path::Path;

use fg_extension::{RunKind, RunTarget};
use tree_sitter::{Node, Tree};

/// The markers for `file`, in source order, when its name is one pytest collects
/// (`test_*.py`, `*_test.py`): a marker on each module-level test function, on
/// each test class that has a test method, and on each of those methods.
pub(crate) fn test_targets(tree: &Tree, source: &str, file: &Path) -> Vec<RunTarget> {
    if !is_test_file(file) {
        return Vec::new();
    }
    let path = file.display().to_string();
    let mut out = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        let definition = definition_of(child);
        match definition.kind() {
            "function_definition" => {
                if let Some(name) = test_function_name(definition, source) {
                    out.push(single(&path, None, name, definition));
                }
            }
            "class_definition" => class_targets(definition, source, &path, &mut out),
            _ => {}
        }
    }
    out
}

fn class_targets(class: Node, source: &str, path: &str, out: &mut Vec<RunTarget>) {
    let Some(class_name) = class.child_by_field_name("name").map(|n| &source[n.byte_range()]) else { return };
    let Some(body) = class.child_by_field_name("body") else { return };
    if !is_test_class(class, class_name, source) {
        return;
    }
    let mut cursor = body.walk();
    let methods: Vec<(&str, Node)> = body
        .named_children(&mut cursor)
        .map(definition_of)
        .filter(|node| node.kind() == "function_definition")
        .filter_map(|node| test_function_name(node, source).map(|name| (name, node)))
        .collect();
    if methods.is_empty() {
        return;
    }
    out.push(RunTarget {
        line: class.child_by_field_name("name").map_or(class.start_position().row, |n| n.start_position().row),
        kind: RunKind::Test,
        entry_point: format!("{path}::{class_name}"),
        label: class_name.to_string(),
    });
    for (name, node) in methods {
        out.push(single(path, Some(class_name), name, node));
    }
}

fn single(path: &str, class: Option<&str>, name: &str, function: Node) -> RunTarget {
    let entry_point = match class {
        Some(class) => format!("{path}::{class}::{name}"),
        None => format!("{path}::{name}"),
    };
    RunTarget {
        line: function.child_by_field_name("name").map_or(function.start_position().row, |n| n.start_position().row),
        kind: RunKind::SingleTest,
        entry_point,
        label: name.to_string(),
    }
}

/// A decorated definition is a wrapper whose `definition` field is the thing
/// itself; anything else is its own definition.
fn definition_of(node: Node) -> Node {
    if node.kind() == "decorated_definition" { node.child_by_field_name("definition").unwrap_or(node) } else { node }
}

fn test_function_name<'a>(function: Node, source: &'a str) -> Option<&'a str> {
    let name = &source[function.child_by_field_name("name")?.byte_range()];
    name.starts_with("test").then_some(name)
}

/// pytest collects `Test*` classes; `unittest` ones count whatever they are
/// called, found by a base named `TestCase`.
fn is_test_class(class: Node, name: &str, source: &str) -> bool {
    name.starts_with("Test")
        || class.child_by_field_name("superclasses").is_some_and(|bases| {
            let text = &source[bases.byte_range()];
            text.split(|c: char| !(c.is_alphanumeric() || c == '_')).any(|word| word == "TestCase")
        })
}

fn is_test_file(file: &Path) -> bool {
    file.file_name().and_then(|n| n.to_str()).is_some_and(|name| {
        name.strip_suffix(".py").is_some_and(|stem| stem.starts_with("test_") || stem.ends_with("_test"))
    })
}

#[cfg(test)]
#[path = "test_entry_test.rs"]
mod test_entry_test;
