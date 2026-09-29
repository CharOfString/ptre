# 插件
## 代码补全服务
代码补全由语言服务器，也就是LSP提供，通过JSON-RPC协议与`ptre`通信。

每个语言服务器需要一份配置文件：一份以语言名称 (实际上是以`ratatui-code-editor`内部的语言字段命名的) 命名的`.conf`配置文件，内含关于调用命令的说明。

`ptre`将会从`~/.config/ptre/plugins/lsp/`或`$XDG_CONFIG_HOME/ptre/plugins/lsp/`读取配置文件。

### 命名
文件名由语言名加上`.conf`扩展名构成，可用的语言名有: `text`、`rust`、`javascript`、`typescript`、`python`、`go`、`java`、 `c_sharp`、`c`、`cpp`、`html`、`css`、`yaml`、`json`、`toml`、`shell`、`markdown`。

### 内部字段
* **command**: 语言服务器名称与参数，以空格分隔。如果参数包含空格，请使用引号。
* **enabled**: (可选) 决定是否启用该语言的补全服务，接受`true`或`false`作为值，默认为启用状态。

### 自带语言支持
`ptre`使用`clangd`作为C/C++的语言服务器；使用`rust-analyzer`作为Rust的语言服务器。如果需要，您可以自行编写`c.conf`、`cpp.conf`和`rust.conf`覆盖它们。


### 行为
如果该语言服务器的二进制可执行文件未找到，则会放弃使用语言服务器，转而改用当前文件里的单词来补全。
