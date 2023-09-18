use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

/// A component bundle for entities with a [`Mesh`] and a [`Material`].
#[derive(Bundle, Clone)]
pub struct TerrainChunkBundle {
    pub terrain: TerrainChunk,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
    pub name: Name,
}

impl Default for TerrainChunkBundle {
    fn default() -> Self {
        Self {
            terrain: Default::default(),
            mesh: Default::default(),
            material: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
            name: Default::default(),
        }
    }
}

impl TerrainChunkBundle {
    pub fn new(position: IVec2, visable: bool) -> Self {
        Self {
            terrain: TerrainChunk::new(position),
            visibility: match visable {
                true => Visibility::Visible,
                false => Visibility::Hidden,
            },
            name: Name::new(format!("TerrainChunk: {:?}", position)),
            ..Default::default()
        }
    }
}

#[derive(Clone, Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct TerrainChunk {
    pub position: IVec2,
}

impl Default for TerrainChunk {
    fn default() -> Self {
        Self {
            position: IVec2::ZERO,
        }
    }
}

impl TerrainChunk {
    pub fn new(position: IVec2) -> Self {
        Self { position }
    }
}
