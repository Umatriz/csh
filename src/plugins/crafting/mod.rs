use bevy::{
    app::Plugin,
    ecs::{component::Component, entity::Entity, system::Commands},
    reflect::{std_traits::ReflectDefault, Reflect},
};

use std::sync::Arc;

use self::{
    logic::{Inventory, Item, ItemKind, ItemProperties, ItemStack},
    systems::WindowSystemsPlugin,
};

use super::player::PlayerProperties;

pub mod logic;
mod macros;
mod systems;

pub use systems::show_item;

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<Inventory>()
            .register_type::<Option<Item>>()
            .register_type_data::<Option<Item>, ReflectDefault>()
            .register_type::<Item>()
            .register_type::<Vec<Item>>()
            .register_type_data::<Vec<Item>, ReflectDefault>()
            .register_type::<Item>()
            .register_type::<ItemKind>()
            .register_type::<ItemStack>()
            .register_type::<ItemProperties>()
            .add_plugins(WindowSystemsPlugin);
    }
}

#[derive(Component)]
pub struct ItemEnchantments(Vec<EnchantmentWithState>);

struct EnchantmentWithState {
    enchantment: Arc<dyn Enchantment>,
    state: bool,
}

impl ItemEnchantments {
    pub fn add(&mut self, enchantment: Arc<dyn Enchantment>) {
        self.0.push(EnchantmentWithState {
            enchantment,
            state: false,
        })
    }

    pub fn apply_unapplied(
        &mut self,
        player_properties: &mut PlayerProperties,
        item_properties: &mut ItemProperties,
        commands: &mut Commands,
        player_entity: Entity,
    ) {
        self.0.iter_mut().filter(|i| !i.state).for_each(|item| {
            item.enchantment.modify_item_properties(item_properties);
            item.enchantment.modify_player_properties(player_properties);
            item.enchantment.modify_world(commands, player_entity);

            item.state = true
        });
    }
}

// Just make sure that we don't add something that not allows us to construct a trait-object
const _: Option<Box<dyn Enchantment>> = None;

/// It represents additional modifiers for [`ItemProperties`] and [`ItemProperties`].
/// Also you can access [`Commands`] and player's [`Entity`]
/// using [`Enchantment::modify_world`] method
///
/// Methods in this trait are only called once when this `Enchantment` is applied
pub trait Enchantment: Reflect {
    fn modify_player_properties(&self, _properties: &mut PlayerProperties) {}
    fn modify_item_properties(&self, _properties: &mut ItemProperties) {}
    fn modify_world(&self, _commands: &mut Commands, _player_entity: Entity) {}
}

#[derive(Component)]
pub struct EnchantingTable;

impl EnchantingTable {
    pub fn enchant(
        &self,
        item_enchantments: &mut ItemEnchantments,
        enchantment: Arc<dyn Enchantment>,
    ) {
        item_enchantments.add(enchantment)
    }
}

#[derive(Reflect)]
struct Power;

#[derive(Component)]
struct PowerTag;

impl Enchantment for Power {
    fn modify_world(&self, commands: &mut Commands, player_entity: Entity) {
        if let Some(mut player) = commands.get_entity(player_entity) {
            player.insert(PowerTag);
        }
    }
}
