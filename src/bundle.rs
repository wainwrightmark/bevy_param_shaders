use bevy::prelude::*;

use crate::{parameterized_shader::*, ShaderUsage};

#[derive(Bundle, Clone, Debug, PartialEq)]
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

impl<Extract: ExtractToShader> Default for ShaderBundle<Extract>
where <Extract as ExtractToShader>::ParamsBundle : Default
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
