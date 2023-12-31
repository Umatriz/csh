use std::marker::PhantomData;

use self::macros::{create_items_map, create_workbench, item, item_kind};
use bevy::{
    app::{Plugin, Startup, Update},
    ecs::{
        component::Component,
        event::{Event, EventReader, EventWriter},
        query::With,
        system::{Commands, Query, Res, Resource},
    },
    reflect::{std_traits::ReflectDefault, Reflect},
    utils::HashMap,
};
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Ui},
    quick::ResourceInspectorPlugin,
};

use super::player::Player;

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<WindowContext>()
            .init_resource::<ClassicalWorkbenchMap>()
            .init_resource::<WindowContext>()
            .add_plugins(ResourceInspectorPlugin::<WindowContext>::default())
            .add_event::<CraftMessage>()
            .add_event::<WorkbenchWindowMessage>()
            .add_systems(Startup, spawn_test_workbench)
            .add_systems(Update, craft_on_classical)
            .add_systems(Update, (handle_workbench_window, handle_inventory_window));
    }
}

fn spawn_test_workbench(mut commands: Commands) {
    commands.spawn(Workbench::<ClassicalWorkbench>::new());
}

#[derive(Event)]
pub struct CraftMessage {
    pub input_item: Item,
}

fn craft_on_classical(
    mut event_message: EventReader<CraftMessage>,
    mut workbench_window_message: EventWriter<WorkbenchWindowMessage>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    workbench_query: Query<&Workbench<ClassicalWorkbench>>,
    workbench_map: Res<ClassicalWorkbenchMap>,
) {
    for event in event_message.read() {
        let mut player_inventory = player_query.get_single_mut().unwrap();
        let workbench = workbench_query.get_single().unwrap();
        if let Some(item) = player_inventory
            .search(event.input_item.clone())
            .and_then(|id| player_inventory.take(id))
            .and_then(|item| workbench.craft(&workbench_map, item))
        {
            player_inventory.add(item.clone())
        } else {
            workbench_window_message.send(WorkbenchWindowMessage {
                message: "Crafting error".to_string(),
            })
        }
    }
}

#[derive(Event)]
pub struct WorkbenchWindowMessage {
    pub message: String,
}

#[derive(Resource, Default, Reflect)]
#[reflect(Default)]
pub struct WindowContext {
    workbench_window: bool,
    inventory_window: bool,
}

fn handle_workbench_window(
    mut contexts: EguiContexts,
    workbench_window_state: Res<WindowContext>,
    mut workbench_window_message: EventReader<WorkbenchWindowMessage>,
    mut craft_event_message: EventWriter<CraftMessage>,
    crafts_map: Res<ClassicalWorkbenchMap>,
    player_query: Query<&Inventory, With<Player>>,
) {
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

    if workbench_window_state.workbench_window {
        let inventory = player_query.get_single().unwrap();
        egui::Window::new("Workbench")
            .resizable(true)
            .show(contexts.ctx_mut(), |ui| {
                for (input, output) in crafts_map.map.iter() {
                    let enabled = inventory.search(input.clone()).is_some();
                    ui.horizontal(|ui| {
                        show_item(input, ui, enabled);
                        ui.separator();
                        show_item(output, ui, enabled);
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

fn handle_inventory_window(mut contexts: EguiContexts, workbench_window_state: Res<WindowContext>) {
    if workbench_window_state.inventory_window {
        egui::Window::new("Inventory").show(contexts.ctx_mut(), |ui| {
            ui.label("text");
        });
    }
}

#[derive(Component, Default)]
pub struct Inventory {
    pub map: Vec<Option<Item>>,
}

impl Inventory {
    pub fn new() -> Self {
        Self { map: Vec::new() }
    }

    pub fn take(&mut self, id: usize) -> Option<Item> {
        self.map.get_mut(id).and_then(|opt| opt.take())
    }

    pub fn search(&self, item: Item) -> Option<usize> {
        self.map
            .iter()
            .filter_map(|i| i.clone())
            .position(|i| i == item)
    }

    pub fn add(&mut self, item: Item) {
        self.map.push(Some(item))
    }
}

#[derive(Component, Clone, Hash, PartialEq, Eq)]
pub enum Item {
    Single(ItemData),
    Multiple(Vec<ItemData>),
}

impl Default for Item {
    fn default() -> Self {
        Self::Single(Default::default())
    }
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
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

#[derive(Default, Clone, Hash, PartialEq, Eq, Debug)]
pub enum ItemKind {
    Complex(ItemProperties),
    #[default]
    Primitive,
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct ItemProperties {}

impl Default for ItemProperties {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct ItemModifiers {
    pub amount: u8,
    pub level: u8,
}

impl Default for ItemModifiers {
    fn default() -> Self {
        Item::Multiple(vec![ItemData {
            name: "Test".to_string(),
            kind: ItemKind::Complex(ItemProperties {}),
            modifiers: ItemModifiers {
                amount: 1,
                level: 1,
            },
        }]);

        Self {
            amount: 1,
            level: 1,
        }
    }
}

pub trait WorkbenchTag {}

pub type CraftsMap = HashMap<Item, Item>;

#[derive(Component)]
pub struct Workbench<T: WorkbenchTag> {
    workbench_tag: PhantomData<T>,
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
                        pub fn craft<'a>(&self, map: &'a [<$name WorkbenchMap>], item: Item) -> Option<&'a Item> {
                            map.map.get(&item)
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
            impl Default for $name {
                fn default() -> Self {
                    Self {
                        map: HashMap::from([
                            $(($in_item, $out_item))*
                        ]),
                    }
                }
            }
        };
    }
    pub(crate) use create_items_map;
}
