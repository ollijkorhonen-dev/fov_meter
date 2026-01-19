use bevy::prelude::*;
use bevy_mod_openxr::{add_xr_plugins, resources::OxrSessionConfig};
use bevy_mod_openxr::types::EnvironmentBlendMode;
use bevy_mod_xr::hand_debug_gizmos;
use bevy_xr_utils;

fn main() {
    App::new()
        .add_plugins(FovMeterPlugin)
        .run();
}

#[derive(Component)]
pub struct AngleText;

#[derive(Component)]
pub struct FovMeterWallLeft;

#[derive(Component)]
pub struct FovMeterWallRight;

pub struct FovMeterPlugin;

impl Plugin for FovMeterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(add_xr_plugins(
                DefaultPlugins.build(),
            ))
            .insert_resource(OxrSessionConfig {
                blend_mode_preference: vec![
                    EnvironmentBlendMode::ALPHA_BLEND,
                    EnvironmentBlendMode::ADDITIVE,
                    EnvironmentBlendMode::OPAQUE,
                ],
                ..default()
            })
            .add_plugins(hand_debug_gizmos::HandGizmosPlugin)
            .add_plugins(bevy_xr_utils::tracking_utils::TrackingUtilitiesPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, fov_meter_controller)
            .insert_resource(ClearColor(Color::NONE));
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // cube aim target
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.1))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 1.6, -2.0),
    ));

    // sphere aim aid
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.04))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 255, 144))),
        Transform::from_xyz(0.0, 1.6, -1.0),
    ));

   /* // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));*/

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, -9.0).looking_at(Vec3::ZERO, Vec3::Y),
        
    ));

    // wall left
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.001, 3.0, 80.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 127, 127))),
        Transform::from_xyz(0.0, 1.5, 0.0).with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        FovMeterWallLeft,
    ));

    // wall right
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.001, 3.0, 80.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(127, 255, 127))),
        Transform::from_xyz(0.0, 1.5, 0.0).with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        FovMeterWallRight,
    ));

    // Add UI for angle display
    commands.spawn((
        Text::new("Fov angle: 0.0"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 1.0),
        AngleText,
    ));

    // floor compass
    let mut a:f32 = 0.0;
    while a<180.0 {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.03, 0.03, 8.0))),
            MeshMaterial3d(materials.add(Color::srgb_u8(127, 127, 127))),
            Transform::from_xyz(0.0, 0.04, 0.0).with_rotation(Quat::from_rotation_y(a.to_radians())),
        ));
        a += 10.0;
    }
}

fn fov_meter_controller(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut text_query: Query<&mut Text, With<AngleText>>,
    mut left_query: Query<&mut Transform, (With<FovMeterWallLeft>, Without<FovMeterWallRight>)>,
    mut right_query: Query<&mut Transform, (With<FovMeterWallRight>, Without<FovMeterWallLeft>)>,
) {
    let dt = time.delta_secs();
    let mut rotation = 0.0;
    let speed = 0.10;

    if keys.pressed(KeyCode::KeyQ) {
        rotation += speed;
    }
    if keys.pressed(KeyCode::KeyE) {
        rotation -= speed;
    }

    if let Some(gamepad) = gamepads.iter().next() {
        let right_trigger = gamepad.get(GamepadButton::RightTrigger).unwrap_or(0.0);
        let left_trigger = gamepad.get(GamepadButton::LeftTrigger).unwrap_or(0.0);
        rotation += (right_trigger - left_trigger) * speed;
    }

    if let Some(gamepad) = gamepads.iter().next() {
        let right_stick_x = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);
        if right_stick_x != 0.0 {
            rotation = right_stick_x * speed;
        }
    }

    if rotation != 0.0 {
        let rot_left = Quat::from_rotation_y(rotation * dt).normalize();
        let rot_right = Quat::from_rotation_y(-rotation * dt).normalize();
        let mut angle1 = Quat::from_rotation_y(0.0);
        let mut angle2= Quat::from_rotation_y(0.0);
        for mut t in left_query.iter_mut() {
            t.rotation = rot_left * t.rotation;
            angle1 = t.rotation;
        }
        for mut t in right_query.iter_mut() {
            t.rotation = rot_right * t.rotation;
            angle2 = t.rotation;
        }
        let angle = 180.0 - angle2.angle_between(angle1).to_degrees().ceil();
        if let Ok(mut text) = text_query.single_mut() {
            **text = format!("Wall angle: {}", angle,
            );
        }
    }
}
