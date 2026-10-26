# python-foxgarden

Python support for [FoxGarden](../foxgarden), as an extension rather than as
part of the editor.

It reaches the editor only through `fg-extension`, like
[`spring-foxgarden`](../spring-foxgarden) and [`kmp-foxgarden`](../kmp-foxgarden).
This checkout must sit beside `foxgarden`, which compiles it in.

## What it contributes today

| Contribution | Detail |
|---|---|
| Language | `python` (`.py`, `.pyi`, `.pyw`) and its file icon |
| Grammar | `tree-sitter-python` with its own highlight query (`queries/highlights_python.scm`, forked unchanged), and the node kinds for sticky scroll, folding and import blocks |
| Build tool | `python`, found by `pyproject.toml`, `requirements.txt`, `setup.py`, `setup.cfg`, `Pipfile` or `main.py`. There is nothing to compile, so a run launches at once, with the project's `.venv`/`venv` interpreter or `python3` (`python` on Windows), unbuffered so the run panel sees output as it happens |
| Run marker | A ▶ on a module-level `if __name__ == "__main__":`, and on each pytest test (`test_*` functions, `Test*` or `unittest.TestCase` classes and their methods) in a `test_*.py` / `*_test.py` file, run as `python -m pytest <node id>` |
| Language server | [`jedi-language-server`](https://github.com/pappasam/jedi-language-server) (Jedi behind LSP): completion, hover, go to definition, references, rename, symbols |

The language server is not bundled; install it with
`pip install jedi-language-server`. FoxGarden looks for it in the project's
virtual environment (`.venv/bin`, `venv/bin`), then on `PATH`; a path set in
Settings ▸ Language Servers… wins over both. A project that has a virtual
environment hands its interpreter to Jedi (`workspace.environmentPath`), so
imports resolve against the packages the project installed.

## Not here yet

A pass/fail summary of pytest runs, run configurations for scripts, a debugger (`debugpy`), a
virtual-environment picker, a project scaffold and an installer for the language
server.

## Development

```sh
cargo test --workspace
```

The language server is exercised end to end by `#[ignore]`d tests in
`foxgarden` (`FOXGARDEN_JEDI_LANGUAGE_SERVER=<path> cargo test -p foxgarden --bin
foxgarden -- --ignored jedi`).
