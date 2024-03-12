use bevy::{
    a11y::accesskit::Size,
    app::{Plugin, Startup, Update},
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        component::Component,
        event::EventReader,
        query::With,
        schedule::{common_conditions::in_state, IntoSystemConfigs, OnEnter},
        system::{Commands, Query, Res},
    },
    hierarchy::BuildChildren,
    input::{keyboard::KeyCode, mouse::MouseMotion, ButtonInput},
    log::error,
    math::{Quat, Vec3},
    render::{
        camera::{Camera, OrthographicProjection},
        color::Color,
    },
    text::TextStyle,
    transform::components::{GlobalTransform, Transform},
    ui::{
        node_bundles::{NodeBundle, TextBundle},
        AlignContent, AlignItems, AlignSelf, JustifyContent, JustifyItems, JustifySelf, Style,
    },
    window::{CursorGrabMode, PrimaryWindow, Window},
};
use bevy_xpbd_3d::math::PI;

use crate::GameState;

use super::{network::LocalPlayer, player::Player};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(OnEnter(GameState::Game), crosshair_setup)
            .add_systems(
                Update,
                (
                    camera_movement,
                    camera_following,
                    toggle_cursor,
                    toggle_side_view,
                )
                    .run_if(in_state(GameState::Game)),
            );
    }
}

#[derive(Component)]
pub struct FPSCamera {
    pub sensitivity: f32,
    pub cursor_lock: bool,
    pub cursor_lock_key: KeyCode,

    pub side_view: bool,
    pub side_view_vec: Vec3,
    pub side_view_key: KeyCode,

    pub rotation: Vec3,
}

impl Default for FPSCamera {
    fn default() -> Self {
        Self {
            sensitivity: 0.001,
            cursor_lock: false,
            cursor_lock_key: KeyCode::Escape,
            rotation: Default::default(),
            side_view: false,
            side_view_key: KeyCode::Tab,
            side_view_vec: Vec3::new(0.0, 1.0, -1.0) * 10.0,
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

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera3dBundle::default(), FPSCamera::default()));
}

fn camera_following(
    mut camera: Query<(&mut Transform, &FPSCamera)>,
    players: Query<(&GlobalTransform, &Player)>,
    local_player: Res<LocalPlayer>,
) {
    if let Ok((mut camera_transform, camera)) = camera.get_single_mut() {
        for (player_transform, player) in players.iter() {
            if player.0 == local_player.0 {
                if camera.side_view {
                    camera_transform.translation = player_transform.translation()
                        + player_transform.back()
                        + camera.side_view_vec
                } else {
                    camera_transform.translation = player_transform.translation() + Vec3::Y * 2.0;
                }
            }
        }
    }
}

fn camera_movement(
    mut camera: Query<(&mut Transform, &mut FPSCamera)>,
    mut motion: EventReader<MouseMotion>,
) {
    let Ok((mut camera_transform, mut camera)) = camera.get_single_mut() else {
        error!("More than one camera found");
        return;
    };
    if camera.cursor_lock {
        for MouseMotion { delta } in motion.read() {
            camera.rotation.y -= delta.x * camera.sensitivity;
            camera.rotation.x -= delta.y * camera.sensitivity;

            camera.rotation.x = f32::clamp(camera.rotation.x, -PI / 2.0, PI / 2.0);
        }

        let x_quat = Quat::from_axis_angle(Vec3::Y, camera.rotation.y);

        let y_quat = Quat::from_axis_angle(Vec3::X, camera.rotation.x);

        camera_transform.rotation = x_quat * y_quat;
    }
}

fn toggle_cursor(
    mut camera: Query<&mut FPSCamera>,
    keys: Res<ButtonInput<KeyCode>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        error!("More than one camera found");
        return;
    };
    if keys.just_pressed(camera.cursor_lock_key) {
        camera.cursor_lock = !camera.cursor_lock;
    }

    let mut window = window.get_single_mut().unwrap();
    if camera.cursor_lock {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    } else {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}

fn toggle_side_view(mut camera: Query<&mut FPSCamera>, keys: Res<ButtonInput<KeyCode>>) {
    let Ok(mut camera) = camera.get_single_mut() else {
        error!("More than one camera found");
        return;
    };
    if keys.just_pressed(camera.side_view_key) {
        camera.side_view = !camera.side_view;
    }
}
