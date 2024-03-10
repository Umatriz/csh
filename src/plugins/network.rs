use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::prelude::*;

use bevy_inspector_egui::{bevy_egui::EguiContexts, egui};
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    renet::{
        transport::{
            ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
            ServerAuthentication, ServerConfig,
        },
        ConnectionConfig, RenetClient, RenetServer,
    },
    RenetChannelsExt, RepliconRenetPlugins,
};

use crate::{GameState, WindowContext};

use super::player::{Player, PlayerBundle, PlayerColor};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuContext>()
            .add_event::<NetworkSpawnStep>()
            .add_systems(Update, show_menu.run_if(in_state(GameState::Menu)))
            .add_systems(
                Update,
                server_event_system.run_if(resource_exists::<RenetServer>),
            );
    }
}

#[derive(Resource)]
pub struct LocalPlayer(pub ClientId);

#[derive(Event)]
pub struct NetworkSpawnStep(pub ClientId);

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
    channels: Res<RepliconChannels>,
    mut game_state: ResMut<NextState<GameState>>,
    mut event: EventWriter<NetworkSpawnStep>,
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
                        let server_channels_config = channels.get_server_configs();
                        let client_channels_config = channels.get_client_configs();

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

                        event.send(NetworkSpawnStep(ClientId::SERVER));

                        commands.spawn(PlayerBundle {
                            player: Player(ClientId::SERVER),
                            replication: Replication,
                            transform: Transform::from_xyz(0.0, 0.0, 0.0),
                            color: PlayerColor(Color::GREEN),
                            ..Default::default()
                        });

                        // let entity = commands
                        //     .spawn(CursorBundle {
                        //         cursor: Cursor(ClientId::SERVER),
                        //         color: CursorColor(Color::BLACK),
                        //         transform: Transform::default(),
                        //         replication: Replication,
                        //     })
                        //     .id();

                        commands.insert_resource(LocalPlayer(ClientId::SERVER))
                    }
                    AppKind::Client { ip, port } => {
                        let server_channels_config = channels.get_server_configs();
                        let client_channels_config = channels.get_client_configs();

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

                        commands.insert_resource(LocalPlayer(ClientId::new(client_id)));
                    }
                }
                game_state.set(GameState::Game)
            }
        });
}

fn server_event_system(mut commands: Commands, mut server_event: EventReader<ServerEvent>) {
    for event in server_event.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("player: {client_id:?} Connected");
                // Generate pseudo random color from client id.
                let r = ((client_id.get() % 23) as f32) / 23.0;
                let g = ((client_id.get() % 27) as f32) / 27.0;
                let b = ((client_id.get() % 39) as f32) / 39.0;

                commands.spawn(PlayerBundle {
                    player: Player(*client_id),
                    replication: Replication,
                    color: PlayerColor(Color::rgb(r, g, b)),
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    ..Default::default()
                });

                // let entity = commands
                //     .spawn(CursorBundle {
                //         cursor: Cursor(*client_id),
                //         color: CursorColor(Color::rgb(r, g, b)),
                //         transform: Transform::default(),
                //         replication: Replication,
                //     })
                //     .id();
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("client {client_id:?} disconnected: {reason}");
            }
        }
    }
}
