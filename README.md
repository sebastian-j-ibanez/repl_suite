# repl_suite
[![term_manager](https://img.shields.io/badge/term_manager-v0.1.2-orange?style=flat)](https://github.com/sebastian-j-ibanez/repl_suite/tree/main/term_manager)
[![repl_lib](https://img.shields.io/badge/repl_lib-v0.1.0-orange?style=flat)](https://github.com/sebastian-j-ibanez/repl_suite/tree/main/repl_lib)
[![repl_demo](https://img.shields.io/badge/repl_lib-v0.1.0-orange?style=flat)](https://github.com/sebastian-j-ibanez/repl_suite/tree/main/repl_demo)
![GitHub License](https://img.shields.io/github/license/sebastian-j-ibanez/repl_suite)

A Rust workspace for building interactive REPL applications.

### Crates

**term_manager**: Wrapper around `libc::termios` for fine-grained stdin/stdout control.

**repl_lib**: Library for creating REPL interpreters with `term_manager`.

**repl_demo**: Demo REPL built using `term_manager` and `repl_lib`.