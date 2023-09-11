#![allow(dead_code, unused_variables)]
use bevy::{prelude::*, input::keyboard};
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    // origin marker
    commands.spawn((
        PbrBundle {
            material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 10.0,
                subdivisions: 10,
            })),
            ..default()
        },
        Name::new("Origin"),
    ));

    // commands.spawn((
    //     TerrainChunkBundle {
    //         terrain: TerrainChunk::default(),
    //         transform: Transform {
    //             scale: Vec3::splat(100.0),
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     Name::new("TerrainChunk"),
    // ));
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