use std::path::PathBuf;

use super::*;

fn context(root: &Path, configured: &str) -> ServerStartContext {
    ServerStartContext {
        configured_binary: configured.to_string(),
        project_root: Some(root.to_path_buf()),
        ..ServerStartContext::default()
    }
}

#[cfg(not(windows))]
fn environment(root: &Path, with_server: bool) -> PathBuf {
    let bin = root.join(".venv/bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(bin.join("python"), "").unwrap();
    if with_server {
        std::fs::write(bin.join(JEDI_BINARY), "").unwrap();
    }
    bin
}

#[test]
fn the_server_serves_python_and_declares_the_binary_to_look_up() {
    let server = contribution();
    assert_eq!(server.language_ids, ["python"]);
    assert_eq!(server.binary_name, "jedi-language-server");
    assert!(server.install.is_none(), "the user installs it");
}

#[test]
fn without_a_project_environment_the_server_is_looked_up_on_the_path() {
    let dir = tempfile::tempdir().unwrap();
    let started = resolve(&context(dir.path(), ""));
    assert_eq!(started.binary, PathBuf::from("jedi-language-server"));
    assert_eq!(started.initialization_options, None);
    assert_eq!(started.restart_key, "");
}

#[cfg(not(windows))]
#[test]
fn a_server_installed_in_the_project_environment_is_preferred_to_the_path() {
    let dir = tempfile::tempdir().unwrap();
    let bin = environment(dir.path(), true);
    assert_eq!(resolve(&context(dir.path(), "")).binary, bin.join("jedi-language-server"));
}

#[cfg(not(windows))]
#[test]
fn the_environments_interpreter_is_handed_to_jedi_to_resolve_imports_against() {
    let dir = tempfile::tempdir().unwrap();
    let bin = environment(dir.path(), false);
    let started = resolve(&context(dir.path(), ""));
    assert_eq!(started.binary, PathBuf::from("jedi-language-server"), "the server itself is not in it");
    let options: serde_json::Value = serde_json::from_str(&started.initialization_options.unwrap()).unwrap();
    assert_eq!(options["workspace"]["environmentPath"], bin.join("python").display().to_string());
    assert_eq!(started.restart_key, bin.join("python").display().to_string());
}

#[cfg(not(windows))]
#[test]
fn a_path_configured_in_settings_wins() {
    let dir = tempfile::tempdir().unwrap();
    environment(dir.path(), true);
    assert_eq!(resolve(&context(dir.path(), "/opt/jedi")).binary, PathBuf::from("/opt/jedi"));
}
