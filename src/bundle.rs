use bevy::prelude::*;

use crate::{ShaderShape, parameterized_shader::*};

#[derive(Bundle, Default, Clone, Debug)]
pub struct ShaderBundle<SHADER: ParameterizedShader> {
    /// The shape, which describes the color, frame, and additional parameters
    pub shape: ShaderShape<SHADER>,
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