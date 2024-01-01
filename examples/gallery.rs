use bevy::{prelude::*, reflect::TypeUuid};
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
            ParamShaderPlugin::<BoxShader>::default(),
            ParamShaderPlugin::<CircleShader>::default(),
            ParamShaderPlugin::<HeartShader>::default(),
            ParamShaderPlugin::<MoonShader>::default(),
            ParamShaderPlugin::<PieShader>::default(),
            ParamShaderPlugin::<EggShader>::default(),
            ParamShaderPlugin::<RoundedXShader>::default(),
            ParamShaderPlugin::<EllipseShader>::default(),
            bevy::diagnostic::LogDiagnosticsPlugin::default(),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            PanCamPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

macro_rules! define_sdf_shader {
    ($name:ident,$uuid:literal,$sdf:literal) => {
        #[repr(C)]
        #[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default)]
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
    "smud::sd_rounded_box(in.pos, vec2<f32>(0.8, 0.9), vec4<f32>(0.1))"
);
define_sdf_shader!(
    CircleShader,
    "77738d8e-8e3a-4c94-bfbc-49620a87918d",
    "smud::sd_circle(in.pos, 1.0)"
);
define_sdf_shader!(
    HeartShader,
    "b3171ec4-2c7d-4095-bbd0-293754b33cd5",
    "smud::sd_heart(in.pos)"
);
define_sdf_shader!(
    MoonShader,
    "e3bda7c4-b689-4ac7-af9b-78ec0a49a2a3",
    "smud::sd_moon(in.pos, 1.0, 0.5, 0.25)"
);
define_sdf_shader!(
    PieShader,
    "563bc173-593c-4261-9091-1881e07dea13",
    "smud::sd_pie(in.pos, vec2<f32>(sin(0.5), cos(0.5)), 1.0)"
);
define_sdf_shader!(
    EggShader,
    "e0d981eb-d382-4e10-bcc9-e4a7a234b746",
    "smud::sd_egg(in.pos, 0.5, 0.25)"
);
define_sdf_shader!(
    RoundedXShader,
    "aaf2157b-08f7-4164-b29a-194421403e26",
    "smud::sd_rounded_x(in.pos, 0.8, 0.5)"
);
define_sdf_shader!(
    EllipseShader,
    "21de381f-01df-4513-8f3d-10da25954ab3",
    "smud::sd_ellipse(in.pos, 0.2,0.8)"
);

#[derive(Component)]
struct Index(usize);

fn setup(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let spacing = 100.0;
    let w = 632;
    let h = w;
    info!("Adding {} shapes", w * h);
    commands.insert_resource(ClearColor(Color::NONE));



    for i in 0..w {
        for j in 0..h {
            let index = i + j * w;

            let color = Color::Rgba {
                red: rng.gen_range(0.1..=1.0),
                green: rng.gen_range(0.1..=1.0),
                blue: rng.gen_range(0.1..=1.0),
                alpha: rng.gen_range(0.5..=1.0),
            };

            let frame = Frame::Quad(1.);

            let transform = Transform::from_translation(Vec3::new(
                i as f32 * spacing - w as f32 * spacing / 2.,
                j as f32 * spacing - h as f32 * spacing / 2.,
                0.,
            )).with_scale(Vec3::ONE * 50.0);

            macro_rules! spawn_bundle {
                ($name:ident) => {
                    commands.spawn((
                        ShaderBundle {
                            transform,
                            shape: ShaderShape::<$name> {
                                parameters: color.into(),
                                frame,
                                ..Default::default()
                            },
                            ..default()
                        },
                        Index(index),
                    ))
                };
            }

            let shader_index = rng.gen_range(0..8);

            match shader_index {
                0 => spawn_bundle!(BoxShader),
                1 => spawn_bundle!(CircleShader),
                2 => spawn_bundle!(HeartShader),
                3 => spawn_bundle!(MoonShader),
                4 => spawn_bundle!(PieShader),
                5 => spawn_bundle!(EggShader),
                6 => spawn_bundle!(RoundedXShader),
                _ => spawn_bundle!(EllipseShader),
            };


        }
    }

    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
