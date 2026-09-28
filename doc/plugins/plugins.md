# Plugins
## Code Completion Service
Code (auto) completion services are provided through LSP. `ptre` communicates w/ it via JSON-RPC protocol.

Each LSP configuration needs one config file: A `.conf` file named with the language name (actually the language name var that is used by `ratatui-code-editor`) with command instructions inside.

`ptre` will read those configuration files from `~/.config/ptre/plugins/lsp/` or `$XDG_CONFIG_HOME/ptre/plugins/lsp/`.

### File name
The file name is just the language name, while the extension name is `.conf`. Availiable names are `text`, `rust`, `javascript`, `typescript`, `python`, `go`, `java`,  `c_sharp`, `c`, `cpp`, `html`, `css`, `yaml`, `json`, `toml`, `shell`, and `markdown`.

### Internal fields
* **command**: The ELF name of LSP and its argument(s) seperated by space. if there is a space inside the argument, please use string quotes.
* **enabled**: (Optional) Decides whether to enable the completion service. Accepts `true` or `false`, and the fall back (default) value is `true`.

### Built-in Language Support
`ptre` uses `clangd` as LSP for C/C++, and `rust-analyzer` for Rust. You may write your own `c.conf`, `cpp.conf`, and`rust.conf` to override the settings.

### Behavior
If the lsp ELF is NOT found, `ptre` will skip using the current LSP and only uses the context from current file to do the completion.
