use bevy::{
    app::{Plugin, Update},
    asset::Handle,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::{Added, With},
        schedule::{common_conditions::in_state, IntoSystemConfigs},
        system::{Commands, Query, Res},
        world::EntityWorldMut,
    },
    math::Vec2,
    render::{camera::Camera, color::Color, texture::Image, view::VisibilityBundle},
    sprite::Sprite,
    time::Time,
    transform::components::{GlobalTransform, Transform},
    window::{PrimaryWindow, Window},
};
use bevy_mod_picking::pointer::PointerLocation;
use bevy_replicon::{
    network_event::{
        client_event::{ClientEventAppExt, FromClient},
        EventType,
    },
    renet::ClientId,
    replicon_core::replication_rules::{AppReplicationExt, Replication},
    server::has_authority,
};
use serde::{Deserialize, Serialize};

use crate::GameState;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.replicate::<CursorColor>()
            .replicate::<Cursor>()
            .add_client_event::<CursorMoveEvent>(EventType::Ordered)
            .add_systems(
                Update,
                (
                    cursor_init,
                    cursor_input.run_if(in_state(GameState::Game)),
                    cursor_movement_event.run_if(has_authority()),
                ),
            );
    }
}

#[derive(Bundle)]
pub struct CursorBundle {
    pub cursor: Cursor,
    pub color: CursorColor,
    pub transform: Transform,
    pub replication: Replication,
}

#[derive(Component, Serialize, Deserialize)]
pub struct CursorColor(pub Color);

#[derive(Component, Serialize, Deserialize)]
pub struct Cursor(pub ClientId);

#[derive(Event, Serialize, Deserialize)]
pub struct CursorMoveEvent(pub Vec2);

fn cursor_init(mut commands: Commands, spawned_players: Query<Entity, Added<Cursor>>) {
    for entity in &spawned_players {
        commands
            .entity(entity)
            .insert((
                GlobalTransform::default(),
                VisibilityBundle::default(),
                Handle::<Image>::default(),
            ))
            .add(|mut c: EntityWorldMut<'_>| {
                if let Some(color) = c.get::<CursorColor>() {
                    c.insert(Sprite {
                        color: color.0,
                        custom_size: Some(Vec2::splat(30.0)),
                        ..Default::default()
                    });
                }
            });
    }
}

fn cursor_movement_event(
    mut move_events: EventReader<FromClient<CursorMoveEvent>>,
    mut cursors: Query<(&Cursor, &mut Transform)>,
) {
    for FromClient { client_id, event } in move_events.read() {
        for (cursor, mut cursor_transform) in cursors.iter_mut() {
            if *client_id == cursor.0 {
                let position = cursor_transform.translation.lerp(event.0.extend(0.0), 0.5);
                cursor_transform.translation = position;
            }
        }
    }
}

fn cursor_input(
    camera: Query<(&Camera, &GlobalTransform)>,
    // window: Query<&Window, With<PrimaryWindow>>,
    pointer: Query<&PointerLocation>,
    mut move_events: EventWriter<CursorMoveEvent>,
) {
    // let window = window.get_single().unwrap();
    let (camera, camera_transform) = camera.get_single().unwrap();
    let pointer = pointer.get_single().unwrap();

    if let Some(world_position) = pointer
        .location()
        .and_then(|location| camera.viewport_to_world(camera_transform, location.position))
        .map(|ray| ray.origin.truncate())
    {
        move_events.send(CursorMoveEvent(world_position))
    }
}
