use std::collections::HashMap;
use std::fs;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
}

pub struct Model {
    pub vbuf: wgpu::Buffer,
    pub ibuf: wgpu::Buffer,
    pub ilen: u32,
}

pub fn load(device: &wgpu::Device, obj: &str, mtl: &str, trns: Mat4) -> Model {
    let mtl = fs::read_to_string(mtl).unwrap();
    let mut mtls = HashMap::new();

    let mut cur_mtl: Option<&str> = None;
    let mut cur_attrs: Option<[f32; 4]> = None;

    for line in mtl.lines().collect::<Vec<_>>() {
        let mut seq = line.split_whitespace();
        let first = seq.next().unwrap_or("");
        let rest: Vec<&str> = seq.collect();

        match first {
            "newmtl" => {
                if let Some(mtl) = cur_mtl {
                    mtls.insert(mtl, cur_attrs.unwrap());
                    cur_attrs = None;
                }

                cur_mtl = Some(rest[0]);
            }

            "Kd" => {
                let rgb: Vec<f32> = rest.iter().map(|c| c.parse().unwrap()).collect();
                let alpha = cur_attrs.map_or(1.0, |a| a[3]);
                cur_attrs = Some([rgb[0], rgb[1], rgb[2], alpha]);
            }

            "d" => {
                let alpha: f32 = rest[0].parse().unwrap();
                let rgb = cur_attrs.map_or([0.0, 0.0, 0.0], |a| [a[0], a[1], a[2]]);
                cur_attrs = Some([rgb[0], rgb[1], rgb[2], alpha]);
            }

            _ => {}
        }
    }

    if let Some(mtl) = cur_mtl {
        mtls.insert(mtl, cur_attrs.unwrap());
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();

    let mut vpos = Vec::new();
    let mut vlookup = HashMap::new();

    let obj = fs::read_to_string(obj).unwrap();
    let mut cur = None;

    for line in obj.lines().collect::<Vec<_>>() {
        let mut seq = line.split_whitespace();
        let first = seq.next().unwrap_or("");
        let rest: Vec<&str> = seq.collect();

        match first {
            "v" => {
                let pos: Vec<f32> = rest.iter().map(|c| c.parse().unwrap()).collect();
                vpos.push([pos[0], pos[1], pos[2]]);
            }

            "vn" => {
                let nml: Vec<f32> = rest.iter().map(|c| c.parse().unwrap()).collect();
                normals.push([nml[0], nml[1], nml[2]]);
            }

            "f" => {
                let (vtxs, nmls): (Vec<u32>, Vec<u32>) =
                    rest
                        .iter()
                        .map(|c| {
                            let mut vals = c.split("//").map(|v| v.parse::<u32>().unwrap() - 1);
                            (vals.next().unwrap(), vals.next().unwrap())
                        })
                        .unzip();

                let color = mtls.get(cur.unwrap()).unwrap();

                let vtxs: Vec<u32> =
                    vtxs
                        .iter()
                        .zip(nmls.iter())
                        .map(|(&vi, &ni)| {
                            vlookup.get(&(vi, ni)).copied().unwrap_or_else(|| {
                                let pos = vpos[vi as usize];
                                let normal = normals[ni as usize];

                                vertices.push(Vertex { pos, normal, color: *color });

                                let i = (vertices.len() - 1) as u32;
                                vlookup.insert((vi, ni), i);
                                i
                            })
                        })
                        .collect();

                indices.extend([vtxs[0], vtxs[1], vtxs[2]]);
            }

            "usemtl" => {
                cur = Some(rest[0]);
            }

            _ => {}
        }
    }

    let (xs, yzs): (Vec<f32>, Vec<(f32, f32)>) =
        vertices.iter().map(|v| (v.pos[0], (v.pos[1], v.pos[2]))).unzip();

    let (ys, zs): (Vec<f32>, Vec<f32>) = yzs.into_iter().unzip();

    let scale =
        [xs, ys, zs]
            .iter()
            .fold(
                0.0,
                |acc, cs| {
                    let (min, max) = cs.iter().copied().fold(
                        (f32::INFINITY, f32::NEG_INFINITY),
                        |(min, max), x| (min.min(x), max.max(x)),
                    );
                    let range = max - min;

                    f32::max(acc, range)
                });

    let trns = trns * Mat4::from_scale(Vec3::splat(1.0 / scale));

    for v in &mut vertices {
        let p = Vec3::from(v.pos);
        let n = Vec3::from(v.normal);
        v.pos = trns.transform_point3(p).into();
        v.normal = trns.transform_vector3(n).into();
    }

    let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    Model { vbuf, ibuf, ilen: indices.len() as u32 }
}
