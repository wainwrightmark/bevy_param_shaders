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
            ParamShaderPlugin::<BevyBirdShader>::default(),
            PanCamPlugin,
        ))
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
pub struct BevyBirdShader;

impl ParameterizedShader for BevyBirdShader {
    type Params = ColorParams;
    type ParamsQuery<'a> = &'a ColorParams;
    type ParamsBundle = ColorParams;

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

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
    ) -> Self::Params {
        *query_item
    }

    const FRAME: Frame = Frame {
        half_width: 295.0,
        half_height: 295.0,
    };
}

fn setup(mut commands: Commands) {
    commands.spawn(ShaderBundle {
        shape: ShaderShape::<BevyBirdShader>::default(),
        parameters: Color::rgb(0.36, 0.41, 0.45).into(),

        ..default()
    });

    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
