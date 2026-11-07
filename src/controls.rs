use crate::{
    math::Vec3,
    scene::{Mode, Planet, Ship},
};
use raylib::prelude::*;
pub fn update(
    rl: &RaylibHandle,
    dt: f32,
    time: f32,
    planets: &[Planet],
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
            ship.pos = p + Vec3::new(planets[i].radius + 6.5, 1.3, 0.0);
            ship.yaw = -std::f32::consts::FRAC_PI_2;
            ship.speed = 0.0;
            *mode = Mode::Space
        }
    }
    match mode {
        Mode::Space => {
            let turn = (rl.is_key_down(KEY_RIGHT) as i32 - rl.is_key_down(KEY_LEFT) as i32) as f32;
            ship.yaw += turn * 1.7 * dt;
            let thrust = (rl.is_key_down(KEY_W) as i32 - rl.is_key_down(KEY_S) as i32) as f32;
            let target = thrust
                * if rl.is_key_down(KEY_LEFT_SHIFT) {
                    29.0
                } else {
                    13.0
                };
            ship.speed += (target - ship.speed) * (dt * 2.0).min(1.0);
            ship.pos += ship.forward() * ship.speed * dt;
            let strafe = (rl.is_key_down(KEY_D) as i32 - rl.is_key_down(KEY_A) as i32) as f32;
            ship.pos += Vec3::new(ship.yaw.cos(), 0.0, ship.yaw.sin()) * strafe * 7.0 * dt;
            ship.pos.y += (rl.is_key_down(KEY_SPACE) as i32
                - rl.is_key_down(KEY_LEFT_CONTROL) as i32) as f32
                * 7.0
                * dt;
            for p in planets {
                let delta = ship.pos - p.position(time);
                let min = p.radius + 0.8;
                if delta.len() < min {
                    ship.pos = p.position(time) + delta.norm() * min;
                    ship.speed = 0.0
                }
            }
            if rl.is_key_pressed(KEY_E) {
                if let Some((i, _)) = planets.iter().enumerate().find(|(_, p)| {
                    (ship.pos - (p.position(time) + Vec3::new(p.radius + 1.2, 0.0, 0.0))).len()
                        < 5.0
                }) {
                    *mode = Mode::Surface {
                        planet: i,
                        radial: Vec3::new(1.0, 0.0, 0.0),
                        yaw: 0.0,
                    };
                    ship.speed = 0.0
                }
            }
        }
        Mode::Surface {
            planet,
            radial,
            yaw,
        } => {
            *yaw += (rl.is_key_down(KEY_RIGHT) as i32 - rl.is_key_down(KEY_LEFT) as i32) as f32
                * 1.8
                * dt;
            *radial = radial.rotate_y(planets[*planet].spin * dt);
            let north = (Vec3::new(0.0, 0.0, -1.0)
                - *radial * radial.dot(Vec3::new(0.0, 0.0, -1.0)))
            .norm();
            let side = radial.cross(north).norm();
            let f = north * yaw.cos() + side * yaw.sin();
            let step = (rl.is_key_down(KEY_W) as i32 - rl.is_key_down(KEY_S) as i32) as f32;
            let strafe = (rl.is_key_down(KEY_D) as i32 - rl.is_key_down(KEY_A) as i32) as f32;
            *radial = (*radial + (f * step + side * strafe) * dt * 0.45).norm();
            if rl.is_key_pressed(KEY_R) {
                let p = planets[*planet].position(time);
                ship.pos = p + *radial * (planets[*planet].radius + 2.5);
                ship.speed = 0.0;
                *mode = Mode::Space
            }
        }
    }
}
