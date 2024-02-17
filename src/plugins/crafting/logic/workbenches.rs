use std::{borrow::Cow, hash::Hash, marker::PhantomData};

use bevy::{
    app::Plugin,
    asset::{Asset, AssetApp, AssetLoader, Assets, AsyncReadExt, Handle},
    ecs::{component::Component, system::Res},
    reflect::{std_traits::ReflectDefault, FromReflect, Reflect, TypePath},
    utils::{hashbrown::HashMap, thiserror::Error},
};
use serde::{Deserialize, Serialize};

use crate::{item, item_kind, workbench};

use super::{Item, ItemBundle, ItemStack, ItemsLayout, Layout};

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
pub struct Workbench<State = Handle<Item>>
where
    State: TypePath + Send + Sync + Hash + Eq + FromReflect + Default,
{
    recipes: HashMap<Vec<(State, ItemStack)>, Vec<(State, ItemStack)>>,
}

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
                        .filter_map(|(h, s)| assets.get(h).map(|a| (a, s)))
                        .collect(),
                    v.iter()
                        .filter_map(|(h, s)| assets.get(h).map(|a| (a, s)))
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

// #[derive(Debug, Deserialize, Serialize, Reflect, PartialEq, Eq, Hash)]
// #[reflect(Default)]
// pub enum MaybeHandle<A: Asset> {
//     Path(String),
//     #[serde(skip)]
//     Handle(Handle<A>),
// }

// #[derive(Debug, Deserialize, PartialEq, Eq, Hash, Default)]
// pub struct PathWithHandle<A: Asset> {
//     path: &'static str,
//     #[serde(skip)]
//     handle: Handle<A>,
// }

// impl<A: Asset> Default for MaybeHandle<A> {
//     fn default() -> Self {
//         Self::Path("".into())
//     }
// }

#[derive(Default)]
struct WorkbenchAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
enum WorkbenchAssetLoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON Error](serde_ron::error::SpannedError)
    #[error("Could not parse RON: {0}")]
    RonError(#[from] ron::error::SpannedError),
}

impl AssetLoader for WorkbenchAssetLoader {
    type Asset = Workbench;

    type Settings = ();

    type Error = WorkbenchAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let unfinished = ron::de::from_bytes::<Workbench<String>>(&bytes)?;

            let mut recipes = HashMap::new();

            for (key, value) in unfinished.recipes.into_iter() {
                let mut k = vec![];
                for (path, stack) in key {
                    let handle = load_context.load(path);
                    k.push((handle, stack));
                }

                let mut v = vec![];
                for (path, stack) in value {
                    let handle = load_context.load(path);
                    v.push((handle, stack));
                }

                recipes.insert(k, v);
            }

            let asset = Workbench { recipes };
            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["workbench.ron"]
    }
}
