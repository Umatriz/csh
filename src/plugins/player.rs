use bevy::{
    app::{Plugin, PreUpdate, Update},
    asset::{Assets, Handle},
    core::Name,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::{Added, With, Without},
        schedule::{
            common_conditions::{in_state, not},
            IntoSystemConfigs, NextState, OnEnter,
        },
        system::{Commands, Query, Res, ResMut, Resource},
        world::EntityWorldMut,
    },
    input::{keyboard::KeyCode, ButtonInput},
    log::{error, info},
    math::{
        primitives::{Capsule3d, Cuboid, Direction3d, Plane3d, Rectangle},
        Quat, Vec2, Vec3,
    },
    pbr::{PbrBundle, StandardMaterial},
    prelude::{Deref, DerefMut},
    reflect::{std_traits::ReflectDefault, Reflect},
    render::{
        color::Color,
        mesh::{shape, Mesh, Meshable},
        texture::Image,
        view::{InheritedVisibility, ViewVisibility, Visibility, VisibilityBundle},
    },
    sprite::{Sprite, SpriteBundle, SpriteSheetBundle, TextureAtlas},
    time::{Time, Timer, TimerMode},
    transform::components::{GlobalTransform, Transform},
};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_replicon::{
    client::{replicon_client::RepliconClient, ClientSet},
    core::replication_rules::{AppReplicationExt, Replication},
    network_event::client_event::{ClientEventAppExt, FromClient},
};
use bevy_replicon::{
    core::{common_conditions::has_authority, replicon_channels::ChannelKind},
    prelude::ClientId,
};
use bevy_xpbd_3d::{
    components::{LinearVelocity, RigidBody},
    plugins::{
        collision::Collider,
        spatial_query::{RayCaster, RayHits},
    },
};
use serde::{Deserialize, Serialize};

use crate::GameState;

use super::{
    camera::{fly_view, FPSCamera},
    crafting::logic::Inventory,
    network::LocalPlayerId,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            PreUpdate,
            (player_init_system, init_local_player).after(ClientSet::Receive),
        )
        .replicate::<PlayerColor>()
        .replicate::<Player>()
        .add_client_event::<MoveDirection>(ChannelKind::Ordered)
        .add_systems(
            Update,
            (
                (movement_system, handle_players_controls).run_if(has_authority), // Runs only on the server or a single player.
                // player_rotation,
                input_system.run_if(not(fly_view)),
            ),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct PlayerCollection {}

#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct Player(pub ClientId);

impl Default for Player {
    fn default() -> Self {
        Self(ClientId::SERVER)
    }
}

#[derive(Debug, Default, Deserialize, Event, Serialize)]
pub struct MoveDirection(pub Vec3);

#[derive(Component, Deserialize, Serialize, Default)]
pub struct PlayerColor(pub Color);

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    pub player: Player,
    pub replication: Replication,
    pub transform: Transform,
    pub color: PlayerColor,
    pub inventory: Inventory,
}

#[derive(Component, Default)]
pub struct PlayerProperties {}

fn player_init_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_material: ResMut<Assets<StandardMaterial>>,
    spawned_players: Query<(Entity, &PlayerColor), Added<Player>>,
) {
    for (entity, color) in &spawned_players {
        info!("PLAYER INIT");
        let mesh_handle = meshes.add(Capsule3d::new(0.4, 1.0).mesh());
        let standard_material_handle = standard_material.add(StandardMaterial {
            base_color: color.0,
            ..Default::default()
        });
        commands.entity(entity).insert((
            RigidBody::Kinematic,
            Collider::capsule(1.0, 0.4),
            GlobalTransform::default(),
            VisibilityBundle::default(),
            RayCaster::new(Vec3::ZERO, Direction3d::NEG_Y),
            mesh_handle,
            standard_material_handle,
        ));
    }
}

#[derive(Component)]
pub struct LocalPLayer;

fn init_local_player(
    mut commands: Commands,
    players: Query<(Entity, &Player)>,
    local_player: Option<Res<LocalPlayerId>>,
) {
    let Some(local_player) = local_player else {
        return;
    };

    for (entity, player) in players.iter() {
        if player.0 == local_player.0 {
            commands.entity(entity).insert(LocalPLayer);
        }
    }
}

// fn player_rotation(
//     mut players: Query<&mut Transform, With<LocalPLayer>>,
//     camera: Query<&Transform, (With<FPSCamera>, Without<LocalPLayer>)>,
// ) {
//     let Ok(mut player_transform) = players.get_single_mut() else {
//         error!("More than one `LocalPlayer` found!");
//         return;
//     };

//     let Ok(camera_transform) = camera.get_single() else {
//         error!("More than one `FPSCamera` found!");
//         return;
//     };

//     let val = camera_transform.rotation;

//     player_transform.rotation = val;
// }

/// Reads player inputs and sends [`MoveCommandEvents`]
fn input_system(
    mut move_events: EventWriter<MoveDirection>,
    input: Res<ButtonInput<KeyCode>>,
    player: Query<&Transform, With<LocalPLayer>>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let mut direction = Vec3::ZERO;
    if input.pressed(KeyCode::KeyD) {
        direction += *player_transform.right();
    }
    if input.pressed(KeyCode::KeyA) {
        direction += *player_transform.left();
    }
    if input.pressed(KeyCode::KeyW) {
        direction += *player_transform.forward();
    }
    if input.pressed(KeyCode::KeyS) {
        direction += *player_transform.back();
    }
    if direction != Vec3::ZERO {
        move_events.send(MoveDirection(direction.normalize_or_zero()));
    }
}

/// Mutates [`PlayerPosition`] based on [`MoveCommandEvents`].
///
/// Fast-paced games usually you don't want to wait until server send a position back because of the latency.
/// But this example just demonstrates simple replication concept.
fn movement_system(
    time: Res<Time>,
    mut move_events: EventReader<FromClient<MoveDirection>>,
    mut players: Query<(&Player, &mut Transform)>,
) {
    const MOVE_SPEED: f32 = 3.0;
    for FromClient { client_id, event } in move_events.read() {
        for (player, mut transform) in &mut players {
            if *client_id == player.0 {
                transform.translation += event.0 * time.delta_seconds() * MOVE_SPEED;
            }
        }
    }
}

fn handle_players_controls(
    mut players: Query<(&RayCaster, &RayHits, &mut LinearVelocity, &Transform), With<Player>>,
) {
    for (ray, hits, mut velocity, transform) in players.iter_mut() {
        for hit in hits.iter() {
            println!(
                "Hit entity {:?} at {} with normal {}",
                hit.entity,
                ray.origin + *ray.direction * hit.time_of_impact,
                hit.normal,
            );
        }
    }
}
