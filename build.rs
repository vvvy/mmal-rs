fn main() {

    let mmal_lib_dir = locate_dir("MMAL_LIB_DIR", "/opt/vc/lib", "MMAL libraries");

    // Tell cargo to tell rustc to link the system shared libraries.
    println!("cargo:rustc-link-lib=mmal_util");
    println!("cargo:rustc-link-lib=mmal_core");
    println!("cargo:rustc-link-lib=mmal_vc_client");
    println!("cargo:rustc-link-lib=vcos");
    println!("cargo:rustc-link-lib=bcm_host");
    println!("cargo:rustc-link-lib=vchiq_arm");
    println!("cargo:rustc-link-lib=vcsm");
    println!("cargo:rerun-if-env-changed=HOST");
    println!("cargo:rerun-if-env-changed=TARGET");
    println!("cargo:rerun-if-env-changed=MMAL_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=MMAL_LIB_DIR");
    println!("cargo:rustc-link-search=native={}", mmal_lib_dir);

    /*
    println!("=======================================================");
    for (v, val) in std::env::vars() {
        println!("{v}={val}");
    }
    println!("=======================================================");
    */
    generate_bindings();
}

#[cfg(not(feature = "generate_bindings"))]
pub fn generate_bindings() { }

#[cfg(feature = "generate_bindings")]
pub fn generate_bindings() { 

    //let host = std::env::var("HOST").unwrap();
    //let target = std::env::var("TARGET").unwrap();

    let mmal_lib_arg = "-I".to_owned() + 
        &locate_dir("MMAL_INCLUDE_DIR", "/opt/vc/include", "MMAL headers");
    
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let mut bindings = bindgen::Builder::default()

        .constified_enum_module(r"MMAL_STATUS_T|MMAL_PARAMETER_CAMERA_CONFIG_TIMESTAMP_MODE_T")

        // Prevent generating bindings including libc!
        .allowlist_type(r"MMAL_.*")
        .allowlist_function(r"(?:mmal_|vcos_|bcm_).*")
        .allowlist_var(r"MMAL_.*")

        .derive_debug(true)
        .impl_debug(true)
        .header("wrapper.h");

    // Include mmal library headers
    bindings = bindings.clang_arg(mmal_lib_arg);

    /*
    // work around mizzing bits/floatn.h when using zig
    bindings = bindings
        .clang_arg("-I/opt/zig-linux-x86_64-0.10.1/lib/libc/include/generic-glibc")
        .clang_arg("-I/opt/zig-linux-x86_64-0.10.1/lib/libc/glibc")
        .clang_arg("-I/opt/zig-linux-x86_64-0.10.1/lib/libc/include/arm-linux-gnueabihf");
    */

    /*
    if target == "armv7-unknown-linux-gnueabihf" && host != target {
        // We're cross-compiling
        bindings = bindings
            .clang_arg("-I/usr/lib/gcc-cross/arm-linux-gnueabihf/4.8/include-fixed/")
            .clang_arg("-I/usr/lib/gcc-cross/arm-linux-gnueabihf/4.8/include/")
            .clang_arg("-I/usr/arm-linux-gnueabihf/include/")
            .clang_arg("-nobuiltininc")
            .clang_arg("-nostdinc++");
    }
    */

    let bindings = bindings
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}


fn locate_dir(varname: &str, default_path: &str, descr: &str) -> String {
    let path = if let Ok(env_path) = std::env::var(varname) {
        env_path
    } else {
        default_path.to_owned()
    };
    if !std::path::Path::new(&path).exists() {
        panic!("Could not locate {} at `{}`\ndefault: {}\nenv {}: {:?}",
            descr, path, default_path, varname, std::env::var(varname)
        );
    }
    path
}

