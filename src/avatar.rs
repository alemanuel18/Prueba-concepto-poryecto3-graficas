use crate::{
    asset::Mesh,
    math::Vec3,
    renderer::{Camera, Renderer, Shader},
    scene::{self, Planet, STARSHIP_SCALE, Ship, SurfaceTarget, WalkProfile},
};

pub const HEIGHT: f32 = 0.8;
pub const RADIUS: f32 = 0.14;
const MODEL_SCALE: f32 = HEIGHT / 1.9;

pub struct Pose {
    pub ground: Vec3,
    pub up: Vec3,
    pub forward: Vec3,
    center: Vec3,
    rotation: f32,
    scale: f32,
    starship: bool,
    planar: bool,
}

impl Pose {
    pub fn new(
        target: SurfaceTarget,
        local: Vec3,
        normal: Vec3,
        yaw: f32,
        planets: &[Planet],
        ship: &Ship,
        time: f32,
        profile: WalkProfile,
    ) -> Self {
        let (center, rotation, scale) = match target {
            SurfaceTarget::Planet(i) => (
                planets[i].position(time),
                time * planets[i].spin,
                planets[i].radius,
            ),
            SurfaceTarget::Starship => (ship.pos, ship.yaw, STARSHIP_SCALE),
        };
        let up = normal.rotate_y(rotation).norm();
        let floor_local = local - normal * 0.015;
        let ground = center + floor_local.rotate_y(rotation) * scale;
        let north = scene::surface_forward(local, normal);
        let side = north.cross(normal).norm();
        let forward = (north * yaw.cos() + side * yaw.sin())
            .rotate_y(rotation)
            .norm();
        let planar = matches!(profile, WalkProfile::Planar);
        Self {
            ground,
            up,
            forward,
            center,
            rotation,
            scale,
            starship: matches!(target, SurfaceTarget::Starship),
            planar,
        }
    }

    fn localize(&self, point: Vec3) -> Vec3 {
        ((point - self.center) * (1.0 / self.scale)).rotate_y(-self.rotation)
    }

    pub fn camera(&self, mesh: &Mesh) -> Camera {
        let s = self.scale;
        // Camera up: for flat/planar planets use world Y (stable), for round use radial
        let cam_up = if self.planar && !self.starship {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            self.up
        };
        let aim = self.ground + self.up * (HEIGHT * 0.8 * s) + self.forward * (HEIGHT * 1.0 * s);
        let desired = if self.starship {
            self.ground + self.up * (HEIGHT * 3.8 * s) - self.forward * (HEIGHT * 4.2 * s)
        } else {
            self.ground + self.up * (HEIGHT * 2.6 * s) - self.forward * (HEIGHT * 5.0 * s)
        };
        let start = self.ground + self.up * (HEIGHT * 0.9 * s);
        let a = self.localize(start);
        let b = self.localize(desired);
        let t = first_intersection(mesh, a, b)
            .map(|hit| (hit - 0.04).max(if self.starship { 0.5 } else { 0.2 }))
            .unwrap_or(1.0);
        Camera {
            pos: start + (desired - start) * t,
            target: aim,
            up: cam_up,
        }
    }

    pub fn draw(&self, renderer: &mut Renderer, mesh: &Mesh, cam: Camera, time: f32) {
        let center = self.ground + self.up * (HEIGHT * 0.5);
        let right = if self.up.y.abs() > 0.95 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0).cross(self.up).norm()
        };
        let base_forward = right.cross(self.up).norm();
        let heading = self
            .forward
            .dot(right)
            .atan2(self.forward.dot(base_forward));
        renderer.draw_mesh_oriented(
            mesh,
            cam,
            center,
            MODEL_SCALE,
            heading,
            self.up,
            Shader::Ship,
            time,
        );
    }
}

fn first_intersection(mesh: &Mesh, start: Vec3, end: Vec3) -> Option<f32> {
    let ray = end - start;
    let mut first = 1.0;
    for tri in mesh.vertices.chunks_exact(3) {
        let a = tri[0].pos;
        let ab = tri[1].pos - a;
        let ac = tri[2].pos - a;
        let p = ray.cross(ac);
        let det = ab.dot(p);
        if det.abs() < 1e-7 {
            continue;
        }
        let inv = 1.0 / det;
        let rel = start - a;
        let u = rel.dot(p) * inv;
        if !(0.0..=1.0).contains(&u) {
            continue;
        }
        let q = rel.cross(ab);
        let v = ray.dot(q) * inv;
        if v < 0.0 || u + v > 1.0 {
            continue;
        }
        let t = ac.dot(q) * inv;
        if t > 0.02 && t < first {
            first = t;
        }
    }
    if first < 1.0 { Some(first) } else { None }
}
