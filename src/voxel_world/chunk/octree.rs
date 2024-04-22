use std::{array::from_fn, thread::current};

use bevy::math::I64Vec3;

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone, Copy)]
pub enum OctreeError {
    SizeLargerThanOctree,
    NotAVoxel,
    NotAnEdge,
    TooSmallToBeSplit,
}

#[derive(Debug, Clone)]
pub struct Octree {
    size: u8,
    content: OctreeContent,

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
        (2 as u64).pow(self.size as u32)
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

        //Code incompréhensible pour incrémenter en cartésiennes selon un shema de Morton sans convertir
        let mut dif = self.pos.0 & self.pos.1 & self.pos.2;
        dif = (dif + (1 << deepest_cube.size)) ^ dif; // Tous les bits qui seront modifiés par l'addition de la puissance e.
        let mut p = (dif + (1 << deepest_cube.size) ) >> 1; // Puissance d'arrêt de retenue        
        let mut i = (self.pos.0 & p) | ((self.pos.1 & p) << 1) | ((self.pos.2 & p) << 2);
        i += p; // Nouveau code pour la profondeur d'arrêt de retenue
        self.pos.0 &= !dif; 
        self.pos.0 |= i & p; // Mise à jour des nouvelles positions
        self.pos.1 &= !dif; 
        self.pos.1 |= (i >> 1) & p;
        self.pos.2 &= !dif; 
        self.pos.2 |= (i >> 2) & p;

        let voxel = match deepest_cube.content {
            OctreeContent::Childs(_) => panic!("Deepest cube can't have childs"),
            OctreeContent::Voxel(voxel) => voxel,
        };

        Some((voxel, previous_pos, deepest_cube.size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

}