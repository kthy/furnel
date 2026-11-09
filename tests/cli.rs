//! End-to-end tests running the furnel binary against scratch directories.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A scratch directory removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("furnel-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn file(&self, rel: &str, contents: &str) -> PathBuf {
        let path = self.0.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, contents).unwrap();
        path
    }

    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(&self.0)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        names
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn furnel(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_furnel"))
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}

fn decompress(path: &Path) -> String {
    let mut out = String::new();
    brotli::Decompressor::new(fs::File::open(path).unwrap(), 4096)
        .read_to_string(&mut out)
        .unwrap();
    out
}

#[test]
fn compresses_and_round_trips() {
    let s = Scratch::new("roundtrip");
    let body = "body { color: red; }\n".repeat(1000);
    s.file("a.css", &body);
    s.file("b.json", "{}");

    let out = furnel(&s.0, &["-q", "."]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
    assert_eq!(s.names(), ["a.css", "a.css.br", "b.json"]);
    assert_eq!(decompress(&s.0.join("a.css.br")), body);
}

#[test]
fn progress_output() {
    let s = Scratch::new("progress");
    s.file("a.css", "a");
    s.file("b.js", "b");

    let out = furnel(&s.0, &[]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), ".. done!\n");
}

#[test]
fn dot_slash_base_path() {
    let s = Scratch::new("dotslash");
    s.file("a.css", "a");

    let out = furnel(&s.0, &["-q", "./"]);
    assert!(out.status.success(), "{out:?}");
    assert!(s.0.join("a.css.br").exists());
}

#[test]
fn base_path_with_glob_characters() {
    let s = Scratch::new("brackets");
    s.file("sub[1]/a.css", "a");

    let out = furnel(&s.0, &["-q", "sub[1]"]);
    assert!(out.status.success(), "{out:?}");
    assert!(s.0.join("sub[1]/a.css.br").exists());
}

#[test]
fn recurse() {
    let s = Scratch::new("recurse");
    s.file("a.css", "a");
    s.file("x/y/b.css", "b");

    assert!(furnel(&s.0, &["-q", "."]).status.success());
    assert!(!s.0.join("x/y/b.css.br").exists());

    assert!(furnel(&s.0, &["-q", "-r", "."]).status.success());
    assert!(s.0.join("x/y/b.css.br").exists());
}

#[test]
fn extension_forms() {
    let s = Scratch::new("extensions");
    s.file("a.css", "a");
    s.file("b.html", "b");
    s.file("c.JSON", "c");

    assert!(furnel(&s.0, &["-q", "-x", ".css,html", "-x", "json"])
        .status
        .success());
    assert_eq!(
        s.names(),
        [
            "a.css",
            "a.css.br",
            "b.html",
            "b.html.br",
            "c.JSON",
            "c.JSON.br"
        ]
    );
}

#[test]
fn only_missing_keeps_existing() {
    let s = Scratch::new("missing");
    s.file("a.css", "a");
    s.file("b.css", "b");
    s.file("a.css.br", "stale");

    assert!(furnel(&s.0, &["-q", "-m"]).status.success());
    assert_eq!(fs::read_to_string(s.0.join("a.css.br")).unwrap(), "stale");
    assert_eq!(decompress(&s.0.join("b.css.br")), "b");
}

#[test]
fn not_a_directory() {
    let s = Scratch::new("notadir");
    s.file("a.css", "a");

    let out = furnel(&s.0, &["a.css"]);
    assert_eq!(out.status.code(), Some(64));
    assert!(String::from_utf8_lossy(&out.stderr).contains("is not a directory"));
}

#[test]
fn bad_pattern_compresses_nothing() {
    let s = Scratch::new("badpattern");
    s.file("a.css", "a");

    let out = furnel(&s.0, &["-x", "css", "-x", "cs["]);
    assert_eq!(out.status.code(), Some(64));
    assert!(String::from_utf8_lossy(&out.stderr).contains("\"./*.cs[\""));
    assert_eq!(s.names(), ["a.css"]);
}

#[cfg(unix)]
#[test]
fn unreadable_file_stops_without_leftovers() {
    use std::os::unix::fs::PermissionsExt;

    let s = Scratch::new("unreadable");
    let a = s.file("a.css", "a");
    s.file("b.css", "b");
    fs::set_permissions(&a, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read(&a).is_ok() {
        // Running as root; permissions are not enforced.
        return;
    }

    let out = furnel(&s.0, &[]);
    assert_eq!(out.status.code(), Some(66), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("error reading"));
    // Stops at the first error: b.css is not compressed, and no temporary file is left.
    assert_eq!(s.names(), ["a.css", "b.css"]);
}
