# sys-kaleido

**sys-kaleido** provides one-stop management for some new opensource system tools, such as [dust](https://github.com/bootandy/dust), [lsd](https://github.com/lsd-rs/lsd) .etc. These tools have better output and/or speed.

**sys-kaleido** supports Linux, Windows and MacOS, and most of these systems tools have consistent experience across OS.

**[NOTICE] sys-kaleido DOSE NOT scan any installed package, so be careful to use, especially for custom packages.**

## Configuration

**sys-kaleido** uses the file [kaleido.toml](https://raw.githubusercontent.com/jinyuli/sys-kaleido/master/kaleido.toml) as all packages configuration, and it will check the file every week to make sure it's update to date. Users may also use command `sys-kaleido config update` to update the file manualy.

Uses could add their own packages/bindles, but it's not recommended to edit **kaleido.toml** directly. Uses could create a file **custom.toml** in folder `~/.sys-kaleido/`, and put all packages you like in that file. when merging **kaleido.toml** and **custom.toml**, packages/bindles in **kaleido.toml** have higher priority.

### package
a package is a system tool, for example [dust](https://github.com/bootandy/dust) is a package, in **kaleido.toml** it looks like this:
```toml
[[packages]]
name = "dust"
url = "https://github.com/bootandy/dust"
version = "1.0.0"
replace = "du"
language = "rust"
bin_name = "dust"
[packages.github]
org = "bootandy"
repo = "dust"
```

### bindle
a bindle is a collection of packages, and users could manage all packages in a bindle easier.

in configuration file:
```toml
[[bindles]]
name = "core"
[[bindles.packages]]
name = "dust"
alias = "du"
[[bindles.packages]]
name = "rmz"
alias = "rm"
[[bindles.packages]]
name = "cpz"
alias = "cp"
```

to install all packages in bindle `core`
```shell
$ sys-kaleido bindle install core
```

to uninstall all packages in bindle `core`
```shell
$ sys-kaleido bindle uninstall core
```

## Install

### Automated

- Linux/macOS (bash/zsh)

  for linux, 'gnu' is the default abi in [target triple](https://doc.rust-lang.org/cargo/commands/cargo-build.html), if you are not sure what to use, use the default.
  ```shell
  $ curl -sSL https://raw.githubusercontent.com/jinyuli/sys-kaleido/master/install.sh | bash
  ```
  the script accepts one argument `--abi`.

  to enable `sys-kaleido` command in current shell
  ```shell
  $ source "$HOME/.sys-kaleido/env"
  ```

- Windows (pwsh)

  ```pwsh
  $ iwr https://raw.githubusercontent.com/jinyuli/sys-kaleido/master/install.ps1 -useb | iex
  ```
  the script accepts two parameters `-abi`(by default it's 'msvc') and `-arch`(by default it's 'x86_64')

### Manual 

- Linux/MacOS
    - Create a directory for `sys-kaleido` (recommended: `~/.sys-kaleido`), download executable file from [releases](https://github.com/jinyuli/sys-kaleido/releases), and copy it to the `bin` subdirectory of the `sys-kaleido` directory (i.e. `~/.sys-kaleido/bin`), rename it to `sys-kaleido`.
    - Update system `PATH` to include `sys-kaleido` path.

- Windows
    - Create a directory for `sys-kaleido` (recommended: `~/.sys-kaleido`), download executable file from [releases](https://github.com/jinyuli/sys-kaleido/releases), and copy it to the `bin` subdirectory of the `sys-kaleido` directory (i.e. `~/.sys-kaleido/bin`), rename it to `sys-kaleido.exe`.
    - Update system `PATH` to include `sys-kaleido.exe` path.

## Features

* **search** all released versions.
* **list** all packages.
* **install** packages.
* **uninstall** installed packages.
* **update** installed packages.
* **config** update sys-kaleido configuration, it mainly contains all supported system tools.
* **upgrade** sys-kaleido to latest version.
* **version** show sys-kaleido current version.

## Usage

update **sys-kaleido** to latest version:
```shell
$ sys-kaleido update
```

list all packages:
```shell
$ sys-kaleido list
```

search a package:
```shell
$ sys-kaleido search du
```

install a package:
```shell
$ sys-kaleido install dust
```

list bindles:
```shell
$ sys-kaleido bindle list
```

list all packages in a bindle:
```shell
$ sys-kaleido bindle list core
```

install all package in a bindle:
```shell
$ sys-kaleido bindle install core
```

## Acknowledgement

supported tools:

* [dust](https://github.com/bootandy/dust)
* [lsd](https://github.com/lsd-rs/lsd)
* [fnc](https://github.com/supercilex/fuc)
* [bat](https://github.com/sharkdp/bat)

