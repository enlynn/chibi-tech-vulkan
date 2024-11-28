# Chibi Tech Vulkan

This is my little vulkan engine I am testing as I convert it to Rust.

# Platform Support

- Linux: Xlib and Wayland supported by default.
- Windows: support is planned and prototyped, but likely not going to work at the moment.

# Dependencies

### Rust

As this project is written in Rust, users must have Rust installed. Follow the instructions listed [here](https://www.rust-lang.org/tools/install).

### Vulkan

User must install the [Vulkan SDK](https://vulkan.lunarg.com/) onto their system. Correct version headers are included with this repo.

### GLFW

GLFW is used for its cross-platform windowing system. Custom binaries are built on first compilation using the `vendor/` crate.

### IMGUI

GLFW is used for its immediate mode GUI. Custom binaries are built on first compilation using the `vendor/` crate.

# Build Instructions

1. Make sure to install the dependencies described above.
2. Clone the repository: `git clone git@github.com:enlynn/chibi-tech-vulkan.git`
3. Navigate to the root of the project: `cd chibi-tech-vulkan`
4. Build: `cargo build` or `cargo build --release`
5. Run the Testbed project: `cargo run testbed`

# Supported Features

- The engine opens a window! :D

# TODO

- Pull vendor libs that require generating bindings into their own crate in hopes of reducing compile time when we need to run the build script.
- Vulkan Setup
-- Convert old C++ code to Rust
-- Road to the Vulkan Triangle
- Engine Features
-- Asset System / File Loading
