# Changelog

<!-- markdownlint-disable-file MD024 -->

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

…

## [0.1.5] - 2026-11-09

### Added

* Man page in the `.deb` package and the Linux release archives.
* `-x` accepts comma-separated lists (`-x css,html`) and extensions with a leading dot (`-x .css`).

### Changed

* Output is written to `<file>.br.tmp` and renamed to `<file>.br` when complete, so an interrupted
  run never leaves a truncated `.br` file that `--only-missing` would then skip.
* Errors print a one-line message and exit with a sysexits code (64 usage, 66 input, 74 output)
  instead of panicking. Furnel stops at the first error.
* All extension patterns are validated before any file is compressed.
* Files are streamed through the encoder instead of being read into memory whole.

### Fixed

* `furnel ./` searched the filesystem root instead of the current directory.
* Base paths containing glob characters such as `[` or `*` matched nothing.
* Write errors on the final brotli block were silently ignored.
* Progress dots were buffered until the end of the run instead of appearing per file.

## [0.1.4] - 2026-10-07

### Changed

* Releases are published to crates.io automatically via trusted publishing when a GitHub release is published.
* Repository moved to <https://github.com/kthy/furnel>.
* Contact addresses moved from the defunct `pyxy.dk` domain to `geologik.dk`.

### Fixed

* `--help` output is wrapped at 80 columns as intended.

## [0.1.3] - 2026-10-07

### Added

* `-V`/`--version` flag.

### Changed

* Minimum supported Rust version raised from 1.56.1 to 1.85, as required by current `clap` and `brotli`.
* Updated dependencies: `brotli` 7.0.0 → 9.0.0, `clap` 4.5.47 → 4.6.7, `glob` 0.3.3 → 0.3.4.
* CI: GitHub Actions pinned to SHA refs, Dependabot and zizmor analysis added.

### Fixed

* `BASE_PATH` is now optional and defaults to `.` as documented; running `furnel` with no arguments no longer errors.
* `--help` shows the default extensions again.
* Usage section in `README.md` now matches `--help`.
* Release workflow now builds and uploads archives for all targets.

## [0.1.2] - 2025-09-09

### Fixed

* Fixed clippy warnings related to MSRV compatibility by replacing `default_values_t` with manual default handling.

## [0.1.1] - 2025-04-10

* Updated dependencies and copyright notices.

### Added

* Release archives now contain `CHANGELOG.md`.

### Removed

* Spurious `doc` directory in release archives.

## [0.1.0] - 2022-01-25

Initial public release of **furnel**.

[Unreleased]: https://github.com/kthy/furnel/compare/v0.1.5...HEAD
[0.1.5]: https://github.com/kthy/furnel/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/kthy/furnel/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/kthy/furnel/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/kthy/furnel/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/kthy/furnel/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/kthy/furnel/releases/tag/v0.1.0
