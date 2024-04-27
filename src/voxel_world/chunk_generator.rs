pub mod world_generator;

use std::ops::Range;
use std::os::unix::thread;

use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use bevy::{math::I64Vec3, utils::HashSet};
use bevy::prelude::*;
use noise::Perlin;

use self::world_generator::WorldGenerator;

use super::chunk::octree::{self, Octree};
use super::chunk::{self, Chunk};

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum ChunkLoadingStatus {
    GenerationRequested(I64Vec3),
    Loaded,
    DestructionRequested,
}

#[derive(Component)]
pub struct ChunkGenerationTask(Task<(Octree, Mesh)>);

pub struct ChunkGeneratorPlugin;

impl Plugin for ChunkGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(WorldGenerator{
                perlin: Perlin::new(65464),
                amplitude: 5.,
                scale: 10.,
                chunk_octree_size: 10,
                world_block_ocree_size: 2,
            })
            .add_systems(Update, (
                chunk_generator_system,
                chunk_generation_system_end_generation,
                chunk_generation_system_start_generation,
                chunk_destroying_system
            ))
        ;
    }
}


#[derive(Component, Debug)]
pub struct ChunkGenerator {
    pub render_cube_size: u32
}

impl ChunkGenerator {
    fn chunk_is_in_loading_radius(&self, player: I64Vec3, chunk_pos: I64Vec3) -> bool {
        let relative_pos = chunk_pos - player;
        let relative_pos_range = self.relative_pos_range();

        relative_pos_range.contains(&relative_pos.x) && relative_pos_range.contains(&relative_pos.y) && relative_pos_range.contains(&relative_pos.z)
    }

    fn relative_pos_range(&self) -> Range<i64> {
        (-(self.render_cube_size as i64))/2..((self.render_cube_size as i64)/2 + self.render_cube_size as i64 % 2)
    }
}


fn chunk_generator_system(
    mut commands: Commands,
    mut chunks_query: Query<(Entity, &Chunk, & mut ChunkLoadingStatus)>,
    mut in_generation_query: Query<(Entity, & mut ChunkLoadingStatus), (With<ChunkGenerationTask>, Without<Chunk>)>,
    generator_query: Query<(&ChunkGenerator, &Transform)>,
) {
    for (generator, transform) in generator_query.iter() {
        let mut to_load = HashSet::<I64Vec3>::with_capacity(generator.render_cube_size.pow(3) as usize);
        let player_pos = chunk::coords_to_chunk_pos(transform.translation);

        for i in generator.relative_pos_range() {
            for j in generator.relative_pos_range() {
                for k in generator.relative_pos_range() {
                    to_load.insert(
                        I64Vec3::new(i, j, k) + 
                        chunk::coords_to_chunk_pos(transform.translation)
                    );
                }
            }
        }

        for (_entity, chunk, mut chunk_loading_status) in chunks_query.iter_mut() {
            if !generator.chunk_is_in_loading_radius(player_pos, chunk.position) {
                match *chunk_loading_status {
                    ChunkLoadingStatus::Loaded => {
                        *chunk_loading_status = ChunkLoadingStatus::DestructionRequested;
                    }
                    _ => {}
                }
            } else {
                to_load.remove(&chunk.position);
            }
        }

        for (_entity, mut chunk_loading_status) in in_generation_query.iter_mut() {
            if let ChunkLoadingStatus::GenerationRequested(pos) = *chunk_loading_status {
                if !generator.chunk_is_in_loading_radius(player_pos, pos) {
                    *chunk_loading_status = ChunkLoadingStatus::DestructionRequested;
                }else {
                    to_load.remove(&pos);
                }
            } 
        }
  
        for pos in to_load {
            commands.spawn(ChunkLoadingStatus::GenerationRequested(pos));
        }
    };

}


fn chunk_generation_system_start_generation (
    mut commands: Commands,
    generation_requested_chunks_query: Query<(Entity, &ChunkLoadingStatus), Without<ChunkGenerationTask>>,
    world_generator: Res<WorldGenerator>
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (entity, status) in generation_requested_chunks_query.iter() {
        if let ChunkLoadingStatus::GenerationRequested(pos) = *status {
            let generator: WorldGenerator = world_generator.clone();

            let task = thread_pool.spawn(async move {
            
                let octree = chunk::generate_octree(pos, &generator).await;
                let mesh = chunk::generate_mesh(&octree).await;
                
                (
                    octree,
                    mesh
                )
            });
            commands.entity(entity).insert(ChunkGenerationTask(task));
        }
    }
}

fn chunk_generation_system_end_generation (
    mut commands: Commands,
    mut in_generation_chunks_query: Query<(Entity, &mut ChunkLoadingStatus, &mut ChunkGenerationTask)>,
    mut mesh_assets_res: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    for (entity, mut status, mut task) in in_generation_chunks_query.iter_mut() {
        match *status {
            ChunkLoadingStatus::GenerationRequested(pos) => {
                if let Some((tree, mesh)) = block_on(poll_once(&mut task.0)) {
                    
                    let mesh_handle = mesh_assets_res.add(mesh);
                    
                    commands.entity(entity).insert((
                        Chunk {
                            octree: tree,
                            position: pos,
                            mesh: mesh_handle.clone(),
                        },
                        PbrBundle {
                            transform: Transform::from_translation(chunk::chunk_pos_to_coords(pos)),
                            mesh: mesh_handle.clone_weak(),
                            material: materials.add(Color::rgb_u8(124, 144, 255)),
                            ..default()
                        },
                
                    ));
                    *status = ChunkLoadingStatus::Loaded;
                    commands.entity(entity).remove::<ChunkGenerationTask>();
                }
            },
            _=>{}
        }
    }
}

fn chunk_destroying_system(
    mut commands: Commands,
    chunks_query: Query<(Entity, &ChunkLoadingStatus), With<Chunk>>,
) {
    for (entity, status) in chunks_query.iter() {
        if status == &ChunkLoadingStatus::DestructionRequested {
            commands.entity(entity).despawn_recursive();
        }
    }
}

