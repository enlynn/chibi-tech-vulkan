use std::path::{Path, PathBuf};

#[cfg(feature = "generator")]
use std::fs;

/* ======================================================================== */
/* Helpers                                                                  */

// on linux, search for system library
#[cfg(feature = "generator")]
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

#[cfg(feature = "generator")]
fn gen_bindings(
    system_lib_name:    Option<&str>,
    wrapper_file:       &str,
    user_includes:      Vec<&str>,
    allowlist_function: Vec<&str>,
    allowlist_type:     Vec<&str>,
    allowlist_var:      Vec<&str>,
) -> bindgen::Bindings
{
    // Build include list
    let mut include_cmds = if cfg!(target_os = "linux") {
        if let Some(lib_name) = system_lib_name {
           get_pkgconfig_include_path_cmds(lib_name).expect(format!("Unable to find library: {}", lib_name).as_str())
        } else {
            Vec::<String>::new()
        }
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
/* IMGUI                                                                    */

const IMGUI_SRC_DIR: &str = "src/imgui/cpp";
const IMGUI_CMAKE:   &str = "CMakeLists.txt";

#[cfg(feature = "generator")]
const IMGUI_WRAPPER: &str = "src/imgui/imgui_wrapper.h";

fn build_imgui_lib() {
    let libdir_path = PathBuf::from(IMGUI_SRC_DIR)
        .canonicalize()
        .expect("cannot canonicalize path");

    let cmake_path = libdir_path.join(IMGUI_CMAKE);
    let imgui_binary_path = "bin/imgui-bin";

    // generate build files
    if !std::process::Command::new("cmake")
        .arg("-S")
        .arg(libdir_path)
        .arg("-B")
        .arg(&imgui_binary_path)
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .arg("-DIMGUI_STATIC=yes")
        //.arg("-DIMGUI_FREETYPE=yes")        //todo: support freetype
        .output()
        .expect("could not spawn `cmake`")
        .status
        .success()
    {
        // Panic if the command was not successful.
        panic!("failed to generate imgui build files");
    }

    // build imgui
    if !std::process::Command::new("cmake")
        .arg("--build")
        .arg(&imgui_binary_path)
        .output()
        .expect("could not spawn `cmake`")
        .status
        .success()
    {
        // Panic if the command was not successful.
        panic!("failed to build imgui");
    }

    let binary_fullpath = PathBuf::from(imgui_binary_path)
            .canonicalize()
            .expect("cannot canonicalize path");


    println!("cargo:rerun-if-changed={}",  cmake_path.to_str().expect("Failed to get string representation of imgui cmake path."));
    println!("cargo:rustc-link-search={}", binary_fullpath.to_str().expect("Failed to get string representation of imgui binary path."));
    println!("cargo:rustc-link-lib=cimgui");
}

#[cfg(feature = "generator")]
fn generate_imgui_bindings() {
    let bindings = gen_bindings(
        None,
        IMGUI_WRAPPER,
        vec![],
        vec!["ig.*"],
        vec!["Im.*"],
        vec!["Im.*"],
    );

    let mut bindings_buf = String::new();
    bindings
        .write(Box::new(unsafe { bindings_buf.as_mut_vec() }))
        .expect("Couldn't write Vulkan bindings.");

    let out_path = PathBuf::from("src/imgui");
    fs::write(out_path.join("imgui_bindings.rs"), bindings_buf).unwrap();

    let wrapper_fullpath = PathBuf::from(IMGUI_WRAPPER)
            .canonicalize()
            .expect("cannot canonicalize path");

    println!("cargo:rerun-if-changed={}",  wrapper_fullpath.to_str().expect("failed to get imgui fullpath"));
}

/* ======================================================================== */
/* GLFW                                                                     */

const GLFW_URL:     &str = "https://github.com/glfw/glfw";
const GLFW_TAG:     &str = "3.4";

#[cfg(feature = "generator")]
const GLFW_WRAPPER: &str = "src/glfw/glfw_wrapper.h";

fn build_glfw_lib() {
    let glfw_target_dir = Path::new("src/glfw/cpp");

    if !glfw_target_dir.exists() {
        // 1. make the output directory
        std::fs::create_dir(&glfw_target_dir).expect("Failed to create output directory for glfw");

        // 2. clone the repo
        if !std::process::Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("--branch")
            .arg(GLFW_TAG)                                                  // GLFW tag to fg=etch
            .arg(GLFW_URL)                                                  // GLFW git url
            .arg(&glfw_target_dir.to_str().expect("failed to make string")) // directory we clone to
            .output()
            .expect("could not spawn `git`")
            .status
            .success()
        {
            // Panic if the command was not successful.
            panic!("failed to build imgui");
        }
    }

    let src_path = PathBuf::from("src/glfw/cpp")
        .canonicalize()
        .expect("cannot canonicalize path");

    let glfw_binary_path = "bin/glfw-bin";

    //
    if !std::process::Command::new("cmake")
        .arg("-S")
        .arg(&src_path)
        .arg("-B")
        .arg(&glfw_binary_path)
        .arg("-DPREFIX=lib")
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .arg("-DGLFW_BUILD_EXAMPLES=OFF")
        .arg("-DGLFW_BUILD_TESTS=OFF")
        .arg("-DGLFW_BUILD_DOCS=OFF")
        .arg("-DGLFW_INSTALL=OFF")
        .arg("-DBUILD_SHARED_LIBS=ON")
        .arg("-DIMGUI_STATIC=yes")
        //.arg("-DIMGUI_FREETYPE=yes")        //todo: support freetype
        .output()
        .expect("could not spawn `cmake`")
        .status
        .success()
    {
        // Panic if the command was not successful.
        panic!("failed to generate imgui build files");
    }

    // build glfw
    if !std::process::Command::new("cmake")
        .arg("--build")
        .arg(&glfw_binary_path)
        .output()
        .expect("could not spawn `cmake`")
        .status
        .success()
    {
        // Panic if the command was not successful.
        panic!("failed to build imgui");
    }

    let cmake_path = src_path.join("CMakeLists.txt");

    let mut binary_fullpath = PathBuf::from(glfw_binary_path)
            .canonicalize()
            .expect("cannot canonicalize path");

    binary_fullpath = binary_fullpath.join("src");

    println!("cargo:rerun-if-changed={}",  cmake_path.to_str().expect("Failed to get string representation of glfw cmake path."));
    println!("cargo:rustc-link-search={}", binary_fullpath.to_str().expect("Failed to get string representation of glfw binary path."));
    println!("cargo:rustc-link-lib=glfw");
}

#[cfg(feature = "generator")]
fn generate_glfw_bindings() {
    let bindings = gen_bindings(
        None,
        GLFW_WRAPPER,
        vec![],
        vec!["glfw.*"],
        vec!["GLFW.*"],
        vec!["GLFW.*"],
    );

    let mut bindings_buf = String::new();
    bindings
        .write(Box::new(unsafe { bindings_buf.as_mut_vec() }))
        .expect("Couldn't write Vulkan bindings.");

    let out_path = PathBuf::from("src/glfw");
    fs::write(out_path.join("glfw_bindings.rs"), bindings_buf).unwrap();

    let wrapper_fullpath = PathBuf::from(GLFW_WRAPPER)
            .canonicalize()
            .expect("cannot canonicalize path");

    println!("cargo:rerun-if-changed={}",  wrapper_fullpath.to_str().expect("failed to get glfw fullpath"));
}

/* ======================================================================== */
/* Vulkan                                                                   */

fn build_vulkan_vma() {
    // Now, let's create a smol library for vma
    //
    let libdir_src_path = PathBuf::from("src/vulkan")
        .canonicalize()
        .expect("cannot canonicalize path");

    let libdir_dst_path_str = "bin/vulkan-bin";
    if !Path::new(libdir_dst_path_str).exists() {
        std::fs::create_dir(&libdir_dst_path_str).expect("Failed to create output directory for vulkan");
    }

    let libdir_dst_path = PathBuf::from(libdir_dst_path_str)
        .canonicalize()
        .expect("cannot canonicalize path");

    // This is the path to the intermediate object file for our library.
    let src_path = libdir_src_path.join("vulkan_wrapper.c");
    let obj_path = libdir_dst_path.join("vulkan_wrapper.o");
    // This is the path to the static library file.
    let lib_path = libdir_dst_path.join("libvma.a");

    // todo: this should change based on the available compiler.
    if !std::process::Command::new("clang++")
        .arg("-std=c++17")
        .arg("-Wno-missing-field-initializers")
        .arg("-Wno-unused-variable")
        .arg("-Wno-unused-parameter")
        .arg("-Wno-unused-private-field")
        .arg("-Wno-reorder")
        .arg("-DVMA_STATIC_VULKAN_FUNCTIONS=0")
        .arg("-DVMA_DYNAMIC_VULKAN_FUNCTIONS=0")
        .arg("-c")
        .arg("-o")
        .arg(&obj_path)
        .arg(&src_path)
        .output()
        .expect("could not spawn `clang`")
        .status
        .success()
    {
        // Panic if the command was not successful.
        panic!("could not compile vma object file");
    }

    // Run `ar` to generate the library
    if !std::process::Command::new("ar")
        .arg("rcs")
        .arg(lib_path)
        .arg(obj_path)
        .output()
        .expect("could not spawn `ar`")
        .status
        .success()
    {
        // Panic if the command was not successful.
        panic!("could not emit library file");
    }

    // Tell cargo to tell rustc to link the vma lib
    println!("cargo:rustc-link-search={}", libdir_dst_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=vma");
}

// create un-optioned function pointer types, I didn't need parsing after all
#[cfg(feature = "generator")]
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

    let out_path = PathBuf::from("src/vulkan");
    fs::write(out_path.join("vulkan_bindings.rs"), bindings).unwrap();
}

#[cfg(feature = "generator")]
fn gen_vulkan_bindings()
{
    let bindings = gen_bindings(
        Some("vulkan"),
        "src/vulkan/vulkan_wrapper.h",
        vec!["-Isrc/vulkan/cpp/"],
        vec!["vk.*", "vma.*"],
        vec!["PFN.*", "Vk.*", "Vma.*"],
        vec!["Vk.*", "VK.*", "Vma.*"],
    );

    let mut bindings_buf = String::new();
    bindings
        .write(Box::new(unsafe { bindings_buf.as_mut_vec() }))
        .expect("Couldn't write Vulkan bindings.");

    parse_and_output_vulkan_bindings(bindings_buf);
}

/* ======================================================================== */
/* Main                                                                     */

fn main() {
    // build vendor libraries
    build_imgui_lib();
    build_glfw_lib();
    build_vulkan_vma();

    // generate library bindings
    #[cfg(feature = "generator")]
    generate_imgui_bindings();
    #[cfg(feature = "generator")]
    generate_glfw_bindings();
    #[cfg(feature = "generator")]
    gen_vulkan_bindings();
}
