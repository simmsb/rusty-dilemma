#![feature(concat_bytes)]

use chrono::Local;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let memory_x = if env::var("CARGO_FEATURE_M2").is_ok() {
        if env::var("CARGO_FEATURE_BOOTLOADER").is_ok() {
            include_bytes!("memory.2m.x").as_slice()
        } else {
            include_bytes!("memory.2m.nobl.x").as_slice()
        }
    } else {
        if env::var("CARGO_FEATURE_BOOTLOADER").is_ok() {
            include_bytes!("memory.16m.x").as_slice()
        } else {
            include_bytes!("memory.16m.nobl.x").as_slice()
        }
    };

    let memory_x_extra = if env::var("CARGO_FEATURE_BINARYINFO").is_ok() {
        if env::var("CARGO_FEATURE_BOOTLOADER").is_ok() {
            panic!("binaryinfo won't be visible if the bootloader is enabled, use the binaryinfo feature of the bootloader")
        }

        include_bytes!("memory.binaryinfo.x").as_slice()
    } else {
        &[]
    };

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    {
        let mut memory_x_f = File::create(out.join("memory.x")).unwrap();

        memory_x_f.write_all(memory_x).unwrap();
        memory_x_f.write_all(memory_x_extra).unwrap();
    }
    File::create(out.join("build_date.txt"))
        .unwrap()
        .write_all(Local::now().date_naive().to_string().as_bytes())
        .unwrap();
    File::create(out.join("build_attribute.txt"))
        .unwrap()
        .write_all(env::var("PROFILE").unwrap().as_bytes())
        .unwrap();

    #[cfg(feature = "display-slint")]
    {
        let path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "pico_st7789",
            "board_config.toml",
        ]
        .iter()
        .collect();
        println!("cargo:BOARD_CONFIG_PATH={}", path.display());
        println!("cargo:EMBED_TEXTURES=1");

        let config = slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
        slint_build::compile_with_config("ui/main.slint", config).unwrap();
        slint_build::print_rustc_flags().unwrap();
    }

    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    if env::var("CARGO_FEATURE_BOOTLOADER").is_err() {
        println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
    }
    if env::var("CARGO_FEATURE_PROBE").is_ok() {
        println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
    }
}
