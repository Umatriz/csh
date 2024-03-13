#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::{
    AssetInspectorPlugin, ResourceInspectorPlugin, WorldInspectorPlugin,
};
use bevy_mod_picking::DefaultPickingPlugins;

use bevy_replicon::client::ClientSet;
use bevy_replicon::core::common_conditions::has_authority;
use bevy_replicon::core::replication_rules::AppReplicationExt;
use bevy_replicon::core::replicon_channels::ChannelKind;
use bevy_replicon::network_event::client_event::{ClientEventAppExt, FromClient};
use bevy_replicon::server::{ServerPlugin, TickPolicy};
use bevy_replicon::RepliconPlugins;
use bevy_replicon_renet::RepliconRenetPlugins;
use bevy_replicon_snap::SnapshotInterpolationPlugin;
use bevy_xpbd_3d::plugins::{PhysicsDebugPlugin, PhysicsPlugins};
use plugins::assets::AssetsLoadingPlugin;
use plugins::crafting::logic::Workbench;
use plugins::environment;
// use plugins::cursor::CursorPlugin;
use plugins::network::NetworkPlugin;
use plugins::{
    camera::CameraPlugin,
    crafting::CraftingPlugin,
    player::{MoveDirection, Player, PlayerColor, PlayerPlugin},
};

pub mod args;
pub mod asset_macro;
pub mod asset_ref;
pub mod plugins;
pub mod utils;

pub use core::stringify;

const MAX_TICK_RATE: u16 = 60;

fn main() {
    App::new()
        .register_type::<WindowContext>()
        .init_resource::<WindowContext>()
        // TODO: Remove `WinitSettings`
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        })
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()))
        .add_plugins((
            RepliconPlugins.set(ServerPlugin {
                tick_policy: TickPolicy::MaxTickRate(MAX_TICK_RATE),
                ..Default::default()
            }),
            RepliconRenetPlugins,
            SnapshotInterpolationPlugin {
                max_tick_rate: MAX_TICK_RATE,
            },
        ))
        .add_plugins((EguiPlugin, WorldInspectorPlugin::new()))
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(ResourceInspectorPlugin::<WindowContext>::default())
        // .add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
        .add_plugins((
            PlayerPlugin,
            CameraPlugin,
            CraftingPlugin,
            AssetsLoadingPlugin,
            NetworkPlugin,
            environment::plugin,
            // CursorPlugin,
            // ChestPlugin
        ))
        .init_state::<GameState>()
        .replicate::<Transform>()
        .add_plugins(AssetInspectorPlugin::<Workbench>::default())
        .run()
}

#[derive(Resource, Default, Reflect)]
#[reflect(Default)]
pub struct WindowContext {
    menu_window: bool,
    workbench_window: bool,
    inventory_window: bool,
    enchantment_window: bool,
    add_item_window: bool,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Menu,
    Game,
}
