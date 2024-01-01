use bevy::{prelude::*, reflect::TypeUuid};
use bevy_pancam::*;
use bevy_param_shaders::prelude::*;

use bytemuck::{Pod, Zeroable};

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.7, 0.8, 0.7)))
        .add_plugins((DefaultPlugins,
            ParamShaderPlugin::<CircleShader>::default(),
            ParamShaderPlugin::<VesicaShader>::default(),
            PanCamPlugin))
        .add_systems(Startup, setup)
        .run();
}


#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default)]
#[uuid = "b5a9b5d8-32b8-4d63-8e01-45ae42b96d1d"]
pub struct VesicaShader;

impl ParameterizedShader for VesicaShader {
    fn fragment_body() -> impl Into<String> {
        SDFAlphaCall{
            sdf: "smud::sd_vesica(in.pos.yx, 1.0, 0.375)",
            fill_alpha:  "smud::sd_fill_alpha_fwidth(d)",
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

    type Params = ColorParams;
}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default)]
#[uuid = "6d310234-5019-4cd4-9f60-ebabd7dca30b"]
pub struct CircleShader;

impl ParameterizedShader for CircleShader {
    fn fragment_body() -> impl Into<String> {
        SDFAlphaCall{
            sdf: "smud::sd_circle(in.pos, 1.0)",
            fill_alpha:  "smud::sd_fill_alpha_fwidth(d)",
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

    type Params = ColorParams;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, Pod, Zeroable)]
pub struct ColorParams {
    pub color: LinearRGBA,
}

impl ShaderParams for ColorParams{

}


fn setup(mut commands: Commands) {
    commands.spawn(ShaderBundle {
        transform: Transform::from_translation(Vec3::Z * 4.).with_scale(Vec3::ONE * 100.0),
        shape: ShaderShape::<CircleShader> {
            parameters: ColorParams { color: Color::RED.with_a(0.8).into() },
            frame: Frame::Quad(1.),
            ..default()
        },
        ..default()
    });
    commands.spawn(ShaderBundle {
        transform: Transform::from_translation(Vec3::Z * 3.).with_scale(Vec3::ONE * 150.0),
        shape: ShaderShape::<VesicaShader> {
            parameters: ColorParams {color: Color::GREEN.with_a(0.8).into()},

            frame: Frame::Quad(1.),
            ..default()
        },
        ..default()
    });

    commands.spawn(ShaderBundle {
        transform: Transform::from_translation(Vec3::Z * 2.).with_scale(Vec3::ONE * 225.0),
        shape: ShaderShape::<CircleShader> {
            parameters: ColorParams {color: Color::WHITE.with_a(0.8).into()},

            frame: Frame::Quad(1.),
            ..default()
        },
        ..default()
    });

    commands.spawn(ShaderBundle {
        transform: Transform::from_translation(Vec3::Z * 1.).with_scale(Vec3::ONE * 300.0),
        shape: ShaderShape::<VesicaShader> {
            parameters: ColorParams {color: Color::BLUE.with_a(0.8).into()},

            frame: Frame::Quad(1.),
            ..default()
        },
        ..default()
    });

    commands.spawn(ShaderBundle {
        transform: Transform::from_translation(Vec3::Z * 0.).with_scale(Vec3::ONE * 450.0),
        shape: ShaderShape::<CircleShader> {
            parameters: ColorParams {color: Color::BLACK.with_a(0.8).into()},

            frame: Frame::Quad(1.),
            ..default()
        },
        ..default()
    });

    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
