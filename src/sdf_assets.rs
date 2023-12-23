// use bevy::prelude::*;

// use crate::{util::generate_shader_id, smud_parameters::ParameterizedShape};

// /// Extension trait for Assets<Shader> for conveniently creating new shaders from code
// pub trait SdfAssets {
//     /// Create a sdf shader from the given wgsl body
//     fn add_sdf_body<T: Into<String>,PARAMETERS: ParameterizedShape>(&mut self, sdf: T, params: PARAMETERS) -> Handle<Shader>;
//     /// Create a sdf shader from the given wgsl expression
//     fn add_sdf_expr<T: Into<String>,PARAMETERS: ParameterizedShape>(&mut self, sdf: T, params: PARAMETERS) -> Handle<Shader>;
//     /// Create a fill shader from the given wgsl body
//     fn add_fill_body<T: Into<String>,PARAMETERS: ParameterizedShape>(&mut self, fill: T, params: PARAMETERS) -> Handle<Shader>;
//     /// Create a fill shader from the given wgsl expression
//     fn add_fill_expr<T: Into<String>,PARAMETERS: ParameterizedShape>(&mut self, fill: T, params: PARAMETERS) -> Handle<Shader>;
// }

// impl SdfAssets for Assets<Shader> {
//     fn add_sdf_body<T: Into<String>,PARAMETERS: ParameterizedShape>(&mut self, sdf: T, params: PARAMETERS) -> Handle<Shader> {
//         let body = sdf.into();
//         let id = generate_shader_id();
//         let param_args = params.func_def_arguments::<true, PARAMETERS>();

//         let shader = Shader::from_wgsl(
//             format!(
//                 r#"
// #define_import_path smud::sdf{id}

// #import smud

// fn sdf(p: vec2<f32>{param_args}) -> f32 {{
//     {body}
// }}
// "#
//             ),
//             file!(),
//         );
//         self.add(shader)
//     }

//     fn add_fill_body<T: Into<String>,PARAMETERS: ParameterizedShape>(&mut self, fill: T, params: PARAMETERS) -> Handle<Shader> {
//         let body = fill.into();
//         let id = generate_shader_id();
//         let param_args = params.func_def_arguments::<false, PARAMETERS>();

//         let shader = Shader::from_wgsl(
//             format!(
//                 r#"
// #define_import_path smud::fill{id}

// #import smud

// fn fill(d: f32, color: vec4<f32>, p: vec2<f32>{param_args}) -> vec4<f32> {{
//     {body}
// }}
// "#
//             ),
//             file!(),
//         );
//         self.add(shader)
//     }

//     fn add_sdf_expr<T: Into<String>,PARAMETERS: ParameterizedShape>(&mut self, sdf: T, params: PARAMETERS) -> Handle<Shader> {
//         let e = sdf.into();
//         self.add_sdf_body::<String, PARAMETERS>(format!("return {e};"), params)
//     }

//     fn add_fill_expr<T: Into<String>,PARAMETERS: ParameterizedShape>(&mut self, fill: T, params: PARAMETERS) -> Handle<Shader> {
//         let e = fill.into();
//         self.add_fill_body::<String, PARAMETERS>(format!("return {e};"), params)
//     }
// }
