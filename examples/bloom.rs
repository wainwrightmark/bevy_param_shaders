use bevy::prelude::*;
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::{
    prelude::*,
    primitives::{CircleShader, PrimitivesPlugin},
};

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland

        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins, PrimitivesPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(ShaderBundle {
        shape: ShaderUsage::<CircleShader>::default(),

        parameters: bevy::color::palettes::css::ORANGE_RED.into(),
        transform: Transform::from_scale(Vec3::ONE * 100.0),
        ..default()
    });

    commands.spawn((
        Camera2d {


            ..default()
        },
        Camera {
            hdr: true,
            ..default()
        },
        bevy::core_pipeline::bloom::Bloom {
            intensity: 0.7,
            ..default()
        },
    ));
}
