use {
    super::Star,
    crate::nalgebra::*,
}; 

//BHOT = Barnes-Hut Oct-Tree
struct BHOTNode  {
    indirection_index: usize,
    leaf: bool,
    total_mass: f32,
    center_of_mass: Vector3<f32>,
}

struct BHOT {
    root: BHOTNode,
    nodes: Vec<BHOTNode>,
}

pub struct Galaxy {
    stars: Vec<Star>,
    bhot: BHOT,
}


