use bevy::{
    app::{Plugin, Startup, Update},
    core::Name,
    ecs::{
        change_detection::DetectChangesMut,
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        query::{With, Without},
        schedule::{common_conditions::in_state, IntoSystemConfigs},
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

use crate::{item, item_kind, layout, utils::squared_distance, GameState};

use super::{
    crafting::{
        logic::{Inventory, Item, ItemBundle, ItemStack},
        show_item,
    },
    player::Player,
};

pub struct ChestPlugin;

impl Plugin for ChestPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<CheckChest>()
            .register_type::<ChestWindowState>()
            .init_resource::<MoveStack>()
            .add_plugins(ResourceInspectorPlugin::<ChestWindowState>::default())
            .add_systems(Startup, spawn_chest)
            .add_systems(
                Update,
                (
                    check_chest,
                    handle_chest_inventory_window.after(check_chest),
                )
                    .run_if(in_state(GameState::Game)),
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
    let item = item! { "ExampleItem1", item_kind!(primitive), amount = 1, level = 1 };
    inventory.add_combine(&mut commands, &mut items_query, vec![item.as_tuple()]);
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
    let item = item! { "ExampleItem2", item_kind!(primitive), amount = 1, level = 1 };
    inventory.add_combine(&mut commands, &mut items_query, vec![item.as_tuple()]);
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

#[derive(Resource, Debug, PartialEq)]
pub struct MoveStack(u8);

impl Default for MoveStack {
    fn default() -> Self {
        Self(1)
    }
}

fn handle_chest_inventory_window(
    mut commands: Commands,
    mut contexts: EguiContexts,
    chest_state: Option<ResMut<ChestWindowState>>,
    mut player_inventory: Query<&mut Inventory, With<Player>>,
    mut chest_query: Query<&mut Inventory, (With<Chest>, Without<Player>)>,
    mut items_query: Query<(&mut Item, &mut ItemStack)>,
    mut move_stack: ResMut<MoveStack>,
) {
    if let Some(mut chest_state) = chest_state {
        let ChestWindowState {
            left_inventory,
            right_inventory,
            ..
        } = chest_state.as_mut();
        let mut chest_inventory = chest_query.get_mut(*right_inventory).unwrap();
        let mut player_inventory = player_inventory.get_mut(*left_inventory).unwrap();

        egui::Window::new("Chest Inventory")
            .open(&mut chest_state.is_open)
            .show(contexts.ctx_mut(), |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    ui.vertical(|ui| {
                        for entity in player_inventory.map.clone().into_iter().flatten() {
                            if let Ok((item, mut stack)) = items_query.get_mut(entity) {
                                if stack.0 == 0 {
                                    continue;
                                }

                                show_item((&item, &stack), ui, true);

                                let max = stack.0;

                                let move_button = ui.button("->");

                                if move_button.clicked()
                                    && player_inventory.take_linear(entity).is_some()
                                {
                                    stack.0 -= move_stack.0;
                                    player_inventory.add_single(entity);

                                    let it = &item.clone();
                                    let stack = &ItemStack(move_stack.0);
                                    let items = vec![(it, stack)];

                                    chest_inventory.add_combine(
                                        &mut commands,
                                        &mut items_query,
                                        items,
                                    );

                                    move_stack.set_if_neq(MoveStack::default());
                                }

                                move_button.context_menu(|ui| {
                                    ui.label("Menu!");
                                    ui.add(egui::Slider::new(&mut move_stack.as_mut().0, 1..=max));
                                });
                            }
                        }
                    });
                    ui.vertical(|ui| {
                        for entity in chest_inventory.map.clone().iter().filter_map(|x| *x) {
                            if let Ok((item, mut stack)) = items_query.get_mut(entity) {
                                if stack.0 == 0 {
                                    continue;
                                }

                                show_item((&item, &stack), ui, true);

                                let max = stack.0;

                                let move_button = ui.button("<-");

                                if move_button.clicked()
                                    && chest_inventory.take_linear(entity).is_some()
                                {
                                    stack.0 -= move_stack.0;
                                    chest_inventory.add_single(entity);

                                    let it = &item.clone();
                                    let stack = &ItemStack(move_stack.0);
                                    let items = vec![(it, stack)];

                                    player_inventory.add_combine(
                                        &mut commands,
                                        &mut items_query,
                                        items,
                                    );

                                    move_stack.set_if_neq(MoveStack::default());
                                }

                                move_button.context_menu(|ui| {
                                    ui.label("Menu!");
                                    ui.add(egui::Slider::new(&mut move_stack.as_mut().0, 1..=max));
                                });
                            }
                        }
                    });
                });
            });
    }
}
