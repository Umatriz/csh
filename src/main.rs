#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};
use bevy_inspector_egui::{
    bevy_egui::{EguiContexts, EguiPlugin},
    egui,
};
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_replicon::{
    client::ClientSet,
    network_event::{
        client_event::{ClientEventAppExt, FromClient},
        EventType,
    },
    renet::{
        transport::{
            ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
            ServerAuthentication, ServerConfig,
        },
        ClientId, ConnectionConfig, RenetClient, RenetServer, ServerEvent,
    },
    replicon_core::{
        replication_rules::{AppReplicationExt, Replication},
        NetworkChannels,
    },
    server::{has_authority, ServerPlugin, TickPolicy, SERVER_ID},
    ReplicationPlugins,
};

use plugins::{
    camera::CameraPlugin,
    crafting::CraftingPlugin,
    player::{MoveDirection, Player, PlayerBundle, PlayerCollection, PlayerColor, PlayerPlugin},
};

pub mod args;
pub mod logic;
pub mod network;
pub mod plugins;
pub mod utils;

pub use core::stringify;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

fn main() {
    App::new()
        .register_type::<WindowContext>()
        .init_resource::<WindowContext>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(ReplicationPlugins.set(ServerPlugin {
            tick_policy: TickPolicy::MaxTickRate(60),
            ..Default::default()
        }))
        .add_plugins((EguiPlugin, WorldInspectorPlugin::new()))
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(ResourceInspectorPlugin::<WindowContext>::default())
        // .add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
        .add_plugins((
            PlayerPlugin,
            CameraPlugin,
            CraftingPlugin,
            // ChestPlugin
        ))
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
                .load_collection::<PlayerCollection>(), // .load_collection::<CursorFolderCollection>(),
        )
        .init_resource::<MenuContext>()
        .replicate::<Transform>()
        .replicate::<PlayerColor>()
        .replicate::<Player>()
        .add_client_event::<MoveDirection>(EventType::Ordered)
        .add_systems(Update, show_menu.run_if(in_state(GameState::Menu)))
        .add_systems(
            Update,
            (
                movement_system.run_if(has_authority()), // Runs only on the server or a single player.
                server_event_system.run_if(resource_exists::<RenetServer>()), // Runs only on the server.
                input_system,
            ),
        )
        .add_systems(PreUpdate, player_init_system.after(ClientSet::Receive))
        .run()
}

#[derive(Resource, Default, Reflect)]
#[reflect(Default)]
pub struct WindowContext {
    workbench_window: bool,
    inventory_window: bool,
    enchantment_window: bool,
    add_item_window: bool,
    menu_window: bool,
}

#[derive(Resource)]
pub struct LocalPlayer(pub ClientId);

const PORT: u16 = 5000;
const PROTOCOL_ID: u64 = 0;

#[derive(Debug, PartialEq)]
enum AppKind {
    Server { ip: IpAddr, port: u16 },
    Client { ip: [u8; 4], port: u16 },
}

impl Default for AppKind {
    fn default() -> Self {
        Self::Server {
            ip: Ipv4Addr::LOCALHOST.into(),
            port: 5000,
        }
    }
}

#[derive(Resource, Default)]
struct MenuContext {
    selected: AppKind,
}

fn show_menu(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut menu_context: ResMut<MenuContext>,
    mut window_context: ResMut<WindowContext>,
    network_channels: Res<NetworkChannels>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    egui::Window::new("Lobby")
        .open(&mut window_context.menu_window)
        .show(contexts.ctx_mut(), |ui| {
            egui::ComboBox::from_label("Select app kind")
                .selected_text(format!("{:?}", menu_context.selected))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut menu_context.selected,
                        AppKind::Client {
                            ip: Ipv4Addr::LOCALHOST.octets(),
                            port: PORT,
                        },
                        "Client",
                    );
                    ui.selectable_value(
                        &mut menu_context.selected,
                        AppKind::Server {
                            ip: Ipv4Addr::LOCALHOST.into(),
                            port: PORT,
                        },
                        "Server",
                    );
                });

            match &mut menu_context.selected {
                AppKind::Server { port, ip } => {
                    ui.label(format!("Your Ip: {}", ip));
                    egui::ComboBox::from_label("Select server IP")
                        .selected_text(format!("{:?}", ip))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(ip, Ipv4Addr::LOCALHOST.into(), "Local host");
                        });
                    ui.add(egui::DragValue::new(port));
                }
                AppKind::Client { ip, port } => {
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut ip[0]));
                        ui.add(egui::DragValue::new(&mut ip[1]));
                        ui.add(egui::DragValue::new(&mut ip[2]));
                        ui.add(egui::DragValue::new(&mut ip[3]));
                    });
                    ui.add(egui::DragValue::new(port));
                    ui.label(format!("Target Ip: {}", IpAddr::from(*ip)));
                    ui.label(format!("Port: {}", *port));
                }
            }

            if ui.button("Play").clicked() {
                match menu_context.selected {
                    AppKind::Server { port, ip } => {
                        let server_channels_config = network_channels.get_server_configs();
                        let client_channels_config = network_channels.get_client_configs();

                        let server = RenetServer::new(ConnectionConfig {
                            server_channels_config,
                            client_channels_config,
                            ..Default::default()
                        });

                        let current_time = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap();
                        let public_addr = SocketAddr::new(ip, port);
                        let socket = UdpSocket::bind(public_addr).unwrap();
                        let server_config = ServerConfig {
                            current_time,
                            max_clients: 10,
                            protocol_id: PROTOCOL_ID,
                            authentication: ServerAuthentication::Unsecure,
                            public_addresses: vec![public_addr],
                        };
                        let transport = NetcodeServerTransport::new(server_config, socket).unwrap();

                        commands.insert_resource(server);
                        commands.insert_resource(transport);

                        commands.spawn(TextBundle::from_section(
                            "Server",
                            TextStyle {
                                font_size: 30.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));

                        commands.spawn(PlayerBundle {
                            player: Player(SERVER_ID),
                            replication: Replication,
                            transform: Transform::from_xyz(0.0, 0.0, 0.0),
                            color: PlayerColor(Color::GREEN),
                            ..Default::default()
                        });

                        commands.insert_resource(LocalPlayer(SERVER_ID))
                    }
                    AppKind::Client { ip, port } => {
                        let server_channels_config = network_channels.get_server_configs();
                        let client_channels_config = network_channels.get_client_configs();

                        let client = RenetClient::new(ConnectionConfig {
                            server_channels_config,
                            client_channels_config,
                            ..Default::default()
                        });

                        let current_time = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap();
                        let client_id = current_time.as_millis() as u64;
                        let ip_addr = ip.into();
                        let server_addr = SocketAddr::new(ip_addr, port);
                        let socket = UdpSocket::bind((ip_addr, 0)).unwrap();
                        let authentication = ClientAuthentication::Unsecure {
                            client_id,
                            protocol_id: PROTOCOL_ID,
                            server_addr,
                            user_data: None,
                        };
                        let transport =
                            NetcodeClientTransport::new(current_time, authentication, socket)
                                .unwrap();

                        commands.insert_resource(client);
                        commands.insert_resource(transport);

                        commands.spawn(TextBundle::from_section(
                            format!("Client: {client_id:?}"),
                            TextStyle {
                                font_size: 30.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));

                        commands.insert_resource(LocalPlayer(ClientId::from_raw(client_id)));
                    }
                }
                game_state.set(GameState::Game)
            }
        });
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

fn server_event_system(mut commands: Commands, mut server_event: EventReader<ServerEvent>) {
    for event in server_event.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("player: {client_id} Connected");
                // Generate pseudo random color from client id.
                let r = ((client_id.raw() % 23) as f32) / 23.0;
                let g = ((client_id.raw() % 27) as f32) / 27.0;
                let b = ((client_id.raw() % 39) as f32) / 39.0;
                commands.spawn(PlayerBundle {
                    player: Player(*client_id),
                    replication: Replication,
                    color: PlayerColor(Color::rgb(r, g, b)),
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    ..Default::default()
                });
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("client {client_id} disconnected: {reason}");
            }
        }
    }
}

/// Reads player inputs and sends [`MoveCommandEvents`]
fn input_system(mut move_events: EventWriter<MoveDirection>, input: Res<Input<KeyCode>>) {
    let mut direction = Vec2::ZERO;
    if input.pressed(KeyCode::Right) {
        direction.x += 1.0;
    }
    if input.pressed(KeyCode::Left) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::Up) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::Down) {
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
        info!("received event {event:?} from client {client_id}");
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
