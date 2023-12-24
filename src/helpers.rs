use std::any::TypeId;

use bevy::{
    math::Vec4,
    render::render_resource::VertexFormat,
};

use crate::prelude::{LinearRGBA, ShaderParams};

pub(crate) fn format_params_locations<PARAMS: ShaderParams>() -> String {
    let mut result = "".to_string();

    let proxy = PARAMS::default();
    let param_count = proxy.field_len();

    let mut loc = 1;

    for index in 0..param_count {
        //let t = crate::parameterized_shader::format_to_name(SHADER::get_format(index));
        let name = proxy.name_at(index).unwrap();
        let type_id = proxy.field_at(index).unwrap().type_id();

        let Some(type_name) = get_wgsl_type_name(type_id) else {
            panic!(
                "Cannot convert {} to wgsl type",
                proxy.field_at(index).unwrap().type_name()
            );
        };

        result.push_str(format!("@location({loc}) {name}: {type_name},\n").as_str());
        loc += 1;
    }

    result
}

pub(crate) fn get_wgsl_type_name(type_id: TypeId) -> Option<&'static str> {
    if type_id == TypeId::of::<f32>() {
        Some("f32")
    } else if type_id == TypeId::of::<u32>() {
        Some("u32")
    } else if type_id == TypeId::of::<i32>() {
        Some("i32")
    } else if type_id == TypeId::of::<bevy::math::Vec2>() {
        Some("vec2<f32>")
    } else if type_id == TypeId::of::<bevy::math::Vec3>() {
        Some("vec3<f32>")
    } else if type_id == TypeId::of::<Vec4>() {
        Some("vec4<f32>")
    } else if type_id == TypeId::of::<LinearRGBA>() {
        Some("vec4<f32>")
    } else {
        None
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
    } else {
        None
    }
}
