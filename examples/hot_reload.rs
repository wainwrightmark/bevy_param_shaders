use bevy::prelude::*;
use bevy_param_shaders::prelude::*;

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((DefaultPlugins, SmudPlugin::<MyShader>::default()))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // When sdfs are loaded from files, hot reloading works as normal
    // Open up assets/bevy.wgsl and make some changes and see them reflected when you save
    let bevy = asset_server.load("bevy.wgsl");

    commands.spawn(ShapeBundle {
        transform: Transform {
            scale: Vec3::splat(0.4),
            ..default()
        },
        shape: SmudShape {
            color: Color::WHITE,

            frame: Frame::Quad(295.),
            ..Default::default()
        },
        sdf: SmudSDF{handle: bevy},
        fill: SmudFill{handle: SIMPLE_FILL_HANDLE},
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}
