
use std::{
    net::{Ipv4Addr, UdpSocket},
    time::SystemTime,
};

use bevy::prelude::*;

use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    renet::{
        transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
        ConnectionConfig, RenetServer,
    },
    RenetChannelsExt,
};

pub use mmo_game_shared::{components::*,};

pub(crate) struct MmoGameNodePlugin;

impl Plugin for MmoGameNodePlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<PlayerPosition>()
            .replicate::<PlayerColor>()
            .add_client_event::<MoveDirection>(ChannelKind::Ordered)
            .add_systems(Startup, Self::listen)
            .add_systems(
                Update,
                (
                    Self::apply_movement.run_if(has_authority), // Runs only on the server or a single player.
                    Self::handle_connections.run_if(server_running), // Runs only on the server.
                ),
            );
    }
}

impl MmoGameNodePlugin {
    fn listen(
        mut commands: Commands,
        channels: Res<RepliconChannels>,
    ) {
        let server_channels_config = channels.get_server_configs();
        let client_channels_config = channels.get_client_configs();

        let server = RenetServer::new(ConnectionConfig {
            server_channels_config,
            client_channels_config,
            ..Default::default()
        });

        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("Time went backwards");
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 5000)).expect("Failed to bind socket");
        let server_config = ServerConfig {
            current_time,
            max_clients: 10,
            protocol_id: PROTOCOL_ID,
            authentication: ServerAuthentication::Unsecure,
            public_addresses: Default::default(),
        };
        let transport = NetcodeServerTransport::new(server_config, socket).expect("Failed to create server transport");

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
        // commands.spawn(PlayerBundle::new(
        //     ClientId::SERVER,
        //     Vec2::ZERO,
        //     GREEN.into(),
        // ));
    }

    /// Logs server events and spawns a new player whenever a client connects.
    fn handle_connections(mut commands: Commands, mut server_events: EventReader<ServerEvent>) {
        for event in server_events.read() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    println!("{client_id:?} connected");
                    // Generate pseudo random color from client id.
                    let r = ((client_id.get() % 23) as f32) / 23.0;
                    let g = ((client_id.get() % 27) as f32) / 27.0;
                    let b = ((client_id.get() % 39) as f32) / 39.0;
                    commands.spawn(PlayerBundle::new(
                        *client_id,
                        Vec2::ZERO,
                        Color::srgb(r, g, b),
                    ));
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    info!("{client_id:?} disconnected: {reason}");
                }
            }
        }
    }

    /// Mutates [`PlayerPosition`] based on [`MoveDirection`] events.
    ///
    /// Fast-paced games usually you don't want to wait until server send a position back because of the latency.
    /// But this example just demonstrates simple replication concept.
    fn apply_movement(
        time: Res<Time>,
        mut move_events: EventReader<FromClient<MoveDirection>>,
        mut players: Query<(&Player, &mut PlayerPosition)>,
    ) {
        const MOVE_SPEED: f32 = 200.0;
        for FromClient { client_id, event } in move_events.read() {
            info!("received event {event:?} from {client_id:?}");
            for (player, mut position) in &mut players {
                if *client_id == player.client_id {
                    **position += event.direction * time.delta_seconds() * MOVE_SPEED;
                }
            }
        }
    }
}

const PROTOCOL_ID: u64 = 0;

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    position: PlayerPosition,
    color: PlayerColor,
    replicated: Replicated,
}

impl PlayerBundle {
    fn new(client_id: ClientId, position: Vec2, color: Color) -> Self {
        Self {
            player: Player{client_id},
            position: PlayerPosition{position},
            color: PlayerColor{color},
            replicated: Replicated,
        }
    }
}