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

To open an existing file in the editor buffer, pass its relative or absolute path:
```bash
$ cargo run -- README.md
```
Directories and multiple arguments are rejected.

## Packaging
### Debian
> **NOTE**: Network connection WILL BE REQUIRED!!

To build a Debian binary package, simply run:

```bash
$ ./build-deb -d
```

Dependencies will automatically being installed to your computer.

Next time you may run:

```bash
$ ./build-deb
```

To do a cleanup, run:
```bash
$ ./build-deb -c
```

## Shortcuts
> **Note**: If we provide shortcuts like `Ctrl + X` `Ctrl + S`, it means that you'll need to press `Ctrl + X` first and 
> hit `Ctrl + S` immediately after then.

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
