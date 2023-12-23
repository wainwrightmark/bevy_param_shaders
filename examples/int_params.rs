use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_smud::{
    param_usage::{ShaderParamUsage, ShaderParameter},
    prelude::*,
    SmudShaders,
};
use rand::prelude::IteratorRandom;

// this example shows how to use per-instance parameters in shapes
// in this simple example, a width and height is passed to a box shape,
// but it could be used for almost anything.

const F_PARAMS: usize = 3;
const U_PARAMS: usize = 1;

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
            SmudPlugin::<F_PARAMS, U_PARAMS>,
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
    const PARAMETERS: &'static [ShaderParameter] = &[
        ShaderParameter::f32(0),
        ShaderParameter::f32(1),
        ShaderParameter::f32(2),
        ShaderParameter::u32(0),
    ];
    let fill_param_usage = ShaderParamUsage(PARAMETERS);

    let circle = shaders.add_sdf_expr("smud::sd_circle(p, 50.)", ShaderParamUsage::NO_PARAMS);

    // The fill takes a distance and a color and returns another color
    let gradient_fill = shaders.add_fill_body(
        r"

        let a = smud::sd_fill_alpha_fwidth(d);
            let other_color = vec3<f32>(param_f_0, param_f_1, param_f_2);
        if param_u_0 == 0u{
             let mixed_color = mix(color.rgb, other_color, (p.x + 0.5) * 0.01);
             return vec4<f32>(mixed_color, a * color.a);
        }else{
            let mixed_color = mix(color.rgb, other_color, (p.y + 0.5) * 0.01);
            return vec4<f32>(mixed_color, a * color.a);
        }


                    ",
        fill_param_usage,
    );

    let padding = 5.; // need some padding for the outline/falloff
    let spacing = 150.;
    let palette = palettes.get(&assets.palette).unwrap();

    let clear_color = palette.lightest();
    commands.insert_resource(ClearColor(clear_color));
    let mut rng = rand::thread_rng();

    for i in 0..25u32 {
        let x = ((i % 5) as f32 - 2.5) * spacing;
        let y = ((i / 5) as f32 - 2.5) * spacing;

        let transform = Transform {
            scale: Vec3::splat(1.),
            translation: Vec3::new(x, y, 0.),
            rotation: Default::default(),
        };

        let color = palette
            .iter()
            .filter(|c| *c != &clear_color)
            .choose(&mut rng)
            .copied()
            .unwrap_or(Color::PINK);

        let color2 = palette
            .iter()
            .filter(|c| *c != &clear_color)
            .choose(&mut rng)
            .copied()
            .unwrap_or(Color::PINK);

        commands.spawn(ShapeBundle::<F_PARAMS, U_PARAMS> {
            transform,
            shape: SmudShape {
                color,

                frame: Frame::Quad(50.0 + padding),
                f_params: [color2.r().into(), color2.g().into(), color2.b().into()],
                u_params: [(i % 2).into()],
            },
            shaders: SmudShaders {
                sdf: circle.clone(),
                fill: gradient_fill.clone(),
                sdf_param_usage: ShaderParamUsage::NO_PARAMS,
                fill_param_usage,
            },
            ..Default::default()
        });
    }

    commands.spawn(Camera2dBundle::default());
}
