pub mod octree;

use bevy::{math::{f32, I64Vec3}, prelude::*, render::{mesh::{Indices, PrimitiveTopology}, render_asset::RenderAssetUsages}};
use self::octree::{Octree, OctreePosition, Voxel};

use super::chunk_generator::world_generator::WorldGenerator;

pub const CHUNK_SIZE: f32 = 10.;

#[derive(Component, Debug)]
pub struct Chunk {
    pub octree: Octree,
    pub position: I64Vec3,
    pub mesh: Handle<Mesh>
}

pub fn octree_to_offset(size: u8, octree_pos: OctreePosition) -> Vec3 {
    Vec3::new(
            octree_pos.0 as f32 / Octree::octree_size_to_cartestian(size) as f32, 
            octree_pos.1 as f32 / Octree::octree_size_to_cartestian(size) as f32, 
            octree_pos.2 as f32 / Octree::octree_size_to_cartestian(size) as f32
        ) 
        * CHUNK_SIZE
}

pub fn octree_to_world(size: u8, position: I64Vec3, octree_pos: OctreePosition) -> Vec3 {
    octree_to_offset(size, octree_pos) + chunk_pos_to_coords(position)
}

pub fn coords_to_chunk_pos(coords: Vec3) -> I64Vec3 {
    I64Vec3 {
        x: (coords.x / CHUNK_SIZE).floor() as i64,
        y: (coords.y / CHUNK_SIZE).floor() as i64,
        z: (coords.z / CHUNK_SIZE).floor() as i64,
    }
}

pub fn chunk_pos_to_coords(position: I64Vec3) -> Vec3 {
    position.as_vec3() * CHUNK_SIZE
}

pub async fn generate_octree(position: I64Vec3, world_generator: &WorldGenerator) -> Octree {
    let mut tree = Octree::new(world_generator.chunk_octree_size, None);

    let delta = tree.size - world_generator.world_block_ocree_size;

    let mut height_map: Vec<Vec<i128>> = Vec::with_capacity(delta as usize);
    
    for i in 0..Octree::octree_size_to_cartestian(delta) as usize {
        height_map.push(Vec::with_capacity(Octree::octree_size_to_cartestian(delta) as usize));
        
        for j in 0..Octree::octree_size_to_cartestian(delta) as usize {
            let map_pos = octree_to_world(delta, position, OctreePosition(i as u64, 0, j as u64)).xz();
            let mut height = world_generator.get_world_height(map_pos) - chunk_pos_to_coords(position).y;
            height /= CHUNK_SIZE;
            height *= Octree::octree_size_to_cartestian(delta) as f32;
            height = height.clamp(0., Octree::octree_size_to_cartestian(delta) as f32);
            
            height_map[i].push(height.floor() as i128);
        }
    }

    tree.fill_with_heigh_map(height_map, world_generator.world_block_ocree_size).await;
    tree
}

pub async fn generate_mesh(tree: &Octree) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32;3]>::new())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0,Vec::<[f32;2]>::new())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL,Vec::<[f32;3]>::new())
    .with_inserted_indices(Indices::U32(Vec::<u32>::new()));
    for (voxel, position, size) in tree.voxel_iterator() {
        if voxel != Voxel::Empty {
            let mut pos = octree_to_offset(tree.size, position);
        
            let world_size = tree.relative_size(size) * CHUNK_SIZE;

            pos.x += world_size / 2.;
            pos.y += world_size / 2.;
            pos.z += world_size / 2.;

            let cube = get_cube_mesh()
            .scaled_by(Vec3::splat(world_size))
            .translated_by(pos);

            mesh.merge(cube);
        }
    }
    mesh
}

pub fn get_cube_mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        // Each array is an [x, y, z] coordinate in local space.
        // Meshes always rotate around their local [0, 0, 0] when a rotation is applied to their Transform.
        // By centering our mesh around the origin, rotating the mesh preserves its center of mass.
        vec![
            // top (facing towards +y)
            [-0.5, 0.5, -0.5], // vertex with index 0
            [0.5, 0.5, -0.5], // vertex with index 1
            [0.5, 0.5, 0.5], // etc. until 23
            [-0.5, 0.5, 0.5],
            // bottom   (-y)
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
            // right    (+x)
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5], // This vertex is at the same position as vertex with index 2, but they'll have different UV and normal
            [0.5, 0.5, -0.5],
            // left     (-x)
            [-0.5, -0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, 0.5, -0.5],
            // back     (+z)
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, -0.5, 0.5],
            // forward  (-z)
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, -0.5, -0.5],
        ],
    )
    // Set-up UV coordinates to point to the upper (V < 0.5), "dirt+grass" part of the texture.
    // Take a look at the custom image (assets/textures/array_texture.png)
    // so the UV coords will make more sense
    // Note: (0.0, 0.0) = Top-Left in UV mapping, (1.0, 1.0) = Bottom-Right in UV mapping
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            // Assigning the UV coords for the top side.
            [0.0, 0.2], [0.0, 0.0], [1.0, 0.0], [1.0, 0.25],
            // Assigning the UV coords for the bottom side.
            [0.0, 0.45], [0.0, 0.25], [1.0, 0.25], [1.0, 0.45],
            // Assigning the UV coords for the right side.
            [1.0, 0.45], [0.0, 0.45], [0.0, 0.2], [1.0, 0.2],
            // Assigning the UV coords for the left side.
            [1.0, 0.45], [0.0, 0.45], [0.0, 0.2], [1.0, 0.2],
            // Assigning the UV coords for the back side.
            [0.0, 0.45], [0.0, 0.2], [1.0, 0.2], [1.0, 0.45],
            // Assigning the UV coords for the forward side.
            [0.0, 0.45], [0.0, 0.2], [1.0, 0.2], [1.0, 0.45],
        ],
    )
    // For meshes with flat shading, normals are orthogonal (pointing out) from the direction of
    // the surface.
    // Normals are required for correct lighting calculations.
    // Each array represents a normalized vector, which length should be equal to 1.0.
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            // Normals for the top side (towards +y)
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            // Normals for the bottom side (towards -y)
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            // Normals for the right side (towards +x)
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            // Normals for the left side (towards -x)
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            // Normals for the back side (towards +z)
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            // Normals for the forward side (towards -z)
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ],
    )
    // Create the triangles out of the 24 vertices we created.
    // To construct a square, we need 2 triangles, therefore 12 triangles in total.
    // To construct a triangle, we need the indices of its 3 defined vertices, adding them one
    // by one, in a counter-clockwise order (relative to the position of the viewer, the order
    // should appear counter-clockwise from the front of the triangle, in this case from outside the cube).
    // Read more about how to correctly build a mesh manually in the Bevy documentation of a Mesh,
    // further examples and the implementation of the built-in shapes.
    .with_inserted_indices(Indices::U32(vec![
        0,3,1 , 1,3,2, // triangles making up the top (+y) facing side.
        4,5,7 , 5,6,7, // bottom (-y)
        8,11,9 , 9,11,10, // right (+x)
        12,13,15 , 13,14,15, // left (-x)
        16,19,17 , 17,19,18, // back (+z)
        20,21,23 , 21,22,23, // forward (-z)
    ]))
}
