use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

#[derive(Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct TerrainRegions( pub Vec<TerrainType>);

impl Default for TerrainRegions {
    fn default() -> Self {
        Self {
            0: vec![
                TerrainType {
                    name: "Water".to_string(),
                    color: Color::rgb(0.0, 0.0, 0.5),
                    height: 0.1,
                },
                TerrainType {
                    name: "Sand".to_string(),
                    color: Color::rgb(0.9, 0.9, 0.5),
                    height: 0.2,
                },
                TerrainType {
                    name: "Grass".to_string(),
                    color: Color::rgb(0.0, 0.5, 0.0),
                    height: 0.4,
                },
                TerrainType {
                    name: "Forest".to_string(),
                    color: Color::rgb(0.0, 0.25, 0.0),
                    height: 0.6,
                },
                TerrainType {
                    name: "Rock".to_string(),
                    color: Color::rgb(0.5, 0.5, 0.5),
                    height: 0.8,
                },
                TerrainType {
                    name: "Snow".to_string(),
                    color: Color::rgb(1.0, 1.0, 1.0),
                    height: 1.0,
                },
            ],
        }
    }
}

impl TerrainRegions {
    pub fn get_color(&self, height: f32) -> Color {
        let mut color = Color::BLACK;
        for region in &self.0 {
            if height <= region.height {
                color = region.color;
                break;
            }
        }
        color
    }
}

#[derive(Clone, Debug, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct TerrainType {
    pub name: String,
    pub color: Color,
    #[inspector(min = 0.0, max = 1.0, speed = 0.01)]    
    pub height: f32,
}