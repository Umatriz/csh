use bevy::{
    app::{Plugin, Startup, Update},
    core::Name,
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        query::{With, Without},
        schedule::IntoSystemConfigs,
        system::{Commands, Query, ResMut, Resource},
    },
    math::Vec2,
    reflect::{std_traits::ReflectDefault, Reflect},
    render::color::Color,
    sprite::{Sprite, SpriteBundle},
    transform::components::{GlobalTransform, Transform},
};
use bevy_inspector_egui::{
    bevy_egui::{egui, EguiContexts},
    quick::ResourceInspectorPlugin,
};
use bevy_mod_picking::{
    events::{Click, Pointer},
    prelude::{ListenerInput, On},
    PickableBundle,
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
        app.add_event::<CheckChest>()
            .register_type::<ChestWindowState>()
            .add_plugins(ResourceInspectorPlugin::<ChestWindowState>::default())
            .add_systems(Startup, spawn_chest)
            .add_systems(
                Update,
                (
                    check_chest,
                    handle_chest_inventory_window.after(check_chest),
                ),
            );
    }
}

#[derive(Resource, Debug, Reflect)]
pub struct ChestWindowState {
    pub is_open: bool,
    pub left_inventory: Entity,
    pub right_inventory: Entity,
}

#[derive(Event, Debug)]
struct CheckChest {
    pub chest: Entity,
}

impl From<ListenerInput<Pointer<Click>>> for CheckChest {
    fn from(value: ListenerInput<Pointer<Click>>) -> Self {
        Self {
            chest: value.target,
        }
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
        &layout![item! { "ExampleItem1", item_kind!(primitive), amount = 1, level = 1 }],
    );
    commands
        .spawn(Chest)
        .insert(inventory)
        .insert(Name::new("Chest"))
        .insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(10.0, 10.0)),
                color: Color::BLUE,
                ..Default::default()
            },
            transform: Transform::from_xyz(10.0, 10.0, 10.0),
            ..Default::default()
        })
        .insert(PickableBundle::default())
        .insert(On::<Pointer<Click>>::send_event::<CheckChest>());

    let mut inventory = Inventory::new();
    inventory.add_combine(
        &mut commands,
        &mut items_query,
        &layout![item! { "ExampleItem2", item_kind!(primitive), amount = 1, level = 1 }],
    );
    commands
        .spawn(Chest)
        .insert(inventory)
        .insert(Name::new("Chest"))
        .insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(10.0, 10.0)),
                color: Color::BLUE,
                ..Default::default()
            },
            transform: Transform::from_xyz(50.0, 10.0, 10.0),
            ..Default::default()
        })
        .insert(PickableBundle::default())
        .insert(On::<Pointer<Click>>::send_event::<CheckChest>());
}

fn check_chest(
    mut check_event: EventReader<CheckChest>,
    player_query: Query<(&GlobalTransform, Entity), With<Player>>,
    mut chest_query: Query<(&mut Sprite, &GlobalTransform, Entity), (With<Chest>, Without<Player>)>,
    mut commands: Commands,
) {
    let (player_transform, player_entity) = player_query.get_single().unwrap();
    for chest in check_event.read() {
        let (mut sprite, chest_transform, chest_entity) = chest_query.get_mut(chest.chest).unwrap();
        if squared_distance(
            player_transform.translation(),
            chest_transform.translation(),
        ) <= 500.0
        {
            commands.insert_resource(ChestWindowState {
                left_inventory: player_entity,
                right_inventory: chest_entity,
                is_open: true,
            });

            sprite.color = Color::GREEN;
        } else {
            sprite.color = Color::RED;
        }
    }
}

fn handle_chest_inventory_window(
    mut contexts: EguiContexts,
    chest_state: Option<ResMut<ChestWindowState>>,
    mut player_inventory: Query<&mut Inventory, With<Player>>,
    mut chest_query: Query<&mut Inventory, (With<Chest>, Without<Player>)>,
    items_query: Query<(&Item, &ItemStack)>,
) {
    if let Some(mut chest_state) = chest_state {
        let ChestWindowState {
            left_inventory,
            right_inventory,
            ..
        } = chest_state.as_mut();
        let chest_inventory = chest_query.get_mut(*right_inventory).unwrap();
        let player_inventory = player_inventory.get_mut(*left_inventory).unwrap();
        egui::Window::new("Chest Inventory")
            .open(&mut chest_state.is_open)
            .show(contexts.ctx_mut(), |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    ui.vertical(|ui| {
                        for entity in player_inventory.map.iter().filter_map(|x| *x) {
                            if let Ok(item) = items_query.get(entity) {
                                show_item(item, ui, true);
                            }
                        }
                    });
                    ui.vertical(|ui| {
                        for entity in chest_inventory.map.iter().filter_map(|x| *x) {
                            if let Ok(item) = items_query.get(entity) {
                                show_item(item, ui, true);
                            }
                        }
                    });
                });
            });
    }
}
