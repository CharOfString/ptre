# Pointer Editor
*Pointer Editor* (以下简称Pointer或`ptre`) 是一个正在早期开发阶段的、使用Rust语言编写的轻量级编辑器。

![Screenshot](./doc/imgs/screenshot.png)

## 构建和运行
请确保您安装了Rust和Cargo。您可以用此指令执行构建：
```bash
$ cargo build
```

二进制文件可以在`target/debug/build/ptre`找到。

要直接运行代码，请执行：
```bash
$ cargo run
```

要在缓冲区打开文件，将其(绝对/相对)路径传入参数，例如：
```bash
$ cargo run -- README.md
```
同时传入多个文件、传入目录会被`ptre`拒绝

## 打包
### Debian
> **注意**: 您必须在打包期间保持网络连接

对于第一次打`.deb`包，请执行：
```bash
$ ./build-deb -d
```

依赖会被自动下载安装至您的系统，下一次只需要执行如下命令即可：

```bash
$ ./build-deb
```

构建后要清理仓库，请执行：
```bash
$ ./build-deb -c
```

## 快捷键
> **注意**: 举个例子，比如这里写了`Ctrl + X` `Ctrl + S`，它的意思是先按`Ctrl + X`，然后马上按`Ctrl + S`。

* **保存**: `Ctrl + X` `Ctrl + S`.
* **退出**: `Ctrl + X` `Ctrl + C`.
* **剪切**: `Ctrl + W`.
* **复制**: `Alt + W`.
* **粘贴**: `Ctrl + Y`.
* **补全列表**: `Alt + /`.
* **自动补全**: `Ctrl + C` `Alt + L`.

## 插件
### 代码补全服务
代码补全由语言服务器，也就是LSP提供，通过JSON-RPC协议与`ptre`通信。

每个语言服务器需要一份配置文件：一份以语言名称 (实际上是以`ratatui-code-editor`内部的语言字段命名的) 命名的`.conf`配置文件，内含关于调用命令的说明。

`ptre`将会从`~/.config/ptre/plugins/lsp/`或`$XDG_CONFIG_HOME/ptre/plugins/lsp/`读取配置文件。

#### 命名
文件名由语言名加上`.conf`扩展名构成，可用的语言名有: `text`、`rust`、`javascript`、`typescript`、`python`、`go`、`java`、 `c_sharp`、`c`、`cpp`、`html`、`css`、`yaml`、`json`、`toml`、`shell`、`markdown`。

#### 内部字段
* **command**: 语言服务器名称与参数，以空格分隔。如果参数包含空格，请使用引号。
* **enabled**: (可选) 决定是否启用该语言的补全服务，接受`true`或`false`作为值，默认为启用状态。

#### 自带语言支持
`ptre`使用`clangd`作为C/C++的语言服务器；使用`rust-analyzer`作为Rust的语言服务器。如果需要，您可以自行编写`c.conf`、`cpp.conf`和`rust.conf`覆盖它们。


#### 行为
如果该语言服务器的二进制可执行文件未找到，则会放弃使用语言服务器，转而改用当前文件里的单词来补全。

## C++ 检查
仅在 C++ 模式下，LSP 菜单显示独立的 Clang-tidy 和 Cpplint 开关，默认均关闭。首次启用 Cpplint 时需要输入其可执行文件路径，支持空格和 `~/`。

* **Cpplint**：先按 `Ctrl + C`，再按普通的 `l`。
* **Clang-tidy**：先按 `Ctrl + C`，再按普通的 `t`；需要在 `PATH` 中安装 `clang-tidy`。

检查前请先保存缓冲区。检查在后台运行，不修改文件。结果窗口显示标准输出、错误输出和退出状态，使用方向键或 PageUp/PageDown 滚动，Esc 关闭。Clang-tidy 使用其常规项目配置；ptre 不额外传入编译参数或自动应用修复。

开关和 Cpplint 路径保存在 `$XDG_CONFIG_HOME/ptre/cpp-checks.json`，未设置时使用 `~/.config/ptre/cpp-checks.json`。可修改其中的 `cpplint_path`，在下次启动时使用新的路径。

## 依赖
特别感谢以下库：
* **Crossterm-rs**: https://github.com/crossterm-rs/crossterm
* **Lsp-types**: https://github.com/gluon-lang/lsp-types
* **Ratatui**: https://github.com/ratatui/ratatui
* **Ratatui-code-editor**: https://github.com/vipmax/ratatui-code-editor
* **Serde-json**: https://github.com/serde-rs/json

要获取所有依赖信息，请检查[Cargo.lock](./Cargo.lock)。

## 许可证
Pointer以*GNU GENERAL PUBLIC LICENSE  Version 3*协议授权，您可以在[这里](./LICENSE)找到许可证文件。
