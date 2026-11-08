use crate::{
    asset::Mesh,
    avatar,
    math::Vec3,
    scene::{
        self, Mode, Placement, Planet, STARSHIP_SCALE, Ship, SurfacePoint, SurfaceTarget,
        WalkProfile,
    },
};
use raylib::prelude::*;
use std::collections::HashMap;
pub fn update(
    rl: &RaylibHandle,
    dt: f32,
    time: f32,
    planets: &[Planet],
    meshes: &HashMap<&str, Mesh>,
    decorations: &[Vec<Placement>],
    landings: &[SurfacePoint],
    starship_landing: SurfacePoint,
    ship: &mut Ship,
    mode: &mut Mode,
) {
    use KeyboardKey::*;
    for (i, key) in [KEY_ONE, KEY_TWO, KEY_THREE, KEY_FOUR, KEY_FIVE]
        .iter()
        .enumerate()
    {
        if rl.is_key_pressed(*key) {
            let p = planets[i].position(time);
            ship.pos = p + Vec3::new(planets[i].radius + 6.0, 2.0, 0.0);
            ship.yaw = -std::f32::consts::FRAC_PI_2;
            ship.speed = 0.0;
            *mode = Mode::Space
        }
    }
    match mode {
        Mode::Space => {
            if rl.is_key_pressed(KEY_C) {
                ship.camera_far = !ship.camera_far;
            }
            let turn = (rl.is_key_down(KEY_RIGHT) as i32 - rl.is_key_down(KEY_LEFT) as i32) as f32;
            ship.yaw += turn * 1.7 * dt;
            let thrust = (rl.is_key_down(KEY_W) as i32 - rl.is_key_down(KEY_S) as i32) as f32;
            let target = thrust
                * if rl.is_key_down(KEY_LEFT_SHIFT) {
                    100.0
                } else {
                    35.0
                };
            ship.speed += (target - ship.speed) * (dt * 2.0).min(1.0);
            ship.pos += ship.forward() * ship.speed * dt;
            let strafe = (rl.is_key_down(KEY_D) as i32 - rl.is_key_down(KEY_A) as i32) as f32;
            ship.pos += Vec3::new(ship.yaw.cos(), 0.0, ship.yaw.sin()) * strafe * 25.0 * dt;
            ship.pos.y += (rl.is_key_down(KEY_SPACE) as i32
                - rl.is_key_down(KEY_LEFT_CONTROL) as i32) as f32
                * 25.0
                * dt;
            for p in planets {
                let delta = ship.pos - p.position(time);
                let min = p.radius + STARSHIP_SCALE + 1.0;
                if delta.len() < min {
                    ship.pos = p.position(time) + delta.norm() * min;
                    ship.speed = 0.0
                }
            }
            if rl.is_key_pressed(KEY_E) {
                if let Some((i, _)) = planets.iter().enumerate().find(|(_, p)| {
                    (ship.pos - (p.position(time) + Vec3::new(p.radius + 2.0, 0.0, 0.0))).len()
                        < 7.0
                }) {
                    let landing = landings[i];
                    *mode = Mode::Surface {
                        target: SurfaceTarget::Planet(i),
                        local: landing.position + landing.normal * 0.015,
                        normal: landing.normal,
                        yaw: 0.0,
                    };
                    ship.speed = 0.0
                }
            }
            if rl.is_key_pressed(KEY_F) {
                *mode = Mode::Surface {
                    target: SurfaceTarget::Starship,
                    local: starship_landing.position + starship_landing.normal * 0.015,
                    normal: starship_landing.normal,
                    yaw: std::f32::consts::PI,
                };
                ship.speed = 0.0;
                ship.camera_far = false;
            }
        }
        Mode::Surface {
            target,
            local,
            normal,
            yaw,
        } => {
            *yaw += (rl.is_key_down(KEY_RIGHT) as i32 - rl.is_key_down(KEY_LEFT) as i32) as f32
                * 1.8
                * dt;
            let north = scene::surface_forward(*local, *normal);
            let side = north.cross(*normal).norm();
            let f = north * yaw.cos() + side * yaw.sin();
            let step = (rl.is_key_down(KEY_W) as i32 - rl.is_key_down(KEY_S) as i32) as f32;
            let strafe = (rl.is_key_down(KEY_D) as i32 - rl.is_key_down(KEY_A) as i32) as f32;
            let (mesh, scale, profile) = match target {
                SurfaceTarget::Planet(i) => (
                    &meshes[planets[*i].mesh],
                    planets[*i].radius,
                    if matches!(planets[*i].surface, scene::Surface::Lava) {
                        WalkProfile::Volcanic
                    } else {
                        WalkProfile::Planar
                    },
                ),
                SurfaceTarget::Starship => {
                    (&meshes["starship"], STARSHIP_SCALE, WalkProfile::Round)
                }
            };
            let delta = (f * step + f.cross(*normal).norm() * strafe) * dt * (3.5 / scale);
            let previous = SurfacePoint {
                position: *local - *normal * 0.015,
                normal: *normal,
            };
            let hit = scene::walk_step(
                mesh,
                previous,
                delta,
                profile,
                avatar::RADIUS / scale,
                avatar::HEIGHT / scale,
            );
            let prop_collision = if let SurfaceTarget::Planet(i) = target {
                decorations[*i].iter().enumerate().any(|(j, prop)| {
                    let solid = match planets[*i].surface {
                        scene::Surface::Grass => j % 3 == 2,
                        scene::Surface::Water => j % 3 == 0,
                        _ => true,
                    };
                    let difference = hit.position - prop.position;
                    let lateral = difference - prop.normal * difference.dot(prop.normal);
                    solid
                        && difference.dot(prop.normal).abs() < avatar::HEIGHT / scale
                        && lateral.len() < avatar::RADIUS / scale + prop.size * 0.16
                })
            } else {
                false
            };
            if !prop_collision && (hit.position - previous.position).len() > 1e-5 {
                *normal = (*normal * 0.65 + hit.normal * 0.35).norm();
                *local = hit.position + *normal * 0.015;
            }
            if rl.is_key_pressed(KEY_R) {
                *mode = Mode::Returning {
                    target: *target,
                    local: *local,
                    normal: *normal,
                    yaw: *yaw,
                    remaining: 0.7,
                };
            }
        }
        Mode::Returning {
            target,
            local,
            normal,
            remaining,
            ..
        } => {
            *remaining -= dt;
            if *remaining <= 0.0 {
                if let SurfaceTarget::Planet(i) = target {
                    let p = planets[*i].position(time);
                    let rotation = time * planets[*i].spin;
                    let up = normal.rotate_y(rotation);
                    ship.pos = p + local.rotate_y(rotation) * planets[*i].radius + up * 4.0;
                    ship.yaw = up.x.atan2(-up.z);
                }
                ship.speed = 0.0;
                *mode = Mode::Space;
            }
        }
    }
}
