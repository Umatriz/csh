use bevy::{
    app::Plugin,
    asset::{Asset, AssetApp, Assets, AsyncReadExt},
    ecs::system::Res,
    reflect::{std_traits::ReflectDefault, Reflect},
    utils::hashbrown::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::{
    asset_macro::impl_asset_loader,
    asset_ref::{AssetRef, Loadable},
};

use super::{Item, ItemBundle, ItemStack, Layout};

pub struct WorkbenchPlugin;

impl Plugin for WorkbenchPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<Workbench>()
            .register_asset_loader(WorkbenchAssetLoader)
            .register_asset_reflect::<Workbench>();
    }
}

#[derive(Debug, Asset, Default, Reflect, Serialize, Deserialize)]
#[reflect(Default)]
pub struct Workbench {
    name: String,
    recipes: HashMap<Vec<(AssetRef<Item>, ItemStack)>, Vec<(AssetRef<Item>, ItemStack)>>,
}

impl_asset_loader! {
    Workbench &["workbench.ron"];
    recipes
}

impl Loadable for ItemStack {}

impl Workbench {
    pub fn craft<'a>(
        &'a self,
        assets: &'a Res<Assets<Item>>,
        layout: &'a Layout<ItemBundle>,
    ) -> Option<Vec<(&Item, &ItemStack)>> {
        // It's expensive, I know
        // FIXME
        let map = self
            .recipes
            .iter()
            .map(|(k, v)| {
                (
                    k.iter()
                        .filter_map(|(h, s)| assets.get(h.get_handle().unwrap()).map(|a| (a, s)))
                        .collect(),
                    v.iter()
                        .filter_map(|(h, s)| assets.get(h.get_handle().unwrap()).map(|a| (a, s)))
                        .collect(),
                )
            })
            .collect::<HashMap<Vec<(&Item, &ItemStack)>, Vec<(&Item, &ItemStack)>>>();

        let layout = layout
            .get()
            .iter()
            .map(|b| (&b.item, &b.stack))
            .collect::<Vec<_>>();

        map.get(&layout).cloned()
    }
}
