use bevy::{prelude::*, reflect::TypeUuid};
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::prelude::*;
use bytemuck::{Pod, Zeroable};

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((DefaultPlugins, ParamShaderPlugin::<SquareShader>::default()))
        .add_systems(Startup, setup)
        .run();
}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, Pod, Zeroable)]
#[uuid = "6d310234-5019-4cd4-9f60-ebabd7dca30b"]
pub struct SquareShader;

impl ParameterizedShader for SquareShader {
    type Params = ColorParams;
    fn fragment_body() -> impl Into<String> {
        "return in.color;"
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [].into_iter()
    }
}



fn setup(mut commands: Commands) {
    commands.spawn(ShaderBundle {
        shape: ShaderShape::<SquareShader> {
            frame: Frame::Quad(100.0),
            parameters: ColorParams {
                color: Color::ORANGE_RED.into(),
            },
        },
        ..default()
    });

    commands.spawn(ShaderBundle {
        shape: ShaderShape::<SquareShader> {
            frame: Frame::Quad(50.0),
            parameters: ColorParams {
                color: Color::BLUE.into(),
            },
        },
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}
