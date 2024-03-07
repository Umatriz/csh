use bevy::{
    app::{Plugin, Update},
    asset::Handle,
    core::Name,
    ecs::{
        bundle::Bundle,
        component::Component,
        event::Event,
        query::With,
        schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    input::{keyboard::KeyCode, ButtonInput},
    math::{Vec2, Vec3},
    prelude::{Deref, DerefMut},
    reflect::{std_traits::ReflectDefault, Reflect},
    render::{
        color::Color,
        texture::Image,
        view::{InheritedVisibility, ViewVisibility, Visibility},
    },
    sprite::{Sprite, SpriteBundle, SpriteSheetBundle, TextureAtlas},
    time::{Time, Timer, TimerMode},
    transform::components::{GlobalTransform, Transform},
};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_replicon::core::replication_rules::Replication;
use bevy_replicon::prelude::ClientId;
use serde::{Deserialize, Serialize};

use crate::GameState;

use super::crafting::logic::Inventory;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // app.add_systems(OnEnter(GameState::Game), spawn_player)
        //     .add_systems(Update, player_movement.run_if(in_state(GameState::Game)));
    }
}

#[derive(AssetCollection, Resource)]
pub struct PlayerCollection {
    // #[asset(texture_atlas(
    //     tile_size_x = 12.,
    //     tile_size_y = 18.,
    //     columns = 4,
    //     rows = 4,
    //     padding_x = 0.,
    //     padding_y = 0.,
    //     offset_x = 0.,
    //     offset_y = 0.
    // ))]
    // #[asset(path = "CharacterSpriteSheet.png")]
    // pub atlas: Handle<TextureAtlas>,
    // #[asset(texture_atlas(
    //     tile_size_x = 16.,
    //     tile_size_y = 24.,
    //     columns = 8,
    //     rows = 4,
    //     padding_x = 0.,
    //     padding_y = 0.,
    //     offset_x = 0.,
    //     offset_y = 0.
    // ))]
    // #[asset(path = "Small-8-Direction-Characters_by_AxulArt.png")]
    // pub atlas: Handle<TextureAtlas>,
}

#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct Player(pub ClientId);

impl Default for Player {
    fn default() -> Self {
        Self(ClientId::SERVER)
    }
}

#[derive(Component, Deserialize, Serialize, Deref, DerefMut)]
pub struct PlayerPosition(pub Vec2);

#[derive(Debug, Default, Deserialize, Event, Serialize)]
pub struct MoveDirection(pub Vec2);

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

fn spawn_player(
    mut commands: Commands,
    texture: Res<PlayerCollection>,
    // mut game_state: ResMut<NextState<GameState>>,
) {
    // let animation_indices = AnimationIndices {
    //     first: 0,
    //     last: 1,
    //     current: 0,
    // };
    // commands
    //     .spawn(PlayerBundle::default())
    //     .insert(Name::new("Player"))
    //     .insert(SpriteSheetBundle {
    //         texture_atlas: texture.atlas.clone(),
    //         sprite: TextureAtlasSprite::new(0),
    //         transform: Transform::from_scale(Vec3::splat(1.0)),
    //         ..Default::default()
    //     })
    // .insert(animation_indices)
    // .insert(AnimationSets {
    //     array: [
    //         [8, 24],
    //         [9, 25],
    //         [10, 26],
    //         [11, 27],
    //         [12, 28],
    //         [13, 29],
    //         [14, 30],
    //         [15, 31],
    //     ],
    // })
    // .insert(AnimationTimer(Timer::from_seconds(
    //     0.4,
    //     TimerMode::Repeating,
    // )))

    // game_state.set(GameState::Game);
}

// fn player_movement(
//     mut player_q: Query<&mut Transform, With<Player>>,
//     keys: Res<ButtonInput<KeyCode>>,
//     time: Res<Time>,
// ) {
//     let mut direction = Vec2::ZERO;
//     if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
//         direction.y += 1.;
//     }
//     if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
//         direction.y -= 1.;
//     }
//     if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
//         direction.x += 1.;
//     }
//     if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
//         direction.x -= 1.;
//     }
//     if direction == Vec2::ZERO {
//         return;
//     }

//     let move_speed = 37.;
//     let move_delta = direction * move_speed * time.delta_seconds();

//     for mut player_transform in player_q.iter_mut() {
//         player_transform.translation += move_delta.extend(0.);
//     }
// }
