#![allow(dead_code, unused_variables)]
use bevy::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use example_common::{prelude::*, toggle_wireframe, toggle_debug_rain, inspector_ui};
use bevy_procedural_landmass::prelude::*;

// Note: Needs debug_rain feature enabled in the procedural_landmass

/// This example shows how to use the endless terrain feature
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,

            // our plugins            
            ProceduralLandmassPlugin,
            TerrainDebugWireframePlugin,
            TerrainDebugRainPlugin,            
            
            ExampleCommonPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (inspector_ui, toggle_wireframe, toggle_debug_rain))
        .run();
}

fn setup(
    mut commands: Commands,    
) {
    // setup camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 200.0, 200.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        CameraController,
        AtmosphereCamera::default(),
                
        FogSettings {
            color: Color::rgba(0.1, 0.2, 0.4, 1.0),
            directional_light_color: Color::rgba(1.0, 0.95, 0.75, 0.5),
            directional_light_exponent: 30.0,
            falloff: FogFalloff::from_visibility_colors(
                500.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
                Color::rgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                Color::rgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
            ),
        },
    ));

    // setup light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(0.0, 1000.0, 1000.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn(TerrainChunkBundle::default());
}
