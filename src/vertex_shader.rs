use bevy::{reflect::Struct, render::render_resource::Shader};

use crate::parameterized_shader::*;

/// Creates a vertex shader with the correct number of arguments
pub(crate) fn create_vertex_shader<SHADER: ParameterizedShader>() -> Shader {
    // TODO Create this string at compile time?

    let proxy = <SHADER::Params as Default>::default();

    let param_count = proxy.field_len();
    let rotation_location = (param_count) + 1;
    let scale_location = (param_count) + 2;
    let frame_location = (param_count) + 3;

    let params_locations = crate::helpers::format_params_locations::<SHADER::Params>();

    let mut params_assignments = "".to_string();
    for index in 0..param_count {
        let name = proxy.name_at(index).unwrap();
        params_assignments.push_str(format!("    out.{name} = vertex.{name};\n").as_str());
    }

    let params_id = SHADER::TYPE_UUID;

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
