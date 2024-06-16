use std::fmt::Debug;

use bevy::prelude::*;

use crate::{parameterized_shader::*, ShaderUsage};

#[derive(Bundle)]
pub struct ShaderBundle<Extract: ExtractToShader> {
    pub parameters: <Extract as ExtractToShader>::ParamsBundle,
    /// A transform, set this to set the position, orientation and scale of the shape
    ///
    /// note: scaling the shape with the transform will also scale the fill, including any outlines etc.
    pub transform: Transform,
    /// Indicates the shader to use
    pub shape: ShaderUsage<Extract>,
    /// A compute transform
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// The inherited visibility of the entity.
    pub inherited_visibility: InheritedVisibility,
    /// The view visibility of the entity.
    pub view_visibility: ViewVisibility,
}

impl<Extract: ExtractToShader<ParamsBundle: Debug>> Debug for ShaderBundle<Extract> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderBundle").field("parameters", &self.parameters).field("transform", &self.transform).field("shape", &self.shape).field("global_transform", &self.global_transform).field("visibility", &self.visibility).field("inherited_visibility", &self.inherited_visibility).field("view_visibility", &self.view_visibility).finish()
    }
}

impl<Extract: ExtractToShader<ParamsBundle: Clone>> Clone for ShaderBundle<Extract> {
    fn clone(&self) -> Self {
        Self {
            parameters: self.parameters.clone(),
            transform: self.transform,
            shape: self.shape,
            global_transform: self.global_transform,
            visibility: self.visibility,
            inherited_visibility: self.inherited_visibility,
            view_visibility: self.view_visibility,
        }
    }
}

impl<Extract: ExtractToShader<ParamsBundle: PartialEq>> PartialEq for ShaderBundle<Extract> {
    fn eq(&self, other: &Self) -> bool {
        self.parameters == other.parameters
            && self.transform == other.transform
            && self.shape == other.shape
            && self.global_transform == other.global_transform
            && self.visibility == other.visibility
            && self.inherited_visibility == other.inherited_visibility
            && self.view_visibility == other.view_visibility
    }
}

impl<Extract: ExtractToShader<ParamsBundle: Eq>> Eq for ShaderBundle<Extract> {}

impl<Extract: ExtractToShader> Default for ShaderBundle<Extract>
where
    <Extract as ExtractToShader>::ParamsBundle: Default,
{
    fn default() -> Self {
        Self {
            parameters: Default::default(),
            transform: Default::default(),
            shape: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            inherited_visibility: Default::default(),
            view_visibility: Default::default(),
        }
    }
}
