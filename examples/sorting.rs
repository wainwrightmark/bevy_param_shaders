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
        .add_plugins((DefaultPlugins, ParamShaderPlugin::<CircleShader>::default(),ParamShaderPlugin::<VesicaShader>::default(), PanCamPlugin))
        .add_systems(Startup, setup)
        .run();
}


#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, Pod, Zeroable)]
#[uuid = "b5a9b5d8-32b8-4d63-8e01-45ae42b96d1d"]
pub struct VesicaShader;

impl ParameterizedShader for VesicaShader {
    fn fragment_body() -> impl std::fmt::Display {
        r#"
        let d = smud::sd_vesica(in.pos.yx, 1.0, 0.375);
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

    type Params = ColorParams;
}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, Pod, Zeroable)]
#[uuid = "6d310234-5019-4cd4-9f60-ebabd7dca30b"]
pub struct CircleShader;

impl ParameterizedShader for CircleShader {
    fn fragment_body() -> impl std::fmt::Display {
        r#"
        let d = smud::sd_circle(in.pos, 1.0);
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
    // pupil
    commands.spawn(ShaderBundle {
        transform: Transform::from_translation(Vec3::Z * 3.).with_scale(Vec3::ONE * 70.0),
        shape: ShaderShape::<CircleShader> {
            parameters: ColorParams { color: Color::rgb(0.0, 0.0, 0.0).into() },
            frame: Frame::Quad(1.),
            ..default()
        },
        ..default()
    });

    // iris
    commands.spawn(ShaderBundle {
        transform: Transform::from_translation(Vec3::Z * 2.).with_scale(Vec3::ONE * 150.0),
        shape: ShaderShape::<CircleShader> {
            parameters: ColorParams {color: Color::rgb(0.46, 0.42, 0.80).into()},

            frame: Frame::Quad(1.),
            ..default()
        },
        ..default()
    });

    // sclera
    commands.spawn(ShaderBundle {
        transform: Transform::from_translation(Vec3::Z * 1.).with_scale(Vec3::ONE * 400.0),
        shape: ShaderShape::<VesicaShader> {
            parameters: ColorParams {color: Color::rgb(0.83, 0.82, 0.80).into()},

            frame: Frame::Quad(1.),
            ..default()
        },
        ..default()
    });

    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
