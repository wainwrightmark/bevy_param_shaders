use std::fmt::Debug;

use crate::shader_params::ShaderParams;
use bevy::{
    ecs::{
        bundle::Bundle,
        query::{ReadOnlyQueryData, WorldQuery},
        system::{ReadOnlySystemParam, SystemParam},
    },
    reflect::TypePath,
};

/// A set of parameters that will extracted to ShaderParams and drawn with a particular shader
pub trait ExtractToShader: Sync + Send + 'static {
    type Shader: ParameterizedShader;
    type ParamsQuery<'a>: ReadOnlyQueryData;
    type ParamsBundle: Bundle;
    type ResourceParams<'w>: SystemParam + ReadOnlySystemParam;

    fn get_params(
        query_item: <Self::ParamsQuery<'_> as WorldQuery>::Item<'_>,
        resource: & <Self::ResourceParams<'_> as SystemParam>::Item<'_, '_>,
    ) -> <Self::Shader as ParameterizedShader>::Params;
}

/// A particular shader
pub trait ParameterizedShader: Sync + Send + TypePath + 'static {
    type Params: ShaderParams;

    /// Get the body of the fragment shader fragment function
    /// This will take an `in` argument with a `pos` parameter and one parameter for each field
    /// It should return `vec4<f32>` representing the color of the pixel
    fn fragment_body() -> impl Into<String>;

    /// An expression that returns a `vec2<f32>` representing the half-width and half-height of the  frame
    fn frame_expression() -> impl Into<String>;

    /// Get imports
    fn imports() -> impl Iterator<Item = FragmentImport>;

    const USE_TIME: bool = false;

    const UUID: u128;
}

pub struct FragmentImport {
    pub path: &'static str,
    pub import_path: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct SDFAlphaCall {
    /// An 'f32' expression
    pub sdf: &'static str,

    /// An 'f32' expression
    /// This may use the 'd' parameter
    pub fill_alpha: &'static str,

    /// A 'vec4<f32>' expression
    pub color: &'static str,
}

impl From<SDFAlphaCall> for String {
    fn from(val: SDFAlphaCall) -> Self {
        let SDFAlphaCall {
            sdf,
            fill_alpha: fill,
            color,
        } = val;
        format!(
            r#"let d = {sdf};
        let a = {fill};
        let c = {color};
        return vec4<f32>(c.rgb, c.a * a);
        "#
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SDFColorCall {
    /// An 'f32' expression
    pub sdf: &'static str,

    /// A 'vec4<f32>' expression
    /// This may use the 'd' parameter
    pub fill_color: &'static str,
}

impl From<SDFColorCall> for String {
    fn from(val: SDFColorCall) -> Self {
        let SDFColorCall { sdf, fill_color } = val;
        format!(
            r#"let d = {sdf};
        let c = {fill_color};
        return c;
        "#
        )
    }
}
