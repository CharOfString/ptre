# Pointer
Pointer is a currently under developing (we just started it) lightweight text editor in Rust.

## Build & Run
Please have Rust & Cargo ready, then you may use the following command to build:
```bash
$ cargo build
```

You may find the binary file on `target/debug/build/ptre`.

To run the code directly:
```bash
$ cargo run
```

Currently, we didn't provide any packaging.

## Shortcuts
> **Note**: If we provide shortcuts like `Ctrl + X` `Ctrl + S`, it means that you'll need to press `Ctrl + X` first and 
> hit `Ctrl + X` immediately after then.

* **Save**: `Ctrl + X` `Ctrl + S`.
* **Quit**: `Ctrl + X` `Ctrl + C`.
* **Cut**: `Ctrl + W`.
* **Copy**: `Alt + W`.
* **Paste**: `Ctrl + Y`.

## Dependencies
Special thanks to these libraries:
* **Ratatui**: for providing TUI framework.
* **Ratatui-code-editor**: Providing the core editor buffer.

For all dependencies, please refer to [Cargo.lock](./Cargo.lock).

## License
Pointer is licensed under *GNU GENERAL PUBLIC LICENSE  Version 3*, you may find a copy of the license [here](./LICENSE).
