use bevy::{
    app::{Plugin, Startup, Update},
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        component::Component,
        event::{EventReader, EventWriter},
        query::With,
        schedule::{
            common_conditions::{in_state, not},
            IntoSystemConfigs, OnEnter,
        },
        system::{Commands, Query, Res, ResMut, Resource},
    },
    hierarchy::BuildChildren,
    input::{keyboard::KeyCode, mouse::MouseMotion, ButtonInput},
    log::{error, warn},
    math::{EulerRot, Quat, Vec3},
    reflect::Reflect,
    render::color::Color,
    text::TextStyle,
    time::Time,
    transform::components::{GlobalTransform, Transform},
    ui::{
        node_bundles::{NodeBundle, TextBundle},
        AlignContent, AlignItems, AlignSelf, JustifyContent, JustifyItems, JustifySelf, Style,
    },
    window::{CursorGrabMode, PrimaryWindow, Window},
};

use crate::GameState;

use super::{
    network::LocalPlayerId,
    player::{Player, RotatePlayer},
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, spawn_camera)
            .insert_resource(FlyView(false))
            .register_type::<FlyView>()
            .register_type::<FPSCamera>()
            .add_systems(OnEnter(GameState::Game), crosshair_setup)
            .add_systems(
                Update,
                (
                    camera_rotation,
                    camera_following.run_if(not(fly_view)),
                    fly_view_camera_movement.run_if(fly_view),
                    toggle_cursor,
                    toggle_fly_view,
                )
                    .run_if(in_state(GameState::Game)),
            );
    }
}

#[derive(Component, Reflect)]
pub struct FPSCamera {
    pub sensitivity: f32,
    pub cursor_lock: bool,
    pub cursor_lock_key: KeyCode,

    pub fly_view_key: KeyCode,
    pub fly_view_speed: f32,

    pub forward_key: KeyCode,
    pub backward_key: KeyCode,
    pub left_key: KeyCode,
    pub right_key: KeyCode,
    pub upward_key: KeyCode,
    pub downward_key: KeyCode,
}

impl Default for FPSCamera {
    fn default() -> Self {
        Self {
            sensitivity: 0.0001,
            cursor_lock: false,
            cursor_lock_key: KeyCode::Escape,
            fly_view_key: KeyCode::Tab,
            forward_key: KeyCode::KeyW,
            backward_key: KeyCode::KeyS,
            left_key: KeyCode::KeyA,
            right_key: KeyCode::KeyD,
            upward_key: KeyCode::Space,
            downward_key: KeyCode::ControlLeft,
            fly_view_speed: 30.0,
        }
    }
}

fn crosshair_setup(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                justify_self: JustifySelf::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                justify_items: JustifyItems::Center,
                align_self: AlignSelf::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "+",
                TextStyle {
                    font_size: 30.0,
                    color: Color::LIME_GREEN,
                    ..Default::default()
                },
            ));
        });
}

#[derive(Resource, Reflect)]
pub struct FlyView(bool);

pub fn fly_view(res: Res<FlyView>) -> bool {
    res.0
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera3dBundle::default(), FPSCamera::default()));
}

fn camera_following(
    mut camera: Query<(&mut Transform, &FPSCamera)>,
    players: Query<(&GlobalTransform, &Player)>,
    local_player: Res<LocalPlayerId>,
) {
    if let Ok((mut camera_transform, camera)) = camera.get_single_mut() {
        for (player_transform, player) in players.iter() {
            if player.0 == local_player.0 {
                camera_transform.translation = player_transform.translation();
            }
        }
    }
}

fn camera_rotation(
    mut camera: Query<(&mut Transform, &FPSCamera)>,
    mut rotate_event: EventWriter<RotatePlayer>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    fly_view: Res<FlyView>,
    mut motion: EventReader<MouseMotion>,
) {
    let Ok((mut camera_transform, camera)) = camera.get_single_mut() else {
        warn!("Cannot get `FPSCamera` in `camera_rotation`!");
        return;
    };

    let Ok(window) = primary_window.get_single() else {
        warn!("Cannot get `PrimaryWindow` in `camera_rotation`!");
        return;
    };

    if camera.cursor_lock {
        for MouseMotion { delta } in motion.read() {
            let (mut yaw, mut pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
            let window_scale = window.height().min(window.width());

            pitch -= (camera.sensitivity * delta.y * window_scale).to_radians();
            yaw -= (camera.sensitivity * delta.x * window_scale).to_radians();

            pitch = pitch.clamp(-1.54, 1.54);

            let y_quat = Quat::from_axis_angle(Vec3::Y, yaw);
            let x_quat = Quat::from_axis_angle(Vec3::X, pitch);

            if !fly_view.0 {
                rotate_event.send(RotatePlayer(y_quat));
            }

            camera_transform.rotation = y_quat * x_quat;
        }
    }
}

fn fly_view_camera_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera: Query<(&mut Transform, &FPSCamera)>,
) {
    let Ok((mut camera_transform, camera)) = camera.get_single_mut() else {
        warn!("Cannot get `FPSCamera` in `fly_view_camera_movement`!");
        return;
    };

    let mut direction = Vec3::ZERO;

    if camera.cursor_lock {
        if keys.pressed(camera.forward_key) {
            direction += *camera_transform.forward();
        }
        if keys.pressed(camera.backward_key) {
            direction += *camera_transform.back();
        }
        if keys.pressed(camera.right_key) {
            direction += *camera_transform.right();
        }
        if keys.pressed(camera.left_key) {
            direction += *camera_transform.left();
        }
        if keys.pressed(camera.upward_key) {
            direction += Vec3::Y;
        }
        if keys.pressed(camera.downward_key) {
            direction += Vec3::NEG_Y;
        }
    }

    if direction != Vec3::ZERO {
        camera_transform.translation +=
            direction.normalize_or_zero() * time.delta_seconds() * camera.fly_view_speed;
    }
}

fn toggle_cursor(
    mut camera: Query<&mut FPSCamera>,
    keys: Res<ButtonInput<KeyCode>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        error!("Cannot find `FPSCamera` in `toggle_cursor`!");
        return;
    };
    let Ok(mut window) = window.get_single_mut() else {
        warn!("Cannot find `PrimaryWindow` in `toggle_cursor`!");
        return;
    };

    if keys.just_pressed(camera.cursor_lock_key) {
        camera.cursor_lock = !camera.cursor_lock;
    }

    if camera.cursor_lock {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    } else {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}

fn toggle_fly_view(
    mut camera: Query<&FPSCamera>,
    mut fly_view: ResMut<FlyView>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(camera) = camera.get_single_mut() else {
        error!("More than one camera found");
        return;
    };

    if keys.just_pressed(camera.fly_view_key) {
        fly_view.0 = !fly_view.0
    }
}
