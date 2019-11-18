# `felloe` – Interactive Helm Version Management


Interactive Helm version management - For cluster hopping and trying new versions helm and tiller before upgrading your production cluster.

![terminal](test/demo.svg)

## Installation

### Binary Release

1) Download a version from one of the [releases](https://github.com/ddlees/felloe/releases)
2) Unpack the archive
3) Move the `felloe` binary and/or add it your PATH

### Homebrew

``` sh
brew install felloe
```

### Chocolatey

``` sh
choco install felloe
```

### Crates.io

Install the package from crates.io:

``` sh
cargo install felloe

# Reinstall or upgrade:
cargo install felloe --force
```

### Build from source

Buiding from source:

``` console
git clone https://github.com/ddlees/felloe.git
cd felloe
cargo install --path .

# Reinstall or upgrade:
cargo install --path . --force
```

## Usage

``` man
felloe 0.1.0
⎈ The helm version manager

USAGE:
    felloe [FLAGS] [OPTIONS] [version] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -l, --latest     Show latest official helm version
    -V, --version    Prints version information

OPTIONS:
        --log-level <log-level>    

ARGS:
    <version>    

SUBCOMMANDS:
    exec         Execute command with modified PATH, so downloaded helm <version> first
    help         Prints this message or the help of the given subcommand(s)
    latest       Install the latest official helm release
    list         Output downloaded versions
    prune        Remove all downloaded versions except the currently installed version
    remove       Remove the given installed version(s)
    run          Execute downloaded helm <version> with [args ...]
    uninstall    Remove the installed helm
    versions     Output matching versions available for download
    which        Output path for downloaded helm <version>
```
