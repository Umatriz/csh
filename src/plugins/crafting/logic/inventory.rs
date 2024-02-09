use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query},
    },
    log::{error, info},
    reflect::{std_traits::ReflectDefault, Reflect},
};
use bevy_replicon::replicon_core::replication_rules::{MapNetworkEntities, Replication};
use serde::{Deserialize, Serialize};

use super::{Item, ItemBundle, ItemStack, Layout};

#[derive(Component, Default, Reflect, Serialize, Deserialize)]
#[reflect(Default)]
pub struct Inventory {
    pub map: Vec<Option<Entity>>,
}

impl MapNetworkEntities for Inventory {
    fn map_entities<T: bevy_replicon::prelude::Mapper>(&mut self, mapper: &mut T) {
        for (opt, ent) in self
            .map
            .iter_mut()
            .filter_map(|opt| (*opt).map(|ent| (opt, ent)))
        {
            *opt = Some(mapper.map(ent))
        }
    }
}

pub type ItemsLayout = Layout<ItemBundle>;

impl Inventory {
    pub fn new() -> Self {
        Self { map: Vec::new() }
    }

    pub fn join(&mut self, other: &mut Self) {
        self.map.append(&mut other.map)
    }

    pub fn take_linear(&mut self, entity: Entity) -> Option<Entity> {
        for item_ref in self.map.iter_mut().filter(|opt| opt.is_some()) {
            if item_ref == &mut Some(entity) {
                return item_ref.take();
            }
        }
        None
    }

    pub fn take_linear_item(&mut self, query: &Query<&Item>, item: &Item) -> Option<Entity> {
        for (id, item_ref) in self
            .map
            .iter()
            .enumerate()
            .filter_map(|(id, opt)| opt.map(|e| (id, e)))
            .filter_map(|(id, entity)| {
                let g = query.get(entity);
                g.ok().map(|i| (id, i))
            })
        {
            if item_ref == item {
                return self.map.get_mut(id).and_then(|item| item.take());
            }
        }
        None
    }

    pub fn take_satisfying_layout(
        &mut self,
        query: &Query<(&Item, &ItemStack)>,
        layout: &ItemsLayout,
    ) -> Option<Vec<Entity>> {
        self.search_satisfying(query, layout).map(|ids| {
            ids.into_iter()
                .filter_map(|id| self.take(id))
                .collect::<Vec<_>>()
        })
    }

    pub fn take(&mut self, id: usize) -> Option<Entity> {
        self.map.get_mut(id).and_then(|opt| opt.take())
    }

    pub fn search_satisfying(
        &self,
        query: &Query<(&Item, &ItemStack)>,
        layout: &ItemsLayout,
    ) -> Option<Vec<usize>> {
        let items = layout.get();
        let mut vec = vec![];
        for ItemBundle { item, stack } in items {
            if let Some(id) = self.search_condition(query, |it, it_stack, _| {
                it.name == item.name && it.kind == item.kind && it_stack.0 >= stack.0
            }) {
                vec.push(id)
            }
        }
        if items.len() == vec.len() {
            return Some(vec);
        }
        None
    }

    pub fn search_condition(
        &self,
        query: &Query<(&Item, &ItemStack)>,
        condition: impl Fn(&Item, &ItemStack, Entity) -> bool,
    ) -> Option<usize> {
        for (id, (it, stack), entity) in self
            .map
            .iter()
            .enumerate()
            .filter_map(|(id, opt)| (*opt).map(|i| (id, i)))
            .filter_map(|(id, entity)| query.get(entity).ok().map(|item| (id, item, entity)))
        {
            if condition(it, stack, entity) {
                return Some(id);
            }
        }
        None
    }

    pub fn add_combine(
        &mut self,
        commands: &mut Commands,
        query: &mut Query<(&mut Item, &mut ItemStack)>,
        layout: Layout<ItemBundle>,
    ) {
        let items = layout.get();

        for ItemBundle { item, stack } in items {
            if let Some((id, item_entity)) = self
                .search_condition(&query.to_readonly(), |it, _, _| {
                    it.name == item.name && it.kind == item.kind
                })
                .and_then(|id| self.take(id).map(|ent| (id, ent)))
            {
                info!("Found existing entity in inventory");
                if let Ok((item_in_inventory, mut item_in_inventory_stack)) =
                    query.get_mut(item_entity)
                {
                    info!("Passed");
                    item_in_inventory_stack.0 += stack.0;
                    dbg!(&item_in_inventory);
                    self.map[id] = Some(item_entity);
                } else {
                    error!("Fail")
                }
            } else {
                info!("Adding single existing entity");
                self.add_single_new(
                    commands,
                    ItemBundle {
                        item: (*item).clone(),
                        stack: (*stack).clone(),
                    },
                );
            }
        }
    }

    pub fn add(&mut self, commands: &mut Commands, layout: ItemsLayout) {
        layout
            .get()
            .iter()
            .for_each(|i| self.add_single_new(commands, i.clone()))
    }

    fn add_single_new(&mut self, commands: &mut Commands, item: ItemBundle) {
        let id = commands.spawn(item).insert(Replication).id();
        self.map.push(Some(id))
    }

    pub fn add_single(&mut self, entity: Entity) {
        self.map.push(Some(entity));
    }
}
