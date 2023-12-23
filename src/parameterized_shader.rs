use bevy::{
    reflect::{GetTypeRegistration, Reflect, Struct, TypeUuid},
    render::render_resource::{Shader, VertexFormat},
};
use bytemuck::{Pod, Zeroable};

pub trait ParameterizedShader:
    Pod
    + Zeroable
    + Copy
    + Sync
    + Send
    + std::fmt::Debug
    + Default
    + TypeUuid
    + Reflect
    + Struct
    + GetTypeRegistration
    + 'static
{
    /// Get the body of the fragment shader fragment function
    /// This will take an `in` argument with a `pos` parameter and one parameter for each field
    /// It should return `vec4<f32>` representing the color of the pixel
    fn fragment_body<'a>() -> &'a str;

    /// Get the text of any helper functions.
    fn fragment_helpers<'a>() -> &'a str;

    // const PARAM_COUNT: u32; //TODO use bevy reflect instead of this
    // fn get_format(index: u32) -> VertexFormat;
}

// pub(crate) fn format_to_name(f: VertexFormat) -> &'static str {
//     match f {
//         VertexFormat::Uint32 => "u32",
//         VertexFormat::Float32 => "f32",
//         _ => todo!("Other format names"),
//     }
// }
