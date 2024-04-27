use bevy::prelude::*;

use crate::{player::{FreeViewMovment, Player}, voxel_world::chunk::{self, octree::Voxel, Chunk, CHUNK_SIZE}};

use super::DebugModeData;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct DebugGizmos {}

pub fn chunk_gizmos_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_mode_data: ResMut<DebugModeData>,
) {
    if keys.just_pressed(KeyCode::KeyG) && debug_mode_data.in_debug_mode {
        if debug_mode_data.chunk_gizmos {
            debug_mode_data.chunk_gizmos = false;
        } else {
            debug_mode_data.chunk_gizmos = true;
        }
    }
}

pub fn draw_octree_borders(
    debug_mode_data: ResMut<DebugModeData>,
    mut debug_gizmos: Gizmos<DebugGizmos>,
    query: Query<&Chunk>,
    player_query: Query<&Transform, With<FreeViewMovment>>
) {
    let transform = player_query.single();
    if debug_mode_data.chunk_gizmos && debug_mode_data.in_debug_mode {
        for chunk in query.iter() {
            if chunk::coords_to_chunk_pos(transform.translation) == chunk.position {
                for (voxel, position, size) in chunk.octree.voxel_iterator() {
                    let mut pos = chunk::octree_to_world(chunk.octree.size, chunk.position, position);
                
                    let world_size = chunk.octree.relative_size(size) * chunk::CHUNK_SIZE;
    
                    pos.x += world_size / 2.;
                    pos.y += world_size / 2.;
                    pos.z += world_size / 2.;
    
                    debug_gizmos.cuboid(
                        Transform::from_translation(pos).with_scale(Vec3::splat(world_size)),
                        Color::RED,
                    );
                }
            }

            let mut pos = chunk::chunk_pos_to_coords(chunk.position);
            pos.x += CHUNK_SIZE /2.;
            pos.y += CHUNK_SIZE /2.;
            pos.z += CHUNK_SIZE /2.;
            
            debug_gizmos.cuboid(
                Transform::from_translation(pos).with_scale(Vec3::splat(CHUNK_SIZE)),
                Color::GREEN,
            );
        }
    }
}