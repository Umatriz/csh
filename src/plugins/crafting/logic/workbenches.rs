use std::marker::PhantomData;

use bevy::{ecs::component::Component, utils::hashbrown::HashMap};

use crate::{item, item_kind, workbench};

use super::ItemsLayout;

pub trait WorkbenchTag: Send + Sync + 'static {}

pub type CraftsMap = HashMap<ItemsLayout, ItemsLayout>;

#[derive(Component)]
pub struct Workbench<T: WorkbenchTag> {
    workbench_tag: PhantomData<T>,
}

impl<T: WorkbenchTag> Default for Workbench<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: WorkbenchTag> Workbench<T> {
    pub fn new() -> Self {
        Self {
            workbench_tag: PhantomData,
        }
    }
}

pub trait Craft {
    fn craft(&self, map: &CraftsMap, layout: &ItemsLayout) -> Option<ItemsLayout> {
        dbg!(&layout);
        map.get(layout).cloned()
    }
}

impl<T: WorkbenchTag> Craft for Workbench<T> {}

workbench! {
    Classical,

    item! { "1", item_kind!(primitive), amount = 1, level = 1 }
    =>
    item! { "2", item_kind!(primitive), amount = 1, level = 1 },
    item! { "1", item_kind!(primitive), amount = 1, level = 1 };

    item! { "1", item_kind!(primitive), amount = 1, level = 1 },
    item! { "2", item_kind!(primitive), amount = 1, level = 1 }
    =>
    item! { "3", item_kind!(primitive), amount = 1, level = 1 },
    item! { "1", item_kind!(primitive), amount = 1, level = 1 },
    item! { "2", item_kind!(primitive), amount = 1, level = 1 };

    item! { "3", item_kind!(primitive), amount = 1, level = 1 }
    =>
    item! { "1", item_kind!(primitive), amount = 1, level = 1 };

    item! { "3", item_kind!(primitive), amount = 2, level = 1 }
    =>
    item! { "4", item_kind!(primitive), amount = 5, level = 1 }
}
