#![allow(dead_code, unused_variables)]
use bevy::{pbr::{NotShadowCaster, PbrPlugin}, prelude::*, core_pipeline::prepass::DepthPrepass};
use bevy_procedural_landmass::prelude::*;
use example_common::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(PbrPlugin {
                    // The prepass is enabled by default on the StandardMaterial,
                    // but you can disable it if you need to.
                    //
                    // prepass_enabled: false,
                    ..default()
                })
                .set(bevy_log_plugin()),
            ProceduralLandmassPlugin,
            TerrainDebugWireframePlugin,
            TerrainDebugRainPlugin,
            WaterPlugin,
            // some quality of life stuff for examples
            ExampleCommonPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (inspector_ui, toggle_wireframe, toggle_debug_rain))
        .insert_resource(Msaa::Sample4)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
) {
    // setup camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 200.0, 200.0).looking_at(Vec3::ZERO, Vec3::Y),
            projection: PerspectiveProjection {
                far: 10_000.0,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        },
        CameraController,
        AtmosphereCamera::default(),
        // added endless terrain to our camera
        EndlessTerrain::default(),
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
        DepthPrepass,
    ));

    // setup light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(0.0, 1000.0, 1000.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // setup water
    commands.spawn((
        MaterialMeshBundle {
            transform: Transform::from_xyz(0.0, 50.0, 0.0), // move water up a bit
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 10000.0,
                subdivisions: 1,
            })),
            material: water_materials.add(WaterMaterial::default()),
            ..default()
        },
        NotShadowCaster,
        Name::new("Water"),
    ));
}
