use bevy::{
    asset::{Asset, Handle},
    reflect::TypePath,
    utils::HashMap,
};

use super::logic::Item;

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

macro_rules! asset_project {
    (
        $($tt:tt)*
    ) => {
        __asset_project_internal! {
            []
            $($tt)*
        }
    };
}

macro_rules! __asset_project_internal {
    (
        [$($attrs:tt)*]

        #[$($attr:tt)*]
        $($tt:tt)*
    ) => {
        __asset_project_internal! {
            [$($attrs)* #[$($attr)*]]
            $($tt)*
        }
    };
    (
        [$($attrs:tt)*]
        $vis:vis $struct_ty_ident:ident $ident:ident
        $($tt:tt)*
    ) => {
        __asset_project_parse! {
            [$($attrs)*]
            [$vis $struct_ty_ident $ident]

            $($tt)*
        }
    };
}

macro_rules! __asset_project_parse {
    (
        [$($attrs:tt)*]
        [$vis:vis $struct_ty_ident:ident $ident:ident]

        {
            $($body_data:tt)*
        }
    ) => {
        __asset_project_expand! {
            [$($attrs)* $vis $struct_ty_ident $ident]
            {
                $($body_data)*
            }
        }
    };
}

macro_rules! __asset_project_expand {
    (
        [$(#[$attrs:meta])* $vis:vis $struct_ty_ident:ident $ident:ident]
        {
            $($body_data:tt)*
        }
    ) => {
        __asset_project_construct! {
            [$(#[$attrs])* $vis $struct_ty_ident $ident]
            {
                $($body_data)*
            }
        }
    };
}

macro_rules! __asset_project_construct {
    (
        [$(#[$attrs:meta])* $vis:vis struct $ident:ident]

        {
            $(
                $field_vis:vis $field:ident: $field_ty:ty
            ),+ $(,)?
        }
    ) => {
        $(#[$attrs])*
        $vis struct $ident
        {
            $(
                $field_vis $field: $field_ty
            ),+
        }
    };
    (
        [$(#[$attrs:meta])* $vis:vis enum $ident:ident]

        {
            $(
                $(#[$variant_attrs:meta])*
                $variant:ident $({
                    $(
                        $field:ident: $field_ty:ty
                    ),+ $(,)?
                })?
            ),+ $(,)?
        }
    ) => {
        $(#[$attrs])*
        $vis enum $ident
        {
            $(
                $(#[$variant_attrs])*
                $variant $({
                    $(
                        $field: $field_ty
                    ),+
                })?
            ),+
        }
    };
}

macro_rules! type_check {( $($input:tt)* ) => (
    muncher! {
        [input: $($input)* ]
        [output: ]
    }
)}
use type_check;

macro_rules! muncher {
    (
        [input:
            Handle<$T:ty $(,)?>
            $($rest:tt)*
        ]
        [output: $($output:tt)* ]
    ) => (muncher! {
        [input:
            $($rest)*
        ]
        [output: $($output)*
            String
        ]
    });

    (
        [input:
            Handle<$T:ty $(,)?>>
            $($rest:tt)*
        ]
        $output:tt
    ) => (muncher! {
        [input:
            Handle<$T> >
            $($rest)*
        ]
        $output
    });

    (
        [input:
            $not_Handle:tt
            $($rest:tt)*
        ]
        [output: $($output:tt)*]
    ) => (muncher! {
        [input:
            $($rest)*
        ]
        [output: $($output)*
            $not_Handle
        ]
    });

    (
        [input: /* nothing left */ ]
        [output: $($output:tt)* ]
    ) => (
        $($output)*
    )
}
use muncher;

type AT = type_check! {
    HashMap<Vec<(Handle<Item>, String)>, Vec<Handle<Item>>>
};

asset_project! {
    #[derive(Asset, TypePath)]
    pub struct A {
        pub test: Vec<Handle<Item>>,
        pub test2: HashMap<Vec<Handle<Item>>, Vec<Handle<Item>>>
    }
}
