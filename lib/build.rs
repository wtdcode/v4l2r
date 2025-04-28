use std::env::{self, VarError};
use std::path::{Path, PathBuf};

include!("bindgen.rs");

/// Environment variable that can be set to point to the directory containing the `videodev2.h`
/// file to use to generate the bindings.
const V4L2R_VIDEODEV_ENV: &str = "V4L2R_VIDEODEV2_H_PATH";

/// Default header file to parse if the `V4L2R_VIDEODEV2_H_PATH` environment variable is not set.
const DEFAULT_VIDEODEV2_H_PATH: &str = "/usr/include/linux";

/// Wrapper file to use as input of bindgen.
const WRAPPER_H: &str = "v4l2r_wrapper.h";

// Fix for https://github.com/rust-lang/rust-bindgen/issues/753
const FIX753_H: &str = "fix753.h";

fn print_cc(cc: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new(cc.to_str().expect("utf-8?"))
        .args(args)
        .output()
        .expect("fail to run cc");
    if !out.status.success() {
        panic!(
            "sysroot not found with {} with {:?}",
            String::from_utf8_lossy(&out.stderr),
            args
        );
    }
    String::from_utf8(out.stdout).expect("non utf-8?!")
}

fn main() {
    let videodev2_h_path = env::var(V4L2R_VIDEODEV_ENV)
        .or_else(|e| {
            if let VarError::NotPresent = e {
                Ok(DEFAULT_VIDEODEV2_H_PATH.to_string())
            } else {
                Err(e)
            }
        })
        .expect("invalid `V4L2R_VIDEODEV2_H_PATH` environment variable");

    let videodev2_h = PathBuf::from(videodev2_h_path.clone()).join("videodev2.h");

    println!("cargo::rerun-if-env-changed={}", V4L2R_VIDEODEV_ENV);
    println!("cargo::rerun-if-changed={}", videodev2_h.display());
    println!("cargo::rerun-if-changed={}", FIX753_H);
    println!("cargo::rerun-if-changed={}", WRAPPER_H);

    let cc = cc::Build::new().get_compiler();

    let cc_sysroot = print_cc(cc.path(), &["--print-sysroot"]).trim().to_string();

    let clang_args = [
        format!("-I{}", videodev2_h_path),
        format!("--sysroot={}", &cc_sysroot),
        format!("-I{}/usr/include/linux", &cc_sysroot),
    ];

    let bindings = v4l2r_bindgen_builder(bindgen::Builder::default())
        .header(WRAPPER_H)
        .clang_args(clang_args)
        .generate()
        .expect("unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("`OUT_DIR` is not set"));
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
