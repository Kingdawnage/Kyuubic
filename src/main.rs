#![allow(dead_code)]
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use std::collections::HashMap;

mod camera;
mod mesh;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(camera::NewFlyCamera::default())
        .add_systems(Startup, setup)
        .insert_resource(mesh::ChunkMap(HashMap::new()))
        .add_systems(Update, camera::process_keyboard)
        .add_systems(Update, camera::process_mouse)
        .add_systems(Update, camera::update_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_map: ResMut<mesh::ChunkMap>,
) {
    // Spawn 3D camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
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
        transform: Transform::from_xyz(4.0, 64.0, 64.0),
        ..Default::default()
    });

    // // Generate and store chunck of voxels
    // let chunk_pos = IVec3::new(0, 0, 0);
    // chunk_map.generate_chunk(chunk_pos);
    // let (vertices, indices, normals, colors) = mesh::generate_mesh(&chunk_map, chunk_pos);

    // let mut mesh = Mesh::new(
    //     PrimitiveTopology::TriangleList,
    //     RenderAssetUsages::default(),
    // );
    // mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    // mesh.insert_indices(Indices::U32(indices));
    // mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    // let mesh_handle = meshes.add(mesh);

    // commands.spawn(PbrBundle {
    //     mesh: mesh_handle,
    //     material: materials.add(StandardMaterial {
    //         cull_mode: None,
    //         ..Default::default()
    //     }),
    //     transform: Transform::from_xyz(-1.0, 0.5, 0.0),
    //     ..Default::default()
    // });

    // Generate and store a world of chunks
    let world_size = IVec3::new(8, 4, 8);
    chunk_map.generate_terrain(world_size);

    // Create entities for each chunk
    for (chunk_pos, _) in chunk_map.0.iter() {
        let (vertices, indices, normals, colors) = mesh::generate_mesh(&chunk_map, *chunk_pos);

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_indices(Indices::U32(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        let mesh_handle = meshes.add(mesh);

        commands.spawn(PbrBundle {
            mesh: mesh_handle,
            material: materials.add(StandardMaterial {
                cull_mode: None,
                ..Default::default()
            }),
            transform: Transform::from_xyz(
                chunk_pos.x as f32 * mesh::CHUNK_SIZE as f32,
                chunk_pos.y as f32 * mesh::CHUNK_SIZE as f32,
                chunk_pos.z as f32 * mesh::CHUNK_SIZE as f32,
            ),
            ..Default::default()
        });
    }
}
