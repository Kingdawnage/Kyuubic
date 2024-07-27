#![allow(dead_code)]
use bevy::prelude::*;
use bracket_noise::prelude::*;
use rand::Rng;

use std::collections::HashMap;

pub const CHUNK_SIZE: i32 = 32;
pub const CHUNK_HEIGHT: i32 = 64;
pub const SEA_LEVEL: i32 = 20;

trait AsVec3 {
    fn as_vec3(&self) -> Vec3;
}

impl AsVec3 for IVec3 {
    fn as_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

#[derive(Debug, PartialEq)]
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
            BlockType::Water => [0.0, 0.0, 1.0, 0.1],
        }
    }
}

#[derive(Debug)]
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
            //println!("Out of bounds: x={}, y={}, z={}", x, y, z);
            return None;
        }

        let index = (x * CHUNK_HEIGHT * CHUNK_SIZE + y * CHUNK_SIZE + z) as usize;
        //println!("Coordinates: x={}, y={}, z={} -> Index: {}", x, y, z, index);
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
        let all_voxels = &chunk.voxels;

        // let neighbour_offsets = [
        //     Vec3::new(0.0, 1.0, 0.0),  // Top voxel
        //     Vec3::new(0.0, -1.0, 0.0), // Bottom voxel
        //     Vec3::new(-1.0, 0.0, 0.0), // Left voxel
        //     Vec3::new(1.0, 0.0, 0.0),  // Right voxel
        //     Vec3::new(0.0, 0.0, 1.0),  // Front voxel
        //     Vec3::new(0.0, 0.0, -1.0), // Back voxel
        // ];

        let mut voxel_map: HashMap<(i32, i32, i32), &Voxel> = HashMap::new();

        for voxel in all_voxels {
            let x = voxel.id % CHUNK_SIZE as i32;
            let y = (voxel.id / CHUNK_SIZE as i32) % CHUNK_HEIGHT as i32;
            let z = voxel.id / (CHUNK_SIZE as i32 * CHUNK_HEIGHT as i32);
            voxel_map.insert((x, y, z), voxel);
        }

        for voxel in all_voxels {
            // if voxel.is_solid {
            let x = voxel.id % CHUNK_SIZE;
            let y = (voxel.id / CHUNK_SIZE) % CHUNK_HEIGHT;
            let z = voxel.id / (CHUNK_SIZE * CHUNK_HEIGHT);

            let voxel_pos = Vec3::new(x as f32, y as f32, z as f32);

            // let top_voxel_pos = voxel_pos + neighbour_offsets[0];
            // let bottom_voxel_pos = voxel_pos + neighbour_offsets[1];
            // let left_voxel_pos = voxel_pos + neighbour_offsets[2];
            // let right_voxel_pos = voxel_pos + neighbour_offsets[3];
            // let front_voxel_pos = voxel_pos + neighbour_offsets[4];
            // let back_voxel_pos = voxel_pos + neighbour_offsets[5];

            // let top_neighbour = all_voxels.iter().find(|v| {
            //     let nx = v.id % CHUNK_SIZE;
            //     let ny = (v.id / CHUNK_SIZE) % CHUNK_HEIGHT;
            //     let nz = v.id / (CHUNK_SIZE * CHUNK_HEIGHT);
            //     Vec3::new(nx as f32, ny as f32, nz as f32) == top_voxel_pos
            // });

            if voxel.is_solid {
                add_top_face(
                    &mut vertices,
                    &mut indices,
                    &mut normals,
                    &mut colors,
                    voxel_pos,
                    &voxel.block_type,
                    index_offset,
                );

                add_bottom_face(
                    &mut vertices,
                    &mut indices,
                    &mut normals,
                    &mut colors,
                    voxel_pos,
                    &voxel.block_type,
                    index_offset,
                );

                add_left_face(
                    &mut vertices,
                    &mut indices,
                    &mut normals,
                    &mut colors,
                    voxel_pos,
                    &voxel.block_type,
                    index_offset,
                );

                add_right_face(
                    &mut vertices,
                    &mut indices,
                    &mut normals,
                    &mut colors,
                    voxel_pos,
                    &voxel.block_type,
                    index_offset,
                );

                add_front_face(
                    &mut vertices,
                    &mut indices,
                    &mut normals,
                    &mut colors,
                    voxel_pos,
                    &voxel.block_type,
                    index_offset,
                );

                add_back_face(
                    &mut vertices,
                    &mut indices,
                    &mut normals,
                    &mut colors,
                    voxel_pos,
                    &voxel.block_type,
                    index_offset,
                );
                index_offset += 24;
            }
            // if voxel.is_solid {
            //     add_voxel_cube(
            //         &mut vertices,
            //         &mut indices,
            //         &mut normals,
            //         &mut colors,
            //         voxel_pos,
            //         &voxel.block_type,
            //         index_offset,
            //     );
            //     index_offset += 24;
            // }
        }
    }
    // }

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

fn add_top_face(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    voxel_pos: Vec3,
    block_type: &BlockType,
    index_offset: u32,
) {
    let cube_vertices = generate_cube_vertices(voxel_pos);
    let cube_indices = generate_cube_indices(index_offset);
    let cube_normals = generate_cube_normals();

    let face_vertices = &cube_vertices[0 * 4..(0 + 1) * 4];
    vertices.extend(face_vertices);

    let face_indices = &cube_indices[0 * 6..(0 + 1) * 6];
    indices.extend(face_indices);

    let face_normals = &cube_normals[0];
    normals.extend([*face_normals; 4]);

    let _test_color: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = match block_type {
        BlockType::Air => [BlockType::Air.color(); 4],
        BlockType::Stone => [BlockType::Stone.color(); 4],
        BlockType::Dirt => [BlockType::Dirt.color(); 4],
        BlockType::Grass => [BlockType::Grass.color(); 4],
        BlockType::Snow => [BlockType::Snow.color(); 4],
        BlockType::Water => [BlockType::Water.color(); 4],
    };
    colors.extend(face_colors);
}

fn add_bottom_face(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    voxel_pos: Vec3,
    block_type: &BlockType,
    index_offset: u32,
) {
    let cube_vertices = generate_cube_vertices(voxel_pos);
    let cube_indices = generate_cube_indices(index_offset);
    let cube_normals = generate_cube_normals();

    let face_vertices = &cube_vertices[1 * 4..(1 + 1) * 4];
    vertices.extend(face_vertices);

    let face_indices = &cube_indices[1 * 6..(1 + 1) * 6];
    indices.extend(face_indices);

    let face_normals = &cube_normals[1];
    normals.extend([*face_normals; 4]);

    let _test_color: [[f32; 4]; 4] = [[1.0, 1.0, 0.0, 1.0]; 4];

    let face_colors = match block_type {
        BlockType::Air => [BlockType::Air.color(); 4],
        BlockType::Stone => [BlockType::Stone.color(); 4],
        BlockType::Dirt => [BlockType::Dirt.color(); 4],
        BlockType::Grass => [BlockType::Grass.color(); 4],
        BlockType::Snow => [BlockType::Snow.color(); 4],
        BlockType::Water => [BlockType::Water.color(); 4],
    };
    colors.extend(face_colors);
}

fn add_left_face(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    voxel_pos: Vec3,
    block_type: &BlockType,
    index_offset: u32,
) {
    let cube_vertices = generate_cube_vertices(voxel_pos);
    let cube_indices = generate_cube_indices(index_offset);
    let cube_normals = generate_cube_normals();

    let face_vertices = &cube_vertices[2 * 4..(2 + 1) * 4];
    vertices.extend(face_vertices);

    let face_indices = &cube_indices[2 * 6..(2 + 1) * 6];
    indices.extend(face_indices);

    let face_normals = &cube_normals[2];
    normals.extend([*face_normals; 4]);

    let _test_color: [[f32; 4]; 4] = [[0.0, 1.0, 0.0, 1.0]; 4];

    let face_colors = match block_type {
        BlockType::Air => [BlockType::Air.color(); 4],
        BlockType::Stone => [BlockType::Stone.color(); 4],
        BlockType::Dirt => [BlockType::Dirt.color(); 4],
        BlockType::Grass => [BlockType::Grass.color(); 4],
        BlockType::Snow => [BlockType::Snow.color(); 4],
        BlockType::Water => [BlockType::Water.color(); 4],
    };
    colors.extend(face_colors);
}

fn add_right_face(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    voxel_pos: Vec3,
    block_type: &BlockType,
    index_offset: u32,
) {
    let cube_vertices = generate_cube_vertices(voxel_pos);
    let cube_indices = generate_cube_indices(index_offset);
    let cube_normals = generate_cube_normals();

    let face_vertices = &cube_vertices[3 * 4..(3 + 1) * 4];
    vertices.extend(face_vertices);

    let face_indices = &cube_indices[3 * 6..(3 + 1) * 6];
    indices.extend(face_indices);

    let face_normals = &cube_normals[3];
    normals.extend([*face_normals; 4]);

    let _test_color: [[f32; 4]; 4] = [[0.0, 0.0, 1.0, 1.0]; 4];

    let face_colors = match block_type {
        BlockType::Air => [BlockType::Air.color(); 4],
        BlockType::Stone => [BlockType::Stone.color(); 4],
        BlockType::Dirt => [BlockType::Dirt.color(); 4],
        BlockType::Grass => [BlockType::Grass.color(); 4],
        BlockType::Snow => [BlockType::Snow.color(); 4],
        BlockType::Water => [BlockType::Water.color(); 4],
    };
    colors.extend(face_colors);
}

fn add_front_face(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    voxel_pos: Vec3,
    block_type: &BlockType,
    index_offset: u32,
) {
    let cube_vertices = generate_cube_vertices(voxel_pos);
    let cube_indices = generate_cube_indices(index_offset);
    let cube_normals = generate_cube_normals();

    let face_vertices = &cube_vertices[4 * 4..(4 + 1) * 4];
    vertices.extend(face_vertices);

    let face_indices = &cube_indices[4 * 6..(4 + 1) * 6];
    indices.extend(face_indices);

    let face_normals = &cube_normals[4];
    normals.extend([*face_normals; 4]);

    let _test_color: [[f32; 4]; 4] = [[1.0, 1.0, 1.0, 1.0]; 4];

    let face_colors = match block_type {
        BlockType::Air => [BlockType::Air.color(); 4],
        BlockType::Stone => [BlockType::Stone.color(); 4],
        BlockType::Dirt => [BlockType::Dirt.color(); 4],
        BlockType::Grass => [BlockType::Grass.color(); 4],
        BlockType::Snow => [BlockType::Snow.color(); 4],
        BlockType::Water => [BlockType::Water.color(); 4],
    };
    colors.extend(face_colors);
}

fn add_back_face(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    voxel_pos: Vec3,
    block_type: &BlockType,
    index_offset: u32,
) {
    let cube_vertices = generate_cube_vertices(voxel_pos);
    let cube_indices = generate_cube_indices(index_offset);
    let cube_normals = generate_cube_normals();

    let face_vertices = &cube_vertices[5 * 4..(5 + 1) * 4];
    vertices.extend(face_vertices);

    let face_indices = &cube_indices[5 * 6..(5 + 1) * 6];
    indices.extend(face_indices);

    let face_normals = &cube_normals[5];
    normals.extend([*face_normals; 4]);

    let _test_color: [[f32; 4]; 4] = [[0.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = match block_type {
        BlockType::Air => [BlockType::Air.color(); 4],
        BlockType::Stone => [BlockType::Stone.color(); 4],
        BlockType::Dirt => [BlockType::Dirt.color(); 4],
        BlockType::Grass => [BlockType::Grass.color(); 4],
        BlockType::Snow => [BlockType::Snow.color(); 4],
        BlockType::Water => [BlockType::Water.color(); 4],
    };
    colors.extend(face_colors);
}

fn add_voxel_cube(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    voxel_pos: Vec3,
    block_type: &BlockType,
    index_offset: u32,
) {
    let cube_vertices = generate_cube_vertices(voxel_pos);
    let cube_indices = generate_cube_indices(index_offset);
    let cube_normals = generate_cube_normals();

    vertices.extend(cube_vertices);
    indices.extend(cube_indices);
    for normal in &cube_normals {
        normals.extend([*normal; 4]);
    }
    // normals.extend(cube_normals);

    let face_colors = match block_type {
        BlockType::Air => [BlockType::Air.color(); 24],
        BlockType::Stone => [BlockType::Stone.color(); 24],
        BlockType::Dirt => [BlockType::Dirt.color(); 24],
        BlockType::Grass => [BlockType::Grass.color(); 24],
        BlockType::Snow => [BlockType::Snow.color(); 24],
        BlockType::Water => [BlockType::Water.color(); 24],
    };

    colors.extend(face_colors);
}
