use std::env;
use std::path::PathBuf;

/* ======================================================================== */
/* Pre-process shaders                                                      */

fn build_shaders() {

    let engine_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .expect("cannot canonicalize path");

    #[cfg(target_os = "linux")]
    let compile_path = engine_path.join("assets/shaders/compile.sh");
    #[cfg(target_os = "windows")]
    let compile_path = engine_path.join("assets/shaders/compile.bat");

    // only rebuild if we've added a shader
    println!("cargo:rerun-if-changed={}", compile_path.to_str().expect("Failed to get string representation of shader compiler path."));

    if !std::process::Command::new(compile_path)
        .output()
        .expect("could not spawn `ar`")
        .status
        .success()
    {
        // Panic if the command was not successful.
        panic!("could not emit library file");
    }
}

fn main() {
    build_shaders();
}
