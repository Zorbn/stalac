mod camera;
mod chunk;
mod cube_mesh;
mod direction;
mod input;
mod instance;
mod math;
mod model;
mod state;
mod texture;
mod texture_array;
mod vertex;

use std::time::Instant;

use crate::state::State;
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

    let mut state = State::new(window).await;
    let mut last_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() && !state.input(event) => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                state.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.resize(**new_inner_size);
            }
            WindowEvent::MouseInput { button, .. } => {
                state.mouse_input(*button);
            }
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            let current_time = Instant::now();
            let delta_time = (current_time - last_frame_time).as_secs_f32();
            last_frame_time = current_time;

            state.update(delta_time);
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size()),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(_) => {}
            }
        }
        Event::DeviceEvent { event, .. } => match event {
            DeviceEvent::MouseMotion { delta } => {
                state.mouse_motion(delta.0 as f32, delta.1 as f32);
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            state.window().request_redraw();
        }
        _ => {}
    });
}

fn get_window_rect(event_loop: &EventLoop<()>) -> (LogicalPosition<f32>, LogicalSize<f32>) {
    let monitor_size = event_loop
        .primary_monitor()
        .expect("No primary monitor!")
        .size()
        .to_logical::<f32>(1.0);
    let window_size = LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    let window_position: LogicalPosition<f32> = LogicalPosition {
        x: monitor_size.width * 0.5 - window_size.width * 0.5,
        y: monitor_size.height * 0.5 - window_size.height * 0.5,
    };

    (window_position, window_size)
}
