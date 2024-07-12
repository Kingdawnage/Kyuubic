#![allow(dead_code)]
use bevy::{input::mouse::MouseMotion, prelude::*};

#[derive(Component, Resource)]
pub struct NewFlyCamera {
    sensitivity: f32,
    position: Vec3,
    front: Vec3,
    up: Vec3,
    right: Vec3,
    worldup: Vec3,
    yaw: f32,
    pitch: f32,
    transform: Transform,
    enabled: bool,
}

impl Default for NewFlyCamera {
    fn default() -> Self {
        NewFlyCamera {
            transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            sensitivity: 0.1,
            position: Vec3::new(0.0, 0.0, 10.0),
            front: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            right: Vec3::new(1.0, 0.0, 0.0),
            worldup: Vec3::new(0.0, 1.0, 0.0),
            yaw: -90.0,
            pitch: 0.0,
            enabled: true,
        }
    }
}

pub fn process_keyboard(
    mut query: Query<&mut NewFlyCamera>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for mut camera in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let delta_time = time.delta_seconds();

        if keys.pressed(KeyCode::KeyW) {
            direction += camera.front * camera.sensitivity;
        }
        if keys.pressed(KeyCode::KeyS) {
            direction -= camera.front * camera.sensitivity;
        }
        if keys.pressed(KeyCode::KeyA) {
            direction -= camera.right * camera.sensitivity;
        }
        if keys.pressed(KeyCode::KeyD) {
            direction += camera.right * camera.sensitivity;
        }
        if keys.pressed(KeyCode::Space) {
            direction += camera.up * camera.sensitivity;
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            direction -= camera.up * camera.sensitivity;
        }

        if direction != Vec3::ZERO {
            direction += direction.normalize() * delta_time;
        }
        camera.position += direction;
    }
}

pub fn process_mouse(
    mut query: Query<&mut NewFlyCamera>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    for mut camera in query.iter_mut() {
        for motion in mouse_motion.read() {
            camera.yaw += motion.delta.x * camera.sensitivity;
            camera.pitch -= motion.delta.y * camera.sensitivity;
        }
        if camera.pitch > 89.0 {
            camera.pitch = 89.0;
        }
        if camera.pitch < -89.0 {
            camera.pitch = -89.0;
        }
        update_camera_vectors(&mut camera)
    }
}

fn update_camera_vectors(camera: &mut NewFlyCamera) {
    let mut front: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    front.x = camera.yaw.to_radians().cos() * camera.pitch.to_radians().cos();
    front.y = camera.pitch.to_radians().sin();
    front.z = camera.yaw.to_radians().sin() * camera.pitch.to_radians().cos();
    camera.front = front.normalize();
    camera.right = camera.front.cross(camera.worldup).normalize();
    camera.up = camera.right.cross(camera.front).normalize();
}

pub fn update_camera(mut query: Query<(&NewFlyCamera, &mut Transform), With<Camera3d>>) {
    for (camera, mut transform) in query.iter_mut() {
        transform.translation = camera.position;
        transform.look_to(camera.front, camera.up);
    }
}
