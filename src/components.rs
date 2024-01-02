use std::marker::PhantomData;

use bevy::prelude::*;

use crate::prelude::ParameterizedShader;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ShaderShape<SHADER: ParameterizedShader> {
    phantom: PhantomData<SHADER>,
}

impl<SHADER: ParameterizedShader> std::fmt::Debug for ShaderShape<SHADER> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderShape").finish()
    }
}

impl<SHADER: ParameterizedShader> Eq for ShaderShape<SHADER> {}

impl<SHADER: ParameterizedShader> PartialEq for ShaderShape<SHADER> {
    fn eq(&self, other: &Self) -> bool {
        self.phantom == other.phantom
    }
}

impl<SHADER: ParameterizedShader> Copy for ShaderShape<SHADER> {}

impl<SHADER: ParameterizedShader> Clone for ShaderShape<SHADER> {
    fn clone(&self) -> Self {
        Self {
            phantom: self.phantom.clone(),
        }
    }
}

impl<SHADER: ParameterizedShader> Default for ShaderShape<SHADER> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// Bounds for describing how far the fragment shader of a shape will reach, should be bigger than the shape unless you want to clip it
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub half_width: f32,
    pub half_height: f32, // todo: it probably makes sense for this to be the full width instead...
}

impl Frame {
    const DEFAULT: Self = Self::square(1.0);

    pub const fn square(radius: f32) -> Self {
        Self {
            half_height: radius,
            half_width: radius,
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::DEFAULT
    }
}
