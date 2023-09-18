use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::std_options::NumberDisplay, prelude::*};

/// Curve types for terrain generation
#[derive(Clone, Reflect, Default, InspectorOptions)]
#[reflect(InspectorOptions)]
pub enum TerrainCurve {
    #[default]
    Linear,
    Inverse,
    SquareIn,
    SquareOut,
    CubicIn,
    CubicOut,
    CubicInOut,
}

impl TerrainCurve {
    pub fn get(&self, height: f32) -> f32 {
        match self {
            TerrainCurve::Linear => height,
            TerrainCurve::Inverse => 1.0 - height,
            TerrainCurve::SquareIn => height.powi(2),
            TerrainCurve::SquareOut => 1.0 - (1.0 - height).powi(2),
            TerrainCurve::CubicIn => height.powi(3),
            TerrainCurve::CubicOut => 1.0 - (1.0 - height).powi(3),
            TerrainCurve::CubicInOut => {
                if height < 0.5 {
                    4.0 * height.powi(3)
                } else {
                    1.0 - (-2.0 * height + 2.0).powi(3) / 2.0
                }
            },
            
        }
    }
}
