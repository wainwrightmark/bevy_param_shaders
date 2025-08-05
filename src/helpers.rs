use std::any::{type_name, Any, TypeId};

use bevy::{color::LinearRgba, log::{info, warn}, math::Vec4, reflect::{PartialReflect, TypeRegistry}, render::render_resource::VertexFormat};

use crate::{primitives::ShaderColor, shader_params::*};

pub(crate) fn format_params_locations<PARAMS: ShaderParams>(previous_params: u32) -> String {
    let mut result = "".to_string();

    let proxy = PARAMS::default();
    let param_count = proxy.field_len();

    let mut loc = previous_params;

    for index in 0..param_count {
        let name = proxy.name_at(index).unwrap();
        let field = proxy.field_at(index).unwrap();

        let Some(type_name) = get_wgsl_type_name(field) else {
            let field_type_path = field
                .get_represented_type_info()
                .map(|info| info.type_path())
                .unwrap_or_else(|| field.reflect_type_path());

            panic!("Cannot convert {}.{name} ({field_type_path}) to wgsl type", type_name::<PARAMS>());
        };

        result.push_str(format!("@location({loc}) {name}: {type_name},\n").as_str());
        loc += 1;
    }

    result
}

pub(crate) fn get_wgsl_type_name(field: &dyn PartialReflect) -> Option<&'static str> {
    let vertex_format = get_vertex_format(field)?;

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

pub(crate) fn get_vertex_format(field: &dyn PartialReflect) -> Option<VertexFormat> {

    if field.represents::<f32>(){
        Some(VertexFormat::Float32)
    } else if field.represents::<u32>() {
        Some(VertexFormat::Uint32)
    } else if field.represents::<i32>() {
        Some(VertexFormat::Sint32)
    } else if field.represents::<bevy::math::Vec2>() {
        Some(VertexFormat::Float32x2)
    } else if field.represents::<bevy::math::Vec3>() {
        Some(VertexFormat::Float32x3)
    } else if field.represents::<Vec4>() {
        Some(VertexFormat::Float32x4)
    } else if field.represents::<bevy::color::LinearRgba>() {
        Some(VertexFormat::Float32x4)
    } else {
        warn!("Field does not have vertex format defined");
        //warn!("Type '{:?}' of {:?}", TypeId::of::<bevy::color::LinearRgba>(), type_name::<bevy::color::LinearRgba>());
        None
    }
}
