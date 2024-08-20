use std::time::Duration;

use bevy::{ecs::component::ComponentId, prelude::*};

pub(crate) struct CheckShapesPlugin;

impl Plugin for CheckShapesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, check_extractables);
    }
}

#[derive(Debug, Resource)]
pub(crate) struct RegisteredExtractables(pub bevy::utils::HashSet<ComponentId>);

//Checks that all shaders have been registered every 3 seconds
#[allow(clippy::needless_pass_by_value)]
fn check_extractables(
    world: &World,
    transitions: Option<Res<RegisteredExtractables>>,
    time: Res<Time>,
    mut remaining_time: Local<Duration>,
) {
    if let Some(new_remaining) = remaining_time.checked_sub(time.delta()) {
        *remaining_time = new_remaining;
    } else {
        *remaining_time = Duration::from_secs(3);

        for component in world.components().iter().filter(|component_info| {
            component_info
                .name()
                .starts_with("bevy_param_shaders::components::ShaderUsage<")
        }) {
            let is_registered = match &transitions {
                Some(r) => r.0.contains(&component.id()),
                None => false,
            };

            if !is_registered {
                warn!("Unregistered Shader Extractable: {}", component.name());
            }
        }
    }
}
