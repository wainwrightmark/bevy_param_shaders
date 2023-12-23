use std::any::TypeId;

use bevy::{
    math::Vec4,
    render::render_resource::{Shader, VertexFormat},
};

use crate::parameterized_shader::*;

/// Creates a vertex shader with the correct number of arguments
pub(crate) fn create_vertex_shader<PARAMETERS: ParameterizedShader>() -> Shader {
    // TODO Create this string at compile time?

    let proxy = PARAMETERS::default();

    let param_count = proxy.field_len();
    let rotation_location = (param_count) + 1;
    let scale_location = (param_count) + 2;
    let frame_location = (param_count) + 3;

    let params_locations = format_params_locations::<PARAMETERS>();

    let mut params_assignments = "".to_string();
    for index in 0..param_count {
        let name = proxy.name_at(index).unwrap();
        params_assignments.push_str(format!("    out.{name} = vertex.{name};\n").as_str());
    }

    let params_id = PARAMETERS::TYPE_UUID;

    let source = format!(
        r##"
#define_import_path smud::vertex_params_{params_id}

struct View {{
    view_proj: mat4x4<f32>,
    world_position: vec3<f32>,
}};
@group(0) @binding(0)
var<uniform> view: View;

// as specified in `specialize()`
struct Vertex {{
@location(0) position: vec3<f32>,
{params_locations}
@location({rotation_location}) rotation: vec2<f32>,
@location({scale_location}) scale: f32,
@location({frame_location}) frame: f32,
}};

struct VertexOutput {{
@builtin(position) clip_position: vec4<f32>,
@location(0) pos: vec2<f32>,
{params_locations}
}};

@vertex
fn vertex(
    vertex: Vertex,
    @builtin(vertex_index) i: u32
) -> VertexOutput {{
var out: VertexOutput;
let x = select(-1., 1., i % 2u == 0u);
let y = select(-1., 1., (i / 2u) % 2u == 0u);
let c = vertex.rotation.x;
let s = vertex.rotation.y;
let rotated = vec2<f32>(x * c - y * s, x * s + y * c);
let pos = vertex.position + vec3<f32>(rotated * vertex.scale * vertex.frame, vertex.position.z);
// Project the world position of the mesh into screen position
out.clip_position = view.view_proj * vec4<f32>(pos, 1.);
{params_assignments}
out.pos = vec2<f32>(x, y) * vertex.frame;
return out;
}}
"##
    );
    //bevy::log::info!("{source}");
    let path = file!();
    Shader::from_wgsl(source, path)
}

pub(crate) fn format_params_locations<PARAMETERS: ParameterizedShader>() -> String {
    let mut result = "".to_string();

    let proxy = PARAMETERS::default();
    let param_count = proxy.field_len();

    let mut loc = 1;

    for index in 0..param_count {
        //let t = crate::parameterized_shader::format_to_name(PARAMETERS::get_format(index));
        let name = proxy.name_at(index).unwrap();
        let type_id = proxy.field_at(index).unwrap().type_id();

        let Some(type_name) = get_wgsl_type_name(type_id) else {
            panic!(
                "Cannot convert {} to wgsl type",
                proxy.field_at(index).unwrap().type_name()
            );
        };

        result.push_str(format!("@location({loc}) {name}: {type_name},\n").as_str());
        loc += 1;
    }

    result
}

pub(crate) fn get_wgsl_type_name(type_id: TypeId) -> Option<&'static str> {
    if type_id == TypeId::of::<f32>() {
        Some("f32")
    } else if type_id == TypeId::of::<u32>() {
        Some("u32")
    } else if type_id == TypeId::of::<i32>() {
        Some("i32")
    } else if type_id == TypeId::of::<bevy::math::Vec2>() {
        Some("vec2<f32>")
    } else if type_id == TypeId::of::<bevy::math::Vec3>() {
        Some("vec3<f32>")
    } else if type_id == TypeId::of::<Vec4>() {
        Some("vec4<f32>")
    } else {
        None
    }
}

pub(crate) fn get_vertex_format(type_id: TypeId) -> Option<VertexFormat> {
    if type_id == TypeId::of::<f32>() {
        Some(VertexFormat::Float32)
    } else if type_id == TypeId::of::<u32>() {
        Some(VertexFormat::Uint32)
    } else if type_id == TypeId::of::<i32>() {
        Some(VertexFormat::Sint32)
    } else if type_id == TypeId::of::<bevy::math::Vec2>() {
        Some(VertexFormat::Float32x2)
    } else if type_id == TypeId::of::<bevy::math::Vec3>() {
        Some(VertexFormat::Float32x3)
    } else if type_id == TypeId::of::<Vec4>() {
        Some(VertexFormat::Float32x4)
    } else {
        None
    }
}
