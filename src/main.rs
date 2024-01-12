use bevy::prelude::*;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use plugins::{camera::CameraPlugin, crafting::CraftingPlugin, player::PlayerPlugin};

pub mod lazy_eq;
pub mod plugins;
pub mod utils;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((EguiPlugin, WorldInspectorPlugin::new()))
        .add_plugins((PlayerPlugin, CameraPlugin, CraftingPlugin))
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Next), // .load_collection::<CursorFolderCollection>(),
        )
        // .add_systems(Startup, setup.run_if(in_state(GameState::Next)))
        .run()
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
