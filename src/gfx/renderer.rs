use crate::entities::actor::Actor;
use crate::gfx::camera::{Camera, CameraOrthographicProjection, CameraPerspectiveProjection};
use crate::gfx::instance::InstanceRaw;
use crate::gfx::model::Model;
use crate::gfx::sprite_mesh::{SPRITE_INDICES, SPRITE_VERTICES, UI_SPRITE_VERTICES};
use crate::gfx::texture::{self, Texture};
use crate::gfx::texture_array::TextureArray;
use crate::gfx::vertex::Vertex;
use crate::input::Input;
use crate::simulation::Simulation;
use cgmath::prelude::*;
use std::borrow::Borrow;
use std::iter::once;
use wgpu::Features;
use winit::dpi::PhysicalSize;
use winit::event::{MouseButton, VirtualKeyCode};
use winit::window::{Fullscreen, Window};

const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 100.0;
const UI_SCALE: f32 = 28.0;

pub struct Renderer {
    window: Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    ui_render_pipeline: wgpu::RenderPipeline,
    texture_array: TextureArray,
    ui_texture_array: TextureArray,
    depth_texture: Texture,
    camera: Camera,
    ui_camera: Camera,
    model: Model,
    ui_model: Model,
}

impl Renderer {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: Default::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: Features::TEXTURE_BINDING_ARRAY | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.describe().srgb)
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let happy_tree = Texture::from_path(&device, &queue, "happy-tree.png").unwrap();
        let sad_tree = Texture::from_path(&device, &queue, "sad-tree.png").unwrap();
        let texture_array = TextureArray::new(&device, vec![happy_tree, sad_tree]).unwrap();
        let glyphs = Texture::from_path(&device, &queue, "bitka.png").unwrap();
        let ui_texture_array = TextureArray::new(&device, vec![glyphs]).unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let ui_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ui_shader.wgsl").into()),
        });

        let camera = Camera::new(
            &device,
            Box::new(CameraPerspectiveProjection::new(
                cgmath::Vector3::zero(),
                90.0,
                Z_NEAR,
                Z_FAR,
                config.width,
                config.height,
            )),
        );

        let ui_camera = Camera::new(
            &device,
            Box::new(CameraOrthographicProjection::new(
                Z_NEAR,
                Z_FAR,
                config.width,
                config.height,
                UI_SCALE,
            )),
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    texture_array.bind_group_layout(),
                    camera.bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });

        let depth_texture =
            Texture::create_depth_texture(&device, config.width, config.height, "depth_texture");

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let ui_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    ui_texture_array.bind_group_layout(),
                    camera.bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });

        let ui_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&ui_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &ui_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &ui_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let model = Model::new(&device, SPRITE_VERTICES, SPRITE_INDICES);
        let ui_model = Model::new(&device, UI_SPRITE_VERTICES, SPRITE_INDICES);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            ui_render_pipeline,
            texture_array,
            ui_texture_array,
            depth_texture,
            camera,
            ui_camera,
            model,
            ui_model,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width < 1 || new_size.height < 1 {
            return;
        }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.camera.resize(self.config.width, self.config.height);
        self.ui_camera.resize(self.config.width, self.config.height);
        self.depth_texture = Texture::create_depth_texture(
            &self.device,
            self.config.width,
            self.config.height,
            "depth_texture",
        );
    }

    pub fn update(&mut self, input: &mut Input, simulation: &mut Simulation) {
        if input.was_mouse_button_pressed_ignore_focus(MouseButton::Left) {
            input.set_focused(&self.window, true);
        }

        if input.was_key_pressed(VirtualKeyCode::Escape) {
            input.set_focused(&self.window, false);
        }

        if input.was_key_pressed(VirtualKeyCode::F11) {
            if self.window.fullscreen().is_none() {
                self.window
                    .set_fullscreen(Some(Fullscreen::Borderless(None)));
            } else {
                self.window.set_fullscreen(None)
            }
        }

        input.update_gui_mouse_position(UI_SCALE, self.config.height);

        if let Some(player) = simulation
            .ecs()
            .manager
            .borrow_components::<Actor>()
            .unwrap()
            .borrow()
            .get(simulation.focused_entity())
        {
            self.camera.rotate(player.look_x(), player.look_y());
            self.camera.teleport(player.head_position());
        }

        self.camera.update(&self.queue);
        self.ui_camera.update(&self.queue);

        simulation.chunk.update_mesh(&self.device);
    }

    pub fn render(&mut self, simulation: &Simulation) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.texture_array.bind_group(), &[]);
            render_pass.set_bind_group(1, self.camera.bind_group(), &[]);

            if let Some(model) = simulation.chunk.model() {
                render_pass.set_vertex_buffer(0, model.vertices().slice(..));
                render_pass.set_index_buffer(model.indices().slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_vertex_buffer(1, model.instances().slice(..));
                render_pass.draw_indexed(0..model.num_indices(), 0, 0..model.num_instances());
            }

            self.model
                .update_instances(&self.device, simulation.entity_instances());
            render_pass.set_vertex_buffer(0, self.model.vertices().slice(..));
            render_pass.set_index_buffer(self.model.indices().slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_vertex_buffer(1, self.model.instances().slice(..));
            render_pass.draw_indexed(
                0..self.model.num_indices(),
                0,
                0..self.model.num_instances(),
            );
        }

        self.queue.submit(once(encoder.finish()));

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.ui_render_pipeline);
            render_pass.set_bind_group(0, self.ui_texture_array.bind_group(), &[]);
            render_pass.set_bind_group(1, self.ui_camera.bind_group(), &[]);

            self.ui_model
                .update_instances(&self.device, simulation.gui_instances());
            render_pass.set_vertex_buffer(0, self.ui_model.vertices().slice(..));
            render_pass
                .set_index_buffer(self.ui_model.indices().slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_vertex_buffer(1, self.ui_model.instances().slice(..));
            render_pass.draw_indexed(
                0..self.ui_model.num_indices(),
                0,
                0..self.ui_model.num_instances(),
            );
        }

        self.queue.submit(once(encoder.finish()));

        output.present();

        Ok(())
    }
}
