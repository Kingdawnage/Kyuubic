#![allow(dead_code)]
use bevy::prelude::*;
use bracket_noise::prelude::*;
use rand::Rng;
// use rayon::vec;

use std::{collections::HashMap, fs::File, io::Write};

pub const CHUNK_SIZE: i32 = 32;
pub const CHUNK_HEIGHT: i32 = 64;
pub const SEA_LEVEL: i32 = 30;

#[derive(Debug)]
pub struct WorldMap {
    pub map: HashMap<(i32, i32, i32), Voxel>,
}

impl WorldMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn collect_voxels(&mut self, chunk_map: &ChunkMap) {
        for (chunk_pos, chunk) in &chunk_map.map {
            for voxel in &chunk.voxels {
                let x = voxel.id % CHUNK_SIZE;
                let y = (voxel.id / CHUNK_SIZE) % CHUNK_HEIGHT;
                let z = voxel.id / (CHUNK_SIZE * CHUNK_HEIGHT);
                let world_pos = (
                    chunk_pos.x * CHUNK_SIZE + x,
                    chunk_pos.y * CHUNK_HEIGHT + y,
                    chunk_pos.z * CHUNK_SIZE + z,
                );
                self.map.insert(world_pos, *voxel);
            }
        }
    }

    fn get_voxel(&self, x: i32, y: i32, z: i32) -> Option<&Voxel> {
        self.map.get(&(x, y, z))
    }
}

trait AsVec3 {
    fn as_vec3(&self) -> Vec3;
}

impl AsVec3 for IVec3 {
    fn as_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BlockType {
    Air,
    Stone,
    Dirt,
    Grass,
    Snow,
    Water,
}

impl BlockType {
    pub fn color(&self) -> [f32; 4] {
        match self {
            BlockType::Air => [0.0, 0.0, 0.0, 0.0],
            BlockType::Stone => [0.5, 0.5, 0.5, 1.0],
            BlockType::Dirt => [0.5, 0.25, 0.0, 1.0],
            BlockType::Grass => [0.0, 0.5, 0.0, 1.0],
            BlockType::Snow => [1.0, 1.0, 1.0, 1.0],
            BlockType::Water => [0.0, 0.0, 1.0, 0.5],
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Voxel {
    pub id: i32,
    pub is_solid: bool,
    pub block_type: BlockType,
}

#[derive(Debug)]
pub struct Chunk {
    pub voxels: Vec<Voxel>,
}

impl Chunk {
    pub fn get_size(&self) -> i32 {
        self.voxels.len() as i32
    }

    fn rendered_voxels(&self) -> Vec<&Voxel> {
        self.voxels.iter().filter(|v| v.is_solid).collect()
    }

    pub fn rendered_voxels_count(&self) -> i32 {
        self.rendered_voxels().len() as i32
    }

    pub fn get_voxel(&self, x: i32, y: i32, z: i32) -> Option<&Voxel> {
        if x < 0 || x >= CHUNK_SIZE || y < 0 || y >= CHUNK_HEIGHT || z < 0 || z >= CHUNK_SIZE {
            return None;
        }

        let index = (x * CHUNK_HEIGHT * CHUNK_SIZE + y * CHUNK_SIZE + z) as usize;
        return self.voxels.get(index);
    }
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

    pub fn insert_chunk(&mut self, chunk_pos: IVec3, chunk: Chunk) {
        self.map.insert(chunk_pos, chunk);
    }

    pub fn create_chunk_heightmap(&mut self, chunk_pos: IVec3) -> Vec<i32> {
        let mut heightmap: Vec<i32> = Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE) as usize); // vector preallocation
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
                let scaled_noise_value = normalized_noise_value * 64.0;
                let final_noise_value = scaled_noise_value as i32;
                // Apply to heightmap
                heightmap.push(final_noise_value);
            }
        }

        return heightmap;
    }

    pub fn create_chunk_voxels(&mut self, chunk_pos: IVec3, heightmap: Vec<i32>) -> Vec<Voxel> {
        let mut voxels: Vec<Voxel> =
            Vec::with_capacity((CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE) as usize); // vector preallocation

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let heightmap_index = (x * CHUNK_SIZE + z) as usize;
                for y in 0..CHUNK_HEIGHT {
                    let voxel_id = x * CHUNK_HEIGHT * CHUNK_SIZE + y * CHUNK_SIZE + z;
                    let voxel_y = chunk_pos.y * CHUNK_HEIGHT + y;
                    let heightmap_value = heightmap[heightmap_index];

                    // let is_solid = voxel_y <= heightmap_value;

                    let block_type = if voxel_y >= 40 && voxel_y <= heightmap_value {
                        BlockType::Snow
                    } else if voxel_y == heightmap_value && voxel_y <= heightmap_value {
                        BlockType::Grass
                    } else if voxel_y > heightmap_value - 10 && voxel_y <= heightmap_value {
                        BlockType::Dirt
                    } else if voxel_y > 0 && voxel_y <= heightmap_value {
                        BlockType::Stone
                    } else if voxel_y <= SEA_LEVEL && voxel_y > heightmap_value {
                        BlockType::Water
                    } else {
                        BlockType::Air
                    };

                    let is_solid = match block_type {
                        BlockType::Air => false,
                        _ => true,
                    };

                    let voxel = Voxel {
                        id: voxel_id,
                        is_solid,
                        block_type,
                    };
                    voxels.push(voxel);
                }
            }
        }

        return voxels;
    }

    pub fn generate_chunk(&mut self, chunk_pos: IVec3) -> Chunk {
        let heightmap = self.create_chunk_heightmap(chunk_pos);
        // println!("Heightmap: {:?}", heightmap);
        let voxels = self.create_chunk_voxels(chunk_pos, heightmap);
        let chunk = Chunk { voxels };
        return chunk;
    }

    pub fn generate_terrain(&mut self, world_size: IVec3) {
        let mut solid_voxels: i32 = 0;
        // println!("{}", self.seed);
        for z in 0..world_size.z {
            for x in 0..world_size.x {
                for y in 0..world_size.y {
                    let chunk_pos: IVec3 = IVec3::new(x, y, z);
                    let chunk = self.generate_chunk(chunk_pos);
                    solid_voxels += chunk.rendered_voxels_count();
                    self.insert_chunk(chunk_pos, chunk);
                }
            }
        }

        println!("Solid Voxels: {}", solid_voxels);
        // collect_terrain_data(self);
    }
}

pub fn collect_terrain_data(chunk_map: &ChunkMap) {
    let mut world_map = WorldMap::new();
    world_map.collect_voxels(chunk_map);
    let terrain_map: HashMap<(i32, i32, i32), Voxel> = world_map.map;
    // Write terrain data to a file
    let mut file: File = File::create("terrain_map.txt").expect("Unable to create file");
    for ((x, y, z), voxel) in &terrain_map {
        writeln!(file, "{},{},{},{}", x, y, z, voxel.is_solid).expect("Unable to write data");
    }
}
