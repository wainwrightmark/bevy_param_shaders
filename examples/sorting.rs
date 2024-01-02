use bevy::{prelude::*, reflect::TypeUuid};
use bevy_pancam::*;
use bevy_param_shaders::prelude::*;

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.7, 0.8, 0.7)))
        .add_plugins((
            DefaultPlugins,
            ParamShaderPlugin::<CircleShader>::default(),
            ParamShaderPlugin::<BoxShader>::default(),
            ParamShaderPlugin::<HeartShader>::default(),
            PanCamPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

macro_rules! define_sdf_shader {
    ($name:ident,$uuid:literal,$sdf:literal) => {
        #[derive(Debug, Reflect, TypeUuid, Default)]
        #[uuid = $uuid]
        pub struct $name;

        impl ParameterizedShader for $name {
            fn fragment_body() -> impl Into<String> {
                SDFAlphaCall {
                    sdf: $sdf,
                    fill_alpha: "smud::sd_fill_alpha_fwidth(d)",
                    color: "in.color",
                }
            }

            fn imports() -> impl Iterator<Item = FragmentImport> {
                [FragmentImport {
                    path: "smud.wgsl",
                    import_path: "smud",
                }]
                .into_iter()
            }

            type Params = ColorParams;
        }
    };
}

define_sdf_shader!(
    BoxShader,
    "051301eb-61ea-4eed-b067-4682feb028a0",
    "smud::sd_rounded_box(in.pos, vec2<f32>(0.9, 0.9), vec4<f32>(0.2))"
);
define_sdf_shader!(
    CircleShader,
    "77738d8e-8e3a-4c94-bfbc-49620a87918d",
    "smud::sd_circle(in.pos, 0.8)"
);
define_sdf_shader!(
    HeartShader,
    "b3171ec4-2c7d-4095-bbd0-293754b33cd5",
    "smud::sd_heart(in.pos + vec2(0.0, 0.5))"
);

fn setup(mut commands: Commands) {
    let spacing = 100.0;
    let w = 5;
    commands.insert_resource(ClearColor(Color::NONE));

    let box_color = Color::BLUE;
    let circle_color = Color::GREEN;
    let heart_color = Color::RED;

    for i in 0..w {
        let frame = Frame::Quad(1.);

        macro_rules! spawn_bundle {
            ($name:ident, $z:literal, $color:ident) => {
                commands.spawn((ShaderBundle {
                    transform: Transform::from_translation(Vec3::new(
                        i as f32 * spacing - w as f32 * spacing / 2.,
                        100.0,
                        ((i as f32) * 100.0) + $z,
                    ))
                    .with_scale(Vec3::ONE * spacing * 0.75),
                    shape: ShaderShape::<$name> {
                        parameters: ($color).into(),
                        frame,
                        ..Default::default()
                    },
                    ..default()
                },))
            };
        }

        spawn_bundle!(BoxShader, 1.0, box_color);
        spawn_bundle!(CircleShader, 2.0, circle_color);
        spawn_bundle!(HeartShader, 3.0, heart_color);
    }

    for i in 0..w {
        let frame = Frame::Quad(1.);

        macro_rules! spawn_bundle {
            ($name:ident, $z:literal, $color:ident) => {
                commands.spawn((ShaderBundle {
                    transform: Transform::from_translation(Vec3::new(
                        i as f32 * spacing - w as f32 * spacing / 2.,
                        -100.0,
                        (i as f32) + ($z * 100.0),
                    ))
                    .with_scale(Vec3::ONE * spacing * 0.75),
                    shape: ShaderShape::<$name> {
                        parameters: ($color).into(),
                        frame,
                        ..Default::default()
                    },
                    ..default()
                },))
            };
        }

        let box_color = Color::BLUE;
        let circle_color = Color::GREEN;
        let heart_color = Color::RED;

        spawn_bundle!(BoxShader, 1.0, box_color);
        spawn_bundle!(CircleShader, 2.0, circle_color);
        spawn_bundle!(HeartShader, 3.0, heart_color);
    }

    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
