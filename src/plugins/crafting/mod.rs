use bevy::{
    app::{Plugin, Startup, Update},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::With,
        reflect::AppTypeRegistry,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    log::error,
    reflect::{std_traits::ReflectDefault, Reflect},
    utils::HashMap,
};
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Ui},
    quick::ResourceInspectorPlugin,
};
use macros::{create_items_map, create_workbench, item, item_kind, layout};
use std::{marker::PhantomData, sync::Arc};

use self::logic::{
    Inventory, Item, ItemBundle, ItemKind, ItemProperties, ItemStack, ItemsLayout, Layout,
};

use super::player::{Player, PlayerProperties};

pub mod logic;
mod macros;

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<WindowContext>()
            .register_type::<Inventory>()
            .register_type::<Option<Item>>()
            .register_type_data::<Option<Item>, ReflectDefault>()
            .register_type::<Item>()
            .register_type::<Vec<Item>>()
            .register_type_data::<Vec<Item>, ReflectDefault>()
            .register_type::<Item>()
            .register_type::<ItemKind>()
            .register_type::<ItemStack>()
            .register_type::<ItemProperties>()
            .init_resource::<WindowContext>()
            .init_resource::<AddItemWindow>()
            .add_event::<CraftMessage>()
            .add_plugins(ResourceInspectorPlugin::<WindowContext>::default())
            .add_systems(Startup, spawn_test_workbench)
            .add_systems(Startup, add_resources)
            .add_systems(Update, craft_on_classical)
            .add_systems(Update, add_item_window)
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

fn add_resources(mut commands: Commands) {
    let map = ClassicalWorkbenchMap::new();
    commands.insert_resource(map);
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
                        &Layout(vec![ItemBundle {
                            item: add_item_window.item.clone(),
                            stack: add_item_window.stack.clone(),
                        }]),
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

fn spawn_test_workbench(mut commands: Commands) {
    commands.spawn(Workbench::<ClassicalWorkbench>::new());
    commands.spawn(EnchantingTable);
}

pub fn enchant(
    commands: &mut Commands,
    enchantment_table: &EnchantingTable,
    player_entity: Entity,
    player_properties: &mut PlayerProperties,
    item_properties: &mut ItemProperties,
    item_enchantments: &mut ItemEnchantments,
    enchantment: Arc<dyn Enchantment>,
) {
    enchantment_table.enchant(item_enchantments, enchantment);
    item_enchantments.apply_unapplied(player_properties, item_properties, commands, player_entity);
}

#[derive(Event)]
pub struct CraftMessage {
    pub input: ItemsLayout,
    pub output: ItemsLayout,
}

fn craft_on_classical(
    mut commands: Commands,
    mut event_message: EventReader<CraftMessage>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    workbench_query: Query<&Workbench<ClassicalWorkbench>>,
    workbench_map: Res<ClassicalWorkbenchMap>,
    mut items_query: Query<(&mut Item, &mut ItemStack)>,
) {
    for event in event_message.read() {
        let CraftMessage { input, output } = event;
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

            if let Some(layout) = workbench.craft(&workbench_map.map, input) {
                player_inventory.add_combine(&mut commands, &mut items_query, &layout);
            } else {
                error!("Crafting failed on stage: 2")
            }
        } else {
            error!("Crafting failed on stage: 1")
        }
    }
}

#[derive(Resource, Reflect)]
#[reflect(Default)]
pub struct WindowContext {
    workbench_window: bool,
    inventory_window: bool,
    enchantment_window: bool,
    add_item_window: bool,
}

impl Default for WindowContext {
    fn default() -> Self {
        Self {
            workbench_window: true,
            inventory_window: true,
            enchantment_window: true,
            add_item_window: true,
        }
    }
}

fn show_item(item_bundle: (&Item, &ItemStack), ui: &mut Ui, enabled: bool) {
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
    mut contexts: EguiContexts,
    window_context: Res<WindowContext>,
    player_query: Query<&Inventory, With<Player>>,
) {
}

fn handle_workbench_window(
    mut contexts: EguiContexts,
    workbench_window_state: Res<WindowContext>,
    mut craft_event_message: EventWriter<CraftMessage>,
    crafts_map: Res<ClassicalWorkbenchMap>,
    player_query: Query<&Inventory, With<Player>>,
    items_query: Query<(&Item, &ItemStack)>,
) {
    if workbench_window_state.workbench_window {
        let inventory = player_query.get_single().unwrap();
        egui::Window::new("Workbench")
            .resizable(true)
            .show(contexts.ctx_mut(), |ui| {
                for (input, output) in crafts_map.map.iter() {
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

#[derive(Component)]
pub struct ItemEnchantments(Vec<EnchantmentWithState>);

struct EnchantmentWithState {
    enchantment: Arc<dyn Enchantment>,
    state: bool,
}

impl ItemEnchantments {
    pub fn add(&mut self, enchantment: Arc<dyn Enchantment>) {
        self.0.push(EnchantmentWithState {
            enchantment,
            state: false,
        })
    }

    pub fn apply_unapplied(
        &mut self,
        player_properties: &mut PlayerProperties,
        item_properties: &mut ItemProperties,
        commands: &mut Commands,
        player_entity: Entity,
    ) {
        self.0.iter_mut().filter(|i| !i.state).for_each(|item| {
            item.enchantment.modify_item_properties(item_properties);
            item.enchantment.modify_player_properties(player_properties);
            item.enchantment.modify_world(commands, player_entity);

            item.state = true
        });
    }
}

// Just make sure that we don't add something that not allows us to construct a trait-object
const _: Option<Box<dyn Enchantment>> = None;

/// It represents additional modifiers for [`ItemProperties`] and [`ItemProperties`].
/// Also you can access [`Commands`] and player's [`Entity`]
/// using [`Enchantment::modify_world`] method
///
/// Methods in this trait are only called once when this `Enchantment` is applied
pub trait Enchantment: Reflect {
    fn modify_player_properties(&self, _properties: &mut PlayerProperties) {}
    fn modify_item_properties(&self, _properties: &mut ItemProperties) {}
    fn modify_world(&self, _commands: &mut Commands, _player_entity: Entity) {}
}

#[derive(Component)]
pub struct EnchantingTable;

impl EnchantingTable {
    pub fn enchant(
        &self,
        item_enchantments: &mut ItemEnchantments,
        enchantment: Arc<dyn Enchantment>,
    ) {
        item_enchantments.add(enchantment)
    }
}

#[derive(Reflect)]
struct Power;

#[derive(Component)]
struct PowerTag;

impl Enchantment for Power {
    fn modify_world(&self, commands: &mut Commands, player_entity: Entity) {
        if let Some(mut player) = commands.get_entity(player_entity) {
            player.insert(PowerTag);
        }
    }
}

pub trait WorkbenchTag {}

pub type CraftsMap = HashMap<ItemsLayout, ItemsLayout>;

#[derive(Component)]
pub struct Workbench<T: WorkbenchTag> {
    workbench_tag: PhantomData<T>,
}

impl<T: WorkbenchTag> Default for Workbench<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: WorkbenchTag> Workbench<T> {
    pub fn new() -> Self {
        Self {
            workbench_tag: PhantomData,
        }
    }
}

pub trait Craft {
    fn craft(&self, map: &CraftsMap, layout: &ItemsLayout) -> Option<ItemsLayout> {
        dbg!(&layout);
        map.get(layout).cloned()
    }
}

impl<T: WorkbenchTag> Craft for Workbench<T> {}

create_workbench! {
    Classical
}

create_items_map! {
    ClassicalWorkbenchMap,

    item! { "1", item_kind!(primitive), amount = 1, level = 1 }
    =>
    item! { "2", item_kind!(primitive), amount = 1, level = 1 },
    item! { "1", item_kind!(primitive), amount = 1, level = 1 };

    item! { "1", item_kind!(primitive), amount = 1, level = 1 },
    item! { "2", item_kind!(primitive), amount = 1, level = 1 }
    =>
    item! { "3", item_kind!(primitive), amount = 1, level = 1 },
    item! { "1", item_kind!(primitive), amount = 1, level = 1 },
    item! { "2", item_kind!(primitive), amount = 1, level = 1 };

    item! { "3", item_kind!(primitive), amount = 1, level = 1 }
    =>
    item! { "1", item_kind!(primitive), amount = 1, level = 1 };

    item! { "3", item_kind!(primitive), amount = 2, level = 1 }
    =>
    item! { "4", item_kind!(primitive), amount = 5, level = 1 }
}
