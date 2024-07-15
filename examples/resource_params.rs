use bevy::{color::palettes, prelude::*};
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::prelude::*;

fn main() {
    let mut app = App::new();

    app
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            ExtractToShaderPlugin::<BevyMorphShader>::default(),
        ))
        .add_systems(Startup, setup);

    app.insert_resource(ColorResource::default());
    app.add_systems(Update, change_color);

    app.run();
}

#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable, Component,
)]
pub struct MorphParams {
    pub color: LinearRgba,
    pub time: f32,
}

impl ShaderParams for MorphParams {}

#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub struct ColorResource {
    pub color: LinearRgba,
}

impl Default for ColorResource {
    fn default() -> Self {
        Self {
            color: palettes::css::ORANGE_RED.into(),
        }
    }
}

fn change_color(mut color: ResMut<ColorResource>, time: Res<Time>) {
    color.color.blue = (color.color.blue + (time.delta_seconds() * 0.2)) % 1.0;
}

#[repr(C)]
#[derive(Debug, TypePath, Default)]
pub struct BevyMorphShader;

impl ExtractToShader for BevyMorphShader {
    type Shader = Self;
    type ParamsQuery<'a> = ();
    type ParamsBundle = ();
    type ResourceParams<'a> = (Res<'a, Time>, Res<'a, ColorResource>);

    fn get_params(
        _query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        resources: &<Self::ResourceParams<'_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> <Self::Shader as ParameterizedShader>::Params {
        MorphParams {
            color: resources.1.color,
            time: resources.0.elapsed_seconds_wrapped(),
        }
    }
}

impl ParameterizedShader for BevyMorphShader {
    type Params = MorphParams;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall{
            sdf:"mix(smud::sd_circle(in.pos, 250.0), smud::bevy::sdf(in.pos), sin(in.time) * 0.5 + 0.5)",
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
    commands.spawn(ShaderBundle {
        shape: ShaderUsage::<BevyMorphShader>::default(),

        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}
