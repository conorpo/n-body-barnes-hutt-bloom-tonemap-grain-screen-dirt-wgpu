use nalgebra::{Const, Matrix4, Vector3};
use bytemuck::{Pod, Zeroable};

pub struct PerspectiveProjection {
    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
}

impl PerspectiveProjection {
    pub fn new(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            fov,
            aspect_ratio,
            near,
            far,
        }
    }
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self {
            fov: 45.0,
            aspect_ratio: 1.0,
            near: 0.1,
            far: 2000.0,
        }
    }
}

pub trait Projection {
    fn get_projection_matrix(&self) -> Matrix4<f32>; 
}

impl Projection for PerspectiveProjection {
    fn get_projection_matrix(&self) -> Matrix4<f32> {
        let mut proj = nalgebra::Perspective3::new(self.aspect_ratio, self.fov, self.near, self.far);
        proj.into_inner()
    }
}

pub struct Camera<T: Projection + Default> {
    pub position: [f32; 3],    
    pub projection: T,
    camera_uniform: wgpu::Buffer,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
}

impl<T: Projection + Default> Camera<T> {
    pub fn new(device: &wgpu::Device) -> Self {
        let camera_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            position: [0.0, -500.0, -500.0],
            projection: T::default(),
            camera_uniform,
        }
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        let eye = nalgebra::Point3::new(self.position[0], self.position[1], self.position[2]);
        let center = nalgebra::Point3::new(0.0, 0.0, 0.0);
        let up = Vector3::new(0.0, -1.0, 0.0);
        Matrix4::look_at_rh(&eye, &center, &up)
    }

    pub fn update(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.camera_uniform, 0, bytemuck::cast_slice(&[CameraUniform {
            view_matrix: self.get_view_matrix().into(),
            proj_matrix: self.projection.get_projection_matrix().into(),
        }]));
    }

    pub fn get_bindgroup_entry(&self) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { 
                buffer: &self.camera_uniform,
                offset: 0,
                size: None,
            }),
        }
    }

    pub fn get_bindgroup_layout_entry(&self) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
}