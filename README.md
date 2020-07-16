[![CI Status](https://github.com/ddlees/felloe/workflows/Continuous%20integration/badge.svg)](https://github.com/ddlees/felloe/actions)
[![Audit Status](https://github.com/ddlees/felloe/workflows/Audit/badge.svg)](https://github.com/ddlees/felloe/actions)
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2Fddlees%2Ffelloe.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2Fddlees%2Ffelloe?ref=badge_shield)

# `felloe` – Interactive Helm Version Management

Interactive Helm version management - For cluster hopping and trying new versions of helm and tiller before upgrading your production cluster.

![terminal](images/demo.svg)

## Installation

### Binary Release

1) Download a version from one of the [releases](https://github.com/ddlees/felloe/releases)
2) Unpack the archive
3) Move the `felloe` binary and/or add it your PATH
4) `chmod a+x path/to/felloe`

### [Homebrew](https://brew.sh)

``` console
brew tap ddlees/felloe
brew install felloe
```

### [Chocolatey](https://chocolatey.org) for Windows

``` console
choco install felloe
```

### Crates.io

Install the package from crates.io:

``` console
cargo install felloe

# Reinstall or upgrade:
cargo install felloe --force
```

### Build from source

Run the following to build from source:

``` console
git clone https://github.com/ddlees/felloe.git
cd felloe
cargo install --path .

# Reinstall or upgrade:
cargo install --path . --force
```

## Completions

#### bash

``` shell
$ felloe completions bash >> ~/.bash_profile # macos
$ felloe completions bash >> ~/.bashrc # linux
```

#### zsh

``` shell
$ felloe completions zsh > /usr/local/share/zsh/site-functions/_felloe
```

#### fish

``` shell
$ mkdir -p ~/.config/fish/completions # (optional)
$ felloe completions fish > ~/.config/fish/completions/felloe.fish
```

#### powershell

The powershell completion scripts require PowerShell v5.0+ (which comes with Windows 10, but can be downloaded separately for windows 7 or 8.1).

``` powershell
# Check for profile
C:\> Test-Path $profile

# Create profile if none exists
C:\> New-Item -path $profile -type file -force

# Append completions to profile
C:\> felloe completions powershell >>
${env:USERPROFILE}\Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1
```

#### elvish

``` shell
# Create completions plugin
~> mkdir -p ~/.elvish/lib/completions
~> felloe completions elvish > ~/.elvish/lib/completions/felloe.elv

# Update rc.elv
~> echo "use completions/felloe" >> ~/.elvish/rc.elv
```

## Usage

`felloe --help`

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
    completions    Generate completions for desired shell
    exec           Execute command with modified PATH, so downloaded helm <version> first
    help           Prints this message or the help of the given subcommand(s)
    latest         Install the latest official helm release
    list           Output downloaded versions
    prune          Remove all downloaded versions except the currently installed version
    remove         Remove the given installed version(s)
    run            Execute downloaded helm <version> with [args ...]
    uninstall      Remove the installed helm
    versions       Output matching versions available for download
    which          Output path for downloaded helm <version>
```

## License

This work is [dual-licensed](LICENSE) under Apache 2.0 and MIT to help avoid problems using this software or its libraries with GPL2.

`SPDX-License-Identifier: Apache-2.0 OR MIT`



[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2Fddlees%2Ffelloe.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2Fddlees%2Ffelloe?ref=badge_large)

## Attributions

This project was inspired by frequent cluster hopping, having to switch between versions of helm and the following projects:

- [pyenv](https://github.com/pyenv/pyenv) - Python version manager
- [n](https://github.com/tj/n) - Node version manager
- [nvm](https://github.com/nvm-sh/nvm) - Node version manager