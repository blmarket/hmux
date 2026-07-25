use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=LIBGHOSTTY_VT_LIB_DIR");
    println!("cargo:rerun-if-env-changed=LIBGHOSTTY_VT_DIR");

    let lib_dir = resolve_lib_dir();
    let archive = lib_dir.join("libghostty-vt.a");
    assert!(
        archive.exists(),
        "libghostty-vt.a not found in {}.",
        lib_dir.display()
    );

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=ghostty-vt");
    // The parent crate's Linux platform uses forkpty(3), which lives in libutil.
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=util");
    }
}

/// Resolve the directory that holds `libghostty-vt.a`, rebuilding source
/// checkouts whenever Cargo reruns this build script.
fn resolve_lib_dir() -> PathBuf {
    if let Some(dir) = env::var_os("LIBGHOSTTY_VT_LIB_DIR") {
        let lib_dir = PathBuf::from(dir);
        println!(
            "cargo:rerun-if-changed={}",
            lib_dir.join("libghostty-vt.a").display()
        );
        return lib_dir;
    }

    let src_dir = env::var_os("LIBGHOSTTY_VT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("vendor/libghostty-vt")
        });
    register_source_inputs(&src_dir);
    build_from_source(&src_dir);

    src_dir.join("zig-out/lib")
}

/// Build the static VT lib from the vendored source via `zig build`.
///
/// A failure here (zig missing, compile error) is surfaced loudly: otherwise the
/// missing archive only shows up later as an opaque `could not find native
/// static library ghostty-vt` linker error.
fn build_from_source(src_dir: &Path) {
    assert!(
        src_dir.join("build.zig").exists(),
        "libghostty-vt source checkout not found in {}",
        src_dir.display()
    );
    let zig = env::var("ZIG").unwrap_or_else(|_| "zig".into());
    let output = Command::new(&zig)
        .current_dir(src_dir)
        .args([
            "build",
            "-Demit-lib-vt",
            "-Doptimize=ReleaseFast",
            "-Dsimd=true",
            "-Demit-xcframework=false",
        ])
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "failed to run `{zig} build` in {}: {e}. Install zig (or set ZIG \
                 to its path).",
                src_dir.display()
            )
        });
    assert!(
        output.status.success(),
        "`{zig} build` failed in {} ({}).\n--- stdout ---\n{}\n--- stderr ---\n{}",
        src_dir.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn register_source_inputs(src_dir: &Path) {
    for relative in ["build.zig", "build.zig.zon", "VERSION", "src", "include"] {
        let path = src_dir.join(relative);
        if path.exists() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
