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
        level = $level:literal
    ) => {
        Item {
            name: $name.to_string(),
            kind: $kind,
            level: $level,
        }
    };
    (
        $name:literal,
        $kind:expr,
        amount = $amount:literal,
        level = $level:literal
    ) => {
        ItemBundle {
            item: Item {
                name: $name.to_string(),
                kind: $kind,
                level: $level,
            },
            stack: ItemStack($amount),
        }
    };
}
pub(crate) use item;

macro_rules! layout {
    (
        $($item:expr),* $(,)?
    ) => {
        Layout(vec![$($item,)*])
    };
}
pub(crate) use layout;

/// Create an `ItemsMap`.
/// This macro just implements `Default` for given `WorkbenchMap`.
/// You must use this macro with `create_workbench!` and `item!`.
/// See `create_workbench!` docs for complex example.
macro_rules! create_items_map {
    (
        $name:ty,
        $($($in_item:expr),* => $($out_item:expr),*);*
    ) => {
        impl $name {
            fn new() -> Self {
                Self {
                    map: HashMap::from([
                        $( (layout![$($in_item,)*], layout![$( $out_item, )*]),)*
                    ]),
                }
            }
        }
    };
}
pub(crate) use create_items_map;
