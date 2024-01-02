use std::fmt::Debug;

use crate::{shader_params::ShaderParams, Frame};
use bevy::{
    ecs::{
        bundle::Bundle,
        query::{ReadOnlyWorldQuery, WorldQuery},
    },
    reflect::TypeUuid,
};

pub trait ParameterizedShader: Sync + Send + TypeUuid + 'static {
    type Params: ShaderParams;
    type ParamsQuery<'a>: ReadOnlyWorldQuery;
    //TODO additional type param for required resources

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as WorldQuery>::Item<'w>,
    ) -> Self::Params;

    /// Get the body of the fragment shader fragment function
    /// This will take an `in` argument with a `pos` parameter and one parameter for each field
    /// It should return `vec4<f32>` representing the color of the pixel
    fn fragment_body() -> impl Into<String>;

    /// Get imports
    fn imports() -> impl Iterator<Item = FragmentImport>;

    const USE_TIME: bool = false;

    /// The frame to use for this shader
    const FRAME: Frame;
}

pub trait BundlableParameterizedShader {
    /// A bundle of the additional parameters needed to use this shader
    type ParamsBundle: Bundle + Default + Clone + Debug + PartialEq;
}

impl<'a, P : Bundle + Default + Clone + Debug + PartialEq, T : ParameterizedShader<ParamsQuery<'a> = &'a P, Params = P>>  BundlableParameterizedShader for T{
    type ParamsBundle = P;
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

impl Into<String> for SDFColorCall {
    fn into(self) -> String {
        let SDFColorCall { sdf, fill_color } = self;
        format!(
            r#"let d = {sdf};
        let c = {fill_color};
        return c;
        "#
        )
    }
}
