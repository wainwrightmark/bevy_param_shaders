use bevy::prelude::*;
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::prelude::*;

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            ExtractToShaderPlugin::<BevyMorphShader>::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

#[repr(C)]
#[derive(Debug, TypePath, Default)]
pub struct BevyMorphShader;

impl ExtractToShader for BevyMorphShader {
    type Shader = Self;
    type ParamsQuery<'a> = &'a ColorParams;
    type ParamsBundle = ColorParams;
    type ResourceParams<'a> = ();

    fn get_params<'w, 'w1, 'w2, 's2, 'a, 'r>(
        query_item: <Self::ParamsQuery<'a> as bevy::ecs::query::WorldQuery>::Item<'w1>,
        _resource: &'r <Self::ResourceParams<'w> as bevy::ecs::system::SystemParam>::Item<'w2, 's2>,
    ) -> <Self::Shader as ParameterizedShader>::Params {
        *query_item
    }
}

impl ParameterizedShader for BevyMorphShader {
    type Params = ColorParams;

    const USE_TIME: bool = true;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall{
            sdf:"mix(smud::sd_circle(in.pos, 250.0), smud::bevy::sdf(in.pos), sin(globals.time) * 0.5 + 0.5)",
            fill_color: "smud::default_fill::fill(d, in.color)"
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
    commands.spawn(ShaderBundle::<BevyMorphShader> {
        parameters: Color::ORANGE_RED.into(),

        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}
