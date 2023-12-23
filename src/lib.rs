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
            BufferVec, CachedRenderPipelineId, ColorTargetState, ColorWrites, Face, FragmentState,
            FrontFace, MultisampleState, PipelineCache, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPipelineDescriptor, ShaderImport, ShaderStages, ShaderType,
            SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat, VertexAttribute,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{
            ExtractedView, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms,
            VisibleEntities,
        },
        Extract, MainWorld, Render, RenderApp, RenderSet,
    },
    utils::{EntityHashMap, FloatOrd, HashMap},
};
use bytemuck::{Pod, Zeroable};
use fixedbitset::FixedBitSet;
use shader_loading::*;
// use ui::UiShapePlugin;

pub use bundle::ShapeBundle;
pub use components::*;

use parameterized_shader::*;

mod bundle;
mod components;
mod fragment_shader;
pub mod parameterized_shader;
mod sdf_assets;
mod shader_loading;
mod util;
mod vertex_shader;
// mod ui;

/// Re-export of the essentials needed for rendering shapes
///
/// Intended to be included at the top of your file to minimize the amount of import noise.
/// ```
/// use bevy_smud::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{parameterized_shader::*, Frame, ShaderShape, ShapeBundle, SmudPlugin};
}

/// Main plugin for enabling rendering of Sdf shapes
pub struct SmudPlugin<PARAMETERS: ParameterizedShader>(PhantomData<PARAMETERS>);

impl<PARAMETERS: ParameterizedShader> Default for SmudPlugin<PARAMETERS> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<PARAMETERS: ParameterizedShader> Plugin for SmudPlugin<PARAMETERS> {
    fn build(&self, app: &mut App) {
        // All the messy boiler-plate for loading a bunch of shaders
        app.add_plugins(ShaderLoadingPlugin::<PARAMETERS>::default());
        // app.add_plugins(UiShapePlugin);

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Transparent2d, DrawSmudShape<PARAMETERS>>()
                .init_resource::<ExtractedShapes<PARAMETERS>>()
                .init_resource::<ShapeMeta<PARAMETERS>>()
                .init_resource::<SpecializedRenderPipelines<SmudPipeline<PARAMETERS>>>()
                .add_systems(
                    ExtractSchedule,
                    (
                        extract_shapes::<PARAMETERS>,
                        //extract_sdf_shaders::<PARAMETERS>,
                    ),
                )
                .add_systems(
                    Render,
                    (
                        queue_shapes::<PARAMETERS>.in_set(RenderSet::Queue),
                        prepare_shapes::<PARAMETERS>.in_set(RenderSet::PrepareBindGroups),
                    ),
                );
        };
        app.register_type::<PARAMETERS>();
    }

    fn finish(&self, app: &mut App) {
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_resource::<SmudPipeline<PARAMETERS>>();
    }
}

type DrawSmudShape<PARAMETERS: ParameterizedShader> = (
    SetItemPipeline,
    SetShapeViewBindGroup<0, PARAMETERS>,
    DrawShapeBatch<PARAMETERS>,
);

struct SetShapeViewBindGroup<const I: usize, PARAMETERS: ParameterizedShader>(
    PhantomData<PARAMETERS>,
);
impl<P: PhaseItem, const I: usize, PARAMETERS: ParameterizedShader> RenderCommand<P>
    for SetShapeViewBindGroup<I, PARAMETERS>
{
    type Param = SRes<ShapeMeta<PARAMETERS>>;
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

struct DrawShapeBatch<PARAMETERS: ParameterizedShader>(PhantomData<PARAMETERS>);
impl<P: PhaseItem, PARAMETERS: ParameterizedShader> RenderCommand<P>
    for DrawShapeBatch<PARAMETERS>
{
    type Param = SRes<ShapeMeta<PARAMETERS>>;
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
        if let Some(buffer) = shape_meta.vertices.buffer(){
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..4, batch.range.clone());
            RenderCommandResult::Success
        }
        else{
            RenderCommandResult::Failure
        }



    }
}

#[derive(Resource)]
struct SmudPipeline<PARAMETERS: ParameterizedShader> {
    view_layout: BindGroupLayout,
    phantom: PhantomData<PARAMETERS>,
}

impl<PARAMETERS: ParameterizedShader> FromWorld for SmudPipeline<PARAMETERS> {
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

impl<PARAMETERS: ParameterizedShader> SpecializedRenderPipeline for SmudPipeline<PARAMETERS> {
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

        let proxy = PARAMETERS::default();
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
            let Some(format) = vertex_shader::get_vertex_format(field.type_id()) else {
                panic!("Cannot convert {} to wgsl type", field.type_name());
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
                shader: shader_loading::get_vertex_handle::<PARAMETERS>().clone_weak(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                buffers: vec![VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Instance,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                shader: shader_loading::get_fragment_handle::<PARAMETERS>().clone_weak(),
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

#[derive(Component, Clone, Debug)]
struct ExtractedShape<PARAMETERS: ParameterizedShader> {
    params: PARAMETERS,
    frame: f32,
    transform: GlobalTransform,
}

#[derive(Resource, Debug)]
struct ExtractedShapes<PARAMETERS: ParameterizedShader> {
    shapes: EntityHashMap<Entity, ExtractedShape<PARAMETERS>>,
}

impl<PARAMETERS: ParameterizedShader> Default for ExtractedShapes<PARAMETERS> {
    fn default() -> Self {
        Self {
            shapes: Default::default(),
        }
    }
}

fn extract_shapes<PARAMETERS: ParameterizedShader>(
    mut extracted_shapes: ResMut<ExtractedShapes<PARAMETERS>>,
    shape_query: Extract<
        Query<(
            Entity,
            &ViewVisibility,
            &ShaderShape<PARAMETERS>,
            &GlobalTransform,
        )>,
    >,
) {
    extracted_shapes.shapes.clear();

    for (entity, view_visibility, shape, transform) in shape_query.iter() {
        if !view_visibility.get() {
            continue;
        }

        let Frame::Quad(frame) = shape.frame;

        extracted_shapes.shapes.insert(
            entity,
            ExtractedShape {
                params: shape.parameters,
                transform: *transform,
                frame,
            },
        );
    }
}

// fork of Mesh2DPipelineKey (in order to remove bevy_sprite dependency)
// todo: merge with SmudPipelineKey?
bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    struct PipelineKey: u32 {
        const MSAA_RESERVED_BITS                = Self::MSAA_MASK_BITS << Self::MSAA_SHIFT_BITS;
        const PRIMITIVE_TOPOLOGY_RESERVED_BITS  = Self::PRIMITIVE_TOPOLOGY_MASK_BITS << Self::PRIMITIVE_TOPOLOGY_SHIFT_BITS;
    }
}

impl PipelineKey {
    const MSAA_MASK_BITS: u32 = 0b111;
    const MSAA_SHIFT_BITS: u32 = 32 - Self::MSAA_MASK_BITS.count_ones();
    const PRIMITIVE_TOPOLOGY_MASK_BITS: u32 = 0b111;
    const PRIMITIVE_TOPOLOGY_SHIFT_BITS: u32 = Self::MSAA_SHIFT_BITS - 3;

    pub fn from_msaa_samples(msaa_samples: u32) -> Self {
        let msaa_bits =
            (msaa_samples.trailing_zeros() & Self::MSAA_MASK_BITS) << Self::MSAA_SHIFT_BITS;
        Self::from_bits(msaa_bits).unwrap()
    }

    pub fn msaa_samples(&self) -> u32 {
        1 << ((self.bits() >> Self::MSAA_SHIFT_BITS) & Self::MSAA_MASK_BITS)
    }

    pub fn from_primitive_topology(primitive_topology: PrimitiveTopology) -> Self {
        let primitive_topology_bits = ((primitive_topology as u32)
            & Self::PRIMITIVE_TOPOLOGY_MASK_BITS)
            << Self::PRIMITIVE_TOPOLOGY_SHIFT_BITS;
        Self::from_bits(primitive_topology_bits).unwrap()
    }

    pub fn primitive_topology(&self) -> PrimitiveTopology {
        let primitive_topology_bits = (self.bits() >> Self::PRIMITIVE_TOPOLOGY_SHIFT_BITS)
            & Self::PRIMITIVE_TOPOLOGY_MASK_BITS;
        match primitive_topology_bits {
            x if x == PrimitiveTopology::PointList as u32 => PrimitiveTopology::PointList,
            x if x == PrimitiveTopology::LineList as u32 => PrimitiveTopology::LineList,
            x if x == PrimitiveTopology::LineStrip as u32 => PrimitiveTopology::LineStrip,
            x if x == PrimitiveTopology::TriangleList as u32 => PrimitiveTopology::TriangleList,
            x if x == PrimitiveTopology::TriangleStrip as u32 => PrimitiveTopology::TriangleStrip,
            _ => PrimitiveTopology::default(),
        }
    }
}

fn queue_shapes<PARAMETERS: ParameterizedShader>(
    mut view_entities: Local<FixedBitSet>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    smud_pipeline: Res<SmudPipeline<PARAMETERS>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SmudPipeline<PARAMETERS>>>,
    pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    extracted_shapes: ResMut<ExtractedShapes<PARAMETERS>>,
    mut views: Query<(
        &mut RenderPhase<Transparent2d>,
        &VisibleEntities,
        &ExtractedView,
    )>,
    // ?
) {
    let draw_smud_shape_function = draw_functions
        .read()
        .get_id::<DrawSmudShape<PARAMETERS>>()
        .unwrap();

    // Iterate over each view (a camera is a view)
    for (mut transparent_phase, visible_entities, view) in &mut views {
        // todo: bevy_sprite does some hdr stuff, should we?
        // let mut view_key = SpritePipelineKey::from_hdr(view.hdr) | msaa_key;

        let mesh_key = PipelineKey::from_msaa_samples(msaa.samples())
            | PipelineKey::from_primitive_topology(PrimitiveTopology::TriangleStrip);

        view_entities.clear();
        view_entities.extend(visible_entities.entities.iter().map(|e| e.index() as usize));

        transparent_phase
            .items
            .reserve(extracted_shapes.shapes.len());

        for (entity, extracted_shape) in extracted_shapes.shapes.iter() {
            let specialize_key = SmudPipelineKey {
                mesh: mesh_key,
                hdr: view.hdr,
            };
            let pipeline = pipelines.specialize(&pipeline_cache, &smud_pipeline, specialize_key);

            // These items will be sorted by depth with other phase items
            let sort_key = FloatOrd(extracted_shape.transform.translation().z);

            // Add the item to the render phase
            transparent_phase.add(Transparent2d {
                draw_function: draw_smud_shape_function,
                pipeline,
                entity: *entity,
                sort_key,
                // batch_range and dynamic_offset will be calculated in prepare_shapes
                batch_range: 0..0,
                dynamic_offset: None,
            });
        }
    }
}

fn prepare_shapes<PARAMETERS: ParameterizedShader>(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut shape_meta: ResMut<ShapeMeta<PARAMETERS>>,
    view_uniforms: Res<ViewUniforms>,
    smud_pipeline: Res<SmudPipeline<PARAMETERS>>,
    extracted_shapes: Res<ExtractedShapes<PARAMETERS>>,
    mut phases: Query<&mut RenderPhase<Transparent2d>>,
    globals_buffer: Res<GlobalsBuffer>,
) {
    let globals = globals_buffer.buffer.binding().unwrap(); // todo if-let

    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        let mut batches: Vec<(Entity, ShapeBatch)> = Vec::with_capacity(*previous_len);

        // Clear the vertex buffer
        shape_meta.vertices.clear();

        shape_meta.view_bind_group = Some(render_device.create_bind_group(
            "smud_shape_view_bind_group",
            &smud_pipeline.view_layout,
            &BindGroupEntries::sequential((view_binding, globals.clone())),
        ));

        // Vertex buffer index
        let mut index = 0;

        for mut transparent_phase in &mut phases {
            let mut batch_item_index = 0;

            let mut start_new_batch = true;

            // Iterate through the phase items and detect when successive shapes that can be batched.
            // Spawn an entity with a `ShapeBatch` component for each possible batch.
            // Compatible items share the same entity.
            for item_index in 0..transparent_phase.items.len() {
                let item = &transparent_phase.items[item_index];
                let Some(extracted_shape) = extracted_shapes.shapes.get(&item.entity) else {
                    start_new_batch = true;
                    continue;
                };

                let position = extracted_shape.transform.translation();
                let position = position.into();

                let rotation_and_scale = extracted_shape
                    .transform
                    .affine()
                    .transform_vector3(Vec3::X)
                    .xy();

                let scale = rotation_and_scale.length();
                let rotation = (rotation_and_scale / scale).into();

                let vertex = ShapeVertex {
                    position,
                    params: extracted_shape.params,
                    rotation,
                    scale,
                    frame: extracted_shape.frame,
                };

                shape_meta.vertices.push(vertex);

                if start_new_batch {
                    batch_item_index = item_index;

                    batches.push((
                        item.entity,
                        ShapeBatch {
                            range: index..index,
                        },
                    ));
                    start_new_batch = false;
                }

                transparent_phase.items[batch_item_index]
                    .batch_range_mut()
                    .end += 1;

                batches.last_mut().unwrap().1.range.end += 1;
                index += 1;
            }
        }

        shape_meta.vertices.write_buffer(&render_device, &render_queue);

        *previous_len = batches.len();
        commands.insert_or_spawn_batch(batches);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct ShapeVertex<PARAMETERS: ParameterizedShader> {
    pub frame: f32,
    pub params: PARAMETERS,
    pub position: [f32; 3],
    pub rotation: [f32; 2],
    pub scale: f32,
}

unsafe impl<PARAMETERS: ParameterizedShader> Zeroable for ShapeVertex<PARAMETERS> {}

unsafe impl<PARAMETERS: ParameterizedShader> Pod for ShapeVertex<PARAMETERS> {}

#[derive(Resource)]
pub(crate) struct ShapeMeta<PARAMETERS: ParameterizedShader> {
    vertices: BufferVec<ShapeVertex<PARAMETERS>>,
    view_bind_group: Option<BindGroup>,
}

impl<PARAMETERS: ParameterizedShader> Default for ShapeMeta<PARAMETERS> {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}

#[derive(Component, Eq, PartialEq, Clone)]
pub(crate) struct ShapeBatch {
    range: Range<u32>,
}
