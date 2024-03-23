use std::hash::Hash;

use bevy::{
    asset::{Asset, Handle},
    reflect::{std_traits::ReflectDefault, Reflect},
    utils::hashbrown::HashMap,
};
use serde::{Deserialize, Serialize};

use self::macros::{impl_loadable_for_tuple, impl_loadable_for_type};

#[derive(Serialize, Deserialize, Reflect, PartialEq, Eq, Hash, Debug, Clone)]
#[reflect(Default)]
pub enum AssetRef<A: Asset> {
    #[serde(skip)]
    Handle(Handle<A>),
    Path(String),
}

impl<A: Asset> Default for AssetRef<A> {
    fn default() -> Self {
        Self::Path("".into())
    }
}

impl<A: Asset> AssetRef<A> {
    pub fn is_handle(&self) -> bool {
        match self {
            AssetRef::Handle(_) => true,
            AssetRef::Path(_) => false,
        }
    }

    pub fn is_path(&self) -> bool {
        !self.is_handle()
    }

    pub fn get_handle(&self) -> Option<&Handle<A>> {
        match self {
            AssetRef::Handle(h) => Some(h),
            AssetRef::Path(_) => None,
        }
    }

    pub fn get_path(&self) -> Option<String> {
        match self {
            AssetRef::Handle(_) => None,
            AssetRef::Path(p) => Some(p.clone()),
        }
    }
}

impl<T> Loadable for &mut T {}

pub trait Loadable {
    fn load(&mut self, _load_context: &mut bevy::asset::LoadContext) {}
}

impl<A: Asset> Loadable for AssetRef<A> {
    fn load(&mut self, load_context: &mut bevy::asset::LoadContext) {
        *self = Self::Handle(load_context.load(self.get_path().unwrap()));
    }
}

impl<L: Loadable> Loadable for Vec<L> {
    fn load(&mut self, load_context: &mut bevy::asset::LoadContext) {
        self.iter_mut().for_each(|i| i.load(load_context));
    }
}

impl<K, V> Loadable for HashMap<K, V>
where
    K: Loadable + Eq + Hash + Clone,
    V: Loadable + Clone,
{
    fn load(&mut self, load_context: &mut bevy::asset::LoadContext) {
        *self = {
            let mut hash_map = HashMap::new();
            for (k, v) in self.iter() {
                let mut k = k.clone();
                k.load(load_context);

                let mut v = v.clone();
                v.load(load_context);

                hash_map.insert(k, v);
            }

            hash_map
        };
    }
}

impl_loadable_for_type! {
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64,
    char, bool,
    &'static str, String, Box<str>
}

impl_loadable_for_tuple!(0);
impl_loadable_for_tuple!(0, 1);
impl_loadable_for_tuple!(0, 1, 2);
impl_loadable_for_tuple!(0, 1, 2, 3);
impl_loadable_for_tuple!(0, 1, 2, 3, 4);
impl_loadable_for_tuple!(0, 1, 2, 3, 4, 5);
impl_loadable_for_tuple!(0, 1, 2, 3, 4, 5, 6);
impl_loadable_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7);
impl_loadable_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8);
impl_loadable_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
impl_loadable_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
impl_loadable_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
impl_loadable_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
impl_loadable_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
impl_loadable_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14);
impl_loadable_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);

mod macros {
    macro_rules! impl_loadable_for_type {
        (
            $($t:ty),*
        ) => {
            $(
                impl Loadable for $t {}
            )*
        };
    }
    pub(crate) use impl_loadable_for_type;

    macro_rules! impl_loadable_for_tuple {
        (
            $($it:literal),*
        ) => {
            paste::paste! {
                impl< $( [<T $it>], )* > Loadable for ( $( [<T $it>], )* )
                where
                    $( [<T $it>]: Loadable, )*
                {
                    fn load(&mut self, load_context: &mut bevy::asset::LoadContext) {
                        $(
                            self.$it.load(load_context);
                        )*
                    }
                }
            }
        };
    }
    pub(crate) use impl_loadable_for_tuple;
}
