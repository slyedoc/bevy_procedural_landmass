mod chunk;
mod debug;
mod egui_helper;
mod endless;
mod erosion;
mod generator;
mod noise;
mod regions;
mod util;
mod water;
use std::sync::Arc;

use debug::RainPaths;
use noise::*;
use regions::*;

use erosion::*;

// public stuff
pub use chunk::*;
pub use endless::*;

use generator::{TerrainGenerator, TerrainSampler};

use bevy::{
    prelude::*,
    render::{
        primitives::Aabb,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;

use crate::generator::TerrainTextureMode;

pub mod prelude {
    pub use crate::{
        chunk::TerrainChunkBundle,
        debug::{
            TerrainDebugRainMode, TerrainDebugRainPlugin, TerrainDebugWireframePlugin,
            TerrainWireframeMode,
        },
        endless::EndlessTerrain,
        erosion::*,
        generator::{TerrainGenerator, TerrainSampler},
        noise::*,
        regions::*,
        util::*,
        ProceduralLandmassPlugin,
        water::*
    };
}

type NoiseMap = Vec<Vec<f32>>;

pub struct ProceduralLandmassPlugin;

impl Plugin for ProceduralLandmassPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreUpdate, (update_endless, create_chunks).chain())
            .add_systems(Update, (update_chunk_visablity, generator_changed).chain())
            .add_systems(Update, (spawn_chunk_tasks, handle_check_tasks))
            .insert_resource(TerrainGenerator::default())
            //.register_type::<TerrainGenerator>()
            .register_type::<EndlessTerrain>()
            .register_type::<TerrainChunk>()
            .register_type::<TerrainRegions>()
            .register_type::<TerrainType>()
            .register_type::<TerrainErosion>()
            .register_type::<TerrainCurve>()
            .register_type::<TerrainCurveMode>()
            .register_type::<TerrainNoise>()
            .register_type::<TerrainNoiseMode>()
            .register_type::<FMBSimplex>()
            .register_type::<HydraulicErosion>();

        // add custom renders
        //let type_registry = app.world.resource::<AppTypeRegistry>();
        //noise::egui::register_ui(type_registry);
    }
}

fn update_endless(mut query: Query<&mut EndlessTerrain>, generator: Res<TerrainGenerator>) {
    if let Ok(mut endless) = query.get_single_mut() {
        if generator.is_changed() || endless.is_changed() {
            endless.chunks_visable_in_view_distance =
                (endless.max_view_distance / generator.world_scale as f32) as usize;
        }
    }
}

fn generator_changed(generator: Res<TerrainGenerator>, mut query: Query<&mut TerrainChunk>) {
    if generator.is_changed() {
        for mut chunk in query.iter_mut() {
            chunk.set_changed()
        }
    }
}

fn create_chunks(
    mut commands: Commands,
    mut endless_query: Query<(&mut EndlessTerrain, &GlobalTransform)>,
    generator: Res<TerrainGenerator>,
) {
    if let Ok((mut endless, trans)) = endless_query.get_single_mut() {
        let current_chunk_coord_x = (trans.translation().x / generator.world_scale as f32) as i32;
        let current_chunk_coord_y = (trans.translation().z / generator.world_scale as f32) as i32;

        let chunks_visable_in_view_distance = endless.chunks_visable_in_view_distance as i32;
        for y in -chunks_visable_in_view_distance..=chunks_visable_in_view_distance {
            for x in -chunks_visable_in_view_distance..=chunks_visable_in_view_distance {
                let chunk_coord = IVec2::new(current_chunk_coord_x + x, current_chunk_coord_y + y);

                if endless.terrain_chunks.contains_key(&chunk_coord) {
                    // TODO
                } else {
                    let entity = commands
                        .spawn((TerrainChunkBundle::new(chunk_coord, false),))
                        .id();
                    endless.terrain_chunks.insert(chunk_coord, entity);
                }
            }
        }
    }
}

fn update_chunk_visablity(
    mut query: Query<(&mut Visibility, &Transform), With<TerrainChunk>>,
    mut endless_query: Query<(&EndlessTerrain, &Transform)>,
    generator: Res<TerrainGenerator>,
    //mut gizmos: Gizmos
) {
    if let Ok((endless, e_trans)) = endless_query.get_single_mut() {
        let endless_translation = e_trans.translation;
        let half_world_size = generator.world_scale as f32 / 2.0;

        // check terrain corners
        let corners = vec![
            Vec3::new(-half_world_size, 0.0, -half_world_size),
            Vec3::new(half_world_size, 0.0, -half_world_size),
            Vec3::new(-half_world_size, 0.0, half_world_size),
            Vec3::new(half_world_size, 0.0, half_world_size),
        ];

        // gizmos.circle( Vec3::new(endless_translation.x, 0.0, endless_translation.z), Vec3::Y, endless.max_view_distance, Color::GREEN);

        for (mut vis, chunk_trans) in query.iter_mut() {
            let chuck_translation = chunk_trans.translation;

            let mut visable = false;
            for corner in corners.iter() {
                // gizmos.line( chuck_translation + *corner, chuck_translation + vec3(0.0, 50.0, 0.0), Color::RED);
                if endless_translation.distance(chuck_translation + *corner)
                    <= endless.max_view_distance
                {
                    visable = true;
                }
            }

            *vis = match visable {
                true => Visibility::Visible,
                false => Visibility::Hidden,
            }
        }
    }
}

struct ComputeResult {
    image: Image,
    mesh: Mesh,
    #[allow(dead_code)]
    noise_map: NoiseMap,
    world_scale: f32,
    rain_paths: Option<Vec<Vec<Vec3>>>,
}

#[derive(Component)]
struct ComputeChunk(Task<ComputeResult>);

fn spawn_chunk_tasks(
    mut commands: Commands,
    query: Query<(Entity, &TerrainChunk, Option<&ComputeChunk>), Changed<TerrainChunk>>,
    generator: ResMut<TerrainGenerator>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    // create a arc of the generator to share with the thread pool
    let generator_arc = Arc::new(generator.clone());
    for (e, chunk, compute) in query.iter() {
        if compute.is_some() {
            // drop the old task
            commands.entity(e).remove::<ComputeChunk>();
        }
        let chunk = chunk.clone();
        let generator = generator_arc.clone();

        let task = thread_pool.spawn(async move {
            // create noise map
            #[allow(unused_mut)]
            let mut noise_map = generator.generate_noise_map(chunk.position);

            let rain_paths = generator.generate_erosion(&mut noise_map);

            // create image
            let image_data = match generator.texture_mode {
                TerrainTextureMode::Color => generator.generate_color_map_image(&noise_map),
                TerrainTextureMode::HeightMap => generator.generate_height_map_image(&noise_map),
            };
            let mut image = Image::new(
                Extent3d {
                    width: generator.chunk_size as u32,
                    height: generator.chunk_size as u32,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                image_data,
                TextureFormat::Rgba8UnormSrgb,
            );

            image.sampler_descriptor = match generator.sampler {
                TerrainSampler::Linear => ImageSampler::linear(),
                TerrainSampler::Nearest => ImageSampler::nearest(),
            };

            // create the mesh
            let mesh = generator.generate_mesh(&noise_map);

            ComputeResult {
                image,
                mesh,
                noise_map,
                world_scale: generator.world_scale,
                rain_paths,
            }
        });
        commands.entity(e).insert(ComputeChunk(task));
    }
}

fn handle_check_tasks(
    mut commands: Commands,
    mut chunk_tasks: Query<(
        Entity,
        &TerrainChunk,
        &mut ComputeChunk,
        &mut Transform,
        &mut Handle<StandardMaterial>,
        &mut Handle<Mesh>,
    )>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    generator: Res<TerrainGenerator>,
) {
    // create the material
    for (e, chunk, mut task, mut trans, mut material, mut mesh) in &mut chunk_tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            // update the transform
            trans.translation = Vec3::new(
                chunk.position.x as f32 * result.world_scale,
                0.0,
                chunk.position.y as f32 * result.world_scale,
            );

            // update material
            *material = materials.add(StandardMaterial {
                base_color_texture: Some(images.add(result.image)),
                base_color: match generator.texture_mode {
                    TerrainTextureMode::Color => Color::WHITE,
                    TerrainTextureMode::HeightMap => Color::WHITE,
                },
                perceptual_roughness: 1.0,
                unlit: match generator.texture_mode {
                    TerrainTextureMode::Color => false,
                    TerrainTextureMode::HeightMap => true,
                },
                ..Default::default()
            });

            // update mesh
            *mesh = meshes.add(result.mesh);

            // TODO: remove this, using it to debug right now
            if let Some(paths) = result.rain_paths {
                commands.entity(e).insert(RainPaths(paths));
            }

            // Update AABB
            // Hack: See https://github.com/bevyengine/bevy/issues/4294
            commands.entity(e).remove::<Aabb>();

            // Task is complete, so remove task component from entity
            commands.entity(e).remove::<ComputeChunk>();
        }
    }
}
