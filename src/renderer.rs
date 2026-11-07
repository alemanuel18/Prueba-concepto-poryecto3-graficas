use crate::{asset::Mesh, math::Vec3, scene::Surface};
pub const W: usize = 640;
pub const H: usize = 360;
#[derive(Clone, Copy)]
pub struct Camera {
    pub pos: Vec3,
    pub target: Vec3,
    pub up: Vec3,
}
#[derive(Clone, Copy)]
pub enum Shader {
    Sun,
    Planet(Surface),
    Ship,
    Ring,
}
pub struct Renderer {
    pub pixels: Vec<u8>,
    depth: Vec<f32>,
    pub triangles: u32,
    pub culled: u32,
}
#[derive(Clone, Copy)]
struct Projected {
    x: f32,
    y: f32,
    z: f32,
    color: [u8; 3],
}
impl Renderer {
    pub fn new() -> Self {
        Self {
            pixels: vec![0; W * H * 4],
            depth: vec![f32::INFINITY; W * H],
            triangles: 0,
            culled: 0,
        }
    }
    pub fn clear(&mut self, time: f32, cam: Camera, sky: &[u8], kind: Option<Surface>) {
        self.depth.fill(f32::INFINITY);
        self.triangles = 0;
        self.culled = 0;
        let forward = (cam.target - cam.pos).norm();
        let yaw = forward.x.atan2(-forward.z);
        let shift = (yaw / std::f32::consts::TAU * 256.0) as i32;
        let base = match kind {
            Some(Surface::Grass) => [4.0, 12.0, 26.0],
            Some(Surface::Cloud) => [12.0, 18.0, 35.0],
            Some(Surface::Ice) => [3.0, 11.0, 30.0],
            Some(Surface::Lava) => [16.0, 5.0, 20.0],
            Some(Surface::Metal) => [7.0, 9.0, 28.0],
            None => [5.0, 7.0, 22.0],
        };
        for y in 0..H {
            for x in 0..W {
                let u = ((x * 256 / W) as i32 + shift).rem_euclid(256) as usize;
                let v = (y * 256 / H + ((time * 0.3) as usize)) % 256;
                let si = (v * 256 + u) * 3;
                let i = (y * W + x) * 4;
                let haze = (1.0 - (y as f32 / H as f32 - 0.5).abs() * 2.0).max(0.0);
                for k in 0..3 {
                    self.pixels[i + k] =
                        (base[k] + haze * base[k] * 0.65 + sky[si + k] as f32 * 0.42).min(255.0)
                            as u8;
                }
                self.pixels[i + 3] = 255;
            }
        }
    }
    fn basis(cam: Camera) -> (Vec3, Vec3, Vec3) {
        let f = (cam.target - cam.pos).norm();
        let r = f.cross(cam.up).norm();
        let u = r.cross(f);
        (r, u, f)
    }
    fn project(p: Vec3, color: [u8; 3], cam: Camera, b: (Vec3, Vec3, Vec3)) -> Option<Projected> {
        let q = p - cam.pos;
        let z = q.dot(b.2);
        if z < 0.08 {
            return None;
        }
        let focal = H as f32 * 0.86;
        Some(Projected {
            x: W as f32 * 0.5 + q.dot(b.0) * focal / z,
            y: H as f32 * 0.5 - q.dot(b.1) * focal / z,
            z,
            color,
        })
    }
    pub fn draw_mesh(
        &mut self,
        mesh: &Mesh,
        cam: Camera,
        center: Vec3,
        scale: f32,
        rotation: f32,
        shader: Shader,
        time: f32,
    ) {
        let b = Self::basis(cam);
        let dist = (center - cam.pos).len();
        if dist > 180.0 || (center - cam.pos).dot(b.2) < -scale {
            self.culled += mesh.vertices.len() as u32 / 3;
            return;
        }
        let sun_dir = (Vec3::new(0.0, 0.0, 0.0) - center).norm();
        for tri in mesh.vertices.chunks_exact(3) {
            let world = [0, 1, 2].map(|i| center + tri[i].pos.rotate_y(rotation) * scale);
            let Some(a) = Self::project(world[0], tri[0].color, cam, b) else {
                continue;
            };
            let Some(c) = Self::project(world[1], tri[1].color, cam, b) else {
                continue;
            };
            let Some(d) = Self::project(world[2], tri[2].color, cam, b) else {
                continue;
            };
            let normal = (world[1] - world[0]).cross(world[2] - world[0]).norm();
            let lit = (0.45 + 0.55 * normal.dot(sun_dir).abs()).clamp(0.25, 1.0);
            self.triangle([a, c, d], shader, lit, time);
            self.triangles += 1;
        }
    }
    fn triangle(&mut self, p: [Projected; 3], shader: Shader, lit: f32, time: f32) {
        let min_x = p
            .iter()
            .map(|v| v.x)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as usize;
        let max_x = p
            .iter()
            .map(|v| v.x)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min((W - 1) as f32) as usize;
        let min_y = p
            .iter()
            .map(|v| v.y)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as usize;
        let max_y = p
            .iter()
            .map(|v| v.y)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min((H - 1) as f32) as usize;
        if min_x > max_x || min_y > max_y {
            return;
        }
        let edge = |a: Projected, b: Projected, x: f32, y: f32| {
            (x - a.x) * (b.y - a.y) - (y - a.y) * (b.x - a.x)
        };
        let area = edge(p[0], p[1], p[2].x, p[2].y);
        if area.abs() < 0.01 {
            return;
        }
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let w0 = edge(p[1], p[2], px, py) / area;
                let w1 = edge(p[2], p[0], px, py) / area;
                let w2 = 1.0 - w0 - w1;
                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }
                let iz = w0 / p[0].z + w1 / p[1].z + w2 / p[2].z;
                let z = 1.0 / iz;
                let idx = y * W + x;
                if z >= self.depth[idx] {
                    continue;
                }
                self.depth[idx] = z;
                let ws = [w0 / p[0].z * z, w1 / p[1].z * z, w2 / p[2].z * z];
                let mut rgb = [0f32; 3];
                for k in 0..3 {
                    rgb[k] = ((p[0].color[k] as f32 * ws[0]
                        + p[1].color[k] as f32 * ws[1]
                        + p[2].color[k] as f32 * ws[2])
                        * lit)
                        .clamp(0.0, 255.0)
                }
                match shader {
                    Shader::Sun => {
                        let swirl =
                            ((px * 0.12 + time * 2.0).sin() * (py * 0.11 - time * 1.3).cos() + 1.0)
                                * 0.5;
                        let pulse = 1.0 + 0.12 * (time * 2.0 + px * 0.025).sin();
                        rgb = [
                            (180.0 + 75.0 * swirl) * pulse,
                            85.0 + 130.0 * swirl,
                            12.0 + 55.0 * swirl,
                        ];
                    }
                    Shader::Planet(Surface::Lava) => {
                        let flow = (time * 1.7 + px * 0.055 + py * 0.03).sin() * 0.5 + 0.5;
                        rgb[0] = (rgb[0] + flow * 48.0).min(255.0);
                        rgb[1] = (rgb[1] + flow * 14.0).min(255.0)
                    }
                    Shader::Planet(Surface::Ice) => {
                        let gleam = (time * 0.7 + px * 0.025).sin().max(0.0) * 28.0;
                        rgb[1] = (rgb[1] + gleam).min(255.0);
                        rgb[2] = (rgb[2] + gleam + 20.0).min(255.0)
                    }
                    Shader::Planet(Surface::Metal) => {
                        let spec = lit.powf(8.0) * 55.0;
                        for v in &mut rgb {
                            *v = (*v + spec).min(255.0)
                        }
                    }
                    Shader::Planet(Surface::Cloud) => rgb[2] = (rgb[2] + 18.0).min(255.0),
                    Shader::Ring => {
                        let p = (time * 3.0).sin() * 0.15 + 1.0;
                        rgb = [
                            (rgb[0] * p + 30.0).min(255.0),
                            (rgb[1] * p + 55.0).min(255.0),
                            255.0,
                        ]
                    }
                    Shader::Ship => {}
                    Shader::Planet(Surface::Grass) => {}
                }
                let i = idx * 4;
                for k in 0..3 {
                    self.pixels[i + k] = rgb[k] as u8
                }
            }
        }
    }
    pub fn draw_line(&mut self, cam: Camera, a: Vec3, b: Vec3, color: [u8; 3]) {
        let basis = Self::basis(cam);
        let (Some(a), Some(b)) = (
            Self::project(a, color, cam, basis),
            Self::project(b, color, cam, basis),
        ) else {
            return;
        };
        let steps = ((a.x - b.x).abs().max((a.y - b.y).abs()) as usize).min(2000);
        for s in 0..=steps {
            let t = s as f32 / steps.max(1) as f32;
            let x = (a.x + (b.x - a.x) * t) as i32;
            let y = (a.y + (b.y - a.y) * t) as i32;
            if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
                continue;
            }
            let i = y as usize * W + x as usize;
            let z = a.z + (b.z - a.z) * t;
            if z < self.depth[i] {
                self.depth[i] = z;
                let k = i * 4;
                self.pixels[k..k + 3].copy_from_slice(&color)
            }
        }
    }
}
