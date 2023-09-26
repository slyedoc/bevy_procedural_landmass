use bevy::prelude::*;
use bevy_inspector_egui::{InspectorOptions, prelude::*};

#[cfg(debug_rain)]
use crate::{TerrainChunk, util::lerp_color};

pub struct TerrainDebugRainPlugin;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum TerrainDebugRainMode {    
    On,
    #[default]
    Off,
}

impl Plugin for TerrainDebugRainPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<TerrainDebugRainMode>();
        #[cfg(debug_rain)]
        app.add_systems(
            Update,            
            (draw_rain).run_if(in_state(TerrainDebugRainMode::On)),
        );
    }
}

#[derive(Component, Reflect, Default, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct RainPaths(pub Vec<Vec<Vec3>>);

#[cfg(debug_rain)]
pub fn draw_rain(query: Query<(&RainPaths, &TerrainChunk)>, mut gizmos: Gizmos) {
    for (paths, _chunk) in query.iter() {
        for path in paths.0.iter().take(10) {
            for i in 0..path.len() - 1 {
                // TODO: due to gismo draw order, bump points up a little to make them more visable
                // remove once gimzo draw order is fixed
                let p1 = path[i] + Vec3::Y;
                let p2 = path[i + 1] + Vec3::Y;

                // draw line
                gizmos.line(
                    p1,
                    p2,
                    lerp_color(
                        Color::ALICE_BLUE,
                        Color::WHITE,
                        i as f32 / path.len() as f32,
                    ),
                );
                
                // draw sphere at start
                if i == 0 {
                    gizmos.sphere(p1, Quat::IDENTITY, 0.1, Color::GREEN);
                }
                
                // draw sphere at end
                if i == path.len() - 2 {
                    gizmos.sphere(p2, Quat::IDENTITY, 0.1, Color::RED);
                }
            }
        }
    }
}
