use bevy::prelude::*;

use crate::voxel_world::chunk::{self, Chunk};

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
    query: Query<&Chunk>
) {
    if debug_mode_data.chunk_gizmos && debug_mode_data.in_debug_mode {
        for chunk in query.iter() {
            for (voxel, position, size) in chunk.octree.voxel_iterator() {
                let mut pos = chunk.octree_to_world(position);
                
                let world_size = chunk.octree_size_to_world(size);

                pos.x += world_size / 2.;
                pos.y += world_size / 2.;
                pos.z += world_size / 2.;

                debug_gizmos.cuboid(
                    Transform::from_translation(pos).with_scale(Vec3::splat(world_size)),
                    Color::RED,
                );
            }
        }
    }
}