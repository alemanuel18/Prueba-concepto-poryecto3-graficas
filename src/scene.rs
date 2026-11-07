use crate::{asset::Mesh, math::Vec3};
#[derive(Clone, Copy)]
pub struct Placement {
    pub position: Vec3,
    pub normal: Vec3,
    pub size: f32,
}
pub fn surface_placements(mesh: &Mesh, seed: u32) -> Vec<Placement> {
    let candidates: Vec<_> = mesh
        .vertices
        .chunks_exact(3)
        .step_by(7)
        .filter_map(|tri| {
            let center = (tri[0].pos + tri[1].pos + tri[2].pos) * (1.0 / 3.0);
            let mut normal = (tri[1].pos - tri[0].pos)
                .cross(tri[2].pos - tri[0].pos)
                .norm();
            if normal.dot(center) < 0.0 {
                normal = normal * -1.0;
            }
            (center.len() > 0.28 && normal.dot(center.norm()) > 0.45).then_some((center, normal))
        })
        .collect();
    let mut state = seed;
    (0..28)
        .filter_map(|i| {
            if candidates.is_empty() {
                return None;
            }
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let (position, normal) = candidates[state as usize % candidates.len()];
            Some(Placement {
                position,
                normal,
                size: 0.10 + (i % 4) as f32 * 0.025,
            })
        })
        .collect()
}
#[derive(Clone, Copy, Debug)]
pub enum Surface {
    Grass,
    Water,
    Ice,
    Lava,
    Metal,
}
pub struct Planet {
    pub name: &'static str,
    pub mesh: &'static str,
    pub low: &'static str,
    pub surface: Surface,
    pub radius: f32,
    pub orbit: f32,
    pub orbit_speed: f32,
    pub spin: f32,
    pub phase: f32,
}
impl Planet {
    pub fn position(&self, t: f32) -> Vec3 {
        let a = self.phase + t * self.orbit_speed;
        Vec3::new(a.cos() * self.orbit, 0.0, a.sin() * self.orbit)
    }
    pub fn scale_at(&self, distance: f32) -> f32 {
        let t = ((distance - (self.radius + 5.0)) / 18.0).clamp(0.0, 1.0);
        let smooth = t * t * (3.0 - 2.0 * t);
        1.0 + 0.55 * (1.0 - smooth)
    }
}
pub fn planets() -> Vec<Planet> {
    vec![
        Planet {
            name: "Yoshi Star",
            mesh: "yoshi",
            low: "yoshi_low",
            surface: Surface::Grass,
            radius: 3.5,
            orbit: 16.0,
            orbit_speed: 0.055,
            spin: 0.22,
            phase: 0.0,
        },
        Planet {
            name: "Fluffy Bluff",
            mesh: "fluffy",
            low: "fluffy_low",
            surface: Surface::Water,
            radius: 3.8,
            orbit: 27.0,
            orbit_speed: 0.038,
            spin: 0.17,
            phase: 1.2,
        },
        Planet {
            name: "Freezeflame Ice",
            mesh: "ice",
            low: "ice_low",
            surface: Surface::Ice,
            radius: 3.7,
            orbit: 38.0,
            orbit_speed: 0.026,
            spin: 0.16,
            phase: 2.4,
        },
        Planet {
            name: "Volcano Sandwich",
            mesh: "lava",
            low: "lava",
            surface: Surface::Lava,
            radius: 4.0,
            orbit: 49.0,
            orbit_speed: 0.019,
            spin: 0.13,
            phase: 3.5,
        },
        Planet {
            name: "Throwback Fortress",
            mesh: "fortress",
            low: "fortress_low",
            surface: Surface::Metal,
            radius: 4.0,
            orbit: 60.0,
            orbit_speed: 0.014,
            spin: 0.12,
            phase: 4.6,
        },
    ]
}
pub struct Ship {
    pub pos: Vec3,
    pub yaw: f32,
    pub speed: f32,
    pub camera_far: bool,
}
impl Ship {
    pub fn forward(&self) -> Vec3 {
        Vec3::new(self.yaw.sin(), 0.0, -self.yaw.cos())
    }
}
#[derive(Clone, Copy)]
pub enum Mode {
    Space,
    Surface {
        planet: usize,
        radial: Vec3,
        yaw: f32,
    },
    Returning {
        planet: usize,
        radial: Vec3,
        yaw: f32,
        remaining: f32,
    },
}

#[cfg(test)]
mod tests {
    #[test]
    fn planet_grows_smoothly_as_camera_approaches() {
        let planet = &super::planets()[0];
        let distant = planet.scale_at(50.0);
        let middle = planet.scale_at(20.0);
        let near = planet.scale_at(planet.radius + 5.0);
        assert!((distant - 1.0).abs() < 1e-6);
        assert!(distant < middle && middle < near);
        assert!((near - 1.55).abs() < 1e-6);
    }
}
