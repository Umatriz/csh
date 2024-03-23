use bevy::{
    app::{Plugin, Update},
    asset::{Assets, Handle},
    ecs::{
        event::{Event, EventReader, EventWriter},
        query::With,
        reflect::AppTypeRegistry,
        schedule::{common_conditions::in_state, IntoSystemConfigs},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    log::{error, warn},
    reflect::TypePath,
};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Ui},
};
use bevy_replicon::{
    core::common_conditions::has_authority, network_event::client_event::FromClient,
};

use crate::{
    debugging::{show_window, InspectorWindowsAppExt},
    plugins::{network::LocalPlayerId, player::Player},
    GameState, InspectorWindows,
};

use super::logic::{
    Inventory, Item, ItemBundle, ItemEvent, ItemEventKind, ItemStack, ItemsLayout, Workbench,
};

pub struct WindowSystemsPlugin;

impl Plugin for WindowSystemsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<AddItemWindow>()
            .add_event::<CraftMessage>()
            // .add_systems(Startup, spawn_test_workbench)
            .register_window::<AddItemWindow>()
            .register_window::<InventoryWindow>()
            .add_systems(Update, craft.run_if(in_state(GameState::Game)))
            .add_systems(
                Update,
                (add_item_window, add_item_event.run_if(has_authority))
                    .run_if(in_state(GameState::Game)),
            )
            .add_systems(
                Update,
                (
                    handle_workbench_window,
                    handle_inventory_window,
                    handle_enchantment_window,
                ),
            );
    }
}

#[derive(AssetCollection, Resource)]
pub struct WorkbenchesCollection {
    #[asset(path = "workbenches", collection(typed))]
    workbenches: Vec<Handle<Workbench>>,
}

#[derive(AssetCollection, Resource)]
pub struct ItemsCollection {
    #[asset(path = "items", collection(typed))]
    items: Vec<Handle<Item>>,
}

// fn spawn_test_workbench(mut commands: Commands) {
//     commands.spawn(Workbench::<ClassicalWorkbench>::new());
//     commands.spawn(Workbench::<SecondWorkbench>::new());
//     // commands.spawn(EnchantingTable);
// }

#[derive(Resource, Default, TypePath)]
struct AddItemWindow {
    stack: ItemStack,
    selected_item: usize,
}

fn add_item_window(
    mut contexts: EguiContexts,
    mut window_context: ResMut<InspectorWindows>,
    mut add_item_window: ResMut<AddItemWindow>,
    items: Res<ItemsCollection>,
    items_asset: Res<Assets<Item>>,
    type_registry: Res<AppTypeRegistry>,
    mut add_item_events: EventWriter<ItemEvent>,
) {
    show_window::<AddItemWindow, _>(window_context.as_mut(), contexts.ctx_mut(), |ui| {
        ui.label("Item:");
        egui::ComboBox::from_label("Select Item")
            .selected_text(format!("{}", add_item_window.selected_item))
            .show_ui(ui, |ui| {
                for i in 0..items.items.len() {
                    let value = ui.selectable_value(
                        &mut &items.items[i],
                        &items.items[add_item_window.selected_item],
                        format!("{:?}", items_asset.get(&items.items[i]).unwrap()),
                    );
                    if value.clicked() {
                        add_item_window.selected_item = i;
                    }
                }
            });
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
                    item: items_asset
                        .get(&items.items[add_item_window.selected_item])
                        .unwrap()
                        .clone(),
                    stack: add_item_window.stack.clone(),
                },
            });
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
                        vec![event.item.as_tuple()],
                    ),
                    ItemEventKind::Remove => warn!("unimplemented"),
                }
            }
        }
    }
}

#[derive(Event)]
pub struct CraftMessage {
    pub input: ItemsLayout,
    pub output: ItemsLayout,
}

fn craft(
    mut commands: Commands,
    mut event_message: EventReader<CraftMessage>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    workbenches: Res<WorkbenchesCollection>,
    item_assets: Res<Assets<Item>>,
    workbench_assets: Res<Assets<Workbench>>,
    mut items_query: Query<(&mut Item, &mut ItemStack)>,
) {
    for event in event_message.read() {
        let CraftMessage { input, .. } = event;
        let mut player_inventory = player_query.get_single_mut().unwrap();

        if let Some(layout) =
            player_inventory.take_satisfying_layout(&items_query.to_readonly(), input)
        {
            for (inp, out_entity) in input.0.iter().zip(layout.into_iter()) {
                if let Ok((_, mut out_stack)) = items_query.get_mut(out_entity) {
                    out_stack.0 -= inp.stack.0;
                    player_inventory.add_single(out_entity);
                }
            }

            for workbench in workbenches
                .workbenches
                .iter()
                .filter_map(|h| workbench_assets.get(h))
            {
                if let Some(layout) = workbench.craft(&item_assets, input) {
                    player_inventory.add_combine(&mut commands, &mut items_query, layout);
                } else {
                    error!("Crafting failed on stage: 2")
                }
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
    _window_context: Res<InspectorWindows>,
    _player_query: Query<&Inventory, With<Player>>,
) {
}

fn handle_workbench_window(
    _contexts: EguiContexts,
    _workbench_window_state: ResMut<InspectorWindows>,
    _craft_event_message: EventWriter<CraftMessage>,
    _player_query: Query<(&Inventory, &Player)>,
    _items_query: Query<(&Item, &ItemStack)>,
    _local_player: Option<Res<LocalPlayerId>>,
) {
    // egui::Window::new(crafts_map.name())
    //     .open(&mut workbench_window_state.workbench_window)
    //     .resizable(true)
    //     .show(contexts.ctx_mut(), |ui| {
    //         if let Some(local_player) = local_player {
    //             for (inventory, player) in player_query.iter() {
    //                 if player.0 == local_player.0 {
    //                     for (input, output) in crafts_map.map().iter() {
    //                         let enabled =
    //                             inventory.search_satisfying(&items_query, input).is_some();

    //                         ui.horizontal(|ui| {
    //                             show_craft_layout(input, ui, enabled);

    //                             ui.separator();

    //                             show_craft_layout(output, ui, enabled);

    //                             if ui
    //                                 .add_enabled(enabled, egui::Button::new("Craft"))
    //                                 .clicked()
    //                             {
    //                                 craft_event_message.send(CraftMessage {
    //                                     input: input.clone(),
    //                                     output: output.clone(),
    //                                     _marker: PhantomData,
    //                                 })
    //                             }
    //                         });
    //                     }
    //                 }
    //             }
    //         } else {
    //             ui.label("No players");
    //         }
    //     });
}

#[derive(TypePath)]
enum InventoryWindow {}

fn handle_inventory_window(
    mut contexts: EguiContexts,
    mut inspector_windows: ResMut<InspectorWindows>,
    player_query: Query<(&Inventory, &Player)>,
    input_items_query: Query<(&Item, &ItemStack)>,
) {
    show_window::<InventoryWindow, _>(inspector_windows.as_mut(), contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            for (inventory, player) in player_query.iter() {
                ui.vertical(|ui| {
                    ui.label(format!("{:?}", player.0));
                    for entity in inventory.map.iter().filter_map(|x| *x) {
                        if let Ok(item) = input_items_query.get(entity) {
                            show_item(item, ui, true);
                        }
                    }
                });
                ui.separator();
            }
        })
    });
}
