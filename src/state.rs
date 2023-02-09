use crate::chunk::Chunk;
use crate::ecs::actor::{Actor, ActorSystem};
use crate::ecs::chase_ai::{ChaseAi, ChaseAiSystem};
use crate::ecs::display::Display;
use crate::ecs::ecs::{EntityManager, SystemManager};
use crate::ecs::entity_instances_system::EntityInstancesSystem;
use crate::ecs::player::{Player, PlayerMovementSystem};
use crate::gfx::camera::{Camera, CameraPerspectiveProjection};
use crate::gfx::instance::InstanceRaw;
use crate::gfx::model::Model;
use crate::gfx::sprite_mesh::{SPRITE_VERTICES, SPRITE_INDICES};
use crate::gfx::texture::{Texture, self};
use crate::gfx::texture_array::TextureArray;
use crate::gfx::vertex::Vertex;
use crate::input::Input;
use crate::rng::Rng;
use cgmath::prelude::*;
use std::borrow::BorrowMut;
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
    ecs: EntityManager,
    systems: SystemManager,
    player: usize,
    entity_cache: Vec<usize>,
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
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let happy_tree = Texture::from_path(&device, &queue, "happy-tree.png").unwrap();
        let sad_tree = Texture::from_path(&device, &queue, "sad-tree.png").unwrap();
        let diffuse_texture_array = TextureArray::new(&device, vec![happy_tree, sad_tree]).unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gfx/shader.wgsl").into()),
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

        let model = Model::new(&device, SPRITE_VERTICES, SPRITE_INDICES);

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

        let mut player_actor = Actor::new(
            cgmath::Vector3::zero(),
            cgmath::Vector3::new(0.5, 0.8, 0.5),
            6.0,
        );

        if let Some(player_spawn) = chunk.get_spawn_position(&mut rng) {
            player_actor.teleport(player_spawn);
        }

        let mut enemy_actor = Actor::new(
            cgmath::Vector3::zero(),
            cgmath::Vector3::new(0.5, 0.8, 0.5),
            6.0,
        );

        if let Some(enemy_spawn) = chunk.get_spawn_position(&mut rng) {
            enemy_actor.teleport(enemy_spawn);
        }

        let mut ecs = EntityManager::new();
        let player = ecs.add_entity();
        ecs.add_component_to_entity(player, player_actor);
        ecs.add_component_to_entity(player, Player {});
        let enemy = ecs.add_entity();
        ecs.add_component_to_entity(enemy, enemy_actor);
        ecs.add_component_to_entity(enemy, ChaseAi::new());
        ecs.add_component_to_entity(enemy, Display::new(1));

        let mut systems = SystemManager::new();
        systems.add_system(ActorSystem {});
        systems.add_system(ChaseAiSystem {});
        systems.add_system(PlayerMovementSystem {});
        systems.add_system(EntityInstancesSystem::new());

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
            ecs,
            systems,
            player,
            entity_cache: Vec::new(),
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

        self.systems.update(
            &mut self.ecs,
            &mut self.entity_cache,
            &self.chunk,
            &mut self.input,
            self.player,
            delta_time,
        );

        if let Some(player) = self
            .ecs
            .borrow_components::<Actor>()
            .unwrap()
            .borrow_mut()
            .get(self.player)
        {
            self.camera.rotate(player.look_x(), player.look_y());
            self.camera.teleport(player.head_position());
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

            let entity_instances_system = self.systems.get::<EntityInstancesSystem>().unwrap();

            self.model
                .update_instances(&self.device, entity_instances_system.instances());
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
