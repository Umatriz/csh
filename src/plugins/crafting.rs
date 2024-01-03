use self::macros::{create_items_map, create_workbench, item, item_kind};
use bevy::{
    app::{Plugin, Startup, Update},
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::With,
        reflect::AppTypeRegistry,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    reflect::{std_traits::ReflectDefault, Reflect},
    utils::HashMap,
};
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Ui},
    quick::ResourceInspectorPlugin,
};
use paste::paste;
use std::{marker::PhantomData, sync::Arc};

use super::player::{Player, PlayerProperties};

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<WindowContext>()
            .register_type::<Inventory>()
            .register_type::<Option<Item>>()
            .register_type_data::<Option<Item>, ReflectDefault>()
            .register_type::<Item>()
            .register_type::<Vec<ItemData>>()
            .register_type_data::<Vec<ItemData>, ReflectDefault>()
            .register_type::<ItemData>()
            .register_type::<ItemKind>()
            .register_type::<ItemModifiers>()
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
    let map = ClassicalWorkbenchMap::new(&mut commands);
    commands.insert_resource(map);
}

#[derive(Resource, Default)]
struct AddItemWindow {
    item: Item,
}

fn add_item_window(
    mut contexts: EguiContexts,
    mut commands: Commands,
    window_context: Res<WindowContext>,
    mut add_item_window: ResMut<AddItemWindow>,
    type_registry: Res<AppTypeRegistry>,
    mut player_query: Query<&mut Inventory, With<Player>>,
) {
    if window_context.add_item_window {
        egui::Window::new("Add Item").show(contexts.ctx_mut(), |ui| {
            bevy_inspector_egui::reflect_inspector::ui_for_value(
                &mut add_item_window.item,
                ui,
                &type_registry.read(),
            );
            if ui.button("Add").clicked() {
                if let Ok(mut inventory) = player_query.get_single_mut() {
                    let id = commands.spawn(add_item_window.item.clone()).id();
                    inventory.add_single(id);
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
    pub input_item: Item,
}

fn craft_on_classical(
    mut event_message: EventReader<CraftMessage>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    workbench_query: Query<&Workbench<ClassicalWorkbench>>,
    workbench_map: Res<ClassicalWorkbenchMap>,
    input_items_query: Query<&Item>,
) {
    for event in event_message.read() {
        let mut player_inventory = player_query.get_single_mut().unwrap();
        let workbench = workbench_query.get_single().unwrap();

        if let Some(entity) =
            player_inventory.take_linear_item(&input_items_query, &event.input_item)
        {
            if let Ok(item) = input_items_query.get(entity) {
                if let Some(ent) = workbench.craft(&workbench_map, item) {
                    player_inventory.add_single(ent)
                } else {
                    println!("fail3")
                }
            } else {
                println!("fail2")
            }
        } else {
            println!("fail1")
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

fn show_item(item: &Item, ui: &mut Ui, enabled: bool) {
    let mut show_single = |item_data: &ItemData| {
        ui.add_enabled(enabled, |ui: &mut Ui| {
            ui.label(item_data.name.to_owned())
                .on_hover_text(format!("Kind: {:?}", item_data.kind))
                .on_hover_text(format!("Modifiers: {:#?}", item_data.modifiers))
        });
    };
    match item {
        Item::Single(item_data) => {
            show_single(item_data);
        }
        Item::Multiple(items_vec) => {
            for item_data in items_vec {
                show_single(item_data);
            }
        }
    }
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
    input_items_query: Query<&Item>,
) {
    if workbench_window_state.workbench_window {
        let inventory = player_query.get_single().unwrap();
        egui::Window::new("Workbench")
            .resizable(true)
            .show(contexts.ctx_mut(), |ui| {
                for (input, output) in crafts_map.map.iter() {
                    let enabled = inventory.search_item(&input_items_query, input).is_some();

                    ui.horizontal(|ui| {
                        show_item(input, ui, enabled);

                        ui.separator();

                        if let Ok(item) = input_items_query.get(*output) {
                            show_item(item, ui, enabled)
                        }

                        if ui
                            .add_enabled(enabled, egui::Button::new("Craft"))
                            .clicked()
                        {
                            craft_event_message.send(CraftMessage {
                                input_item: input.clone(),
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
    input_items_query: Query<&Item>,
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

#[derive(Component, Default, Reflect)]
#[reflect(Default)]
pub struct Inventory {
    /// [`Entity`] should refer to the [`Item::Single`]
    pub map: Vec<Option<Entity>>,
}

impl Inventory {
    pub fn new() -> Self {
        Self { map: Vec::new() }
    }

    pub fn take_linear(&mut self, entity: Entity) -> Option<Entity> {
        for item_ref in self.map.iter_mut().filter(|opt| opt.is_some()) {
            if item_ref == &mut Some(entity) {
                return item_ref.take();
            }
        }
        None
    }

    pub fn take_linear_item(&mut self, query: &Query<&Item>, item: &Item) -> Option<Entity> {
        for (id, item_ref) in self
            .map
            .iter()
            .enumerate()
            .filter_map(|(id, opt)| opt.map(|e| (id, e)))
            .filter_map(|(id, entity)| {
                let g = query.get(entity);
                g.ok().map(|i| (id, i))
            })
        {
            if item_ref == item {
                return self.map.get_mut(id).and_then(|item| item.take());
            }
        }
        None
    }

    // pub fn add(&mut self, item: Item) {
    //     match item {
    //         Item::Single(_) => self.add(item),
    //         Item::Multiple(_) => self.add_vec(item.into()),
    //     }
    // }

    pub fn take(&mut self, id: usize) -> Option<Entity> {
        self.map.get_mut(id).and_then(|opt| opt.take())
    }

    pub fn search(&self, item: Entity) -> Option<usize> {
        self.map.iter().filter_map(|i| *i).position(|i| i == item)
    }

    pub fn search_item(&self, query: &Query<&Item>, item: &Item) -> Option<usize> {
        self.map
            .iter()
            .filter_map(|i| *i)
            .filter_map(|entity| query.get(entity).ok())
            .position(|i| i == item)
    }

    pub fn add_single(&mut self, item: Entity) {
        self.map.push(Some(item))
    }

    pub fn add_vec(&mut self, vec: Vec<Entity>) {
        let mut new_vec = vec.into_iter().map(Some).collect::<Vec<_>>();
        self.map.append(&mut new_vec);
    }
}

#[derive(Bundle)]
pub struct ItemBundle {
    pub item: Item,
    pub enchantments: ItemEnchantments,
}

#[derive(Debug, Component, Clone, Hash, PartialEq, Eq, Reflect)]
#[reflect(Default)]
pub enum Item {
    Single(ItemData),
    Multiple(Vec<ItemData>),
}

impl From<Item> for Vec<Item> {
    fn from(value: Item) -> Self {
        match value {
            Item::Single(data) => vec![Item::Single(data)],
            Item::Multiple(data_vec) => data_vec.into_iter().map(Item::Single).collect(),
        }
    }
}

impl From<Vec<Item>> for Item {
    fn from(value: Vec<Item>) -> Self {
        let mut vec = vec![];
        value.into_iter().for_each(|item| match item {
            Item::Single(data) => vec.push(data),
            Item::Multiple(mut data_vec) => vec.append(&mut data_vec),
        });
        Item::Multiple(vec)
    }
}

impl Default for Item {
    fn default() -> Self {
        Self::Single(Default::default())
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

#[derive(Hash, Clone, PartialEq, Eq, Debug, Reflect)]
#[reflect(Default)]
pub struct ItemData {
    pub name: String,
    pub kind: ItemKind,
    pub modifiers: ItemModifiers,
}

impl Default for ItemData {
    fn default() -> Self {
        Self {
            name: "TestItem".to_string(),
            kind: ItemKind::Primitive,
            modifiers: Default::default(),
        }
    }
}

#[derive(Default, Clone, Hash, PartialEq, Eq, Debug, Reflect)]
#[reflect(Default)]
pub enum ItemKind {
    Complex(ItemProperties),
    #[default]
    Primitive,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug, Reflect)]
#[reflect(Default)]
pub struct ItemProperties {}

impl Default for ItemProperties {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Hash, Clone, PartialEq, Eq, Debug, Reflect)]
#[reflect(Default)]
pub struct ItemModifiers {
    pub amount: u8,
    pub level: u8,
}

impl Default for ItemModifiers {
    fn default() -> Self {
        Self {
            amount: 1,
            level: 1,
        }
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

pub type CraftsMap = HashMap<Item, Entity>;

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

create_workbench! {
    Classical
}

create_items_map! {
    ClassicalWorkbenchMap,
    item! {
        "1",
        item_kind!(primitive),
        amount = 1,
        level = 1
    } => item! {
        "2",
        item_kind!(complex {}),
        amount = 1,
        level = 1;

        "3",
        item_kind!(complex {}),
        amount = 1,
        level = 1
    }
}

mod macros {
    /// Creates `WorkbenchTag` and implements `craft` method for `Workbench`
    /// using this tag
    /// ```rust
    /// create_workbench! {
    ///     Classical
    /// }
    /// ```
    /// You must use this macro with `create_items_map!`
    /// ```rust
    /// create_workbench! {
    ///     Classical
    /// }
    /// create_items_map! {
    ///     ClassicalWorkbenchMap,
    ///     item! {
    ///         "ExampleItem1",
    ///         item_kind!(primitive),
    ///         amount = 1,
    ///         level = 1
    ///     } => item! {
    ///         "ExampleItem2",
    ///         item_kind!(complex {}),
    ///         amount = 1,
    ///         level = 1
    ///     }
    /// }
    /// ```
    macro_rules! create_workbench {
        (
            $($name:ident),*
        ) => {
            paste::paste! {
                $(
                    pub enum [<$name Workbench>] {}
                    impl WorkbenchTag for [<$name Workbench>] {}

                    #[derive(Resource)]
                    pub struct [<$name WorkbenchMap>] {
                        pub map: CraftsMap,
                    }

                    impl Workbench<[<$name Workbench>]> {
                        pub fn craft(&self, map: &[<$name WorkbenchMap>], item: &Item) -> Option<Entity> {
                            map.map.get(item).copied()
                        }
                    }
                )*
            }
        };
    }
    pub(crate) use create_workbench;

    /// Select the `ItemKind` uses in `item! macro
    /// Example:
    /// ```rust
    /// item_kind!(primitive) // ItemKind::Primitive
    /// item_kind!(complex {}) // ItemKind::Complex(ItemProperties {})
    /// ```
    macro_rules! item_kind {
        (primitive) => {
            ItemKind::Primitive
        };
        (complex {}) => {
            ItemKind::Complex(ItemProperties {})
        };
    }
    pub(crate) use item_kind;

    /// Create an `ItemBundle`
    /// Example:
    /// ```rust
    /// item! { "ExampleItem", item_kind!(primitive), amount = 1, level = 1 }
    /// ```
    macro_rules! item {
        (
            $name:literal,
            $kind:expr,
            amount = $amount:literal,
            level = $level:literal
        ) => {
            Item::Single(ItemData {
                name: $name.to_string(),
                kind: $kind,
                modifiers: ItemModifiers {
                    amount: $amount,
                    level: $level,
                },
            })
        };
        (
            $(
                $name:literal,
                $kind:expr,
                amount = $amount:literal,
                level = $level:literal
            );+
        ) => {
            Item::Multiple(vec![
                $(ItemData {
                    name: $name.to_string(),
                    kind: $kind,
                    modifiers: ItemModifiers {
                        amount: $amount,
                        level: $level,
                    },
                },)+
            ])
        };
    }
    pub(crate) use item;

    /// Create an `ItemsMap`.
    /// This macro just implements `Default` for given `WorkbenchMap`.
    /// You must use this macro with `create_workbench!` and `item!`.
    /// See `create_workbench!` docs for complex example.
    macro_rules! create_items_map {
        (
            $name:ty,
            $($in_item:expr => $out_item:expr)*
        ) => {
            paste! {
                #[derive(Component)]
                struct [< $name TagComponent >];

                impl $name {
                    fn new(commands: &mut Commands) -> Self {
                        Self {
                            map: HashMap::from([
                                $(($in_item, commands.spawn($out_item).insert([< $name TagComponent >]).id()))*
                            ]),
                        }
                    }
                }
            }
        };
    }
    pub(crate) use create_items_map;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_convert_test() {
        let single_vec: Vec<Item> = item! {
            "2",
            item_kind!(complex {}),
            amount = 1,
            level = 1;

            "3",
            item_kind!(complex {}),
            amount = 1,
            level = 1
        }
        .into();
        dbg!(single_vec);

        let multiple: Item = vec![
            item! {
                "2",
                item_kind!(complex {}),
                amount = 1,
                level = 1
            },
            item! {
                "3",
                item_kind!(complex {}),
                amount = 1,
                level = 1
            },
        ]
        .into();
        dbg!(multiple);
    }
}
