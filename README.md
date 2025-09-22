# repl_suite
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/sebastian-j-ibanez/repl_suite/cargo.yml)
![GitHub License](https://img.shields.io/github/license/sebastian-j-ibanez/repl_suite?color=orange)

A Rust workspace for building interactive REPL applications.

### Crates

[term_manager](https://github.com/sebastian-j-ibanez/repl_suite/tree/main/term_manager): Wrapper around `libc::termios` for fine-grained stdin/stdout control.

[repl_lib](https://github.com/sebastian-j-ibanez/repl_suite/tree/main/repl_lib): Library for creating REPL interpreters with `term_manager`.

[repl_demo](https://github.com/sebastian-j-ibanez/repl_suite/tree/main/repl_demo): Demo REPL built using `term_manager` and `repl_lib`.
