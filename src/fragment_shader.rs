use crate::prelude::ParameterizedShader;

pub(crate) fn create_fragment_shader<Shader: ParameterizedShader>() -> bevy::render::render_resource::Shader {
    let params_locations = crate::helpers::format_params_locations::<Shader::Params>(1);

    let fragment_body: String = Shader::fragment_body().into();

    let imports = Shader::imports()
        .map(|x| format!("#import {}", x.import_path))
        .collect::<Vec<String>>()
        .join("\n");

    let (time_import, time_group) = if Shader::USE_TIME {
        (
            "#import bevy_render::globals::Globals",
            "@group(0) @binding(1)
        var<uniform> globals: Globals;",
        )
    } else {
        ("", "")
    };

    let source = format!(
        r#"
{time_import}
{imports}

{time_group}

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

    let tn = Shader::TYPE_UUID;

    bevy::render::render_resource::Shader::from_wgsl(source, format!("fragment_{tn}"))
}
