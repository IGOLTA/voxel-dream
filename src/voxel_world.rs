pub mod chunk;
pub mod chunk_generator;

use bevy::prelude::*;

use self::chunk_generator::ChunkGeneratorPlugin;

pub struct VoxelWorld;

impl Plugin for VoxelWorld {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ChunkGeneratorPlugin);
    }
}
