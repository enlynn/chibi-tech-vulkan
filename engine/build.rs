use std::env;
use std::path::PathBuf;
use std::fs;

/* ======================================================================== */
/* Helpers                                                                  */

// on linux, search for system library
#[cfg(target_os = "linux")]
fn get_pkgconfig_include_path_cmds(lib_name: &str) -> Option<Vec<String>>
{
    let lib = pkg_config::probe_library(lib_name).ok()?;

    let mut str_paths = Vec::<String>::with_capacity(lib.include_paths.len());
    for path in lib.include_paths {
        if let Some(valid_path) = path.to_str() {
            let mut cmd = String::with_capacity(valid_path.len() + 2);
            cmd.push_str("-I");
            cmd.push_str(valid_path);
            str_paths.push(cmd);
        }
    }
    Some(str_paths)
}

fn gen_bindings(
    lib_name: &str,
    wrapper_file: &str,
    user_includes: Vec<&str>,
    allowlist_function: Vec<&str>,
    allowlist_type: Vec<&str>,
    allowlist_var: Vec<&str>,
) -> bindgen::Bindings
{
    // Build include list
    let mut include_cmds = if cfg!(target_os = "linux") {
        get_pkgconfig_include_path_cmds(lib_name).expect(format!("Unable to find library: {}", lib_name).as_str())
    } else {
        panic!("Unknown platform");
    };

    for (_, include) in user_includes.iter().enumerate() {
        include_cmds.push(include.to_string());
    }

    // only rebuild bindings if the wrapper file has upded
    println!("cargo:rerun-if-changed={}", wrapper_file);

    // generate the bindings
    let mut builder = bindgen::Builder::default()
        .header(wrapper_file)
        .clang_args(include_cmds)
        .blocklist_file(".*/math.h")
        .blocklist_file(".*/stdint.h")
        .blocklist_file(".*/inttypes.h")
        .blocklist_file(".*/features.h")
        .blocklist_file(".*/stdc-predef.h")
        .generate_comments(false)
        .prepend_enum_name(false);

    for func in allowlist_function {
        builder = builder.allowlist_function(func);
    }
    for type_ in allowlist_type {
        builder = builder.allowlist_type(type_);
    }
    for var in allowlist_var {
        builder = builder.allowlist_var(var);
    }

    builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect(format!("Unable to generate bindings for {}", wrapper_file).as_str())
}

/* ======================================================================== */
/* Vulkan bindings                                                          */

// create un-optioned function pointer types, I didn't need parsing after all
fn parse_and_output_vulkan_bindings(mut bindings: String)
{
    static FUNC_PREFIX: &str = "pub type PFN_";
    static DECL_PREFIX: &str = "Option<";
    static DECL_SUFFIX: &str = ">;";

    let mut unwrapped_fn_pointer_types = String::new();

    for func in bindings.match_indices(FUNC_PREFIX) {
        /**/

        let func_str = bindings.split_at(func.0).1.strip_prefix(FUNC_PREFIX).unwrap();
        let func = func_str.split_at(func_str.find(' ').unwrap()).0;

        let decl_str = func_str
            .split_at(func_str.find(DECL_PREFIX).unwrap())
            .1
            .strip_prefix(DECL_PREFIX)
            .unwrap();
        let decl_full = decl_str.split_at(decl_str.find(DECL_SUFFIX).unwrap()).0.trim();
        let last_ch = decl_full.len() - 1;
        let decl = if &decl_full[last_ch..decl_full.len()] == "," {
            decl_full.split_at(last_ch).0
        } else {
            decl_full
        };

        unwrapped_fn_pointer_types.push_str(format!("pub type FN_{} = {};\n", func, decl).as_str());
    }

    bindings += &unwrapped_fn_pointer_types;

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_path.join("vulkan_bindings.rs"), bindings).unwrap();
}

fn gen_vulkan_bindings()
{
    let bindings = gen_bindings(
        "vulkan",
        "vendor/vulkan/vulkan_wrapper.h",
        vec!["-Ivendor/vulkan/1.3.296/"],
        vec!["vk.*"],
        vec!["PFN.*", "Vk.*"],
        vec!["Vk.*", "VK.*"],
    );

    let mut bindings_buf = String::new();
    bindings
        .write(Box::new(unsafe { bindings_buf.as_mut_vec() }))
        .expect("Couldn't write Vulkan bindings");

    parse_and_output_vulkan_bindings(bindings_buf);
}

/* ======================================================================== */
/* GLFW bindings                                                          */

fn generate_glfw_bindings() {
    // This is the directory where the `c` library is located.
    let libdir_path = PathBuf::from("vendor/glfw/3.4/bin/glfw/build-release/src/")
        // Canonicalize the path as `rustc-link-search` requires an absolute
        // path.
        .canonicalize()
        .expect("cannot canonicalize path");

    let libdir_path_str = libdir_path.to_str().expect("Path is not a valid string.");

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", libdir_path_str);

    // Tell cargo to tell rustc to link the shared libraries.
    // todo: the compiler is not picking up my compiled glfw. i'll figure that out later.
    println!("cargo:rustc-link-lib=glfw");

    // only rebuild bindings if the wrapper file has upded
    println!("cargo:rerun-if-changed={}", "vendor/glfw/glfw_wrapper.h");
    println!("cargo:rerun-if-changed={}", "vendor/glfw/glfw_wrapper_native.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate bindings for.
        .header("vendor/glfw/glfw_wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate glfw bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("glfw_bindings.rs"))
        .expect("Couldn't write glfw bindings!");

    // generate bindings for the native api (it conflicts with the normal glfw api)
    // let native_bindings = bindgen::Builder::default()
    //     .header("vendor/glfw/glfw_wrapper_native.h")
    //     .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    //     .generate()
    //     .expect("Unable to generate glfw native bindings");

    // native_bindings
    //     .write_to_file(out_path.join("glfw_native_bindings.rs"))
    //     .expect("Couldn't write native glfw bindings!");
}

fn main() {
    gen_vulkan_bindings();
    generate_glfw_bindings();
}
