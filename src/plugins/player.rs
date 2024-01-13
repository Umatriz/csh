use bevy::{
    app::{Plugin, Startup, Update},
    core::Name,
    ecs::{
        bundle::Bundle,
        component::Component,
        query::With,
        system::{Commands, Query, Res},
    },
    input::{keyboard::KeyCode, Input},
    math::Vec2,
    reflect::{std_traits::ReflectDefault, Reflect},
    sprite::{Sprite, SpriteBundle},
    time::Time,
    transform::components::Transform,
};

use super::crafting::logic::Inventory;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, player_movement);
    }
}

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Default)]
pub struct Player;

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    pub player: Player,
    pub inventory: Inventory,
    pub properties: PlayerProperties,
}

#[derive(Component, Default)]
pub struct PlayerProperties {}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn(PlayerBundle::default())
        .insert(Name::new("Player"))
        .insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(15.0, 15.0)),
                ..Default::default()
            },
            ..Default::default()
        });
}

fn player_movement(
    mut player_q: Query<&mut Transform, With<Player>>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut direction = Vec2::ZERO;
    if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
        direction.y += 1.;
    }
    if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
        direction.y -= 1.;
    }
    if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
        direction.x += 1.;
    }
    if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
        direction.x -= 1.;
    }
    if direction == Vec2::ZERO {
        return;
    }

    let move_speed = 37.;
    let move_delta = direction * move_speed * time.delta_seconds();

    for mut player_transform in player_q.iter_mut() {
        player_transform.translation += move_delta.extend(0.);
    }
}
