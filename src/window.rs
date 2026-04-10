use crate::state::State;

use std::sync::Arc;
use std::sync::mpsc::Receiver;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
    rx: Receiver<Vec<[f64; 3]>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes().with_title("leds");
        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        self.window = Some(window.clone());
        self.state = Some(pollster::block_on(State::new(window)));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let mut colors = None;

        while let Ok(cs) = self.rx.try_recv() {
            colors = Some(cs);
        }

        if let Some(cs) = colors && let Some(state) = self.state.as_mut() {
            state.update_lights(cs);
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(size) => {
                 if let Some(s) = self.state.as_mut() {
                     s.resize(size.width, size.height);
                 }
            }
            WindowEvent::RedrawRequested => {
                if let Some(s) = self.state.as_mut() {
                    s.render();
                }
                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(key),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                let Some(s) = self.state.as_mut() else {
                    return;
                };

                match key {
                    KeyCode::KeyA => s.move_camera(-1.0, 0.0),
                    KeyCode::KeyD => s.move_camera(1.0, 0.0),
                    _ => {}
                }
            }

            WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, amt), .. } => {
                if let Some(s) = self.state.as_mut() {
                    s.move_camera(0.0, -amt);
                }
            }

            _ => (),
        }
    }
}

pub fn start(rx: Receiver<Vec<[f64; 3]>>) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App { rx, window: None, state: None };
    event_loop.run_app(&mut app).unwrap();
}
