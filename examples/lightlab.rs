//! Lighting test bed: a stand-in site lit by the real pipeline.
//!   cargo run --release --example lightlab -- --clock "2026-06-21 4:45pm" --screenshot out.png
//! Flags: --clock, --lat, --lon, --cover 0..1, --storm 0..1, --wet 0..1,
//!        --view wide|default|tele|low, --seed N, --screenshot FILE, --frames N
#[path = "../src/gfx.rs"]
mod gfx;
#[path = "../src/sky.rs"]
mod sky;

use gfx::{mat, Geo, Gfx, Lens};
use macroquad::prelude::*;

const TW: usize = 26;
const TILE: f32 = 1.65;
const STEP: f32 = 0.62;

fn conf() -> Conf {
    Conf { window_title: "lightlab".into(), window_width: 1920, window_height: 1080, sample_count: 1, ..Default::default() }
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}

struct Site {
    statics: Vec<Mesh>,
    gmap: Vec<u8>,
}

fn build(seed: u32) -> Site {
    let mut g = Geo::new();
    let mut gmap = vec![0u8; TW * TW * 4];
    let h = |x: i32, z: i32| -> f32 {
        let n = gfx::hash3(seed, (x / 5) as u32, (z / 6) as u32);
        let ridge = ((x as f32 * 0.31 + z as f32 * 0.17).sin() * 0.5 + 0.5) * 1.4 + n * 0.9;
        ((ridge - 0.9).max(0.0).floor()).min(2.0) * STEP
    };
    let hh = |x: i32, z: i32| if x < 0 || z < 0 || x >= TW as i32 || z >= TW as i32 { 0.0 } else { h(x, z) };
    // a wide apron so the site sits in country, not on a table
    let ap = 260.0;
    let w = TW as f32 * TILE;
    let grass = rgb(96, 116, 62);
    let apron = [
        (vec3(-ap, 0.0, -ap), vec3(w + ap, 0.0, 0.0)),
        (vec3(-ap, 0.0, w), vec3(w + ap, 0.0, w + ap)),
        (vec3(-ap, 0.0, 0.0), vec3(0.0, 0.0, w)),
        (vec3(w, 0.0, 0.0), vec3(w + ap, 0.0, w)),
    ];
    for (a, b) in apron {
        g.quad([vec3(a.x, 0.0, a.z), vec3(a.x, 0.0, b.z), vec3(b.x, 0.0, b.z), vec3(b.x, 0.0, a.z)], Vec3::Y, grass, [1.0; 4], mat::GROUND);
    }
    for z in 0..TW as i32 {
        for x in 0..TW as i32 {
            let y = h(x, z);
            let (x0, z0) = (x as f32 * TILE, z as f32 * TILE);
            let (x1, z1) = (x0 + TILE, z0 + TILE);
            // fields: plough, crop, pasture, track
            let field = gfx::hash3(seed ^ 77, (x / 7) as u32, (z / 9) as u32);
            let mut c = if field < 0.3 { rgb(122, 100, 70) } else if field < 0.62 { rgb(132, 138, 66) } else { grass };
            let mut ty = if field < 0.3 { 85u8 } else if field < 0.62 { 168 } else { 0 };
            if (z == 12 || z == 13) && y == 0.0 { c = rgb(128, 116, 96); ty = 255; }
            let o = (z as usize * TW + x as usize) * 4;
            gmap[o] = (c.r * 255.0) as u8; gmap[o + 1] = (c.g * 255.0) as u8; gmap[o + 2] = (c.b * 255.0) as u8; gmap[o + 3] = ty;
            // corner occlusion from higher neighbours
            let occ = |dx: i32, dz: i32| -> f32 {
                let up = [hh(x + dx, z), hh(x, z + dz), hh(x + dx, z + dz)].iter().filter(|v| **v > y + 0.1).count();
                1.0 - 0.22 * up as f32
            };
            g.quad([vec3(x0, y, z0), vec3(x0, y, z1), vec3(x1, y, z1), vec3(x1, y, z0)], Vec3::Y, c, [occ(-1, -1), occ(-1, 1), occ(1, 1), occ(1, -1)], mat::GROUND);
            let bank = rgb(104, 88, 64);
            let side = |g: &mut Geo, lo: f32, p: [Vec3; 2], n: Vec3| {
                if lo < y - 0.01 {
                    g.quad([vec3(p[0].x, lo, p[0].z), vec3(p[1].x, lo, p[1].z), vec3(p[1].x, y, p[1].z), vec3(p[0].x, y, p[0].z)], n, bank, [0.5, 0.5, 1.0, 1.0], mat::GROUND);
                }
            };
            side(&mut g, hh(x, z + 1), [vec3(x0, 0.0, z1), vec3(x1, 0.0, z1)], Vec3::Z);
            side(&mut g, hh(x, z - 1), [vec3(x1, 0.0, z0), vec3(x0, 0.0, z0)], -Vec3::Z);
            side(&mut g, hh(x + 1, z), [vec3(x1, 0.0, z1), vec3(x1, 0.0, z0)], Vec3::X);
            side(&mut g, hh(x - 1, z), [vec3(x0, 0.0, z0), vec3(x0, 0.0, z1)], -Vec3::X);

            let r = gfx::hash3(seed ^ 5, x as u32, z as u32);
            let (cx, cz) = (x0 + TILE * 0.5, z0 + TILE * 0.5);
            let on_track = z == 12 || z == 13;
            if r < 0.07 && !on_track {
                // tree
                let th = 2.6 + r * 30.0;
                g.column(vec3(cx, y, cz), vec3(cx, y + th * 0.55, cz), 0.16, 0.10, rgb(74, 60, 46), mat::BUILT);
                let lc = rgb(54 + (r * 300.0) as u8, 84, 40);
                g.blob(vec3(cx, y + th * 0.75, cz), vec3(1.5, th * 0.38, 1.5), lc, mat::FOLIAGE, x as u32 * 31 + z as u32);
                g.blob(vec3(cx + 0.7, y + th * 0.6, cz - 0.4), vec3(1.0, th * 0.25, 1.0), lc, mat::FOLIAGE, x as u32 * 17 + z as u32);
            } else if r > 0.955 && !on_track {
                // boulder
                g.blob(vec3(cx, y + 0.3, cz), vec3(0.6, 0.45, 0.5), rgb(120, 116, 108), mat::BUILT, x as u32 + 99 * z as u32);
            }
        }
    }
    // farmhouse with gabled roof
    let house = |g: &mut Geo, lo: Vec3, hi: Vec3, wall: Color, roof: Color| {
        g.boxy(lo, hi, wall, mat::BUILT, 0.55);
        let mid = (lo.x + hi.x) * 0.5;
        let top = hi.y + (hi.x - lo.x) * 0.32;
        let o = 0.25;
        let n1 = vec3(-(top - hi.y), mid - lo.x, 0.0).normalize();
        let n2 = vec3(top - hi.y, hi.x - mid, 0.0).normalize();
        g.quad([vec3(lo.x - o, hi.y - 0.1, lo.z - o), vec3(lo.x - o, hi.y - 0.1, hi.z + o), vec3(mid, top, hi.z + o), vec3(mid, top, lo.z - o)], n1, roof, [0.8, 0.8, 1.0, 1.0], mat::BUILT);
        g.quad([vec3(hi.x + o, hi.y - 0.1, hi.z + o), vec3(hi.x + o, hi.y - 0.1, lo.z - o), vec3(mid, top, lo.z - o), vec3(mid, top, hi.z + o)], n2, roof, [0.8, 0.8, 1.0, 1.0], mat::BUILT);
        for z in [lo.z, hi.z] {
            let n = if z == lo.z { -Vec3::Z } else { Vec3::Z };
            g.tri([vec3(lo.x, hi.y, z), vec3(hi.x, hi.y, z), vec3(mid, top, z)], [n; 3], wall, [0.9; 3], mat::BUILT);
        }
    };
    house(&mut g, vec3(6.0 * TILE, 0.0, 15.0 * TILE), vec3(10.0 * TILE, 3.0, 19.0 * TILE), rgb(186, 176, 160), rgb(120, 70, 56));
    house(&mut g, vec3(14.0 * TILE, 0.0, 16.0 * TILE), vec3(19.0 * TILE, 4.2, 22.0 * TILE), rgb(130, 60, 48), rgb(92, 96, 100));
    // tin shed and a low wall
    g.boxy(vec3(11.0 * TILE, 0.0, 8.0 * TILE), vec3(13.0 * TILE, 2.4, 10.0 * TILE), rgb(150, 156, 160), mat::METAL, 0.6);
    g.boxy(vec3(4.0 * TILE, 0.0, 14.0 * TILE - 0.2), vec3(12.0 * TILE, 0.9, 14.0 * TILE), rgb(150, 140, 126), mat::BUILT, 0.6);
    // people-sized figures along the track
    for k in 0..8 {
        let x = (5.0 + k as f32 * 2.1) * TILE;
        let z = 12.6 * TILE + (k % 3) as f32 * 0.7;
        let c = rgb(84, 92, 70);
        g.column(vec3(x - 0.1, 0.0, z), vec3(x - 0.1, 0.9, z), 0.09, 0.11, rgb(60, 62, 54), mat::SKIN);
        g.column(vec3(x + 0.1, 0.0, z), vec3(x + 0.1, 0.9, z), 0.09, 0.11, rgb(60, 62, 54), mat::SKIN);
        g.boxy(vec3(x - 0.22, 0.88, z - 0.13), vec3(x + 0.22, 1.5, z + 0.13), c, mat::SKIN, 0.8);
        g.blob(vec3(x, 1.66, z), vec3(0.12, 0.13, 0.12), rgb(70, 76, 60), mat::SKIN, k);
    }
    Site { statics: g.finish(), gmap }
}

#[macroquad::main(conf)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let num = |k: &str, d: f32| arg(k).and_then(|s| s.parse::<f32>().ok()).unwrap_or(d);
    let moment = arg("--clock").and_then(|s| sky::parse_clock(&s)).unwrap_or(sky::Moment::new(2026, 6, 21, 14.0 * 60.0));
    let (lat, lon) = (num("--lat", 48.0), num("--lon", 2.0));
    let (cover, storm, wet) = (num("--cover", 0.0), num("--storm", 0.0), num("--wet", 0.0));
    let seed = num("--seed", 7.0) as u32;
    let view = arg("--view").unwrap_or("default".into());
    let shot = arg("--screenshot");
    let frames = num("--frames", 12.0) as u32;
    let speed = num("--speed", 0.0); // game minutes per real second

    gl_set_drawcall_buffer_capacity(65_535, 400_000);
    let site = build(seed);
    let mut gfx = Gfx::new(1920, 1080);
    gfx.set_ground_map(TW as u16, TW as u16, &site.gmap, vec2(0.0, 0.0), vec2(TW as f32 * TILE, TW as f32 * TILE));
    let c = vec3(TW as f32 * TILE * 0.5, 0.6, TW as f32 * TILE * 0.5);
    let (range, elev, az, fov_deg) = match view.as_str() {
        "wide" => (420.0, 38.0f32, 215.0f32, 19.5f32),
        "tele" => (420.0, 30.0, 215.0, 3.4),
        "site" => (420.0, 36.0, 215.0, 6.2),
        "low" => (380.0, 16.0, 150.0, 7.5),
        "top" => (420.0, 62.0, 200.0, 8.0),
        _ => (420.0, 36.0, 215.0, 8.6),
    };
    let mut frame = 0u32;
    let mut t = 0.0f32;
    let mut ft_acc = 0.0f32;
    let mut ft_n = 0u32;
    loop {
        let dt = get_frame_time();
        t += dt;
        let m = moment.plus_seconds(t * speed * 60.0);
        let mut light = gfx::light_at(m, lat, lon, cover, storm, vec3(0.10, 0.12, 0.05));
        light.wet = wet;
        let (e, a) = (elev.to_radians(), az.to_radians());
        // slow drift of a stabilised long lens
        let drift = vec3((t * 0.21).sin(), (t * 0.17).cos(), 0.0) * 0.05f32.to_radians() * range;
        let pos = c + vec3(e.cos() * a.sin(), e.sin(), -e.cos() * a.cos()) * range;
        let cam = Camera3D { position: pos, target: c + drift, up: Vec3::Y, fovy: fov_deg.to_radians(), z_near: range - 160.0, z_far: range + 420.0, ..Default::default() };

        gfx.shadow_begin(&light, c, 36.0);
        for msh in &site.statics { draw_mesh(msh); }
        gfx.scene_begin(&cam, &light);
        for msh in &site.statics { draw_mesh(msh); }
        // see-through test: a smoke puff, a move marker and a tracer
        gfx.overlay_begin(&light, 0.0);
        draw_cube(c + vec3(6.0, 1.2, 4.0), vec3(1.6, 1.2, 1.6), None, Color::new(0.5, 0.5, 0.52, 0.45));
        draw_cube(c + vec3(-4.0, 0.03, 6.0), vec3(1.5, 0.02, 1.5), None, Color::new(0.22, 0.68, 0.82, 0.35));
        gfx.overlay_begin(&light, 6.0);
        draw_line_3d(c + vec3(-8.0, 1.2, 8.0), c + vec3(4.0, 1.2, 9.0), Color::new(1.0, 0.8, 0.3, 1.0));
        draw_cube(c + vec3(8.0, 0.5, 8.0), vec3(0.5, 0.9, 0.5), None, Color::new(1.0, 0.45, 0.06, 0.9));
        let lens = Lens { adapt_secs: if shot.is_some() { 0.0 } else { 1.5 }, vignette_stops: 0.25, ..Default::default() };
        gfx.finish(dt, &lens);

        if frame > 30 { ft_acc += dt; ft_n += 1; }
        if shot.is_none() {
            let sun = m.sun_at(lat, lon);
            draw_text(&format!("{}  sun {:.0}° az {:.0}°  {:.2} ms", sky::clock_text(m.minute), sun.elevation.to_degrees(), sun.azimuth.to_degrees(), dt * 1000.0), 16.0, 28.0, 24.0, WHITE);
        }
        frame += 1;
        if let Some(p) = &shot {
            if frame >= frames {
                get_screen_data().export_png(p);
                if ft_n > 0 { println!("frame_ms {:.2}", ft_acc / ft_n as f32 * 1000.0); }
                std::process::exit(0);
            }
        }
        if is_key_pressed(KeyCode::Escape) { break; }
        next_frame().await;
    }
}
