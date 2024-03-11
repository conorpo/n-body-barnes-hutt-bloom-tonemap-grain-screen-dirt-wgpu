use nalgebra::Vector3;
use bytemuck::{Pod, Zeroable};
use wgpu::core::device;

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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Star {
    position: [f32; 3],
}

pub struct Galaxy {
    stars: Vec<Star>,
    stars_buffer: wgpu::Buffer,
    //bhot: BHOT,
}

use rand::Rng;
use rand_distr::StandardNormal;

fn gaussian_random() -> f32 {
    let mut rng = rand::thread_rng();
    
}

impl Galaxy {
    pub fn new(device: &wgpu::Device, config: &crate::config::Config) -> Self {
        let star_count = config.sim_config.star_count as usize;

        let mut stars = Vec::with_capacity(star_count);
        let mut rng = rand::thread_rng();

        for _ in 0..star_count {
            let arm: u32 = rng.gen::<u32>() % config.sim_config.arm_count;
            let r = rng.gen::<f32>() * config.sim_config.galaxy_radius;
            let _starting_theta = ((arm as f32) / (config.sim_config.arm_count as f32)) * 2.0 * std::f32::consts::PI;
            let theta = _starting_theta + r * config.sim_config.spiralness;

            let random_offset: (f32, f32, f32) = (rng.sample(StandardNormal), rng.sample(StandardNormal), rng.sample(StandardNormal));

            
            let x = r * theta.cos() + random_offset.0 * config.sim_config.noise_scale;
            let y = r * theta.sin() + random_offset.1 * config.sim_config.noise_scale;
            let z = random_offset.2 * config.sim_config.noise_scale;

            stars.push(Star {
                position: [x, y, z],
            });
        }

        todo!()
    }
}


impl Star {
    const ATTRIBS : [wgpu::VertexAttribute; 1] = 
        wgpu::vertex_attr_array![0 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn new(position: [f32; 3]) -> Self {
        Self {
            position,
        }
    }
}

