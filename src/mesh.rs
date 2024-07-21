#![allow(dead_code)]
use bevy::prelude::*;
use bracket_noise::prelude::*;
use rand::Rng;
use std::hash::{Hash, Hasher};

use std::{collections::HashMap, hash::DefaultHasher};

pub const CHUNK_SIZE: i32 = 32;

#[derive(Debug)]
pub struct Voxel {
    pub id: i32,
    pub is_solid: bool,
}

#[derive(Debug)]
pub struct Chunk {
    pub voxels: Vec<Voxel>,
}

#[derive(Debug, Resource)]
pub struct ChunkMap {
    pub map: HashMap<IVec3, Chunk>,
    pub seed: u64,
}

impl ChunkMap {
    pub fn new() -> Self {
        let seed = rand::thread_rng().gen();
        Self {
            map: HashMap::new(),
            seed,
        }
    }

    fn calculate_seed(&self, chunk_pos: &IVec3) -> u64 {
        // (chunk_pos.x as u64)
        //     .wrapping_add((chunk_pos.y as u64).wrapping_mul(31))
        //     .wrapping_add((chunk_pos.z as u64).wrapping_mul(17))
        let mut hasher = DefaultHasher::new();
        chunk_pos.hash(&mut hasher);
        hasher.finish()
    }

    pub fn generate_chunk(&mut self, chunk_pos: IVec3) -> Chunk {
        let mut voxels: Vec<Voxel> = Vec::new();
        let seed = self.calculate_seed(&chunk_pos);
        let mut noise: FastNoise = FastNoise::seeded(seed);
        noise.set_noise_type(NoiseType::Perlin);
        noise.set_frequency(0.05);
        // noise.set_fractal_type(FractalType::FBM);
        // noise.set_fractal_octaves(4);
        // noise.set_fractal_gain(0.5);
        // noise.set_fractal_lacunarity(2.0);

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let global_x = chunk_pos.x * CHUNK_SIZE + x;
                let global_z = chunk_pos.z * CHUNK_SIZE + z;
                let height =
                    (noise.get_noise(global_x as f32, global_z as f32) * 32.0 + 32.0) as i32;
                for y in 0..CHUNK_SIZE {
                    let global_y = chunk_pos.y * CHUNK_SIZE + y;
                    //println!("Global Y: {}, Height: {}", global_y, height);

                    let is_solid = global_y <= height;
                    //println!("Is solid: {}", is_solid);
                    let voxel = Voxel {
                        id: x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z,
                        is_solid,
                    };
                    voxels.push(voxel);
                }
            }
        }

        let chunk = Chunk { voxels };
        //self.insert_chunk(chunk_pos, chunk);
        return chunk;
    }

    pub fn insert_chunk(&mut self, chunk_pos: IVec3, chunk: Chunk) {
        self.map.insert(chunk_pos, chunk);
    }

    pub fn render_chunk(&mut self, chunk_pos: IVec3, chunk: Chunk) {
        self.generate_chunk(chunk_pos);
        self.map.insert(chunk_pos, chunk);
        // if let Some(chunk) = self.0.get(&chunk_pos) {
        //     for voxel in &chunk.voxels {
        //         println!("Voxel: {:?}", voxel);
        //     }
        // }
    }

    pub fn generate_terrain(&mut self, world_size: IVec3) {
        let mut noise = FastNoise::new();
        noise.set_noise_type(NoiseType::Perlin);
        noise.set_frequency(0.05);
        // noise.set_fractal_type(FractalType::FBM);
        // noise.set_fractal_octaves(4);
        // noise.set_fractal_gain(0.5);
        // noise.set_fractal_lacunarity(2.0);

        for x in 0..world_size.x {
            for z in 0..world_size.z {
                for y in 0..world_size.y {
                    let chunk_pos: IVec3 = IVec3::new(x, y, z);
                    let chunk = self.generate_chunk(chunk_pos);
                    self.insert_chunk(chunk_pos, chunk);
                }
            }
        }
    }

    pub fn create_chunk_heightmap(&mut self, chunk_pos: IVec3) -> Vec<i32> {
        let mut heightmap: Vec<i32> = Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE) as usize); // vector preallocation
                                                                                              //let seed = self.calculate_seed(&chunk_pos);
        let mut noise: FastNoise = FastNoise::seeded(self.seed);
        noise.set_noise_type(NoiseType::Simplex);
        noise.set_frequency(0.3);

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                // Get voxel X and Z position in global space
                let voxel_x = chunk_pos.x * CHUNK_SIZE + x;
                let voxel_z = chunk_pos.z * CHUNK_SIZE + z;
                let noise_value1 =
                    noise.get_noise(voxel_x as f32 / 16.0, voxel_z as f32 / 16.0) * 0.5;
                let noise_value2 =
                    noise.get_noise(voxel_x as f32 / 32.0, voxel_z as f32 / 32.0) * 0.25;
                let noise_value3 =
                    noise.get_noise(voxel_x as f32 / 64.0, voxel_z as f32 / 64.0) * 0.25;

                let noise_value = noise_value1 + noise_value2 + noise_value3;
                //println!("Noise Value: {}", noise_value);
                let normalized_noise_value = (noise_value + 1.0) / 2.0;
                let scaled_noise_value = normalized_noise_value * 32.0;
                let final_noise_value = scaled_noise_value as i32;
                // Apply to heightmap
                heightmap.push(final_noise_value);
            }
        }

        // return heightmap;
        // Gaussian blur kernel
        let kernel = [[1.0, 2.0, 1.0], [2.0, 4.0, 2.0], [1.0, 2.0, 1.0]];

        let kernel_sum: f32 = kernel.iter().flatten().sum();

        // // Smoothing the heightmap using Gaussian blur
        // let mut smoothed_heightmap = vec![0; heightmap.len()];
        // for z in 0..CHUNK_SIZE {
        //     for x in 0..CHUNK_SIZE {
        //         let mut total_height = 0.0;

        //         for dz in -1..=1 {
        //             for dx in -1..=1 {
        //                 let nx = x as i32 + dx;
        //                 let nz = z as i32 + dz;

        //                 if nx >= 0 && nx < CHUNK_SIZE as i32 && nz >= 0 && nz < CHUNK_SIZE as i32 {
        //                     let kernel_value = kernel[(dx + 1) as usize][(dz + 1) as usize];
        //                     total_height += heightmap[(nx + nz * CHUNK_SIZE as i32) as usize]
        //                         as f32
        //                         * kernel_value;
        //                 }
        //             }
        //         }

        //         smoothed_heightmap[(x + z * CHUNK_SIZE) as usize] =
        //             (total_height / kernel_sum) as i32;
        //     }
        // }

        // smoothed_heightmap
        return heightmap;
    }

    pub fn create_chunk_voxels(&mut self, chunk_pos: IVec3, heightmap: Vec<i32>) -> Vec<Voxel> {
        let mut voxels: Vec<Voxel> =
            Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize); // vector preallocation

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let heightmap_index = (x * CHUNK_SIZE + z) as usize;
                for y in 0..CHUNK_SIZE {
                    let voxel_id = x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z;
                    let voxel_y = chunk_pos.y * CHUNK_SIZE + y;
                    let heightmap_value = heightmap[heightmap_index];

                    let is_solid = voxel_y <= heightmap_value;
                    let voxel = Voxel {
                        id: voxel_id,
                        is_solid,
                    };
                    voxels.push(voxel);
                }
            }
        }

        return voxels;
    }

    pub fn generate_chunk_v2(&mut self, chunk_pos: IVec3) -> Chunk {
        let heightmap = self.create_chunk_heightmap(chunk_pos);
        // println!("Heightmap: {:?}", heightmap);
        let voxels = self.create_chunk_voxels(chunk_pos, heightmap);
        let chunk = Chunk { voxels };
        return chunk;
    }

    pub fn generate_terrain_v2(&mut self, world_size: IVec3) {
        println!("{}", self.seed);
        for z in 0..world_size.z {
            for x in 0..world_size.x {
                for y in 0..world_size.y {
                    let chunk_pos: IVec3 = IVec3::new(x, y, z);
                    let chunk = self.generate_chunk_v2(chunk_pos);
                    self.insert_chunk(chunk_pos, chunk);
                }
            }
        }
    }
}

pub fn generate_mesh(
    chunk_map: &ChunkMap,
    chunk_pos: IVec3,
) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 3]>, Vec<[f32; 4]>) {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut index_offset = 0;

    if let Some(chunk) = chunk_map.map.get(&chunk_pos) {
        for voxel in &chunk.voxels {
            if voxel.is_solid {
                let x = voxel.id % CHUNK_SIZE;
                let y = (voxel.id / CHUNK_SIZE) % CHUNK_SIZE;
                let z = voxel.id / (CHUNK_SIZE * CHUNK_SIZE);

                let voxel_pos = Vec3::new(x as f32, y as f32, z as f32);

                let cube_vertices = generate_cube_vertices(voxel_pos);
                vertices.extend(&cube_vertices);
                indices.extend(generate_cube_indices(index_offset));

                let cube_normals = generate_cube_normals();
                for normal in cube_normals.iter() {
                    normals.extend([*normal; 4]);
                }
                // normals.extend(generate_cube_normals());
                colors.extend([[0.0, 1.0, 0.0, 1.0]; 24]);

                index_offset += 24;
            }
        }
    }

    return (vertices, indices, normals, colors);
}

pub fn generate_cube_vertices(pos: Vec3) -> Vec<[f32; 3]> {
    let x = pos.x;
    let y = pos.y;
    let z = pos.z;

    vec![
        [x + 0.0, y + 1.0, z + 1.0], // 0
        [x + 0.0, y + 1.0, z + 0.0], // 1 // Top face
        [x + 1.0, y + 1.0, z + 0.0], // 2
        [x + 1.0, y + 1.0, z + 1.0], // 3
        [x + 1.0, y + 0.0, z + 1.0], // 4
        [x + 0.0, y + 0.0, z + 1.0], // 5 // Bottom face
        [x + 0.0, y + 0.0, z + 0.0], // 6
        [x + 1.0, y + 0.0, z + 0.0], // 7
        [x + 0.0, y + 0.0, z + 1.0], // 8
        [x + 0.0, y + 1.0, z + 1.0], // 9 // Left face
        [x + 0.0, y + 1.0, z + 0.0], // 10
        [x + 0.0, y + 0.0, z + 0.0], // 11
        [x + 1.0, y + 0.0, z + 1.0], // 12
        [x + 1.0, y + 1.0, z + 1.0], // 13 // Right face
        [x + 1.0, y + 1.0, z + 0.0], // 14
        [x + 1.0, y + 0.0, z + 0.0], // 15
        [x + 0.0, y + 0.0, z + 1.0], // 16
        [x + 0.0, y + 1.0, z + 1.0], // 17 // Front face
        [x + 1.0, y + 1.0, z + 1.0], // 18
        [x + 1.0, y + 0.0, z + 1.0], // 19
        [x + 0.0, y + 0.0, z + 0.0], // 20
        [x + 0.0, y + 1.0, z + 0.0], // 21 // Back face
        [x + 1.0, y + 1.0, z + 0.0], // 22
        [x + 1.0, y + 0.0, z + 0.0], // 23
    ]
}

pub fn generate_cube_indices(index: u32) -> Vec<u32> {
    vec![
        0, 1, 2, 2, 3, 0, // Top face
        4, 5, 6, 6, 7, 4, // Bottom face
        8, 9, 10, 10, 11, 8, // Left face
        12, 13, 14, 14, 15, 12, // Right face
        16, 17, 18, 18, 19, 16, // Front face
        20, 21, 22, 22, 23, 20, // Back face
    ]
    .into_iter()
    .map(|i| i + index)
    .collect()
}

pub fn generate_cube_normals() -> Vec<[f32; 3]> {
    vec![
        // Top face normals
        [0.0, 1.0, 0.0],
        // Bottom face normals
        [0.0, -1.0, 0.0],
        // Left face normals
        [-1.0, 0.0, 0.0],
        // Right face normals
        [1.0, 0.0, 0.0],
        // Front face normals
        [0.0, 0.0, 1.0],
        // Back face normals
        [0.0, 0.0, -1.0],
    ]
}
