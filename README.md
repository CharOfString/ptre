# Pointer Editor
*Pointer Editor* (we'll call it Pointer or `ptre` in the rest of the document) is a currently under developing (we just started it) lightweight text editor in Rust.

![Screenshot](./doc/imgs/screenshot.png)

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
* **Complete** (open/close the completion popup): `Alt + /`.
* **Toggle auto completion**: `Ctrl + C` `Alt + L`.
* **Trig Clang-tidy check**: `Ctrl + C` `t`.
* **Trig Cpplint check**: `Ctrl + C` `l`.

## Plugins
### Code Completion Service
Code (auto) completion services are provided through LSP. `ptre` communicates w/ it via JSON-RPC protocol.

Each LSP configuration needs one config file: A `.conf` file named with the language name (actually the language name var that is used by `ratatui-code-editor`) with command instructions inside.

`ptre` will read those configuration files from `~/.config/ptre/plugins/lsp/` or `$XDG_CONFIG_HOME/ptre/plugins/lsp/`.

#### File name
The file name is just the language name, while the extension name is `.conf`. Availiable names are `text`, `rust`, `javascript`, `typescript`, `python`, `go`, `java`,  `c_sharp`, `c`, `cpp`, `html`, `css`, `yaml`, `json`, `toml`, `shell`, and `markdown`.

#### Internal fields
* **command**: The ELF name of LSP and its argument(s) seperated by space. if there is a space inside the argument, please use string quotes.
* **enabled**: (Optional) Decides whether to enable the completion service. Accepts `true` or `false`, and the fall back (default) value is `true`.

#### Built-in Language Support
`ptre` uses `clangd` as LSP for C/C++, and `rust-analyzer` for Rust. You may write your own `c.conf`, `cpp.conf`, and`rust.conf` to override the settings.

#### Behavior
If the lsp ELF is NOT found, `ptre` will skip using the current LSP and only uses the context from current file to do the completion.

#### Diagnostics
Errors and warnings from the LSP are shown in the editor: a `●` next to the line number, an underline under the problem, and the message after the end of the line (red for errors, yellow for warnings). When the cursor stops on a problem, a popup below the cursor shows the full message.

## C++ Checks Service
When the language of Editor buffer is set to C/C++, there will be a C/C++ menu on the top bar with Clang-Tidy/Cpplint check switch (they are defaulting to `OFF`).

When you use the Cpplint for the first time, you will need to enter the PATH to Cpplint (you may install it through pip if you don't have one). You may include spaces or `~/` in your path.

Before running code checking manually, please save the buffer first (they only runs on saved buffers). Your report will be automatically poped up after checking is done. `ptre` won't pass extra arguments or apply automatic fix.

The config including Cpplint path may be found at `$XDG_CONFIG_HOME/ptre/cpp-checks.json`. You may manually modify the `cpplint_path` and the new path will be used the next time `ptre` runs.


## Dependencies
Special thanks to these libraries:
* **Crossterm-rs**: https://github.com/crossterm-rs/crossterm
* **Lsp-types**: https://github.com/gluon-lang/lsp-types
* **Ratatui**: https://github.com/ratatui/ratatui
* **Ratatui-code-editor**: https://github.com/vipmax/ratatui-code-editor
* **Serde-json**: https://github.com/serde-rs/json

For all dependencies, please refer to [Cargo.lock](./Cargo.lock).

## License
Pointer is licensed under *GNU GENERAL PUBLIC LICENSE  Version 3*, you may find a copy of the license [here](./LICENSE).
