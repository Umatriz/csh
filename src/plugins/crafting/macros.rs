/// Select the `ItemKind` uses in `item! macro
/// Example:
/// ```rust
/// item_kind!(primitive) // ItemKind::Primitive
/// item_kind!(complex {}) // ItemKind::Complex(ItemProperties {})
/// ```
#[macro_export]
macro_rules! item_kind {
    (primitive) => {
        $crate::plugins::crafting::logic::ItemKind::Primitive
    };
    (complex {}) => {
        $crate::plugins::crafting::logic::ItemKind::Complex(
            $crate::plugins::crafting::logic::ItemProperties {},
        )
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
        $crate::plugins::crafting::logic::ItemBundle {
            item: $crate::plugins::crafting::logic::Item {
                name: $name.to_string(),
                kind: $kind,
                level: $level,
            },
            stack: $crate::plugins::crafting::logic::ItemStack($amount),
        }
    };
}

#[macro_export]
/// ```
/// layout![item! { "ExampleItem", item_kind!(primitive), amount = 1, level = 1 }]
/// ```
macro_rules! layout {
    (
        $($item:expr),* $(,)?
    ) => {
        $crate::plugins::crafting::logic::Layout(vec![$($item,)*])
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

            impl $crate::plugins::crafting::logic::WorkbenchMap for [<$name WorkbenchMap>] {
                fn name(&self) -> &'static str {
                    stringify!([<$name Workbench>])
                }

                fn map(&self) -> &CraftsMap {
                    &self.map
                }
            }

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
