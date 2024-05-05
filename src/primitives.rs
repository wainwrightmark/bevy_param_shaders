use crate::prelude::*;
use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bytemuck::Pod;
use bytemuck::Zeroable;
use std::fmt::Debug;

pub struct PrimitivesPlugin;

impl Plugin for PrimitivesPlugin {
    fn build(&self, app: &mut App) {

        embedded_asset!(app, "", "shaders/fill/fill_with_border.wgsl");
        embedded_asset!(app, "", "shaders/fill/simple.wgsl");
        embedded_asset!(app, "", "shaders/sdf/rounded_rect.wgsl");
        embedded_asset!(app, "", "shaders/sdf/circle.wgsl");

        app.add_plugins(ExtractToShaderPlugin::<RoundedRectShaderExtraction>::default());
        app.add_plugins(ExtractToShaderPlugin::<RoundedRectWithBorderShader>::default());
        app.add_plugins(ExtractToShaderPlugin::<CircleShader>::default());


    }
}

#[derive(Debug, Clone, Copy, TypePath, Default, PartialEq)]
pub struct RoundedRectShader;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RoundedRectShaderExtraction;

impl ExtractToShader for RoundedRectShaderExtraction {
    type Shader = RoundedRectShader;
    type ParamsQuery<'a> = (&'a ShaderColor, &'a ShaderRounding, &'a ShaderProportions);
    type ParamsBundle = (ShaderColor, ShaderRounding, ShaderProportions);
    type ResourceParams<'w> = ();

    fn get_params(
        query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        _resource: &<Self::ResourceParams<'_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> <Self::Shader as ParameterizedShader>::Params {
        RoundedRectShaderParams {
            color: query_item.0.color.into(),
            rounding: query_item.1.rounding,
            height: query_item.2.height,
            width: query_item.2.width,
        }
    }
}

impl ParameterizedShader for RoundedRectShader {
    type Params = RoundedRectShaderParams;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "bps::rounded_rect::sdf(in.pos, in.width, in.height, in.rounding)",
            fill_color: "bps::simple_fill::fill(d, in.color, in.pos)",
        }
    }

    fn frame_expression() -> impl Into<String> {
        Frame::square(1.0)
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [imports::fill::SIMPLE_FILL, imports::sdf::ROUNDED_RECT].into_iter()
    }

    const UUID: u128 = 0xa31d800c02a24db78aaf1caa2bd1dc37;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct RoundedRectShaderParams {
    pub color: LinearRGBA,
    // Width as a proportion of scale in range 0..=1.0
    pub width: f32,

    // Height as a proportion of scale in range 0..=1.0
    pub height: f32,
    pub rounding: f32,
}

impl ShaderParams for RoundedRectShaderParams {}

#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub struct ShaderColor {
    pub color: Color,
}

impl From<Color> for ShaderColor {
    fn from(color: Color) -> Self {
        Self { color }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub struct ShaderSecondColor {
    pub color: Color,
}

impl From<Color> for ShaderSecondColor {
    fn from(color: Color) -> Self {
        Self { color }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct ShaderRounding {
    pub rounding: f32,
}

impl From<f32> for ShaderRounding {
    fn from(rounding: f32) -> Self {
        Self { rounding }
    }
}
impl Default for ShaderRounding {
    fn default() -> Self {
        Self { rounding: 0.0 }
    }
}

/// height / width
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct ShaderProportions {
    /// width in range 0..=1.0
    pub width: f32,
    /// height in range 0..=1.0
    pub height: f32,
}

impl Default for ShaderProportions {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
        }
    }
}

impl From<Vec2> for ShaderProportions {
    fn from(value: Vec2) -> Self {
        Self {
            width: value.x,
            height: value.y,
        }
    }
}

#[derive(Debug, Clone, Copy, TypePath, Default, PartialEq)]
pub struct RoundedRectWithBorderShader;

impl ExtractToShader for RoundedRectWithBorderShader {
    type Shader = Self;

    type ParamsQuery<'a> = (
        &'a ShaderColor,
        &'a ShaderRounding,
        &'a ShaderProportions,
        &'a ShaderBorder,
    );
    type ParamsBundle = (ShaderColor, ShaderRounding, ShaderProportions, ShaderBorder);
    type ResourceParams<'w> = ();

    fn get_params(
        query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        _resource: &<Self::ResourceParams<'_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> <Self::Shader as ParameterizedShader>::Params {
        RoundedRectWithBorderShaderParams {
            color: query_item.0.color.into(),
            rounding: query_item.1.rounding,

            width: query_item.2.width,
            height: query_item.2.height,
            border_color: query_item.3.border_color.into(),
            border: query_item.3.border,
        }
    }
}

impl ParameterizedShader for RoundedRectWithBorderShader {
    type Params = RoundedRectWithBorderShaderParams;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "bps::rounded_rect::sdf(in.pos, in.width, in.height, in.rounding)",
            fill_color: "bps::fill_with_border::fill(d, in.color, in.border, in.border_color)",
        }
    }

    fn frame_expression() -> impl Into<String> {
        Frame::square(1.0)
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [imports::fill::FILL_WITH_OUTLINE, imports::sdf::ROUNDED_RECT].into_iter()
    }

    const UUID: u128 = 0xdf3562db60d2471a81ac616fb633c7e7;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct RoundedRectWithBorderShaderParams {
    pub width: f32,
    pub height: f32,
    pub rounding: f32,

    pub color: LinearRGBA,
    pub border_color: LinearRGBA,
    pub border: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub struct ShaderBorder {
    pub border_color: Color,
    pub border: f32,
}

impl ShaderBorder {
    pub const NONE: Self = ShaderBorder {
        border_color: Color::NONE,
        border: 0.0,
    };

    pub fn from_color(color: Color) -> Self {
        Self {
            border_color: color,
            border: 0.005,
        }
    }
}

impl ShaderParams for RoundedRectWithBorderShaderParams {}

#[derive(Debug, TypePath, Default, PartialEq, Clone, Copy)]

pub struct CircleShader;

impl ExtractToShader for CircleShader {
    type Shader = Self;
    type ParamsQuery<'a> = &'a ShaderColor;
    type ParamsBundle = ShaderColor;
    type ResourceParams<'w> = ();

    fn get_params(
        query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        _resource: &<Self::ResourceParams<'_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> <Self::Shader as ParameterizedShader>::Params {
        ColorParams {
            color: query_item.color.into(),
        }
    }
}

impl ParameterizedShader for CircleShader {
    type Params = ColorParams;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "bps::circle::sdf(in.pos)",
            fill_color: "bps::simple_fill::fill(d, in.color, in.pos)",
        }
    }

    fn frame_expression() -> impl Into<String> {
        Frame::square(1.0)
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [imports::fill::SIMPLE_FILL, imports::sdf::CIRCLE].into_iter()
    }

    const UUID: u128 = 0x9a8df8ca0f854ccfb0a3ad366a6e8b4b;
}


pub mod imports {

    pub mod fill {
        use crate::FragmentImport;
        pub const SIMPLE_FILL: FragmentImport = FragmentImport {
            path: "embedded://bevy_param_shaders/shaders/fill/simple.wgsl",
            import_path: "bps::simple_fill",
        };

        pub const FILL_WITH_OUTLINE: FragmentImport = FragmentImport {
            path: "embedded://bevy_param_shaders/shaders/fill/fill_with_border.wgsl",
            import_path: "bps::fill_with_border",
        };
    }

    pub mod sdf {
        use crate::FragmentImport;
        pub const CIRCLE: FragmentImport = FragmentImport {
            path: "embedded://bevy_param_shaders/shaders/sdf/circle.wgsl",
            import_path: "bps::circle",
        };

        pub const ROUNDED_RECT: FragmentImport = FragmentImport {
            path: "embedded://bevy_param_shaders/shaders/sdf/rounded_rect.wgsl",
            import_path: "bps::rounded_rect",
        };
    }
}
