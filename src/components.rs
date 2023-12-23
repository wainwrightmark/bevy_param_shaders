use bevy::prelude::*;

use crate::prelude::ParameterizedShader;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
/// Describes an SDF shape. Must be used with `SmudShaders`
pub struct ShaderShape<PARAMETERS: ParameterizedShader> {
    /// The outer bounds for the shape, should be bigger than the sdf shape
    pub frame: Frame,
    pub parameters: PARAMETERS,
}

impl<PARAMETERS: ParameterizedShader> Default for ShaderShape<PARAMETERS> {
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
