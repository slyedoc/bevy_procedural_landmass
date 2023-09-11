use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_inspector_egui::prelude::*;
use noisy_bevy::*;

use crate::{
    terrain_generator::{TerrainGenerator, TerrainMeshMode},
    NoiseMap,
};

/// A component bundle for entities with a [`Mesh`] and a [`Material`].
#[derive(Bundle, Clone)]
pub struct TerrainChunkBundle {
    pub terrain: TerrainChunk,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
    pub name: Name,
}

impl Default for TerrainChunkBundle {
    fn default() -> Self {
        Self {
            terrain: Default::default(),
            mesh: Default::default(),
            material: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
            name: Default::default(),
        }
    }
}

impl TerrainChunkBundle {
    pub fn new(position: IVec2, visable: bool) -> Self {
        Self {
            terrain: TerrainChunk::new(position),
            visibility: match visable {
                true => Visibility::Visible,
                false => Visibility::Hidden,
            },
            name: Name::new(format!("TerrainChunk: {:?}", position)),
            ..Default::default()
        }
    }
}

#[derive(Clone, Component, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct TerrainChunk {
    pub position: IVec2,
}

impl Default for TerrainChunk {
    fn default() -> Self {
        Self {
            position: IVec2::ZERO,
        }
    }
}

impl TerrainChunk {
    pub fn new(position: IVec2) -> Self {
        Self { position }
    }

    pub fn update_noise_map(&self, generator: &TerrainGenerator) -> NoiseMap {
        let mut noise_map = vec![vec![0f32; generator.chunk_size]; generator.chunk_size];

        if generator.noise_scale < 0.0 {
            panic!("Scale must be greater than 0");
        }

        let mut max_noise_height = f32::MIN;
        let mut min_noise_height = f32::MAX;

        let half_size = generator.chunk_size as f32 / 2.0;

        for y in 0..generator.chunk_size {
            for x in 0..generator.chunk_size {
                let pos = Vec2::new(
                    (x as f32 - half_size)
                        + (self.position.x as f32 * generator.chunk_size as f32)
                        + generator.offset.x,
                    (y as f32 - half_size)
                        + (self.position.y as f32 * generator.chunk_size as f32)
                        + generator.offset.y,
                ) / (generator.noise_scale * generator.chunk_size as f32);

                let noise_height =
                    fbm_simplex_2d(pos, generator.octaves, generator.lacunarity, generator.gain);

                if noise_height > max_noise_height {
                    max_noise_height = noise_height;
                } else if noise_height < min_noise_height {
                    min_noise_height = noise_height;
                }
                noise_map[x][y] = noise_height;
            }
        }

        for y in 0..generator.chunk_size {
            for x in 0..generator.chunk_size {
                // inverse lerp?
                noise_map[x][y] = ((noise_map[x][y] - min_noise_height)
                    / (max_noise_height - min_noise_height))
                    .abs();
            }
        }
        noise_map
    }

    pub fn fbm_simplex_2d_seeded(
        pos: Vec2,
        octaves: usize,
        lacunarity: f32,
        gain: f32,
        seed: f32,
    ) -> f32 {
        let mut sum = 0.;
        let mut amplitude = 1.;
        let mut frequency = 1.;

        for _ in 0..octaves {
            sum += simplex_noise_2d_seeded(pos * frequency, seed) * amplitude;
            amplitude *= gain;
            frequency *= lacunarity;
        }

        sum
    }

    pub fn generate_color_map_image(
        &self,
        noise_map: &NoiseMap,
        generator: &TerrainGenerator,
    ) -> Vec<u8> {
        let mut image_data = vec![0u8; (generator.chunk_size * generator.chunk_size) as usize * 4];
        for y in 0..generator.chunk_size {
            for x in 0..generator.chunk_size {
                let height = noise_map[x as usize][y as usize];
                let color = generator.regions.get_color(height);
                let j = ((y * generator.chunk_size) + x) as usize * 4;
                image_data[j] = (color.r() * 255.0) as u8;
                image_data[j + 1] = (color.g() * 255.0) as u8;
                image_data[j + 2] = (color.b() * 255.0) as u8;
                image_data[j + 3] = 255;
            }
        }
        image_data
    }

    pub fn generate_height_map_image(
        &self,
        noise_map: &NoiseMap,
        generator: &TerrainGenerator,
    ) -> Vec<u8> {
        let mut image_data = vec![0u8; (generator.chunk_size * generator.chunk_size) as usize * 4];
        for y in 0..generator.chunk_size {
            for x in 0..generator.chunk_size {
                let height = noise_map[x as usize][y as usize];
                let j = ((y * generator.chunk_size) + x) as usize * 4;
                let val = (height * 255.0) as u8;
                image_data[j] = val;
                image_data[j + 1] = val;
                image_data[j + 2] = val;
                image_data[j + 3] = 255;
            }
        }
        image_data
    }

    pub fn generate_mesh(&self, noise_map: &NoiseMap, generator: &TerrainGenerator) -> Mesh {
        let size = generator.chunk_size;

        let num_vertices = size * size;
        let num_indices = (size - 1) * (size - 1) * 6;

        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);

        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

        let half_size = size as f32 / 2.0;
        for y in 0..size {
            for x in 0..size {
                let i = (y * size) + x;
                // find the position of the vertex and center, with height_multiplier
                let pos = [
                    (x as f32 - half_size) * generator.world_scale / size as f32,
                    (noise_map[x][y] * generator.height_multiplier * generator.world_scale) as f32,
                    (y as f32 - half_size) * generator.world_scale / size as f32,
                ];

                positions.push(pos);
                uvs.push([x as f32 / size as f32, y as f32 / size as f32]);

                if x < size - 1 && y < size - 1 {
                    let a = i;
                    let b = i + size;
                    let c = i + size + 1;
                    let d = i + 1;

                    indices.push(a as u32);
                    indices.push(b as u32);
                    indices.push(c as u32);

                    indices.push(c as u32);
                    indices.push(d as u32);
                    indices.push(a as u32);
                }
            }
        }

        // build our mesh
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        // compute normals
        match generator.mesh_mode {
            TerrainMeshMode::Flat => {
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                mesh.duplicate_vertices();
                mesh.compute_flat_normals();
            }
            TerrainMeshMode::Smooth => {
                let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
                for y in 0..size {
                    for x in 0..size {
                        let pos: Vec3 = positions[(y * size + x) as usize].into();
                        if x < size - 1 && y < size - 1 {
                            let pos_right: Vec3 = positions[(y * size + x + 1) as usize].into();
                            let pos_up: Vec3 = positions[((y + 1) * size + x) as usize].into();
                            let tangent1 = pos_right - pos;
                            let tangent2 = pos_up - pos;
                            let normal = tangent2.cross(tangent1);
                            normals.push(normal.normalize().into());
                        } else {
                            normals.push(Vec3::Y.into());
                        }
                    }
                }
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            }
        }

        mesh
    }
}
