use crate::bind::{self, BindData};
use crate::camera::Camera;
use crate::lights::Lights;
use crate::obj::{self, Model, Vertex};

use std::sync::Arc;

use glam::Mat4;
use glam::Vec3;
use wgpu::CurrentSurfaceTexture;
use winit::window::Window;

const SHADER: &str = include_str!("shader.wgsl");
const LIGHT_SHADER: &str = include_str!("light.wgsl");

const HEADER: &str = include_str!("header.wgsl");

pub struct Internals {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    pipeline: wgpu::RenderPipeline,
    light_pipeline: wgpu::RenderPipeline,

    cam_bg: BindData,
    lights_bg: BindData,

    depth: wgpu::TextureView,

    robor: Model,
    led: Model,
}

impl Internals {
    pub async fn new(window: Arc<Window>, camera: &Camera, lights: &Lights) -> Self {
        let size = window.inner_size();
        let scale = window.scale_factor();

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await.unwrap();

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: (size.width as f64 * scale) as u32,
            height: (size.width as f64 * scale) as u32,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let led = obj::load(
            &device,
            "led.obj",
            "led.mtl",
            Mat4::from_scale(Vec3::splat(0.03)),
        );

        let robor = obj::load(
            &device,
            "robor.obj",
            "robor.mtl",
            Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2) * Mat4::from_scale(Vec3::splat(2.0)),
        );

        let cam_bg = bind::create(&device, bytemuck::cast_slice(&[camera.mvp(size.width, size.height)]));
        let lights_bg = bind::create(&device, bytemuck::cast_slice(std::slice::from_ref(lights)));

        let pipeline = create_pipeline(&device, format, &[&cam_bg.layout, &lights_bg.layout], SHADER);
        let light_pipeline = create_pipeline(&device, format, &[&cam_bg.layout, &lights_bg.layout], LIGHT_SHADER);

        let depth = create_depth_view(&device, size.width, size.height);

        Self { surface, device, queue, config, pipeline, light_pipeline, cam_bg, lights_bg, depth, led, robor }
    }

    pub fn render(&self) {
        let frame = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => frame,
            _ => return,
        };

        let view = frame.texture.create_view(&Default::default());
        let mut enc = self.device.create_command_encoder(&Default::default());

        {
            let mut rpass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.08, g: 0.08, b: 0.12, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.cam_bg.bg, &[]);
            rpass.set_bind_group(1, &self.lights_bg.bg, &[]);
            rpass.set_vertex_buffer(0, self.robor.vbuf.slice(..));
            rpass.set_index_buffer(self.robor.ibuf.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..self.robor.ilen, 0, 0..1);

            rpass.set_pipeline(&self.light_pipeline);

            rpass.set_vertex_buffer(0, self.led.vbuf.slice(..));
            rpass.set_index_buffer(self.led.ibuf.slice(..), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..self.led.ilen, 0, 0..Lights::len());
        }

        self.queue.submit([enc.finish()]);
        frame.present();
    }

    pub fn update_camera(&self, camera: &Camera) {
        let mvp = camera.mvp(self.config.width, self.config.height);
        self.update(&self.cam_bg.buf, bytemuck::cast_slice(&[mvp]));
    }
    pub fn update_lights(&self, lights: &Lights) {
        self.update(&self.lights_bg.buf, bytemuck::cast_slice(std::slice::from_ref(lights)));
    }
    fn update(&self, buf: &wgpu::Buffer, data: &[u8]) {
        self.queue.write_buffer(buf, 0, data);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;

        self.surface.configure(&self.device, &self.config);
        self.depth = create_depth_view(&self.device, width, height);
    }
}

pub fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bg_layouts: &[&wgpu::BindGroupLayout],
    shader: &str,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(format!("{HEADER}\n{shader}").into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &bg_layouts.iter().copied().map(Some).collect::<Vec<_>>(),
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vtx"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x4],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("frag"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    })
}

pub fn create_depth_view(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    }).create_view(&Default::default())
}
