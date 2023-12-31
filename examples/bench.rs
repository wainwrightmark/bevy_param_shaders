use std::{borrow::Cow, fmt::Display};

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy::{prelude::*, reflect::TypeUuid};
use bevy_pancam::*;
use bevy_param_shaders::prelude::*;
use bytemuck::{Pod, Zeroable};
use rand::prelude::*;

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            ParamShaderPlugin::<MyShader>::default(),
            PanCamPlugin,
        ))
        .add_systems(Startup, setup)
        // .add_system_set(SystemSet::on_update(GameState::Running).with_system(update))
        .run();
}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, Pod, Zeroable)]
#[uuid = "6d310234-5019-4cd4-9f60-ebabd7dca30b"]
pub struct MyShader;

impl ParameterizedShader for MyShader {
    fn fragment_body() -> impl Display {
        r#"
        let d = smud::bevy::sdf(in.pos);
        let color = smud::default_fill::fill(d, in.color);
        return color;
        "#
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [
            FragmentImport {
                path: "smud.wgsl",
                import_path: "smud",
            },
            FragmentImport {
                path: "bevy.wgsl",
                import_path: "smud::bevy",
            },
            FragmentImport{
                path: "cubic_falloff.wgsl",
                import_path: "smud::default_fill"
            }
        ]
        .into_iter()
    }

    type Params = MyParams;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct MyParams {
    pub color: LinearRGBA,
}

impl ShaderParams for MyParams {}

#[derive(Component)]
struct Index(usize);

fn setup(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let spacing = 800.0;
    let w = 632;
    let h = w;
    info!("Adding {} shapes", w * h);

    let clear_color = Color::NONE;
    commands.insert_resource(ClearColor(clear_color));

    for i in 0..w {
        for j in 0..h {
            let color = Color::Rgba {
                red: rng.gen_range(0.0..=1.0),
                green: rng.gen_range(0.0..=1.0),
                blue: rng.gen_range(0.0..=1.0),
                alpha: rng.gen_range(0.0..=1.0),
            };

            commands.spawn((
                ShaderBundle {
                    shape: ShaderShape::<MyShader> {
                        frame: Frame::Quad(295.0),
                        parameters: MyParams {
                            color: color.into(),
                        },
                    },
                    transform: Transform::from_translation(Vec3::new(
                        i as f32 * spacing - w as f32 * spacing / 2.,
                        j as f32 * spacing - h as f32 * spacing / 2.,
                        0.,
                    )),
                    ..default()
                },
                Index(i + j * w),
            ));
        }
    }
    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}

// fn update(mut query: Query<(&mut Transform, &Index), With<ShaderShape::<MyShader>>>, time: Res<Time>) {
//     let t = time.time_since_startup().as_secs_f64();

//     for (mut tx, index) in query.iter_mut() {
//         let s = f64::sin(t + (index.0 as f64) / 1.0) as f32;
//         tx.scale = Vec3::splat(s);
//     }
// }
