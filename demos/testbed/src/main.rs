// temporary
#![allow(unused)]

#![feature(iter_advance_by)]

extern crate chibi_engine;

use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::io::prelude::*;
use std::fs::File;

use chibi_engine::core::engine::*;
use chibi_engine::core::asset_system::AssetDrive;
use chibi_engine::math::{
    *,
    float2::*,
    float3::*,
    float4::*,
    float4x4::*,
};

use chibi_engine::window::*;

use chibi_engine::renderer::{
    command_buffer::*,
    mesh::Vertex,
};

struct Testbed{
    engine: Rc<Engine>,
    mesh:   ChibiGeometry,

    camera: Camera,

    // Event listeners for the window
    //

    // The receiever is where events come down throw
    event_reciever: chibi_engine::window::EventReciever,
    // Store the listener so we can send all events down the same pipe
    event_listener: chibi_engine::window::EventListener,
}

#[derive(Debug, Copy, Clone)]
struct ObjFaceIdx {
    pos:  Option<i32>,
    norm: Option<i32>,
    uv0:  Option<i32>,
}

#[derive(Debug)]
struct ObjFace{
    v0: ObjFaceIdx,
    v1: ObjFaceIdx,
    v2: ObjFaceIdx,
}

impl Default for ObjFaceIdx {
    fn default() -> Self {
        Self{
            pos:  None,
            norm: None,
            uv0:  None,
        }
    }
}

#[derive(Debug)]
struct ObjGeometry {
    name:       String,
    positions:  Vec<Float3>,
    normals:    Vec<Float3>,
    tex_coords: Vec<Float2>,
    faces:      Vec<ObjFace>,
}

#[derive(Debug)]
struct ChibiGeometry {
    vertices: Vec<Vertex>,
    indices:  Vec<u32>,
}

fn get_mesh_directory(engine: &Engine) -> PathBuf {
    let asset_dir  = engine.get_asset_dir(AssetDrive::Res);
    return asset_dir.join("geometry");
}

fn parse_float(tokens: &mut std::str::Split<'_, char>) -> f32 {
    if let Some(pos) = tokens.next() {
        pos.parse().unwrap_or(0.0)
    } else { 0.0 }
}

fn parse_int(tokens: &mut std::str::Split<'_, char>) -> i32 {
    if let Some(pos) = tokens.next() {
        pos.parse().unwrap_or(0)
    } else { 0 }
}

fn parse_int_str(tokens: &str) -> i32 {
    tokens.parse().unwrap_or(0) as i32
}

fn parse_float3(tokens: &mut std::str::Split<'_, char>) -> Float3 {
    let pos0: f32 = parse_float(tokens);
    let pos1: f32 = parse_float(tokens);
    let pos2: f32 = parse_float(tokens);
    return Float3::new(pos0, pos1, pos2);
}

fn parse_float2(tokens: &mut std::str::Split<'_, char>) -> Float2 {
    let pos0: f32 = parse_float(tokens);
    let pos1: f32 = parse_float(tokens);
    return Float2::new(pos0, pos1);
}

fn parse_obj_face_idx(tokens: &mut std::str::Split<'_, char>) -> ObjFaceIdx {
    let mut indices = if let Some(pair) = tokens.next() {
        pair.split("/")
    } else { panic!("Malformed face."); };

    // note: obj indices start at 1
    let pos = if let Some(p) = indices.next() {
        if p.len() > 0 {
            Some((parse_int_str(p) - 1))
        } else { None }
    } else { panic!("Failed to fetch position token"); };

    let uv0 = if let Some(p) = indices.next() {
        if p.len() > 0 {
            Some((parse_int_str(p) - 1))
        } else { None }
    } else { None };

    let norm = if let Some(p) = indices.next() {
        if p.len() > 0 {
            Some((parse_int_str(p) - 1))
        } else { None }
    } else { None };

    return ObjFaceIdx{
        pos,
        norm,
        uv0,
    };
}

fn parse_obj_file(file: &str) -> ObjGeometry {
    let mut result = ObjGeometry{
        name:       String::new(),
        positions:  Vec::new(),
        normals:    Vec::new(),
        tex_coords: Vec::new(),
        faces:      Vec::new(),
    };

    let mut lines = file.lines();

    while let Some(line) = lines.next() {
        if line.starts_with('#'){ //@assume: comments are at the start of a line
            continue; //skip comments
        }

        let mut tokens = line.split(' ');
        match tokens.advance_by(1) {
            Ok(_)  => {},
            Err(e) => continue, // this is an invalid line
        }

        // todo: test a file with multiple (sub)objects
        if line.starts_with('o') { //@assume: this is new geometry
            if let Some(name) = tokens.next() {
                result.name = String::from(name);
            }
        }
        else if line.starts_with("vt") { // texture vertex
            result.tex_coords.push(parse_float2(&mut tokens));
        }
        else if line.starts_with("vn") { // normal vertex
            result.normals.push(parse_float3(&mut tokens));
        }
        else if line.starts_with('v') { // geometric vertex
            result.positions.push(parse_float3(&mut tokens));
        }
        else if line.starts_with('s') { // smooth shading
            //todo: smooth shading.
        }
        else if line.starts_with('f') { // vertex face, @assume: 3 vertices per face
            let vert_count = tokens.clone().count();
            assert!(vert_count == 3 || vert_count == 4, "Only supports quads or tris. Found: {} vertices. {}", vert_count, line);

            if vert_count == 3 {
                let v0 = parse_obj_face_idx(&mut tokens);
                let v1 = parse_obj_face_idx(&mut tokens);
                let v2 = parse_obj_face_idx(&mut tokens);
                result.faces.push(ObjFace{ v0, v1, v2 });
            } else {
                // An OBJ file can have varying types of "faces". For now, assume
                // a face will have either 3 (tris) or 4 (quads) vertices. When a
                // quad is supplied, it is split into two triangles using:
                //
                // 0 1 2 3 -> [0 1 2], [0 2 3]
                //
                // Alternatively: 0 1 2 3 -> [0 1 2] [2 3 1]
                //
                // 3-------2
                // |      /|
                // |    /  |
                // |  /    |
                // |/      |
                // 0-------1
                //
                // todo: test to find the right orientation.
                //
                let v0 = parse_obj_face_idx(&mut tokens);
                let v1 = parse_obj_face_idx(&mut tokens);
                let v2 = parse_obj_face_idx(&mut tokens);
                let v3 = parse_obj_face_idx(&mut tokens);

                let tri0 = ObjFace{ v0, v1, v2 };
                let tri1 = ObjFace{ v0, v1: v2, v2: v3 };

                result.faces.push(tri0);
                result.faces.push(tri1);
            }
        }
        else {
            //note: unsupported token, return an error?
        }
    }

    println!("Parsed obj:\n{:?}", result);
    return result;
}

fn make_vertex_from_face(obj: &ObjGeometry, face: ObjFaceIdx) -> Vertex {
    let position: Float3 = {
        if let Some(idx) = face.pos {
            if obj.positions.len() > 0 {
                if idx < 0 {
                    let wrapped_idx = (obj.positions.len() as i32 + idx) as usize;
                    obj.positions[wrapped_idx]
                } else {
                    obj.positions[idx as usize]
                }
            } else { Float3::zero() }
        } else { Float3::zero() }
    };

    let tex_coord: Float2 = {
        if let Some(idx) = face.uv0 {
            if obj.tex_coords.len() > 0 {
                if idx < 0 {
                    let wrapped_idx = (obj.tex_coords.len() as i32 + idx) as usize;
                    obj.tex_coords[wrapped_idx]
                } else {
                    obj.tex_coords[idx as usize]
                }
            } else { Float2::zero() }
        } else { Float2::zero() }
    };

    let normal: Float3 = {
        if let Some(idx) = face.norm {
            if obj.normals.len() > 0 {
                if idx < 0 {
                    let wrapped_idx = (obj.normals.len() as i32 + idx) as usize;
                    obj.normals[wrapped_idx]
                } else {
                    obj.normals[idx as usize]
                }
            } else { Float3::zero() }
        } else { Float3::zero() }
    };

    return Vertex{
        position,
        uv_x: tex_coord.x,
        normal,
        uv_y: tex_coord.y,
        color: Float4::new(0.0, 0.0, 0.0, 1.0),
    };
}

fn convert_obj_file(obj: &ObjGeometry) -> ChibiGeometry {
    let mut result = ChibiGeometry{
        vertices: Vec::new(),
        indices:  Vec::new(),
    };

    assert!(obj.positions.len() > 0);
    result.vertices.reserve(obj.positions.len());

    let mut index_count: u32 = 0;
    for face in &obj.faces {
        //note: this might result in duplicated geometry

        let v0 = make_vertex_from_face(&obj, face.v0);
        let v1 = make_vertex_from_face(&obj, face.v1);
        let v2 = make_vertex_from_face(&obj, face.v2);

        result.vertices.push(v0);
        result.vertices.push(v1);
        result.vertices.push(v2);

        result.indices.push(index_count + 0);
        result.indices.push(index_count + 1);
        result.indices.push(index_count + 2);

        index_count += 3;
    }

    for i in 0..obj.positions.len() {
        let mut vertex = Vertex::new();
        vertex.position = obj.positions[i];
    }

    return result;
}

fn import_obj_file(asset_path: &PathBuf, name: &str) -> ChibiGeometry {
    let mut name_str = String::from_str(name).expect("Failed to construct string.");
    name_str.push_str(".obj");

    let mesh_file = asset_path.join(name_str);

    // let's read the file
    let mut file = match File::open(&mesh_file) {
        Err(why) => panic!("couldn't open {}: {}", mesh_file.display(), why),
        Ok(file) => file,
    };

    // todo: validate the file is UTF8

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut file_str = String::new();
    match file.read_to_string(&mut file_str) {
        Err(why) => panic!("couldn't read {}: {}", mesh_file.display(), why), //todo: Result<ChibiGeometry, ParserError>
        Ok(_)    => {},
    }

    let parsed_obj = parse_obj_file(&file_str);
    return convert_obj_file(&parsed_obj);
}

impl Game for Testbed {
    fn on_init(&mut self) -> bool {
        // register for window events
        self.engine.register_window_event(WindowEventType::OnKeyboardKey, self.event_listener.clone());
        self.engine.register_window_event(WindowEventType::OnMouseButton, self.event_listener.clone());
        self.engine.register_window_event(WindowEventType::OnMouseMove,   self.event_listener.clone());
        self.engine.register_window_event(WindowEventType::OnMouseScroll, self.event_listener.clone());

        let mesh_dir = get_mesh_directory(&self.engine);
        //println!("Mesh dir: {:?}", mesh_dir);

        //let cube_pos      = import_obj_file(&mesh_dir, "cube_pos");
        //let cube_pos_norm = import_obj_file(&mesh_dir, "cube_pos_norm");
        //let cube_pos_tex  = import_obj_file(&mesh_dir, "cube_pos_tex");
        //let cube_all      = import_obj_file(&mesh_dir, "cube");

        self.mesh = import_obj_file(&mesh_dir, "suzanne");
        //println!("Parsed file:\n{:?}\n\n", self.mesh);

        let mut upload_commands = RenderCommandBuffer::default();
        let mesh_info = CreateMeshInfo{
            vertices:     self.mesh.vertices.as_ptr(),
            vertex_count: self.mesh.vertices.len(),
            indices:      self.mesh.indices.as_ptr(),
            index_count:  self.mesh.indices.len(),
            transform:    Float4x4::get_rotate_z_matrix(180.0),
            engine_id:    0,
        };

        upload_commands.add_command(RenderCommand::CreateMesh(mesh_info));
        self.engine.submit_render_command_buffer(upload_commands);

        return true;
    }

    fn on_update(&mut self) -> bool {
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
                WindowEvent::MouseScroll(scroll)         => {},
                default                                  => {},
            }
        }

        self.camera.on_update();

        let camera_info = RenderCommand::UpdateCamera(CameraStateInfo {
            view_matrix:        self.camera.get_view_matrix(),
            perspective_matrix: Float4x4::get_perspective_matrix(45.0, 1920.0/1080.0, 0.01, 1000.0),
        });

        let mut render_commands = RenderCommandBuffer::default();
        render_commands.add_command(camera_info);
        self.engine.submit_render_command_buffer(render_commands);

        return true;
    }

    fn on_render(&mut self)   -> bool { return true; }
    fn on_shutdown(&mut self) -> bool { return true; }
}

impl Default for ChibiGeometry {
    fn default() -> Self {
        Self{
            vertices: Vec::new(),
            indices:  Vec::new(),
        }
    }
}

fn get_info() -> GameInfo {
    GameInfo{
        title:         String::from("Chibi EngineTestbed"),
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
        mesh:           ChibiGeometry::default(),
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
            movement_speed:       0.15,
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
        if self.is_mouse_right_click {
            let is_pressed = ev.state == KeyState::Pressed || ev.state == KeyState::Held;
            let velocity = self.movement_speed * 0.000016; //todo: delta time

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

    pub fn on_update(&mut self) {
        if self.is_mouse_right_click {
            let velocity = self.movement_speed * 0.016; //todo: delta time

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
