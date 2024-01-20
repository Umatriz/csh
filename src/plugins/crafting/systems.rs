use bevy::{
    app::{Plugin, Startup, Update},
    ecs::{
        event::{Event, EventReader, EventWriter},
        query::With,
        reflect::AppTypeRegistry,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    log::error,
    reflect::{std_traits::ReflectDefault, Reflect},
};
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Ui},
    quick::ResourceInspectorPlugin,
};
use std::marker::PhantomData;

use crate::{layout, plugins::player::Player};

use super::logic::{
    ClassicalWorkbench, ClassicalWorkbenchMap, Craft, Inventory, Item, ItemBundle, ItemStack,
    ItemsLayout, SecondWorkbench, SecondWorkbenchMap, Workbench, WorkbenchMap, WorkbenchTag,
};

pub struct WindowSystemsPlugin;

impl Plugin for WindowSystemsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<WindowContext>()
            .init_resource::<WindowContext>()
            .init_resource::<AddItemWindow>()
            .add_event::<CraftMessage<ClassicalWorkbench>>()
            .add_event::<CraftMessage<SecondWorkbench>>()
            .add_plugins(ResourceInspectorPlugin::<WindowContext>::default())
            .add_systems(Startup, spawn_test_workbench)
            .add_systems(
                Update,
                (
                    craft::<ClassicalWorkbench, ClassicalWorkbenchMap>,
                    craft::<SecondWorkbench, SecondWorkbenchMap>,
                ),
            )
            .add_systems(Update, add_item_window)
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
}

fn add_item_window(
    mut contexts: EguiContexts,
    mut commands: Commands,
    window_context: Res<WindowContext>,
    mut add_item_window: ResMut<AddItemWindow>,
    type_registry: Res<AppTypeRegistry>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    mut items_query: Query<(&mut Item, &mut ItemStack)>,
) {
    if window_context.add_item_window {
        egui::Window::new("Add Item").show(contexts.ctx_mut(), |ui| {
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
                if let Ok(mut inventory) = player_query.get_single_mut() {
                    inventory.add_combine(
                        &mut commands,
                        &mut items_query,
                        layout![ItemBundle {
                            item: add_item_window.item.clone(),
                            stack: add_item_window.stack.clone(),
                        }],
                    );
                } else {
                    println!("err")
                }
            }
            if ui.button("Clear Inventory").clicked() {
                if let Ok(mut inventory) = player_query.get_single_mut() {
                    inventory.map = vec![]
                } else {
                    println!("err")
                }
            }
        });
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

#[derive(Resource, Default, Reflect)]
#[reflect(Default)]
pub struct WindowContext {
    workbench_window: bool,
    inventory_window: bool,
    enchantment_window: bool,
    add_item_window: bool,
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
    workbench_window_state: Res<WindowContext>,
    mut craft_event_message: EventWriter<CraftMessage<W>>,
    crafts_map: Res<M>,
    player_query: Query<&Inventory, With<Player>>,
    items_query: Query<(&Item, &ItemStack)>,
) {
    if workbench_window_state.workbench_window {
        let inventory = player_query.get_single().unwrap();
        egui::Window::new(crafts_map.name())
            .resizable(true)
            .show(contexts.ctx_mut(), |ui| {
                for (input, output) in crafts_map.map().iter() {
                    let enabled = inventory.search_satisfying(&items_query, input).is_some();

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
            });
    }
}

fn handle_inventory_window(
    mut contexts: EguiContexts,
    workbench_window_state: Res<WindowContext>,
    player_query: Query<&Inventory, With<Player>>,
    input_items_query: Query<(&Item, &ItemStack)>,
) {
    if workbench_window_state.inventory_window {
        let inventory = player_query.get_single().unwrap();
        egui::Window::new("Inventory").show(contexts.ctx_mut(), |ui| {
            for entity in inventory.map.iter().filter_map(|x| *x) {
                if let Ok(item) = input_items_query.get(entity) {
                    show_item(item, ui, true);
                }
            }
        });
    }
}
