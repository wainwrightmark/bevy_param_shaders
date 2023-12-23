use bevy::{prelude::*, reflect::TypeUuid};
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::prelude::*;
use bytemuck::{Pod, Zeroable};

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((DefaultPlugins, SmudPlugin::<MyShader>::default()))
        .add_systems(Startup, setup)
        .run();
}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, Pod, Zeroable)]
#[uuid = "6d310234-5019-4cd4-9f60-ebabd7dca30b"]
pub struct MyShader
{
    pub red: f32
}

impl ParameterizedShader for MyShader {
    fn fragment_body<'a>() -> &'a str {
        "return vec4<f32>(in.red,0.2,0.2,1.0);"
    }

    fn fragment_helpers<'a>() -> &'a str {
        ""
    }
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(ShapeBundle {
        shape: ShaderShape {
            frame: Frame::Quad(100.0),
            parameters: MyShader{red: 0.8},
        },
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}
