use bevy::{
    ecs::{bundle::Bundle, component::Component},
    reflect::{std_traits::ReflectDefault, Reflect},
};

#[derive(Component, Hash, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Reflect)]
#[reflect(Default)]
pub struct Item {
    pub name: String,
    pub kind: ItemKind,
    pub level: u8,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            name: "TestItem".to_string(),
            kind: ItemKind::Primitive,
            level: 1,
        }
    }
}

#[derive(Default, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Reflect)]
#[reflect(Default)]
pub enum ItemKind {
    Complex(ItemProperties),
    #[default]
    Primitive,
}

#[derive(Hash, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Reflect, Default)]
#[reflect(Default)]
pub struct ItemProperties {}

// impl Default for ItemProperties {
//     fn default() -> Self {
//         Self {}
//     }
// }

#[derive(Component, Hash, Clone, PartialEq, Eq, Debug, Reflect)]
#[reflect(Default)]
pub struct ItemStack(pub u8);

impl Default for ItemStack {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Bundle, PartialEq, Eq, Hash, Clone, Debug)]
pub struct ItemBundle {
    pub item: Item,
    pub stack: ItemStack,
}
