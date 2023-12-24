use bevy::reflect::{GetTypeRegistration, Reflect, Struct, TypeUuid};
use bytemuck::{Pod, Zeroable};

pub trait ShaderParams:
    Pod + Zeroable + Copy + std::fmt::Debug + Default + Reflect + Struct
{
}

pub trait ParameterizedShader: Sync + Send + TypeUuid + GetTypeRegistration + 'static {
    type Params: ShaderParams;
    /// Get the body of the fragment shader fragment function
    /// This will take an `in` argument with a `pos` parameter and one parameter for each field
    /// It should return `vec4<f32>` representing the color of the pixel
    fn fragment_body<'a>() -> &'a str;

    /// Get the text of any helper functions.
    fn fragment_helpers<'a>() -> &'a str;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect, Pod, Zeroable)]
pub struct NoParams;

impl ShaderParams for NoParams {}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct LinearRGBA {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl From<bevy::prelude::Color> for LinearRGBA {
    fn from(value: bevy::prelude::Color) -> Self {
        let [red, green, blue, alpha] = value.as_linear_rgba_f32();
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct LinearRGB {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl From<bevy::prelude::Color> for LinearRGB {
    fn from(value: bevy::prelude::Color) -> Self {
        let [red, green, blue, _alpha] = value.as_linear_rgba_f32();
        Self {
            red,
            green,
            blue,
        }
    }
}
