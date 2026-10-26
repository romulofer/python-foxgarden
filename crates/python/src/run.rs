//! Running a Python file: the interpreter a project asks for, and the process
//! that launches the file with it.
//!
//! There is nothing to compile, so a run starts the file at once.

use std::path::{Path, PathBuf};

use fg_extension::{CommandSpec, RunSpec};

/// Folders a project keeps its virtual environment in, in the order they are
/// looked for.
const ENVIRONMENT_DIRS: [&str; 2] = [".venv", "venv"];

/// The interpreter of the virtual environment inside the project, if it has one.
pub(crate) fn environment_python(project_root: &Path) -> Option<PathBuf> {
    environment_program(project_root, if cfg!(windows) { "python.exe" } else { "python" })
}

/// A program installed into the project's virtual environment (`bin/<name>`,
/// `Scripts/<name>.exe` on Windows), if the project has the environment and the
/// program is in it.
pub(crate) fn environment_program(project_root: &Path, name: &str) -> Option<PathBuf> {
    let bin = if cfg!(windows) { "Scripts" } else { "bin" };
    ENVIRONMENT_DIRS
        .iter()
        .map(|dir| project_root.join(dir).join(bin).join(name))
        .find(|candidate| candidate.is_file())
}

/// The interpreter for the project at `project_root`: the one in its virtual
/// environment when it has one, else whatever `python3` (`python` on Windows)
/// the path finds.
pub(crate) fn interpreter(project_root: &Path) -> PathBuf {
    environment_python(project_root).unwrap_or_else(|| PathBuf::from(if cfg!(windows) { "python" } else { "python3" }))
}

/// The process that runs `run.entry_point`, a path to the file, with the
/// project's interpreter. Output is unbuffered: the run panel reads a pipe,
/// and Python buffers a pipe's output until the program ends.
pub(crate) fn command(project_root: &Path, run: &RunSpec) -> CommandSpec {
    let cwd = run.working_dir.clone().unwrap_or_else(|| project_root.to_path_buf());
    let mut spec = CommandSpec::new(interpreter(project_root), cwd)
        .args(run.vm_args.split_whitespace())
        .arg(run.entry_point.clone())
        .args(run.program_args.split_whitespace())
        .env("PYTHONUNBUFFERED", "1");
    for (key, value) in &run.env {
        spec = spec.env(key.clone(), value.clone());
    }
    spec
}

#[cfg(test)]
#[path = "run_test.rs"]
mod run_test;
