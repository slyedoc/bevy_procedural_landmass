use crate::NoiseMap;
use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::std_options::NumberDisplay, prelude::*};

#[derive(Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub enum TerrainErosion {
    None,
    Hydraulic(HydraulicErosion),
}

impl Default for TerrainErosion {
    fn default() -> Self {
        TerrainErosion::Hydraulic(HydraulicErosion::default())
    }
}

// Based on https://github.com/SebLague/Hydraulic-Erosion/blob/master/Assets/Scripts/Erosion.cs
#[derive(Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct HydraulicErosion {
    #[inspector(min = 0, max = 100_000, display = NumberDisplay::Slider)]
    pub iterations: usize,
    #[inspector(min = 2, max = 8, display = NumberDisplay::Slider)]
    pub erosion_radius: usize,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub inertia: f32,
    sediment_capacity_factor: f32,
    min_sediment_capacity: f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub erode_speed: f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub deposit_speed: f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub evaporate_speed: f32,
    gravity: f32,
    max_droplet_lifetime: u32,
    initial_water_volume: f32,
    initial_speed: f32,
    pub seed: u64,

}

impl Default for HydraulicErosion {
    fn default() -> Self {
        Self {
            iterations: 100,
            erosion_radius: 3,
            inertia: 0.05,
            sediment_capacity_factor: 4.0,
            min_sediment_capacity: 0.01,
            erode_speed: 0.3,
            deposit_speed: 0.3,
            evaporate_speed: 0.01,
            gravity: 4.0,
            max_droplet_lifetime: 10,
            initial_water_volume: 1.0,
            initial_speed: 1.0,
            seed: 0,
        }
    }
}

impl HydraulicErosion {
    pub fn erode(&self, 
        map: &mut NoiseMap, 
        map_size: usize, 
        #[cfg(debug_rain)]
        wolrd_scale: f32,
        #[cfg(debug_rain)]
         height_multiplyer: f32
        
    ) -> Vec<Vec<Vec3>> {
        
        let erosion_brushes = initialize_brushes(map_size, self.erosion_radius);
        
        // paths rain drops take, for debuging only
        #[cfg(debug_rain)] {
        let mut rain_paths : Vec<Vec<Vec3>> = vec![vec![]; self.iterations];        
        let xy_rain_scale = wolrd_scale  as f32 / map_size as f32;
        let half_size = map_size as f32 / 2.0;
    }

        fastrand::seed(self.seed);

        
        for _ in 0..self.iterations {
            
            // Create water droplet at random point on map
            let mut pos_x = fastrand::f32() * (map_size - 1) as f32;
            let mut pos_y = fastrand::f32() * (map_size - 1) as f32;

            let mut dir_x = 0.0;
            let mut dir_y = 0.0;
            let mut speed = self.initial_speed;
            let mut water = self.initial_water_volume;
            let mut sediment = 0.0;

            #[cfg(debug_rain)]           
            rain_paths[i].push(Vec3 {
                x: (pos_x - half_size) * xy_rain_scale,                
                y: (map[pos_x as usize][pos_y as usize] * wolrd_scale * height_multiplyer) + 10.0,
                z: (pos_y - half_size) * xy_rain_scale,
            });

            for _ in 0..self.max_droplet_lifetime {                
                let node_x = pos_x as usize;
                let node_y = pos_y as usize;

                // Calculate droplet's offset inside the cell
                let cell_offset_x = pos_x - node_x as f32;
                let cell_offset_y = pos_y - node_y as f32;

                // Calculate droplet's height and direction of flow with bilinear interpolation of surrounding heights
                let height_and_gradient = calculate_height_and_gradient(map, pos_x, pos_y);

                // Update the droplet's direction and position
                dir_x = dir_x * self.inertia - height_and_gradient.gradient_x * (1.0 - self.inertia);
                dir_y = dir_y * self.inertia - height_and_gradient.gradient_y * (1.0 - self.inertia);
                // Normalize direction
                let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
                if len != 0.0 {
                    dir_x /= len;
                    dir_y /= len;
                }
                pos_x += dir_x;
                pos_y += dir_y;
                
                #[cfg(debug_rain)]
                rain_paths[i].push(Vec3 {
                    x: (pos_x - half_size) * xy_rain_scale,
                    y: map[node_x][node_y] * wolrd_scale * height_multiplyer,
                    z: (pos_y - half_size) * xy_rain_scale,
                });


                // Stop simulating droplet if it's not moving or has flowed over edge of map
                if (dir_x == 0.0 && dir_y == 0.0)
                    || pos_x < 0.0
                    || pos_x >= map_size as f32 - 1.0
                    || pos_y < 0.0
                    || pos_y >= map_size as f32 - 1.0
                {
                    break;
                }

                // Find the droplet's new height and calculate the delta_height
                let new_height = calculate_height_and_gradient(map, pos_x, pos_y).height;
                let delta_height = new_height - height_and_gradient.height;

                // Calculate the droplet's sediment capacity
                let sediment_capacity =
                    (-delta_height * speed * water * self.sediment_capacity_factor)
                        .max(self.min_sediment_capacity);

                // If carrying more sediment than capacity, or if flowing uphill:
                if sediment > sediment_capacity || delta_height > 0.0 {
                    
                    // If moving uphill try to fill up to the current height, otherwise deposit a fraction of the excess sediment
                    let amount_to_deposit = if delta_height > 0.0 {                        
                        delta_height.min(sediment)
                    } else {                        
                        (sediment - sediment_capacity) * self.deposit_speed
                    };
                    sediment -= amount_to_deposit;

                    // Add the sediment to the four nodes of the current cell
                     let ammount = (
                        amount_to_deposit * (1.0 - cell_offset_x) * (1.0 - cell_offset_y),
                        amount_to_deposit * (1.0 - cell_offset_x) * cell_offset_y,
                        amount_to_deposit * cell_offset_x * (1.0 - cell_offset_y),
                        amount_to_deposit * cell_offset_x * cell_offset_y
                     );

                    map[node_x][node_y] += ammount.0;
                    map[node_x][node_y + 1] += ammount.1;
                    map[node_x + 1][node_y] += ammount.2;
                    map[node_x + 1][node_y + 1] += ammount.3;
                } else {
                    // Erode a fraction of the droplet's current carry capacity
                    let amount_to_erode = ((sediment_capacity - sediment) * self.erode_speed).min(-delta_height);                    
                    // Use erosion brush to erode from all nodes inside the droplet's erosion radius
                    for brush in erosion_brushes[node_x][node_y].iter() {

                        let weighed_erode_amount = amount_to_erode * brush.weight;                        
                        let delta_sediment = map[brush.x][brush.y].min(weighed_erode_amount);

                        map[brush.x][brush.y] -= delta_sediment;
                        sediment += delta_sediment;
                    }
                }

                // Update droplet's speed and water content
                speed = (speed * speed + delta_height * self.gravity).sqrt();
                water *= 1.0 - self.evaporate_speed;
            }
        }
        
            
        

        #[cfg(debug_rain)]
        rain_paths;

        vec![]


    }
}


#[derive(Clone, Debug)]
struct Brush {
    x: usize,
    y: usize,
    weight: f32,
}
fn initialize_brushes(map_size: usize, radius: usize) -> Vec<Vec<Vec<Brush>>> {
    let mut erosion_brushs: Vec<Vec<Vec<Brush>>> = vec![vec![vec![]; map_size]; map_size];    

    let mut x_offsets = vec![0; radius * radius * 4];
    let mut y_offsets = vec![0; radius * radius * 4];
    let mut weights = vec![0.0; radius * radius * 4];

    for y in 0..map_size {
        for x in 0..map_size {
            if y <= radius
                || y >= map_size - radius
                || x <= radius + 1
                || x >= map_size - radius
            {
                let mut weight_sum = 0.0;
                let mut add_index = 0;

                for by in -(radius as isize)..=radius as isize {
                    for bx in -(radius as isize)..=radius as isize {
                        let sqr_dst = (bx * bx + by * by) as f32;
                        if sqr_dst < (radius * radius) as f32 {
                            let coord_x = (bx as isize + bx) as usize;
                            let coord_y = (by as isize + by) as usize;

                            if coord_x < map_size && coord_y < map_size {
                                let weight = 1.0 - (sqr_dst.sqrt()) / radius as f32;
                                weight_sum += weight;
                                weights[add_index] = weight;
                                x_offsets[add_index] = bx;
                                y_offsets[add_index] = by;
                                add_index += 1;
                            }
                        }
                    }
                }

                for j in 0..add_index {
                    erosion_brushs[x][y].push(Brush { 
                        x: (x_offsets[j] + x as isize) as usize,
                        y: (y_offsets[j] + y as isize) as usize,
                        weight: weights[j] / weight_sum });                    
                }
            }
        }
    }
    erosion_brushs
}



struct HeightAndGradient {
    height: f32,
    gradient_x: f32,
    gradient_y: f32,
}

fn calculate_height_and_gradient(
    map: &Vec<Vec<f32>>,
    pos_x: f32,
    pos_y: f32,
) -> HeightAndGradient {
    
    let coord_x = pos_x as usize;
    let coord_y = pos_y as usize;

    // Calculate droplet's offset inside the cell
    let x = pos_x - coord_x as f32;
    let y = pos_y - coord_y as f32;

    // Ensure you don't exceed bounds
    if coord_x + 1 >= map.len() || coord_y + 1 >= map[coord_x].len() {
        panic!("Coordinates exceed map dimensions");
    } 
    // Calculate heights of the four nodes of the droplet's cell
    let height_nw = map[coord_x][coord_y];
    let height_ne = map[coord_x][coord_y + 1];
    let height_sw = map[coord_x + 1][coord_y];
    let height_se = map[coord_x + 1][coord_y + 1];

    // Calculate droplet's direction of flow with bilinear interpolation of height difference along the edges
    let gradient_y = (height_ne - height_nw) * (1.0 - y) + (height_se - height_sw) * y;
    let gradient_x = (height_sw - height_nw) * (1.0 - x) + (height_se - height_ne) * x;

    // Calculate height with bilinear interpolation of the heights of the nodes of the cell
    let height = height_nw * (1.0 - x) * (1.0 - y)
        + height_ne * x * (1.0 - y)
        + height_sw * (1.0 - x) * y
        + height_se * x * y;

    HeightAndGradient {
        height,
        gradient_x,
        gradient_y,
    }
}
