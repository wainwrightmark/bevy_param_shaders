use bevy::{prelude::*, reflect::TypeUuid};
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
        .add_plugins((DefaultPlugins, ParamShaderPlugin::<CircleShader>::default()))
        .add_systems(Startup, setup)
        .run();
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable, Component)]
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

#[repr(C)]
#[derive(Debug, TypeUuid, Default)]
#[uuid = "6d310234-5019-4cd4-9f60-ebabd7dca30b"]
pub struct CircleShader;

impl ParameterizedShader for CircleShader {
    type Params = ColorParams;
    type ParamsQuery<'a> = &'a ColorParams;
    type ParamsBundle = ColorParams;

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

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
    ) -> Self::Params {
        *query_item
    }

    const FRAME: Frame = Frame::square(1.0);
}

fn setup(mut commands: Commands) {
    commands.spawn(ShaderBundle {
        shape: ShaderShape::<CircleShader>::default(),


        parameters: ColorParams {
            color: Color::ORANGE_RED.into(),
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
