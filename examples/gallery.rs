use bevy::prelude::*;
//use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_param_shaders::prelude::*;
use rand::prelude::*;

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            ExtractToShaderPlugin::<BoxShader>::default(),
            ExtractToShaderPlugin::<CircleShader>::default(),
            ExtractToShaderPlugin::<HeartShader>::default(),
            ExtractToShaderPlugin::<MoonShader>::default(),
            ExtractToShaderPlugin::<PieShader>::default(),
            ExtractToShaderPlugin::<EggShader>::default(),
            ExtractToShaderPlugin::<RoundedXShader>::default(),
            ExtractToShaderPlugin::<EllipseShader>::default(),
            bevy::diagnostic::LogDiagnosticsPlugin::default(),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            //PanCamPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

macro_rules! define_sdf_shader {
    ($name:ident,$uuid:literal,$sdf:literal) => {
        #[derive(Debug, TypePath, Default)]

        pub struct $name;

        impl ExtractToShader for $name {
            type Shader = Self;
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

        impl ParameterizedShader for $name {
            type Params = ColorParams;

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

            fn frame_expression() -> impl Into<String> {
                Frame::square(1.)
            }

            const UUID: u128 = $uuid;
        }
    };
}

define_sdf_shader!(
    BoxShader,
    0x051301eb61ea4eedb0674682feb028a0,
    "smud::sd_rounded_box(in.pos, vec2<f32>(0.8, 0.8), vec4<f32>(0.2))"
);
define_sdf_shader!(
    CircleShader,
    0x77738d8e8e3a4c94bfbc49620a87918d,
    "smud::sd_circle(in.pos, 0.9)"
);
define_sdf_shader!(
    HeartShader,
    0xb3171ec42c7d4095bbd0293754b33cd5,
    "smud::sd_heart(in.pos + vec2(0.0, 0.5))"
);
define_sdf_shader!(
    MoonShader,
    0xe3bda7c4b6894ac7af9b78ec0a49a2a3,
    "smud::sd_moon(in.pos, 0.5, 0.9, 0.9)"
);
define_sdf_shader!(
    PieShader,
    0x563bc173593c426190911881e07dea13,
    "smud::sd_pie(in.pos, vec2<f32>(sin(0.5), cos(0.5)), 1.0)"
);
define_sdf_shader!(
    EggShader,
    0xe0d981ebd3824e10bcc9e4a7a234b746,
    "smud::sd_egg(in.pos, 0.5, 0.25)"
);
define_sdf_shader!(
    RoundedXShader,
    0xaaf2157b08f74164b29a194421403e26,
    "smud::sd_rounded_x(in.pos, 0.8, 0.2)"
);
define_sdf_shader!(
    EllipseShader,
    0x21de381f01df45138f3d10da25954ab3,
    "smud::sd_ellipse(in.pos, 0.2,0.8)"
);


fn setup(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let spacing = 100.0;
    let w = 1024;
    let h = w;
    info!("Adding {} shapes", w * h);
    commands.insert_resource(ClearColor(Color::NONE));

    for i in 0..w {
        for j in 0..h {
            let index = i + j * w;

            let color = Color::Srgba(Srgba {
                red: rng.gen_range(0.1..=1.0),
                green: rng.gen_range(0.1..=1.0),
                blue: rng.gen_range(0.1..=1.0),
                alpha: rng.gen_range(0.5..=1.0),
            });

            let transform = Transform::from_translation(Vec3::new(
                i as f32 * spacing - w as f32 * spacing / 2.,
                j as f32 * spacing - h as f32 * spacing / 2.,
                0.,
            ))
            .with_rotation(Quat::from_rotation_z(
                rng.gen_range(0.0..std::f32::consts::TAU),
            ))
            .with_scale(Vec3::ONE * spacing * 0.5);

            macro_rules! spawn_bundle {
                ($name:ident) => {
                    commands.spawn((
                        ShaderBundle::<$name> {
                            transform,
                            parameters: color.into(),
                            ..default()
                        },
                    ))
                };
            }

            //let shader_index = rng.gen_range(0..8);

            match index % 8 {
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

    commands.spawn((Camera2dBundle::default()));
    //commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
