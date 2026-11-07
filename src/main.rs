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
    meshes.insert("ship", Mesh::ship());
    meshes.insert("grass_core", Mesh::sphere_colored([53, 143, 66]));
    meshes.insert("water_core", Mesh::sphere_colored([28, 110, 177]));
    meshes.insert("ice_core", Mesh::sphere_colored([130, 205, 231]));
    meshes.insert("lava_core", Mesh::sphere_colored([153, 52, 29]));
    meshes.insert("metal_core", Mesh::sphere_colored([111, 132, 151]));
    let skies: HashMap<&str, Vec<u8>> = [
        "galaxy",
        "yoshi_sky",
        "fluffy_sky",
        "ice_sky",
        "lava_sky",
        "fortress_sky",
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
    let mut ship = Ship {
        pos: Vec3::new(24.0, 1.5, 0.0),
        yaw: -std::f32::consts::FRAC_PI_2,
        speed: 0.0,
    };
    let mut mode = Mode::Space;
    let mut renderer = Renderer::new();
    let mut time = 0.0f32;
    if std::env::args().any(|a| a == "--render-video") {
        render_video(&mut renderer, &meshes, &skies, &planets);
        return;
    }
    if std::env::args().any(|a| a == "--snapshot-surface") {
        mode = Mode::Surface {
            planet: 0,
            radial: Vec3::new(1.0, 0.0, 0.0),
            yaw: 0.0,
        };
    }
    if std::env::args().any(|a| a.starts_with("--snapshot")) {
        render_frame(&mut renderer, &meshes, &skies, &planets, &ship, mode, 0.0);
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
        controls::update(&rl, dt, time, &planets, &mut ship, &mut mode);
        render_frame(&mut renderer, &meshes, &skies, &planets, &ship, mode, time);
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
                    "Shift impulso  1-5 viajar  E anillo estelar cercano",
                    22,
                    69,
                    15,
                    Color::SKYBLUE,
                );
            }
            Mode::Surface { planet, .. } => {
                d.draw_text(
                    &format!(
                        "Explorando {}  |  W/S/A/D caminar  Flechas girar",
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
    ship: &Ship,
    mode: Mode,
    time: f32,
) {
    let cam = match mode {
        Mode::Space => {
            let f = ship.forward();
            Camera {
                pos: ship.pos - f * 4.5 + Vec3::new(0.0, 3.0, 0.0),
                target: ship.pos + f * 5.0,
                up: Vec3::new(0.0, 1.0, 0.0),
            }
        }
        Mode::Surface {
            planet,
            radial,
            yaw,
        }
        | Mode::Returning {
            planet,
            radial,
            yaw,
            ..
        } => {
            let p = planets[planet].position(time);
            let eye = p + radial * (planets[planet].radius * 0.65);
            let north =
                (Vec3::new(0.0, 0.0, -1.0) - radial * radial.dot(Vec3::new(0.0, 0.0, -1.0))).norm();
            let side = north.cross(radial).norm();
            let dir = north * yaw.cos() + side * yaw.sin();
            Camera {
                pos: eye,
                target: eye + dir * 4.0 - radial * 1.5,
                up: radial,
            }
        }
    };
    let nearest = planets
        .iter()
        .enumerate()
        .map(|(i, p)| (i, (p.position(time) - cam.pos).len()))
        .min_by(|a, b| a.1.total_cmp(&b.1));
    let sky = if let Some((i, d)) = nearest {
        if d < planets[i].radius + 18.0 {
            Some(i)
        } else {
            None
        }
    } else {
        None
    };
    let key = match sky {
        Some(0) => "yoshi_sky",
        Some(1) => "fluffy_sky",
        Some(2) => "ice_sky",
        Some(3) => "lava_sky",
        Some(4) => "fortress_sky",
        _ => "galaxy",
    };
    renderer.clear(time, cam, &skies[key], sky.map(|i| planets[i].surface));
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
        4.0,
        time * 0.2,
        Shader::Sun,
        time,
    );
    for (i, p) in planets.iter().enumerate() {
        let position = p.position(time);
        let dist = (position - cam.pos).len();
        let low = dist > 17.0;
        let mesh = if low { p.low } else { p.mesh };
        let core = match p.surface {
            Surface::Grass => "grass_core",
            Surface::Water => "water_core",
            Surface::Ice => "ice_core",
            Surface::Lava => "lava_core",
            Surface::Metal => "metal_core",
        };
        renderer.draw_mesh(
            &meshes[core],
            cam,
            position,
            p.radius * 0.55,
            time * p.spin,
            Shader::Planet(p.surface),
            time,
        );
        if dist < 18.0 {
            let props: &[&str] = match p.surface {
                Surface::Grass => &["bush", "flower", "pipe"],
                Surface::Water => &["fountain", "flower", "bush"],
                Surface::Ice => &["crate", "pipe"],
                Surface::Lava => &["geyser", "crate"],
                Surface::Metal => &["falling_crate", "pipe", "crate"],
            };
            for j in 0..28 {
                let fy = 1.0 - 2.0 * (j as f32 + 0.5) / 28.0;
                let a = j as f32 * 2.3999632 + i as f32 * 0.79;
                let r = (1.0 - fy * fy).sqrt();
                let radial = Vec3::new(r * a.cos(), fy, r * a.sin()).rotate_y(time * p.spin);
                let size = 0.12 + (j % 4) as f32 * 0.025;
                let at = position + radial * (p.radius * 0.55 + size * 0.3);
                renderer.draw_mesh_oriented(
                    &meshes[props[j % props.len()]],
                    cam,
                    at,
                    size,
                    a,
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
                position + Vec3::new(p.radius + 1.2, 0.0, 0.0),
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
        radial, remaining, ..
    } = mode
    {
        let direction = (cam.target - cam.pos).norm();
        renderer.draw_mesh_oriented(
            &meshes["ring"],
            cam,
            cam.pos + direction * 1.8,
            0.2 + (0.7 - remaining) * 0.45,
            time * 3.0,
            radial,
            Shader::Ring,
            time,
        );
    }
    if matches!(mode, Mode::Space) {
        renderer.draw_mesh(
            &meshes["ship"],
            cam,
            ship.pos,
            0.85,
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
                let a = (u - 0.55) / 0.23 * 0.75;
                let radial = Vec3::new(a.cos(), 0.0, a.sin());
                (
                    Ship {
                        pos: center + radial * (p.radius + 4.0),
                        yaw: 0.0,
                        speed: 0.0,
                    },
                    Mode::Surface {
                        planet: i,
                        radial,
                        yaw: 0.25,
                    },
                )
            } else {
                let a = if u < 0.55 { (u / 0.55) * 2.3 } else { 2.3 };
                let radial = Vec3::new(a.cos(), 0.12, a.sin()).norm();
                let distance = if u < 0.25 {
                    14.0 - (u / 0.25) * 8.0
                } else if u < 0.78 {
                    p.radius + 2.5
                } else {
                    p.radius + 2.5 + (u - 0.78) / 0.22 * 14.0
                };
                let pos = center + radial * distance;
                let yaw = (-radial.x).atan2(radial.z);
                (
                    Ship {
                        pos,
                        yaw,
                        speed: 0.0,
                    },
                    Mode::Space,
                )
            };
            render_frame(renderer, meshes, skies, planets, &ship, mode, time);
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
