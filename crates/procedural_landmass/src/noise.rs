use bevy::prelude::*;
use bevy_inspector_egui::{prelude::*, inspector_options::std_options::NumberDisplay};
use noisy_bevy::{fbm_simplex_2d_seeded, simplex_noise_2d_seeded};

#[derive(Clone, Reflect, InspectorOptions)]
#[reflect(Default, InspectorOptions)]
pub enum TerrainNoise {
    FMBSimplex(FMBSimplex),
    Simplex
}

impl Default for TerrainNoise {
    fn default() -> Self {
        TerrainNoise::FMBSimplex(FMBSimplex::default())
    }
}

impl TerrainNoise {
    pub fn get(&self, pos: Vec2, seed: f32) -> f32 {
         match self {
            TerrainNoise::FMBSimplex(x) => x.get(pos, seed),
            TerrainNoise::Simplex => simplex_noise_2d_seeded(pos, seed),
        }
    }
}

#[derive(Clone, Reflect, InspectorOptions)]
#[reflect(Default, InspectorOptions)]
pub struct FMBSimplex {
    #[inspector(min = 1, max = 10, display = NumberDisplay::Slider)]
    pub octaves: usize,
    #[inspector(min = 1.0, max = 10.0, display = NumberDisplay::Slider)]
    pub lacunarity: f32,
    #[inspector(min = 0.01, max = 0.9, display = NumberDisplay::Slider)]
    pub gain: f32,
}

impl Default for FMBSimplex {
    fn default() -> Self {
        Self {
            octaves: 4,
            lacunarity: 0.5,
            gain: 0.5,
        }
    }
}

impl FMBSimplex {
    pub fn get(&self, pos: Vec2, seed: f32) -> f32 {
        let height = fbm_simplex_2d_seeded(pos, self.octaves, self.lacunarity, self.gain, seed);
        
        // Approximate min and max for fbm with simplex noise given octaves, gain, and initial amplitude of 1.
        let approx_max = (1.0 - self.gain.powf(self.octaves as f32)) / (1.0 - self.gain);
        let approx_min = -approx_max;
        
        remap(height, approx_min, approx_max, 0.0, 1.0)
    }
}

/// Remaps a value from one range to another range.
fn remap(value: f32, original_min: f32, original_max: f32, target_min: f32, target_max: f32) -> f32 {
    target_min + (value - original_min) * (target_max - target_min) / (original_max - original_min)
}
