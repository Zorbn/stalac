mod a_star;
mod bytes;
mod chunk;
mod direction;
mod entities;
mod gfx;
mod input;
mod ray;
mod rng;
mod simulation;

use std::time::Instant;

use crate::simulation::Simulation;
use gfx::renderer::Renderer;
use input::Input;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 480.0;

pub async fn run() {
    let event_loop = EventLoop::new();
    let window_rect = get_window_rect(&event_loop);
    let window = WindowBuilder::new()
        .with_position(window_rect.0)
        .with_inner_size(window_rect.1)
        .build(&event_loop)
        .expect("Failed to create window!");

    let mut input = Input::new();
    let mut simulation = Simulation::new();
    let mut renderer = Renderer::new(window).await;
    let mut last_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == renderer.window().id() && !input.process_button(event) => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                renderer.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                renderer.resize(**new_inner_size);
            }
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == renderer.window().id() => {
            let current_time = Instant::now();
            let delta_time = (current_time - last_frame_time).as_secs_f32();
            // println!("{}", 1.0 / delta_time);
            last_frame_time = current_time;

            simulation.update(&mut input, delta_time);
            renderer.update(&mut input, &mut simulation);
            input.update();

            match renderer.render(&simulation) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size()),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(_) => {}
            }
        }
        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta: (x, y) },
            ..
        } => input.process_mouse_motion(x as f32, y as f32),
        Event::MainEventsCleared => {
            renderer.window().request_redraw();
        }
        _ => {}
    });
}

fn get_window_rect(event_loop: &EventLoop<()>) -> (LogicalPosition<f32>, LogicalSize<f32>) {
    let monitor = event_loop.primary_monitor().expect("No primary monitor!");
    let monitor_position = monitor.position().to_logical::<f32>(1.0);
    let monitor_size = monitor.size().to_logical::<f32>(1.0);
    let window_size = LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    let window_position: LogicalPosition<f32> = LogicalPosition {
        x: monitor_position.x + monitor_size.width * 0.5 - window_size.width * 0.5,
        y: monitor_position.y + monitor_size.height * 0.5 - window_size.height * 0.5,
    };

    (window_position, window_size)
}
