
use std::cell::RefCell;
use std::ptr;

use crate::window;
use crate::renderer;

use vendor::imgui::*;

pub trait Game {
    fn on_init(&mut self)     -> bool;
    fn on_update(&mut self)   -> bool;
    fn on_render(&mut self)   -> bool; //Will this function be necessary?
    fn on_shutdown(&mut self) -> bool;
}

pub struct DefaultGame {}
impl Game for DefaultGame {
    fn on_init(&mut self)     -> bool { return false; }
    fn on_update(&mut self)   -> bool { return false; }
    fn on_render(&mut self)   -> bool { return false; }
    fn on_shutdown(&mut self) -> bool { return false; }
}

pub struct Engine {
    game:          RefCell<Box<dyn Game>>,
    window_system: window::WindowSystem,
    client_window: window::Window,
    render_system: RefCell<renderer::RenderSystem>,
}

impl Engine {
    pub fn new() -> Engine {
        let game = RefCell::new(Box::new(DefaultGame{}));
        let window_system = window::WindowSystem::new();
        // todo: set window title/width/height based on user game data.
        let client_window = window_system.create_window("Chibi Vulkan", 1920, 1080);
        let mut render_system = renderer::RenderSystem::new(renderer::RendererCreateInfo{
            surface: client_window.get_native_surface(),
        });


        let (width, height) = client_window.get_framebuffer_size();
        render_system.on_resize(width, height);

        let ig_ctx = unsafe { igCreateContext(std::ptr::null_mut()) };

        return Engine{
            game,
            window_system,
            client_window,
            render_system: RefCell::new(render_system),
        }
    }

    pub fn register_game(&self, game: Box<dyn Game>) {
        self.game.replace(game);
    }

    pub fn run(&self) {
        let mut game = self.game.borrow_mut();

        // run post-game engine setup
        let mut frame_index: usize = 0;

        // initialize the game
        let mut game_res = game.on_init();
        if !game_res {
            return;
        }

        // let's upload some test geometry
        //
        let mut upload_commands = renderer::RenderCommandBuffer::default();

        {
            use crate::math::{float3::*, float4::*};
            use renderer::{Vertex, RenderCommand, CreateMeshInfo};

            let mut vertices: [Vertex; 4] = [Vertex::default(); 4];

            vertices[0].position = Float3{ x:  0.5, y: -0.5, z: 0.0 };
            vertices[1].position = Float3{ x:  0.5, y:  0.5, z: 0.0 };
            vertices[2].position = Float3{ x: -0.5, y: -0.5, z: 0.0 };
            vertices[3].position = Float3{ x: -0.5, y:  0.5, z: 0.0 };

            vertices[0].color = Float4{ x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
            vertices[1].color = Float4{ x: 0.5, y: 0.5, z: 0.5, w: 1.0 };
            vertices[2].color = Float4{ x: 1.0, y: 0.0, z: 0.0, w: 1.0 };
            vertices[3].color = Float4{ x: 0.0, y: 1.0, z: 0.0, w: 1.0 };

            let indices: [u32; 6] = [
                0, 1, 2, 2, 1, 3,
            ];

            let mesh_info = CreateMeshInfo{
                vertices:     vertices.as_ptr(),
                vertex_count: vertices.len(),
                indices:      indices.as_ptr(),
                index_count:  indices.len(),
                engine_id:    0,
            };

            upload_commands.add_command(RenderCommand::CreateMesh(mesh_info));
        }

        self.render_system.borrow_mut().submit_render_commands(upload_commands);

        // let's read back the command list
        //   note: this will eventually be deferred.
        //
        // todo:

        //use std::time::{Duration, Instant};
        //let mut current_time = Instant::now();

        loop {
            if !self.window_system.pump_window_message() {
                break;
            }

            //let mut elapsed_time = Instant::now();
            //println!("\tFinished polling input. {}", (elapsed_time - current_time).as_millis_f64());
            //current_time = elapsed_time;

            if self.client_window.should_window_close() {
                break;
            }

            game_res = game.on_update();
            if !game_res {
                break;
            }

            // this should be the "editor_begin_frame()"
            if true {
                use vendor::imgui::*;
                use crate::util::ffi::*;

                ig_vulkan_new_frame();
                ig_glfw_new_frame();
                call!(igNewFrame);

                let mut is_open = true;
                call!(igShowDemoWindow, &mut is_open);

                self.render_system.borrow_mut().on_editor_update();

                call!(igEndFrame);
            }

            //elapsed_time = Instant::now();
            //println!("\tFinished updating imgui. {}", (elapsed_time - current_time).as_millis_f64());
            //current_time = elapsed_time;

            game_res = game.on_render();
            if !game_res {
                break;
            }

            let empty_cmd_buffer = renderer::RenderCommandBuffer::default();
            self.render_system.borrow_mut().render(empty_cmd_buffer);

            // this should be the "editor_render_external_windows()" - render imgui viewports
            unsafe {
                let io = { &mut *vendor::imgui::igGetIO() }; // gets a mutable reference
                if (io.ConfigFlags & vendor::imgui::ImGuiConfigFlags_ViewportsEnable as i32) != 0
                {
                    vendor::imgui::igUpdatePlatformWindows();
                    vendor::imgui::igRenderPlatformWindowsDefault(ptr::null_mut(), ptr::null_mut());
                }
            }

            //elapsed_time = Instant::now();
            //println!("\tFinished rendering. {}", (elapsed_time - current_time).as_millis_f64());
            //current_time = elapsed_time;

            frame_index += 1;
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let mut game = self.game.borrow_mut();
        game.on_shutdown();

        self.render_system.borrow_mut().destroy();
    }
}
