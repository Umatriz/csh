use bevy::{
    app::{Plugin, Startup, Update},
    ecs::{
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::With,
        reflect::AppTypeRegistry,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    log::{error, warn},
};
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Ui},
};
use bevy_replicon::{network_event::client_event::FromClient, server::has_authority};
use std::marker::PhantomData;

use crate::{plugins::player::Player, LocalPlayer, WindowContext};

use super::logic::{
    ClassicalWorkbench, ClassicalWorkbenchMap, Craft, Inventory, Item, ItemBundle, ItemEvent,
    ItemEventKind, ItemStack, ItemsLayout, Layout, SecondWorkbench, SecondWorkbenchMap, Workbench,
    WorkbenchMap, WorkbenchTag,
};

pub struct WindowSystemsPlugin;

impl Plugin for WindowSystemsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<AddItemWindow>()
            .add_event::<CraftMessage<ClassicalWorkbench>>()
            .add_event::<CraftMessage<SecondWorkbench>>()
            .add_systems(Startup, spawn_test_workbench)
            .add_systems(
                Update,
                (
                    craft::<ClassicalWorkbench, ClassicalWorkbenchMap>,
                    craft::<SecondWorkbench, SecondWorkbenchMap>,
                ),
            )
            .add_systems(
                Update,
                (add_item_window, add_item_event.run_if(has_authority())),
            )
            .init_resource::<SecondWorkbenchMap>()
            .init_resource::<ClassicalWorkbenchMap>()
            .add_systems(
                Update,
                (
                    handle_workbench_window::<ClassicalWorkbench, ClassicalWorkbenchMap>,
                    handle_workbench_window::<SecondWorkbench, SecondWorkbenchMap>,
                    handle_inventory_window,
                    handle_enchantment_window,
                ),
            );
    }
}

fn spawn_test_workbench(mut commands: Commands) {
    commands.spawn(Workbench::<ClassicalWorkbench>::new());
    commands.spawn(Workbench::<SecondWorkbench>::new());
    // commands.spawn(EnchantingTable);
}

#[derive(Resource, Default)]
struct AddItemWindow {
    item: Item,
    stack: ItemStack,
    selected_player: usize,
}

fn add_item_window(
    mut contexts: EguiContexts,
    mut window_context: ResMut<WindowContext>,
    mut add_item_window: ResMut<AddItemWindow>,
    type_registry: Res<AppTypeRegistry>,

    mut add_item_events: EventWriter<ItemEvent>,
) {
    egui::Window::new("Add Item")
        .open(&mut window_context.add_item_window)
        .show(contexts.ctx_mut(), |ui| {
            ui.label("Item:");
            bevy_inspector_egui::reflect_inspector::ui_for_value(
                &mut add_item_window.item,
                ui,
                &type_registry.read(),
            );
            ui.label("Stack:");
            bevy_inspector_egui::reflect_inspector::ui_for_value(
                &mut add_item_window.stack,
                ui,
                &type_registry.read(),
            );
            if ui.button("Add").clicked() {
                add_item_events.send(ItemEvent {
                    kind: ItemEventKind::Add,
                    inventory: None,
                    item: ItemBundle {
                        item: add_item_window.item.clone(),
                        stack: add_item_window.stack.clone(),
                    },
                })
            }
            if ui.button("Clear Inventory").clicked() {
                // if let Ok(mut inventory) = player_query.get_single_mut() {
                //     inventory.map = vec![]
                // } else {
                //     println!("err")
                // }
            }
        });
}

fn add_item_event(
    mut commands: Commands,
    mut add_item_events: EventReader<FromClient<ItemEvent>>,
    mut player_query: Query<(&mut Inventory, &Player)>,
    mut items_query: Query<(&mut Item, &mut ItemStack)>,
) {
    for FromClient { client_id, event } in add_item_events.read() {
        for (mut inventory, player) in player_query.iter_mut() {
            if *client_id == player.0 {
                match event.kind {
                    ItemEventKind::Add => inventory.add_combine(
                        &mut commands,
                        &mut items_query,
                        Layout(vec![event.item.clone()]),
                    ),
                    ItemEventKind::Remove => warn!("unimplemented"),
                }
            }
        }
    }
}

#[derive(Event)]
pub struct CraftMessage<W: WorkbenchTag> {
    pub input: ItemsLayout,
    pub output: ItemsLayout,
    _marker: PhantomData<W>,
}

fn craft<W: WorkbenchTag, M: WorkbenchMap + Resource>(
    mut commands: Commands,
    mut event_message: EventReader<CraftMessage<W>>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    workbench_query: Query<&Workbench<W>>,
    workbench_map: Res<M>,
    mut items_query: Query<(&mut Item, &mut ItemStack)>,
) {
    for event in event_message.read() {
        let CraftMessage { input, .. } = event;
        let mut player_inventory = player_query.get_single_mut().unwrap();
        let workbench = workbench_query.get_single().unwrap();

        if let Some(layout) =
            player_inventory.take_satisfying_layout(&items_query.to_readonly(), input)
        {
            for (inp, out_entity) in input.0.iter().zip(layout.into_iter()) {
                if let Ok((_, mut out_stack)) = items_query.get_mut(out_entity) {
                    out_stack.0 -= inp.stack.0;
                    player_inventory.add_single(out_entity);
                }
            }

            if let Some(layout) = workbench.craft(workbench_map.map(), input) {
                player_inventory.add_combine(&mut commands, &mut items_query, layout);
            } else {
                error!("Crafting failed on stage: 2")
            }
        } else {
            error!("Crafting failed on stage: 1")
        }
    }
}

pub fn show_item(item_bundle: (&Item, &ItemStack), ui: &mut Ui, enabled: bool) {
    ui.add_enabled(enabled, |ui: &mut Ui| {
        ui.horizontal(|ui| {
            ui.label(item_bundle.0.name.to_owned())
                .on_hover_text(format!("Kind: {:?}", item_bundle.0.kind))
                .on_hover_text(format!("Level: {:?}", item_bundle.0.level))
                .on_hover_text(format!("Stack: {:#?}", item_bundle.1 .0));
            ui.label(format!("{:?}", item_bundle.0.kind));
            ui.label(format!("{:?}", item_bundle.1 .0))
                .on_hover_text("Items in the stack")
        })
        .response
    });
}

fn show_craft_layout(layout: &ItemsLayout, ui: &mut Ui, enabled: bool) {
    layout.get().iter().for_each(|item| {
        let ItemBundle { item, stack } = item;
        show_item((item, stack), ui, enabled);
    });
}

fn handle_enchantment_window(
    _contexts: EguiContexts,
    _window_context: Res<WindowContext>,
    _player_query: Query<&Inventory, With<Player>>,
) {
}

fn handle_workbench_window<W: WorkbenchTag, M: WorkbenchMap + Resource>(
    mut contexts: EguiContexts,
    mut workbench_window_state: ResMut<WindowContext>,
    mut craft_event_message: EventWriter<CraftMessage<W>>,
    crafts_map: Res<M>,
    player_query: Query<(&Inventory, &Player)>,
    items_query: Query<(&Item, &ItemStack)>,
    local_player: Option<Res<LocalPlayer>>,
) {
    egui::Window::new(crafts_map.name())
        .open(&mut workbench_window_state.workbench_window)
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            if let Some(local_player) = local_player {
                for (inventory, player) in player_query.iter() {
                    if player.0 == local_player.0 {
                        for (input, output) in crafts_map.map().iter() {
                            let enabled =
                                inventory.search_satisfying(&items_query, input).is_some();

                            ui.horizontal(|ui| {
                                show_craft_layout(input, ui, enabled);

                                ui.separator();

                                show_craft_layout(output, ui, enabled);

                                if ui
                                    .add_enabled(enabled, egui::Button::new("Craft"))
                                    .clicked()
                                {
                                    craft_event_message.send(CraftMessage {
                                        input: input.clone(),
                                        output: output.clone(),
                                        _marker: PhantomData,
                                    })
                                }
                            });
                        }
                    }
                }
            } else {
                ui.label("No players");
            }
        });
}

fn handle_inventory_window(
    mut contexts: EguiContexts,
    mut workbench_window_state: ResMut<WindowContext>,
    player_query: Query<(&Inventory, &Player)>,
    input_items_query: Query<(&Item, &ItemStack)>,
) {
    egui::Window::new("Inventory")
        .open(&mut workbench_window_state.inventory_window)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                for (inventory, player) in player_query.iter() {
                    ui.vertical(|ui| {
                        ui.label(format!("{}", player.0));
                        for entity in inventory.map.iter().filter_map(|x| *x) {
                            if let Ok(item) = input_items_query.get(entity) {
                                show_item(item, ui, true);
                            }
                        }
                    });
                    ui.separator();
                }
            });
        });
}
