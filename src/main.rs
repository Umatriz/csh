#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use bevy_inspector_egui::bevy_egui::{EguiContexts, EguiPlugin};

use bevy_inspector_egui::egui::Ui;
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};

use bevy_mod_picking::DefaultPickingPlugins;

use bevy_replicon::core::replication_rules::AppReplicationExt;

use bevy_replicon::server::{ServerPlugin, TickPolicy};
use bevy_replicon::RepliconPlugins;
use bevy_replicon_renet::RepliconRenetPlugins;
use bevy_replicon_snap::SnapshotInterpolationPlugin;
use bevy_xpbd_3d::plugins::{PhysicsDebugPlugin, PhysicsPlugins};
use debugging::InspectorPlugin;
use plugins::assets::AssetsLoadingPlugin;

use plugins::environment;
use plugins::gen::chunking::{ChunkingPlugin, CreatedPoints};
use plugins::gen::GenPlugins;
// use plugins::cursor::CursorPlugin;
use plugins::network::NetworkPlugin;
use plugins::{camera::CameraPlugin, crafting::CraftingPlugin, player::PlayerPlugin};

pub mod args;
pub mod asset_macro;
pub mod asset_ref;
pub mod debugging;
pub mod plugins;
pub mod utils;

pub use core::stringify;

pub use debugging::InspectorWindows;

const MAX_TICK_RATE: u16 = 60;

fn main() {
    App::new()
        .register_type::<InspectorWindows>()
        .init_resource::<InspectorWindows>()
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
        .add_plugins((
            EguiPlugin,
            InspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Backquote)),
        ))
        .add_plugins(DefaultPickingPlugins)
        // .add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
        .add_plugins((
            GenPlugins,
            PlayerPlugin,
            CameraPlugin,
            CraftingPlugin,
            AssetsLoadingPlugin,
            NetworkPlugin,
            environment::plugin,
            plugins::gen::noises::perlin_noise,
            // CursorPlugin,
            // ChestPlugin
        ))
        .add_systems(Startup, init_loaders)
        .init_state::<GameState>()
        .replicate::<Transform>()
        .run()
}

fn init_loaders(mut contexts: EguiContexts) {
    egui_extras::install_image_loaders(contexts.ctx_mut())
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Menu,
    Game,
}
