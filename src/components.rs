use bevy::prelude::*;

use crate::prelude::ParameterizedShader;

#[derive(Component, Reflect, Debug, Clone)]
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
#[derive(Reflect, Debug, Clone, Copy)]
pub enum Frame {
    /// A quad with a given half-size (!)
    Quad(f32), // todo: it probably makes sense for this to be the full width instead...
}

impl Frame {
    const DEFAULT_QUAD: Self = Self::Quad(1.);
}

impl Default for Frame {
    fn default() -> Self {
        Self::DEFAULT_QUAD
    }
}
