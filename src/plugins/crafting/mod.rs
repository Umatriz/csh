use bevy::{
    app::{Plugin, Startup, Update},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::With,
        reflect::AppTypeRegistry,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    log::error,
    reflect::{std_traits::ReflectDefault, Reflect},
};
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Ui},
    quick::ResourceInspectorPlugin,
};
use std::{marker::PhantomData, sync::Arc};

use self::{
    logic::{
        ClassicalWorkbench, ClassicalWorkbenchMap, Craft, Inventory, Item, ItemBundle, ItemKind,
        ItemProperties, ItemStack, ItemsLayout, Layout, Workbench, WorkbenchTag,
    },
    systems::WindowSystemsPlugin,
};

use super::player::{Player, PlayerProperties};

pub mod logic;
mod macros;
mod systems;

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
            .add_systems(Startup, add_resources)
            .add_plugins(WindowSystemsPlugin);
    }
}

fn add_resources(mut commands: Commands) {
    let map = ClassicalWorkbenchMap::default();
    commands.insert_resource(map);
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
