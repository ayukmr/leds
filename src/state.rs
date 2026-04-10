use crate::camera::Camera;
use crate::internals::Internals;
use crate::lights::Lights;

use std::sync::Arc;
use winit::window::Window;

pub struct State {
    internals: Internals,
    camera: Camera,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Self {
        let lights = Lights::default();
        let camera = Camera::default();

        let internals = Internals::new(window, &camera, &lights).await;

        Self { internals, camera }
    }

    pub fn render(&self) {
        self.internals.render();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.internals.resize(width, height);
        self.update_camera();
    }

    pub fn move_camera(&mut self, dyaw: f32, ddist: f32) {
        self.camera.yaw += dyaw * 0.25;
        self.camera.dist = f32::max(1.0, self.camera.dist + ddist * self.camera.dist * 0.05);
        self.update_camera();
    }

    pub fn update_camera(&self) {
        self.internals.update_camera(&self.camera);
    }

    pub fn update_lights(&mut self, colors: Vec<[f64; 3]>) {
        self.internals.update_lights(&Lights::from(colors));
    }
}
