use std::f32::consts;

use bevy::prelude::*;
// The prelude contains the basic things needed to create shapes
use bevy::log::LogPlugin;
use bevy_param_shaders::{
    prelude::*,
    primitives::{
        PrimitivesPlugin, RectShaderExtraction, RoundedRectShaderExtraction, ShaderProportions,
        ShaderRounding,
    },
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // DefaultPlugins.set(
            //     LogPlugin {
            //     filter: "info,wgpu_core=debug,wgpu_hal=debug,mygame=debug".into(),
            //     level: bevy::log::Level::DEBUG,
            //     custom_layer: |_| None,
            // }),
            PrimitivesPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update,gizmo)
        .run();
}

fn gizmo(
    mut gizmos: Gizmos,
){
    gizmos
        .grid_2d(
            Isometry2d::IDENTITY,
            UVec2::new(16, 9),
            Vec2::new(80., 80.),
            // Dark gray
            LinearRgba::gray(0.05),
        )
        .outer_edges();
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
            .with_scale(Vec3::splat(100.0))
            .with_translation(Vec3::Z),
        ..default()
    });

    commands.spawn(Camera2d::default());
}
