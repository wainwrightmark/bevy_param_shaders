use bevy::{core::FrameCount, prelude::*, reflect::TypeUuid, core_pipeline::bloom::BloomSettings};
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::prelude::*;

fn main() {
    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            ExtractToShaderPlugin::<WordLineSegmentShader>::default(),
        ))
        .add_systems(Startup, setup)
        //.insert_resource(ClearColor(Color::WHITE))
        .init_resource::<WordLineGlobalTargets>()
        .insert_resource(WordLineGlobalValues::default())
        .add_systems(Update, do_updates)
        .add_systems(Update, transition_word_line.after(do_updates))
        .run();
}

const SCALE_FACTOR: f32 = 100.0;

//static DOWN_RIGHT: Vec2 = Vec2::new(0.5, 0.75f32.sqrt() * -1.0);

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },

            ..default()
        },
        bevy::core_pipeline::bloom::BloomSettings {
            intensity: 0.7,
            ..default()
        },
    ));
}

fn do_updates(
    mut commands: Commands,
    mut entities: Query<(Entity, &mut WordLineDirection, &mut ShaderSecondColor)>,
    frame_count: Res<FrameCount>,
    mut targets: ResMut<WordLineGlobalTargets>,
    mut bloom: Query<&mut BloomSettings>,
    mut bloom_target: Local<f32>
) {
    if *bloom_target == 0.0{
        *bloom_target = 0.7;
    }

    for mut b in bloom.iter_mut(){
        match b.intensity.total_cmp(&bloom_target){
            std::cmp::Ordering::Less => b.intensity += 0.0015,
            std::cmp::Ordering::Equal => {},
            std::cmp::Ordering::Greater => b.intensity = *bloom_target,
        }
    }

    const PACE: u32 = 40;
    if frame_count.0 % PACE == 0 {
        let index = (frame_count.0 / PACE) % 11;

        if index < 4 {
            for (_, mut existing, _) in entities.iter_mut() {
                existing.is_final_segment = false;
            }
            targets.target_progress = ProgressTarget::ResetThenIncreaseToOne;
            spawn_shape(&mut commands, index as usize);
            *bloom_target = 0.7 + (index as f32 * 0.01);
        }
        else if index == 4{
            *bloom_target = 1.00;
        }
        else if index == 5{

        }


         else if index == 10 {
            for (e, _, _) in entities.iter() {
                commands.entity(e).despawn_recursive();
            }
            *bloom_target = 0.7;
        }
    }
}



fn spawn_shape(commands: &mut Commands, index: usize) {
    let down_right = Vec2::new(0.5, 0.75f32.sqrt() * -1.0);
    let up_right = Vec2::new(0.5, 0.75f32.sqrt());

    let point2 = if index % 2 == 0 { down_right } else { up_right };

    let mut position = Vec2::ZERO;

    for x in 0..=index {
        let next_point = if x % 2 == 0 { down_right } else { up_right };
        position += next_point;
    }
    position.y = 0.0;

    commands.spawn(ShaderBundle::<WordLineSegmentShader> {
        parameters: (
            WordLineDirection {
                point2,
                is_final_segment: true,
            },
            ShaderColor {
                color: index_to_color(index),
            },
            ShaderSecondColor {
                color: index_to_color(index + 1),
            },
        ),
        transform: Transform {
            translation: (position * SCALE_FACTOR * 2.0).extend(0.0),
            rotation: Default::default(),
            scale: Vec3::ONE * SCALE_FACTOR,
        },
        ..Default::default()
    });
}

fn index_to_color(index: usize) -> Color {
    match index {
        0 | 1 => Color::rgba(0.05, 0.37, 0.29, 1.0),
        2 => Color::rgba(0.05, 0.5, 0.27, 1.0),
        3 => Color::rgba(0.05, 0.59, 0.26, 1.0),
        _ => Color::rgba(0.11, 0.64, 0.23, 1.0),
    }
}

#[derive(Debug, Clone, Component, PartialEq, Default)]
pub struct WordLineDirection {
    pub point2: Vec2,
    pub is_final_segment: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub struct ShaderColor {
    pub color: Color,
}

impl From<Color> for ShaderColor {
    fn from(color: Color) -> Self {
        Self { color }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Component, Default)]
pub struct ShaderSecondColor {
    pub color: Color,
}

impl From<Color> for ShaderSecondColor {
    fn from(color: Color) -> Self {
        Self { color }
    }
}

impl ExtractToShader for WordLineSegmentShader {
    type Shader = Self;
    type ParamsQuery<'a> = (
        &'a WordLineDirection,
        &'a ShaderColor,
        &'a ShaderSecondColor,
    );
    type ParamsBundle = (WordLineDirection, ShaderColor, ShaderSecondColor);
    type ResourceParams<'w> = Res<'w, WordLineGlobalValues>;

    fn get_params(
        query_item: <Self::ParamsQuery<'_> as bevy::ecs::query::WorldQuery>::Item<'_>,
        resource: &<Self::ResourceParams<'_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> <Self::Shader as ParameterizedShader>::Params {
        let progress = if query_item.0.is_final_segment {
            resource.progress
        } else {
            1.0
        };

        WordLineSegmentShaderParams {
            line_width: resource.line_width,
            point2: query_item.0.point2,

            color1: query_item.1.color.into(),
            color2: query_item.2.color.into(),
            progress,
        }
    }
}

pub struct WordLineParams {}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WordLineSegmentShaderParams {
    pub line_width: f32,
    pub point2: Vec2,
    pub progress: f32,
    pub color1: LinearRGBA,
    pub color2: LinearRGBA,
}

impl ShaderParams for WordLineSegmentShaderParams {}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, TypeUuid, Default, PartialEq)]
#[uuid = "a68d3916-1385-4269-a512-4561eccd664d"]
struct WordLineSegmentShader;

impl ParameterizedShader for WordLineSegmentShader {
    type Params = WordLineSegmentShaderParams;

    fn fragment_body() -> impl Into<String> {
        SDFColorCall {
            sdf: "sdf::word_line_segment::sdf(in.pos, in.line_width, in.point2, in.progress)",
            fill_color: "fill::simple::fill(d, mix(in.color1, in.color2, in.progress) , in.pos)",
        }
    }

    fn imports() -> impl Iterator<Item = bevy_param_shaders::prelude::FragmentImport> {
        const WORDLINE_IMPORT: FragmentImport = FragmentImport {
            path: "word_line_segment.wgsl",
            import_path: "sdf::word_line_segment",
        };

        const SIMPLE_FILL_IMPORT: FragmentImport = FragmentImport {
            path: "simple.wgsl",
            import_path: "fill::simple",
        };

        [WORDLINE_IMPORT, SIMPLE_FILL_IMPORT].into_iter()
    }

    const FRAME: Frame = Frame::square(2.0); //TODO can this be lower?
}

#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Default, Reflect, bytemuck::Pod, bytemuck::Zeroable, Component,
)]
pub struct ColorParams {
    pub color: LinearRGBA,
}

impl ShaderParams for ColorParams {}

impl From<bevy::prelude::Color> for ColorParams {
    fn from(value: bevy::prelude::Color) -> Self {
        Self {
            color: value.into(),
        }
    }
}

const FULL_LINE_WIDTH: f32 = 0.5;
const PULSED_LINE_WIDTH: f32 = FULL_LINE_WIDTH * 1.2;
const ZERO_LINE_WIDTH: f32 = -0.01; //slightly below zero to prevent artifacts
const LINE_WIDTH_DECREASE_SPEED: f32 = FULL_LINE_WIDTH * 1.2;
const LINE_WIDTH_INCREASE_SPEED: f32 = FULL_LINE_WIDTH * 4.0;
const LINE_WIDTH_PULSE_SPEED: f32 = FULL_LINE_WIDTH * 0.5;
const PROGRESS_SPEED: f32 = 4.0;
const RESET_PROGRESS: f32 = 0.00;

#[derive(Debug, Resource, PartialEq)]
struct WordLineGlobalValues {
    pub line_width: f32,
    pub progress: f32,
}

#[derive(Debug, Resource)]
struct WordLineGlobalTargets {
    pub target_progress: ProgressTarget,
}

impl Default for WordLineGlobalValues {
    fn default() -> Self {
        Self {
            line_width: FULL_LINE_WIDTH,
            progress: 0.0,
        }
    }
}
impl Default for WordLineGlobalTargets {
    fn default() -> Self {
        Self {
            target_progress: ProgressTarget::IncreaseToOne,
        }
    }
}

#[derive(Debug, Resource, PartialEq)]
enum ProgressTarget {
    One,
    IncreaseToOne,
    DecreaseToZero,
    ResetThenIncreaseToOne,
    OneThenDecreaseToZero,
}

fn transition_word_line(
    mut values: ResMut<WordLineGlobalValues>,
    mut targets: ResMut<WordLineGlobalTargets>,
    time: Res<Time>,
) {
    let progress_change = time.delta_seconds() * PROGRESS_SPEED;


    let progress = match targets.target_progress {
        ProgressTarget::IncreaseToOne => (values.progress + progress_change).min(1.0),
        ProgressTarget::DecreaseToZero => (values.progress - progress_change).max(0.0),
        ProgressTarget::ResetThenIncreaseToOne => {
            targets.target_progress = ProgressTarget::IncreaseToOne;
            RESET_PROGRESS // + progress_change.min(1.0)
        }
        ProgressTarget::OneThenDecreaseToZero => {
            targets.target_progress = ProgressTarget::DecreaseToZero;
            1.0
        }
        ProgressTarget::One => {
            targets.target_progress = ProgressTarget::IncreaseToOne;
            1.0
        }
    };

    let new_values = WordLineGlobalValues {
        progress,
        line_width: FULL_LINE_WIDTH
    };

    values.set_if_neq(new_values);
}
