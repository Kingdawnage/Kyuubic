use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn 3D camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

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
        mesh: meshes.add(Cuboid::default()),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.0, 0.0),
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
}
