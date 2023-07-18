#![feature(concat_bytes)]

use chrono::Local;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let memory_x = if env::var("CARGO_FEATURE_BOOTLOADER").is_ok() {
        include_bytes!("memory.bootloader.x").as_slice()
    } else if env::var("CARGO_FEATURE_M2").is_ok() {
        include_bytes!("memory.2m.x").as_slice()
    } else if env::var("CARGO_FEATURE_BINARYINFO").is_ok() {
        concat_bytes!(
            include_bytes!("memory.16m.x"),
            include_bytes!("memory.binaryinfo.x")
        )
        .as_slice()
    } else {
        include_bytes!("memory.16m.x").as_slice()
    };

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(memory_x)
        .unwrap();
    let mut build_date = File::create(out.join("build_date.txt")).unwrap();
    write!(build_date, r#""{}""#, Local::now().date_naive()).ok();
    let mut build_attribute = File::create(out.join("build_attribute.txt")).unwrap();
    write!(build_attribute, r#""{}""#, env::var("PROFILE").unwrap()).ok();

    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=memory.bootloader.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    if env::var("CARGO_FEATURE_BOOTLOADER").is_err() {
        println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
    }
    if env::var("CARGO_FEATURE_PROBE").is_ok() {
        println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    }
}
