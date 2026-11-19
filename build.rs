//! Embeds every locale file under `resources/` (Datafaker's `src/main/resources/**/*.yml`) into the crate, so nothing
//! is read from disk at run time.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn collect(dir: &Path, root: &Path, out: &mut Vec<(String, PathBuf)>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir).expect("resources/").map(|e| e.expect("entry").path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect(&path, root, out);
        } else if path.extension().is_some_and(|e| e == "yml") {
            let rel = path.strip_prefix(root).expect("prefix").to_string_lossy().replace('\\', "/");
            out.push((rel, path));
        }
    }
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir")).join("resources");
    println!("cargo:rerun-if-changed=resources");
    let mut files = Vec::new();
    collect(&root, &root, &mut files);
    let mut src = String::from("/// Datafaker's locale files: path relative to its `src/main/resources`, and content.\n");
    src.push_str("pub static FILES: &[(&str, &str)] = &[\n");
    for (rel, path) in &files {
        writeln!(src, "    ({rel:?}, include_str!({:?})),", path.to_string_lossy()).expect("write");
    }
    src.push_str("];\n");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("out dir")).join("resources.rs");
    std::fs::write(out, src).expect("write resources.rs");
}
