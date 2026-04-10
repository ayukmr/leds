use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
pub struct Light {
    pos: [f32; 3],
    yaw: f32,
    color: [f32; 3],
    _pad: f32,
}

impl Light {
    pub fn new(pos: [f32; 3], yaw: f32, color: [f32; 3]) -> Self {
        Self { pos, yaw, color, _pad: 0.0 }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Lights {
    lights: [Light; 50],
    len: u32,
    _pad: [u32; 7],
}

const FRONT_BACK_X: f32 = 0.49;
const RIGHT_X: f32 = 1.02;

const STRIP_Y: f32 = 0.85;

const FRONT_Z: f32 = -0.945;
const RIGHT_Z: f32 = FRONT_Z - 0.015;
const BACK_Z: f32 = -1.29;

const LED_WIDTH: f32 = 2.6;
const LED_SPACING: f32 = 1.1;
const LED_SCALE: f32 = 0.01;
const LED_PITCH: f32 = LED_WIDTH * LED_SPACING * LED_SCALE;

const FRONT_COUNT: usize = 19;
const RIGHT_COUNT: usize = 12;

impl Lights {
    pub fn from(colors: Vec<[f64; 3]>) -> Self {
        let lights: [Light; 50] = std::array::from_fn(|i| {
            let color = colors[i];

            let (pos, yaw) = if i < FRONT_COUNT {
                (
                    [FRONT_BACK_X + LED_PITCH * i as f32, STRIP_Y, FRONT_Z],
                    0.0,
                )
            } else if i < FRONT_COUNT + RIGHT_COUNT {
                let ii = (i - FRONT_COUNT) as f32;
                (
                    [RIGHT_X, STRIP_Y, RIGHT_Z - LED_PITCH * ii],
                    std::f32::consts::FRAC_PI_2,
                )
            } else {
                let ii = (FRONT_COUNT - (i - (FRONT_COUNT + RIGHT_COUNT)) - 1) as f32;
                (
                    [FRONT_BACK_X + LED_PITCH * ii, STRIP_Y, BACK_Z],
                    std::f32::consts::PI,
                )
            };

            Light::new(pos, yaw, [color[0] as f32, color[1] as f32, color[2] as f32])
        });

        Self { lights, len: 50, _pad: [0; 7] }
    }

    pub fn len() -> u32 {
        50
    }
}

impl Default for Lights {
    fn default() -> Self {
        Self { lights: [Light::default(); 50], len: 50, _pad: [0; 7] }
    }
}
