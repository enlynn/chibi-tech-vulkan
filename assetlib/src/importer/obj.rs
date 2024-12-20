use common::math::{*, float3::*, float4::*, float2::*};

use crate::material::*;
use crate::mesh::*;

use meshopt::ffi::*;

use std::ffi::{c_int, c_void};
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;

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

/*
0. Color on and Ambient off
1. Color on and Ambient on
2. Highlight on
3. Reflection on and Ray trace on
4. Transparency: Glass on, Reflection: Ray trace on
5. Reflection: Fresnel on and Ray trace on
6. Transparency: Refraction on, Reflection: Fresnel off and Ray trace on
7. Transparency: Refraction on, Reflection: Fresnel on and Ray trace on
8. Reflection on and Ray trace off
9. Transparency: Glass on, Reflection: Ray trace off
10. Casts shadows onto invisible surfaces
*/
#[derive(Debug)]
struct ObjMaterial {
    name:                      String,
    ambient_color:             Float3,
    diffuse_color:             Float3,
    specular_color:            Float3,
    specular_exponent:         Float,
    dissolve:                  Float, //@note: d=1.0/Tr=0.0 means fully opaque. Tr = 1 - d
    transmission_filter_color: Float3,
    is_xyz_color_space:        bool,
    //spectral_curve_file:       String, //-> ignored
    index_of_refraction:       Float,
    illumination_mode:         i32,

    diffuse_map:               PathBuf,
    displacement_map:          PathBuf,
    ambient_map:               PathBuf,
}

#[derive(Debug)]
struct ObjGeometry {
    name:           String,
    faces:          Vec<ObjFace>,
    material_index: Option<usize>,
}

#[derive(Debug)]
struct ObjModel {
    positions:  Vec<Float3>,
    normals:    Vec<Float3>,
    tex_coords: Vec<Float2>,
    geoms:      Vec<ObjGeometry>,
    materials:  Vec<ObjMaterial>,
}

impl Default  for ObjGeometry {
    fn default() -> Self {
        Self{
            name:           String::new(),
            faces:          Vec::new(),
            material_index: None,
        }
    }
}

impl Default for ObjModel {
    fn default() -> Self {
        Self {
            positions:  Vec::new(),
            normals:    Vec::new(),
            tex_coords: Vec::new(),
            geoms:      Vec::new(),
            materials:  Vec::new(),
        }
    }
}

impl Default for ObjMaterial {
    fn default() -> Self {
        Self{
            name:                      String::new(),
            ambient_color:             Float3::zero(),
            diffuse_color:             Float3::zero(),
            specular_color:            Float3::zero(),
            specular_exponent:         0.0,
            dissolve:                  1.0, //obj note: d=1.0/Tr=0.0 means fully opaque. Tr = 1 - d
            transmission_filter_color: Float3::zero(),
            is_xyz_color_space:        false,
            //spectral_curve_file:       String, //-> ignored
            index_of_refraction:       0.0,
            illumination_mode:         1,
            diffuse_map:               PathBuf::new(),
            displacement_map:          PathBuf::new(),
            ambient_map:               PathBuf::new(),
        }
    }
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

fn parse_obj_material_file(material_filename: &str, directory_path: &Path) -> Vec<ObjMaterial> {
    let mut result: Vec<ObjMaterial> = Vec::new();
    let mut material_index = 0;

    let fullpath = directory_path.join(material_filename);

    let parent_path = fullpath.parent().expect("Failed to get parent path for the material file");

    // let's read the material file
    let mut file = match File::open(&fullpath) {
        Err(why) => { println!("couldn't open {}: {}", fullpath.display(), why); return result; },
        Ok(file) => file,
    };

    // todo: validate the file is UTF8

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut file_str = String::new();
    match file.read_to_string(&mut file_str) {
        Err(why) => { println!("couldn't read {}: {}", fullpath.display(), why); return result; },
        Ok(_)    => {},
    }

    let mut lines = file_str.lines();
    while let Some(line) = lines.next() {
        if line.starts_with('#'){ //@assume: comments are at the start of a line
            continue; //skip comments
        }

        let mut tokens = line.split(' ');
        match tokens.advance_by(1) {
            Ok(_)  => {},
            Err(e) => continue, // this is an invalid line
        }

        if line.starts_with("newmtl ") { //@assume: this is new geometry
            if let Some(name) = tokens.next() {
                material_index = result.len();
                result.push(ObjMaterial::default());

                result[material_index].name = String::from(name);
            }
        }
        else if line.starts_with("Ns ") {
            result[material_index].specular_exponent = parse_float(&mut tokens);
        }
        else if line.starts_with("Ka ") {
            result[material_index].ambient_color = parse_float3(&mut tokens);
        }
        else if line.starts_with("Kd ") {
            result[material_index].diffuse_color = parse_float3(&mut tokens);
        }
        else if line.starts_with("Ks ") {
            result[material_index].specular_color = parse_float3(&mut tokens);
        }
        else if line.starts_with("Ni ") {
            result[material_index].index_of_refraction = parse_float(&mut tokens);
        }
        else if line.starts_with("d ") { // todo: how to handle transparency?
            result[material_index].dissolve = parse_float(&mut tokens);
        }
        else if line.starts_with("illum ") {
            result[material_index].illumination_mode = parse_int(&mut tokens);
        }
        else if line.starts_with("map_Kd ") {
            if let Some(name) = tokens.next() {
                result[material_index].diffuse_map = parent_path.join(String::from(name));
            }
        }
        else if line.starts_with("map_Disp ") {
            if let Some(name) = tokens.next() {
                result[material_index].displacement_map = parent_path.join(String::from(name));
            }
        }
        else if line.starts_with("map_Ka ") {
            if let Some(name) = tokens.next() {
                result[material_index].ambient_map = parent_path.join(String::from(name));
            }
        }
    }

    return result;
}

fn parse_obj_file(file: &str, directory_path: &Path) -> ObjModel {
    let mut result = ObjModel::default();
    let mut geom_idx = 0;

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

        if line.starts_with('o') { //@assume: this is new geometry
            if let Some(name) = tokens.next() {
                geom_idx = result.geoms.len();
                result.geoms.push(ObjGeometry::default());

                result.geoms[geom_idx].name = String::from(name);
            }
        }
        else if line.starts_with("vt ") { // texture vertex
            result.tex_coords.push(parse_float2(&mut tokens));
        }
        else if line.starts_with("vn ") { // normal vertex
            result.normals.push(parse_float3(&mut tokens));
        }
        else if line.starts_with("v ") { // geometric vertex
            result.positions.push(parse_float3(&mut tokens));
        }
        else if line.starts_with("s ") { // smooth shading
            //todo: smooth shading.
        }
        else if line.starts_with('f') { // vertex face, @assume: 3 vertices per face
            let vert_count = tokens.clone().count();
            assert!(vert_count == 3, "Only supports tris. Found: {} vertices. {}", vert_count, line);

            let v0 = parse_obj_face_idx(&mut tokens);
            let v1 = parse_obj_face_idx(&mut tokens);
            let v2 = parse_obj_face_idx(&mut tokens);
            result.geoms[geom_idx].faces.push(ObjFace{ v0, v1, v2 });
        }
        else if line.starts_with("mtllib ") {
            let material_file = if let Some(name) = tokens.next() {
                String::from(name)
            } else {
                panic!("mtllib filename missing!");
            };

            result.materials = parse_obj_material_file(material_file.as_str(), &directory_path);
        }
        else if line.starts_with("usemtl ") {
            let index = if let Some(name) = tokens.next() {
                let mut result_idx: Option<usize> = None;

                for i in 0..result.materials.len() {
                    if result.materials[i].name == name {
                        result_idx = Some(i);
                        break;
                    }
                }

                result_idx
            } else { None };

            result.geoms[geom_idx].material_index = index;
        }
        else {
            //note: unsupported token, return an error?
        }
    }

    return result;
}

fn make_vertex_from_face(model: &ObjModel, obj: &ObjGeometry, face: ObjFaceIdx) -> Vertex {
    let position: Float3 = {
        if let Some(idx) = face.pos {
            if model.positions.len() > 0 {
                let mut index = idx as usize;
                if idx < 0 {
                    index = ((model.positions.len() as i32) + idx) as usize;
                }

                if index < model.positions.len() {
                    model.positions[index]
                } else { Float3::zero() }
            } else { Float3::zero() }
        } else { Float3::zero() }
    };

    let tex_coord: Float2 = {
        if let Some(idx) = face.uv0 {
            if model.tex_coords.len() > 0 {
                let mut index = idx as usize;
                if idx < 0 {
                    index = ((model.tex_coords.len() as i32) + idx) as usize;
                }

                if index < model.tex_coords.len() {
                    model.tex_coords[index]
                } else { Float2::zero() }

            } else { Float2::zero() }
        } else { Float2::zero() }
    };

    let normal: Float3 = {
        if let Some(idx) = face.norm {
            if model.normals.len() > 0 {
                let mut index = idx as usize;
                if idx < 0 {
                    index = ((model.normals.len() as i32) + idx) as usize;
                }

                if index < model.normals.len() {
                    model.normals[index]
                } else { Float3::zero() }
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

fn convert_obj_file(model: &ObjModel) -> ChibiImportMesh {
    let mut unopt_vertex_count = 0;
    let mut vertex_count = 0;

    let mut unopt_index_count = 0;
    let mut opt_index_count = 0;

    let mut result = ChibiImportMesh{ geoms: Vec::new(), materials: Vec::new() };

    for obj_mat in &model.materials {
        result.materials.push(ChibiImportMaterial{
            ambient_color: obj_mat.ambient_color,
            ambient_map:   obj_mat.ambient_map.clone(),
        });
    }

    for obj in &model.geoms {
        let mut chibi_geom = ChibiImportGeometry{
            vertices:       Vec::new(),
            indices:        Vec::new(),
            material_index: obj.material_index,
        };

        assert!(model.positions.len() > 0);
        chibi_geom.vertices.reserve(obj.faces.len() * 3);

        let mut index_count: u32 = 0;
        for face in &obj.faces {
            //note: this might result in duplicated geometry

            let v0 = make_vertex_from_face(&model, &obj, face.v0);
            let v1 = make_vertex_from_face(&model, &obj, face.v1);
            let v2 = make_vertex_from_face(&model, &obj, face.v2);

            chibi_geom.vertices.push(v0);
            chibi_geom.vertices.push(v1);
            chibi_geom.vertices.push(v2);

            chibi_geom.indices.push(index_count + 0);
            chibi_geom.indices.push(index_count + 1);
            chibi_geom.indices.push(index_count + 2);

            index_count += 3;
        }

        unopt_vertex_count += chibi_geom.vertices.len();
        unopt_index_count  += chibi_geom.indices.len();

        unsafe { // Optimize Index Buffer and remap Vertices
            let mut remapped: Vec<u32> = Vec::new();
            remapped.resize(chibi_geom.indices.len(), 0);

            let vertex_count = meshopt_generateVertexRemap(
                remapped.as_mut_ptr(),
                chibi_geom.indices.as_ptr(),
                chibi_geom.indices.len(),
                chibi_geom.vertices.as_ptr() as *const c_void,
                chibi_geom.vertices.len(),
                std::mem::size_of::<Vertex>(),
            );

            let mut vertices: Vec<Vertex> = Vec::with_capacity(vertex_count);
            vertices.resize(vertex_count, Vertex::default());

            let mut indices: Vec<u32> = Vec::with_capacity(remapped.len());
            indices.resize(remapped.len(), 0);

            meshopt_remapIndexBuffer(indices.as_mut_ptr(), chibi_geom.indices.as_ptr(), chibi_geom.indices.len(), remapped.as_ptr());
            meshopt_remapVertexBuffer(
                vertices.as_mut_ptr()        as *mut   c_void,
                chibi_geom.vertices.as_ptr() as *const c_void,
                chibi_geom.vertices.len(),
                std::mem::size_of::<Vertex>(),
                remapped.as_ptr());

            chibi_geom.indices  = indices;
            chibi_geom.vertices = vertices;
        }

        unsafe  { // Vertex Cache Optimizations
            meshopt_optimizeVertexCache(
                chibi_geom.indices.as_mut_ptr(),
                chibi_geom.indices.as_ptr(),
                chibi_geom.indices.len(),
                chibi_geom.vertices.len());
        }

        unsafe { //overdraw optimization
            // The algorithm tries to maintain a balance between vertex cache efficiency and overdraw; the threshold determines how much the
            // algorithm can compromise the vertex cache hit ratio, with 1.05 meaning that the resulting ratio should be at most 5% worse
            // than before the optimization.
            meshopt_optimizeOverdraw(
                chibi_geom.indices.as_mut_ptr(),
                chibi_geom.indices.as_ptr(),
                chibi_geom.indices.len(),
                chibi_geom.vertices.as_ptr() as *const f32, //note: assume that the position is the first element in a vertex
                chibi_geom.vertices.len(),
                std::mem::size_of::<Vertex>(),
                1.05);
        }

        unsafe { // Vertex fetch optimization
            meshopt_optimizeVertexFetch(
                chibi_geom.vertices.as_mut_ptr() as *mut c_void,
                chibi_geom.indices.as_mut_ptr(),
                chibi_geom.indices.len(),
                chibi_geom.vertices.as_ptr() as *const c_void,
                chibi_geom.vertices.len(),
                std::mem::size_of::<Vertex>());
        }

        //todo: vertex quantization
        //todo: Shadow indexing
        //todo: Vertex/index buffer compression

        vertex_count    += chibi_geom.vertices.len();
        opt_index_count += chibi_geom.indices.len();

        result.geoms.push(chibi_geom);
    }

    println!("Parsed Obj File.
        \tGeometry Count({}),
        \tOriginal Vertex Count({}), Optimized Vertex Count({})
        \tOriginal Index Count({}), Optimizaed Index Count({})",
        result.geoms.len(),
        unopt_vertex_count, vertex_count,
        unopt_index_count, opt_index_count
    );

    return result;
}

pub fn import_obj_file(asset_path: &PathBuf, name: &str) -> ChibiImportMesh {
    let mut name_str = String::from_str(name).expect("Failed to construct string.");
    name_str.push_str(".obj");

    let mesh_file = asset_path.join(name_str);
    let mesh_fullpath = mesh_file.parent().expect("Failed to acquire parent directory to obj file");

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

    let parsed_obj = parse_obj_file(&file_str, &mesh_fullpath);
    return convert_obj_file(&parsed_obj);
}
