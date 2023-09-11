use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

use crate::regions::TerrainRegions;

#[derive(Clone, Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct TerrainGenerator {
    // Terrain generation settings
    pub texture_mode: TerrainTextureMode,
    pub mesh_mode: TerrainMeshMode,
    pub sampler: TerrainSampler,
    #[inspector(min = 1)]
    pub chunk_size: usize,

    #[inspector(min = 0.0, max = 10.0)]
    pub height_multiplier: f32,
    #[inspector(min = 0.0, max = 5.0)]
    pub noise_scale: f32,
    #[inspector(min = 1)]
    pub octaves: usize,
    #[inspector(min = 1.0, max = 10.0)]
    pub lacunarity: f32,
    pub gain: f32,
    pub offset: Vec2,
    pub seed: f32,

    pub world_scale: f32,

    pub regions: TerrainRegions,
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self {
            chunk_size: 10,

            texture_mode: TerrainTextureMode::Color,
            mesh_mode: TerrainMeshMode::Smooth,
            sampler: TerrainSampler::Linear,
            height_multiplier: 0.3,
            noise_scale: 1.0,
            octaves: 4,
            gain: 0.5,
            lacunarity: 2.0,
            seed: 0.0,
            offset: Vec2::ZERO,
            world_scale: 100.0,
            regions: TerrainRegions::default(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Reflect)]
pub enum TerrainTextureMode {
    HeightMap,
    Color,
}

#[derive(Clone, PartialEq, Eq, Debug, Reflect)]
pub enum TerrainMeshMode {
    Flat,
    Smooth,
}

#[derive(Clone, PartialEq, Eq, Debug, Reflect)]
pub enum TerrainSampler {
    Linear,
    Nearest,
}
