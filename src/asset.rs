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
        let mut vertices = Vec::new();
        let stacks = 18;
        let slices = 36;
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
                    vertices.push(Vertex {
                        pos,
                        color: [245, 170, 62],
                    });
                }
            }
        }
        Self { vertices }
    }
}
impl Mesh {
    pub fn ship() -> Self {
        let mut vertices = Vec::new();
        let mut tri = |a: [f32; 3], b: [f32; 3], c: [f32; 3], color: [u8; 3]| {
            for p in [a, b, c] {
                vertices.push(Vertex {
                    pos: Vec3::new(p[0], p[1], p[2]),
                    color,
                });
            }
        };
        // Original compact ship: pointed hull, blue cockpit, swept wings and engines.
        let nose = [0.0, 0.0, -1.3];
        let left = [-0.34, -0.12, 0.65];
        let right = [0.34, -0.12, 0.65];
        let top = [0.0, 0.32, 0.25];
        let bottom = [0.0, -0.24, 0.50];
        tri(nose, left, top, [232, 230, 218]);
        tri(nose, top, right, [239, 241, 232]);
        tri(nose, right, bottom, [150, 169, 184]);
        tri(nose, bottom, left, [123, 148, 170]);
        tri(left, right, top, [72, 158, 218]);
        tri(left, bottom, right, [61, 95, 130]);
        tri(
            [-0.23, 0.26, -0.2],
            [0.23, 0.26, -0.2],
            [0.0, 0.39, 0.40],
            [53, 208, 249],
        );
        tri(
            [-0.23, 0.26, -0.2],
            [0.0, 0.39, 0.40],
            [-0.38, 0.04, 0.2],
            [28, 94, 175],
        );
        tri(
            [0.23, 0.26, -0.2],
            [0.38, 0.04, 0.2],
            [0.0, 0.39, 0.40],
            [28, 94, 175],
        );
        for sign in [-1.0, 1.0] {
            let root = [sign * 0.28, -0.08, 0.22];
            let tip = [sign * 1.07, -0.16, 0.86];
            let back = [sign * 0.25, -0.11, 0.88];
            tri(root, tip, back, [218, 67, 55]);
            tri(root, back, tip, [126, 38, 54]);
            let a = [sign * 0.23, -0.12, 0.62];
            let b = [sign * 0.45, -0.12, 0.62];
            let c = [sign * 0.34, -0.17, 1.04];
            tri(a, b, c, [75, 174, 249]);
            tri(c, b, a, [36, 91, 199]);
        }
        Self { vertices }
    }
}
