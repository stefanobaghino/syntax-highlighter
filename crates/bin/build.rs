//! Pre-compile every bundled grammar to `.pegb` so the `demo` binary
//! can `include_bytes!` it from `OUT_DIR`. Skips the parse+compile
//! round at every demo startup (~3 ms steady-state vs. ~12–17 ms on
//! the larger grammars) and lets the linker drop `pegc` from `demo`'s
//! image entirely. `pegc` and `pegdb` still depend on the compiler at
//! runtime; only `demo` benefits from the AOT path.

use std::{env, fs, path::PathBuf};

use syntax_highlighter::pegb;
use syntax_highlighter_compiler::pegc;

fn main() {
    let grammars_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("..")
        .join("..")
        .join("grammars");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", grammars_dir.display());

    for entry in fs::read_dir(&grammars_dir).expect("read grammars dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|s| s.to_str()) != Some("peg") {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());
        let src =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
        let program =
            pegc::compile(&src).unwrap_or_else(|e| panic!("compile {}: {}", path.display(), e));
        let bytes = pegb::encode(&program);
        let name = path.file_stem().unwrap().to_str().unwrap();
        let target = out_dir.join(format!("{name}.pegb"));
        fs::write(&target, bytes).unwrap_or_else(|e| panic!("write {}: {}", target.display(), e));
    }
}
