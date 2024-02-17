use bevy::app::Plugin;
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};

use crate::GameState;

use super::{
    crafting::{ItemsCollection, WorkbenchesCollection},
    player::PlayerCollection,
};

pub struct AssetsLoadingPlugin;

impl Plugin for AssetsLoadingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
                .load_collection::<PlayerCollection>() // .load_collection::<CursorFolderCollection>(),
                .load_collection::<WorkbenchesCollection>()
                .load_collection::<ItemsCollection>(),
        );
    }
}
