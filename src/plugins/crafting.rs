use std::marker::PhantomData;

use self::macros::{create_items_map, create_workbench, item, item_kind};
use bevy::{
    ecs::{bundle::Bundle, component::Component, system::Resource},
    utils::HashMap,
};

pub struct CraftingPlugin;

#[derive(Component)]
pub struct Inventory {
    // inner: HashMap<>
}

// TODO: Separate into enum

#[derive(Component, Hash, PartialEq, Eq)]
pub enum Item {
    Single(ItemData),
    Multiple(Vec<ItemData>),
}

impl Default for Item {
    fn default() -> Self {
        Self::Single(Default::default())
    }
}

#[derive(Hash, PartialEq, Eq)]
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

#[derive(Default, Hash, PartialEq, Eq)]
pub enum ItemKind {
    Complex(ItemProperties),
    #[default]
    Primitive,
}

#[derive(Hash, PartialEq, Eq)]
pub struct ItemProperties {}

impl Default for ItemProperties {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Hash, PartialEq, Eq)]
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

pub type ItemsMap = HashMap<Item, Item>;

#[derive(Component)]
pub struct Workbench<T: WorkbenchTag> {
    workbench_tag: PhantomData<T>,
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
                        pub map: ItemsMap,
                    }

                    impl Workbench<[<$name Workbench>]> {
                        pub fn craft<'a>(map: &'a [<$name WorkbenchMap>], item: &'a Item) -> Option<&'a Item> {
                            map.map.get(item)
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
