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
                $(#[$field_attr:meta])*
                $field_vis:vis $field:ident: $field_ty:ty
            ),+ $(,)?
        }
    ) => {
        $(#[$attrs])*
        $vis struct $ident
        {
            $(
                $(#[$field_attr])*
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

macro_rules! __type_check {
    ( [s] $($input:tt)* ) => (
        __type_check_muncher! {
            [input: $($input)* ]
            [output: ]
            [mode: string]
        }
    );

    ( $($input:tt)* ) => (
        __type_check_muncher! {
            [input: $($input)* ]
            [output: ]
            [mode: default]
        }
    );
}
use __type_check;

macro_rules! __type_check_muncher {
    (
        [input:
            Handle<$T:ty $(,)?>
            $($rest:tt)*
        ]
        [output: $($output:tt)* ]
        $mode:tt
    ) => (__type_check_muncher! {
        [input:
            $($rest)*
        ]
        [output: $($output)*
            String
        ]
        $mode
    });

    (
        [input:
            Handle<$T:ty $(,)?>>
            $($rest:tt)*
        ]
        [output: $($output:tt)* ]
        $mode:tt
    ) => (__type_check_muncher! {
        [input:
            Handle<$T> >
            $($rest)*
        ]
        [output: $($output)*]
        $mode
    });

    (
        [input:
            ( $($group:tt)* )
            $($rest:tt)*
        ]
        [output: $($output:tt)* ]
        $mode:tt
    ) => (
        __type_check_muncher! {
            [input: $($rest)* ]
            [output: $($output)*
                __type_check_muncher! {
                    [input: $($group)*]
                    [output: ]
                    [mode: parentheses]
                }
            ]
            $mode
        }
    );

    (
        [input:
            { $($group:tt)* }
            $($rest:tt)*
        ]
        [output: $($output:tt)* ]
        $mode:tt
    ) => (
        __type_check_muncher! {
            [input: $($rest)* ]
            [output: $($output)*
                __type_check_muncher! {
                    [input: $($group)*]
                    [output: ]
                    [mode: braces]
                }
            ]
            $mode
        }
    );

    (
        [input:
            [ $($group:tt)* ]
            $($rest:tt)*
        ]
        [output:
            $($output:tt)*
        ]
        $mode:tt
    ) => (__type_check_muncher! {
        [input:
            $($rest)*
        ]
        [output:
            $($output)*
            __type_check_muncher! {
                [input: $($group)*]
                [output: ]
                [mode: square_brackets]
            }
        ]
        $mode
    });

    (
        [input:
            $not_Handle:tt
            $($rest:tt)*
        ]
        [output: $($output:tt)*]
        $mode:tt
    ) => (__type_check_muncher! {
        [input:
            $($rest)*
        ]
        [output: $($output)*
            $not_Handle
        ]
        $mode
    });

    (
        [input: /* nothing left */ ]
        [output: $($output:tt)* ]
        [mode: default]
    ) => (
        $($output)*
    );

    (
        [input: /* nothing left */ ]
        [output: $($output:tt)* ]
        [mode: string]
    ) => (
        stringify!($($output)*)
    );

    (
        [input: /* nothing left! */]
        [output: $($output:tt)*]
        [mode: parentheses]
    ) => (
        ( $($output)* )
    );

    (
        [input: /* nothing left! */]
        [output: $($output:tt)*]
        [mode: square_brackets]
    ) => (
        [ $($output)* ]
    );

    (
        [input: /* nothing left! */]
        [output: $($output:tt)*]
        [mode: braces]
    ) => (
        { $($output)* }
    );
}
use __type_check_muncher;

asset_project! {
    #[derive(Asset, TypePath)]
    pub struct A {
        pub test: Vec<Handle<Item>>,
        pub test2: HashMap<Vec<Handle<Item>>, Vec<Handle<Item>>>
    }
}

#[test]
fn feature() {
    dbg!(__type_check! {
        [s]
        pub test: Vec<Handle<Item>>,
        pub(crate) test2: Vec<Handle<Item>>,
    });
}
