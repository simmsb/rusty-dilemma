use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let memory_x = if env::var("CARGO_FEATURE_PROBE").is_err() {
        include_bytes!("memory.bootloader.x").as_slice()
    } else {
        include_bytes!("memory.x").as_slice()
    };

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(memory_x)
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=memory.bootloader.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    if env::var("CARGO_FEATURE_PROBE").is_ok() {
        println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
        println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    }
}
