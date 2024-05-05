use bevy::prelude::*;
use bevy_param_shaders::prelude::*;
//Hot reload works! Try modifying bevy.wgsl


fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            ExtractToShaderPlugin::<BevyBirdShader>::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}
#[repr(C)]
#[derive(Debug, Default, TypePath)]
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

    fn frame_expression() -> impl Into<String> {
        Frame::square(295.0)
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

    const UUID: u128 = 0x6d31023450194cd49f60ebabd7dca30b;
}


fn setup(mut commands: Commands) {
    // When sdfs are loaded from files, hot reloading works as normal
    // Open up assets/bevy.wgsl and make some changes and see them reflected when you save

    commands.spawn(ShaderBundle {
        transform: Transform {
            scale: Vec3::splat(0.4),
            ..default()
        },
        shape: ShaderUsage::<BevyBirdShader>::default(),
        parameters: Color::WHITE.into(),

        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}
