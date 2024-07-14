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
        mesh: mesh_handle.clone(),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.0, 0.0),
            cull_mode: None,
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
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

                vertices.extend(generate_cube_vertices(voxel_pos));
                indices.extend(generate_cube_indices(index_offset));
                normals.extend([[1.0, 0.0, 0.0]; 6]);
                colors.extend([[0.0, 1.0, 0.0, 1.0]; 8]);

                index_offset += 8;
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
        [x + 0.0, y + 1.0, z + 0.0], // 1
        [x + 1.0, y + 1.0, z + 0.0], // 2
        [x + 1.0, y + 1.0, z + 1.0], // 3
        [x + 1.0, y + 0.0, z + 1.0], // 4
        [x + 0.0, y + 0.0, z + 1.0], // 5
        [x + 0.0, y + 0.0, z + 0.0], // 6
        [x + 1.0, y + 0.0, z + 0.0], // 7
    ]
}

fn generate_cube_indices(index: u32) -> Vec<u32> {
    vec![
        0, 1, 2, 2, 3, 0, // Top face
        5, 6, 7, 7, 4, 5, // Bottom face
        6, 1, 0, 0, 5, 6, // Left face
        4, 3, 2, 2, 7, 4, // Right face
        5, 0, 3, 3, 4, 5, // Front face
        6, 1, 2, 2, 7, 6, // Back face
    ]
    .into_iter()
    .map(|i| i + index)
    .collect()
}
