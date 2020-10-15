extern crate bindgen;

use bindgen::EnumVariation;
use std::env;
use std::path::PathBuf;

use std::fs::File;
use std::io::Write;

fn select_lib_dir(base: &PathBuf) -> PathBuf {
    base.join("cortex-m4").join("fpv4-sp-d16-hard")
}

fn main() {
    // examples

    if env::var_os("CARGO_FEATURE_STM32F4XX").is_some() {
        // Put `memory.x` in our output directory and ensure it's
        // on the linker search path.
        let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
        File::create(out.join("memory.x"))
            .unwrap()
            .write_all(include_bytes!("memory.x"))
            .unwrap();
        println!("cargo:rustc-link-search={}", out.display());

        // By default, Cargo will re-run a build script whenever
        // any file in the project changes. By specifying `memory.x`
        // here, we ensure the build script is only re-run when
        // `memory.x` is changed.
        println!("cargo:rerun-if-changed=memory.x");
    }

    // bindgen

    let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let base_search_dir = project_dir.join("BSEC-Arduino-library").join("src");

    let search_dir = select_lib_dir(&base_search_dir);

    println!(
        "cargo:rustc-link-search={}",
        search_dir.as_path().to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=static=algobsec");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/wrapper.h")
        .clang_arg("-I./BSEC-Arduino-library/src")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // we are no_std
        .use_core()
        // use cty for ctypes
        .ctypes_prefix("cty")
        // whitelist only bsec types
        .whitelist_function("bsec_.*")
        .whitelist_type("bsec_.*")
        .whitelist_var("bsesc_.*")
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: false,
        })
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
