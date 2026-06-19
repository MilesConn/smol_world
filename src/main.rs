use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use std::f32::consts::PI;

// --- Components ---

/// Marks the player entity
#[derive(Component)]
struct Player;

/// Marks entities that can become transparent when occluding the player
#[derive(Component)]
struct Occludable {
    original_color: Color,
}

/// Marks the main camera
#[derive(Component)]
struct MainCamera;

/// Orbital camera controller state
#[derive(Resource)]
struct OrbitCamera {
    /// Horizontal angle (yaw) in radians
    yaw: f32,
    /// Vertical angle (pitch) in radians, clamped
    pitch: f32,
    /// Distance from the target (fixed)
    distance: f32,
    /// The point the camera orbits around
    target: Vec3,
    /// Whether right mouse button is held for rotation
    rotating: bool,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw: PI / 4.0,
            pitch: PI / 6.0,
            distance: 12.0,
            target: Vec3::ZERO,
            rotating: false,
        }
    }
}

/// Marks a garden plot
#[derive(Component)]
struct Garden;

/// Marks the house structure
#[derive(Component)]
struct House;

/// Player movement state
#[derive(Resource, Default)]
struct PlayerState {
    /// Which face of the cube the player is on (0-5)
    current_face: usize,
}

// --- Constants ---
const CUBE_SIZE: f32 = 4.0;
const HALF_CUBE: f32 = CUBE_SIZE / 2.0;
const PLAYER_HEIGHT: f32 = 0.4;
const PLAYER_RADIUS: f32 = 0.2;
const MOVE_SPEED: f32 = 3.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Smol World - Cube Garden".into(),
                resolution: (800., 600.).into(),
                canvas: Some("#bevy".to_owned()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .init_resource::<OrbitCamera>()
        .init_resource::<PlayerState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            orbit_camera_input,
            orbit_camera_update,
            player_movement,
            occlusion_system,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Cube planet - the ground
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.6, 0.2),
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO),
    ));

    // Player - a small capsule on top of the cube
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(PLAYER_RADIUS, PLAYER_HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.4, 0.9),
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, HALF_CUBE + PLAYER_HEIGHT / 2.0 + PLAYER_RADIUS, 0.0)),
        Player,
    ));

    // House - a group of shapes
    spawn_house(&mut commands, &mut meshes, &mut materials);

    // Garden - some plants/flowers
    spawn_garden(&mut commands, &mut meshes, &mut materials);

    // Isometric camera
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scale: 0.02,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(Vec3::new(8.0, 6.0, 8.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    // Directional light (sun)
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0)),
    ));
}

fn spawn_house(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let house_pos = Vec3::new(1.2, HALF_CUBE, 1.2);
    let wall_color = Color::srgba(0.8, 0.6, 0.4, 1.0);
    let roof_color = Color::srgba(0.7, 0.2, 0.1, 1.0);

    // House walls
    let wall_material = materials.add(StandardMaterial {
        base_color: wall_color,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.2, 1.0, 1.2))),
        MeshMaterial3d(wall_material.clone()),
        Transform::from_translation(house_pos + Vec3::new(0.0, 0.5, 0.0)),
        Occludable { original_color: wall_color },
        House,
    ));

    // Roof
    let roof_material = materials.add(StandardMaterial {
        base_color: roof_color,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.4, 0.3, 1.4))),
        MeshMaterial3d(roof_material.clone()),
        Transform::from_translation(house_pos + Vec3::new(0.0, 1.15, 0.0)),
        Occludable { original_color: roof_color },
        House,
    ));

    // Second floor / chimney
    let chimney_color = Color::srgba(0.5, 0.3, 0.3, 1.0);
    let chimney_material = materials.add(StandardMaterial {
        base_color: chimney_color,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.3, 0.5, 0.3))),
        MeshMaterial3d(chimney_material),
        Transform::from_translation(house_pos + Vec3::new(0.3, 1.55, 0.3)),
        Occludable { original_color: chimney_color },
        House,
    ));
}

fn spawn_garden(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let garden_base = Vec3::new(-1.2, HALF_CUBE, -0.8);

    // Garden fence posts
    let fence_color = Color::srgb(0.6, 0.4, 0.2);
    let fence_material = materials.add(StandardMaterial {
        base_color: fence_color,
        ..default()
    });

    let fence_positions = [
        Vec3::new(-0.8, 0.15, -0.6),
        Vec3::new(0.8, 0.15, -0.6),
        Vec3::new(-0.8, 0.15, 0.6),
        Vec3::new(0.8, 0.15, 0.6),
    ];

    for pos in &fence_positions {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.08, 0.3, 0.08))),
            MeshMaterial3d(fence_material.clone()),
            Transform::from_translation(garden_base + *pos),
            Garden,
        ));
    }

    // Plants / flowers (colored spheres)
    let plant_configs = [
        (Vec3::new(-0.4, 0.2, -0.2), Color::srgb(0.9, 0.2, 0.3), 0.12),
        (Vec3::new(0.0, 0.25, 0.0), Color::srgb(1.0, 0.8, 0.1), 0.15),
        (Vec3::new(0.4, 0.18, -0.3), Color::srgb(0.9, 0.4, 0.7), 0.1),
        (Vec3::new(-0.3, 0.2, 0.3), Color::srgb(0.4, 0.8, 0.9), 0.11),
        (Vec3::new(0.3, 0.22, 0.2), Color::srgb(1.0, 0.5, 0.0), 0.13),
        (Vec3::new(0.0, 0.3, 0.4), Color::srgb(0.8, 0.2, 0.8), 0.14),
    ];

    for (offset, color, radius) in &plant_configs {
        // Stem
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.03, offset.y * 2.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.5, 0.1),
                ..default()
            })),
            Transform::from_translation(garden_base + Vec3::new(offset.x, offset.y / 2.0, offset.z)),
            Garden,
        ));

        // Flower head
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(*radius))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: *color,
                ..default()
            })),
            Transform::from_translation(garden_base + *offset),
            Garden,
        ));
    }

    // Garden soil patches
    let soil_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.25, 0.1),
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.6, 0.05, 1.2))),
        MeshMaterial3d(soil_material),
        Transform::from_translation(garden_base + Vec3::new(0.0, 0.025, 0.0)),
        Garden,
    ));
}

/// Handle camera orbit input (right mouse drag + scroll)
fn orbit_camera_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mut orbit: ResMut<OrbitCamera>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Track right mouse button state for rotation
    orbit.rotating = mouse_button.pressed(MouseButton::Right);

    // Also allow Q/E keys for rotation
    if keys.pressed(KeyCode::KeyQ) {
        orbit.yaw += 0.02;
    }
    if keys.pressed(KeyCode::KeyE) {
        orbit.yaw -= 0.02;
    }

    // Mouse rotation
    if orbit.rotating {
        for event in mouse_motion.read() {
            orbit.yaw -= event.delta.x * 0.005;
            orbit.pitch -= event.delta.y * 0.005;
        }
    } else {
        mouse_motion.clear();
    }

    // Clamp pitch
    orbit.pitch = orbit.pitch.clamp(0.1, PI / 2.5);

    // Scroll to zoom (for orthographic, adjust scale conceptually via distance)
    for event in scroll_events.read() {
        let scroll = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 100.0,
        };
        orbit.distance -= scroll * 0.5;
        orbit.distance = orbit.distance.clamp(6.0, 20.0);
    }
}

/// Update camera position from orbital parameters
fn orbit_camera_update(
    orbit: Res<OrbitCamera>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
) {
    let Ok((mut transform, mut projection)) = camera_query.get_single_mut() else {
        return;
    };

    // Calculate camera position on sphere around target
    let x = orbit.distance * orbit.pitch.cos() * orbit.yaw.sin();
    let y = orbit.distance * orbit.pitch.sin();
    let z = orbit.distance * orbit.pitch.cos() * orbit.yaw.cos();

    let eye = orbit.target + Vec3::new(x, y, z);
    *transform = Transform::from_translation(eye).looking_at(orbit.target, Vec3::Y);

    // Adjust orthographic scale based on distance
    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale = orbit.distance * 0.002;
    }
}

/// Handle player movement with WASD relative to camera direction
fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    orbit: Res<OrbitCamera>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut player_state: ResMut<PlayerState>,
) {
    let Ok(mut player_transform) = player_query.get_single_mut() else {
        return;
    };

    let mut input = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        input.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        input.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        input.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        input.x += 1.0;
    }

    if input.length_squared() < 0.01 {
        return;
    }

    input = input.normalize();

    // Get camera forward/right on the XZ plane
    let forward = Vec3::new(-orbit.yaw.sin(), 0.0, -orbit.yaw.cos()).normalize();
    let right = Vec3::new(forward.z, 0.0, -forward.x);

    let movement = (forward * input.y + right * input.x) * MOVE_SPEED * time.delta_secs();

    // Update face position
    let current_pos = player_transform.translation;
    let new_pos = current_pos + movement;

    // Determine which face and clamp position
    let (face, clamped_pos) = clamp_to_cube_surface(new_pos);
    player_state.current_face = face;

    player_transform.translation = clamped_pos;

    // Orient player based on face normal
    let up = get_face_normal(face);
    if up != Vec3::ZERO {
        let look_dir = movement.normalize_or_zero();
        if look_dir.length_squared() > 0.01 {
            // Simple orientation: just keep upright relative to face
            let forward_on_face = look_dir - up * look_dir.dot(up);
            if forward_on_face.length_squared() > 0.001 {
                player_transform.look_to(forward_on_face.normalize(), up);
            }
        }
    }
}

/// Clamp position to cube surface and determine face
fn clamp_to_cube_surface(pos: Vec3) -> (usize, Vec3) {
    let extent = HALF_CUBE - 0.3; // Keep player slightly inset from edges
    let player_offset = PLAYER_HEIGHT / 2.0 + PLAYER_RADIUS;

    // Find which face the player should be on based on position
    let abs_pos = pos.abs();

    // Determine dominant axis
    if abs_pos.y >= abs_pos.x && abs_pos.y >= abs_pos.z {
        // Top or bottom face
        if pos.y >= 0.0 {
            // Top face (face 0)
            let x = pos.x.clamp(-extent, extent);
            let z = pos.z.clamp(-extent, extent);
            (0, Vec3::new(x, HALF_CUBE + player_offset, z))
        } else {
            // Bottom face (face 1)
            let x = pos.x.clamp(-extent, extent);
            let z = pos.z.clamp(-extent, extent);
            (1, Vec3::new(x, -HALF_CUBE - player_offset, z))
        }
    } else if abs_pos.x >= abs_pos.z {
        // Right or left face
        if pos.x >= 0.0 {
            // Right face (face 2)
            let y = pos.y.clamp(-extent, extent);
            let z = pos.z.clamp(-extent, extent);
            (2, Vec3::new(HALF_CUBE + player_offset, y, z))
        } else {
            // Left face (face 3)
            let y = pos.y.clamp(-extent, extent);
            let z = pos.z.clamp(-extent, extent);
            (3, Vec3::new(-HALF_CUBE - player_offset, y, z))
        }
    } else {
        // Front or back face
        if pos.z >= 0.0 {
            // Front face (face 4)
            let x = pos.x.clamp(-extent, extent);
            let y = pos.y.clamp(-extent, extent);
            (4, Vec3::new(x, y, HALF_CUBE + player_offset))
        } else {
            // Back face (face 5)
            let x = pos.x.clamp(-extent, extent);
            let y = pos.y.clamp(-extent, extent);
            (5, Vec3::new(x, y, -HALF_CUBE - player_offset))
        }
    }
}

/// Get normal vector for a face
fn get_face_normal(face: usize) -> Vec3 {
    match face {
        0 => Vec3::Y,
        1 => Vec3::NEG_Y,
        2 => Vec3::X,
        3 => Vec3::NEG_X,
        4 => Vec3::Z,
        5 => Vec3::NEG_Z,
        _ => Vec3::Y,
    }
}

/// Occlusion system: makes objects between camera and player transparent
fn occlusion_system(
    camera_query: Query<&Transform, With<MainCamera>>,
    player_query: Query<&Transform, With<Player>>,
    mut occludable_query: Query<(
        &Transform,
        &Occludable,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let camera_pos = camera_transform.translation;
    let player_pos = player_transform.translation;

    // Direction from camera to player
    let to_player = player_pos - camera_pos;
    let ray_length = to_player.length();
    let ray_dir = to_player / ray_length;

    for (obj_transform, occludable, material_handle) in occludable_query.iter_mut() {
        let obj_pos = obj_transform.translation;

        // Check if object is between camera and player
        let to_obj = obj_pos - camera_pos;
        let proj = to_obj.dot(ray_dir);

        // Object must be between camera and player (along the ray)
        let is_between = proj > 0.0 && proj < ray_length;

        // Check perpendicular distance to the ray
        let closest_point_on_ray = camera_pos + ray_dir * proj;
        let distance_to_ray = (obj_pos - closest_point_on_ray).length();

        // Object is occluding if it's close to the ray and between camera and player
        let is_occluding = is_between && distance_to_ray < 1.2;

        // Update material alpha
        if let Some(material) = materials.get_mut(material_handle) {
            let target_alpha = if is_occluding { 0.2 } else { 1.0 };
            let current = material.base_color.alpha();
            // Smooth transition
            let new_alpha = current + (target_alpha - current) * 0.1;

            let original = occludable.original_color;
            material.base_color = original.with_alpha(new_alpha);
        }
    }
}
