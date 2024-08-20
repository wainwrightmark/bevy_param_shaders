use std::f32::consts;

use bevy::{prelude::*, reflect::TypePath};
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::prelude::*;

/// This example tests that warning messages are logged when you forget to add a shader extraction on debug mode.
fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            ExtractToShaderPlugin::<BlueSquare>::default(),
            //ExtractToShaderPlugin::<RedSquare>::default(), //Whoops we forgot to register this
        ))
        .add_systems(Startup, setup)
        .run();
}

pub struct BlueSquare;

impl ExtractToShader for BlueSquare {
    type Shader = RectangleShader;
    type ParamsQuery<'a> = ();
    type ParamsBundle = ();
    type ResourceParams<'a> = ();

    fn get_params(
        _query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        _r: &(),
    ) -> <Self::Shader as ParameterizedShader>::Params {
        ColorParams {
            color: bevy::color::palettes::css::BLUE.with_alpha(0.5).into(),
        }
    }
}

pub struct RedSquare;

impl ExtractToShader for RedSquare {
    type Shader = RectangleShader;
    type ParamsQuery<'a> = ();
    type ParamsBundle = ();
    type ResourceParams<'a> = ();

    fn get_params(
        _query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        _r: &(),
    ) -> <Self::Shader as ParameterizedShader>::Params {
        ColorParams {
            color: bevy::color::palettes::css::RED.with_alpha(0.5).into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, TypePath, Default)]

pub struct RectangleShader;

impl ParameterizedShader for RectangleShader {
    type Params = ColorParams;

    fn fragment_body() -> impl Into<String> {
        "return in.color;"
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [].into_iter()
    }

    fn frame_expression() -> impl Into<String> {
        Frame {
            half_width: 100.0,
            half_height: 50.0,
        }
    }

    const UUID: u128 = 0x6d31023450194cd49f60ebabd7dca30b;
}

fn setup(mut commands: Commands) {
    commands.spawn(ShaderBundle::<RedSquare> {
        parameters: (),

        ..default()
    });

    commands.spawn(ShaderBundle::<BlueSquare> {
        parameters: (),
        transform: Transform::from_rotation(Quat::from_rotation_z(consts::FRAC_PI_4)),
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}
