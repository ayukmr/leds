use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

pub struct Camera {
    pub yaw: f32,
    pub dist: f32,
}

impl Camera {
    pub fn mvp(&self, width: u32, height: u32) -> CameraData {
        let aspect = width as f32 / height as f32;
        let target = Vec3::new(0.65, 0.6, -0.7);

        let off = Mat4::from_rotation_y(self.yaw).transform_vector3(Vec3::Z * self.dist);

        let pos = target + off;

        let v = Mat4::look_at_rh(pos, target, Vec3::Y);
        let p = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect, 0.1, 100.0);

        CameraData {
            mvp: (p * v).to_cols_array_2d(),
            pos: pos.to_array(),
            _pad: 0.0,
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self { yaw: 0.0, dist: 4.0 }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CameraData {
    mvp: [[f32; 4]; 4],
    pos: [f32; 3],
    _pad: f32,
}
