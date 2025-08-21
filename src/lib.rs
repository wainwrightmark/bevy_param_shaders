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
    math::{FloatOrd, Vec3Swizzles},
    platform::collections::HashSet,
    prelude::*,
    render::{
        globals::GlobalsBuffer,
        render_phase::{
            AddRenderCommand, DrawFunctionId, DrawFunctions, PhaseItem, PhaseItemExtraIndex,
            RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
            ViewSortedRenderPhases,
        },
        render_resource::{
            BindGroup, BindGroupEntries, BufferUsages, PipelineCache, PrimitiveTopology,
            RawBufferVec, SpecializedRenderPipelines,
        },
        renderer::{RenderDevice, RenderQueue},
        view::{ExtractedView, ViewUniformOffset, ViewUniforms},
        Extract, Render, RenderApp, RenderSet,
    },
};
use bytemuck::{NoUninit, Zeroable};
use check_shapes::CheckShapesPlugin;
use pipeline_key::MeshPipelineKey;
use shader_loading::*;

use parameterized_shader::*;
use shader_params::ShaderParams;
use shader_pipeline::*;

pub use bundle::ShaderBundle;
pub use components::*;

pub mod bundle;
mod check_shapes;
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
        bundle::ShaderCheckVisibility, frame::Frame, parameterized_shader::*, shader_params::*,
        ExtractToShaderPlugin, ShaderBundle, ShaderUsage,
    };
}

pub struct ExtractToShaderPlugin<Extractable: ExtractToShader>(PhantomData<Extractable>);

impl<Extractable: ExtractToShader> Plugin for ExtractToShaderPlugin<Extractable> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ParamShaderPlugin<Extractable::Shader>>() {
            app.add_plugins(ParamShaderPlugin::<Extractable::Shader>::default());
        }

        //todo in debug mode add a system to check that all shaders have the right parameters

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(ExtractSchedule, (extract_shapes::<Extractable>,));

            render_app.init_resource::<ShapeBatches<Extractable::Shader>>();
        };
        #[cfg(debug_assertions)]
        {
            info!(
                "Trying to register {}",
                std::any::type_name::<Extractable>()
            );
            let component_id = app
                .world_mut()
                .register_component::<ShaderUsage<Extractable>>();

            if let Some(mut rt) = app
                .world_mut()
                .get_resource_mut::<crate::check_shapes::RegisteredExtractables>()
            {
                bevy::log::info!("Registered {}", std::any::type_name::<Extractable>());
                rt.0.insert(component_id);
            } else {
                let mut set = HashSet::new();
                set.insert(component_id);
                app.insert_resource(crate::check_shapes::RegisteredExtractables(set));
            }
        }
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
        // //todo improve check visibility
        // app.add_systems(
        //     PostUpdate,
        //     bevy::render::view::check_visibility::<With<ShaderCheckVisibility>>
        //         .in_set(bevy::render::view::VisibilitySystems::CheckVisibility),
        // );

        #[cfg(debug_assertions)]
        {
            if !app.is_plugin_added::<CheckShapesPlugin>() {
                app.add_plugins(CheckShapesPlugin);
            }
        }
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

        //todo in debug mode add a system to check that all shaders have the right parameters

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
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
    type Param = (SRes<ExtractedShapes<Shader>>, SRes<ShapeBatches<Shader>>);
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        _view: (),
        _entity: Option<()>,
        (shape_meta, shape_batches): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        //info!("Render!");
        let shape_meta = shape_meta.into_inner();

        let Some(buffer) = shape_meta.vertices.buffer() else {
            warn!("Render Failed: Could not get buffer");
            return RenderCommandResult::Skip;
        };

        for range in shape_batches.ranges.iter() {
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..4, range.clone()); //0..4 as there are four vertices
        }
        RenderCommandResult::Success
    }
}

#[derive(Resource)]
struct ExtractedShapes<Shader: ParameterizedShader> {
    vertices: RawBufferVec<ShapeVertex<Shader::Params>>,
    view_bind_group: Option<BindGroup>,
}

impl<Shader: ParameterizedShader> Default for ExtractedShapes<Shader> {
    fn default() -> Self {
        Self {
            // shapes: Default::default(),
            vertices: RawBufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}

fn extract_shapes<'w, Extractable: ExtractToShader>(
    mut extracted_shapes: ResMut<ExtractedShapes<Extractable::Shader>>,
    shape_query: Extract<
        Query<
            'w,
            '_,
            (
                &ViewVisibility,
                Extractable::ParamsQuery<'_>,
                &GlobalTransform,
            ),
            With<ShaderUsage<Extractable>>,
        >,
    >,
    resource_params: Extract<StaticSystemParam<Extractable::ResourceParams<'w>>>,
) {
    let resource = resource_params;
    //info!("extract shapes a");

    for (view_visibility, params_item, transform) in shape_query.iter() {
        //info!("extract shapes b");
        if !view_visibility.get() {
            //TODO put back
            //info!("Ignoring {view_visibility:?} - Not Visible ",);
            continue;
        }

        let params = Extractable::get_params(params_item, &resource);

        let shape_vertex = ShapeVertex::new(transform, params);

        //info!("Extracting {view_visibility:?} - {shape_vertex:?} ",);

        extracted_shapes.vertices.push(shape_vertex);
    }
}

fn sort_shapes<Shader: ParameterizedShader>(mut extracted_shapes: ResMut<ExtractedShapes<Shader>>) {
    //info!("Sorting {} shapes", extracted_shapes.vertices.len());

    radsort::sort_by_key(extracted_shapes.as_mut().vertices.values_mut(), |item| {
        item.z_index()
    });
}

fn queue_shapes<Shader: ParameterizedShader>(
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    pipeline: Res<ShaderPipeline<Shader>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<ShaderPipeline<Shader>>>,
    pipeline_cache: ResMut<PipelineCache>,
    extracted_shapes: Res<ExtractedShapes<Shader>>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    views: Query<(&ExtractedView, &Msaa)>,
    mut batches: ResMut<ShapeBatches<Shader>>,
) {
    let draw_function: DrawFunctionId = draw_functions
        .read()
        .get_id::<DrawShaderShape<Shader>>()
        .unwrap();
    // info!(
    //     "Queue Shapes a: {} extracted shapes",
    //     extracted_shapes.vertices.len()
    // );

    // Iterate over each view (a camera is a view)
    for (view, msaa) in views.iter() {
        //info!("Queue Shapes b");
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let mesh_key = MeshPipelineKey::from_msaa_samples(msaa.samples())
            | MeshPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleStrip)
            | MeshPipelineKey::from_hdr(view.hdr);

        let pipeline = pipelines.specialize(&pipeline_cache, &pipeline, mesh_key);

        let mut index = 0;
        while let Some(first_shape) = extracted_shapes.vertices.values().get(index) {
            //info!("Queue Shapes c");
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
                //info!("Queue Shapes d");
                index += 1;
            }

            let sort_key = FloatOrd(z);

            batches.ranges.push((start as u32)..(index as u32));

            // Add the item to the render phase
            transparent_phase.add(Transparent2d {
                draw_function,
                pipeline,
                entity: (
                    Entity::PLACEHOLDER,
                    bevy::render::sync_world::MainEntity::from(Entity::PLACEHOLDER),
                ),
                sort_key,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: false,
                extracted_index: start,
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

    //info!("Preparing {} shapes", extracted_shapes.vertices.len());

    extracted_shapes
        .vertices
        .write_buffer(&render_device, &render_queue);
}

fn cleanup_shapes<Shader: ParameterizedShader>(
    mut extracted_shapes: ResMut<ExtractedShapes<Shader>>,
    mut shape_batches: ResMut<ShapeBatches<Shader>>,
) {
    extracted_shapes.vertices.clear();
    shape_batches.ranges.clear();
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

unsafe impl<PARAMS: ShaderParams> NoUninit for ShapeVertex<PARAMS> {}

#[derive(Debug, Resource)]
pub(crate) struct ShapeBatches<Shader: ParameterizedShader> {
    pub ranges: Vec<Range<u32>>,
    phantom: PhantomData<Shader>,
}

impl<Shader: ParameterizedShader> Default for ShapeBatches<Shader> {
    fn default() -> Self {
        Self {
            ranges: Default::default(),
            phantom: PhantomData,
        }
    }
}
