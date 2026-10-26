use fg_extension::Registry;

use super::*;

const SAMPLE: &str = "import os\n\n\nclass Greeter:\n    def hello(self, name):\n        return f\"hi {name}\"\n";

fn grammar() -> GrammarContribution {
    PythonExtension.contributions().grammars.remove(0)
}

fn parse(source: &str) -> tree_sitter::Tree {
    let GrammarSource::Builtin(language) = grammar().source else { panic!("a builtin grammar") };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language.into()).expect("the bundled grammar must load");
    parser.parse(source, None).expect("parse")
}

fn kinds(tree: &tree_sitter::Tree) -> Vec<&'static str> {
    let mut found = Vec::new();
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        found.push(node.kind());
        let mut cursor = node.walk();
        stack.extend(node.children(&mut cursor));
    }
    found
}

#[test]
fn the_manifest_names_the_extension() {
    let manifest = PythonExtension.manifest();
    assert_eq!(manifest.id, "python");
    assert_eq!(manifest.name, "Python");
    assert_eq!(manifest.schema_version, CURRENT_SCHEMA_VERSION);
}

#[test]
fn the_highlight_query_compiles_against_the_grammar() {
    let GrammarSource::Builtin(language) = grammar().source else { panic!("a builtin grammar") };
    let query = grammar().highlight_query.expect("a highlight query");
    tree_sitter::Query::new(&language.into(), &query).expect("the query must compile against its own grammar");
}

#[test]
fn every_node_kind_the_editor_is_told_about_exists_in_a_parse() {
    let found = kinds(&parse(SAMPLE));
    let node_kinds = grammar().node_kinds;
    for kind in node_kinds.scopes.iter().chain(&node_kinds.foldable).chain(node_kinds.import.iter()) {
        assert!(found.contains(&kind.as_str()), "{kind} is not a node of the sample's parse");
    }
}

#[test]
fn python_files_are_recognized_by_extension() {
    let mut registry = Registry::new();
    registry.register(Box::new(PythonExtension)).unwrap();
    for file in ["main.py", "types.pyi", "run.pyw"] {
        let extension = file.rsplit('.').next().unwrap();
        assert_eq!(registry.language_for_extension(extension).map(|l| l.language.id.as_str()), Some(PYTHON), "{file}");
    }
}

#[test]
fn the_extension_declares_that_it_starts_the_interpreter() {
    assert_eq!(PythonExtension.capabilities(), vec![Capability::ProcessExec]);
}

#[test]
fn a_python_project_is_found_by_its_marker_files_and_nothing_compiles() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("pyproject.toml"), "").unwrap();
    let mut registry = Registry::new();
    registry.register(Box::new(PythonExtension)).unwrap();
    let tool = registry.detect_build_tool(dir.path()).expect("pyproject.toml marks a Python project");
    assert_eq!(tool.id, PYTHON_TOOL);
    assert!(tool.command(dir.path(), fg_extension::BuildTask::Compile).is_none());
}

#[test]
fn the_tool_launches_a_file_through_the_interpreter() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("main.py"), "").unwrap();
    let mut registry = Registry::new();
    registry.register(Box::new(PythonExtension)).unwrap();
    let tool = registry.detect_build_tool(dir.path()).unwrap();
    let run = RunSpec {
        entry_point: "main.py".to_string(),
        ..RunSpec::default()
    };
    let launch = tool.run_command(dir.path(), &run).unwrap().unwrap();
    assert_eq!(launch.args, ["main.py"]);
}

#[test]
fn a_main_guard_is_a_run_target_only_for_python() {
    let source = "if __name__ == \"__main__\":\n    pass\n";
    let tree = parse(source);
    assert_eq!(PythonExtension.run_targets(PYTHON, &tree, source, Path::new("a.py")).len(), 1);
    assert!(PythonExtension.run_targets("java", &tree, source, Path::new("a.py")).is_empty());
}
