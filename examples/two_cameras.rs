use bevy::{color::palettes, prelude::*};
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::prelude::*;

// Should show the same circle twice with different bloom settings.
// Currently does not work
fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins,
            ExtractToShaderPlugin::<CircleShader>::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

impl ExtractToShader for CircleShader {
    type Shader = Self;
    type ParamsQuery<'a> = &'a ColorParams;
    type ParamsBundle = ColorParams;
    type ResourceParams<'a> = ();

    fn get_params(
        query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        _r: &(),
    ) -> <Self::Shader as ParameterizedShader>::Params {
        *query_item
    }
}

#[repr(C)]
#[derive(Debug, TypePath, Default)]
pub struct CircleShader;

impl ParameterizedShader for CircleShader {
    type Params = ColorParams;

    fn fragment_body() -> impl Into<String> {
        SDFAlphaCall {
            sdf: "smud::sd_circle(in.pos, 1.0)",
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

    const UUID: u128 = 0x6d31023450194cd49f60ebabd7dca30b;
}

fn setup(mut commands: Commands) {
    commands.spawn(ShaderBundle {
        shape: ShaderUsage::<CircleShader>::default(),

        parameters: ColorParams {
            color: palettes::css::ORANGE_RED.into(),
        },

        transform: Transform::from_scale(Vec3::ONE * 100.0),
        ..default()
    });

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: 1,
                ..default()
            },
            transform: Transform::from_translation(Vec3::X * -500.0),
            ..default()
        },
        bevy::core_pipeline::bloom::BloomSettings {
            intensity: 0.9,
            ..default()
        },
    ));
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: 2,

                ..default()
            },
            transform: Transform::from_translation(Vec3::X * 500.0),

            ..default()
        },
        bevy::core_pipeline::bloom::BloomSettings {
            intensity: 0.4,
            ..default()
        },
    ));
}
