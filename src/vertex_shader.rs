use bevy::reflect::Struct;

use crate::parameterized_shader::*;

/// Creates a vertex shader with the correct number of arguments
pub(crate) fn create_vertex_shader<Shader: ParameterizedShader>() -> bevy::render::render_resource::Shader {
    // TODO Create this string at compile time?

    let proxy = <Shader::Params as Default>::default();

    let param_count = proxy.field_len();

    let vertex_params_locations = crate::helpers::format_params_locations::<Shader::Params>(3);
    let fragment_params_locations = crate::helpers::format_params_locations::<Shader::Params>(1);

    let mut params_assignments = "".to_string();
    for index in 0..param_count {
        let name = proxy.name_at(index).unwrap();
        params_assignments.push_str(format!("    out.{name} = vertex.{name};\n").as_str());
    }

    let tp = Shader::type_path();

    let frame_expression: String = Shader::frame_expression().into();

    let source = format!(
        r##"
#define_import_path param_shaders::vertex_params_{tp}

struct View {{
    view_proj: mat4x4<f32>,
    world_position: vec3<f32>,
}};
@group(0) @binding(0)
var<uniform> view: View;

// as specified in `specialize()`
struct Vertex {{
@location(0) rotation: vec2<f32>,
@location(1) position: vec3<f32>,
@location(2) scale: f32,
{vertex_params_locations}
}};



struct VertexOutput {{
@builtin(position) clip_position: vec4<f32>,
@location(0) pos: vec2<f32>,
{fragment_params_locations}
}};

@vertex
fn vertex(
    vertex: Vertex,
    @builtin(vertex_index) i: u32
) -> VertexOutput {{
var out: VertexOutput;
let frame = {frame_expression};

let x = select(-1., 1., i % 2u == 0u);
let y = select(-1., 1., (i / 2u) % 2u == 0u);
let c = vertex.rotation.x;
let s = vertex.rotation.y;
let rotated = vec2<f32>(x * c - y * s, x * s + y * c);
let pos = vertex.position + vec3<f32>(rotated * vertex.scale * frame, vertex.position.z);
// Project the world position of the mesh into screen position
out.clip_position = view.view_proj * vec4<f32>(pos, 1.);
{params_assignments}
out.pos = vec2<f32>(x, y) * frame;
return out;
}}
"##
    );
    //bevy::log::info!("{source}");
    let path = file!();
    bevy::render::render_resource::Shader::from_wgsl(source, path)
}
