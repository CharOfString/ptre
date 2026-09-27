# Project instructions
- Keep every individual Rust source file (`*.rs`) at or below 600 lines. Split a file into modules before it exceeds this limit.
- After finishing code changes, run `update-license` to add missing license headers, then run `./format-code` to format Rust code.
- After that, you'll have to run `./lint-code` to ensure you have passed the linting tests. Any changes that failed the linting tests WILL BE REJECTED.
- Before reporting completion, verify that no `*.rs` file exceeds 600 lines and that `./update_license --check` and `./format-code --check` pass.
