use std::f32::consts;

use bevy::prelude::*;
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::{
    prelude::*,
    primitives::{
        PrimitivesPlugin, RectShaderExtraction, RoundedRectShaderExtraction, ShaderProportions, ShaderRounding
    },
};

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((DefaultPlugins, PrimitivesPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(ShaderBundle::<RectShaderExtraction> {
        parameters: (
            bevy::color::palettes::css::ORANGE_RED.into(),
            ShaderProportions {
                width: 0.75,
                height: 1.5,
            },
        ),
        transform: Transform::from_scale(Vec3::splat(100.0)),
        visibility: Visibility::Visible,
        view_visibility: ViewVisibility::default(),
        inherited_visibility: InheritedVisibility::VISIBLE,
        ..Default::default()
    });



    commands.spawn(ShaderBundle::<RoundedRectShaderExtraction> {
        parameters: (
            bevy::color::palettes::css::BLUE.with_alpha(0.7).into(),
            ShaderRounding { rounding: 0.1 },
            ShaderProportions {
                width: 1.5,
                height: 0.75,
            },
        ),
        transform: Transform::from_rotation(Quat::from_rotation_z(consts::FRAC_PI_4))
            .with_scale(Vec3::splat(100.0)).with_translation(Vec3::Z),
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}
