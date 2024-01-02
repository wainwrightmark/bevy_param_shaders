use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{fragment_shader, parameterized_shader::*, vertex_shader};

pub const fn get_vertex_handle<SHADER: ParameterizedShader>() -> Handle<Shader> {
    let id = 41038750339221851237u128;
    let new_id = id.wrapping_add(SHADER::TYPE_UUID.as_u128().wrapping_mul(2875688479) as u128);

    Handle::weak_from_u128(new_id)
}

pub const fn get_fragment_handle<SHADER: ParameterizedShader>() -> Handle<Shader> {
    let id = 24284142412967609353u128;
    let new_id = id.wrapping_add(SHADER::TYPE_UUID.as_u128().wrapping_mul(2875688479) as u128);

    Handle::weak_from_u128(new_id)
}

pub struct ShaderLoadingPlugin<SHADER: ParameterizedShader>(PhantomData<SHADER>);

impl<SHADER: ParameterizedShader> Default for ShaderLoadingPlugin<SHADER> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<SHADER: ParameterizedShader> Plugin for ShaderLoadingPlugin<SHADER> {
    fn build(&self, app: &mut App) {
        let vertex_shader = vertex_shader::create_vertex_shader::<SHADER>();

        let asset_server = app.world.resource_mut::<AssetServer>();

        for import in SHADER::imports() {
            let shader: Handle<Shader> = asset_server.load(import.path);
            Box::leak(Box::new(shader));
        }

        let mut shaders = app.world.resource_mut::<Assets<Shader>>();
        let fragment_shader = fragment_shader::create_fragment_shader::<SHADER>();

        shaders.insert(get_vertex_handle::<SHADER>(), vertex_shader);
        shaders.insert(get_fragment_handle::<SHADER>(), fragment_shader);
    }
}
