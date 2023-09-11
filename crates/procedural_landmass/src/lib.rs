mod debug;
mod endless_terrain;
mod regions;
mod terrain_chunk;
mod terrain_generator;

use std::sync::Arc;

use debug::wireframe::TerrainDebugWireframePlugin;
pub use endless_terrain::*;
pub use regions::*;
pub use terrain_chunk::*;
use terrain_generator::{TerrainGenerator, TerrainSampler};

use bevy::{
    prelude::*,
    render::{
        primitives::Aabb,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use futures_lite::future;

pub use crate::debug::wireframe::TerrainWireframeMode;
use crate::terrain_generator::TerrainTextureMode;

type NoiseMap = Vec<Vec<f32>>;

pub struct ProceduralLandmassPlugin;

impl Plugin for ProceduralLandmassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TerrainDebugWireframePlugin,
            ResourceInspectorPlugin::<TerrainGenerator>::default(),
        ))
        .add_systems(PreUpdate, (update_endless, create_chunks).chain())
        .add_systems(Update, (update_chunk_visablity, generator_changed).chain())
        .add_systems(Update, (spawn_chunk_tasks, handle_check_tasks))
        .insert_resource(TerrainGenerator::default())
        .register_type::<TerrainGenerator>()
        .register_type::<EndlessTerrain>()
        .register_type::<TerrainChunk>()
        .register_type::<TerrainRegions>();
    }
}

fn update_endless(mut query: Query<&mut EndlessTerrain>, generator: Res<TerrainGenerator>) {
    if let Ok(mut endless) = query.get_single_mut() {
        if generator.is_changed() || endless.is_changed() {
            endless.chunks_visable_in_view_distance =
                (endless.max_view_distance / generator.chunk_size as f32) as usize;
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
        let current_chunk_coord_x = (trans.translation().x / generator.chunk_size as f32) as i32;
        let current_chunk_coord_y = (trans.translation().z / generator.chunk_size as f32) as i32;

        let chunks_visable_in_view_distance = endless.chunks_visable_in_view_distance as i32;
        for y_offset in -chunks_visable_in_view_distance..=chunks_visable_in_view_distance {
            for x_offset in -chunks_visable_in_view_distance..=chunks_visable_in_view_distance {
                let chunk_coord = IVec2::new(
                    current_chunk_coord_x + x_offset,
                    current_chunk_coord_y + y_offset,
                );

                if endless.terrain_chunks.contains_key(&chunk_coord) {
                    // TODO
                } else {
                    let entity = commands
                        .spawn((TerrainChunkBundle::new(
                            chunk_coord,
                            generator.chunk_size,
                            false,
                        ),))
                        .id();
                    endless.terrain_chunks.insert(chunk_coord, entity);
                }
            }
        }
    }
}

fn update_chunk_visablity(
    mut query: Query<(&mut Visibility, &GlobalTransform), With<TerrainChunk>>,
    mut endless_query: Query<(&EndlessTerrain, &GlobalTransform)>,
    // mut gizmos: Gizmos
) {
    if let Ok((endless, e_trans)) = endless_query.get_single_mut() {
        let endless_translation = e_trans.translation();
        // gizmos.circle( Vec3::new(endless_translation.x, 0.0, endless_translation.z), Vec3::Y, endless.max_view_distance, Color::GREEN);

        for (mut vis, chunk_trans) in query.iter_mut() {
            let chuck_translation = chunk_trans.translation();

            let e_pos = Vec2::new(endless_translation.x, endless_translation.z);
            let t_pos = Vec2::new(chuck_translation.x, chuck_translation.z);

            // TODO: check terrain corners

            let dst = e_pos.distance(t_pos);
            *vis = match dst <= endless.max_view_distance {
                true => Visibility::Visible,
                false => Visibility::Hidden,
            }
        }
    }
}

struct ComputeResult {
    image: Image,
    mesh: Mesh,
    noise_map: NoiseMap,
    chunk_size: usize,
    world_scale: f32,
}

#[derive(Component)]
struct ComputeChunk(Task<ComputeResult>);

fn spawn_chunk_tasks(
    mut commands: Commands,
    mut query: Query<(Entity, &TerrainChunk, Option<&mut ComputeChunk>), Changed<TerrainChunk>>,
    mut generator: ResMut<TerrainGenerator>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    // create a arc of the generator to share with the thread pool
    let generator_arc = Arc::new(generator.clone());
    for (e, chunk, mut compute) in query.iter_mut() {
        if let Some(mut c) = compute {
            // drop the old task
            commands.entity(e).remove::<ComputeChunk>();
        }
        let chunk = chunk.clone();
        let generator = generator_arc.clone();

        let task = thread_pool.spawn(async move {
            // create noise map
            let noise_map = chunk.update_noise_map(&generator);

            // create image
            let image_data = match generator.texture_mode {
                TerrainTextureMode::Color => chunk.generate_color_map_image(&noise_map, &generator),
                TerrainTextureMode::HeightMap => {
                    chunk.generate_height_map_image(&noise_map, &generator)
                }
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
            let mesh = chunk.generate_mesh(&noise_map, &generator);

            ComputeResult {
                image,
                mesh,
                noise_map,
                chunk_size: generator.chunk_size,
                world_scale: generator.world_scale,
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

            // Update AABB
            // Hack: See https://github.com/bevyengine/bevy/issues/4294
            commands.entity(e).remove::<Aabb>();

            // Task is complete, so remove task component from entity
            commands.entity(e).remove::<ComputeChunk>();
        }
    }
}
