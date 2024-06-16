//#![warn(missing_docs)]
#![allow(clippy::too_many_arguments)]

use std::{marker::PhantomData, ops::Range};

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            StaticSystemParam, SystemParamItem,
        },
    },
    math::Vec3Swizzles,
    prelude::*,
    render::{
        globals::GlobalsBuffer,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupEntries, BufferUsages, BufferVec, PipelineCache, PrimitiveTopology,
            SpecializedRenderPipelines,
        },
        renderer::{RenderDevice, RenderQueue},
        view::{ExtractedView, ViewUniformOffset, ViewUniforms},
        Extract, Render, RenderApp, RenderSet,
    },
    utils::FloatOrd,
};
use bytemuck::{Pod, Zeroable};
use pipeline_key::PipelineKey;
use shader_loading::*;

use parameterized_shader::*;
use shader_params::ShaderParams;
use shader_pipeline::*;

pub use bundle::ShaderBundle;
pub use components::*;

mod bundle;
mod components;
mod fragment_shader;
pub mod frame;
mod helpers;
pub mod parameterized_shader;
mod pipeline_key;
mod shader_loading;
pub mod shader_params;
mod shader_pipeline;
mod vertex_shader;

pub mod primitives;

/// Re-export of the essentials needed for rendering shapes
///
/// Intended to be included at the top of your file to minimize the amount of import noise.
/// ```
/// use bevy_param_shaders::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        frame::Frame, parameterized_shader::*, shader_params::*, ExtractToShaderPlugin,
        ShaderBundle, ShaderUsage,
    };
}

pub struct ExtractToShaderPlugin<Extractable: ExtractToShader>(PhantomData<Extractable>);

impl<Extractable: ExtractToShader> Plugin for ExtractToShaderPlugin<Extractable> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ParamShaderPlugin<Extractable::Shader>>() {
            app.add_plugins(ParamShaderPlugin::<Extractable::Shader>::default());
        }

        //todo in debug mode add a system to check that all shader plugins are registered
        //todo in debug mode add a system to check that all shaders have the right parameters

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(ExtractSchedule, (extract_shapes::<Extractable>,));
        };
    }
}

impl<Extractable: ExtractToShader> Default for ExtractToShaderPlugin<Extractable> {
    fn default() -> Self {
        Self(Default::default())
    }
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

struct ParamShaderPlugin<Shader: ParameterizedShader>(PhantomData<Shader>);

impl<Shader: ParameterizedShader> Default for ParamShaderPlugin<Shader> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Shader: ParameterizedShader> Plugin for ParamShaderPlugin<Shader> {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShaderLoadingPlugin::<Shader>::default());
        if !app.is_plugin_added::<ParameterShadersPlugin>() {
            app.add_plugins(ParameterShadersPlugin);
        }

        //todo in debug mode add a system to check that all shader plugins are registered
        //todo in debug mode add a system to check that all shaders have the right parameters

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Transparent2d, DrawShaderShape<Shader>>()
                .init_resource::<ExtractedShapes<Shader>>()
                .init_resource::<SpecializedRenderPipelines<ShaderPipeline<Shader>>>()
                .add_systems(
                    Render,
                    (
                        (sort_shapes::<Shader>, queue_shapes::<Shader>)
                            .chain()
                            .in_set(RenderSet::Queue),
                        prepare_shapes::<Shader>.in_set(RenderSet::PrepareBindGroups),
                        cleanup_shapes::<Shader>.in_set(RenderSet::Cleanup),
                    ),
                );
        };
        //app.register_type::<Shader>();
    }

    fn finish(&self, app: &mut App) {
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_resource::<ShaderPipeline<Shader>>();
    }
}

type DrawShaderShape<Shader> = (
    SetItemPipeline,
    SetShapeViewBindGroup<0, Shader>,
    DrawShapeBatch<Shader>,
);

struct SetShapeViewBindGroup<const I: usize, Shader: ParameterizedShader>(PhantomData<Shader>);
impl<P: PhaseItem, const I: usize, Shader: ParameterizedShader> RenderCommand<P>
    for SetShapeViewBindGroup<I, Shader>
{
    type Param = SRes<ExtractedShapes<Shader>>;
    type ViewQuery = Read<ViewUniformOffset>;
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        view_uniform: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
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

struct DrawShapeBatch<Shader: ParameterizedShader>(PhantomData<Shader>);
impl<P: PhaseItem, Shader: ParameterizedShader> RenderCommand<P> for DrawShapeBatch<Shader> {
    type Param = SRes<ExtractedShapes<Shader>>;
    type ViewQuery = ();
    type ItemQuery = Read<ShapeBatch>;

    fn render<'w>(
        _item: &P,
        _view: (),
        batch: Option<&'_ ShapeBatch>,
        shape_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let shape_meta = shape_meta.into_inner();
        if let Some(batch) = batch {
            if let Some(buffer) = shape_meta.vertices.buffer() {
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..4, batch.range.clone()); //0..4 as there are four vertices
                RenderCommandResult::Success
            } else {
                warn!("Render Fail {:?}", batch);
                RenderCommandResult::Failure
            }
        } else {
            RenderCommandResult::Failure
        }
    }
}

#[derive(Resource)]
struct ExtractedShapes<Shader: ParameterizedShader> {
    vertices: BufferVec<ShapeVertex<Shader::Params>>,
    view_bind_group: Option<BindGroup>,
}

impl<Shader: ParameterizedShader> Default for ExtractedShapes<Shader> {
    fn default() -> Self {
        Self {
            // shapes: Default::default(),
            vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}

fn extract_shapes<'w, 's, 'a, Extractable: ExtractToShader>(
    mut extracted_shapes: ResMut<ExtractedShapes<Extractable::Shader>>,
    shape_query: Extract<
        Query<
            'w,
            's,
            (
                &ViewVisibility,
                Extractable::ParamsQuery<'a>,
                &GlobalTransform,
            ),
            With<ShaderUsage<Extractable>>,
        >,
    >,
    resource_params: Extract<StaticSystemParam<Extractable::ResourceParams<'w>>>,
) {
    let resource = resource_params;

    for (view_visibility, params_item, transform) in shape_query.iter() {
        if !view_visibility.get() {
            continue;
        }

        let params = Extractable::get_params(params_item, &resource);

        let shape_vertex = ShapeVertex::new(transform, params);

        extracted_shapes.vertices.push(shape_vertex);
    }
}

fn sort_shapes<Shader: ParameterizedShader>(mut extracted_shapes: ResMut<ExtractedShapes<Shader>>) {
    radsort::sort_by_key(extracted_shapes.as_mut().vertices.values_mut(), |item| {
        item.z_index()
    });
}

fn queue_shapes<Shader: ParameterizedShader>(
    mut commands: Commands,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    pipeline: Res<ShaderPipeline<Shader>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<ShaderPipeline<Shader>>>,
    pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    extracted_shapes: Res<ExtractedShapes<Shader>>,
    mut views: Query<(&mut RenderPhase<Transparent2d>, &ExtractedView)>,
) {
    let draw_function = draw_functions
        .read()
        .get_id::<DrawShaderShape<Shader>>()
        .unwrap();

    // Iterate over each view (a camera is a view)
    for (mut transparent_phase, view) in &mut views {
        // todo: bevy_sprite does some hdr stuff, should we?
        // let mut view_key = SpritePipelineKey::from_hdr(view.hdr) | msaa_key;

        let mesh_key = PipelineKey::from_msaa_samples(msaa.samples())
            | PipelineKey::from_primitive_topology(PrimitiveTopology::TriangleStrip);

        let specialize_key = ShaderPipelineKey {
            mesh: mesh_key,
            hdr: view.hdr,
        };
        let pipeline = pipelines.specialize(&pipeline_cache, &pipeline, specialize_key);

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

fn prepare_shapes<Shader: ParameterizedShader>(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    view_uniforms: Res<ViewUniforms>,
    pipeline: Res<ShaderPipeline<Shader>>,
    mut extracted_shapes: ResMut<ExtractedShapes<Shader>>,
    globals_buffer: Res<GlobalsBuffer>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    if Shader::USE_TIME {
        let Some(globals) = globals_buffer.buffer.binding() else {
            return;
        };

        extracted_shapes.view_bind_group = Some(render_device.create_bind_group(
            "param_shader_view_bind_group",
            &pipeline.view_layout,
            &BindGroupEntries::sequential((view_binding, globals.clone())),
        ));
    } else {
        extracted_shapes.view_bind_group = Some(render_device.create_bind_group(
            "param_shader_view_bind_group",
            &pipeline.view_layout,
            &BindGroupEntries::single(view_binding),
        ));
    }

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

fn cleanup_shapes<Shader: ParameterizedShader>(
    mut extracted_shapes: ResMut<ExtractedShapes<Shader>>,
) {
    extracted_shapes.vertices.clear();
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Zeroable)]
struct ShapeVertex<PARAMS: ShaderParams> {
    pub rotation: [f32; 2],
    pub position: [f32; 3],
    pub scale: f32,
    pub params: PARAMS,
}

impl<PARAMS: ShaderParams> ShapeVertex<PARAMS> {
    pub fn new(transform: &GlobalTransform, params: PARAMS) -> Self {
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
