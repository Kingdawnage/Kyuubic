#![allow(dead_code)]
use bevy::prelude::*;
mod camera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(camera::NewFlyCamera::default())
        .add_systems(Startup, setup)
        // .add_systems(Update, fly_camera_system)
        .add_systems(Update, camera::process_keyboard)
        .add_systems(Update, camera::process_mouse)
        .add_systems(Update, camera::update_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
            range: 100.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    // Add a cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::from_size(Vec3::new(5.0, 5.0, 5.0))),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.0, 0.0),
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
}
