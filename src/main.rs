use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_aabb_instancing::VertexPullingRenderPlugin;
use bevy_rapier3d::prelude::*;

const TAU: f32 = 6.283185307179586476925286766559;
const GRAVITY: f32 = 3.;
const PLAYER_WIDTH: f32 = 0.2;
const PLAYER_HEIGHT: f32 = 1.6;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(VertexPullingRenderPlugin::default())
        .init_resource::<Settings>()
        .add_startup_system(setup)
        .add_system(grab_cursor)
        .add_system(camera_movement)
        .add_system(camera_follow)
        .add_system(controller_update)
        .add_system(movement)
        .add_system(friction)
        .add_system(gravity)
        .add_system(debug_log)
        .run();
}

#[derive(Component)]
struct Camera;

#[derive(Component)]
struct Player {
    is_grounded: bool,
    velocity: Vec3,
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
    // Floor
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(100.0).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },
        Collider::cuboid(50.0, 0.1, 50.0),
    ));
    // Box
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Cube::new(1.0).into()),
            material: materials.add(Color::rgb(0.9, 0.1, 0.2).into()),
            transform: Transform::from_xyz(1.0, 0.5, 1.0),
            ..default()
        },
        Collider::cuboid(0.5, 0.5, 0.5),
    ));
    // Lighting
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
        TransformBundle::default(),
        RigidBody::KinematicVelocityBased,
        KinematicCharacterController { ..default() },
        Collider::cuboid(PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_WIDTH),
        Player {
            is_grounded: true,
            velocity: Vec3::ZERO,
        },
    ));
    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, PLAYER_HEIGHT, 0.0),
            ..default()
        },
        Camera,
    ));

    if let Ok(mut window) = primary_window.get_single_mut() {
        window.cursor.grab_mode = CursorGrabMode::Confined;
        window.cursor.visible = false;
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

fn camera_follow(
    mut camera_transform: Query<&mut Transform, With<Camera>>,
    player_transform: Query<&GlobalTransform, With<Player>>,
) {
    let mut camera_transform = camera_transform
        .get_single_mut()
        .expect("Camera has transform");
    let player_transform = player_transform.get_single().expect("Player has transform");
    camera_transform.translation = player_transform.translation() + Vec3::Y * PLAYER_HEIGHT;
}

fn camera_movement(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera: Query<&mut Transform, With<Camera>>,
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

                // TODO: Fix bug where looking all the way down causes an error due to normalizing
                // a zero vector.
                // pitch = pitch.clamp(-TAU / 4.0, TAU / 4.0);
                pitch = pitch.clamp(-TAU / 5.0, TAU / 5.0);

                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            }
        }
    }
}

fn controller_update(
    time: Res<Time>,
    mut player: Query<(&mut KinematicCharacterController, &Player)>,
) {
    let (mut controller, player) = player.get_single_mut().expect("Player exists");
    controller.translation = Some(player.velocity * time.delta_seconds());
}

fn movement(
    keys: Res<Input<KeyCode>>,
    camera_transform: Query<&Transform, With<Camera>>,
    mut player: Query<&mut Player>,
) {
    let mut player = player.get_single_mut().expect("Player exists");
    let camera_transform = camera_transform.get_single().expect("Camera exists");

    let forward = {
        let mut v = camera_transform.forward();
        v.y = 0.0;
        v.normalize()
    };
    let right = Vec3::new(-forward.z, 0.0, forward.x);
    let speed = 1.0;
    let jump_speed = 2.0;

    for &key in keys.get_pressed() {
        match key {
            KeyCode::E => player.velocity += forward * speed,
            KeyCode::S => player.velocity -= right * speed,
            KeyCode::D => player.velocity -= forward * speed,
            KeyCode::F => player.velocity += right * speed,
            KeyCode::Space if player.is_grounded => {
                player.velocity.y += jump_speed;
                player.is_grounded = false;
            }
            _ => {}
        }
    }
}

fn gravity(time: Res<Time>, mut player: Query<(&mut Transform, &mut KinematicCharacterControllerOutput, &mut Player)>) {
    let (mut transform, mut controller, mut player) = player.get_single_mut().expect("Player exists");
    if !player.is_grounded {
        player.velocity.y -= GRAVITY * time.delta_seconds();
    }
    if transform.translation.y < 0.0 {
        transform.translation.y = 0.0;
        player.velocity.y = 0.0;
        player.is_grounded = true;
    }
}

fn friction(mut player: Query<&mut Player>) {
    let slow_down = 0.8;
    let mut player = player.get_single_mut().expect("Player exists");
    if player.is_grounded {
        player.velocity *= slow_down;
    }
}

fn debug_log(player: Query<(&Transform, &Player)>) {
    for (transform, player) in player.iter() {
        dbg!(&transform);
        dbg!(&player.velocity);
    }
}
