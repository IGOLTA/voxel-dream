pub mod chunk;
pub mod map;

use bevy::prelude::*;

use self::chunk::*;

pub struct VoxelWorld;

impl Plugin for VoxelWorld {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, generate_basic_chunk)
            .add_systems(Update, chunk_generation_system);
    }
}

fn generate_basic_chunk (
    mut commands: Commands,
) {
    commands.spawn(ChunkBundle::default());
}