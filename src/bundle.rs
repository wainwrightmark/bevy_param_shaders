use bevy::prelude::*;

use crate::{parameterized_shader::*, ShaderShape};

#[derive(Bundle, Default, Clone, Debug, PartialEq)]
pub struct ShaderBundle<SHADER: ParameterizedShader> {
    /// The shader to use
    pub shape: ShaderShape<SHADER>,

    pub parameters: <SHADER as ParameterizedShader>::ParamsBundle,

    /// A transform, set this to set the position, orientation and scale of the shape
    ///
    /// note: scaling the shape with the transform will also scale the fill, including any outlines etc.
    pub transform: Transform,
    /// A compute transform
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// The inherited visibility of the entity.
    pub inherited_visibility: InheritedVisibility,
    /// The view visibility of the entity.
    pub view_visibility: ViewVisibility,
}
