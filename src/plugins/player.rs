use bevy::{
    app::{Plugin, PreUpdate, Update},
    asset::{Assets, Handle},
    core::Name,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::{Added, With},
        schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter},
        system::{Commands, Query, Res, ResMut, Resource},
        world::EntityWorldMut,
    },
    input::{keyboard::KeyCode, ButtonInput},
    log::info,
    math::{
        primitives::{Cuboid, Plane3d},
        Vec2, Vec3,
    },
    pbr::{PbrBundle, StandardMaterial},
    prelude::{Deref, DerefMut},
    reflect::{std_traits::ReflectDefault, Reflect},
    render::{
        color::Color,
        mesh::{shape, Mesh},
        texture::Image,
        view::{InheritedVisibility, ViewVisibility, Visibility, VisibilityBundle},
    },
    sprite::{Sprite, SpriteBundle, SpriteSheetBundle, TextureAtlas},
    time::{Time, Timer, TimerMode},
    transform::components::{GlobalTransform, Transform},
};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_replicon::{
    client::ClientSet,
    core::replication_rules::{AppReplicationExt, Replication},
    network_event::client_event::{ClientEventAppExt, FromClient},
};
use bevy_replicon::{
    core::{common_conditions::has_authority, replicon_channels::ChannelKind},
    prelude::ClientId,
};
use serde::{Deserialize, Serialize};

use crate::GameState;

use super::crafting::logic::Inventory;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreUpdate, player_init_system.after(ClientSet::Receive))
            .replicate::<PlayerColor>()
            .replicate::<Player>()
            .add_client_event::<MoveDirection>(ChannelKind::Ordered)
            .add_systems(
                Update,
                (
                    movement_system.run_if(has_authority), // Runs only on the server or a single player.
                    input_system,
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
        let mesh_handle = meshes.add(Cuboid::from_size(Vec3::ONE / 2.0));
        let standard_material_handle = standard_material.add(StandardMaterial {
            base_color: color.0,
            ..Default::default()
        });
        commands.entity(entity).insert((
            GlobalTransform::default(),
            VisibilityBundle::default(),
            mesh_handle,
            standard_material_handle,
        ));
    }
}

/// Reads player inputs and sends [`MoveCommandEvents`]
fn input_system(mut move_events: EventWriter<MoveDirection>, input: Res<ButtonInput<KeyCode>>) {
    let mut direction = Vec3::ZERO;
    if input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::ArrowUp) {
        direction.z += 1.0;
    }
    if input.pressed(KeyCode::ArrowDown) {
        direction.z -= 1.0;
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
        for (player, mut position) in &mut players {
            if *client_id == player.0 {
                position.translation += event.0 * time.delta_seconds() * MOVE_SPEED;
            }
        }
    }
}
