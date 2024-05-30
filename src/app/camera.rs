use nalgebra::{Const, Matrix4, OPoint, Vector3};
use bytemuck::{Pod, Zeroable};

use crate::config::Config;

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
            far: 3000.0,
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
    pub spherical_position: SphericalCoordinate,   
    pub projection: T,
    camera_uniform: wgpu::Buffer,
    pub orbit_speed: f32,
    pub zoom_speed: f32,

}

pub struct SphericalCoordinate {
    pub r: f32,
    pub theta: f32,
    pub phi: f32,
}

impl SphericalCoordinate {
    pub fn to_cartesian(&self) -> OPoint<f32, Const<3>> {
        nalgebra::Point3::new(
            self.r * self.phi.sin() * self.theta.cos(),
            self.r * self.phi.cos(),
            self.r * self.phi.sin() * self.theta.sin(),
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
}

impl<T: Projection + Default> Camera<T> {
    pub fn new(device: &wgpu::Device, config: &Config) -> Self {
        let camera_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let spherical_position = SphericalCoordinate {
            r: 500.0,
            theta: 0.0,
            phi: 20.0,
        };

        Self {
            spherical_position,
            projection: T::default(),
            camera_uniform,
            orbit_speed: config.sim_config.orbit_speed,
            zoom_speed: config.sim_config.zoom_speed,
        }
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        let eye = self.spherical_position.to_cartesian();
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

    pub fn orbit_horizontal(&mut self, d_theta: f32) {
        self.spherical_position.theta += d_theta;
    }

    pub fn orbit_vertical(&mut self, d_phi: f32) {
        self.spherical_position.phi += d_phi;
    }

    pub fn zoom(&mut self, d_r: f32) {
        self.spherical_position.r += d_r;
    }
}