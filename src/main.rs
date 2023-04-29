use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion},
    math::vec3,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use num::traits::Zero;
use std::{collections::hash_map::HashMap, ops::AddAssign};

const TAU: f32 = 6.283185307179586476925286766559;
const GRAVITY: f32 = 6.9;
const PLAYER_HEIGHT: f32 = 1.6;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Settings>()
        .add_startup_system(setup)
        .add_system(grab_cursor)
        .add_system(camera_movement)
        .add_system(movement)
        .add_system(gravity)
        .run();
}

#[derive(Component)]
struct Player {
    vertical_velocity: f32,
    is_grounded: bool,
    transform: Transform,
}

#[derive(Resource)]
struct Settings {
    sensitivity: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { sensitivity: 0.001 }
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(20.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Cube::new(1.0).into()),
        material: materials.add(Color::rgb(0.9, 0.1, 0.2).into()),
        transform: Transform::from_xyz(1.0, 0.5, 1.0),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Player
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, PLAYER_HEIGHT, 0.0),
            ..default()
        },
        Player {
            vertical_velocity: 0.0,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            is_grounded: true,
        },
    ));

    if let Ok(mut window) = primary_window.get_single_mut() {
        window.cursor.grab_mode = CursorGrabMode::Confined;
        window.cursor.visible = false;
    }
}

fn camera_movement(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera: Query<&mut Transform, With<Player>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<Settings>,
) {
    if let Ok(win) = primary_window.get_single() {
        for MouseMotion { delta } in mouse_motion_events.iter() {
            for mut transform in camera.iter_mut() {
                let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                match win.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        yaw -= delta.x * settings.sensitivity;
                        pitch -= delta.y * settings.sensitivity;
                    }
                }

                dbg!(&yaw, &pitch);
                // TODO: Fix bug where looking all the way down causes an error due to normalizing
                // a zero vector.
                // pitch = pitch.clamp(-TAU / 4.0, TAU / 4.0);
                pitch = pitch.clamp(-TAU / 5.0, TAU / 5.0);

                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
                dbg!(&transform);
            }
        }
    }
}

fn grab_cursor(
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut win = window.single_mut();
    if keys.just_pressed(KeyCode::Escape) {
        win.cursor.grab_mode = CursorGrabMode::None;
        win.cursor.visible = true;
    }
    if mouse_buttons.just_pressed(MouseButton::Left) {
        win.cursor.grab_mode = CursorGrabMode::Confined;
        win.cursor.visible = false;
    }
}

fn movement(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut player: Query<(&mut Transform, &mut Player)>,
) {
    for (mut transform, mut player) in player.iter_mut() {
        let forward = player.transform.forward();
        let right = player.transform.right();
        let up = Vec3::Y;
        let speed = 3.0;

        for &key in keys.get_pressed() {
            match key {
                KeyCode::E => {
                    player.transform.translation += forward * speed * time.delta_seconds()
                }
                KeyCode::S => player.transform.translation += -right * speed * time.delta_seconds(),
                KeyCode::D => {
                    player.transform.translation += -forward * speed * time.delta_seconds()
                }
                KeyCode::F => player.transform.translation += right * speed * time.delta_seconds(),
                KeyCode::Space if player.is_grounded => {
                    player.vertical_velocity += speed;
                    player.is_grounded = false;
                }
                _ => {}
            }
        }

        transform.translation += up * player.vertical_velocity * time.delta_seconds()
    }
}

fn gravity(time: Res<Time>, mut players: Query<(&mut Transform, &mut Player)>) {
    for (mut transform, mut player) in players.iter_mut() {
        if transform.translation.y > PLAYER_HEIGHT {
            player.vertical_velocity -= GRAVITY * time.delta_seconds();
        } else if transform.translation.y < PLAYER_HEIGHT {
            transform.translation.y = PLAYER_HEIGHT;
            player.vertical_velocity = 0.;
            player.is_grounded = true;
        }
    }
}
