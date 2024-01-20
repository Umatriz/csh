#![allow(clippy::type_complexity)]

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_inspector_egui::{
    bevy_egui::{EguiContexts, EguiPlugin},
    egui,
};
use bevy_mod_picking::DefaultPickingPlugins;
use egui_plot::{AxisHints, Line};
use plugins::{
    camera::CameraPlugin, chest::ChestPlugin, crafting::CraftingPlugin, player::PlayerPlugin,
};

pub mod logic;
pub mod plugins;
pub mod utils;

pub use core::stringify;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((EguiPlugin, WorldInspectorPlugin::new()))
        .add_plugins(DefaultPickingPlugins)
        .add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
        .add_plugins((PlayerPlugin, CameraPlugin, CraftingPlugin, ChestPlugin))
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Next), // .load_collection::<CursorFolderCollection>(),
        )
        // .add_systems(Startup, setup.run_if(in_state(GameState::Next)))
        .add_systems(Update, show_plot_window)
        .init_resource::<FpsPlot>()
        .run()
}

#[derive(Resource, Debug)]
struct FpsPlot {
    timer: Timer,
    points: Vec<[f64; 2]>,
}

impl Default for FpsPlot {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            points: Default::default(),
        }
    }
}

fn show_plot_window(
    mut contexts: EguiContexts,
    mut points_resource: ResMut<FpsPlot>,
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
) {
    egui::Window::new("FPS").show(contexts.ctx_mut(), |ui| {
        let val_opt = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed());
        if let Some(value) = val_opt {
            if points_resource.timer.tick(time.delta()).just_finished() {
                let point = [time.elapsed_seconds_f64(), value];
                points_resource.points.push(point);
            }
        }
        let line = Line::new(points_resource.points.clone());
        ui.label(
            egui::RichText::new(format!(
                "FPS: {}",
                val_opt
                    .map(|v| v.round().to_string())
                    .unwrap_or("N/A".to_string()),
            ))
            .size(20.0),
        );

        let x_axes = vec![AxisHints::default().label("Time")];
        let y_axes = vec![AxisHints::default().label("FPS")];

        egui_plot::Plot::new("FPS")
            .view_aspect(2.0)
            .allow_zoom(false)
            .custom_x_axes(x_axes)
            .custom_y_axes(y_axes)
            .show(ui, |ui| ui.line(line))
    });
}

// TODO: UI doesnt work

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Next,
}

// #[derive(AssetCollection, Resource)]
// pub struct CursorFolderCollection {
//     #[asset(path = "cursor_png", collection(typed))]
//     pub folder: Vec<Handle<Image>>,
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
