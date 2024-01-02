use bevy::prelude::*;

use crate::prelude::ParameterizedShader;

#[derive(Component, Reflect, Debug, Clone, PartialEq)]
#[reflect(Component)]
pub struct ShaderShape<SHADER: ParameterizedShader> {
    /// The outer bounds for the shape, should be bigger than the shape
    pub frame: Frame,
    pub parameters: SHADER::Params,
}

impl<SHADER: ParameterizedShader> Default for ShaderShape<SHADER> {
    fn default() -> Self {
        Self {
            frame: Default::default(),
            parameters: Default::default(),
        }
    }
}

/// Bounds for describing how far the fragment shader of a shape will reach, should be bigger than the shape unless you want to clip it
#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct  Frame {
    pub half_width: f32,
    pub half_height: f32// todo: it probably makes sense for this to be the full width instead...
}

impl Frame {
    const DEFAULT: Self = Self::square(1.0);

    pub const  fn square(radius: f32)-> Self{
        Self{
            half_height: radius,
            half_width: radius
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::DEFAULT
    }
}
