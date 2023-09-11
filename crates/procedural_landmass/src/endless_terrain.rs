use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::prelude::*;

#[derive(Clone, Component, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct EndlessTerrain {
    pub max_view_distance: f32,
    #[reflect(ignore)]
    pub chunks_visable_in_view_distance: usize,
    #[reflect(ignore)]
    pub terrain_chunks: HashMap<IVec2, Entity>
}

impl Default for EndlessTerrain {
    fn default() -> Self {
        Self {
            max_view_distance: 500.0,
            chunks_visable_in_view_distance: 5,
            terrain_chunks: HashMap::default(),
        }
    }
}
