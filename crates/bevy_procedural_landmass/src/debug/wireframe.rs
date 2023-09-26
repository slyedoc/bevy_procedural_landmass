use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
};

use crate::TerrainChunk;

pub struct TerrainDebugWireframePlugin;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum TerrainWireframeMode {
    On,
    #[default]
    Off,
}

impl Plugin for TerrainDebugWireframePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<WireframePlugin>() {            
            app.add_plugins(WireframePlugin);
        } 
        app.add_state::<TerrainWireframeMode>()
            .add_systems(
                Update,
                (add_wireframes).run_if(in_state(TerrainWireframeMode::On)),
            )
            .add_systems(OnEnter(TerrainWireframeMode::Off), remove_wireframe);
    }
}

fn add_wireframes(
    mut commands: Commands,
    mut query: Query<Entity, (With<TerrainChunk>, Without<Wireframe>)>,
) {
    for entity in query.iter_mut() {
        commands.entity(entity).insert(Wireframe);
    }
}

fn remove_wireframe(
    mut commands: Commands,
    query: Query<Entity, (With<TerrainChunk>, With<Wireframe>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<Wireframe>();
    }
}
