#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
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
use bevy_replicon::RepliconPlugins;
use bevy_replicon_renet::RepliconRenetPlugins;
use bevy_xpbd_2d::plugins::{PhysicsDebugPlugin, PhysicsPlugins};
use plugins::assets::AssetsLoadingPlugin;
use plugins::crafting::logic::Workbench;
use plugins::cursor::CursorPlugin;
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
pub mod spring;
pub mod utils;

pub use core::stringify;

fn main() {
    App::new()
        .register_type::<WindowContext>()
        .init_resource::<WindowContext>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()))
        .add_plugins((RepliconPlugins, RepliconRenetPlugins))
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
            CursorPlugin,
            // ChestPlugin
        ))
        .init_state::<GameState>()
        .replicate::<Transform>()
        .replicate::<PlayerColor>()
        .replicate::<Player>()
        .add_client_event::<MoveDirection>(ChannelKind::Ordered)
        .add_systems(
            Update,
            (
                movement_system.run_if(has_authority), // Runs only on the server or a single player.
                input_system,
            ),
        )
        .add_plugins(AssetInspectorPlugin::<Workbench>::default())
        .add_systems(PreUpdate, player_init_system.after(ClientSet::Receive))
        .run()
}

#[derive(Resource)]
struct Wrapper(Handle<Workbench>);

// fn test_workbench_loading(
//     asset_server: Res<AssetServer>,
//     mut commands: Commands,
//     // mut workbenches: ResMut<Assets<WorkbenchAsset>>,
// ) {
//     let handle = asset_server.load::<Workbench>("classical.workbench.ron");

//     commands.insert_resource(Wrapper(handle));
// }

#[derive(Resource, Default, Reflect)]
#[reflect(Default)]
pub struct WindowContext {
    menu_window: bool,
    workbench_window: bool,
    inventory_window: bool,
    enchantment_window: bool,
    add_item_window: bool,
}

fn player_init_system(mut commands: Commands, spawned_players: Query<Entity, Added<Player>>) {
    for entity in &spawned_players {
        warn!("PLAYER INIT");
        commands
            .entity(entity)
            .insert((
                GlobalTransform::default(),
                VisibilityBundle::default(),
                Handle::<Image>::default(),
            ))
            .add(|mut c: EntityWorldMut<'_>| {
                if let Some(color) = c.get::<PlayerColor>() {
                    c.insert(Sprite {
                        color: color.0,
                        custom_size: Some(Vec2::splat(20.0)),
                        ..Default::default()
                    });
                }
            });
    }
}

/// Reads player inputs and sends [`MoveCommandEvents`]
fn input_system(mut move_events: EventWriter<MoveDirection>, input: Res<ButtonInput<KeyCode>>) {
    let mut direction = Vec2::ZERO;
    if input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if direction != Vec2::ZERO {
        move_events.send(MoveDirection(direction.normalize_or_zero()));
    }
}

/// Mutates [`PlayerPosition`] based on [`MoveCommandEvents`].
///
/// Fast-paced games usually you don't want to wait until server send a position back because of the latency.
/// But this example just demonstrates simple replication concept.
fn movement_system(
    time: Res<Time>,
    mut move_events: EventReader<FromClient<MoveDirection>>,
    mut players: Query<(&Player, &mut Transform)>,
) {
    const MOVE_SPEED: f32 = 300.0;
    for FromClient { client_id, event } in move_events.read() {
        for (player, mut position) in &mut players {
            if *client_id == player.0 {
                position.translation += (event.0 * time.delta_seconds() * MOVE_SPEED).extend(0.0);
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Menu,
    Game,
}

// #[derive(Resource, Debug)]
// struct FpsPlot {
//     timer: Timer,
//     points: Vec<[f64; 2]>,
// }

// impl Default for FpsPlot {
//     fn default() -> Self {
//         Self {
//             timer: Timer::from_seconds(1.0, TimerMode::Repeating),
//             points: Default::default(),
//         }
//     }
// }

// fn show_plot_window(
//     mut contexts: EguiContexts,
//     mut points_resource: ResMut<FpsPlot>,
//     diagnostics: Res<DiagnosticsStore>,
//     time: Res<Time>,
// ) {
//     egui::Window::new("FPS").show(contexts.ctx_mut(), |ui| {
//         let val_opt = diagnostics
//             .get(FrameTimeDiagnosticsPlugin::FPS)
//             .and_then(|fps| fps.smoothed());
//         if let Some(value) = val_opt {
//             if points_resource.timer.tick(time.delta()).just_finished() {
//                 let point = [time.elapsed_seconds_f64(), value];
//                 points_resource.points.push(point);
//             }
//         }
//         let line = Line::new(points_resource.points.clone());
//         ui.label(
//             egui::RichText::new(format!(
//                 "FPS: {}",
//                 val_opt
//                     .map(|v| v.round().to_string())
//                     .unwrap_or("N/A".to_string()),
//             ))
//             .size(20.0),
//         );

//         let x_axes = vec![AxisHints::default().label("Time")];
//         let y_axes = vec![AxisHints::default().label("FPS")];

//         egui_plot::Plot::new("FPS")
//             .view_aspect(2.0)
//             .allow_zoom(false)
//             .custom_x_axes(x_axes)
//             .custom_y_axes(y_axes)
//             .show(ui, |ui| ui.line(line))
//     });
// }

// fn setup(
//     mut commands: Commands,
//     cursor_folder: Res<CursorFolderCollection>,
//     mut textures_atlas: ResMut<Assets<TextureAtlas>>,
//     mut textures: ResMut<Assets<Image>>,
//     window: Query<Entity, (With<Window>, With<PrimaryWindow>)>,
// ) {
//     // Spawn cursor
//     let mut texture_atlas_builder = TextureAtlasBuilder::default();
//     for handle in cursor_folder.folder.iter() {
//         let id = handle.id();
//         let Some(texture) = textures.get(id) else {
//             warn!(
//                 "{:?} did not resolve to an `Image` asset.",
//                 handle.path().unwrap()
//             );
//             continue;
//         };

//         texture_atlas_builder.add_texture(id, texture);
//     }

//     let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();

//     commands.spawn((
//         Cursor::new()
//             .with_os_cursor(false)
//             .add_sprite_offset(Vec2::splat(14.0))
//             .add_sprite_offset(Vec2::new(10.0, 12.0))
//             .add_sprite_offset(Vec2::splat(40.0)),
//         SpriteSheetBundle {
//             texture_atlas: textures_atlas.add(texture_atlas),
//             transform: Transform {
//                 translation: Vec3::new(0.0, 0.0, 800.0),
//                 scale: Vec3::new(0.4, 0.4, 1.0),
//                 ..default()
//             },
//             sprite: TextureAtlasSprite {
//                 color: Color::rgba(252. / 255., 226. / 255., 8. / 255., 2.0).with_l(0.68),
//                 anchor: bevy::sprite::Anchor::TopLeft,
//                 ..default()
//             },
//             ..default()
//         },
//     ));
// }
