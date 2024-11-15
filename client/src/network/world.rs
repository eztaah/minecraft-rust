use bevy::prelude::*;
use bevy_renet::renet::{DefaultChannel, RenetClient};
use bincode::Options;
use shared::{
    messages::ServerToClientMessage,
    world::{block_to_chunk_coord, chunk_in_radius},
};

use crate::{
    player::Player,
    world::{RenderDistance, WorldRenderRequestUpdateEvent},
};

use super::api::send_network_action;

pub fn update_world_from_network(
    client: &mut ResMut<RenetClient>,
    world: &mut ResMut<crate::world::WorldMap>,
    ev_render: &mut EventWriter<WorldRenderRequestUpdateEvent>,
    player_pos: Query<&Transform, With<Player>>,
    render_distance: Res<RenderDistance>,
) {
    let player_pos = player_pos.single();
    let player_pos = IVec3::new(
        block_to_chunk_coord(player_pos.translation.x as i32),
        0,
        block_to_chunk_coord(player_pos.translation.z as i32),
    );
    let r = render_distance.distance as i32;

    while let Some(bytes) = client.receive_message(DefaultChannel::ReliableUnordered) {
        let msg = bincode::options()
            .deserialize::<ServerToClientMessage>(&bytes)
            .unwrap();
        if let ServerToClientMessage::WorldUpdate(world_update) = msg {
            println!(
                "Received world update, {} chunks received",
                world_update.new_map.len()
            );

            println!("Chunks positions : {:?}", world_update.new_map.keys());

            for (pos, chunk) in world_update.new_map {
                // If the chunk is not in render distance range or is empty, do not consider it
                if !chunk_in_radius(&player_pos, &pos, r) || chunk.map.is_empty() {
                    continue;
                }

                let chunk = crate::world::Chunk {
                    map: chunk.map,
                    entity: {
                        if let Some(c) = world.map.get(&pos) {
                            c.entity
                        } else {
                            None
                        }
                    },
                };

                world.map.insert(pos, chunk);
                ev_render.send(WorldRenderRequestUpdateEvent::ChunkToReload(pos));
            }
        }
    }
}

pub fn request_world_update(
    client: &mut ResMut<RenetClient>,
    requested_chunks: Vec<IVec3>,
    render_distance: &RenderDistance,
    player_chunk_pos: IVec3,
) {
    send_network_action(
        client,
        super::api::NetworkAction::WorldUpdateRequest {
            requested_chunks,
            player_chunk_pos,
            render_distance: render_distance.distance,
        },
    );
}
