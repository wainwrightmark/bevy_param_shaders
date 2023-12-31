use std::{borrow::Cow, fmt::Display};

use bevy::{prelude::*, reflect::TypeUuid};
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::prelude::*;
use bytemuck::{Pod, Zeroable};

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins, ParamShaderPlugin::<MyShader>::default()))
        .add_systems(Startup, setup)
        .run();
}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, Pod, Zeroable)]
#[uuid = "6d310234-5019-4cd4-9f60-ebabd7dca30b"]
pub struct MyShader;

impl ParameterizedShader for MyShader {
    fn fragment_body() -> impl Display {
        r#"
        let d = smud::sd_circle(in.pos, 0.7);
        let a = smud::sd_fill_alpha_fwidth(d);
        return vec4<f32>(in.color.rgb, a * in.color.a);
        "#
    }

    fn imports() -> impl Iterator<Item = FragmentImport> {
        [FragmentImport {
            path: "smud.wgsl",
            import_path: "smud",
        }]
        .into_iter()
    }

    type Params = MyParams;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct MyParams {
    pub color: LinearRGBA,
}

impl ShaderParams for MyParams {}

fn setup(mut commands: Commands) {
    commands.spawn(ShaderBundle {
        shape: ShaderShape::<MyShader> {
            frame: Frame::Quad(1.0),
            parameters: MyParams {
                color: Color::ORANGE_RED.into(),
            },
        },
        transform: Transform::from_scale(Vec3::ONE * 100.0),
        ..default()
    });

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },

            ..default()
        },
        bevy::core_pipeline::bloom::BloomSettings {
            intensity: 0.7,
            ..default()
        },
    ));
}
