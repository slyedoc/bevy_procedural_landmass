#![allow(dead_code, unused_variables)]
use bevy::prelude::*;
use example_common::prelude::*;
use procedural_landmass::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ProceduralLandmassPlugin,
            ExampleCommonPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_wireframe)
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
    ));

    // setup light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(0.0, 1000.0, 1000.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn(TerrainChunkBundle::default());
}

fn toggle_wireframe(
    terrain_wireframe: Res<State<TerrainWireframeMode>>,
    mut next_state: ResMut<NextState<TerrainWireframeMode>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Key1) {
        match terrain_wireframe.get() {
            TerrainWireframeMode::On => next_state.set(TerrainWireframeMode::Off),
            TerrainWireframeMode::Off => next_state.set(TerrainWireframeMode::On),
        }
    }
}
