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
    A lightweight text editor written using Rust w/ EMACS styled key binding.
    <br />
    <a href="./doc/"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/CharOfString/ptre/actions">CI Status</a>
    &middot;
    <a href="https://github.com/CharOfString/ptre/issues">Report Bug</a>
    &middot;
    <a href="https://github.com/CharOfString/ptre/issues">Request Feature</a>
  </p>
</div>



<!-- ABOUT THE PROJECT -->
## About The Project
![Screenshot](./doc/imgs/screenshot.png)
*Pointer Editor* (we'll call it Pointer or `ptre` in the rest of the document) is a currently under developing (we just started it) lightweight text editor in Rust.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With

* Rust
* Cargo
* Ratatui

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started

### Prerequisites

You will need to have Rust & Cargo ready on your system. Please note that network is required as Cargo will download its dependencies.

### Build & Run
Please have Rust & Cargo ready, then you may use the following command to build:
```shell
$ cargo build
```

You may find the binary file on `target/debug/build/ptre`.

To run the code directly:
```shell
$ cargo run
```

To open an existing file in the editor buffer, pass its relative or absolute path:
```shell
$ cargo run -- README.md
```
Directories and multiple arguments are rejected.

### Packaging
#### Debian
> **NOTE**: Network connection WILL BE REQUIRED!!

To build a Debian binary package, simply run:

```shell
$ ./build-deb -d
```

Dependencies will automatically being installed to your computer.

Next time you may run:

```shell
$ ./build-deb
```

To do a cleanup, run:
```shell
$ ./build-deb -c
```

Note that this will also clear the artifacts generated.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage

The usage section has been moved to the `doc/` folder. The list below shows some handy documents.

* [Shortcut Keys](./doc/shortcuts/shortcuts.md)
* [Plugins](./doc/plugins/plugins.md)

_For more examples, please refer to the [documentation directory](./doc/)._

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ROADMAP -->
## Roadmap

- [ ] C/C++ editor buffer
  - [X] C/C++ code auto completion.
  - [ ] C/C++ Clang-tidy/Cpplint error displaying.
  - [ ] C/C++ error autofix.
- [ ] Rust editor buffer
  - [ ] Provide code auto completion.
  - [ ] Provide linting.
- [ ] Application
  - [ ] Provide color icon mode. 
  - [ ] Provide i18n translations.
- [ ] Packaging
  - [X] Debian.
  - [ ] Archlinux.
  - [ ] Fedore/OpenSUSE.
  - [ ] Nix.
  - [ ] AmberPM.

See the [open issues](https://github.com/CharOfString/ptre/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

Please refer to the *[Contributor Guide](./CONTRIBUTING.md)* and *[Code of Conduct](./CODE_OF_CONDUCT.md)* before contributing, thanks.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Top contributors:

<a href="https://github.com/CharOfString/ptre/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=CharOfString/ptre" alt="contrib.rocks image" />
</a>



<!-- LICENSE -->
## License
`ptre` is licensed under *GNU GENERAL PUBLIC LICENSE  Version 3*, you may find a copy of the license [here](./LICENSE).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments
Special thanks to these libraries:
* **Crossterm-rs**: https://github.com/crossterm-rs/crossterm
* **Lsp-types**: https://github.com/gluon-lang/lsp-types
* **Ratatui**: https://github.com/ratatui/ratatui
* **Ratatui-code-editor**: https://github.com/vipmax/ratatui-code-editor
* **Serde-json**: https://github.com/serde-rs/json
* **Best-readme-template**: https://github.com/othneildrew/Best-README-Template

For all dependencies, please refer to [Cargo.lock](./Cargo.lock).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



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
