use bevy::{prelude::*, reflect::TypeUuid};
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::prelude::*;

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            ParamShaderPlugin::<BevyMorphShader>::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable, Component,
)]
pub struct MorphParams {
    pub color: LinearRGBA,
    pub time: f32
}

impl ShaderParams for MorphParams {}

#[derive(Debug, Clone, Copy, PartialEq, Default, Component)]
pub struct ColorParams {
    pub color: LinearRGBA,
}



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
pub struct BevyMorphShader;

impl ParameterizedShader for BevyMorphShader {
    type Params = MorphParams;
    type ParamsQuery<'a> = &'a ColorParams;
    type ParamsBundle = ColorParams;
    type ResourceParams<'a> = Res<'a, Time>;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall{
            sdf:"mix(smud::sd_circle(in.pos, 250.0), smud::bevy::sdf(in.pos), sin(in.time) * 0.5 + 0.5)",
            fill_color: "smud::default_fill::fill(d, in.color)"
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

    const FRAME: Frame = Frame::square(295.0);

    fn get_params<'w, 'w1, 'w2, 's2, 'a, 'r>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w1>,
        resource: &'r <Self::ResourceParams<'w> as bevy::ecs::system::SystemParam>::Item<'w2, 's2>,
    ) -> Self::Params {
        MorphParams{
            color: query_item.color,
            time: resource.elapsed_seconds_wrapped()
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(ShaderBundle {
        shape: ShaderShape::<BevyMorphShader>::default(),
        parameters: Color::ORANGE_RED.into(),

        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}
