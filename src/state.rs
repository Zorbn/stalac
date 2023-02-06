use crate::camera::{Camera, CameraPerspectiveProjection};
use crate::chase_ai::ChaseAi;
use crate::chunk::Chunk;
use crate::entities::Entities;
use crate::entity::Entity;
use crate::input::Input;
use crate::instance::{Instance, InstanceRaw};
use crate::model::Model;
use crate::player_ai::PlayerAi;
use crate::rng::Rng;
use crate::sprite_mesh::{SPRITE_INDICES, SPRITE_VERTICES};
use crate::texture::{self, Texture};
use crate::texture_array::TextureArray;
use crate::vertex::Vertex;
use cgmath::prelude::*;
use std::iter::once;
use std::time::{SystemTime, UNIX_EPOCH};
use wgpu::Features;
use winit::dpi::PhysicalSize;
use winit::event::{KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
use winit::window::{Fullscreen, Window};

/*
 * TODO:
 * Orthographic camera/ ui
 * For collisions, consider building a grid of buckets that entities can be placed in:
 * Entities are stored in a hashmap with an id,
 * As entities move, they are placed into buckets based on their approximate position and removed from the one they used to be in if necessary,
 * To find what entities to check against for collisions an entity can get all the entity ids in the 3x3 of buckets surrounding them, and find
 * the entities they need based on their ids.
 */

pub struct State {
    window: Window,
    input: Input,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    diffuse_texture_array: TextureArray,
    depth_texture: Texture,
    camera: Camera,
    model: Model,
    chunk: Chunk,
    entities: Entities,
    player_id: u32,
}

impl State {
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
            .filter(|f| f.describe().srgb)
            .next()
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
        let diffuse_texture_array = TextureArray::new(&device, vec![happy_tree, sad_tree]).unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera = Camera::new(
            &device,
            CameraPerspectiveProjection::new(
                cgmath::Vector3::zero(),
                90.0,
                0.1,
                100.0,
                config.width,
                config.height,
            ),
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &diffuse_texture_array.bind_group_layout(),
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

        let mut model = Model::new(&device, SPRITE_VERTICES, SPRITE_INDICES);
        let instances = vec![
            Instance {
                position: cgmath::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                rotation: cgmath::Quaternion::zero(),
            },
            Instance {
                position: cgmath::Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: -1.0,
                },
                rotation: cgmath::Quaternion::zero(),
            },
            Instance {
                position: cgmath::Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: 10.0,
                },
                rotation: cgmath::Quaternion::zero(),
            },
        ];
        model.update_instances(&device, &instances);

        let input = Input::new();

        let mut rng = Rng::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Inaccurate system time!")
                .as_millis() as u32,
        );
        let mut chunk = Chunk::new();
        chunk.generate_blocks(&mut rng);
        chunk.generate_mesh(&device);

        let player_ai = Box::new(PlayerAi {});
        let mut player = Entity::new(
            cgmath::Vector3::zero(),
            cgmath::Vector3::new(0.5, 0.8, 0.5),
            6.0,
            Some(player_ai),
        );

        if let Some(player_spawn) = chunk.get_spawn_position(&mut rng) {
            player.actor.teleport(player_spawn);
        }

        let chase_ai = Box::new(ChaseAi::new());
        let mut enemy = Entity::new(
            cgmath::Vector3::zero(),
            cgmath::Vector3::new(0.5, 0.8, 0.5),
            6.0,
            Some(chase_ai),
        );

        if let Some(enemy_spawn) = chunk.get_spawn_position(&mut rng) {
            enemy.actor.teleport(enemy_spawn);
        }

        let mut entities = Entities::new();
        let player_id = entities.insert(player, true);
        entities.insert(enemy, false);

        Self {
            window,
            input,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            diffuse_texture_array,
            depth_texture,
            camera,
            model,
            chunk,
            entities,
            player_id,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
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
        self.depth_texture = Texture::create_depth_texture(
            &self.device,
            self.config.width,
            self.config.height,
            "depth_texture",
        );
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                self.input.key_state_changed(*keycode, *state);
                true
            }
            _ => false,
        }
    }

    pub fn mouse_motion(&mut self, delta_x: f32, delta_y: f32) {
        self.input.mouse_moved(delta_x, delta_y);
    }

    pub fn mouse_input(&mut self, button: MouseButton) {
        self.input.mouse_state_changed(button);
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.input.was_mouse_button_pressed(MouseButton::Left, true) {
            self.input.set_focused(&self.window, true);
        }

        if self.input.was_key_pressed(VirtualKeyCode::Escape) {
            self.input.set_focused(&self.window, false);
        }

        if self.input.was_key_pressed(VirtualKeyCode::F11) {
            if self.window.fullscreen().is_none() {
                self.window
                    .set_fullscreen(Some(Fullscreen::Borderless(None)));
            } else {
                self.window.set_fullscreen(None)
            }
        }

        let player_position = match self.entities.get(self.player_id) {
            Some(player) => player.actor.position(),
            _ => cgmath::Vector3::zero(),
        };

        self.entities
            .update(&mut self.input, player_position, &self.chunk, delta_time);

        if let Some(player) = self.entities.get(self.player_id) {
            self.camera
                .rotate(player.actor.look_x(), player.actor.look_y());
            self.camera.teleport(player.actor.head_position());
        }

        self.camera.update(&self.queue);

        self.input.update();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                    view: &self.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.diffuse_texture_array.bind_group(), &[]);
            render_pass.set_bind_group(1, self.camera.bind_group(), &[]);

            if let Some(model) = &self.chunk.model {
                render_pass.set_vertex_buffer(0, model.vertices().slice(..));
                render_pass.set_index_buffer(model.indices().slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_vertex_buffer(1, model.instances().slice(..));
                render_pass.draw_indexed(0..model.num_indices(), 0, 0..model.num_instances());
            }

            self.model
                .update_instances(&self.device, &self.entities.instances());
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
        output.present();

        Ok(())
    }
}
