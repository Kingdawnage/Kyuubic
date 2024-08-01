#![allow(dead_code)]
use crate::block::{self, BlockType};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
}

impl MeshData {
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

    pub fn insert_mesh(&mut self, mesh: &MeshData) {
        self.vertices.extend(&mesh.vertices);
        self.indices.extend(&mesh.indices);
        self.normals.extend(&mesh.normals);
        self.colors.extend(&mesh.colors);
    }
}

pub fn generate_mesh(chunk_map: &block::ChunkMap) -> MeshData {
    let mut mesh = MeshData::new();
    let mut index_offset: u32 = 0;
    let mut world_map: block::WorldMap = block::WorldMap::new();
    world_map.collect_voxels(chunk_map);

    let terrain_voxels: HashMap<(i32, i32, i32), block::Voxel> = world_map.map;
    for ((x, y, z), voxel) in &terrain_voxels {
        let voxel_pos = Vec3::new(*x as f32, *y as f32, *z as f32);

        if voxel.is_solid {
            // Add top face
            if !terrain_voxels
                .get(&(*x, y + 1, *z))
                .map_or(false, |v| v.is_solid)
                || terrain_voxels
                    .get(&(*x, y + 1, *z))
                    .map_or(false, |v| v.block_type == BlockType::Water)
            {
                add_top(&mut mesh, voxel_pos, &voxel.block_type, index_offset);
                index_offset += 4;
            }
            // Add bottom face
            if !terrain_voxels
                .get(&(*x, y - 1, *z))
                .map_or(false, |v| v.is_solid)
                || terrain_voxels
                    .get(&(*x, y - 1, *z))
                    .map_or(false, |v| v.block_type == BlockType::Water)
            {
                add_bottom(&mut mesh, voxel_pos, &voxel.block_type, index_offset);
                index_offset += 4;
            }
            // Add left face
            if !terrain_voxels
                .get(&(*x - 1, *y, *z))
                .map_or(false, |v| v.is_solid)
                || terrain_voxels
                    .get(&(*x - 1, *y, *z))
                    .map_or(false, |v| v.block_type == BlockType::Water)
            {
                add_left(&mut mesh, voxel_pos, &voxel.block_type, index_offset);
                index_offset += 4;
            }
            // Add right face
            if !terrain_voxels
                .get(&(*x + 1, *y, *z))
                .map_or(false, |v| v.is_solid)
                || terrain_voxels
                    .get(&(*x + 1, *y, *z))
                    .map_or(false, |v| v.block_type == BlockType::Water)
            {
                add_right(&mut mesh, voxel_pos, &voxel.block_type, index_offset);
                index_offset += 4;
            }
            // Add front face
            if !terrain_voxels
                .get(&(*x, *y, z + 1))
                .map_or(false, |v| v.is_solid)
                || terrain_voxels
                    .get(&(*x, *y, *z + 1))
                    .map_or(false, |v| v.block_type == BlockType::Water)
            {
                add_front(&mut mesh, voxel_pos, &voxel.block_type, index_offset);
                index_offset += 4;
            }
            // Add back face
            if !terrain_voxels
                .get(&(*x, *y, z - 1))
                .map_or(false, |v| v.is_solid)
                || terrain_voxels
                    .get(&(*x, *y, *z - 1))
                    .map_or(false, |v| v.block_type == BlockType::Water)
            {
                add_back(&mut mesh, voxel_pos, &voxel.block_type, index_offset);
                index_offset += 4;
            }
        }
    }
    return mesh;
}

fn add_top(mesh: &mut MeshData, voxel_pos: Vec3, block_type: &BlockType, index_offset: u32) {
    let x = voxel_pos.x;
    let y = voxel_pos.y;
    let z = voxel_pos.z;

    let face_vertices = vec![
        [x + 0.0, y + 1.0, z + 1.0], // 0
        [x + 0.0, y + 1.0, z + 0.0], // 1 // Top face
        [x + 1.0, y + 1.0, z + 0.0], // 2
        [x + 1.0, y + 1.0, z + 1.0], // 3
    ];
    mesh.vertices.extend(face_vertices);

    let face_indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0]
        .into_iter()
        .map(|i| i + index_offset)
        .collect();
    mesh.indices.extend(face_indices);

    let face_normals = vec![[0.0, 1.0, 0.0]; 4];
    mesh.normals.extend(&face_normals);

    let _test_color: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = vec![block_type.color(); 4];
    mesh.colors.extend(face_colors);
}

fn add_bottom(mesh: &mut MeshData, voxel_pos: Vec3, block_type: &BlockType, index_offset: u32) {
    let x = voxel_pos.x;
    let y = voxel_pos.y;
    let z = voxel_pos.z;

    let face_vertices = vec![
        [x + 1.0, y + 0.0, z + 1.0], // 4
        [x + 0.0, y + 0.0, z + 1.0], // 5 // Bottom face
        [x + 0.0, y + 0.0, z + 0.0], // 6
        [x + 1.0, y + 0.0, z + 0.0], // 7
    ];
    mesh.vertices.extend(face_vertices);

    let face_indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0]
        .into_iter()
        .map(|i| i + index_offset)
        .collect();
    mesh.indices.extend(face_indices);

    let face_normals = vec![[0.0, -1.0, 0.0]; 4];
    mesh.normals.extend(&face_normals);

    let _test_color: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = vec![block_type.color(); 4];
    mesh.colors.extend(face_colors);
}

fn add_left(mesh: &mut MeshData, voxel_pos: Vec3, block_type: &BlockType, index_offset: u32) {
    let x = voxel_pos.x;
    let y = voxel_pos.y;
    let z = voxel_pos.z;

    let face_vertices = vec![
        [x + 0.0, y + 0.0, z + 1.0], // 8
        [x + 0.0, y + 1.0, z + 1.0], // 9 // Left face
        [x + 0.0, y + 1.0, z + 0.0], // 10
        [x + 0.0, y + 0.0, z + 0.0], // 11
    ];
    mesh.vertices.extend(face_vertices);

    let face_indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0]
        .into_iter()
        .map(|i| i + index_offset)
        .collect();
    mesh.indices.extend(face_indices);

    let face_normals = vec![[-1.0, 0.0, 0.0]; 4];
    mesh.normals.extend(&face_normals);

    let _test_color: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = vec![block_type.color(); 4];
    mesh.colors.extend(face_colors);
}

fn add_right(mesh: &mut MeshData, voxel_pos: Vec3, block_type: &BlockType, index_offset: u32) {
    let x = voxel_pos.x;
    let y = voxel_pos.y;
    let z = voxel_pos.z;

    let face_vertices = vec![
        [x + 1.0, y + 0.0, z + 1.0], // 12
        [x + 1.0, y + 1.0, z + 1.0], // 13 // Right face
        [x + 1.0, y + 1.0, z + 0.0], // 14
        [x + 1.0, y + 0.0, z + 0.0], // 15
    ];
    mesh.vertices.extend(face_vertices);

    let face_indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0]
        .into_iter()
        .map(|i| i + index_offset)
        .collect();
    mesh.indices.extend(face_indices);

    let face_normals = vec![[-1.0, 0.0, 0.0]; 4];
    mesh.normals.extend(&face_normals);

    let _test_color: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = vec![block_type.color(); 4];
    mesh.colors.extend(face_colors);
}

fn add_front(mesh: &mut MeshData, voxel_pos: Vec3, block_type: &BlockType, index_offset: u32) {
    let x = voxel_pos.x;
    let y = voxel_pos.y;
    let z = voxel_pos.z;

    let face_vertices = vec![
        [x + 0.0, y + 0.0, z + 1.0], // 16
        [x + 0.0, y + 1.0, z + 1.0], // 17 // Front face
        [x + 1.0, y + 1.0, z + 1.0], // 18
        [x + 1.0, y + 0.0, z + 1.0], // 19
    ];
    mesh.vertices.extend(face_vertices);

    let face_indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0]
        .into_iter()
        .map(|i| i + index_offset)
        .collect();
    mesh.indices.extend(face_indices);

    let face_normals = vec![[-1.0, 0.0, 0.0]; 4];
    mesh.normals.extend(&face_normals);

    let _test_color: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = vec![block_type.color(); 4];
    mesh.colors.extend(face_colors);
}

fn add_back(mesh: &mut MeshData, voxel_pos: Vec3, block_type: &BlockType, index_offset: u32) {
    let x = voxel_pos.x;
    let y = voxel_pos.y;
    let z = voxel_pos.z;

    let face_vertices = vec![
        [x + 0.0, y + 0.0, z + 0.0], // 20
        [x + 0.0, y + 1.0, z + 0.0], // 21 // Back face
        [x + 1.0, y + 1.0, z + 0.0], // 22
        [x + 1.0, y + 0.0, z + 0.0], // 23
    ];
    mesh.vertices.extend(face_vertices);

    let face_indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0]
        .into_iter()
        .map(|i| i + index_offset)
        .collect();
    mesh.indices.extend(face_indices);

    let face_normals = vec![[-1.0, 0.0, 0.0]; 4];
    mesh.normals.extend(&face_normals);

    let _test_color: [[f32; 4]; 4] = [[1.0, 0.0, 0.0, 1.0]; 4];

    let face_colors = vec![block_type.color(); 4];
    mesh.colors.extend(face_colors);
}

fn add_voxel_cube(mesh: &mut MeshData, voxel_pos: Vec3, block_type: &BlockType, index_offset: u32) {
    let cube_vertices = generate_cube_vertices(voxel_pos);
    let cube_indices = generate_cube_indices(index_offset);
    let cube_normals = generate_cube_normals();

    mesh.vertices.extend(cube_vertices);
    mesh.indices.extend(cube_indices);
    for normal in &cube_normals {
        mesh.normals.extend([*normal; 4]);
    }
    let face_colors = vec![block_type.color(); 4];

    mesh.colors.extend(face_colors);
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
