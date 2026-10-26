use super::*;

fn targets_in(file: &str, source: &str) -> Vec<RunTarget> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into()).unwrap();
    let tree = parser.parse(source, None).unwrap();
    test_targets(&tree, source, Path::new(file))
}

fn summary(targets: &[RunTarget]) -> Vec<(usize, RunKind, &str)> {
    targets.iter().map(|t| (t.line, t.kind, t.entry_point.as_str())).collect()
}

#[test]
fn a_test_function_gets_a_marker_that_runs_that_one_test() {
    let found = targets_in("/p/test_calc.py", "def helper():\n    pass\n\n\ndef test_adds():\n    assert 1 + 1 == 2\n");
    assert_eq!(summary(&found), [(4, RunKind::SingleTest, "/p/test_calc.py::test_adds")]);
    assert_eq!(found[0].label, "test_adds");
}

#[test]
fn a_test_class_gets_a_marker_and_so_does_each_of_its_methods() {
    let found = targets_in(
        "/p/calc_test.py",
        "class TestCalc:\n    def setup_method(self):\n        pass\n\n    def test_adds(self):\n        pass\n\n    def test_subtracts(self):\n        pass\n",
    );
    assert_eq!(
        summary(&found),
        [
            (0, RunKind::Test, "/p/calc_test.py::TestCalc"),
            (4, RunKind::SingleTest, "/p/calc_test.py::TestCalc::test_adds"),
            (7, RunKind::SingleTest, "/p/calc_test.py::TestCalc::test_subtracts"),
        ]
    );
    assert_eq!(found[0].label, "TestCalc");
}

#[test]
fn decorated_and_async_tests_count() {
    let found = targets_in(
        "/p/test_x.py",
        "@pytest.mark.parametrize('n', [1, 2])\ndef test_a(n):\n    pass\n\n\nasync def test_b():\n    pass\n",
    );
    assert_eq!(
        summary(&found),
        [(1, RunKind::SingleTest, "/p/test_x.py::test_a"), (5, RunKind::SingleTest, "/p/test_x.py::test_b")]
    );
}

#[test]
fn a_unittest_case_counts_whatever_it_is_called() {
    let found = targets_in(
        "/p/test_x.py",
        "import unittest\n\n\nclass Calc(unittest.TestCase):\n    def test_adds(self):\n        pass\n",
    );
    assert_eq!(found[0].entry_point, "/p/test_x.py::Calc");
    assert_eq!(found.len(), 2);
}

#[test]
fn a_class_that_is_not_a_test_class_or_has_no_test_method_gets_nothing() {
    assert!(targets_in("/p/test_x.py", "class Helper:\n    def test_adds(self):\n        pass\n").is_empty());
    assert!(targets_in("/p/test_x.py", "class TestEmpty:\n    def helper(self):\n        pass\n").is_empty());
}

#[test]
fn a_file_pytest_does_not_collect_has_no_markers() {
    let source = "def test_adds():\n    pass\n";
    assert!(targets_in("/p/calc.py", source).is_empty());
    assert!(targets_in("/p/testing.py", source).is_empty());
    assert_eq!(targets_in("/p/calc_test.py", source).len(), 1);
}
