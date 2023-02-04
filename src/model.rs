use crate::{instance::Instance, vertex::Vertex};
use wgpu::{util::DeviceExt, RenderPass};

pub struct Model {
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub instances: wgpu::Buffer,
    pub num_indices: u32,
    pub num_instances: u32,
}

impl Model {
    pub fn new(device: &wgpu::Device, vertex_array: &[Vertex], index_array: &[u16]) -> Self {
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertex_array),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(index_array),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = index_array.len() as u32;

        let instances = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            vertices,
            indices,
            instances,
            num_indices,
            num_instances: 0,
        }
    }

    pub fn update_instances(&mut self, device: &wgpu::Device, instances: &Vec<Instance>) {
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        self.instances = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.num_instances = instances.len() as u32;
    }

    pub fn vertices(&self) -> &wgpu::Buffer {
        &self.vertices
    }

    pub fn indices(&self) -> &wgpu::Buffer {
        &self.indices
    }

    pub fn instances(&self) -> &wgpu::Buffer {
        &self.instances
    }

    pub fn num_indices(&self) -> u32 {
        self.num_indices
    }

    pub fn num_instances(&self) -> u32 {
        self.num_instances
    }
}
