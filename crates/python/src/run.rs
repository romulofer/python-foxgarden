//! Running a Python file: the interpreter a project asks for, and the process
//! that launches the file with it.
//!
//! There is nothing to compile, so a run starts the file at once.

use std::path::{Path, PathBuf};

use fg_extension::{CommandSpec, RunSpec};

/// Folders a project keeps its virtual environment in, in the order they are
/// looked for.
const ENVIRONMENT_DIRS: [&str; 2] = [".venv", "venv"];

/// The interpreter for the project at `project_root`: the one in its virtual
/// environment when it has one, else whatever `python3` (`python` on Windows)
/// the path finds.
pub(crate) fn interpreter(project_root: &Path) -> PathBuf {
    let relative: [&str; 2] = if cfg!(windows) { ["Scripts", "python.exe"] } else { ["bin", "python"] };
    ENVIRONMENT_DIRS
        .iter()
        .map(|dir| project_root.join(dir).join(relative[0]).join(relative[1]))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| PathBuf::from(if cfg!(windows) { "python" } else { "python3" }))
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
