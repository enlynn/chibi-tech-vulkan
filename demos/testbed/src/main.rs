// temporary
#![allow(unused)]

extern crate chibi_engine;

use std::path::PathBuf;
use std::rc::Rc;

use chibi_engine::core::engine::*;
use chibi_engine::core::asset_system::AssetDrive;
use common::math::{
    *,
    float3::*,
    float4x4::*,
};

use chibi_engine::window::*;

use chibi_engine::renderer::command_buffer::*;

use assetlib::mesh::*;

struct Testbed{
    engine: Rc<Engine>,
    mesh:   ChibiModel,

    camera: Camera,

    // Event listeners for the window
    //

    // The receiever is where events come down throw
    event_reciever: chibi_engine::window::EventReciever,
    // Store the listener so we can send all events down the same pipe
    event_listener: chibi_engine::window::EventListener,
}

fn get_mesh_directory(engine: &Engine) -> PathBuf {
    let asset_dir  = engine.get_asset_dir(AssetDrive::Res);
    return asset_dir.join("geometry");
}

impl Game for Testbed {
    fn on_init(&mut self) -> bool {
        // register for window events
        self.engine.register_window_event(WindowEventType::OnKeyboardKey, self.event_listener.clone());
        self.engine.register_window_event(WindowEventType::OnMouseButton, self.event_listener.clone());
        self.engine.register_window_event(WindowEventType::OnMouseMove,   self.event_listener.clone());
        self.engine.register_window_event(WindowEventType::OnMouseScroll, self.event_listener.clone());

        let mesh_dir = get_mesh_directory(&self.engine);

        //let suzanne = import_obj_file(&mesh_dir, "suzanne");
        self.mesh = assetlib::importer::obj::import_obj_file(&mesh_dir, "sponza/sponza");

        let mut upload_commands = RenderCommandBuffer::default();

        for geom in &self.mesh.geoms {
            let mesh_info = CreateMeshInfo{
                vertices:     geom.vertices.as_ptr(),
                vertex_count: geom.vertices.len(),
                indices:      geom.indices.as_ptr(),
                index_count:  geom.indices.len(),
                transform:    mul_rh(Float4x4::get_rotate_z_matrix(180.0), Float4x4::get_scale_matrix(0.5, 0.5, 0.5)),
                engine_id:    0,
            };

            upload_commands.add_command(RenderCommand::CreateMesh(mesh_info));
        }

        self.engine.submit_render_command_buffer(upload_commands);

        return true;
    }

    fn on_update(&mut self, frame_time_ms: f64) -> bool {
        while let Ok(ev) = self.event_reciever.try_recv() {
            match ev {
                WindowEvent::KeyPress(key_event)         => {
                    self.camera.on_key_event(key_event);
                },
                WindowEvent::MousePress(mouse_event)     => {
                    self.camera.on_mouse_press(mouse_event);
                },
                WindowEvent::MouseMove(mouse_move_event) => {
                    self.camera.on_mouse_move(mouse_move_event);
                },
                WindowEvent::MouseScroll(_scroll)        => {},
                default                                  => {},
            }
        }

        self.camera.on_update(frame_time_ms);

        let camera_info = RenderCommand::UpdateCamera(CameraStateInfo {
            view_matrix:        self.camera.get_view_matrix(),
            perspective_matrix: Float4x4::get_perspective_matrix(45.0, 1920.0/1080.0, 0.01, 2000.0),
        });

        let mut render_commands = RenderCommandBuffer::default();
        render_commands.add_command(camera_info);
        self.engine.submit_render_command_buffer(render_commands);

        return true;
    }

    fn on_render(&mut self)   -> bool { return true; }
    fn on_shutdown(&mut self) -> bool { return true; }
}

fn get_info() -> GameInfo {
    GameInfo{
        title:         String::from("Chibi Engine Testbed"),
        game_version:  chibi_engine::make_app_version(0, 0, 1),
        window_width:  1920,
        window_height: 1080,
        manifest_dir:  PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    }
}

fn main() {
    let chibi_engine = chibi_engine::new_engine(get_info());

    let (listener, reciever) = chibi_engine::window::make_event_channels();

    let testbed = Box::new(Testbed{
        engine:         chibi_engine.clone(),
        mesh:           ChibiModel{ geoms: Vec::new(), materials: Vec::new() },
        camera:         Camera::default(),
        event_listener: listener,
        event_reciever: reciever,
    });

    chibi_engine.register_game(testbed);
    chibi_engine.run();
}

struct Camera {
    position:          Float3,
    front:             Float3,
    up:                Float3,
    right:             Float3,
    world_up:          Float3,
    // euler Angles
    yaw:               f32,
    pitch:             f32,
    // camera options
    movement_speed:    f32,
    mouse_sensitivity: f32,
    zoom:              f32,

    // for input updates
    is_mouse_right_click: bool,
    is_w_held:            bool,
    is_s_held:            bool,
    is_a_held:            bool,
    is_d_held:            bool,
    mouse_x:              f32,
    mouse_y:              f32,
    delta_mouse_x:        f32,
    delta_mouse_y:        f32,

}

impl Default for Camera {
    fn default() -> Self {
        let mut result = Self{
            position:             Float3::zero(),
            front:                Float3::new(0.0, 0.0, -1.0),
            up:                   Float3::new(0.0, 1.0, 0.0),
            right:                Float3::zero(),
            world_up:             Float3::new(0.0, 1.0, 0.0),
            yaw:                  90.0,
            pitch:                0.0,
            movement_speed:       100.15,
            mouse_sensitivity:    0.1,
            zoom:                 45.0,
            is_mouse_right_click: false,
            is_w_held:            false,
            is_s_held:            false,
            is_a_held:            false,
            is_d_held:            false,
            mouse_x:              f32::MAX,
            mouse_y:              f32::MAX,
            delta_mouse_x:        0.0,
            delta_mouse_y:        0.0,
        };

        result.update_camera_vecs();
        return result;
    }
}

impl Camera {
    pub fn new(pos: Float3, up: Float3, yaw: f32, pitch: f32) -> Self {
        let mut result = Self::default();
        result.position = pos;
        result.world_up = up;
        result.pitch    = pitch;
        result.yaw      = yaw;
        result.update_camera_vecs();

        return result;
    }

    pub fn get_view_matrix(&self) -> Float4x4 {
        return Float4x4::get_look_at_matrix(self.position, self.position + self.front, self.up);
    }

    pub fn on_key_event(&mut self, ev: KeyEvent) {
        let is_pressed = ev.state == KeyState::Pressed || ev.state == KeyState::Held;
        if !is_pressed || self.is_mouse_right_click {
            if ev.key == KeyboardKey::W {        // forward
                self.is_w_held = is_pressed;
            } else if ev.key == KeyboardKey::S { // backwards
                self.is_s_held = is_pressed;
            } else if ev.key == KeyboardKey::A { // left
                self.is_a_held = is_pressed;
            } else if ev.key == KeyboardKey::D { // right
                self.is_d_held = is_pressed;
            }
        }
    }

    pub fn on_mouse_move(&mut self, ev: MouseMoveEvent) {
        if self.is_mouse_right_click {
            if self.mouse_x != f32::MAX && self.mouse_y != f32::MAX {
                self.delta_mouse_x += (ev.pos_x as f32 - self.mouse_x) * self.mouse_sensitivity;
                self.delta_mouse_y += (ev.pos_y as f32 - self.mouse_y) * self.mouse_sensitivity;
            }

            self.mouse_x = ev.pos_x as f32;
            self.mouse_y = ev.pos_y as f32;
        }
    }

    pub fn on_mouse_press(&mut self, ev: MouseEvent) {
        let is_pressed = ev.state == KeyState::Pressed || ev.state == KeyState::Held;
        self.is_mouse_right_click = ev.button == MouseButton::ButtonRight && is_pressed;

        if !self.is_mouse_right_click {
            self.mouse_x = f32::MAX;
            self.mouse_y = f32::MAX;
        }
    }

    pub fn on_mouse_wheel(&mut self, ev: i32) {
        //todo: update zoom or speed? ... maybe ctrl-wheel is speed
    }

    pub fn on_update(&mut self, frame_time_ms: f64) {
        let frame_time_s = (frame_time_ms / 1000.0) as f32;

        if self.is_mouse_right_click {
            let velocity = self.movement_speed * frame_time_s; //todo: delta time

            if self.is_w_held {        // forward
                self.position = self.position + (self.front * velocity);
            }

            if self.is_s_held { // backwards
                self.position = self.position - (self.front * velocity);
            }

            if self.is_a_held { // left
                self.position = self.position - (self.right * velocity);
            }

            if self.is_d_held { // right
                self.position = self.position + (self.right * velocity);
            }

            self.yaw   += self.delta_mouse_x;
            self.pitch += self.delta_mouse_y;

            let constrain_pitch = true;
            if constrain_pitch {
                if (self.pitch >  89.0) { self.pitch =  89.0; }
                if (self.pitch < -89.0) { self.pitch = -89.0; }
            }
        }

        self.update_camera_vecs();

        // reset mouse delta for next frame
        self.delta_mouse_x = 0.0;
        self.delta_mouse_y = 0.0;
    }

    pub fn update_camera_vecs(&mut self) {
        let yaw   = degrees_to_radians(self.yaw);
        let pitch = degrees_to_radians(self.pitch);

        let cos_pitch = pitch.cos();

        let mut front = Float3::zero();
        front.x = yaw.cos() * cos_pitch;
        front.y = pitch.sin();
        front.z = yaw.sin() * cos_pitch;
        self.front = front.unit();

        // normalize the vectors, because their length gets closer to 0 the more you look up or down which results in slower movement.
        self.right = self.front.cross(self.world_up).unit();
        self.up    = self.right.cross(self.front).unit();
    }
}
