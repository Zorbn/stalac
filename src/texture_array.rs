use crate::texture;
use std::num::NonZeroU32;

pub struct TextureArray {
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl TextureArray {
    pub fn new(device: &wgpu::Device, textures: Vec<texture::Texture>) -> Result<Self, String> {
        let texture_count = textures.len();
        if texture_count < 1 {
            return Err("Attempted to create a texture array without any textures!".to_string());
        }

        let bind_group_layout: wgpu::BindGroupLayout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: NonZeroU32::new(texture_count as u32),
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: NonZeroU32::new(texture_count as u32),
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let mut texture_views = Vec::new();
        let mut texture_samplers = Vec::new();
        for i in 0..textures.len() {
            texture_views.push(textures[i].view());
            texture_samplers.push(textures[i].sampler());
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(texture_views.as_slice()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::SamplerArray(texture_samplers.as_slice()),
                },
            ],
            label: Some("Texture array"),
        });

        Ok(Self {
            bind_group_layout,
            bind_group,
        })
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}