use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::std_options::NumberDisplay, prelude::*};

#[derive(Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct TerrainCurve {
    pub mode: TerrainCurveMode,
    pub invert: bool,
    pub clamp: Vec2,
    #[inspector(min = -1.0, max = 1.0, display = NumberDisplay::Slider)]
    pub offset: f32,
}

impl Default for TerrainCurve {
    fn default() -> Self {
        Self {
            mode: TerrainCurveMode::default(),
            invert: false,
            clamp: Vec2::new(0.0, 1.0),
            offset: 0.0,
        }
    }
}

/// Curve types for terrain generation
#[derive(Clone, PartialEq, Eq, Reflect, Default, InspectorOptions, Debug)]
#[reflect(InspectorOptions)]
pub enum TerrainCurveMode {
    #[default]
    Linear,
    SquareIn,
    SquareOut,
    CubicIn,
    CubicOut,
    CubicInOut,
}

impl TerrainCurve {
    pub fn get(&self, height: f32) -> f32 {
        let mut x = height + self.offset;
        x = match self.mode {
            TerrainCurveMode::Linear => x,
            TerrainCurveMode::SquareIn => x.powi(2),
            TerrainCurveMode::SquareOut => 1.0 - (1.0 - x).powi(2),
            TerrainCurveMode::CubicIn => x.powi(3),
            TerrainCurveMode::CubicOut => 1.0 - (1.0 - x).powi(3),
            TerrainCurveMode::CubicInOut => {
                if x < 0.5 {
                    4.0 * x.powi(3)
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
                }
            }
        };

        // Clamp the value
        x = (x).clamp(self.clamp.x.min(self.clamp.y), self.clamp.x.max(self.clamp.y));
        
        if self.invert {
            1.0 - x
        } else {
            x
        }
    }
}
