extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    let bindings;

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=c++");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");

        cc::Build::new()
            .include("vendor/Mac/Include")
            .file("src/lib.cpp")
            .file("vendor/Mac/Include/BlackmagicRawAPIDispatch.cpp")
            .compile("braw");

        let sdk_root = "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk";

        bindings = bindgen::Builder::default()
            .clang_arg("-x")
            .clang_arg("objective-c++")
            .clang_arg(format!("-isysroot{}", sdk_root))
            .clang_arg("-Ivendor/Mac/Include")
    }

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=stdc++");

        cc::Build::new()
            .include("vendor/Linux/Include")
            .file("src/lib.cpp")
            .file("vendor/Linux/Include/BlackmagicRawAPIDispatch.cpp")
            .compile("braw");

        bindings = bindgen::Builder::default()
            .clang_arg("-Ivendor/Linux/Include")
    }

    let bindings = bindings.header("src/lib.hpp")
        .whitelist_function(".*blackmagic_raw.+")
        .whitelist_type("_BlackmagicRaw.+")
        .generate()
        .expect("unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("unable to write bindings");
}
