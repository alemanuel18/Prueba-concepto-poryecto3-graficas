use crate::{asset::Mesh, math::Vec3};
#[derive(Clone, Copy)]
pub struct Placement {
    pub position: Vec3,
    pub normal: Vec3,
    pub size: f32,
}
#[derive(Clone, Copy, Debug)]
pub struct SurfacePoint {
    pub position: Vec3,
    pub normal: Vec3,
}
#[derive(Clone, Copy, Debug)]
pub enum SurfaceTarget {
    Planet(usize),
    Starship,
}
#[derive(Clone, Copy, Debug)]
pub enum WalkProfile {
    Planar,
    Round,
    Volcanic,
}

fn is_hot_lava(tri: &[crate::asset::Vertex]) -> bool {
    let [r, g, b] = [0, 1, 2].map(|channel| {
        tri.iter()
            .map(|vertex| vertex.color[channel] as f32)
            .sum::<f32>()
            / 3.0
    });
    r > 90.0 && r > g * 1.35 && r > b * 1.25
}

pub const STARSHIP_SCALE: f32 = 3.5;

pub fn surface_forward(local: Vec3, normal: Vec3) -> Vec3 {
    let toward_center = local * -1.0;
    let flat = toward_center - normal * toward_center.dot(normal);
    if flat.len() > 0.08 {
        return flat.norm();
    }
    let base = if normal.z.abs() > 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 0.0, -1.0)
    };
    (base - normal * base.dot(normal)).norm()
}

pub fn landing_point(mesh: &Mesh, kind: Surface) -> SurfacePoint {
    let mut best = None;
    let mut best_score = f32::NEG_INFINITY;
    // Entry locations correspond to the first open walking areas visible in the reference footage.
    // Coordinates are normalized positions in each supplied level mesh.
    let entry = match kind {
        Surface::Grass => Vec3::new(-0.20, 0.122, -0.15),
        Surface::Water => Vec3::new(0.38, -0.212, 0.50),
        Surface::Ice => Vec3::new(-0.39, -0.118, 0.10),
        Surface::Metal => Vec3::new(0.608, -0.101, 0.395),
        Surface::Lava => Vec3::new(0.0, 0.0, 0.0),
    };
    for tri in mesh.vertices.chunks_exact(3) {
        let a = tri[0].pos;
        let b = tri[1].pos;
        let c = tri[2].pos;
        let cross = (b - a).cross(c - a);
        let area = cross.len();
        if area < 1e-7 {
            continue;
        }
        let position = (a + b + c) * (1.0 / 3.0);
        if matches!(kind, Surface::Lava) && is_hot_lava(tri) {
            continue;
        }
        let mut normal = cross * (1.0 / area);
        if matches!(kind, Surface::Lava) {
            if normal.dot(position) < 0.0 {
                normal = normal * -1.0;
            }
            if normal.dot(position.norm()) < 0.82 || !(-0.1..=0.45).contains(&position.y) {
                continue;
            }
        } else {
            if normal.y < 0.0 {
                normal = normal * -1.0;
            }
            if normal.y < 0.78 || position.y < -0.4 {
                continue;
            }
        }
        let rgb = [0, 1, 2].map(|k| {
            (tri[0].color[k] as f32 + tri[1].color[k] as f32 + tri[2].color[k] as f32) / 3.0
        });
        let material_ok = match kind {
            Surface::Grass | Surface::Water => rgb[1] > rgb[0] * 1.2 && rgb[1] > rgb[2] * 1.1,
            Surface::Ice => rgb.iter().sum::<f32>() > 390.0,
            Surface::Metal => rgb[1] > rgb[0] * 1.05,
            Surface::Lava => true,
        };
        if !material_ok {
            continue;
        }
        let score = if matches!(kind, Surface::Lava) {
            area * (1.0 - position.y.abs() * 0.4)
        } else {
            let distance = position - entry;
            -distance.dot(distance) + area * 0.001
        };
        if score > best_score {
            best_score = score;
            best = Some(SurfacePoint { position, normal });
        }
    }
    best.unwrap_or(SurfacePoint {
        position: Vec3::new(0.0, 0.5, 0.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
    })
}

pub fn starship_landing(mesh: &Mesh) -> SurfacePoint {
    let mut best = None;
    let mut best_score = f32::NEG_INFINITY;
    let entry = Vec3::new(-0.25, 0.69, 0.25);
    for tri in mesh.vertices.chunks_exact(3) {
        let a = tri[0].pos;
        let b = tri[1].pos;
        let c = tri[2].pos;
        let cross = (b - a).cross(c - a);
        let area = cross.len();
        if area < 1e-7 {
            continue;
        }
        let position = (a + b + c) * (1.0 / 3.0);
        if position.y < 0.58 {
            continue;
        }
        let light = tri
            .iter()
            .map(|v| v.color[0] as u32 + v.color[1] as u32 + v.color[2] as u32)
            .sum::<u32>();
        if light < 3 * 3 * 145 {
            continue;
        }
        let radial = position.norm();
        let mut normal = cross * (1.0 / area);
        if normal.dot(radial) < 0.0 {
            normal = normal * -1.0
        }
        if normal.dot(radial) < 0.8 {
            continue;
        }
        let offset = position - entry;
        let score = -offset.dot(offset) + area * 0.001;
        if score > best_score {
            best_score = score;
            best = Some(SurfacePoint { position, normal });
        }
    }
    best.unwrap_or(SurfacePoint {
        position: Vec3::new(0.0, 0.5, 0.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
    })
}

fn closest_point_on_triangle(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return a + ab * (d1 / (d1 - d3));
    }
    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return a + ac * (d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    a + ab * (vb / (va + vb + vc)) + ac * (vc / (va + vb + vc))
}

pub fn closest_floor(
    mesh: &Mesh,
    query: Vec3,
    previous: SurfacePoint,
    profile: WalkProfile,
    max_distance: f32,
) -> Option<SurfacePoint> {
    let mut best = None;
    let mut best_d2 = max_distance * max_distance;
    let gravity_up = match profile {
        WalkProfile::Planar => Vec3::new(0.0, 1.0, 0.0),
        WalkProfile::Round | WalkProfile::Volcanic => query.norm(),
    };
    for tri in mesh.vertices.chunks_exact(3) {
        if matches!(profile, WalkProfile::Volcanic) && is_hot_lava(tri) {
            continue;
        }
        let a = tri[0].pos;
        let b = tri[1].pos;
        let c = tri[2].pos;
        let cross = (b - a).cross(c - a);
        let length = cross.len();
        if length < 1e-7 {
            continue;
        }
        let mut normal = cross * (1.0 / length);
        if normal.dot(gravity_up) < 0.0 {
            normal = normal * -1.0;
        }
        if normal.dot(gravity_up) < 0.78 || normal.dot(previous.normal) < 0.7 {
            continue;
        }
        let position = closest_point_on_triangle(query, a, b, c);
        if (position - previous.position).dot(gravity_up).abs() > 0.045 {
            continue;
        }
        let d2 = (position - query).dot(position - query);
        if d2 < best_d2 {
            best_d2 = d2;
            best = Some(SurfacePoint { position, normal });
        }
    }
    best
}

pub fn walk_step(
    mesh: &Mesh,
    start: SurfacePoint,
    delta: Vec3,
    profile: WalkProfile,
    radius: f32,
    height: f32,
) -> SurfacePoint {
    if delta.len() < 1e-6 {
        return start;
    }
    let count = (delta.len() / 0.012).ceil().max(1.0) as usize;
    let step = delta * (1.0 / count as f32);
    let mut current = start;
    for _ in 0..count {
        let Some(next) = closest_floor(mesh, current.position + step, current, profile, 0.035)
        else {
            break;
        };
        if (next.position - current.position).dot(step.norm()) < step.len() * 0.3 {
            break;
        }
        let forward = (step - next.normal * step.dot(next.normal)).norm();
        let side = forward.cross(next.normal).norm();
        let clearance = radius * 0.55;
        let probes = [
            next.position + forward * radius,
            next.position + side * radius,
            next.position - side * radius,
        ];
        if probes.iter().any(|&probe| {
            closest_floor(mesh, probe, next, profile, clearance)
                .is_none_or(|floor| (floor.position - probe).len() > clearance)
        }) {
            break;
        }
        if !wall_clear(mesh, next, profile, radius, height) {
            break;
        }
        current = next;
    }
    current
}

fn wall_clear(
    mesh: &Mesh,
    floor: SurfacePoint,
    profile: WalkProfile,
    radius: f32,
    height: f32,
) -> bool {
    let up = match profile {
        WalkProfile::Planar => Vec3::new(0.0, 1.0, 0.0),
        WalkProfile::Round | WalkProfile::Volcanic => floor.position.norm(),
    };
    let torso = floor.position + up * (height * 0.52);
    for tri in mesh.vertices.chunks_exact(3) {
        let a = tri[0].pos;
        let b = tri[1].pos;
        let c = tri[2].pos;
        let cross = (b - a).cross(c - a);
        if cross.len() < 1e-7 || cross.norm().dot(up).abs() >= 0.78 {
            continue;
        }
        let nearest = closest_point_on_triangle(torso, a, b, c);
        if (nearest - torso).len() < radius {
            return false;
        }
    }
    true
}
pub fn surface_placements(mesh: &Mesh, seed: u32, kind: Surface) -> Vec<Placement> {
    let candidates: Vec<_> = mesh
        .vertices
        .chunks_exact(3)
        .step_by(7)
        .filter_map(|tri| {
            if matches!(kind, Surface::Lava) && is_hot_lava(tri) {
                return None;
            }
            let center = (tri[0].pos + tri[1].pos + tri[2].pos) * (1.0 / 3.0);
            let gravity_up = if matches!(kind, Surface::Lava) {
                center.norm()
            } else {
                Vec3::new(0.0, 1.0, 0.0)
            };
            let mut normal = (tri[1].pos - tri[0].pos)
                .cross(tri[2].pos - tri[0].pos)
                .norm();
            if normal.dot(gravity_up) < 0.0 {
                normal = normal * -1.0;
            }
            (center.len() > 0.25
                && normal.dot(gravity_up) > 0.78
                && (matches!(kind, Surface::Lava) || center.y > -0.4))
                .then_some((center, normal))
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
}
pub fn planets() -> Vec<Planet> {
    vec![
        Planet {
            name: "Yoshi Star",
            mesh: "yoshi",
            low: "yoshi_low",
            surface: Surface::Grass,
            radius: 10.0,
            orbit: 55.0,
            orbit_speed: 0.021,
            spin: 0.22,
            phase: 0.0,
        },
        Planet {
            name: "Fluffy Bluff",
            mesh: "fluffy",
            low: "fluffy_low",
            surface: Surface::Water,
            radius: 12.0,
            orbit: 112.0,
            orbit_speed: 0.012,
            spin: 0.17,
            phase: 1.2,
        },
        Planet {
            name: "Freezeflame Ice",
            mesh: "ice",
            low: "ice_low",
            surface: Surface::Ice,
            radius: 11.0,
            orbit: 170.0,
            orbit_speed: 0.008,
            spin: 0.16,
            phase: 2.4,
        },
        Planet {
            name: "Volcano Sandwich",
            mesh: "lava",
            low: "lava",
            surface: Surface::Lava,
            radius: 13.0,
            orbit: 232.0,
            orbit_speed: 0.006,
            spin: 0.13,
            phase: 3.5,
        },
        Planet {
            name: "Throwback Fortress",
            mesh: "fortress",
            low: "fortress_low",
            surface: Surface::Metal,
            radius: 12.0,
            orbit: 298.0,
            orbit_speed: 0.004,
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
        target: SurfaceTarget,
        local: Vec3,
        normal: Vec3,
        yaw: f32,
    },
    Returning {
        target: SurfaceTarget,
        local: Vec3,
        normal: Vec3,
        yaw: f32,
        remaining: f32,
    },
}

#[cfg(test)]
mod tests {
    use crate::{
        asset::{Mesh, Vertex},
        math::Vec3,
    };
    #[test]
    fn planet_scale_is_constant_at_every_distance() {
        let planets = super::planets();
        assert_eq!(planets[0].radius, 10.0);
        assert!(
            planets
                .windows(2)
                .all(|pair| pair[1].orbit - pair[0].orbit > pair[0].radius + pair[1].radius)
        );
    }
    #[test]
    fn floor_follows_mesh_triangle_instead_of_fixed_radius() {
        let mesh = Mesh {
            vertices: vec![
                Vertex {
                    pos: Vec3::new(0.0, 0.0, 0.0),
                    color: [0; 3],
                },
                Vertex {
                    pos: Vec3::new(1.0, 0.5, 0.0),
                    color: [0; 3],
                },
                Vertex {
                    pos: Vec3::new(0.0, 0.0, 1.0),
                    color: [0; 3],
                },
            ],
        };
        let hit = super::closest_floor(
            &mesh,
            Vec3::new(0.5, 0.30, 0.2),
            super::SurfacePoint {
                position: Vec3::new(0.5, 0.25, 0.2),
                normal: Vec3::new(-0.45, 0.9, 0.0),
            },
            super::WalkProfile::Planar,
            0.2,
        )
        .unwrap();
        assert!((hit.position.y - 0.25).abs() < 0.04);
        assert!(hit.normal.y > 0.8 && hit.normal.x < -0.4);
    }
    #[test]
    fn steep_wall_is_not_walkable() {
        let mesh = Mesh {
            vertices: vec![
                Vertex {
                    pos: Vec3::new(0.0, 0.0, 0.0),
                    color: [0; 3],
                },
                Vertex {
                    pos: Vec3::new(0.0, 1.0, 0.0),
                    color: [0; 3],
                },
                Vertex {
                    pos: Vec3::new(0.0, 0.0, 1.0),
                    color: [0; 3],
                },
            ],
        };
        let previous = super::SurfacePoint {
            position: Vec3::new(0.01, 0.1, 0.1),
            normal: Vec3::new(0.0, 1.0, 0.0),
        };
        assert!(
            super::closest_floor(
                &mesh,
                previous.position,
                previous,
                super::WalkProfile::Planar,
                0.2
            )
            .is_none()
        );
    }
    #[test]
    fn round_body_accepts_a_side_that_is_flat_to_local_gravity() {
        let mesh = Mesh {
            vertices: vec![
                Vertex {
                    pos: Vec3::new(1.0, 0.0, 0.0),
                    color: [0; 3],
                },
                Vertex {
                    pos: Vec3::new(1.0, 1.0, 0.0),
                    color: [0; 3],
                },
                Vertex {
                    pos: Vec3::new(1.0, 0.0, 1.0),
                    color: [0; 3],
                },
            ],
        };
        let previous = super::SurfacePoint {
            position: Vec3::new(1.0, 0.2, 0.2),
            normal: Vec3::new(1.0, 0.0, 0.0),
        };
        let query = Vec3::new(1.03, 0.2, 0.2);
        assert!(
            super::closest_floor(&mesh, query, previous, super::WalkProfile::Round, 0.1).is_some()
        );
        assert!(
            super::closest_floor(&mesh, query, previous, super::WalkProfile::Planar, 0.1).is_none()
        );
    }
    #[test]
    fn volcanic_floor_excludes_lava_but_keeps_cold_stone() {
        let mut mesh = Mesh {
            vertices: [
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(1.0, 1.0, 0.0),
                Vec3::new(1.0, 0.0, 1.0),
            ]
            .map(|pos| Vertex {
                pos,
                color: [220, 30, 10],
            })
            .to_vec(),
        };
        let previous = super::SurfacePoint {
            position: Vec3::new(1.0, 0.2, 0.2),
            normal: Vec3::new(1.0, 0.0, 0.0),
        };
        let query = Vec3::new(1.03, 0.2, 0.2);
        assert!(
            super::closest_floor(&mesh, query, previous, super::WalkProfile::Volcanic, 0.1)
                .is_none()
        );
        for vertex in &mut mesh.vertices {
            vertex.color = [130, 125, 115];
        }
        assert!(
            super::closest_floor(&mesh, query, previous, super::WalkProfile::Volcanic, 0.1)
                .is_some()
        );
    }
    #[test]
    fn avatar_cannot_walk_through_a_wall() {
        let points = [
            Vec3::new(-1.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.3, 0.0, -0.5),
            Vec3::new(0.3, 1.0, -0.5),
            Vec3::new(0.3, 0.0, 0.5),
        ];
        let mesh = Mesh {
            vertices: points
                .into_iter()
                .map(|pos| Vertex {
                    pos,
                    color: [130, 130, 130],
                })
                .collect(),
        };
        let start = super::SurfacePoint {
            position: Vec3::new(0.1, 0.0, 0.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
        };
        let end = super::walk_step(
            &mesh,
            start,
            Vec3::new(0.3, 0.0, 0.0),
            super::WalkProfile::Planar,
            0.04,
            0.2,
        );
        assert!(end.position.x > 0.1 && end.position.x < 0.27);
    }
    #[test]
    fn every_landing_has_a_walkable_neighbor() {
        for planet in super::planets() {
            let mesh = Mesh::load(planet.mesh).unwrap();
            let start = super::landing_point(&mesh, planet.surface);
            let profile = if matches!(planet.surface, super::Surface::Lava) {
                super::WalkProfile::Volcanic
            } else {
                super::WalkProfile::Planar
            };
            let forward = super::surface_forward(start.position, start.normal);
            let side = forward.cross(start.normal).norm();
            let directions = [forward, forward * -1.0, side, side * -1.0];
            assert!(
                directions.iter().any(|dir| {
                    let mut current = start;
                    for _ in 0..12 {
                        let next = super::walk_step(
                            &mesh,
                            current,
                            *dir * 0.01,
                            profile,
                            crate::avatar::RADIUS / planet.radius,
                            crate::avatar::HEIGHT / planet.radius,
                        );
                        if (next.position - current.position).len() < 0.001 {
                            return false;
                        }
                        current = next;
                    }
                    true
                }),
                "{} has no useful walkable path",
                planet.name
            );
        }
        let mesh = Mesh::load("starship").unwrap();
        let start = super::starship_landing(&mesh);
        let forward = super::surface_forward(start.position, start.normal);
        let mut current = start;
        for _ in 0..30 {
            let next = super::walk_step(
                &mesh,
                current,
                forward * 0.01,
                super::WalkProfile::Round,
                crate::avatar::RADIUS / super::STARSHIP_SCALE,
                crate::avatar::HEIGHT / super::STARSHIP_SCALE,
            );
            assert!((next.position - current.position).len() > 0.001);
            current = next;
        }
    }
}
