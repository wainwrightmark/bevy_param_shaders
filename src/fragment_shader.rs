use bevy::render::render_resource::Shader;

use crate::prelude::ParameterizedShader;

pub(crate) fn create_fragment_shader<PARAMETERS: ParameterizedShader>() -> Shader {
    let params_locations = crate::vertex_shader::format_params_locations::<PARAMETERS>();

    let fragment_body = PARAMETERS::fragment_body();
    let helpers = PARAMETERS::fragment_helpers();

    let source = format!(
        r#"
#import bevy_render::globals::Globals
@group(0) @binding(1)
var<uniform> globals: Globals;

struct FragmentInput {{
@location(0) pos: vec2<f32>,
{params_locations}
}};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {{
    {fragment_body}
}}

{helpers}
"#
    );
    //bevy::log::info!("{source}");

    let tn = PARAMETERS::TYPE_UUID;

    let generated_shader = Shader::from_wgsl(source, format!("fragment_{tn}"));

    return generated_shader;
}
