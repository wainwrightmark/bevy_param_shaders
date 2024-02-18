use std::f32::consts;

use bevy::{prelude::*, reflect::TypeUuid};
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::prelude::*;

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            ExtractToShaderPlugin::<ExtractColorToSquare>::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

pub struct ExtractColorToSquare;

impl ExtractToShader for ExtractColorToSquare {
    type Shader = SquareShader;
    type ParamsQuery<'a> = &'a ColorParams;
    type ParamsBundle = ColorParams;
    type ResourceParams<'a> = ();

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
        _r: &(),
    ) -> <Self::Shader as ParameterizedShader>::Params {
        *query_item
    }
}


#[derive(Debug, TypePath, Default)]

pub struct SquareShader;

impl ParameterizedShader for SquareShader {
    type Params = ColorParams;

    fn fragment_body() -> impl Into<String> {
        "return in.color;"
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [].into_iter()
    }

    const FRAME: Frame = Frame {
        half_width: 100.0,
        half_height: 50.0,
    };

    const UUID: u128 = 0x6d31023450194cd49f60ebabd7dca30b;
}

#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable, Component,
)]
pub struct ColorParams {
    pub color: LinearRGBA,
}

impl ShaderParams for ColorParams {}

impl From<bevy::prelude::Color> for ColorParams {
    fn from(value: bevy::prelude::Color) -> Self {
        Self {
            color: value.into(),
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(ShaderBundle::<ExtractColorToSquare> {
        parameters: Color::ORANGE_RED.into(),

        ..default()
    });

    commands.spawn(ShaderBundle::<ExtractColorToSquare> {
        parameters: Color::BLUE.with_a(0.8).into(),
        transform: Transform::from_rotation(Quat::from_rotation_z(consts::FRAC_PI_4)),
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}
