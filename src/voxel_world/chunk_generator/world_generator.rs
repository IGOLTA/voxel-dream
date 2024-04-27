use bevy::prelude::*;

use noise::{NoiseFn, Perlin};

#[derive(Resource, Clone)]
pub struct WorldGenerator {
    pub perlin: Perlin,
    pub amplitude: f32,
    pub scale: f32,
    pub chunk_octree_size: u8,
    pub world_block_ocree_size: u8,
}

impl WorldGenerator {
    pub fn get_world_height(& self, mut pos: Vec2) -> f32 {
        pos /= self.scale;
        self.perlin.get([pos.x as f64, pos.y as f64]) as f32 * self.amplitude
    }
}