use bevy::{
    app::{Plugin, Startup, Update},
    asset::{Asset, Assets},
    core_pipeline::{core_2d::Camera2dBundle, core_3d::Camera3dBundle},
    ecs::{
        query::{With, Without},
        schedule::{common_conditions::in_state, IntoSystemConfigs},
        system::{Commands, Query, Res, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
    log::error,
    math::{primitives::Plane3d, Vec3},
    pbr::PbrBundle,
    render::{
        camera::{Camera, OrthographicProjection},
        mesh::{shape, Mesh},
    },
    time::Time,
    transform::components::{GlobalTransform, Transform},
};

use crate::GameState;

use super::{network::LocalPlayer, player::Player};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, spawn_camera).add_systems(Update, camera_following.run_if(in_state(GameState::Game)))
        // .add_systems(
        //     Update,
        //     (
        //         // camera_follow,
        //         camera_movement
        //     )
        //         .run_if(in_state(GameState::Game)),
        // )
        ;
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn camera_following(
    mut camera: Query<&mut Transform, With<Camera>>,
    players: Query<(&GlobalTransform, &Player)>,
    local_player: Res<LocalPlayer>,
) {
    for (player_transform, player) in players.iter() {
        if player.0 == local_player.0 {
            if let Ok(mut camera_transform) = camera.get_single_mut() {
                camera_transform.look_at(player_transform.translation(), Vec3::Y);
            } else {
                error!("More than one camera found")
            }
        }
    }
}

// fn camera_following(
//     mut camera_q: Query<&mut Transform, (With<Camera>, Without<Player>)>,
//     player_q: Query<&GlobalTransform, With<Player>>,
//     time: Res<Time>,
// ) {
//     for mut camera_transform in camera_q.iter_mut() {
//         let player_transform = player_q.get_single().unwrap();
//         let position = camera_transform
//             .translation
//             .lerp(player_transform.translation(), time.delta_seconds());

//         camera_transform.translation = position;
//     }
// }

// fn camera_follow(
//     local_players: Res<LocalPlayers>,
//     players: Query<(&Player, &Transform)>,
//     mut cameras: Query<&mut Transform, (With<Camera>, Without<Player>)>,
// ) {
//     for (player, player_transform) in &players {
//         // only follow the local player
//         if !local_players.0.contains(&player.handle) {
//             continue;
//         }

//         let pos = player_transform.translation;

//         for mut transform in &mut cameras {
//             transform.translation.x = pos.x;
//             transform.translation.y = pos.y;
//         }
//     }
// }

pub fn camera_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
) {
    for mut ortho in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        // if keyboard_input.pressed(KeyCode::A) {
        //     direction -= Vec3::new(1.0, 0.0, 0.0);
        // }

        // if keyboard_input.pressed(KeyCode::D) {
        //     direction += Vec3::new(1.0, 0.0, 0.0);
        // }

        // if keyboard_input.pressed(KeyCode::W) {
        //     direction += Vec3::new(0.0, 1.0, 0.0);
        // }

        // if keyboard_input.pressed(KeyCode::S) {
        //     direction -= Vec3::new(0.0, 1.0, 0.0);
        // }

        if keyboard_input.pressed(KeyCode::KeyZ) {
            ortho.scale += 0.1;
        }

        if keyboard_input.pressed(KeyCode::KeyX) {
            ortho.scale -= 0.1;
        }

        if ortho.scale < 0.5 {
            ortho.scale = 0.5;
        }

        // let z = transform.translation.z;
        // transform.translation += time.delta_seconds() * direction * 500.;
        // // Important! We need to restore the Z values when moving the camera around.
        // // Bevy has a specific camera setup and this can mess with how our layers are shown.
        // transform.translation.z = z;
    }
}
