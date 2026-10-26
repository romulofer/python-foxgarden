use super::*;

fn targets(source: &str) -> Vec<RunTarget> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into()).unwrap();
    let tree = parser.parse(source, None).unwrap();
    main_guard(&tree, source, Path::new("/p/app.py"))
}

#[test]
fn a_main_guard_gets_a_marker_naming_the_file() {
    let found = targets("def main():\n    pass\n\n\nif __name__ == \"__main__\":\n    main()\n");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line, 4);
    assert_eq!(found[0].kind, RunKind::Main);
    assert_eq!(found[0].entry_point, "/p/app.py");
    assert_eq!(found[0].label, "app.py");
}

#[test]
fn single_quotes_and_the_reverse_order_count_too() {
    assert_eq!(targets("if __name__ == '__main__':\n    pass\n").len(), 1);
    assert_eq!(targets("if '__main__' == __name__:\n    pass\n").len(), 1);
    assert_eq!(targets("if (__name__ == \"__main__\"):\n    pass\n").len(), 1);
}

#[test]
fn a_file_without_a_main_guard_has_no_marker() {
    assert!(targets("import os\n\n\ndef main():\n    pass\n").is_empty());
    assert!(targets("if __name__ == \"other\":\n    pass\n").is_empty());
}

#[test]
fn a_guard_inside_a_function_is_not_a_way_to_run_the_file() {
    assert!(targets("def f():\n    if __name__ == \"__main__\":\n        pass\n").is_empty());
}
