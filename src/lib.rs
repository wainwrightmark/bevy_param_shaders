//#![warn(missing_docs)]
#![allow(clippy::too_many_arguments)]

use std::{marker::PhantomData, ops::Range};

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    math::Vec3Swizzles,
    prelude::*,
    render::{
        globals::{GlobalsBuffer, GlobalsUniform},
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutDescriptor,
            BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, BufferUsages,
            BufferVec, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace,
            MultisampleState, PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPipelineDescriptor, ShaderStages, ShaderType, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureFormat, VertexAttribute, VertexBufferLayout,
            VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
        Extract, Render, RenderApp, RenderSet,
    },
    utils::FloatOrd,
};
use bytemuck::{Pod, Zeroable};
use pipeline_key::PipelineKey;
use shader_loading::*;

pub use bundle::ShaderBundle;
pub use components::*;

use parameterized_shader::*;
use shader_params::ShaderParams;

mod bundle;
mod components;
mod fragment_shader;
mod helpers;
pub mod parameterized_shader;
mod pipeline_key;
mod shader_loading;
pub mod shader_params;
mod util;
mod vertex_shader;

/// Re-export of the essentials needed for rendering shapes
///
/// Intended to be included at the top of your file to minimize the amount of import noise.
/// ```
/// use bevy_param_shaders::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        parameterized_shader::*, shader_params::*, Frame, ParamShaderPlugin, ShaderBundle,
        ShaderShape,
    };
}

#[derive(Debug, Default)]
struct ParameterShadersPlugin;

impl Plugin for ParameterShadersPlugin {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(
                Render,
                join_adjacent_batches.in_set(RenderSet::PrepareBindGroups),
            );
        };
    }
}

/// Main plugin for enabling rendering of Sdf shapes
pub struct ParamShaderPlugin<SHADER: ParameterizedShader>(PhantomData<SHADER>);

impl<SHADER: ParameterizedShader> Default for ParamShaderPlugin<SHADER> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<SHADER: ParameterizedShader> Plugin for ParamShaderPlugin<SHADER> {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShaderLoadingPlugin::<SHADER>::default());
        if !app.is_plugin_added::<ParameterShadersPlugin>() {
            app.add_plugins(ParameterShadersPlugin);
        }

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Transparent2d, DrawSmudShape<SHADER>>()
                .init_resource::<ExtractedShapes<SHADER>>()
                .init_resource::<SpecializedRenderPipelines<SmudPipeline<SHADER>>>()
                .add_systems(ExtractSchedule, (extract_shapes::<SHADER>,))
                .add_systems(
                    Render,
                    (
                        (sort_shapes::<SHADER>, queue_shapes::<SHADER>)
                            .chain()
                            .in_set(RenderSet::Queue),
                        prepare_shapes::<SHADER>.in_set(RenderSet::PrepareBindGroups),
                    ),
                );
        };
        app.register_type::<SHADER>();
    }

    fn finish(&self, app: &mut App) {
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_resource::<SmudPipeline<SHADER>>();
    }
}

type DrawSmudShape<SHADER> = (
    SetItemPipeline,
    SetShapeViewBindGroup<0, SHADER>,
    DrawShapeBatch<SHADER>,
);

struct SetShapeViewBindGroup<const I: usize, SHADER: ParameterizedShader>(PhantomData<SHADER>);
impl<P: PhaseItem, const I: usize, SHADER: ParameterizedShader> RenderCommand<P>
    for SetShapeViewBindGroup<I, SHADER>
{
    type Param = SRes<ExtractedShapes<SHADER>>;
    type ViewWorldQuery = Read<ViewUniformOffset>;
    type ItemWorldQuery = ();

    fn render<'w>(
        _item: &P,
        view_uniform: ROQueryItem<'w, Self::ViewWorldQuery>,
        _view: (),
        shape_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(
            I,
            shape_meta.into_inner().view_bind_group.as_ref().unwrap(),
            &[view_uniform.offset],
        );
        RenderCommandResult::Success
    }
}

struct DrawShapeBatch<SHADER: ParameterizedShader>(PhantomData<SHADER>);
impl<P: PhaseItem, SHADER: ParameterizedShader> RenderCommand<P> for DrawShapeBatch<SHADER> {
    type Param = SRes<ExtractedShapes<SHADER>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<ShapeBatch>;

    fn render<'w>(
        _item: &P,
        _view: (),
        batch: &'_ ShapeBatch,
        shape_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let shape_meta = shape_meta.into_inner();
        if let Some(buffer) = shape_meta.vertices.buffer() {
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..4, batch.range.clone()); //0..4 as there are four vertices
            RenderCommandResult::Success
        } else {
            warn!("Render Fail {:?}", batch);
            RenderCommandResult::Failure
        }
    }
}

#[derive(Resource)]
struct SmudPipeline<SHADER: ParameterizedShader> {
    view_layout: BindGroupLayout,
    phantom: PhantomData<SHADER>,
}

impl<SHADER: ParameterizedShader> FromWorld for SmudPipeline<SHADER> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(ViewUniform::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GlobalsUniform::min_size()),
                    },
                    count: None,
                },
            ],
            label: Some("shape_view_layout"),
        });

        Self {
            view_layout,
            phantom: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct SmudPipelineKey {
    mesh: PipelineKey,
    hdr: bool,
}

impl<SHADER: ParameterizedShader> SpecializedRenderPipeline for SmudPipeline<SHADER> {
    type Key = SmudPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // let fragment_shader = self.shaders.fragment_shaders.get(&key.shader).unwrap();
        // debug!("specializing for {fragment_shader:?}");

        // an f32 is 4 bytes
        const WORD_BYTE_LENGTH: u64 = 4;

        const FRAME_WORDS: u64 = 1;
        const POSITION_WORDS: u64 = 3;
        const ROTATION_WORDS: u64 = 2;
        const SCALE_WORDS: u64 = 1;

        let proxy = <SHADER::Params as Default>::default();
        let param_count = proxy.field_len() as u32;

        // (GOTCHA! attributes are sorted alphabetically, and offsets need to reflect this)

        let pre_param_attributes: [VertexAttribute; 1] = [
            // Frame
            VertexAttribute {
                format: VertexFormat::Float32,
                offset: 0,
                shader_location: param_count + 3,
            },
        ];

        const POST_PARAM_ATTRIBUTES_LEN: usize = 3;

        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position, color and params
        let mut vertex_attributes = Vec::with_capacity(
            pre_param_attributes.len() + param_count as usize + POST_PARAM_ATTRIBUTES_LEN,
        );

        vertex_attributes.extend_from_slice(&pre_param_attributes);

        let mut offset = (FRAME_WORDS) * WORD_BYTE_LENGTH;
        let mut shader_location: u32 = 1;

        for field in proxy.iter_fields() {
            let Some(format) = helpers::get_vertex_format(field.type_id()) else {
                panic!(
                    "Cannot convert {} to wgsl type",
                    field
                        .get_represented_type_info()
                        .map(|info| info.type_path())
                        .unwrap_or_else(|| field.reflect_type_path())
                );
            };

            vertex_attributes.push(VertexAttribute {
                format,
                offset,
                shader_location,
            });
            offset += format.size();
            shader_location += 1;
        }

        let post_param_attributes: [VertexAttribute; POST_PARAM_ATTRIBUTES_LEN] = [
            // Position
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: offset,
                shader_location: 0,
            },
            // Rotation
            VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: offset + (POSITION_WORDS * WORD_BYTE_LENGTH),
                shader_location: shader_location,
            },
            // Scale
            VertexAttribute {
                format: VertexFormat::Float32,
                offset: offset + ((POSITION_WORDS + ROTATION_WORDS) * WORD_BYTE_LENGTH),
                shader_location: shader_location + 1,
            },
        ];

        vertex_attributes.extend_from_slice(&post_param_attributes);

        // This is the sum of the size of the attributes above
        let vertex_array_stride =
            offset + ((POSITION_WORDS + ROTATION_WORDS + SCALE_WORDS) * WORD_BYTE_LENGTH);

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: shader_loading::get_vertex_handle::<SHADER>().clone_weak(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                buffers: vec![VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Instance,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                shader: shader_loading::get_fragment_handle::<SHADER>().clone_weak(),
                entry_point: "fragment".into(),
                shader_defs: Vec::new(),
                targets: vec![Some(ColorTargetState {
                    format: if key.hdr {
                        ViewTarget::TEXTURE_FORMAT_HDR
                    } else {
                        TextureFormat::bevy_default()
                    },
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: vec![
                // Bind group 0 is the view uniform
                self.view_layout.clone(),
            ],
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false, // What is this?
                polygon_mode: PolygonMode::Fill,
                conservative: false, // What is this?
                topology: key.mesh.primitive_topology(),
                strip_index_format: None, // TODO: what does this do?
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.mesh.msaa_samples(),
                mask: !0,                         // what does the mask do?
                alpha_to_coverage_enabled: false, // what is this?
            },
            label: Some("bevy_smud_pipeline".into()),
            push_constant_ranges: Vec::new(),
        }
    }
}

#[derive(Resource)]
struct ExtractedShapes<SHADER: ParameterizedShader> {
    vertices: BufferVec<ShapeVertex<SHADER::Params>>,
    view_bind_group: Option<BindGroup>,
}

impl<SHADER: ParameterizedShader> Default for ExtractedShapes<SHADER> {
    fn default() -> Self {
        Self {
            // shapes: Default::default(),
            vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}

fn extract_shapes<SHADER: ParameterizedShader>(
    mut extracted_shapes: ResMut<ExtractedShapes<SHADER>>,
    shape_query: Extract<Query<(&ViewVisibility, &ShaderShape<SHADER>, &GlobalTransform)>>,
) {
    extracted_shapes.vertices.clear();

    for (view_visibility, shape, transform) in shape_query.iter() {
        if !view_visibility.get() {
            continue;
        }

        let Frame::Quad(frame) = shape.frame;

        let shape_vertex = ShapeVertex::new(transform, frame, shape.parameters);

        extracted_shapes.vertices.push(shape_vertex);
    }
}

fn sort_shapes<SHADER: ParameterizedShader>(mut extracted_shapes: ResMut<ExtractedShapes<SHADER>>) {
    radsort::sort_by_key(
        &mut extracted_shapes.as_mut().vertices.values_mut(),
        |item| item.z_index(),
    );
}

fn queue_shapes<SHADER: ParameterizedShader>(
    mut commands: Commands,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    smud_pipeline: Res<SmudPipeline<SHADER>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SmudPipeline<SHADER>>>,
    pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    extracted_shapes: Res<ExtractedShapes<SHADER>>,
    mut views: Query<(&mut RenderPhase<Transparent2d>, &ExtractedView)>,
) {
    let draw_function = draw_functions
        .read()
        .get_id::<DrawSmudShape<SHADER>>()
        .unwrap();

    // Iterate over each view (a camera is a view)
    for (mut transparent_phase, view) in &mut views {
        // todo: bevy_sprite does some hdr stuff, should we?
        // let mut view_key = SpritePipelineKey::from_hdr(view.hdr) | msaa_key;

        let mesh_key = PipelineKey::from_msaa_samples(msaa.samples())
            | PipelineKey::from_primitive_topology(PrimitiveTopology::TriangleStrip);

        let specialize_key = SmudPipelineKey {
            mesh: mesh_key,
            hdr: view.hdr,
        };
        let pipeline = pipelines.specialize(&pipeline_cache, &smud_pipeline, specialize_key);

        let mut index = 0;
        while let Some(first_shape) = extracted_shapes.vertices.values().get(index) {
            let start = index;
            index += 1;
            let z = first_shape.z_index();
            //these will always be batched with shapes with the same z index
            while extracted_shapes
                .vertices
                .values()
                .get(index)
                .is_some_and(|n| n.z_index() == z)
            {
                index += 1;
            }

            let sort_key = FloatOrd(z);
            let range = (start as u32)..(index as u32);
            let entity = commands.spawn(ShapeBatch { range }).id();

            // Add the item to the render phase
            transparent_phase.add(Transparent2d {
                draw_function,
                pipeline,
                entity,
                sort_key,
                batch_range: 0..1,
                dynamic_offset: None,
            });
        }
    }
}

fn prepare_shapes<SHADER: ParameterizedShader>(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    view_uniforms: Res<ViewUniforms>,
    smud_pipeline: Res<SmudPipeline<SHADER>>,
    mut extracted_shapes: ResMut<ExtractedShapes<SHADER>>,
    globals_buffer: Res<GlobalsBuffer>,
) {
    let Some(globals) = globals_buffer.buffer.binding() else {
        return;
    };

    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    extracted_shapes.view_bind_group = Some(render_device.create_bind_group(
        "smud_shape_view_bind_group",
        &smud_pipeline.view_layout,
        &BindGroupEntries::sequential((view_binding, globals.clone())),
    ));

    extracted_shapes
        .vertices
        .write_buffer(&render_device, &render_queue);
}

fn join_adjacent_batches(
    mut phases: Query<&mut RenderPhase<Transparent2d>>,
    mut batches: Query<&mut ShapeBatch>,
) {
    for mut transparent_phase in &mut phases {
        let mut index = 0;

        while let Some(item) = transparent_phase.items.get(index) {
            let item_index = index;
            index += 1;

            let entity = item.entity;
            let Ok(batch) = batches.get(entity) else {
                continue;
            };
            let mut range = batch.range.clone();
            let mut extra_count = 0;

            'concat: while let Some(next) = transparent_phase.items.get(index) {
                if item.draw_function != next.draw_function {
                    break 'concat;
                }
                let next_entity = next.entity;
                let Ok(next_batch) = batches.get(next_entity) else {
                    break 'concat;
                };
                range.end = next_batch.range.end;
                index += 1;
                extra_count += 1;
            }

            if extra_count > 0 {
                //we are doing a concat

                let Some(item) = transparent_phase.items.get_mut(item_index) else {
                    continue;
                };

                let Ok(mut batch) = batches.get_mut(entity) else {
                    continue;
                };

                item.batch_range.end += extra_count;
                batch.range = range;
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Zeroable)]
struct ShapeVertex<PARAMS: ShaderParams> {
    pub frame: f32,
    pub params: PARAMS,
    pub position: [f32; 3],
    pub rotation: [f32; 2],
    pub scale: f32,
}

impl<PARAMS: ShaderParams> ShapeVertex<PARAMS> {
    pub fn new(transform: &GlobalTransform, frame: f32, params: PARAMS) -> Self {
        let position = transform.translation();
        let position = position.into();

        let rotation_and_scale = transform.affine().transform_vector3(Vec3::X).xy();

        let scale = rotation_and_scale.length();
        let rotation = (rotation_and_scale / scale).into();

        ShapeVertex {
            position,
            params,
            rotation,
            scale,
            frame,
        }
    }

    pub fn z_index(&self) -> f32 {
        self.position[2]
    }
}

unsafe impl<PARAMS: ShaderParams> Pod for ShapeVertex<PARAMS> {}

#[derive(Component, Eq, PartialEq, Clone, Debug)]
pub(crate) struct ShapeBatch {
    range: Range<u32>,
}
