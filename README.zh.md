<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a id="readme-top"></a>

<!-- PROJECT SHIELDS -->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![project_license][license-shield]][license-url]
[![Built With Ratatui](https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff)](https://ratatui.rs/)



<!-- PROJECT LOGO -->
<br />
<div align="center">
  <!--
  <a href="https://github.com/CharOfString/ptre">
    <img src="images/logo.png" alt="Logo" width="80" height="80">
  </a>
  -->

<h3 align="center">Pointer Editor</h3>

  <p align="center">
    使用Rust+ratatui开发的文本编辑器项目，目前正在早期开发阶段
    <br />
    <a href="./doc/"><strong>探索文档 »</strong></a>
    <br />
    <br />
    <a href="https://github.com/CharOfString/ptre/actions">CI状态</a>
    &middot;
    <a href="https://github.com/CharOfString/ptre/issues">报告问题</a>
    &middot;
    <a href="https://github.com/CharOfString/ptre/issues">请求功能</a>
  </p>
</div>



<!-- ABOUT THE PROJECT -->
## 关于项目
![Screenshot](./doc/imgs/screenshot.png)
*Pointer Editor* (以下简称Pointer或`ptre`) 是一个正在早期开发阶段的、使用Rust语言编写的轻量级编辑器。

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>



### 技术栈

* Rust
* Cargo
* Ratatui

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>



<!-- GETTING STARTED -->
## 开始构建

### 前置要求
您需要在构建机上安装Rust与Cargo。请注意，在构建期间您需要保持网络连接以允许Cargo下载其依赖。

### 构建和运行
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

### 打包
#### Debian
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

请注意：执行清理后构建产物会被一并删除。

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>



<!-- USAGE EXAMPLES -->
## 使用方式
「使用方式」章节现在已经移动至`doc/`目录，以下列表列出了一些有用的文档：

* [快捷键](./doc/shortcuts/shortcuts.zh.md)
* [插件](./doc/plugins/plugins.zh.md)

_对于更多示例，请访问[文档目录](./doc/)。_

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>



<!-- ROADMAP -->
## 里程碑

- [X] C/C++编辑器
  - [X] C/C++代码自动补全
  - [X] C/C++ Clang-tidy/Cpplint报错显示
  - [X] C/C++报错自动修复
- [ ] Rust编辑器
  - [ ] 提供代码自动补全
  - [ ] 提供linting支持
- [ ] 程序
  - [ ] 提供图标模式
  - [ ] 提供国际化翻译
- [ ] 打包
  - [X] Debian
  - [ ] Archlinux
  - [ ] Fedore/OpenSUSE
  - [ ] Nix
  - [ ] AmberPM

对于提出的功能请求 (以及已知缺陷)，可以参考[开启的issues](https://github.com/CharOfString/ptre/issues)。

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>



<!-- CONTRIBUTING -->
## Contributing
正是各种贡献，让开源社区成为了一个学习、激发灵感和进行创造的绝佳之地。我们**非常感谢**您的任何贡献。

与此同时，在贡献前烦请阅读 *[贡献指南](./CONTRIBUTING.md)* 与 *[行为守则](./CODE_OF_CONDUCT.md)*，万分感谢！

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>

### 本仓库的贡献者

<a href="https://github.com/CharOfString/ptre/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=CharOfString/ptre" alt="contrib.rocks image" />
</a>



<!-- LICENSE -->
## 许可证
`ptre`以*GNU GENERAL PUBLIC LICENSE  Version 3*协议授权，您可以在[这里](./LICENSE)找到许可证文件。

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## 鸣谢
特别感谢以下库：
* **Crossterm-rs**: https://github.com/crossterm-rs/crossterm
* **Lsp-types**: https://github.com/gluon-lang/lsp-types
* **Ratatui**: https://github.com/ratatui/ratatui
* **Ratatui-code-editor**: https://github.com/vipmax/ratatui-code-editor
* **Serde-json**: https://github.com/serde-rs/json
* **Best-readme-template**: https://github.com/othneildrew/Best-README-Template

要获取所有依赖信息，请检查[Cargo.lock](./Cargo.lock)。

<p align="right">(<a href="#readme-top">回到顶部</a>)</p>



<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/CharOfString/ptre.svg?style=flat
[contributors-url]: https://github.com/CharOfString/ptre/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/CharOfString/ptre.svg?style=flat
[forks-url]: https://github.com/CharOfString/ptre/network/members
[stars-shield]: https://img.shields.io/github/stars/CharOfString/ptre.svg?style=flat
[stars-url]: https://github.com/CharOfString/ptre/stargazers
[issues-shield]: https://img.shields.io/github/issues/CharOfString/ptre.svg?style=flat
[issues-url]: https://github.com/CharOfString/ptre/issues
[license-shield]: https://img.shields.io/github/license/CharOfString/ptre.svg?style=flat
[license-url]: https://github.com/CharOfString/ptre/blob/master/LICENSE.txt
[product-screenshot]: images/screenshot.png
<!-- Shields.io badges. You can a comprehensive list with many more badges at: https://github.com/inttter/md-badges -->
[Next.js]: https://img.shields.io/badge/next.js-000000?style=flat&logo=nextdotjs&logoColor=white
[Next-url]: https://nextjs.org/
[React.js]: https://img.shields.io/badge/React-20232A?style=flat&logo=react&logoColor=61DAFB
[React-url]: https://reactjs.org/
[Vue.js]: https://img.shields.io/badge/Vue.js-35495E?style=flat&logo=vuedotjs&logoColor=4FC08D
[Vue-url]: https://vuejs.org/
[Angular.io]: https://img.shields.io/badge/Angular-DD0031?style=flat&logo=angular&logoColor=white
[Angular-url]: https://angular.io/
[Svelte.dev]: https://img.shields.io/badge/Svelte-4A4A55?style=flat&logo=svelte&logoColor=FF3E00
[Svelte-url]: https://svelte.dev/
[Laravel.com]: https://img.shields.io/badge/Laravel-FF2D20?style=flat&logo=laravel&logoColor=white
[Laravel-url]: https://laravel.com
[Bootstrap.com]: https://img.shields.io/badge/Bootstrap-563D7C?style=flat&logo=bootstrap&logoColor=white
[Bootstrap-url]: https://getbootstrap.com
[JQuery.com]: https://img.shields.io/badge/jQuery-0769AD?style=flat&logo=jquery&logoColor=white
[JQuery-url]: https://jquery.com 
