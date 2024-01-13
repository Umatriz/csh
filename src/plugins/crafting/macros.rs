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
                impl $crate::plugins::crafting::logic::WorkbenchTag for [<$name Workbench>] {}

                #[derive(bevy::prelude::Resource)]
                pub struct [<$name WorkbenchMap>] {
                    pub map: $crate::plugins::crafting::logic::CraftsMap,
                }
            )*
        }
    };
}


/// Select the `ItemKind` uses in `item! macro
/// Example:
/// ```rust
/// item_kind!(primitive) // ItemKind::Primitive
/// item_kind!(complex {}) // ItemKind::Complex(ItemProperties {})
/// ```
#[macro_export]
macro_rules! item_kind {
    (primitive) => {
        $crate::plugins::crafting::ItemKind::Primitive
    };
    (complex {}) => {
        $crate::plugins::crafting::ItemKind::Complex($crate::plugins::crafting::ItemProperties {})
    };
}


/// Create an `ItemBundle`
/// Example:
/// ```rust
/// item! { "ExampleItem", item_kind!(primitive), amount = 1, level = 1 }
/// ```
#[macro_export]
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
        $crate::plugins::crafting::ItemBundle {
            item: $crate::plugins::crafting::Item {
                name: $name.to_string(),
                kind: $kind,
                level: $level,
            },
            stack: $crate::plugins::crafting::ItemStack($amount),
        }
    };
}


#[macro_export]
macro_rules! layout {
    (
        $($item:expr),* $(,)?
    ) => {
        $crate::plugins::crafting::Layout(vec![$($item,)*])
    };
}


/// Create an `ItemsMap`.
/// This macro just implements `Default` for given `WorkbenchMap`.
/// You must use this macro with `create_workbench!` and `item!`.
/// See `create_workbench!` docs for complex example.
macro_rules! create_items_map {
    (
        $name:ty,
        $($($in_item:expr),* => $($out_item:expr),*);*
    ) => {
        impl Default for $name {
            fn default() -> Self {
                Self {
                    map: bevy::utils::hashbrown::HashMap::from([
                        $( (layout![$($in_item,)*], layout![$( $out_item, )*]),)*
                    ]),
                }
            }
        }
    };
}


/// It can be used to create a new Workbench
/// ```
/// workbench! {
///     Classical,
///     item! { "1", item_kind!(primitive), amount = 1, level = 1 }
///     =>
///     item! { "2", item_kind!(primitive), amount = 1, level = 1 },
/// }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! workbench {
    (
        $name:ident,
        $($($in_item:expr),* => $($out_item:expr),*);*
    ) => {
        paste::paste! {
            pub enum [<$name Workbench>] {}
            impl $crate::plugins::crafting::logic::WorkbenchTag for [<$name Workbench>] {}

            #[derive(bevy::prelude::Resource)]
            pub struct [<$name WorkbenchMap>] {
                pub map: $crate::plugins::crafting::logic::CraftsMap,
            }

            impl Default for [<$name WorkbenchMap>] {
                fn default() -> Self {
                    Self {
                        map: bevy::utils::hashbrown::HashMap::from([
                            $( (layout![$($in_item,)*], layout![$( $out_item, )*]),)*
                        ]),
                    }
                }
            }
        }
    };
}

