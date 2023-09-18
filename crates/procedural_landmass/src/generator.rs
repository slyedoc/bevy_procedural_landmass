use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}};
use bevy_inspector_egui::{prelude::*, inspector_options::std_options::NumberDisplay};

use crate::{regions::TerrainRegions, NoiseMap, noise::TerrainNoise, curve::TerrainCurve};

#[derive(Clone, Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct TerrainGenerator {
    // Terrain generation settings
    pub texture_mode: TerrainTextureMode,
    pub mesh_mode: TerrainMeshMode,
    pub sampler: TerrainSampler,
    #[inspector(min = 1)]
    pub chunk_size: usize,

    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub height_multiplier: f32,
    #[inspector(min = 0.01, max = 5.0, display = NumberDisplay::Slider)]
    pub noise_scale: f32,
    pub noise: TerrainNoise,
    pub curve: TerrainCurve,
    pub offset: Vec2,
    pub seed: f32,

    pub world_scale: f32,

    pub regions: TerrainRegions,
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self {
            chunk_size: 50,
            texture_mode: TerrainTextureMode::Color,
            mesh_mode: TerrainMeshMode::Smooth,
            sampler: TerrainSampler::Nearest,
            height_multiplier: 0.3,
            noise_scale: 1.0,
            noise: TerrainNoise::default(),
            curve: TerrainCurve::default(),
            seed: 0.0,
            offset: Vec2::ZERO,
            world_scale: 100.0,
            regions: TerrainRegions::default(),
        }
    }
}

impl TerrainGenerator {
    pub fn update_noise_map(&self, position: IVec2) -> NoiseMap {
        let size = self.chunk_size + 1;
        let mut noise_map = vec![vec![0f32; size]; size];

        if self.noise_scale < 0.0 {
            panic!("Terrain Noise Scale must be greater than 0");
        }
        
        let half_size = size as f32 / 2.0;

        for y in 0..size {
            for x in 0..size {
                let pos = Vec2::new(
                    (x as f32 - half_size)
                        + (position.x as f32 * size as f32)
                        + self.offset.x,
                    (y as f32 - half_size)
                        + (position.y as f32 * size as f32)
                        + self.offset.y,
                ) / (self.noise_scale * size as f32);

                let noise_height = self.noise.get(pos, self.seed);
                let height = self.curve.get(noise_height);

                noise_map[x][y] = height;
            }
        }
        noise_map
    }

    pub fn generate_color_map_image(
        &self,
        noise_map: &NoiseMap,        
    ) -> Vec<u8> {
        let mut image_data = vec![0u8; (self.chunk_size * self.chunk_size) as usize * 4];
        for y in 0..self.chunk_size {
            for x in 0..self.chunk_size {
                let height = noise_map[x as usize][y as usize];
                let color = self.regions.get_color(height);
                let j = ((y * self.chunk_size) + x) as usize * 4;
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
    ) -> Vec<u8> {
        let mut image_data = vec![0u8; (self.chunk_size * self.chunk_size) as usize * 4];
        for y in 0..self.chunk_size {
            for x in 0..self.chunk_size {
                let height = noise_map[x as usize][y as usize];
                let j = ((y * self.chunk_size) + x) as usize * 4;
                let val = (height * 255.0) as u8;
                image_data[j] = val;
                image_data[j + 1] = val;
                image_data[j + 2] = val;
                image_data[j + 3] = 255;
            }
        }
        image_data
    }

    pub fn generate_mesh(&self, noise_map: &NoiseMap) -> Mesh {
        let size = self.chunk_size;

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
                    (x as f32 - half_size) * self.world_scale / size as f32,
                    (noise_map[x][y] * self.height_multiplier * self.world_scale) as f32,
                    (y as f32 - half_size) * self.world_scale / size as f32,
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
        match self.mesh_mode {
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

#[derive(Clone, PartialEq, Eq, Debug, Reflect)]
pub enum TerrainTextureMode {
    HeightMap,
    Color,
}

#[derive(Clone, PartialEq, Eq, Debug, Reflect)]
pub enum TerrainMeshMode {
    Flat,
    Smooth,
}

#[derive(Clone, PartialEq, Eq, Debug, Reflect)]
pub enum TerrainSampler {
    Linear,
    Nearest,
}
