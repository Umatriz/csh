use bevy::app::{PluginGroup, PluginGroupBuilder};

use self::chunking::ChunkingPlugin;

pub mod chunking;
pub mod noises;
pub mod point;

pub struct GenPlugins;

impl PluginGroup for GenPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(ChunkingPlugin::default())
    }
}
