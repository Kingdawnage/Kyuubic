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
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    // // Add a cube
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Cuboid::from_size(Vec3::new(5.0, 5.0, 5.0))),
    //     material: materials.add(StandardMaterial {
    //         base_color: Color::srgb(0.8, 0.0, 0.0),
    //         ..Default::default()
    //     }),
    //     transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //     ..Default::default()
    // });

    // Generate and store chunck of voxels
    let chunk_pos = IVec3::new(0, 0, 0);
    chunk_map.generate_chunk(chunk_pos);
    let (vertices, indices) = generate_mesh(&chunk_map, chunk_pos);

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));

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

fn generate_mesh(chunk_map: &ChunkMap, chunk_pos: IVec3) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices = Vec::new();
    let mut index_offset = 0;

    if let Some(chunk) = chunk_map.0.get(&chunk_pos) {
        for voxel in &chunk.voxels {
            if voxel.is_solid {
                let x = voxel.id % CHUNK_SIZE;
                let y = (voxel.id / CHUNK_SIZE) % CHUNK_SIZE;
                let z = voxel.id / (CHUNK_SIZE * CHUNK_SIZE);

                let voxel_pos = Vec3::new(x as f32, y as f32, z as f32);

                vertices.extend(generate_cube_vertices(voxel_pos));
                indices.extend(generate_cube_indices(index_offset));

                index_offset += 8;
            }
        }
    }

    return (vertices, indices);
}

fn generate_cube_vertices(pos: Vec3) -> Vec<[f32; 3]> {
    let x = pos.x;
    let y = pos.y;
    let z = pos.z;

    vec![
        // Back face
        [x + 0.0, y + 0.0, z + 0.0],
        [x + 1.0, y + 0.0, z + 0.0],
        [x + 1.0, y + 1.0, z + 0.0],
        [x + 0.0, y + 1.0, z + 0.0],
        // Front face
        [x + 0.0, y + 0.0, z + 1.0],
        [x + 1.0, y + 0.0, z + 1.0],
        [x + 1.0, y + 1.0, z + 1.0],
        [x + 0.0, y + 1.0, z + 1.0],
        // Top face
        [x + 0.0, y + 1.0, z + 1.0],
        [x + 1.0, y + 1.0, z + 1.0],
        [x + 1.0, y + 1.0, z + 0.0],
        [x + 0.0, y + 1.0, z + 0.0],
        // Bottom face
        [x + 0.0, y + 0.0, z + 1.0],
        [x + 1.0, y + 0.0, z + 1.0],
        [x + 1.0, y + 0.0, z + 0.0],
        [x + 0.0, y + 0.0, z + 0.0],
        // Left face
        [x + 0.0, y + 0.0, z + 0.0],
        [x + 0.0, y + 0.0, z + 1.0],
        [x + 0.0, y + 1.0, z + 1.0],
        [x + 0.0, y + 1.0, z + 0.0],
        // Right face
        [x + 1.0, y + 0.0, z + 1.0],
        [x + 1.0, y + 0.0, z + 0.0],
        [x + 1.0, y + 1.0, z + 0.0],
        [x + 1.0, y + 1.0, z + 1.0],
    ]
}

fn generate_cube_indices(index: u32) -> Vec<u32> {
    vec![
        // Back face
        0, 1, 3, 0, 3, 2, // Front face
        4, 5, 7, 4, 7, 6, // Top face
        6, 7, 3, 6, 3, 2, // Bottom face
        4, 5, 1, 4, 1, 0, // Left face
        0, 4, 6, 0, 6, 2, // Right face
        5, 1, 3, 5, 3, 7,
    ]
    .into_iter()
    .map(|i| i + index)
    .collect()
}
