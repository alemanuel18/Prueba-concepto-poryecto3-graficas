use crate::math::Vec3;
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
