mod asset;
mod controls;
mod math;
mod renderer;
mod scene;
use asset::Mesh;
use math::Vec3;
use raylib::prelude::*;
use renderer::{Camera, H, Renderer, Shader, W};
use scene::{Mode, Ship, Surface};
use std::collections::HashMap;
fn main() {
    let names = [
        "ring",
        "yoshi",
        "fluffy",
        "ice",
        "lava",
        "fortress",
        "yoshi_low",
        "fluffy_low",
        "ice_low",
        "fortress_low",
        "bush",
        "flower",
        "pipe",
        "fountain",
        "geyser",
        "crate",
        "falling_crate",
        "starship",
        "starship_low",
    ];
    let mut meshes: HashMap<&str, Mesh> = names
        .into_iter()
        .map(|n| {
            (
                n,
                Mesh::load(n).unwrap_or_else(|e| {
                    panic!("asset {n}: {e}. Run python3 tools/convert_assets.py")
                }),
            )
        })
        .collect();
    meshes.insert("sun", Mesh::sphere());
    let skies: HashMap<&str, Vec<u8>> = [
        "galaxy",
        "yoshi_surface",
        "fluffy_surface",
        "ice_surface",
        "lava_surface",
        "fortress_surface",
    ]
    .into_iter()
    .map(|n| {
        (
            n,
            std::fs::read(format!("assets/processed/{n}.sky")).expect("sky asset"),
        )
    })
    .collect();
    let planets = scene::planets();
    let landings: Vec<_> = planets
        .iter()
        .map(|p| scene::landing_point(&meshes[p.mesh], p.surface))
        .collect();
    let decorations: Vec<_> = planets
        .iter()
        .enumerate()
        .map(|(i, p)| scene::surface_placements(&meshes[p.mesh], 0xCAFE_2026 + i as u32))
        .collect();
    let mut ship = Ship {
        pos: planets[0].position(0.0) + Vec3::new(planets[0].radius + 6.0, 2.0, 0.0),
        yaw: -std::f32::consts::FRAC_PI_2,
        speed: 0.0,
        camera_far: false,
    };
    let mut mode = Mode::Space;
    let mut renderer = Renderer::new();
    let mut time = 0.0f32;
    if std::env::args().any(|a| a == "--render-video") {
        render_video(
            &mut renderer,
            &meshes,
            &skies,
            &planets,
            &decorations,
            &landings,
        );
        return;
    }
    if std::env::args().any(|a| a == "--snapshot-far") {
        ship.camera_far = true;
    }
    if let Some(surface_arg) = std::env::args().find(|a| a.starts_with("--snapshot-surface")) {
        let planet = surface_arg
            .split('=')
            .nth(1)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0)
            .min(planets.len() - 1);
        mode = Mode::Surface {
            planet,
            local: landings[planet].position + landings[planet].normal * 0.015,
            normal: landings[planet].normal,
            yaw: 0.0,
        };
    }
    if std::env::args().any(|a| a.starts_with("--snapshot")) {
        render_frame(
            &mut renderer,
            &meshes,
            &skies,
            &planets,
            &decorations,
            &ship,
            mode,
            0.0,
        );
        let mut ppm = format!("P6\n{} {}\n255\n", W, H).into_bytes();
        for px in renderer.pixels.chunks_exact(4) {
            ppm.extend_from_slice(&px[..3]);
        }
        std::fs::write("/tmp/space_travel.ppm", ppm).expect("snapshot");
        println!("/tmp/space_travel.ppm");
        return;
    }
    let (mut rl, thread) = raylib::init()
        .size(1280, 720)
        .title("Space Travel | Galaxy Tour")
        .resizable()
        .build();
    rl.set_target_fps(60);
    let mut texture = rl
        .load_texture_from_image(
            &thread,
            &Image::gen_image_color(W as i32, H as i32, Color::BLACK),
        )
        .expect("framebuffer texture");
    while !rl.window_should_close() {
        let dt = rl.get_frame_time().min(0.05);
        time += dt;
        controls::update(
            &rl, dt, time, &planets, &meshes, &landings, &mut ship, &mut mode,
        );
        render_frame(
            &mut renderer,
            &meshes,
            &skies,
            &planets,
            &decorations,
            &ship,
            mode,
            time,
        );
        texture
            .update_texture(&renderer.pixels)
            .expect("upload framebuffer");
        let sw = rl.get_screen_width();
        let sh = rl.get_screen_height();
        let fps = rl.get_fps();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(
            &texture,
            Rectangle::new(0.0, 0.0, W as f32, H as f32),
            Rectangle::new(0.0, 0.0, sw as f32, sh as f32),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        d.draw_rectangle(10, 10, sw.min(600) - 20, 88, Color::new(4, 7, 23, 210));
        d.draw_text("SPACE TRAVEL  /  GALAXY TOUR", 22, 18, 22, Color::GOLD);
        match mode {
            Mode::Space => {
                d.draw_text(
                    "W/S acelerar  A/D lateral  Flechas girar  Espacio/Ctrl altura",
                    22,
                    49,
                    15,
                    Color::RAYWHITE,
                );
                d.draw_text(
                    "Shift impulso  C camara lejana  1-5 viajar  E anillo",
                    22,
                    69,
                    15,
                    Color::SKYBLUE,
                );
            }
            Mode::Surface { planet, .. } => {
                d.draw_text(
                    &format!(
                        "Explorando {}  |  W/S/A/D caminar  Flechas orientar",
                        planets[planet].name
                    ),
                    22,
                    49,
                    15,
                    Color::RAYWHITE,
                );
                d.draw_text(
                    "R invocar anillo estelar y volver a la nave",
                    22,
                    69,
                    15,
                    Color::SKYBLUE,
                );
            }
            Mode::Returning { planet, .. } => {
                d.draw_text(
                    &format!("Anillo estelar invocado en {}", planets[planet].name),
                    22,
                    49,
                    15,
                    Color::RAYWHITE,
                );
                d.draw_text("Regresando a la nave...", 22, 69, 15, Color::SKYBLUE);
            }
        }
        d.draw_text(
            &format!(
                "{} FPS  |  {} triangulos  |  {} descartados",
                fps, renderer.triangles, renderer.culled
            ),
            14,
            sh - 28,
            16,
            Color::LIGHTGRAY,
        );
    }
}

fn render_frame(
    renderer: &mut Renderer,
    meshes: &HashMap<&str, Mesh>,
    skies: &HashMap<&str, Vec<u8>>,
    planets: &[scene::Planet],
    decorations: &[Vec<scene::Placement>],
    ship: &Ship,
    mode: Mode,
    time: f32,
) {
    let cam = match mode {
        Mode::Space => {
            let f = ship.forward();
            Camera {
                pos: ship.pos - f * if ship.camera_far { 13.0 } else { 4.5 }
                    + Vec3::new(0.0, if ship.camera_far { 5.0 } else { 3.0 }, 0.0),
                target: ship.pos + f * 5.0,
                up: Vec3::new(0.0, 1.0, 0.0),
            }
        }
        Mode::Surface {
            planet,
            local,
            normal,
            yaw,
        }
        | Mode::Returning {
            planet,
            local,
            normal,
            yaw,
            ..
        } => {
            let p = &planets[planet];
            let rotation = time * p.spin;
            let floor = p.position(time) + local.rotate_y(rotation) * p.radius;
            let up = normal.rotate_y(rotation).norm();
            let north = scene::surface_forward(local, normal);
            let side = north.cross(normal).norm();
            let forward = (north * yaw.cos() + side * yaw.sin()).rotate_y(rotation);
            let eye = floor + up * (p.radius * 0.35);
            let look_ahead = if matches!(p.surface, Surface::Lava) {
                0.12
            } else {
                0.35
            };
            Camera {
                pos: eye,
                target: floor + forward * (p.radius * look_ahead),
                up,
            }
        }
    };
    let surface_planet = match mode {
        Mode::Surface { planet, .. } | Mode::Returning { planet, .. } => Some(planet),
        Mode::Space => None,
    };
    let overlay = surface_planet.map(|i| {
        let name = [
            "yoshi_surface",
            "fluffy_surface",
            "ice_surface",
            "lava_surface",
            "fortress_surface",
        ][i];
        skies[name].as_slice()
    });
    renderer.clear(
        time,
        cam,
        &skies["galaxy"],
        overlay,
        surface_planet.map(|i| planets[i].surface),
    );
    for p in planets {
        let mut last = None;
        for i in 0..=90 {
            let a = i as f32 / 90.0 * std::f32::consts::TAU;
            let point = Vec3::new(a.cos() * p.orbit, 0.0, a.sin() * p.orbit);
            if let Some(prev) = last {
                renderer.draw_line(cam, prev, point, [47, 45, 83]);
            }
            last = Some(point)
        }
    }
    renderer.draw_mesh(
        &meshes["sun"],
        cam,
        Vec3::new(0.0, 0.0, 0.0),
        13.0,
        time * 0.2,
        Shader::Sun,
        time,
    );
    for (i, p) in planets.iter().enumerate() {
        let position = p.position(time);
        let dist = (position - cam.pos).len();
        let low = dist > p.radius * 3.0;
        let mesh = if low { p.low } else { p.mesh };
        let props: &[&str] = match p.surface {
            Surface::Grass => &["bush", "flower", "pipe"],
            Surface::Water => &["fountain", "flower", "bush"],
            Surface::Ice => &["crate", "pipe"],
            Surface::Lava => &["geyser", "crate"],
            Surface::Metal => &["falling_crate", "pipe", "crate"],
        };
        if dist < p.radius * 2.3 {
            for (j, placement) in decorations[i].iter().enumerate() {
                let radial = placement.normal.rotate_y(time * p.spin);
                let at = position
                    + placement.position.rotate_y(time * p.spin) * p.radius
                    + radial * 0.05;
                renderer.draw_mesh_oriented(
                    &meshes[props[j % props.len()]],
                    cam,
                    at,
                    placement.size * p.radius * 0.2,
                    j as f32 * 0.7,
                    radial,
                    Shader::Planet(p.surface),
                    time,
                );
            }
        }
        renderer.draw_mesh(
            &meshes[mesh],
            cam,
            position,
            p.radius,
            time * p.spin,
            Shader::Planet(p.surface),
            time,
        );
        // Launch star sits above each planet's visible hemisphere.
        if !matches!(mode, Mode::Surface {planet: active,..} | Mode::Returning {planet: active,..} if active == i)
        {
            renderer.draw_mesh(
                &meshes["ring"],
                cam,
                position + Vec3::new(p.radius + 2.0, 0.0, 0.0),
                0.75,
                time * 1.6,
                Shader::Ring,
                time,
            );
        }
        if matches!(p.surface, Surface::Lava) {
            renderer.draw_line(
                cam,
                position + Vec3::new(0.0, p.radius, 0.0),
                position + Vec3::new(0.0, p.radius + 2.0 + (time * 2.0).sin() * 0.5, 0.0),
                [255, 85, 18],
            );
        }
    }
    if let Mode::Returning {
        planet,
        normal,
        remaining,
        ..
    } = mode
    {
        let direction = (cam.target - cam.pos).norm();
        let up = normal.rotate_y(time * planets[planet].spin);
        renderer.draw_mesh_oriented(
            &meshes["ring"],
            cam,
            cam.pos + direction * 1.8,
            0.2 + (0.7 - remaining) * 0.45,
            time * 3.0,
            up,
            Shader::Ring,
            time,
        );
    }
    if matches!(mode, Mode::Space) {
        renderer.draw_mesh(
            &meshes[if ship.camera_far {
                "starship_low"
            } else {
                "starship"
            }],
            cam,
            ship.pos,
            1.0,
            ship.yaw,
            Shader::Ship,
            time,
        )
    }
}

fn render_video(
    renderer: &mut Renderer,
    meshes: &HashMap<&str, Mesh>,
    skies: &HashMap<&str, Vec<u8>>,
    planets: &[scene::Planet],
    decorations: &[Vec<scene::Placement>],
    landings: &[scene::SurfacePoint],
) {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    std::fs::create_dir_all("media").expect("media directory");
    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-y",
            "-loglevel",
            "error",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb24",
            "-video_size",
            "640x360",
            "-framerate",
            "12",
            "-i",
            "-",
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-crf",
            "20",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            "media/galaxy-tour.mp4",
        ])
        .stdin(Stdio::piped())
        .spawn()
        .expect("ffmpeg is needed to create the video");
    let mut input = ffmpeg.stdin.take().expect("ffmpeg input");
    let mut frame = vec![0u8; W * H * 3];
    for (i, p) in planets.iter().enumerate() {
        for j in 0..60 {
            let u = j as f32 / 60.0;
            let time = i as f32 * 6.0 + u * 6.0;
            let center = p.position(time);
            let (ship, mode) = if u >= 0.55 && u < 0.78 {
                let landing = landings[i];
                let local = landing.position + landing.normal * 0.015;
                (
                    Ship {
                        pos: center
                            + local.rotate_y(time * p.spin) * p.radius
                            + landing.normal.rotate_y(time * p.spin) * 4.0,
                        yaw: 0.0,
                        speed: 0.0,
                        camera_far: false,
                    },
                    Mode::Surface {
                        planet: i,
                        local,
                        normal: landing.normal,
                        yaw: (u - 0.55) * 3.0,
                    },
                )
            } else {
                let a = if u < 0.55 { (u / 0.55) * 2.3 } else { 2.3 };
                let radial = Vec3::new(a.cos(), 0.12, a.sin()).norm();
                let distance = if u < 0.25 {
                    p.radius * 3.5 - (u / 0.25) * (p.radius * 2.5 - 3.0)
                } else if u < 0.78 {
                    p.radius + 3.0
                } else {
                    p.radius + 3.0 + (u - 0.78) / 0.22 * p.radius * 2.5
                };
                let pos = center + radial * distance;
                let yaw = (-radial.x).atan2(radial.z);
                (
                    Ship {
                        pos,
                        yaw,
                        speed: 0.0,
                        camera_far: u < 0.12,
                    },
                    Mode::Space,
                )
            };
            render_frame(
                renderer,
                meshes,
                skies,
                planets,
                decorations,
                &ship,
                mode,
                time,
            );
            for (idx, px) in renderer.pixels.chunks_exact(4).enumerate() {
                frame[idx * 3..idx * 3 + 3].copy_from_slice(&px[..3]);
            }
            input.write_all(&frame).expect("encode frame");
        }
        println!("Rendered {} / 5 planets", i + 1);
    }
    drop(input);
    assert!(
        ffmpeg.wait().expect("ffmpeg").success(),
        "video encoder failed"
    );
    println!("media/galaxy-tour.mp4");
}
