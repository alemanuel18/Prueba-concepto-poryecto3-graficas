use crate::math::Vec3;
use std::{fs, io};
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: Vec3,
    pub color: [u8; 3],
}
pub struct Mesh {
    pub vertices: Vec<Vertex>,
}
impl Mesh {
    pub fn load(name: &str) -> io::Result<Self> {
        let data = fs::read(format!("assets/processed/{name}.mesh"))?;
        if data.len() < 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "mesh header"));
        }
        let count = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
        if data.len() != 4 + count * 15 || count % 3 != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "mesh size"));
        }
        let mut vertices = Vec::with_capacity(count);
        for c in data[4..].chunks_exact(15) {
            let f = |i| f32::from_le_bytes(c[i..i + 4].try_into().unwrap());
            vertices.push(Vertex {
                pos: Vec3::new(f(0), f(4), f(8)),
                color: [c[12], c[13], c[14]],
            });
        }
        Ok(Self { vertices })
    }
}
impl Mesh {
    pub fn sphere() -> Self {
        Self::sphere_colored([245, 170, 62])
    }
    pub fn sphere_colored(color: [u8; 3]) -> Self {
        let mut vertices = Vec::new();
        let stacks = 24;
        let slices = 48;
        for y in 0..stacks {
            for x in 0..slices {
                let point = |ix: i32, iy: i32| {
                    let latitude = (iy as f32 / stacks as f32 - 0.5) * std::f32::consts::PI;
                    let longitude = ix as f32 / slices as f32 * std::f32::consts::TAU;
                    Vec3::new(
                        latitude.cos() * longitude.cos(),
                        latitude.sin(),
                        latitude.cos() * longitude.sin(),
                    )
                };
                let a = point(x, y);
                let b = point(x + 1, y);
                let c = point(x + 1, y + 1);
                let d = point(x, y + 1);
                for pos in [a, b, c, a, c, d] {
                    let noise =
                        (pos.x * 9.0).sin() * (pos.y * 7.0).cos() + 0.45 * (pos.z * 13.0).sin();
                    let shade = (0.86 + 0.14 * noise).clamp(0.65, 1.08);
                    let color = color.map(|v| (v as f32 * shade).clamp(0.0, 255.0) as u8);
                    vertices.push(Vertex {
                        pos: pos * (1.0 + 0.018 * noise),
                        color,
                    });
                }
            }
        }
        Self { vertices }
    }
}
