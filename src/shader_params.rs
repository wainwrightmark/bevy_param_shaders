use bevy::reflect::{Reflect, Struct};
use bytemuck::{Pod, Zeroable};
pub trait ShaderParams:
    Pod + Zeroable + Copy + std::fmt::Debug + Default + Reflect + Struct
{
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
        Self { red, green, blue }
    }
}

impl From<bevy::prelude::Color> for ColorParams {
    fn from(value: bevy::prelude::Color) -> Self {
        Self {
            color: value.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct ColorParams {
    pub color: LinearRGBA,
}

impl ShaderParams for ColorParams {}
