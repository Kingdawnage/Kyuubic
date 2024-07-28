#![allow(dead_code)]
use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
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
        .add_plugins(WireframePlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(camera::FlyCamera::default())
        .add_systems(Startup, (setup, utils::setup_fps_counter))
        .add_systems(Update, (utils::update_fps, utils::toggle_wireframe_system))
        .insert_resource(mesh::ChunkMap::new())
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::WHITE,
            ..Default::default()
        })
        .insert_resource(utils::WireframeState::default())
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
            transform: camera::FlyCamera::default().get_transform(),
            ..Default::default()
        },
        camera::FlyCamera::default(),
    ));

    // Add light source
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2000.0,
            range: 1000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 64.0, 64.0),
        ..Default::default()
    });

    // Generate terrain with heightmap
    let world_size = IVec3::new(3, 1, 3);
    chunk_map.generate_terrain(world_size);

    let (vertices, indices, normals, colors) = mesh::generate_mesh(&chunk_map);

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
        ..Default::default()
    });
}
