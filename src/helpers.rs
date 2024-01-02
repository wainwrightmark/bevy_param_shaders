use std::any::TypeId;

use bevy::{math::Vec4, render::render_resource::VertexFormat};

use crate::{
    prelude::*,
    shader_params::{LinearRGB, LinearRGBA},
};

pub(crate) fn format_params_locations<PARAMS: ShaderParams>(previous_params: u32) -> String {
    let mut result = "".to_string();

    let proxy = PARAMS::default();
    let param_count = proxy.field_len();

    let mut loc = previous_params;

    for index in 0..param_count {
        //let t = crate::parameterized_shader::format_to_name(SHADER::get_format(index));
        let name = proxy.name_at(index).unwrap();
        let type_id = proxy.field_at(index).unwrap().type_id();

        let Some(type_name) = get_wgsl_type_name(type_id) else {
            let field = proxy.field_at(index).unwrap();
            let name = field
                .get_represented_type_info()
                .map(|info| info.type_path())
                .unwrap_or_else(|| field.reflect_type_path());

            panic!("Cannot convert {name} to wgsl type",);
        };

        result.push_str(format!("@location({loc}) {name}: {type_name},\n").as_str());
        loc += 1;
    }

    result
}

pub(crate) fn get_wgsl_type_name(type_id: TypeId) -> Option<&'static str> {
    let vertex_format = get_vertex_format(type_id)?;

    match vertex_format {
        VertexFormat::Float32 => Some("f32"),
        VertexFormat::Float32x2 => Some("vec2<f32>"),
        VertexFormat::Float32x3 => Some("vec3<f32>"),
        VertexFormat::Float32x4 => Some("vec4<f32>"),
        VertexFormat::Uint32 => Some("u32"),
        VertexFormat::Uint32x2 => Some("vec2<u32>"),
        VertexFormat::Uint32x3 => Some("vec3<u32>"),
        VertexFormat::Uint32x4 => Some("vec4<u32>"),
        VertexFormat::Sint32 => Some("i32"),
        VertexFormat::Sint32x2 => Some("vec2<i32>"),
        VertexFormat::Sint32x3 => Some("vec3<i32>"),
        VertexFormat::Sint32x4 => Some("vec4<i32>"),
        VertexFormat::Float64 => Some("f64"),
        VertexFormat::Float64x2 => Some("vec2<f64>"),
        VertexFormat::Float64x3 => Some("vec3<f64>"),
        VertexFormat::Float64x4 => Some("vec4<f64>"),
        _ => None,
    }
}

pub(crate) fn get_vertex_format(type_id: TypeId) -> Option<VertexFormat> {
    if type_id == TypeId::of::<f32>() {
        Some(VertexFormat::Float32)
    } else if type_id == TypeId::of::<u32>() {
        Some(VertexFormat::Uint32)
    } else if type_id == TypeId::of::<i32>() {
        Some(VertexFormat::Sint32)
    } else if type_id == TypeId::of::<bevy::math::Vec2>() {
        Some(VertexFormat::Float32x2)
    } else if type_id == TypeId::of::<bevy::math::Vec3>() {
        Some(VertexFormat::Float32x3)
    } else if type_id == TypeId::of::<Vec4>() {
        Some(VertexFormat::Float32x4)
    } else if type_id == TypeId::of::<LinearRGBA>() {
        Some(VertexFormat::Float32x4)
    } else if type_id == TypeId::of::<LinearRGB>() {
        Some(VertexFormat::Float32x3)
    } else {
        None
    }
}
