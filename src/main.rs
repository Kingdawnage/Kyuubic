#![allow(dead_code)]
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use bracket_noise::prelude::*;
use std::collections::HashMap;

mod camera;

const CHUNK_SIZE: i32 = 8;

#[derive(Debug)]
struct Voxel {
    id: i32,
    is_solid: bool,
}

#[derive(Debug)]
struct Chunk {
    voxels: Vec<Voxel>,
}

#[derive(Debug, Resource)]
struct ChunkMap(HashMap<IVec3, Chunk>);

impl ChunkMap {
    fn generate_chunk(&mut self, chunk_pos: IVec3) {
        let mut voxels: Vec<Voxel> = Vec::new();
        let mut noise: FastNoise = FastNoise::seeded(42);
        noise.set_frequency(1.0);

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let noise_value = noise.get_noise3d(x as f32, y as f32, z as f32);
                    let is_solid = noise_value > 0.0;
                    let voxel = Voxel {
                        id: x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z,
                        is_solid,
                    };
                    voxels.push(voxel);
                }
            }
        }

        let chunk = Chunk { voxels };
        self.0.insert(chunk_pos, chunk);
        //println!("{:?}", self);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(camera::NewFlyCamera::default())
        .add_systems(Startup, setup)
        .insert_resource(ChunkMap(HashMap::new()))
        .add_systems(Update, camera::process_keyboard)
        .add_systems(Update, camera::process_mouse)
        .add_systems(Update, camera::update_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_map: ResMut<ChunkMap>,
) {
    // Spawn 3D camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        camera::NewFlyCamera::default(),
    ));

    // Add light source
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 13.0, 4.0),
        ..Default::default()
    });

    // Generate and store chunck of voxels
    let chunk_pos = IVec3::new(0, 0, 0);
    chunk_map.generate_chunk(chunk_pos);
    let (vertices, indices, normals, colors) = generate_mesh(&chunk_map, chunk_pos);

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    let mesh_handle = meshes.add(mesh);

    // Add a cube
    commands.spawn(PbrBundle {
        mesh: mesh_handle,
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.0, 0.0),
            cull_mode: None,
            ..Default::default()
        }),
        transform: Transform::from_xyz(20.0, 0.5, 0.0),
        ..Default::default()
    });

    // Iterate through voxels in the chunck and spawn them
    if let Some(chunk) = chunk_map.0.get(&chunk_pos) {
        for voxel in &chunk.voxels {
            if voxel.is_solid {
                // Calculate voxel position based on chunck_pos and voxel id
                let x = voxel.id % CHUNK_SIZE;
                let y = (voxel.id / CHUNK_SIZE) % CHUNK_SIZE;
                let z = voxel.id / (CHUNK_SIZE * CHUNK_SIZE);

                // Spawn a voxel cube
                commands.spawn(PbrBundle {
                    mesh: meshes.add(Cuboid::from_size(Vec3::new(1.0, 1.0, 1.0))),
                    material: materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        ..Default::default()
                    }),
                    transform: Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32)),
                    ..Default::default()
                });
            }
        }
    }
}

fn generate_mesh(
    chunk_map: &ChunkMap,
    chunk_pos: IVec3,
) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 3]>, Vec<[f32; 4]>) {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut index_offset = 0;

    if let Some(chunk) = chunk_map.0.get(&chunk_pos) {
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
                colors.extend([[0.0, 2.0, 0.0, 1.0]; 24]);

                index_offset += 24;
            }
        }
    }

    return (vertices, indices, normals, colors);
}

fn generate_cube_vertices(pos: Vec3) -> Vec<[f32; 3]> {
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

fn generate_cube_indices(index: u32) -> Vec<u32> {
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

fn generate_cube_normals() -> Vec<[f32; 3]> {
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
