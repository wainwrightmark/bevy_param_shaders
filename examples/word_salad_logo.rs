use bevy::{
    core::FrameCount,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};
use bevy_image_export::{
    ImageExportBundle, ImageExportPlugin, ImageExportSettings, ImageExportSource,
};
// The prelude contains the basic things needed to create shapes
use bevy_param_shaders::prelude::*;

const WIDTH: u32 = 768;
const HEIGHT: u32 = 768;


fn main() {
    let export_plugin = ImageExportPlugin::default();
    let export_threads = export_plugin.threads.clone();

    App::new()
        // bevy_smud comes with anti-aliasing built into the standards fills
        // which is more efficient than MSAA, and also works on Linux, wayland
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (WIDTH as f32, HEIGHT as f32).into(),
                    ..default()
                }),
                ..default()
            }),
            ExtractToShaderPlugin::<WordLineSegmentShader>::default(),
            export_plugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_camera)
        .insert_resource(ClearColor(Color::WHITE))
        .init_resource::<WordLineGlobalTargets>()
        .insert_resource(WordLineGlobalValues::default())
        .add_systems(Update, do_updates)
        .add_systems(Update, transition_word_line.after(do_updates))
        .run();

    export_threads.finish();
}

const SCALE_FACTOR: f32 = 100.0;

//static DOWN_RIGHT: Vec2 = Vec2::new(0.5, 0.75f32.sqrt() * -1.0);

fn setup_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut export_sources: ResMut<Assets<ImageExportSource>>,
) {
    // Create an output texture.
    let output_texture_handle = {
        let size = Extent3d {
            width: WIDTH,
            height: HEIGHT,
            ..default()
        };
        let mut export_texture = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::COPY_SRC
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };
        export_texture.resize(size);

        images.add(export_texture)
    };

    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_translation(5.0 * Vec3::Z),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(Camera2dBundle {
                camera: Camera {
                    // Connect the output texture to a camera as a RenderTarget.
                    target: RenderTarget::Image(output_texture_handle.clone()),
                    ..default()
                },
                ..default()
            });
        });

    // Spawn the ImageExportBundle to initiate the export of the output texture.
    commands.spawn(ImageExportBundle {
        source: export_sources.add(output_texture_handle),
        settings: ImageExportSettings {
            // Frames will be saved to "./out/[#####].png".
            output_dir: "out".into(),
            // Choose "exr" for HDR renders.
            extension: "png".into(),
        },
    });
}

fn setup(mut commands: Commands) {
    commands.spawn((
        // Camera2dBundle {
        //     camera: Camera {
        //         hdr: true,
        //         ..default()
        //     },

        //     ..default()
        // },
        bevy::core_pipeline::bloom::BloomSettings {
            intensity: 0.1,
            composite_mode: bevy::core_pipeline::bloom::BloomCompositeMode::Additive,
            ..default()
        },
    ));
}

fn do_updates(
    mut commands: Commands,
    mut entities: Query<(Entity, &mut WordLineDirection, &mut ShaderSecondColor)>,
    frame_count: Res<FrameCount>,
    mut targets: ResMut<WordLineGlobalTargets>,
    mut bloom_target: Local<f32>,
) {
    if *bloom_target == 0.0 {
        *bloom_target = 0.7;
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
        } else if index == 4 {
            *bloom_target = 1.00;
        } else if index == 5 {
        } else if index == 10 {
            for (e, _, _) in entities.iter() {
                commands.entity(e).despawn_recursive();
            }
            *bloom_target = 0.7;
        }
    }
}

fn spawn_shape(commands: &mut Commands, index: usize) {
    let w = 0.4695;
    let h = 0.883;

    let down_right = Vec2::new(w, h * -1.0); //62 degrees 32 /7
    let up_right = Vec2::new(w, h);

    let point2 = if index % 2 == 0 { down_right } else { up_right };

    let mut position = Vec2 {
        y: 0.0,
        x: w * -2.0,
    };

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
        0 | 1 => Color::srgb(0.09, 0.34, 0.27),
        2 => Color::srgb(0.05, 0.43, 0.24),
        3 => Color::srgb(0.01, 0.53, 0.22),
        _ => Color::srgb(0.25, 0.67, 0.21),
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
    pub color1: LinearRgba,
    pub color2: LinearRgba,
}

impl ShaderParams for WordLineSegmentShaderParams {}

#[repr(C)]
#[derive(Debug, Reflect, Clone, Copy, Default, PartialEq)]

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

    fn frame_expression() -> impl Into<String> {
        Frame::square(2.0)
    }

    const UUID: u128 = 0xa68d391613854269a5124561eccd664d;
}

const FULL_LINE_WIDTH: f32 = 0.6; // 7.0 / 30.0;
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
    IncreaseToOne,
    ResetThenIncreaseToOne,
}

fn transition_word_line(
    mut values: ResMut<WordLineGlobalValues>,
    mut targets: ResMut<WordLineGlobalTargets>,
    time: Res<Time>,
) {
    let progress_change = time.delta_seconds() * PROGRESS_SPEED;

    let progress = match targets.target_progress {
        ProgressTarget::IncreaseToOne => (values.progress + progress_change).min(1.0),
        ProgressTarget::ResetThenIncreaseToOne => {
            targets.target_progress = ProgressTarget::IncreaseToOne;
            RESET_PROGRESS // + progress_change.min(1.0)
        }
    };

    let new_values = WordLineGlobalValues {
        progress,
        line_width: FULL_LINE_WIDTH,
    };

    values.set_if_neq(new_values);
}
