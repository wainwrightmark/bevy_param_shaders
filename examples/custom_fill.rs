use bevy::prelude::*;
use bevy_pancam::*;
use bevy_smud::{param_usage::ShaderParamUsage, prelude::*,  SIMPLE_FILL_HANDLE};

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins,
            SmudPlugin::<SmudSDF, SmudFill>::default(),
            PanCamPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    // The fill takes a distance and a color and returns another color
    let sin_fill = shaders.add_fill_body::<&str, ()>(
        "return vec4<f32>(color.rgb, sin(d));",
        ShaderParamUsage::NO_PARAMS,
    );

    commands.spawn(ShapeBundle {
        shape: SmudShape {
            color: Color::TEAL,

            frame: Frame::Quad(295.),
            ..Default::default()
        },
        sdf: SmudSDF {
            handle: asset_server.load("bevy.wgsl"),
        },
        fill: SmudFill { handle: sin_fill },
        ..default()
    });

    commands.spawn(ShapeBundle {
        transform: Transform::from_translation(Vec3::X * 600.),
        shape: SmudShape {
            color: Color::BLUE,

            frame: Frame::Quad(295.),
            ..Default::default()
        },
        sdf: SmudSDF {
            handle: asset_server.load("bevy.wgsl"),
        },
        fill: SmudFill {
            handle: SIMPLE_FILL_HANDLE,
        },
        ..default()
    });

    commands.spawn(ShapeBundle {
        transform: Transform::from_translation(Vec3::X * -600.),
        shape: SmudShape {
            color: Color::ORANGE,

            frame: Frame::Quad(295.),
            ..Default::default()
        },
        sdf: SmudSDF {
            handle: asset_server.load("bevy.wgsl"),
        },
        fill: SmudFill {
            handle: shaders.add_fill_body::<&str, ()>(
                r"
let d_2 = abs(d - 1.) - 1.;
let a = smud::sd_fill_alpha_fwidth(d_2);
return vec4<f32>(color.rgb, a * color.a);
        ",
                ShaderParamUsage::NO_PARAMS,
            ),
        },
        ..default()
    });

    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
