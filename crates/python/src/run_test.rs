use super::*;

fn spec(entry_point: &str) -> RunSpec {
    RunSpec {
        entry_point: entry_point.to_string(),
        ..RunSpec::default()
    }
}

#[test]
fn a_project_without_an_environment_uses_the_interpreter_on_the_path() {
    let dir = tempfile::tempdir().unwrap();
    let expected = if cfg!(windows) { "python" } else { "python3" };
    assert_eq!(interpreter(dir.path()), PathBuf::from(expected));
}

#[cfg(not(windows))]
#[test]
fn a_virtual_environment_in_the_project_wins() {
    let dir = tempfile::tempdir().unwrap();
    let python = dir.path().join(".venv/bin/python");
    std::fs::create_dir_all(python.parent().unwrap()).unwrap();
    std::fs::write(&python, "").unwrap();
    assert_eq!(interpreter(dir.path()), python);
}

#[cfg(not(windows))]
#[test]
fn dot_venv_is_preferred_over_venv() {
    let dir = tempfile::tempdir().unwrap();
    for env in ["venv", ".venv"] {
        let python = dir.path().join(env).join("bin/python");
        std::fs::create_dir_all(python.parent().unwrap()).unwrap();
        std::fs::write(python, "").unwrap();
    }
    assert_eq!(interpreter(dir.path()), dir.path().join(".venv/bin/python"));
}

#[test]
fn the_file_runs_with_flags_before_it_and_arguments_after() {
    let dir = tempfile::tempdir().unwrap();
    let run = RunSpec {
        entry_point: "/p/app.py".to_string(),
        vm_args: "-X dev".to_string(),
        program_args: "--port 80".to_string(),
        env: vec![("MODE".to_string(), "test".to_string())],
        working_dir: None,
    };
    let command = command(dir.path(), &run);
    assert_eq!(command.args, ["-X", "dev", "/p/app.py", "--port", "80"]);
    assert_eq!(command.cwd, dir.path());
    assert!(command.env.contains(&("MODE".to_string(), "test".to_string())));
}

#[test]
fn output_is_unbuffered_so_the_run_panel_sees_it_as_it_happens() {
    let dir = tempfile::tempdir().unwrap();
    let command = command(dir.path(), &spec("app.py"));
    assert!(command.env.contains(&("PYTHONUNBUFFERED".to_string(), "1".to_string())));
}

#[test]
fn a_working_directory_of_the_run_is_honoured() {
    let dir = tempfile::tempdir().unwrap();
    let run = RunSpec {
        working_dir: Some(PathBuf::from("/elsewhere")),
        ..spec("app.py")
    };
    assert_eq!(command(dir.path(), &run).cwd, PathBuf::from("/elsewhere"));
}

#[test]
fn a_test_run_hands_the_node_id_to_pytest_through_the_interpreter() {
    let dir = tempfile::tempdir().unwrap();
    let command = test_command(dir.path(), "/p/test_calc.py::TestCalc::test_adds").to_command();
    let args: Vec<_> = command.get_args().map(|a| a.to_string_lossy().into_owned()).collect();
    assert_eq!(args, ["-m", "pytest", "/p/test_calc.py::TestCalc::test_adds"]);
    assert_eq!(command.get_current_dir(), Some(dir.path()));
}
