pub mod octree;

use bevy::{math::{f32, I64Vec3}, prelude::*};

use self::octree::{Octree, OctreePosition, Voxel};

pub const CHUNK_SIZE: f32 = 10.;

#[derive(Bundle, Default)]
pub struct  ChunkBundle {
    pub chunk: Chunk,
}

#[derive(Component)]
pub struct Chunk {
    pub octree: Octree,
    pub position: I64Vec3,
}

impl Default for Chunk {
    fn default() -> Self {
        Self { 
            octree: Octree::new(32, None),
            position: I64Vec3::ZERO
        }

    }
}

impl Chunk {
    pub fn get_world_position(&self) -> Vec3 {
        self.position.as_vec3() * CHUNK_SIZE
    }

    pub fn octree_size_to_world(&self, octree_size: u8) -> f32{
        CHUNK_SIZE * (2 as u64).pow(octree_size as u32) as f32 / self.octree.cart_size() as f32
    }

    pub fn octree_to_world(&self, octree_pos: OctreePosition,) -> Vec3 {
        let mut x = self.get_world_position().x;
        let mut y = self.get_world_position().y;
        let mut z = self.get_world_position().z;

        x += octree_pos.0 as f32 / self.octree.cart_size() as f32;
        y += octree_pos.1 as f32 / self.octree.cart_size() as f32;
        z += octree_pos.2 as f32 / self.octree.cart_size() as f32;

        return Vec3::new(x, y, z) * CHUNK_SIZE;
    }
}

pub fn chunk_generation_system(
    mut query: Query<
        (Entity, &mut Chunk),
        Added<Chunk>
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (entity, mut chunk) in query.iter_mut() {
        chunk.octree.set_voxel(OctreePosition(0, 2, 6), 0, Voxel::Empty);
    }
}