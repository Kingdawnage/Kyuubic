#![allow(dead_code)]
use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};

mod camera;
mod mesh;
mod utils;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(camera::FlyCamera::default())
        .add_systems(Startup, (setup, utils::setup_fps_counter))
        .add_systems(Update, utils::update_fps)
        .insert_resource(mesh::ChunkMap::new())
        .add_systems(
            Update,
            (
                camera::process_keyboard,
                camera::process_mouse,
                camera::update_camera,
            ),
        )
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
            transform: Transform::from_xyz(3.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        camera::FlyCamera::default(),
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

    // Generate terrain with heightmap
    let world_size = IVec3::new(6, 1, 6);
    chunk_map.generate_terrain(world_size);

    for (chunk_pos, _) in chunk_map.map.iter() {
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
                //base_color: Color::srgb(0.8, 0.0, 0.0),
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
