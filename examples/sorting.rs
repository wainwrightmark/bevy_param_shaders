use bevy::prelude::*;
use bevy_pancam::*;
use bevy_smud::{param_usage::ShaderParamUsage, prelude::*};

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.7, 0.8, 0.7)))
        .add_plugins((DefaultPlugins, SmudPlugin::<SmudSDF,SmudFill>::default(), PanCamPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut shaders: ResMut<Assets<Shader>>) {
    // pupil
    commands.spawn(ShapeBundle {
        transform: Transform::from_translation(Vec3::Z * 3.),
        shape: SmudShape {
            color: Color::rgb(0.0, 0.0, 0.0),

            frame: Frame::Quad(80.),
            ..default()
        },

        sdf: SmudSDF{handle: shaders.add_sdf_body::<&str, ()>(
            "return smud::sd_circle(p, 70.);",
            ShaderParamUsage::NO_PARAMS,
        )},
        fill: SmudFill::default(),
        ..default()
    });

    // iris
    commands.spawn(ShapeBundle {
        transform: Transform::from_translation(Vec3::Z * 2.),
        shape: SmudShape {
            color: Color::rgb(0.46, 0.42, 0.80),

            frame: Frame::Quad(200.),
            ..default()
        },
        sdf: SmudSDF{handle: shaders.add_sdf_body::<&str, ()>(
            "return smud::sd_circle(p, 150.);",
            ShaderParamUsage::NO_PARAMS,
        )},
        fill: SmudFill::default(),
        ..default()
    });

    // sclera
    commands.spawn(ShapeBundle {
        transform: Transform::from_translation(Vec3::Z * 1.),
        shape: SmudShape {
            color: Color::rgb(0.83, 0.82, 0.80),

            frame: Frame::Quad(400.),
            ..default()
        },

        sdf: SmudSDF{handle: shaders.add_sdf_body::<&str, ()>(
            "return smud::sd_vesica(p.yx, 400., 150.);",
            ShaderParamUsage::NO_PARAMS,
        )},
        fill: SmudFill::default(),
        ..default()
    });

    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
