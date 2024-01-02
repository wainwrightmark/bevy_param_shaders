use bevy::{prelude::*, reflect::TypeUuid};
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_param_shaders::prelude::*;
use bytemuck::{Pod, Zeroable};

/// This example just shows that transforms work

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
    let transform = Transform {
        scale: Vec3::splat(0.05),
        translation: Vec3::new(62., 137., 0.),
        rotation: Quat::from_rotation_z(1.0),
    };

    let shape = ShaderShape::<BevyBirdShader> {
        parameters: Color::rgb(0.36, 0.41, 0.45).into(),
        frame: Frame::square(295.),
        ..default()
    };

    // Bevies, all the way down
    commands
        .spawn(ShaderBundle {
            shape: shape.clone(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ShaderBundle {
                    transform,
                    shape: shape.clone(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(ShaderBundle {
                        transform,
                        shape: shape.clone(),
                        ..default()
                    });
                });
        });

    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
