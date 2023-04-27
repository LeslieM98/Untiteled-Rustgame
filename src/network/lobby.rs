use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_renet::renet::{DefaultChannel, RenetServer};
use serde::{Deserialize, Serialize};

use crate::network::server::MAX_CONNECTIONS;
use crate::network::server::{ClientConnectedEvent, ClientDisconnectedEvent};

use super::packet_communication::{Packet, PacketMetaData, PacketType, ReceivedMessages, Sender};
use super::remote_player::{spawn_remote_player, PlayerID};

type SpawnFunction = Box<dyn Fn(&mut Commands, u64) -> Entity + Send + Sync>;

#[derive(Default)]
pub struct LobbyClientPlugin;

impl Plugin for LobbyClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Lobby::default())
            .add_system(client_apply_sync);
    }
}

#[derive(Default)]
pub struct LobbyServerPlugin;

impl Plugin for LobbyServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Lobby::default())
            .add_system(server_client_connected)
            .add_system(server_client_disconnected)
            .add_system(
                server_send_sync
                    .after(server_client_connected)
                    .after(server_client_disconnected)
                    .run_if(|lobby: Res<Lobby>| lobby.is_changed()),
            );
    }
}

#[derive(Resource, Default)]
pub struct Lobby {
    connected_clients: HashMap<u64, Entity>,
}

impl Lobby {
    pub fn get_map(&self) -> &HashMap<u64, Entity> {
        &self.connected_clients
    }

    pub fn generate_sync_packet(&self) -> LobbySync {
        let mut lobby_sync = LobbySync::default();
        for (i, (id, _)) in self.connected_clients.iter().enumerate() {
            lobby_sync.connected_clients[i] = Some(*id);
        }
        return lobby_sync;
    }

    fn disconnect_clients(&mut self, sync: &LobbySync) -> Vec<Entity> {
        let mut disconnected_entities = Vec::new();
        let mut disconnected_ids = Vec::new();
        for (id, entity) in &self.connected_clients {
            if sync.connected_clients.contains(&Some(*id)) {
                disconnected_entities.push(*entity);
                disconnected_ids.push(*id);
            }
        }
        for id in disconnected_ids {
            self.connected_clients.remove(&id);
        }
        return disconnected_entities;
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct LobbySync {
    pub connected_clients: [Option<u64>; MAX_CONNECTIONS],
}

impl PacketMetaData for LobbySync {
    fn get_packet_type() -> PacketType {
        PacketType::LobbySync
    }

    fn get_content_size(&self) -> u128 {
        bincode::serialized_size(self).unwrap().into()
    }
}
fn server_client_connected(
    mut lobby: ResMut<Lobby>,
    mut client_connected_events: EventReader<ClientConnectedEvent>,
    mut commands: Commands,
) {
    for event in client_connected_events.iter() {
        let entity = commands
            .spawn(TransformBundle::default())
            .insert(PlayerID::new(event.id))
            .id();
        lobby.connected_clients.insert(event.id, entity);
    }
}

fn server_client_disconnected(
    mut lobby: ResMut<Lobby>,
    mut client_disconnected_events: EventReader<ClientDisconnectedEvent>,
    mut commands: Commands,
) {
    for event in client_disconnected_events.iter() {
        let entity = lobby.connected_clients.get(&event.id);
        if let Some(client_entity) = entity {
            commands.entity(*client_entity).despawn();
        }
        lobby.connected_clients.remove(&event.id);
    }
}

fn server_send_sync(lobby: Res<Lobby>, mut server: ResMut<RenetServer>) {
    let sync_packet = Packet::new(&lobby.generate_sync_packet(), Sender::Server);
    let serialized = bincode::serialize(&sync_packet).unwrap();

    server.broadcast_message(DefaultChannel::Reliable, serialized);
}

fn client_apply_sync(
    mut lobby: ResMut<Lobby>,
    recv_messages: Res<ReceivedMessages>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let syncs = recv_messages.deserialize::<LobbySync>();
    for (_, sync) in syncs {
        for entity in lobby.disconnect_clients(&sync).iter() {
            commands.entity(*entity).despawn();
        }
        for client in sync.connected_clients {
            if let Some(id) = client {
                if !lobby.connected_clients.contains_key(&id) {
                    let player_model = asset_server.load("glTF/base model/base_model.gltf#Scene0");
                    lobby
                        .connected_clients
                        .insert(id, spawn_remote_player(&mut commands, id, player_model));
                }
            }
        }
    }
}
