use nalgebra::Vector3;
use bytemuck::{Pod, Zeroable};
use wgpu::{core::device, util::DeviceExt, BufferUsages, Queue};

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
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Star {
    pub position: [f32; 3],
}

#[derive(Debug)]
pub struct Galaxy {
    pub stars: Vec<Star>,
    pub stars_buffer: wgpu::Buffer,
    //bhot: BHOT,
}

use rand::Rng;
use rand_distr::StandardNormal;

use crate::config::Config;

impl Galaxy {
    pub fn new(device: &wgpu::Device, config: &crate::config::Config) -> Self {
        let star_count = 0;

        let mut stars = Vec::with_capacity(star_count);
        let mut rng = rand::thread_rng();

        for _ in 0..star_count {
            let arm: u32 = rng.gen::<u32>() % config.sim_config.arm_count;
            let _starting_theta = ((arm as f32) / (config.sim_config.arm_count as f32)) * 2.0 * std::f32::consts::PI;

            let spiralness_diff: f32 = rng.sample(StandardNormal);
            let r: f32 = rng.gen::<f32>() * config.sim_config.galaxy_radius;
            let theta: f32 = _starting_theta + r * (config.sim_config.spiralness + spiralness_diff * 0.0005);

            let random_offset: (f32, f32, f32) = (rng.sample(StandardNormal), rng.sample(StandardNormal), rng.sample(StandardNormal));

            let x = r * theta.cos() + random_offset.0 * config.sim_config.noise_scale;
            let y = 0.0;// random_offset.1 * config.sim_config.noise_scale;
            let z = r * theta.sin()  + random_offset.2 * config.sim_config.noise_scale;

            stars.push(Star {
                position: [x, y, z],
            });
        }

        let stars_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Stars Buffer"),
            size: 12 * 10000000,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        Self {
            stars,
            stars_buffer: stars_buffer,
        }
    }

    pub fn add_stars(&mut self, config: &Config, queue: &Queue, count: usize) {
        let mut rng = rand::thread_rng();

        let mut new_stars: Vec<Star> = (0..count).map(|_| {
            let arm: u32 = rng.gen::<u32>() % config.sim_config.arm_count;
            let _starting_theta = ((arm as f32) / (config.sim_config.arm_count as f32)) * 2.0 * std::f32::consts::PI;
    
            let spiralness_diff: f32 = rng.sample(StandardNormal);
            let r: f32 = rng.gen::<f32>() * config.sim_config.galaxy_radius;
            let theta: f32 = _starting_theta + r * (config.sim_config.spiralness + spiralness_diff * 0.0005);
    
            let random_offset: (f32, f32, f32) = (rng.sample(StandardNormal), rng.sample(StandardNormal), rng.sample(StandardNormal));
    
            let x = r * theta.cos() + random_offset.0 * config.sim_config.noise_scale;
            let y = 0.0;// random_offset.1 * config.sim_config.noise_scale;
            let z = r * theta.sin()  + random_offset.2 * config.sim_config.noise_scale;

            Star {
                position: [x, y, z],
            }
        }).collect();
        
        queue.write_buffer(&self.stars_buffer, 12 * self.stars.len() as u64, bytemuck::cast_slice(&new_stars));
        self.stars.append(&mut new_stars);
    }
}


impl Star {
    const ATTRIBS : [wgpu::VertexAttribute; 1] = 
        wgpu::vertex_attr_array![0 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Star>() as wgpu::BufferAddress,
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

