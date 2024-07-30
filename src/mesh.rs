#![allow(dead_code)]
use bevy::prelude::*;
use bracket_noise::prelude::*;
use rand::Rng;
// use rayon::vec;

use std::{collections::HashMap, fs::File, io::Write};

pub const CHUNK_SIZE: i32 = 32;
pub const CHUNK_HEIGHT: i32 = 64;
pub const SEA_LEVEL: i32 = 20;

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.normals.clear();
        self.colors.clear();
    }

    pub fn insert_attribute(&mut self, attribute: &str, data: Vec<[f32; 3]>) {
        match attribute {
            "position" => self.vertices = data,
            "normal" => self.normals = data,
            _ => panic!("Invalid attribute name: {}", attribute),
        }
    }

    pub fn insert_indices(&mut self, data: Vec<u32>) {
        self.indices = data;
    }

    pub fn insert_colors(&mut self, data: Vec<[f32; 4]>) {
        self.colors = data;
    }
}

pub struct MeshCollection {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
}

impl MeshCollection {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.normals.clear();
        self.colors.clear();
    }

    pub fn insert_mesh(&mut self, mesh: &Mesh) {
        self.vertices.extend(&mesh.vertices);
        self.indices.extend(&mesh.indices);
        self.normals.extend(&mesh.normals);
        self.colors.extend(&mesh.colors);
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
    }
}

pub fn generate_mesh(
    chunk_map: &ChunkMap,
) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 3]>, Vec<[f32; 4]>) {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();

    let mut index_offset: u32 = 0;

    let terrain_voxels = gather_voxels(chunk_map);

    for ((x, y, z), voxel) in &terrain_voxels {
        let voxel_pos = Vec3::new(*x as f32, *y as f32, *z as f32);

        if voxel.is_solid {
            if !terrain_voxels
                .get(&(*x, y - 1, *z))
                .map_or(false, |v| v.is_solid)
            {
                add_bottom(
                    &mut vertices,
                    &mut indices,
                    &mut normals,
                    &mut colors,
                    voxel_pos,
                    &voxel.block_type,
                    index_offset,
                );
                index_offset += 4;
            }

            if !terrain_voxels
                .get(&(*x, *y + 1, *z))
                .map_or(false, |v| v.is_solid)
            {
                add_top(
                    &mut vertices,
                    &mut indices,
                    &mut normals,
                    &mut colors,
                    voxel_pos,
                    &voxel.block_type,
                    index_offset,
                );
                index_offset += 4;
            }
            if !terrain_voxels
                .get(&(*x - 1, *y, *z))
                .map_or(false, |v| v.is_solid)
            {
                add_left(
                    &mut vertices,
                    &mut indices,
                    &mut normals,
                    &mut colors,
                    voxel_pos,
                    &voxel.block_type,
                    index_offset,
                );
                index_offset += 4;
            }

            if !terrain_voxels
                .get(&(*x + 1, *y, *z))
                .map_or(false, |v| v.is_solid)
            {
                add_right(
                    &mut vertices,
                    &mut indices,
                    &mut normals,
                    &mut colors,
                    voxel_pos,
                    &voxel.block_type,
                    index_offset,
                );
                index_offset += 4;
            }

            if !terrain_voxels
                .get(&(*x, *y, *z + 1))
                .map_or(false, |v| v.is_solid)
            {
                add_front(
                    &mut vertices,
                    &mut indices,
                    &mut normals,
                    &mut colors,
                    voxel_pos,
                    &voxel.block_type,
                    index_offset,
                );
                index_offset += 4;
            }

            if !terrain_voxels
                .get(&(*x, *y, *z - 1))
                .map_or(false, |v| v.is_solid)
            {
                add_back(
                    &mut vertices,
                    &mut indices,
                    &mut normals,
                    &mut colors,
                    voxel_pos,
                    &voxel.block_type,
                    index_offset,
                );
                index_offset += 4;
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
    let face_colors = vec![block_type.color(); 4];

    colors.extend(face_colors);
}

pub fn collect_terrain_data(chunk_map: &ChunkMap) {
    let terrain_map = gather_voxels(chunk_map);
    // Write terrain data to a file
    let mut file = File::create("terrain_map.txt").expect("Unable to create file");
    for ((x, y, z), voxel) in &terrain_map {
        writeln!(file, "{},{},{},{}", x, y, z, voxel.is_solid).expect("Unable to write data");
    }
}

pub fn gather_voxels(chunk_map: &ChunkMap) -> HashMap<(i32, i32, i32), &Voxel> {
    let mut voxel_map: HashMap<(i32, i32, i32), &Voxel> = HashMap::new();

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
            voxel_map.insert(world_pos, voxel);
        }
    }

    voxel_map
}

pub fn test_mesh() -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 3]>, Vec<[f32; 4]>) {
    let mut map: HashMap<IVec3, &Voxel> = HashMap::new();

    let voxel = Voxel {
        id: 0,
        is_solid: true,
        block_type: BlockType::Grass,
    };

    let voxel_positions = vec![
        IVec3::new(0, 0, 0),
        IVec3::new(1, 0, 0),
        IVec3::new(2, 0, 0),
        IVec3::new(0, 1, 0),
        IVec3::new(1, 1, 0),
        IVec3::new(2, 1, 0),
        IVec3::new(0, 2, 0),
        IVec3::new(1, 2, 0),
        IVec3::new(2, 2, 0), //
        IVec3::new(0, 0, 1),
        IVec3::new(1, 0, 1),
        IVec3::new(2, 0, 1),
        IVec3::new(0, 1, 1),
        IVec3::new(1, 1, 1),
        IVec3::new(2, 1, 1),
        IVec3::new(0, 2, 1),
        IVec3::new(1, 2, 1),
        IVec3::new(2, 2, 1), //
        IVec3::new(0, 0, 2),
        IVec3::new(1, 0, 2),
        IVec3::new(2, 0, 2),
        IVec3::new(0, 1, 2),
        IVec3::new(1, 1, 2),
        IVec3::new(2, 1, 2),
        IVec3::new(0, 2, 2),
        IVec3::new(1, 2, 2),
        IVec3::new(2, 2, 2),
    ];
    for voxel_pos in voxel_positions {
        map.insert(voxel_pos, &voxel);
    }

    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut index_offset: u32 = 0;

    for (voxel_pos, voxel) in &map {
        let voxel_pos = voxel_pos.as_vec3();
        {
            add_top(
                &mut vertices,
                &mut indices,
                &mut normals,
                &mut colors,
                voxel_pos,
                &voxel.block_type,
                index_offset,
            );
        }
        index_offset += 4;
    }

    return (vertices, indices, normals, colors);
}

fn add_top(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    voxel_pos: Vec3,
    block_type: &BlockType,
    index_offset: u32,
) {
    let x = voxel_pos.x;
    let y = voxel_pos.y;
    let z = voxel_pos.z;

    let face_vertices = vec![
        [x + 0.0, y + 1.0, z + 1.0], // 0
        [x + 0.0, y + 1.0, z + 0.0], // 1 // Top face
        [x + 1.0, y + 1.0, z + 0.0], // 2
        [x + 1.0, y + 1.0, z + 1.0], // 3
    ];
    vertices.extend(face_vertices);

    let face_indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0]
        .into_iter()
        .map(|i| i + index_offset)
        .collect();
    indices.extend(face_indices);

    let face_normals = vec![[0.0, 1.0, 0.0]; 4];
    normals.extend(&face_normals);

    let _test_color: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = vec![block_type.color(); 4];
    colors.extend(face_colors);
}

fn add_bottom(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    voxel_pos: Vec3,
    block_type: &BlockType,
    index_offset: u32,
) {
    let x = voxel_pos.x;
    let y = voxel_pos.y;
    let z = voxel_pos.z;

    let face_vertices = vec![
        [x + 1.0, y + 0.0, z + 1.0], // 4
        [x + 0.0, y + 0.0, z + 1.0], // 5 // Bottom face
        [x + 0.0, y + 0.0, z + 0.0], // 6
        [x + 1.0, y + 0.0, z + 0.0], // 7
    ];
    vertices.extend(face_vertices);

    let face_indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0]
        .into_iter()
        .map(|i| i + index_offset)
        .collect();
    indices.extend(face_indices);

    let face_normals = vec![[0.0, -1.0, 0.0]; 4];
    normals.extend(&face_normals);

    let _test_color: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = vec![block_type.color(); 4];
    colors.extend(face_colors);
}

fn add_left(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    voxel_pos: Vec3,
    block_type: &BlockType,
    index_offset: u32,
) {
    let x = voxel_pos.x;
    let y = voxel_pos.y;
    let z = voxel_pos.z;

    let face_vertices = vec![
        [x + 0.0, y + 0.0, z + 1.0], // 8
        [x + 0.0, y + 1.0, z + 1.0], // 9 // Left face
        [x + 0.0, y + 1.0, z + 0.0], // 10
        [x + 0.0, y + 0.0, z + 0.0], // 11
    ];
    vertices.extend(face_vertices);

    let face_indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0]
        .into_iter()
        .map(|i| i + index_offset)
        .collect();
    indices.extend(face_indices);

    let face_normals = vec![[-1.0, 0.0, 0.0]; 4];
    normals.extend(&face_normals);

    let _test_color: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = vec![block_type.color(); 4];
    colors.extend(face_colors);
}

fn add_right(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    voxel_pos: Vec3,
    block_type: &BlockType,
    index_offset: u32,
) {
    let x = voxel_pos.x;
    let y = voxel_pos.y;
    let z = voxel_pos.z;

    let face_vertices = vec![
        [x + 1.0, y + 0.0, z + 1.0], // 12
        [x + 1.0, y + 1.0, z + 1.0], // 13 // Right face
        [x + 1.0, y + 1.0, z + 0.0], // 14
        [x + 1.0, y + 0.0, z + 0.0], // 15
    ];
    vertices.extend(face_vertices);

    let face_indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0]
        .into_iter()
        .map(|i| i + index_offset)
        .collect();
    indices.extend(face_indices);

    let face_normals = vec![[-1.0, 0.0, 0.0]; 4];
    normals.extend(&face_normals);

    let _test_color: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = vec![block_type.color(); 4];
    colors.extend(face_colors);
}

fn add_front(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    voxel_pos: Vec3,
    block_type: &BlockType,
    index_offset: u32,
) {
    let x = voxel_pos.x;
    let y = voxel_pos.y;
    let z = voxel_pos.z;

    let face_vertices = vec![
        [x + 0.0, y + 0.0, z + 1.0], // 16
        [x + 0.0, y + 1.0, z + 1.0], // 17 // Front face
        [x + 1.0, y + 1.0, z + 1.0], // 18
        [x + 1.0, y + 0.0, z + 1.0], // 19
    ];
    vertices.extend(face_vertices);

    let face_indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0]
        .into_iter()
        .map(|i| i + index_offset)
        .collect();
    indices.extend(face_indices);

    let face_normals = vec![[-1.0, 0.0, 0.0]; 4];
    normals.extend(&face_normals);

    let _test_color: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = vec![block_type.color(); 4];
    colors.extend(face_colors);
}

fn add_back(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    voxel_pos: Vec3,
    block_type: &BlockType,
    index_offset: u32,
) {
    let x = voxel_pos.x;
    let y = voxel_pos.y;
    let z = voxel_pos.z;

    let face_vertices = vec![
        [x + 0.0, y + 0.0, z + 0.0], // 20
        [x + 0.0, y + 1.0, z + 0.0], // 21 // Back face
        [x + 1.0, y + 1.0, z + 0.0], // 22
        [x + 1.0, y + 0.0, z + 0.0], // 23
    ];
    vertices.extend(face_vertices);

    let face_indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0]
        .into_iter()
        .map(|i| i + index_offset)
        .collect();
    indices.extend(face_indices);

    let face_normals = vec![[-1.0, 0.0, 0.0]; 4];
    normals.extend(&face_normals);

    let _test_color: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = vec![block_type.color(); 4];
    colors.extend(face_colors);
}
