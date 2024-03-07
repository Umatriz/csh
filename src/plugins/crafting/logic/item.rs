use std::num::Saturating;

use bevy::{
    asset::Asset,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::{Entity, MapEntities},
        event::Event,
    },
    reflect::{std_traits::ReflectDefault, Reflect},
};
use serde::{Deserialize, Serialize};

#[derive(Component, Hash, Clone, PartialEq, Eq, Debug, Reflect, Serialize, Deserialize, Asset)]
#[reflect(Default)]
pub struct Item {
    pub name: String,
    pub kind: ItemKind,
    pub level: u8,
}

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

impl ItemBundle {
    pub fn as_tuple(&self) -> (&Item, &ItemStack) {
        (&self.item, &self.stack)
    }
}

#[derive(Debug, Clone, Event, Deserialize, Serialize)]
pub struct ItemEvent {
    pub kind: ItemEventKind,
    pub inventory: Option<Entity>,
    pub item: ItemBundle,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ItemEventKind {
    Add,
    Remove,
}

impl MapEntities for ItemEvent {
    fn map_entities<M: bevy::prelude::EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.inventory = self.inventory.map(|ent| entity_mapper.map_entity(ent));
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
