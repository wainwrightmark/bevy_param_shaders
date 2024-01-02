use bevy::{prelude::*, reflect::TypeUuid};
use bevy_param_shaders::prelude::*;
use bytemuck::{Pod, Zeroable};

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            ParamShaderPlugin::<BevyBirdShader>::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default)]
#[uuid = "6d310234-5019-4cd4-9f60-ebabd7dca30b"]
pub struct BevyBirdShader;

impl ParameterizedShader for BevyBirdShader {
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

    type Params = ColorParams;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct MyParams {
    pub color: LinearRGBA,
}

impl ShaderParams for MyParams {}

fn setup(mut commands: Commands) {
    // When sdfs are loaded from files, hot reloading works as normal
    // Open up assets/bevy.wgsl and make some changes and see them reflected when you save

    commands.spawn(ShaderBundle {
        transform: Transform {
            scale: Vec3::splat(0.4),
            ..default()
        },
        shape: ShaderShape::<BevyBirdShader> {
            parameters: Color::WHITE.into(),

            frame: Frame::Quad(295.),
            ..Default::default()
        },
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}
