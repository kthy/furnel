# Furnel

A command-line utility to compress files using the [brotli] algorithm. Built because I wanted a
simple tool to pre-compress files for a static website.

[![Crates.io][badge-crates-svg]][badge-crates-href]
[![Deps.rs][badge-depsrs-svg]][badge-depsrs-href]
[![Continuous Integration workflow status badge][badge-ci-svg]][badge-ci-href]
[![Security Audit workflow status badge][badge-audit-svg]][badge-audit-href]
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Why “Furnel”?

The brotli algorithm is named for *Brötli*, which is the Swiss German word for bread rolls. Bread
rolls are baked in an oven, and an oven in Romansh - another Swiss language - is called *furnel*.

![Furnel logo][og-image-url]

**Furnel** makes **brotli**.

## Installation

### From GitHub

1. Go to the [Furnel releases page][releases].
1. Download the version you need for the OS you have.
1. Unpack the self-contained executable somewhere on your path.

### Using `cargo`

If you are a Rust developer, you can install the binary from [Crates.io][badge-crates-href]:

```bash
cargo install furnel
```

## Usage

```text
Usage: furnel [OPTIONS] [BASE_PATH]

Arguments:
  [BASE_PATH]  The base path to search [default: .]

Options:
  -x, --extension <EXTENSION>  File extensions to process, for example `-x css
                               -x html` or `-x css,html` [default: css html js
                               svg txt]
  -l, --license                Display full license notice
  -m, --only-missing           Only compress missing files, i.e. those where no
                               corresponding .br files are present
  -q, --quiet                  Disable progress indicator
  -r, --recurse                Recurse into subdirs below the base path
  -h, --help                   Print help
  -V, --version                Print version
```

Each file is compressed into a temporary `<file>.br.tmp` and renamed to `<file>.br` once complete,
so an interrupted run never leaves a truncated `.br` file behind. Furnel stops at the first error.

### Exit status

| Code | Meaning                                                       |
|------|---------------------------------------------------------------|
| 0    | Success                                                       |
| 64   | Usage error: base path is not a directory, or invalid pattern |
| 66   | An input file or directory could not be read                  |
| 74   | An output file could not be written                           |

## Building

Furnel is written in Rust, so you will need a Rust installation to compile it.

### Using GitHub Codespaces or Dev Containers

The easiest way to get started is using the provided dev container configuration:

1. **GitHub Codespaces**: Click the "Code" button and select "Create codespace on main"
2. **VS Code Dev Containers**: Open the project in VS Code and select "Reopen in Container" when prompted
3. **Other editors**: Use any editor that supports dev containers with the `.devcontainer/devcontainer.json` configuration

The dev container includes:
- Latest Rust toolchain
- Rust Analyzer for IDE support
- Clippy for linting
- All necessary VS Code extensions pre-configured

### Installing Rust Locally

#### On Windows
Download and run the [`rustup-init.exe`][rustup] installer. As part of the installation process,
it will instruct you to install the Microsoft Visual C++ Build Tools 2019 and provide a link.

#### On macOS/Linux
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build, Test and Run

```text
git clone https://github.com/kthy/furnel.git
cd furnel
cargo build
cargo test
cargo run -- -h
```

### Debian Package

The `.deb` package includes a man page generated from the `--help` output with [help2man]. To
build the package locally you need `help2man` and [`cargo-deb`][cargo-deb]:

```text
cargo build --release
doc/mkman.sh
cargo deb --no-build
```

[badge-audit-href]: https://github.com/kthy/furnel/actions/workflows/audit-dep.yml
[badge-audit-svg]: https://github.com/kthy/furnel/actions/workflows/audit-dep.yml/badge.svg
[badge-ci-href]: https://github.com/kthy/furnel/actions/workflows/ci.yml
[badge-ci-svg]: https://github.com/kthy/furnel/actions/workflows/ci.yml/badge.svg
[badge-crates-href]: https://crates.io/crates/furnel
[badge-crates-svg]: https://img.shields.io/crates/v/furnel.svg
[badge-depsrs-href]: https://deps.rs/crate/furnel
[badge-depsrs-svg]: https://deps.rs/repo/github/kthy/furnel/status.svg
[brotli]: https://en.wikipedia.org/wiki/Brotli
[cargo-deb]: https://github.com/kornelski/cargo-deb
[help2man]: https://www.gnu.org/software/help2man/
[og-image-url]: https://repository-images.githubusercontent.com/451275347/f342ccad-8e6c-4815-be3e-2375f970694b
[releases]: https://github.com/kthy/furnel/releases
[rustup]: https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe
