use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_param_shaders::prelude::*;

/// This example just shows that transforms work

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.7, 0.8, 0.7)))
        .add_plugins((
            DefaultPlugins,
            ExtractToShaderPlugin::<BevyBirdShader>::default(),
            PanCamPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}


#[repr(C)]
#[derive(Debug, TypePath, Default)]

pub struct BevyBirdShader;

impl ExtractToShader for BevyBirdShader {
    type Shader = BevyBirdShader;
    type ParamsQuery<'a> = &'a ColorParams;
    type ParamsBundle = ColorParams;
    type ResourceParams<'a> = ();

    fn get_params<'w, 'a>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w>,
        _r: &(),
    ) -> <Self::Shader as ParameterizedShader>::Params {
        *query_item
    }
}

impl ParameterizedShader for BevyBirdShader {
    type Params = ColorParams;

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

    fn frame_expression() -> impl Into<String> {
        Frame::square(295.0)
    }

    const UUID: u128 = 0x6d31023450194cd49f60ebabd7dca30b;
}

fn setup(mut commands: Commands) {
    let transform = Transform {
        scale: Vec3::splat(0.05),
        translation: Vec3::new(62., 137., 0.),
        rotation: Quat::from_rotation_z(1.0),
    };

    // Bevies, all the way down
    commands
        .spawn(ShaderBundle {
            shape: ShaderUsage::<BevyBirdShader>::default(),
            parameters: Color::rgb(0.36, 0.41, 0.45).into(),

            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ShaderBundle {
                    transform,
                    shape: ShaderUsage::<BevyBirdShader>::default(),
                    parameters: Color::rgb(0.36, 0.41, 0.45).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(ShaderBundle {
                        transform,
                        shape: ShaderUsage::<BevyBirdShader>::default(),
                        parameters: Color::rgb(0.36, 0.41, 0.45).into(),

                        ..default()
                    });
                });
        });

    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}
