
use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::std_options::NumberDisplay, prelude::*};
use noisy_bevy::{fbm_simplex_2d_seeded, simplex_noise_2d_seeded};

use crate::util;



#[derive(Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct TerrainNoise {
    pub mode: TerrainNoiseMode,

    #[inspector(min = 0.01, max = 100.0, display = NumberDisplay::Slider)]
    pub scale: f32,
    pub offset: Vec2,
    pub seed: f32,
    /// Curve applied to the noise to allow for more control over the terrain
    pub correction: TerrainCurve,
}

impl Default for TerrainNoise {
    fn default() -> Self {
        Self {
            mode: TerrainNoiseMode::default(),            
            scale: 0.7,
            offset: Vec2::ZERO,
            seed: 0.0,
            correction: TerrainCurve::default(),
        }
    }
}

impl TerrainNoise {
    pub fn get(&self, pos: Vec2, seed: f32) -> f32 {
        let height = match &self.mode {
            TerrainNoiseMode::FMBSimplex(x) => x.get(pos, seed),
            TerrainNoiseMode::Simplex => simplex_noise_2d_seeded(pos, seed),
        };
        self.correction.get(height)
    }
}


#[derive(Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub enum TerrainNoiseMode {
    FMBSimplex(FMBSimplex),
    Simplex,
}

impl Default for TerrainNoiseMode {
    fn default() -> Self {
        TerrainNoiseMode::FMBSimplex(FMBSimplex::default())
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
            octaves: 6,
            lacunarity: 4.0,
            gain: 0.3,
        }
    }
}

impl FMBSimplex {
    pub fn get(&self, pos: Vec2, seed: f32) -> f32 {
        let height = fbm_simplex_2d_seeded(pos, self.octaves, self.lacunarity, self.gain, seed);

        // Clamp height to 0.0 - 1.0, approximate min and max for fbm with simplex noise given octaves, gain, and initial amplitude of 1.
        let approx_max = (1.0 - self.gain.powf(self.octaves as f32)) / (1.0 - self.gain);
        let approx_min = -approx_max;

        util::remap(height, approx_min, approx_max, 0.0, 1.0)
    }
}


pub mod egui {
    use std::any::{Any, TypeId};

    use crate::{egui_helper::many_unimplemented, TerrainCurve, util};
    use bevy::prelude::*;
    use bevy_inspector_egui::{
        egui::plot::{Line, Plot, PlotPoints},
        inspector_egui_impls::InspectorEguiImpl,
        reflect_inspector::InspectorUi,
        *,
    };

    pub fn register_ui(type_registry: &AppTypeRegistry) {
        let mut type_registry = type_registry.write();
        type_registry
            .get_mut(TypeId::of::<TerrainCurve>())
            .unwrap_or_else(|| panic!("{:?} not registered", std::any::type_name::<TerrainCurve>()))
            .insert(InspectorEguiImpl::new(
                curve_ui,
                curve_ui_readonly,
                many_unimplemented::<TerrainCurve>,
            ));
    }

    fn curve_ui(
        value: &mut dyn Any,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) -> bool {
        let value = value.downcast_mut::<TerrainCurve>().unwrap();

        let mut changed = false;

        ui.collapsing("Curve", |ui| {
            ui.horizontal(|ui| {
                ui.label("Invert");
                changed |= env.ui_for_reflect(&mut value.invert, ui);
            });
            ui.horizontal(|ui| {
                ui.label("offset");
                changed |= env.ui_for_reflect(&mut value.offset, ui);
            });
            ui.horizontal(|ui| {
                ui.label("clamp");
                changed |= env.ui_for_reflect(&mut value.clamp, ui);
            });
            ui.horizontal(|ui| {
                ui.label("mode");
                changed |= env.ui_for_reflect(&mut value.mode, ui);
            });

            ui.horizontal(|ui| {
                let sample_size = 100;
                let range = Vec2::new(-0.5, 1.5);
                let points: PlotPoints = (0..sample_size)
                    .map(|i| {
                        let x =
                            util::remap(i as f32 / sample_size as f32, 0.0, 1.0, range.x, range.y);
                        let height = value.get(x);
                        [x as f64, height as f64]
                    })
                    .collect();
                let line = Line::new(points);
                Plot::new("curve")
                    .allow_boxed_zoom(false)
                    .allow_scroll(false)
                    .allow_drag(false)
                    .allow_zoom(false)
                    .view_aspect(1.0)
                    .data_aspect(1.0)
                    .show_y(true)
                    .show_x(true)
                    .min_size(egui::Vec2::splat(1.25))
                    .height(100.0)
                    .show(ui, |plot_ui| plot_ui.line(line));
            })
        });
        changed
    }

    fn curve_ui_readonly(
        value: &dyn Any,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        mut _env: InspectorUi<'_, '_>,
    ) {
        let curve = value.downcast_ref::<TerrainCurve>().unwrap();
        ui.label(format!("curve readonly: {:?}", curve.mode));
    }
}

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

        // Apply the curve
        let mut x = match self.mode {
            TerrainCurveMode::Linear => height,
            TerrainCurveMode::SquareIn => height.powi(2),
            TerrainCurveMode::SquareOut => 1.0 - (1.0 - height).powi(2),
            TerrainCurveMode::CubicIn => height.powi(3),
            TerrainCurveMode::CubicOut => 1.0 - (1.0 - height).powi(3),
            TerrainCurveMode::CubicInOut => {
                if height < 0.5 {
                    4.0 * height.powi(3)
                } else {
                    1.0 - (-2.0 * height + 2.0).powi(3) / 2.0
                }
            }
        };

        // Apply the offset
        x = x + self.offset;
        
        // Clamp the value
        x = (x).clamp(self.clamp.x.min(self.clamp.y), self.clamp.x.max(self.clamp.y));
        
        if self.invert {
            1.0 - x
        } else {
            x
        }
    }
}
