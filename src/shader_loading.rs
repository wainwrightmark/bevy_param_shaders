use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{fragment_shader, parameterized_shader::*, vertex_shader};

const VIEW_BINDINGS_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(11792080578571156967);

pub const DEFAULT_SDF_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(87169507495872744019129766473752);

pub const fn get_vertex_handle<PARAMETERS: ParameterizedShader>() -> Handle<Shader> {
    let id = 41038750339221851237u128;
    let new_id = id.wrapping_add(PARAMETERS::TYPE_UUID.as_u128().wrapping_mul(2875688479) as u128);

    Handle::weak_from_u128(new_id)
}

pub const fn get_fragment_handle<PARAMETERS: ParameterizedShader>() -> Handle<Shader> {
    let id = 24284142412967609353u128;
    let new_id = id.wrapping_add(PARAMETERS::TYPE_UUID.as_u128().wrapping_mul(2875688479) as u128);

    Handle::weak_from_u128(new_id)
}

pub struct ShaderLoadingPlugin<PARAMETERS: ParameterizedShader>(PhantomData<PARAMETERS>);

impl<PARAMETERS: ParameterizedShader> Default for ShaderLoadingPlugin<PARAMETERS> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<PARAMETERS: ParameterizedShader> Plugin for ShaderLoadingPlugin<PARAMETERS> {
    fn build(&self, app: &mut App) {
        let vertex_shader = vertex_shader::create_vertex_shader::<PARAMETERS>();
        let fragment_shader = fragment_shader::create_fragment_shader::<PARAMETERS>();

        let mut shaders = app.world.resource_mut::<Assets<Shader>>();


        shaders.insert(get_vertex_handle::<PARAMETERS>(), vertex_shader);
        shaders.insert(get_fragment_handle::<PARAMETERS>(), fragment_shader);

        // load_internal_asset!(
        //     app,
        //     PRELUDE_SHADER_HANDLE,
        //     "../assets/prelude.wgsl",
        //     Shader::from_wgsl
        // );

        // load_internal_asset!(
        //     app,
        //     SMUD_SHADER_HANDLE,
        //     "../assets/smud.wgsl",
        //     Shader::from_wgsl
        // );

        // load_internal_asset!(
        //     app,
        //     VIEW_BINDINGS_SHADER_HANDLE,
        //     "../assets/view_bindings.wgsl",
        //     Shader::from_wgsl
        // );

        // load_internal_asset!(
        //     app,
        //     DEFAULT_FILL_HANDLE,
        //     "../assets/fills/cubic_falloff.wgsl",
        //     Shader::from_wgsl
        // );

        // load_internal_asset!(
        //     app,
        //     SIMPLE_FILL_HANDLE,
        //     "../assets/fills/simple.wgsl",
        //     Shader::from_wgsl
        // );

        // load_internal_asset!(
        //     app,
        //     DEFAULT_SDF_HANDLE,
        //     "../assets/sdf/anywhere.wgsl",
        //     Shader::from_wgsl
        // );
    }
}
