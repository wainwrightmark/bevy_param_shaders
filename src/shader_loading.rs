use std::marker::PhantomData;

use bevy::{prelude::*, utils::HashSet};

use crate::{fragment_shader, parameterized_shader::*, vertex_shader};

pub const fn get_vertex_asset_id<Shader: ParameterizedShader>(
) -> AssetId<bevy::render::render_resource::Shader> {
    let id = 41038750339221851237u128;
    let new_id = id.wrapping_add(Shader::UUID.wrapping_mul(2875688479));

    AssetId::Uuid{ uuid: uuid::Uuid::from_u128(new_id)}
}

pub const fn get_fragment_asset_id<Shader: ParameterizedShader>(
) -> AssetId<bevy::render::render_resource::Shader> {
    let id = 24284142412967609353u128;
    let new_id = id.wrapping_add(Shader::UUID.wrapping_mul(2875688479));

    AssetId::Uuid{ uuid: uuid::Uuid::from_u128(new_id)}
}

pub const fn get_vertex_handle<Shader: ParameterizedShader>(
) -> Handle<bevy::render::render_resource::Shader> {
    Handle::Weak( get_vertex_asset_id::<Shader>())
}

pub const fn get_fragment_handle<Shader: ParameterizedShader>(
) -> Handle<bevy::render::render_resource::Shader> {
    Handle::Weak( get_fragment_asset_id::<Shader>())
}

pub struct ShaderLoadingPlugin<Shader: ParameterizedShader> {
    phantom: PhantomData<Shader>,
}

impl<Shader: ParameterizedShader> Default for ShaderLoadingPlugin<Shader> {
    fn default() -> Self {
        Self {
            phantom: Default::default(),
        }
    }
}

#[derive(Debug, Resource, Default)]
struct LoadedShaderHandles {
    set: HashSet<Handle<bevy::render::render_resource::Shader>>,
}

impl<Shader: ParameterizedShader> Plugin for ShaderLoadingPlugin<Shader> {
    fn build(&self, app: &mut App) {

        app.init_resource::<LoadedShaderHandles>();
        let vertex_shader = vertex_shader::create_vertex_shader::<Shader>();


        let asset_server = app.world_mut().resource_mut::<AssetServer>();

        let mut handles: Vec<Handle<bevy::render::render_resource::Shader>> = vec![];

        for import in Shader::imports() {
            let handle: Handle<bevy::render::render_resource::Shader> =
                asset_server.load(import.path);

            handles.push(handle);
        }

        let mut handles_res = app.world_mut().resource_mut::<LoadedShaderHandles>();

        handles_res.set.extend(handles);

        let mut shaders = app
            .world_mut()
            .resource_mut::<Assets<bevy::render::render_resource::Shader>>();
        let fragment_shader = fragment_shader::create_fragment_shader::<Shader>();

        shaders.insert(get_vertex_asset_id::<Shader>(), vertex_shader);
        shaders.insert(get_fragment_asset_id::<Shader>(), fragment_shader);
    }
}
