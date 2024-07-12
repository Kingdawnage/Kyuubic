#![allow(dead_code)]
use bevy::{input::mouse::MouseMotion, prelude::*};

#[derive(Component, Resource)]
struct FlyCamera {
    speed: f32,
    sensitivity: f32,
    enabled: bool,
}

impl Default for FlyCamera {
    fn default() -> Self {
        FlyCamera {
            speed: 12.0,
            sensitivity: 2.0,
            enabled: true,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(FlyCamera::default())
        .add_systems(Startup, setup)
        .add_systems(Update, fly_camera_system)
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
            transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        FlyCamera::default(),
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
        mesh: meshes.add(Cuboid::default()),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.0, 0.0),
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
}

fn fly_camera_system(
    time: Res<Time>,
    mut mouse_motion: EventReader<MouseMotion>,
    keys: Res<ButtonInput<KeyCode>>,
    camera: Res<FlyCamera>,
    mut query: Query<&mut Transform, With<FlyCamera>>,
) {
    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        // Movement keys
        if keys.pressed(KeyCode::KeyW) {
            direction.z -= 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            direction.z += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        if keys.pressed(KeyCode::Space) {
            direction.y += 1.0;
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            direction.y -= 1.0;
        }

        // Normalize direction to ensure consistent speed
        if direction != Vec3::ZERO {
            direction = direction.normalize();
        }

        // Update position based on direction and speed
        let delta_time = time.delta_seconds();
        transform.translation += direction * camera.speed * delta_time;

        for motion in mouse_motion.read() {
            let yaw = -motion.delta.x * 0.003;
            let pitch = -motion.delta.y * 0.002;
            // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
            transform.rotate_y(yaw);
            transform.rotate_local_x(pitch);
        }
    }
}
