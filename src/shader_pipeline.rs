use bevy::{
    core_pipeline::core_2d::CORE_2D_DEPTH_FORMAT, prelude::*, render::{
        globals::GlobalsUniform,
        mesh::BaseMeshPipelineKey,
        render_resource::{
            BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Face, FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState, RenderPipelineDescriptor, ShaderStages, ShaderType, SpecializedRenderPipeline, StencilFaceState, StencilState, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode
        },
        renderer::RenderDevice,
        view::{ViewTarget, ViewUniform},
    }
};

use crate::{parameterized_shader::*, pipeline_key::MeshPipelineKey};

use std::{any::Any, marker::PhantomData};

#[derive(Resource)]
pub(crate) struct ShaderPipeline<Shader: ParameterizedShader> {
    pub view_layout: BindGroupLayout,
    phantom: PhantomData<Shader>,
}

impl<Shader: ParameterizedShader> FromWorld for ShaderPipeline<Shader> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        const ENTRIES_WITH_TIME: &[BindGroupLayoutEntry] = &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(<ViewUniform as ShaderType>::METADATA.min_size().0),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(<GlobalsUniform as ShaderType>::METADATA.min_size().0),
                },
                count: None,
            },
        ];

        const ENTRIES_WITHOUT_TIME: &[BindGroupLayoutEntry] = &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: Some(<ViewUniform as ShaderType>::METADATA.min_size().0),
            },
            count: None,
        }];

        let view_layout = if Shader::USE_TIME {
            render_device.create_bind_group_layout("shape_view_layout", ENTRIES_WITH_TIME)
        } else {
            render_device.create_bind_group_layout("shape_view_layout", ENTRIES_WITHOUT_TIME)
        };

        Self {
            view_layout,
            phantom: PhantomData,
        }
    }
}

// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
// pub(crate) struct ShaderPipelineKey {
//     pub mesh: MeshPipelineKey,
//     pub hdr: bool,
// }

impl<Shader: ParameterizedShader> SpecializedRenderPipeline for ShaderPipeline<Shader> {
    type Key = MeshPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // debug!("specializing for {fragment_shader:?}");

        // an f32 is 4 bytes
        const WORD_BYTE_LENGTH: u64 = 4;

        const ROTATION_WORDS: u64 = 2;
        const POSITION_WORDS: u64 = 3;
        const SCALE_WORDS: u64 = 1;

        const CONSTANT_PARAMS: usize = 3;

        let proxy = <Shader::Params as Default>::default();
        let param_count = proxy.field_len() as u32;

        // (GOTCHA! attributes are sorted alphabetically, and offsets need to reflect this)

        let pre_param_attributes: [VertexAttribute; CONSTANT_PARAMS] = [
            // Rotation
            VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            // Position
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: ROTATION_WORDS * WORD_BYTE_LENGTH,
                shader_location: 1,
            },
            // Scale
            VertexAttribute {
                format: VertexFormat::Float32,
                offset: (ROTATION_WORDS + POSITION_WORDS) * WORD_BYTE_LENGTH,
                shader_location: 2,
            },
        ];

        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position, color and params
        let mut vertex_attributes =
            Vec::with_capacity(pre_param_attributes.len() + param_count as usize);

        vertex_attributes.extend_from_slice(&pre_param_attributes);

        let mut offset = (POSITION_WORDS + ROTATION_WORDS + SCALE_WORDS) * WORD_BYTE_LENGTH;
        let mut shader_location: u32 = CONSTANT_PARAMS as u32;

        for field in proxy.iter_fields() {
            let Some(format) = crate::helpers::get_vertex_format(field) else {
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

        let depth_stencil = Some(DepthStencilState {
            format: CORE_2D_DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
        });

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: crate::shader_loading::get_vertex_handle::<Shader>().clone_weak(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                buffers: vec![VertexBufferLayout {
                    array_stride: offset,
                    step_mode: VertexStepMode::Instance,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                shader: crate::shader_loading::get_fragment_handle::<Shader>().clone_weak(),
                entry_point: "fragment".into(),
                shader_defs: Vec::new(),
                targets: vec![Some(ColorTargetState {
                    format: if key.hdr() {
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
                topology: key.primitive_topology(),
                strip_index_format: None, // TODO: what does this do?
            },
            depth_stencil,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,                         // what does the mask do?
                alpha_to_coverage_enabled: false, // what is this?
            },
            label: Some("param_shader_pipeline".into()),
            push_constant_ranges: Vec::new(),
            zero_initialize_workgroup_memory: false,
        }
    }
}

// Some(DepthStencilState {
//     format: TextureFormat::Stencil8,
//     depth_write_enabled: false,
//     depth_compare: CompareFunction::Always,
//     stencil: StencilState {
//         front: stencil_face_state,
//         back: stencil_face_state,
//         read_mask: 1,
//         write_mask: 1,
//     },
//     bias: default(),
// })
