use bevy::render::render_resource::Shader;

use crate::prelude::ParameterizedShader;

pub(crate) fn create_fragment_shader<SHADER: ParameterizedShader>() -> Shader {
    let params_locations = crate::helpers::format_params_locations::<SHADER::Params>();

    let fragment_body: String = SHADER::fragment_body().into();

    let imports = SHADER::imports()
        .map(|x| format!("#import {}", x.import_path))
        .collect::<Vec<String>>()
        .join("\n");

    let source = format!(
        r#"
#import bevy_render::globals::Globals
{imports}

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


"#
    );
    //bevy::log::info!("{source}");

    let tn = SHADER::TYPE_UUID;

    Shader::from_wgsl(source, format!("fragment_{tn}"))
}
