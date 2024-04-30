use std::f32::consts;

use bevy::{prelude::*, reflect::TypePath};
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::prelude::*;

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            ExtractToShaderPlugin::<BlueSquare>::default(),
            ExtractToShaderPlugin::<RedSquare>::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

pub struct BlueSquare;

impl ExtractToShader for BlueSquare {
    type Shader = SquareShader;
    type ParamsQuery<'a> = ();
    type ParamsBundle = ();
    type ResourceParams<'a> = ();

    fn get_params<'w, 'a>(
        _query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
        _r: &(),
    ) -> <Self::Shader as ParameterizedShader>::Params {
        Color::BLUE.with_a(0.5).into()
    }
}

pub struct RedSquare;

impl ExtractToShader for RedSquare {
    type Shader = SquareShader;
    type ParamsQuery<'a> = ();
    type ParamsBundle = ();
    type ResourceParams<'a> = ();

    fn get_params<'w, 'a>(
        _query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
        _r: &(),
    ) -> <Self::Shader as ParameterizedShader>::Params {
        Color::RED.with_a(0.5).into()
    }
}

#[repr(C)]
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
