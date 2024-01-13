use bevy::{
    app::{Plugin, Startup, Update},
    core::Name,
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query},
    },
    math::Vec2,
    reflect::{std_traits::ReflectDefault, Reflect},
    render::color::Color,
    sprite::{Sprite, SpriteBundle},
    transform::components::{GlobalTransform, Transform},
};
use bevy_inspector_egui::{
    bevy_egui::{egui, EguiContexts},
    egui::Id,
};

use crate::{item, item_kind, layout, utils::squared_distance};

use super::{
    crafting::{
        logic::{Inventory, Item, ItemStack},
        show_item,
    },
    player::Player,
};

pub struct ChestPlugin;

impl Plugin for ChestPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, spawn_chest)
            .add_systems(Update, check_chest);
    }
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Default)]
pub struct Chest;

fn spawn_chest(mut commands: Commands, mut items_query: Query<(&mut Item, &mut ItemStack)>) {
    let mut inventory = Inventory::new();
    inventory.add_combine(
        &mut commands,
        &mut items_query,
        &layout![item! { "ExampleItem", item_kind!(primitive), amount = 1, level = 1 }],
    );
    commands
        .spawn(Chest)
        .insert(inventory)
        .insert(Name::new("Player"))
        .insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(10.0, 10.0)),
                color: Color::GREEN,
                ..Default::default()
            },
            transform: Transform::from_xyz(10.0, 10.0, 10.0),
            ..Default::default()
        });
}

fn check_chest(
    mut contexts: EguiContexts,
    items_query: Query<(&Item, &ItemStack)>,
    mut player_query: Query<(&mut Inventory, &GlobalTransform), With<Player>>,
    mut chest_query: Query<
        (&mut Inventory, &GlobalTransform, Entity),
        (With<Chest>, Without<Player>),
    >,
) {
    let (mut player_inventory, player_transform) = player_query.get_single_mut().unwrap();
    for (mut chest_inventory, chest_transform, chest_entity) in chest_query.iter_mut() {
        if dbg!(squared_distance(
            player_transform.translation(),
            chest_transform.translation(),
        )) <= 500.0
        {
            egui::Window::new("Chest")
                .id(Id::new(chest_entity))
                .enabled(true)
                .show(contexts.ctx_mut(), |ui| {
                    ui.label("You fond a chest!");
                    for entity in chest_inventory.map.iter().filter_map(|x| *x) {
                        if let Ok(item) = items_query.get(entity) {
                            show_item(item, ui, true);
                        }
                    }
                    if ui.button("Take all items").clicked() {
                        player_inventory.join(&mut chest_inventory);
                    }
                });
        }
    }
}
