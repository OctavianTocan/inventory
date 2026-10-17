//! Links libvips. By default that is the build sharp ships, fetched into `rust/libvips/<platform>/` by
//! `rust/fetch-libvips.sh`, so images decode and encode as in the JS server whatever the machine has installed.
//! `MYMIND_VIPS_DIR` names another directory holding that `libvips-cpp` library, or `system` for the system libvips.

use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=MYMIND_VIPS_DIR");
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let dir = match std::env::var("MYMIND_VIPS_DIR") {
        Ok(value) if value == "system" => {
            for name in ["vips", "gobject-2.0", "glib-2.0"] {
                println!("cargo:rustc-link-lib={name}");
            }
            return;
        }
        Ok(value) => PathBuf::from(value),
        Err(_) => {
            let platform = format!("{}-{}", if os == "macos" { "darwin" } else { &os }, if arch == "x86_64" { "x64" } else if arch == "aarch64" { "arm64" } else { &arch });
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libvips").join(platform)
        }
    };
    println!("cargo:rerun-if-changed={}", dir.display());
    let library = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .find(|name| name.starts_with("libvips-cpp."));
    let (Some(library), Ok(dir)) = (library, dir.canonicalize()) else {
        panic!("no libvips-cpp library in {}: run rust/fetch-libvips.sh, or set MYMIND_VIPS_DIR=system", dir.display());
    };
    println!("cargo:rustc-link-search=native={}", dir.display());
    if os == "macos" {
        println!("cargo:rustc-link-lib=dylib={}", library.trim_start_matches("lib").trim_end_matches(".dylib"));
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
    } else {
        println!("cargo:rustc-link-lib=dylib:+verbatim={library}");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    }
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dir.display());
}
