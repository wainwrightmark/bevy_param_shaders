use bevy::reflect::TypeUuid;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_pancam::*;
use bevy_param_shaders::prelude::*;
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
            ExtractToShaderPlugin::<BevyBirdShader>::default(),
            PanCamPlugin,
        ))
        .add_systems(Startup, setup)
        // .add_system_set(SystemSet::on_update(GameState::Running).with_system(update))
        .run();
}

#[repr(C)]
#[derive(Debug, TypeUuid, Default)]
#[uuid = "6d310234-5019-4cd4-9f60-ebabd7dca30b"]
pub struct BevyBirdShader;

impl ExtractToShader for BevyBirdShader {
    type Shader = BevyBirdShader;
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

impl ParameterizedShader for BevyBirdShader {
    type Params = ColorParams;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "smud::bevy::sdf(in.pos)",
            fill_color: "smud::default_fill::fill(d, in.color)",
        }
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
            FragmentImport {
                path: "cubic_falloff.wgsl",
                import_path: "smud::default_fill",
            },
        ]
        .into_iter()
    }

    const FRAME: Frame = Frame {
        half_width: 295.0,
        half_height: 295.0,
    };
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

#[derive(Component)]
struct Index(usize);

fn setup(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let spacing = 800.0;
    let w = 1024;
    let h = w;
    info!("Adding {} shapes", w * h);

    let clear_color = Color::NONE;
    commands.insert_resource(ClearColor(clear_color));

    for i in 0..w {
        for j in 0..h {
            let color = Color::Rgba {
                red: rng.gen_range(0.1..=1.0),
                green: rng.gen_range(0.1..=1.0),
                blue: rng.gen_range(0.1..=1.0),
                alpha: rng.gen_range(0.5..=1.0),
            };

            commands.spawn((
                ShaderBundle::<BevyBirdShader> {
                    parameters: color.into(),
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
