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

## 依赖
特别感谢以下库：
* **Ratatui**: 提供了TUI图形库。
* **Ratatui-code-editor**: 驱动了我们的编辑器缓冲区。

要获取所有依赖信息，请检查[Cargo.lock](./Cargo.lock)。

## 许可证
Pointer以*GNU GENERAL PUBLIC LICENSE  Version 3*协议授权，您可以在[这里](./LICENSE)找到许可证文件。
