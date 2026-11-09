//! A command-line utility to compress files using the brotli algorithm.

/*************************************************************************************************
 * Copyright © 2022-2026 Kristian Thy (<thy@42.dk>) and released under the MIT license.          *
 * This file is part of Furnel: <https://github.com/kthy/furnel>                                 *
 *************************************************************************************************/

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{env, fmt};

use brotli::enc::BrotliEncoderParams;
use clap::Parser;
use glob::{GlobError, Pattern, PatternError};

use crate::statics::{ABOUT, AFTER_HELP, GLOPTS, LICENSES};

mod statics;

/// Brotli quality level, 0-11.
const QUALITY: i32 = 11;
/// Base-2 logarithm of the brotli sliding window size.
const LG_WINDOW: i32 = 21;

/// Exit codes from BSD `sysexits.h`.
mod sysexits {
    /// The command was used incorrectly.
    pub const EX_USAGE: u8 = 64;
    /// An input file did not exist or was not readable.
    pub const EX_NOINPUT: u8 = 66;
    /// An error occurred while doing I/O on some file.
    pub const EX_IOERR: u8 = 74;
}

/// Command-line options.
#[derive(Debug, Parser)]
#[command(about = ABOUT, after_help = AFTER_HELP, term_width = 80, version)]
#[allow(clippy::struct_excessive_bools, reason = "CLI flags")]
struct Args {
    /// The base path to search
    #[arg(default_value = ".")]
    base_path: PathBuf,
    /// File extensions to process, for example `-x css -x html` or `-x css,html`
    #[arg(
        short = 'x',
        long = "extension",
        value_name = "EXTENSION",
        value_delimiter = ',',
        default_values = ["css", "html", "js", "svg", "txt"]
    )]
    extensions: Vec<String>,
    /// Display full license notice
    #[arg(short, long)]
    license: bool,
    /// Only compress missing files, i.e. those where no corresponding .br files are present
    #[arg(short = 'm', long)]
    only_missing: bool,
    /// Disable progress indicator
    #[arg(short, long)]
    quiet: bool,
    /// Recurse into subdirs below the base path
    #[arg(short, long)]
    recurse: bool,
}

/// Everything that can make a run fail.
#[derive(Debug)]
enum Error {
    /// The base path is not a readable directory.
    NotADirectory(PathBuf),
    /// The base path cannot be expressed as a glob pattern.
    NonUtf8Path(PathBuf),
    /// A file extension produced an invalid glob pattern.
    Pattern {
        pattern: String,
        source: PatternError,
    },
    /// A directory could not be traversed.
    Glob(GlobError),
    /// An input file could not be read.
    Read { path: PathBuf, source: io::Error },
    /// An output file could not be written.
    Write { path: PathBuf, source: io::Error },
    /// Reading the input or writing the output failed during compression.
    Compress {
        input: PathBuf,
        output: PathBuf,
        source: io::Error,
    },
    /// Progress output could not be written.
    Stdout(io::Error),
}

impl Error {
    const fn exit_code(&self) -> u8 {
        match self {
            Self::NotADirectory(_) | Self::NonUtf8Path(_) | Self::Pattern { .. } => {
                sysexits::EX_USAGE
            }
            Self::Glob(_) | Self::Read { .. } => sysexits::EX_NOINPUT,
            Self::Write { .. } | Self::Compress { .. } | Self::Stdout(_) => sysexits::EX_IOERR,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotADirectory(path) => {
                write!(f, "{} is not a directory I can read", path.display())?;
                if let Ok(cwd) = env::current_dir() {
                    write!(f, " from {}", cwd.display())?;
                }
                Ok(())
            }
            Self::NonUtf8Path(path) => write!(f, "{} is not valid UTF-8", path.display()),
            Self::Pattern { pattern, .. } => write!(f, "{pattern:?} is not a valid pattern"),
            Self::Glob(e) => write!(f, "error reading {}", e.path().display()),
            Self::Read { path, .. } => write!(f, "error reading {}", path.display()),
            Self::Write { path, .. } => write!(f, "error writing {}", path.display()),
            Self::Compress { input, output, .. } => write!(
                f,
                "error compressing {} into {}",
                input.display(),
                output.display()
            ),
            Self::Stdout(_) => write!(f, "error writing progress to stdout"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotADirectory(_) | Self::NonUtf8Path(_) => None,
            Self::Pattern { source, .. } => Some(source),
            Self::Glob(e) => Some(e.error()),
            Self::Read { source, .. }
            | Self::Write { source, .. }
            | Self::Compress { source, .. }
            | Self::Stdout(source) => Some(source),
        }
    }
}

fn main() -> ExitCode {
    let args = Args::parse();

    if args.license {
        println!("{LICENSES}");
        return ExitCode::SUCCESS;
    }

    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let mut message = e.to_string();
            let mut source = std::error::Error::source(&e);
            while let Some(cause) = source {
                message = format!("{message}: {cause}");
                source = cause.source();
            }
            eprintln!("furnel: {message}");
            ExitCode::from(e.exit_code())
        }
    }
}

fn run(args: &Args) -> Result<(), Error> {
    if !args.base_path.is_dir() {
        return Err(Error::NotADirectory(args.base_path.clone()));
    }
    let paths = candidates(args)?;
    bake_all(&paths, args.quiet)
}

/// Collect the files to compress, validating every pattern before touching any file.
fn candidates(args: &Args) -> Result<Vec<PathBuf>, Error> {
    let prefix = glob_prefix(&args.base_path, args.recurse)?;
    let patterns = args
        .extensions
        .iter()
        .map(|extension| {
            let pattern = format!("{prefix}{}", extension.trim_start_matches('.'));
            match glob::glob_with(&pattern, GLOPTS) {
                Ok(paths) => Ok(paths),
                Err(source) => Err(Error::Pattern { pattern, source }),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut paths = Vec::new();
    for entry in patterns.into_iter().flatten() {
        let path = entry.map_err(Error::Glob)?;
        // A directory can match too, e.g. one named `assets.js`.
        if path.is_file() && (!args.only_missing || !append_br(&path).exists()) {
            paths.push(path);
        }
    }
    // The same file can match several extensions, e.g. `-x css -x CSS`.
    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// Compress every file in `paths`, stopping at the first error.
fn bake_all(paths: &[PathBuf], quiet: bool) -> Result<(), Error> {
    let mut stdout = io::stdout().lock();
    let mut progress = |text: &str| -> Result<(), Error> {
        if quiet {
            return Ok(());
        }
        stdout
            .write_all(text.as_bytes())
            .and_then(|()| stdout.flush())
            .map_err(Error::Stdout)
    };

    for path in paths {
        progress(".")?;
        if let Err(e) = bake_file(path) {
            // Best effort: end the line of dots so the error starts on its own line.
            let _ = progress("\n");
            return Err(e);
        }
    }
    progress(" done!\n")
}

/// Compress `path` into `path.br`, going via a temporary file so a failed or interrupted run
/// never leaves a truncated `.br` behind.
fn bake_file(path: &Path) -> Result<(), Error> {
    let target = append_br(path);
    let tmp = append_suffix(&target, ".tmp");
    let result = compress(path, &tmp).and_then(|()| {
        fs::rename(&tmp, &target).map_err(|source| Error::Write {
            path: target.clone(),
            source,
        })
    });
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

/// Stream `input` through the brotli encoder into a newly created `output`.
fn compress(input: &Path, output: &Path) -> Result<(), Error> {
    let read_err = |source| Error::Read {
        path: input.to_path_buf(),
        source,
    };
    let write_err = |source| Error::Write {
        path: output.to_path_buf(),
        source,
    };

    let mut reader = File::open(input).map_err(read_err)?;
    let mut writer = BufWriter::new(File::create(output).map_err(write_err)?);
    let params = BrotliEncoderParams {
        quality: QUALITY,
        lgwin: LG_WINDOW,
        ..BrotliEncoderParams::default()
    };
    // BrotliCompress does not say whether a failure came from the reader or the writer.
    brotli::BrotliCompress(&mut reader, &mut writer, &params).map_err(|source| {
        Error::Compress {
            input: input.to_path_buf(),
            output: output.to_path_buf(),
            source,
        }
    })?;
    let file = writer.into_inner().map_err(|e| write_err(e.into_error()))?;
    file.sync_all().map_err(write_err)
}

fn append_br(path: &Path) -> PathBuf {
    append_suffix(path, ".br")
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut os_string: OsString = path.into();
    os_string.push(suffix);
    os_string.into()
}

/// Build the glob pattern prefix that an extension is appended to.
fn glob_prefix(base_path: &Path, recurse: bool) -> Result<String, Error> {
    let base = base_path
        .to_str()
        .ok_or_else(|| Error::NonUtf8Path(base_path.to_path_buf()))?;
    let separator = if base.ends_with(std::path::is_separator) {
        ""
    } else {
        "/"
    };
    let subdirs = if recurse { "**/" } else { "" };
    Ok(format!("{}{separator}{subdirs}*.", Pattern::escape(base)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_br() {
        assert_eq!(append_br(Path::new("")), Path::new(".br"));
        assert_eq!(append_br(Path::new("test")), Path::new("test.br"));
        assert_eq!(append_br(Path::new("test.ext")), Path::new("test.ext.br"));
    }

    #[test]
    fn test_glob_prefix() {
        let prefix = |p: &str, r| glob_prefix(Path::new(p), r).unwrap();
        assert_eq!(prefix("foo", false), "foo/*.");
        assert_eq!(prefix("foo/", false), "foo/*.");
        assert_eq!(prefix("foo", true), "foo/**/*.");
        assert_eq!(prefix(".", false), "./*.");
        assert_eq!(prefix("./", false), "./*.");
        assert_eq!(prefix("/", false), "/*.");
        assert_eq!(prefix("sub[1]", false), "sub[[]1[]]/*.");
    }

    #[test]
    fn test_defaults() {
        let mut args = Args::try_parse_from(["furnel"]).unwrap();
        assert_eq!(args.base_path, Path::new("."));
        assert_eq!(args.extensions, ["css", "html", "js", "svg", "txt"]);

        args = Args::try_parse_from(["furnel", "-x", "json", "-x", "xml"]).unwrap();
        assert_eq!(args.extensions, ["json", "xml"]);

        args = Args::try_parse_from(["furnel", "-x", "json,xml"]).unwrap();
        assert_eq!(args.extensions, ["json", "xml"]);
    }
}
