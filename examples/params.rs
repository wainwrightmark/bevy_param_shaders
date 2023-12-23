use std::f32::consts::PI;

use bevy::{prelude::*, reflect::TypeUuid};
use bevy_asset_loader::prelude::*;
use bevy_smud::{
    param_usage::{ShaderParamUsage, ShaderParameter},
    prelude::*,
};
use rand::{prelude::IteratorRandom, random};

// this example shows how to use per-instance parameters in shapes
// in this simple example, a width and height is passed to a box shape,
// but it could be used for almost anything.

#[derive(Debug, Component, Reflect, TypeUuid, Clone, PartialEq, Default)]
#[uuid = "8cf0eefc-9eb2-11ee-8c90-0242ac120002"]
pub struct MySDFShader{
    pub params: [f32; 2],
    pub handle: Handle<Shader>
}

impl MySDFShader{
    const PARAMETERS: &'static [ShaderParameter] =
        &[ShaderParameter(0), ShaderParameter(1)];

}

impl SmudSDFShader for MySDFShader{}

impl SmudShader for MySDFShader{
    type Params = [f32; 2];

    fn params(&self) -> Self::Params {
        self.params
    }

    fn shader(&self) -> Handle<Shader> {
        self.handle.clone()
    }

    fn param_usage(&self) -> ShaderParamUsage {
        ShaderParamUsage(Self::PARAMETERS)
    }
}

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Running),
        )
        .add_collection_to_loading_state::<_, AssetHandles>(GameState::Loading)
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            SmudPlugin::<MySDFShader, SmudFill>::default(),
            bevy_lospec::PalettePlugin,
        ))
        .add_systems(OnEnter(GameState::Running), setup)
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, States, Default)]
enum GameState {
    #[default]
    Loading,
    Running,
}

#[derive(Resource, AssetCollection)]
struct AssetHandles {
    #[asset(path = "vinik24.json")]
    palette: Handle<bevy_lospec::Palette>,
}

fn setup(
    mut commands: Commands,
    mut shaders: ResMut<Assets<Shader>>,
    assets: Res<AssetHandles>,
    palettes: Res<Assets<bevy_lospec::Palette>>,
) {


    let box_sdf = shaders.add_sdf_expr::<&str, [f32;2]>(
        "smud::sd_box(p, vec2<f32>(param_sdf_0, param_sdf_1))",
        ShaderParamUsage(MySDFShader::PARAMETERS),
    );
    let padding = 5.; // need some padding for the outline/falloff
    let spacing = 70.;
    let palette = palettes.get(&assets.palette).unwrap();

    let clear_color = palette.lightest();
    commands.insert_resource(ClearColor(clear_color));
    let mut rng = rand::thread_rng();

    for i in 0..100 {
        let size = Vec2::new(random::<f32>() * 20. + 1., random::<f32>() * 20. + 1.);
        let x = ((i % 10) as f32 - 4.5) * spacing;
        let y = ((i / 10) as f32 - 4.5) * spacing;

        let transform = Transform {
            scale: Vec3::splat(1.),
            translation: Vec3::new(x, y, 0.),
            rotation: Quat::from_rotation_z(random::<f32>() * PI),
        };

        let color = palette
            .iter()
            .filter(|c| *c != &clear_color)
            .choose(&mut rng)
            .copied()
            .unwrap_or(Color::PINK);

        commands.spawn(ShapeBundle::<MySDFShader, SmudFill> {
            transform,
            shape: SmudShape {
                color,

                frame: Frame::Quad(size.x.max(size.y) + padding),

                ..Default::default()
            },

            fill: SmudFill{handle: SIMPLE_FILL_HANDLE},
            sdf: MySDFShader{
                handle: box_sdf.clone(),
                params:[size.x.into(), size.y.into()],
            },

            ..Default::default()
        });
    }

    commands.spawn(Camera2dBundle::default());
}
