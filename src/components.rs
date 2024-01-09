use std::marker::PhantomData;

use bevy::prelude::*;

use crate::prelude::ExtractToShader;

/// Indicates that a particular shader should be used to draw this entity.
/// The entity may need additional components to be extracted for drawing
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ShaderUsage<Extract: ExtractToShader> {
    phantom: PhantomData<Extract>,
}

impl<Extract: ExtractToShader> std::fmt::Debug for ShaderUsage<Extract> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderUsage").finish()
    }
}

impl<Extract: ExtractToShader> Eq for ShaderUsage<Extract> {}

impl<Extract: ExtractToShader> PartialEq for ShaderUsage<Extract> {
    fn eq(&self, other: &Self) -> bool {
        self.phantom == other.phantom
    }
}

impl<Extract: ExtractToShader> Copy for ShaderUsage<Extract> {}

impl<Extract: ExtractToShader> Clone for ShaderUsage<Extract> {
    fn clone(&self) -> Self {
        Self {
            phantom: self.phantom.clone(),
        }
    }
}

impl<Extract: ExtractToShader> Default for ShaderUsage<Extract> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

