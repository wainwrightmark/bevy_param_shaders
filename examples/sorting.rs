use bevy::{color::palettes, prelude::*};
//use bevy_pancam::*;
use bevy_param_shaders::prelude::*;

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::srgb(0.7, 0.8, 0.7)))
        .add_plugins((
            DefaultPlugins,
            ExtractToShaderPlugin::<CircleShader>::default(),
            ExtractToShaderPlugin::<BoxShader>::default(),
            ExtractToShaderPlugin::<HeartShader>::default(),
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
                Frame::square(1.0)
            }

            const UUID: u128 = $uuid;
        }
    };
}


define_sdf_shader!(
    BoxShader,
    0x051301eb61ea4eedb0674682feb028a0,
    "smud::sd_rounded_box(in.pos, vec2<f32>(0.9, 0.9), vec4<f32>(0.2))"
);
define_sdf_shader!(
    CircleShader,
    0x77738d8e8e3a4c94bfbc49620a87918d,
    "smud::sd_circle(in.pos, 0.8)"
);
define_sdf_shader!(
    HeartShader,
    0xb3171ec42c7d4095bbd0293754b33cd5,
    "smud::sd_heart(in.pos + vec2(0.0, 0.5))"
);

fn setup(mut commands: Commands) {
    let spacing = 100.0;
    let w = 5;
    commands.insert_resource(ClearColor(Color::NONE));

    let box_color = palettes::css::BLUE;
    let circle_color = palettes::css::GREEN;
    let heart_color = palettes::css::RED;

    for i in 0..w {
        macro_rules! spawn_bundle {
            ($name:ident, $z:literal, $color:ident) => {
                commands.spawn((ShaderBundle::<$name> {
                    transform: Transform::from_translation(Vec3::new(
                        i as f32 * spacing - w as f32 * spacing / 2.,
                        100.0,
                        ((i as f32) * 100.0) + $z,
                    ))
                    .with_scale(Vec3::ONE * spacing * 0.75),
                    parameters: ($color).into(),

                    ..default()
                },))
            };
        }

        spawn_bundle!(BoxShader, 1.0, box_color);
        spawn_bundle!(CircleShader, 2.0, circle_color);
        spawn_bundle!(HeartShader, 3.0, heart_color);
    }

    for i in 0..w {
        macro_rules! spawn_bundle {
            ($name:ident, $z:literal, $color:ident) => {
                commands.spawn((ShaderBundle::<$name> {
                    transform: Transform::from_translation(Vec3::new(
                        i as f32 * spacing - w as f32 * spacing / 2.,
                        -100.0,
                        (i as f32) + ($z * 100.0),
                    ))
                    .with_scale(Vec3::ONE * spacing * 0.75),
                    parameters: ($color).into(),
                    ..default()
                },))
            };
        }

        let box_color = palettes::css::BLUE;
        let circle_color = palettes::css::GREEN;
        let heart_color = palettes::css::RED;

        spawn_bundle!(BoxShader, 1.0, box_color);
        spawn_bundle!(CircleShader, 2.0, circle_color);
        spawn_bundle!(HeartShader, 3.0, heart_color);
    }

    //commands.spawn((Camera2dBundle::default(), PanCam::default()));
    commands.spawn(Camera2dBundle::default());
}
