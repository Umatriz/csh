use bevy::{
    ecs::{bundle::Bundle, component::Component, entity::Entity, event::Event},
    reflect::{std_traits::ReflectDefault, Reflect},
};
use bevy_replicon::replicon_core::replication_rules::MapNetworkEntities;
use serde::{Deserialize, Serialize};

#[derive(
    Component, Hash, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Reflect, Serialize, Deserialize,
)]
#[reflect(Default)]
pub struct Item {
    pub name: String,
    pub kind: ItemKind,
    pub level: u8,
}

#[derive(Debug, Deserialize, Event, Serialize)]
pub struct ItemEvent {
    pub kind: ItemEventKind,
    pub inventory: Option<Entity>,
    pub item: ItemBundle,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ItemEventKind {
    Add,
    Remove,
}

impl MapNetworkEntities for ItemEvent {
    fn map_entities<T: bevy_replicon::prelude::Mapper>(&mut self, mapper: &mut T) {
        self.inventory = self.inventory.map(|ent| mapper.map(ent));
    }
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

#[derive(
    Default, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Reflect, Serialize, Deserialize,
)]
#[reflect(Default)]
pub enum ItemKind {
    Complex(ItemProperties),
    #[default]
    Primitive,
}

#[derive(
    Hash, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Reflect, Default, Serialize, Deserialize,
)]
#[reflect(Default)]
pub struct ItemProperties {}

// impl Default for ItemProperties {
//     fn default() -> Self {
//         Self {}
//     }
// }

#[derive(Component, Hash, Clone, PartialEq, Eq, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Default)]
pub struct ItemStack(pub u8);

impl Default for ItemStack {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Bundle, PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct ItemBundle {
    pub item: Item,
    pub stack: ItemStack,
}
