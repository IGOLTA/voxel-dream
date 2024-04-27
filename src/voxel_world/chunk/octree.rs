use std::{array::from_fn, future};

use bevy::{render::{mesh::{Indices, Mesh, PrimitiveTopology}, render_asset::RenderAssetUsages}, utils::futures};
use ::futures::future::join_all;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OctreeContent {
    Childs([Box<Octree>; 8]),
    Voxel(Voxel)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Voxel {
    Empty,
    Dirt,
    Stone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OctreePosition(pub u64, pub u64, pub u64); 

impl OctreePosition {
    fn get_child_indice(&self, size: u8) -> usize {
        (((self.0 >> size) & 1) + (((self.1 >> size) & 1) << 1) + (((self.2 >> size) & 1) << 2)).try_into().unwrap() // entre 0 et 7
    }

    fn morton_increment(&mut self, size: u8) {
        //Code incompréhensible pour incrémenter en cartésiennes selon un shema de Morton sans convertir
        let mut dif = self.0 & self.1 & self.2;
        dif = (dif + (1 << size)) ^ dif; // Tous les bits qui seront modifiés par l'addition de la puissance e.
        let mut p = (dif + (1 << size) ) >> 1; // Puissance d'arrêt de retenue        
        let mut i = (self.0 & p) | ((self.1 & p) << 1) | ((self.2 & p) << 2);
        i += p; // Nouveau code pour la profondeur d'arrêt de retenue
        self.0 &= !dif; 
        self.0 |= i & p; // Mise à jour des nouvelles positions
        self.1 &= !dif; 
        self.1 |= (i >> 1) & p;
        self.2 &= !dif; 
        self.2 |= (i >> 2) & p;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OctreeError {
    SizeLargerThanOctree,
    NotAVoxel,
    NotAnEdge,
    TooSmallToBeSplit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Octree {
    pub size: u8,
    pub content: OctreeContent,

}

impl Octree {
    //Can panic if size>63
    pub fn new(size: u8, voxel: Option<Voxel>) -> Self {
        if size > 63 {
            panic!("Max size is 63");
        }
        Self {
            size,
            content: OctreeContent::Voxel(voxel.unwrap_or(Voxel::Empty))
        }
    }

    pub fn voxel_iterator<'a>(&'a self) -> OctreeIterator<'a> {
        OctreeIterator::new(self)
    }

    pub fn get_voxel(& self, pos: OctreePosition) -> Voxel {
        let cube = self.get_cube(pos, 0).unwrap();

        if let OctreeContent::Voxel(voxel) = cube.content {
            voxel
            
        } else {
            panic!("Get cube at size 0 did not return a voxel");
        }
    }

    pub fn get_cube(& self, pos: OctreePosition, min_size: u8) -> Result<& Octree, OctreeError> {
        if min_size > self.size {
            return Err(OctreeError::SizeLargerThanOctree);
        }
        
        let mut current_size = self.size;
        let mut current_cube = self;
        while min_size < current_size {
            if let OctreeContent::Childs(ref childs) = current_cube.content {
                current_size -= 1;
                current_cube = childs[pos.get_child_indice(current_size)].as_ref();
            } else {
                break;
            }
        }

        Ok(current_cube)
    }

    pub fn get_cube_mut(&mut self, pos: OctreePosition, min_size: u8)-> Result<& mut Octree, OctreeError> {
        if min_size > self.size {
            return Err(OctreeError::SizeLargerThanOctree);
        }
        
        let mut current_size = self.size;
        let mut current_cube = self;
        while min_size < current_size {
            if let OctreeContent::Childs(ref mut childs) = current_cube.content {
                current_size -= 1;
                current_cube = childs[pos.get_child_indice(current_size)].as_mut();
            } else {
                break;
            }
        }

        Ok(current_cube)
    } 

    pub fn split(& mut self) -> Result<(), OctreeError>{
        if self.size == 0 {
            return Err(OctreeError::TooSmallToBeSplit);
        }
        if let OctreeContent::Voxel(voxel) = self.content {
            let new_childs: [Box<Octree>; 8] = from_fn(|_i| Box::new(Octree::new(self.size - 1, Some(voxel))));
            self.content = OctreeContent::Childs(new_childs);

            Ok(())
        } else {
            Err(OctreeError::NotAVoxel)
        }
    }

    pub fn set_voxel(&mut self, pos: OctreePosition, size: u8, voxel: Voxel) -> Result<(), OctreeError> {
        let cube = self.get_cube_mut(pos, size)?;
        
        match cube.content {
            OctreeContent::Childs(_) => {
                cube.content = OctreeContent::Voxel(voxel);
            }
            OctreeContent::Voxel(_) => {
                match size.cmp(&cube.size) {
                    std::cmp::Ordering::Less => {
                        cube.split().unwrap();
                        cube.set_voxel(pos, size, voxel)?;
                    },
                    std::cmp::Ordering::Equal => {
                        cube.content = OctreeContent::Voxel(voxel);
                    }
                    std::cmp::Ordering::Greater => panic!("The fond cube size cannot be grater that size since it is a voxel"),
                }
            }
        }
        
        Ok(())
    }

    pub fn cart_size(&self) -> u64 {
        Self::octree_size_to_cartestian(self.size)
    }

    pub fn octree_size_to_cartestian(size: u8) -> u64 {
        (2 as u64).pow(size as u32)
    }

    pub fn optimize(& mut self) {
        let mut pos = OctreePosition(0, 0, 0);
        let limit =  self.cart_size();
        while pos.0 < limit && pos.1 < limit && pos.2 < limit {
            let voxel;
            let deepest_cube_size;

            {
                let deepest_cube = self.get_cube(pos, 0).unwrap();

                voxel = match deepest_cube.content {
                    OctreeContent::Childs(_) => panic!("Deepest cube can't have childs"),
                    OctreeContent::Voxel(voxel) => voxel,
                };
                deepest_cube_size = deepest_cube.size;
            }
            
            if let Ok(mut parent) = self.get_cube_mut(pos, deepest_cube_size + 1) {
                let mut all_childs_same_voxel = true;
                let childs = match parent.content {
                    OctreeContent::Childs(ref mut childs) => childs,
                    OctreeContent::Voxel(voxel) => panic!("Parents have childs"),
                };
    
                for child in childs {
                    if let OctreeContent::Voxel(child_voxel) = child.content {
                        if child_voxel != voxel {
                            all_childs_same_voxel = false
                        }
                    } else {
                        all_childs_same_voxel = false;
                    }
                } 

                if all_childs_same_voxel {
                    parent.content = OctreeContent::Voxel(voxel);
                    continue;
                }
            }

            pos.morton_increment(deepest_cube_size);       

        }
    }

    pub async fn fill_with_heigh_map(& mut self, heigh_map: Vec<Vec<i128>>, block_size: u8) {
        let size_delta = self.size - block_size;

        assert_eq!(heigh_map.len(), Octree::octree_size_to_cartestian(size_delta) as usize, "Heigh map octree size and block size are not matching");
        assert_eq!(heigh_map[0].len(), Octree::octree_size_to_cartestian(size_delta) as usize, "Heigh map octree size and block size are not matching");

        let (lowest_point_res, highest_point_res)  = Self::generate_res_high_maps(heigh_map, size_delta).await;

        self.content = OctreeContent::Voxel(Voxel::Empty);
        self.apply_res_heigh_map(block_size, OctreePosition(0, 0, 0), &lowest_point_res, &highest_point_res).await;

    }

    async fn generate_res_high_maps(heigh_map: Vec<Vec<i128>>, heigh_map_size: u8) -> (Vec<Vec<Vec<i128>>>, Vec<Vec<Vec<i128>>>) {

        let mut highest_point_res: Vec<Vec<Vec<i128>>> = Vec::with_capacity(heigh_map_size as usize + 1);
        let mut lowest_point_res:  Vec<Vec<Vec<i128>>> = Vec::with_capacity(heigh_map_size as usize + 1);
        
        highest_point_res.push(heigh_map.clone());
        lowest_point_res.push(heigh_map);

        let mut current_len = Octree::octree_size_to_cartestian(heigh_map_size) as usize;

        for i in 0..heigh_map_size as usize {
            current_len /= 2;
            let mut highest_point: Vec<Vec<i128>> = Vec::with_capacity(current_len);
            let mut lowest_point: Vec<Vec<i128>> = Vec::with_capacity(current_len);
            for j in 0..current_len {
                highest_point.push(Vec::with_capacity(current_len));
                lowest_point.push(Vec::with_capacity(current_len));
                for k in 0..current_len {
                    let sub_points = [
                        highest_point_res[i][j * 2][k * 2], 
                        highest_point_res[i][j * 2 + 1][k * 2], 
                        highest_point_res[i][j * 2][k * 2 + 1], 
                        highest_point_res[i][j * 2 + 1][k * 2 + 1]
                        ];
                    highest_point[j].push(*sub_points.iter().max().unwrap()); 

                    let sub_points = [
                        lowest_point_res[i][j * 2][k * 2], 
                        lowest_point_res[i][j * 2 + 1][k * 2], 
                        lowest_point_res[i][j * 2][k * 2 + 1], 
                        lowest_point_res[i][j * 2 + 1][k * 2 + 1]
                        ];
                    lowest_point[j].push(*sub_points.iter().min().unwrap()); 
                }
            }

            highest_point_res.push(highest_point);
            lowest_point_res.push(lowest_point);
        }

        (lowest_point_res, highest_point_res)
    }

    //Can panic if Octree is not empty
    async fn apply_res_heigh_map(&mut self, block_size: u8, pos: OctreePosition, lowest_point_res: &Vec<Vec<Vec<i128>>>, highest_point_res: &Vec<Vec<Vec<i128>>>) {
        if block_size == self.size {
            let height = highest_point_res[0][pos.0 as usize][pos.2 as usize];
            if height > pos.1 as i128 {
                self.content = OctreeContent::Voxel(Voxel::Stone);
            } else if height <= pos.1 as i128  {
                self.content = OctreeContent::Voxel(Voxel::Empty);
            }
        } else {
            let delta_size = self.size - block_size;
            let map_x = pos.0 / (2 as u64).pow(delta_size as u32);
            let map_y = pos.2 / (2 as u64).pow(delta_size as u32);

            let highest_point = highest_point_res[delta_size as usize][map_x as usize][map_y as usize];
            let lowest_point = lowest_point_res[delta_size as usize][map_x as usize][map_y as usize];

            if highest_point <= pos.1 as i128 {
                self.content = OctreeContent::Voxel(Voxel::Empty);
            } else if lowest_point >= pos.1 as i128 + (2 as i128).pow(delta_size as u32) {
                self.content = OctreeContent::Voxel(Voxel::Stone);
            } else {
                self.split().unwrap();

                let mut childs: &mut [Box<Octree>] = match self.content {
                    OctreeContent::Childs(ref mut childs) => childs,
                    OctreeContent::Voxel(_) => panic!("This node has just been splitted but does not have childs"),
                };

                let mut child_pos = pos;
                let mut futures = Vec::with_capacity(8);
                for _ in 0..8 {
                    let (child, remaining_childs) = childs.split_first_mut().unwrap();
                    childs =  remaining_childs;
                    let future = child.apply_res_heigh_map(block_size, child_pos, lowest_point_res, highest_point_res);
                    futures.push(future);
                    child_pos.morton_increment(delta_size - 1);
                }

                join_all(futures).await;
            }
        }
    }

    pub fn relative_size(&self, size: u8) -> f32 {
        Octree::octree_size_to_cartestian(size) as f32 / Octree::octree_size_to_cartestian(self.size) as f32
    }

}


pub struct OctreeIterator<'a> {
    pos: OctreePosition,
    octree: &'a Octree,
    limit: u64
}

impl <'a> OctreeIterator<'a> {
    pub fn new(octree: &'a Octree) -> Self {
        OctreeIterator { 
            pos: OctreePosition(0, 0, 0), 
            octree: octree,
            limit: octree.cart_size()
        }
    }
}

impl<'a> Iterator for OctreeIterator<'a> {
    type Item = (Voxel, OctreePosition, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos.0 >= self.limit || self.pos.1 >= self.limit|| self.pos.2 >= self.limit {
            return None;
        } 

        let deepest_cube = self.octree.get_cube(self.pos, 0).unwrap();
        
        let previous_pos = self.pos;

        self.pos.morton_increment(deepest_cube.size); 

        let voxel = match deepest_cube.content {
            OctreeContent::Childs(_) => panic!("Deepest cube can't have childs"),
            OctreeContent::Voxel(voxel) => voxel,
        };

        Some((voxel, previous_pos, deepest_cube.size))
    }
}

#[cfg(test)]
mod tests {
    use bevy::tasks::block_on;

    use super::*;

    #[test]
    fn morton_increment() {
        let mut pos = OctreePosition(0, 0, 0);

        pos.morton_increment(0);
        assert_eq!(pos, OctreePosition(1, 0, 0));
        pos.morton_increment(0);
        assert_eq!(pos, OctreePosition(0, 1, 0));
        pos.morton_increment(0);
        assert_eq!(pos, OctreePosition(1, 1, 0));
        pos.morton_increment(0);
        assert_eq!(pos, OctreePosition(0, 0, 1));
        pos.morton_increment(0);
        assert_eq!(pos, OctreePosition(1, 0, 1));
        pos.morton_increment(0);
        assert_eq!(pos, OctreePosition(0, 1, 1));
        pos.morton_increment(0);
        assert_eq!(pos, OctreePosition(1, 1, 1));
        pos.morton_increment(0);
        assert_eq!(pos, OctreePosition(2, 0, 0));

        pos.morton_increment(1);
        assert_eq!(pos, OctreePosition(0, 2, 0));
    }

    #[test]
    fn get_child_indice_test() {
        let pos = OctreePosition(2, 5, 1);

        assert_eq!(pos.get_child_indice(7), 0b000);
        assert_eq!(pos.get_child_indice(6), 0b000);
        assert_eq!(pos.get_child_indice(5), 0b000);
        assert_eq!(pos.get_child_indice(4), 0b000);
        assert_eq!(pos.get_child_indice(3), 0b000);
        assert_eq!(pos.get_child_indice(2), 0b010);
        assert_eq!(pos.get_child_indice(1), 0b001);
        assert_eq!(pos.get_child_indice(0), 0b110);
    }

    #[test]
    fn test_octree_equal() {
        let tree_1 = Octree{
            size: 8,
            content: OctreeContent::Childs([
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Stone)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Stone)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Empty)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Empty)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Stone)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Stone)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Empty)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Empty)
                }),
            ]),
        };

        let tree_2 = tree_1.clone();

        assert_eq!(tree_1, tree_2);
    }

    #[test]
    fn basic_octree_creation() {
        let tree = Octree::new(5, Some(Voxel::Empty));

        match tree.content {
            OctreeContent::Voxel(Voxel::Empty) => {
                return;
            }
            _ => {
                panic!("Failed to create a basic octree");
            }
        }
    }

    #[test]
    fn octree_split() {
        let mut tree = Octree::new(8, Some(Voxel::Empty));

        tree.split().unwrap();

        match tree.content {
            OctreeContent::Voxel(_) => {
                panic!("Failed to split octree");
            }
            OctreeContent::Childs(ref childs) => {
                for child in childs {
                    if let OctreeContent::Voxel(voxel) = child.content {
                        if voxel != Voxel::Empty {
                            panic!("Splitted voxel has changed type");
                        }
                    } else {
                        panic!("Failed to split voxel");
                    }
                }
            }
        }
    }

    #[test]
    fn octree_access() {
        let mut tree = Octree::new(8, Some(Voxel::Stone));

        if let OctreeContent::Voxel(voxel) = tree.get_cube(OctreePosition(4, 4, 1), 0).unwrap().content {
            assert_eq!(Voxel::Stone, voxel);
        } else {
            panic!("Failed to access tree");
        }

        assert_eq!(Voxel::Stone, tree.get_voxel(OctreePosition(2, 4, 1)));

        tree.split().unwrap();

        assert_eq!(Voxel::Stone, tree.get_voxel(OctreePosition(2, 7, 1)));
    }

    #[test]
    fn octree_set_complex() {
        let mut tree = Octree::new(8, Some(Voxel::Empty));

        tree.set_voxel(OctreePosition(2, 5, 1), 0, Voxel::Stone).unwrap();


        assert_eq!(Voxel::Stone, tree.get_voxel(OctreePosition(2, 5, 1)));

        assert_eq!(Voxel::Empty, tree.get_voxel(OctreePosition(2, 5, 2)));
        assert_eq!(Voxel::Empty, tree.get_voxel(OctreePosition(3, 5, 1)));
        assert_eq!(Voxel::Empty, tree.get_voxel(OctreePosition(4, 5, 1)));
        assert_eq!(Voxel::Empty, tree.get_voxel(OctreePosition(2, 7, 1)));
        assert_eq!(Voxel::Empty, tree.get_voxel(OctreePosition(2, 0, 1)));

        let mut tree = Octree::new(8, Some(Voxel::Empty));

        tree.set_voxel(OctreePosition(0, 0, 0), 7, Voxel::Stone).unwrap();
        
        assert_eq!(Voxel::Stone, tree.get_voxel(OctreePosition(10, 10, 10)));
    }

    #[test]
    fn octree_iterator() {
        let mut tree = Octree::new(8, Some(Voxel::Empty));

        tree.set_voxel(OctreePosition(2, 5, 1), 0, Voxel::Stone).unwrap();

        let mut len = 0;
        for (voxel, pos, _size) in tree.voxel_iterator() {
            len += 1;
            println!("{:?}", (voxel, pos, _size));
            if voxel == Voxel::Stone {
                assert_eq!(pos, OctreePosition(2, 5, 1));
            }
        }
        
        assert_eq!(len, 7 * 7 + 8);
    }

    #[test]
    fn res_high_map_generatrion() {
        let heigh_map: Vec<Vec<i128>> = vec![
            vec![0, 0, 1, 2, 2, 2, 2, 2],
            vec![1, 0, 2, 0, 1, 1, 1, 3],
            vec![2, 2, 2, 1, 3, 4, 2, 5],
            vec![0, 1, 1, 1, 1, 0, 1, 4],
            vec![0, 1, 2, 0, 0, 0, -1, 2],
            vec![3, 2, 2, 2, 1, 0, 2, 0],
            vec![1, 1, 1, 1, 1, 1, 1, 1],
            vec![2, 3, 2, 1, 0, 1, 3, 4],
        ];

        let (lowest_point_res, highest_point_res) = block_on(Octree::generate_res_high_maps(heigh_map.clone(), 3));

        println!("Highest point: {highest_point_res:?}");
        assert_eq!(highest_point_res, vec![
            heigh_map.clone(),
            vec![
                vec![1, 2, 2, 3],
                vec![2, 2, 4, 5],
                vec![3, 2, 1, 2],
                vec![3, 2, 1, 4]
            ],
            vec![
                vec![2, 5],
                vec![3, 4],
            ],
            vec![vec![5]]
        ]);
        
        println!("Lowest point: {lowest_point_res:?}");
        assert_eq!(lowest_point_res, vec![
            heigh_map.clone(),
            vec![
                vec![0, 0, 1, 1],
                vec![0, 1, 0, 1],
                vec![0, 0, 0, -1],
                vec![1, 1, 0, 1]
            ],
            vec![
                vec![0, 0],
                vec![0, -1],
            ],
            vec![vec![-1]]
        ]);
    }

    #[test]
    fn res_hight_map_application_chunk_too_high() {
        let mut tree = Octree::new(8, None);

        let heigh_map: Vec<Vec<i128>> = vec![
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0]
        ];

        let (lowest_point_res, highest_point_res) = block_on(Octree::generate_res_high_maps(heigh_map, 2));

        block_on(tree.apply_res_heigh_map(6, OctreePosition(0, 0, 0), &highest_point_res, &lowest_point_res));

        match tree.content {
            OctreeContent::Childs(_) => panic!(),
            OctreeContent::Voxel(voxel) => assert_eq!(voxel, Voxel::Empty),
        }
    }

    #[test]
    fn res_hight_map_application_chunk_under_ground() {
        let mut tree = Octree::new(8, None);

        let heigh_map: Vec<Vec<i128>> = vec![
            vec![4, 4, 4, 4],
            vec![4, 4, 4, 4],
            vec![4, 4, 4, 4],
            vec![4, 4, 4, 4],
        ];

        let (lowest_point_res, highest_point_res) = block_on(Octree::generate_res_high_maps(heigh_map.clone(), 2));

        block_on(tree.apply_res_heigh_map(6, OctreePosition(0, 0, 0), &highest_point_res, &lowest_point_res));

        match tree.content {
            OctreeContent::Childs(_) => panic!(),
            OctreeContent::Voxel(voxel) => assert_ne!(voxel, Voxel::Empty),
        }
    }
    
    #[test]
    fn res_hight_map_application() {
        let mut test_tree = Octree::new(8, None);

        let heigh_map: Vec<Vec<i128>> = vec![
            vec![1, 2],
            vec![0, 0],
        ];

        let (lowest_point_res, highest_point_res) = block_on(Octree::generate_res_high_maps(heigh_map.clone(), 1));

        block_on(test_tree.apply_res_heigh_map(7, OctreePosition(0, 0, 0), &lowest_point_res, &highest_point_res));
        
        let mut result_tree = Octree{
            size: 8,
            content: OctreeContent::Childs([
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Stone)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Empty)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Empty)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Empty)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Stone)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Empty)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Stone)
                }),
                Box::new(Octree { 
                    size: 7, 
                    content: OctreeContent::Voxel(Voxel::Empty)
                }),
            ]),
        };

        assert_eq!(result_tree, test_tree);
    }
}