use macroquad::prelude::*;
use macroquad::miniquad::{UniformDesc, UniformType, PipelineParams, BlendState, BlendFactor, BlendValue, Equation};
use crate::data::GameData;

const VERT: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
uniform mat4 Model;
uniform mat4 Projection;
varying vec2 uv;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}"#;

// One full-screen pass. Everything is lit from the real sun direction: the
// albedo is Natural Earth 1:50m shaded relief, the packed map supplies the
// exact coastline, biome and a 32-pixel coastal falloff. Water gets an
// animated normal, Fresnel sky reflection, sun glitter and shore foam.
const FRAG: &str = r#"#version 100
precision highp float;
varying vec2 uv;
uniform vec2 resolution;
uniform vec2 center;
uniform float radius;
uniform float yaw;
uniform float pitch;
uniform float time;
uniform vec3 sun_dir;
uniform vec2 texel;
uniform sampler2D World;
uniform sampler2D WorldData;
uniform sampler2D Lights;

float hash21(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}
float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float a = hash21(i);
    float b = hash21(i + vec2(1.0, 0.0));
    float c = hash21(i + vec2(0.0, 1.0));
    float d = hash21(i + vec2(1.0, 1.0));
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}
float fbm(vec2 p) {
    float v = 0.0;
    float a = 0.5;
    mat2 rot = mat2(0.80, -0.60, 0.60, 0.80);
    for (int i = 0; i < 5; i++) {
        v += a * noise(p);
        p = rot * p * 2.03 + vec2(13.7, 7.1);
        a *= 0.5;
    }
    return v;
}
float lum(vec3 c) { return dot(c, vec3(0.299, 0.587, 0.114)); }
vec3 biome_tint(float t) {
    if (t < 0.5)  return vec3(0.075, 0.215, 0.075);   // forest
    if (t < 1.5)  return vec3(0.285, 0.320, 0.125);   // farmland
    if (t < 2.5)  return vec3(0.315, 0.335, 0.140);
    if (t < 3.5)  return vec3(0.345, 0.335, 0.150);
    if (t < 4.5)  return vec3(0.410, 0.350, 0.205);
    if (t < 5.5)  return vec3(0.300, 0.285, 0.250);   // mountain rock
    if (t < 6.5)  return vec3(0.050, 0.205, 0.060);   // jungle
    if (t < 7.5)  return vec3(0.555, 0.460, 0.275);   // desert
    if (t < 8.5)  return vec3(0.520, 0.430, 0.260);
    if (t < 9.5)  return vec3(0.760, 0.820, 0.875);   // polar
    if (t < 10.5) return vec3(0.085, 0.235, 0.095);   // taiga
    if (t < 11.5) return vec3(0.310, 0.330, 0.255);   // tundra
    return vec3(0.145, 0.260, 0.300);
}
vec3 rot_inv(vec3 q) {
    float cy = cos(yaw), sy = sin(yaw), cp = cos(pitch), sp = sin(pitch);
    float y = q.y * cp + q.z * sp;
    float z1 = -q.y * sp + q.z * cp;
    float x = q.x * cy - z1 * sy;
    float z = q.x * sy + z1 * cy;
    return vec3(x, y, z);
}

void main() {
    vec2 fc = gl_FragCoord.xy;
    // the frame is drawn with y running down the screen; centre and markers use the same
    vec2 fcs = vec2(fc.x, resolution.y - fc.y);
    vec2 d = (fcs - center) / radius;
    float r2 = dot(d, d);
    float r = sqrt(max(r2, 0.0));
    vec3 col = vec3(0.0025, 0.0060, 0.0140);

    // deep space: fine stars and a faint dust lane
    vec2 sp = floor(fc / 1.6);
    float sh = hash21(sp);
    if (sh > 0.9972) {
        float tw = 0.55 + 0.45 * sin(time * 1.4 + sh * 91.0);
        col += vec3(0.55, 0.68, 0.98) * tw * (0.30 + 0.70 * hash21(sp + 3.7));
    }
    float neb = fbm(fc * 0.0020 + vec2(3.0, 1.0));
    col += vec3(0.018, 0.010, 0.042) * pow(neb, 3.0) * 2.6;
    col += vec3(0.000, 0.009, 0.016) * pow(fbm(fc * 0.004 + 9.0), 2.0) * 3.0;

    if (r2 < 1.0) {
        vec3 q = vec3(d.x, -d.y, sqrt(max(0.0, 1.0 - r2)));
        vec3 n = rot_inv(q);
        float lat = asin(clamp(n.y, -1.0, 1.0));
        float lon = atan(n.x, n.z);
        vec2 tuv = vec2(fract(lon / 6.28318530718 + 0.5), 0.5 - lat / 3.14159265359);

        vec3 alb = texture2D(World, tuv).rgb;
        vec4 w = texture2D(WorldData, tuv);
        float land = smoothstep(0.38, 0.62, w.r);
        float terr = floor(w.g * 15.0 + 0.5);
        float coast = w.b;

        float detail = fbm(tuv * vec2(180.0, 90.0) + vec2(0.0, 1.0));
        float fine = fbm(tuv * vec2(300.0, 150.0));
        float ndl = dot(n, sun_dir);
        float day = smoothstep(-0.13, 0.20, ndl);
        float diff = max(ndl, 0.0);
        float sunrise = day * (1.0 - day) * 4.0;

        // local east / north frame for relief and wave normals
        vec3 east = normalize(vec3(cos(lon), 0.0, -sin(lon)));
        vec3 north = normalize(cross(east, n));
        float hL = lum(texture2D(World, tuv - vec2(texel.x, 0.0)).rgb);
        float hR = lum(texture2D(World, tuv + vec2(texel.x, 0.0)).rgb);
        float hD = lum(texture2D(World, tuv - vec2(0.0, texel.y)).rgb);
        float hU = lum(texture2D(World, tuv + vec2(0.0, texel.y)).rgb);
        vec3 ln = normalize(n + east * (hL - hR) * 2.4 + north * (hD - hU) * 2.4);

        // ---------------- ocean
        vec2 wuv = vec2(lon * 74.0, lat * 50.0);
        float wave = fbm(wuv + vec2(time * 0.085, time * 0.030));
        float wave2 = fbm(wuv * 2.6 - vec2(time * 0.055, time * 0.020));
        float wave3 = fbm(wuv * 3.4 + vec2(time * 0.10, -time * 0.040));
        vec3 on = normalize(n + east * (wave - 0.5) * 0.16 + north * (wave2 - 0.5) * 0.10
                              + east * (wave3 - 0.5) * 0.025);
        float shallow = smoothstep(0.22, 0.98, coast);
        vec3 deep = vec3(0.002, 0.012, 0.045);
        vec3 mid = vec3(0.008, 0.062, 0.155);
        vec3 shoal = vec3(0.030, 0.240, 0.310);
        vec3 ocean = mix(deep, mid, 0.34 + 0.42 * wave);
        ocean = mix(ocean, shoal, shallow * 0.80);
        ocean *= 0.40 + 0.80 * day;
        ocean += vec3(0.004, 0.010, 0.020) * (1.0 - day);
        ocean += vec3(0.020, 0.060, 0.110) * day;
        vec3 refl = reflect(-sun_dir, on);
        float spec = pow(max(dot(refl, q), 0.0), 150.0) * day;
        float glit = pow(max(dot(refl, q), 0.0), 420.0) * day * (0.35 + 0.65 * wave2);
        ocean += vec3(1.0, 0.80, 0.50) * (spec * 0.85 + glit * 0.9);
        float fres = pow(1.0 - max(dot(on, q), 0.0), 3.0);
        ocean = mix(ocean, vec3(0.08, 0.26, 0.44), fres * 0.42 * day);
        float foam = smoothstep(0.76, 0.99, coast) * (0.22 + 0.78 * wave);
        foam *= smoothstep(-0.10, 0.22, ndl);
        ocean = mix(ocean, vec3(0.72, 0.82, 0.85), foam * 0.62);

        // ---------------- land: use the photographic albedo, then light it
        float al = lum(alb);
        // Keep the photographic albedo; only lift its washed-out saturation.
        vec3 landc = mix(vec3(al), alb, 1.72);
        landc = pow(clamp(landc, 0.0, 1.0), vec3(1.06));
        landc *= 0.09 + 0.82 * diff;
        landc += vec3(0.010, 0.018, 0.028) * (1.0 - day);
        landc += vec3(0.22, 0.085, 0.025) * sunrise * 0.45;
        // mountains and valleys read through the relief normals
        float relief = max(dot(ln, sun_dir), 0.0);
        landc *= 0.83 + 0.34 * relief;
        // polar ice and Greenland get a cool specular sheen
        float ice = smoothstep(0.62, 0.86, lum(alb)) * smoothstep(0.50, 0.82, abs(lat)) * land;
        landc = mix(landc, vec3(0.90, 0.95, 1.0), ice * 0.28);
        landc += vec3(0.6, 0.8, 1.0) * ice * pow(max(dot(reflect(-sun_dir, ln), q), 0.0), 32.0) * 0.15;
        // beaches
        vec3 sand = vec3(0.68, 0.60, 0.43) * (0.3 + diff);
        landc = mix(landc, sand, smoothstep(0.40, 0.0, coast) * 0.26 * land);
        // vegetation grain
        landc *= 0.955 + 0.09 * fine;
        landc = mix(landc, landc * 1.15 + vec3(0.02, 0.03, 0.015), smoothstep(0.45, 0.7, detail) * 0.35);

        vec3 surf = mix(ocean, landc, land);

        // ---------------- clouds, two layers with shadows
        vec2 cuv = tuv * vec2(10.0, 5.2) + vec2(time * 0.0038, 0.0);
        float cl = fbm(cuv + fbm(cuv * 1.8 + time * 0.0013) * 1.4);
        float cl2 = fbm(cuv * 3.1 + vec2(19.0, 7.0) - time * 0.0021);
        float cloud = smoothstep(0.72, 0.88, cl * 0.90 + cl2 * 0.18);
        cloud *= 0.46;
        float cloudLight = 0.10 + 1.05 * max(ndl + 0.12, 0.0);
        vec3 ccol = mix(vec3(0.035, 0.060, 0.085), vec3(0.98, 0.99, 1.0), cloudLight);
        surf *= 1.0 - cloud * 0.30 * day;
        surf = mix(surf, ccol, cloud * 0.56);
        surf += vec3(0.30, 0.18, 0.08) * cloud * sunrise * 0.10;

        // ---------------- city lights on the night side
        float night = 1.0 - day;
        // Real lights (NASA Earth at Night). The brightness a place needs
        // before it shows falls as the sun sinks further below its horizon,
        // so the big cities come on first and the small towns last.
        float lamp = texture2D(Lights, tuv).r;
        float dusk = 1.0 - smoothstep(-0.30, 0.03, ndl);
        float need = mix(0.62, 0.035, dusk);
        float city = lamp * smoothstep(need, need + 0.10, lamp) * (1.0 - smoothstep(-0.04, 0.06, ndl));
        vec3 lampc = mix(vec3(1.0, 0.58, 0.22), vec3(1.0, 0.84, 0.60), smoothstep(0.35, 0.9, lamp));
        surf += lampc * city * (1.0 - cloud * 0.65) * 2.6;

        // aurora over the poles
        float aur = smoothstep(0.88, 1.0, fbm(vec2(lon * 9.0 + time * 0.02, lat * 14.0)))
                  * smoothstep(0.92, 1.35, abs(lat)) * night;
        surf += vec3(0.08, 0.55, 0.34) * aur * 0.55;

        // atmospheric limb on the sphere itself
        float rim = pow(1.0 - max(dot(n, q), 0.0), 4.2);
        surf += vec3(0.10, 0.30, 0.68) * rim * (0.25 + 0.75 * day) * 0.30;
        surf += vec3(0.55, 0.68, 0.92) * rim * smoothstep(-0.1, 0.5, ndl) * 0.12;

        col = surf;
    }

    // atmosphere halo outside the disc
    if (r >= 1.0 && r < 1.22) {
        float a = pow(1.0 - (r - 1.0) / 0.22, 3.2);
        vec3 rn = normalize(vec3(d.x, -d.y, 0.15));
        float lit = smoothstep(-0.30, 0.45, dot(rn, sun_dir));
        col += vec3(0.13, 0.42, 0.95) * a * (0.25 + 0.75 * lit) * 1.05;
        col += vec3(0.45, 0.22, 0.10) * a * (1.0 - lit) * 0.30;
    }

    // sun glow, projected into view space
    {
        float cy = cos(-yaw), sy = sin(-yaw);
        float cp = cos(-pitch), sp = sin(-pitch);
        vec3 s1 = vec3(sun_dir.x * cy + sun_dir.z * sy, sun_dir.y, -sun_dir.x * sy + sun_dir.z * cy);
        vec3 s2 = vec3(s1.x, s1.y * cp - s1.z * sp, s1.y * sp + s1.z * cp);
        vec2 sp2 = center + vec2(s2.x, -s2.y) * radius * 3.6;
        float dd = length(fcs - sp2);
        float g = exp(-dd / (radius * 0.32));
        col += vec3(1.0, 0.60, 0.26) * g * 0.34;
    }

    // filmic shoulder so the sunlit limb keeps its detail
    col = vec3(1.0) - exp(-max(col, vec3(0.0)) * 1.32);

    // gentle vignette keeps the eye on the planet
    vec2 vuv = fc / resolution;
    float vig = smoothstep(0.98, 0.32, length(vuv - 0.5));
    col *= 0.70 + 0.30 * vig;
    gl_FragColor = vec4(col, 1.0);
}"#;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ZoomDir { In, Out }

pub struct ZoomAnim {
    pub t: f32,
    pub dur: f32,
    pub from_zoom: f32,
    pub to_zoom: f32,
    pub from_yaw: f32,
    pub to_yaw: f32,
    pub from_pitch: f32,
    pub to_pitch: f32,
    pub dir: ZoomDir,
}

pub struct Globe {
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub time: f32,
    /// Unit vector toward the sun in the shader's frame (`sky::globe_sun_dir`).
    pub sun: Vec3,
    pub lights: Texture2D,
    pub mat: Option<Material>,
    pub albedo: Texture2D,
    pub data_tex: Texture2D,
    pub drag: bool,
    pub drag_start: Vec2,
    pub last_mouse: Vec2,
    pub moved: f32,
    pub anim: Option<ZoomAnim>,
    pub auto_rotate: f32,
}

impl Globe {
    pub fn new(data: &GameData) -> Self {
        let (albedo, data_tex) = data.globe_textures();
        let params = MaterialParams {
            uniforms: vec![
                UniformDesc::new("resolution", UniformType::Float2),
                UniformDesc::new("center", UniformType::Float2),
                UniformDesc::new("radius", UniformType::Float1),
                UniformDesc::new("yaw", UniformType::Float1),
                UniformDesc::new("pitch", UniformType::Float1),
                UniformDesc::new("time", UniformType::Float1),
                UniformDesc::new("sun_dir", UniformType::Float3),
                UniformDesc::new("texel", UniformType::Float2),
            ],
            textures: vec!["World".to_string(), "WorldData".to_string(), "Lights".to_string()],
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
                ..Default::default()
            },
            ..Default::default()
        };
        let lights = Texture2D::from_file_with_format(include_bytes!("../assets/earth_lights.png"), Some(ImageFormat::Png));
        lights.set_filter(FilterMode::Linear);
        let mat = match load_material(ShaderSource::Glsl { vertex: VERT, fragment: FRAG }, params) { Ok(m) => Some(m), Err(e) => { eprintln!("globe shader failed: {e:?}"); None } };
        Self {
            yaw: 1.35,
            pitch: 0.18,
            zoom: 1.0,
            time: 0.0,
            sun: vec3(-0.94, 0.16, 0.31).normalize(),
            lights,
            mat,
            albedo,
            data_tex,
            drag: false,
            drag_start: Vec2::ZERO,
            last_mouse: Vec2::ZERO,
            moved: 0.0,
            anim: None,
            auto_rotate: 0.0,
        }
    }

    pub fn radius(&self) -> f32 {
        screen_height().min(screen_width()) * 0.372 * self.zoom
    }
    pub fn center(&self) -> Vec2 {
        vec2(screen_width() * 0.465, screen_height() * 0.505)
    }

    pub fn project(&self, lat: f32, lon: f32) -> Option<(Vec2, f32)> {
        let cl = lat.to_radians().cos();
        let x = cl * lon.to_radians().sin();
        let y = lat.to_radians().sin();
        let z = cl * lon.to_radians().cos();
        let (sy, cy) = (self.yaw.sin(), self.yaw.cos());
        let x1 = x * cy + z * sy;
        let z1 = -x * sy + z * cy;
        let (sp, cp) = (self.pitch.sin(), self.pitch.cos());
        let y2 = y * cp - z1 * sp;
        let z2 = y * sp + z1 * cp;
        if z2 <= 0.0 { return None; }
        let r = self.radius();
        Some((self.center() + vec2(x1, -y2) * r, z2))
    }

    pub fn screen_to_latlon(&self, p: Vec2) -> Option<(f32, f32)> {
        let d = (p - self.center()) / self.radius();
        if d.length_squared() >= 1.0 { return None; }
        let q = vec3(d.x, -d.y, (1.0 - d.length_squared()).sqrt());
        let (sy, cy) = (self.yaw.sin(), self.yaw.cos());
        let (sp, cp) = (self.pitch.sin(), self.pitch.cos());
        let y = q.y * cp + q.z * sp;
        let z1 = -q.y * sp + q.z * cp;
        let x = q.x * cy - z1 * sy;
        let z = q.x * sy + z1 * cy;
        Some((y.asin().to_degrees(), x.atan2(z).to_degrees()))
    }

    pub fn handle_input(&mut self, dt: f32, blocked: bool, _allow_rotate: bool) -> Option<Vec2> {
        let (mx, my) = mouse_position();
        let mp = vec2(mx, my);
        let wheel = mouse_wheel().1;
        if !blocked {
            if wheel.abs() > 0.01 && self.anim.is_none() {
                self.zoom = (self.zoom * (-wheel * 0.13).exp()).clamp(0.85, 18.0);
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                self.drag = true;
                self.drag_start = mp;
                self.last_mouse = mp;
                self.moved = 0.0;
            }
            if self.drag {
                let delta = mp - self.last_mouse;
                self.last_mouse = mp;
                self.moved += delta.length();
                if self.moved > 4.0 && self.anim.is_none() {
                    self.yaw -= delta.x * 0.0052;
                    self.pitch = (self.pitch + delta.y * 0.0045).clamp(-1.35, 1.35);
                    self.auto_rotate = 6.0;
                }
            }
            if is_mouse_button_released(MouseButton::Left) {
                self.drag = false;
                if self.moved < 6.0 { return Some(mp); }
            }
        } else {
            self.drag = false;
        }
        if !self.drag {
            self.auto_rotate -= dt;
            if self.auto_rotate < -12.0 && self.anim.is_none() { self.yaw += dt * 0.010; }
        }
        None
    }

    /// Bring a latitude/longitude to the centre of the disc and dive in.
    pub fn start_zoom(&mut self, dir: ZoomDir, lat: f32, lon: f32, return_zoom: f32) {
        let (ty, tp) = (-lon.to_radians(), lat.to_radians());
        let (from_zoom, to_zoom) = match dir {
            ZoomDir::In => (self.zoom.max(0.8), 22.0),
            ZoomDir::Out => (self.zoom.max(1.0), return_zoom.max(0.85)),
        };
        self.zoom = from_zoom;
        self.anim = Some(ZoomAnim {
            t: 0.0,
            dur: if dir == ZoomDir::In { 1.5 } else { 1.0 },
            from_zoom, to_zoom,
            from_yaw: self.yaw, to_yaw: ty,
            from_pitch: self.pitch, to_pitch: tp,
            dir,
        });
    }

    pub fn update_anim(&mut self, dt: f32) -> Option<ZoomDir> {
        let mut done = None;
        if let Some(a) = &mut self.anim {
            a.t += dt / a.dur;
            let t = a.t.min(1.0);
            let e = t * t * (3.0 - 2.0 * t);
            self.zoom = (a.from_zoom.ln() + (a.to_zoom.ln() - a.from_zoom.ln()) * e).exp();
            self.yaw = a.from_yaw + (a.to_yaw - a.from_yaw) * e;
            self.pitch = a.from_pitch + (a.to_pitch - a.from_pitch) * e;
            if a.t >= 1.0 { done = Some(a.dir); self.anim = None; }
        }
        done
    }

    pub fn draw(&self, time: f32) {
        let w = screen_width();
        let h = screen_height();
        if let Some(m) = &self.mat {
            let sun = self.sun;
            let dpi = screen_dpi_scale();
            m.set_uniform("resolution", vec2(w * dpi, h * dpi));
            m.set_uniform("center", self.center() * dpi);
            m.set_uniform("radius", self.radius() * dpi);
            m.set_uniform("yaw", self.yaw);
            m.set_uniform("pitch", self.pitch);
            m.set_uniform("time", time);
            m.set_uniform("sun_dir", sun);
            m.set_uniform("texel", vec2(1.0 / 4096.0, 1.0 / 2048.0));
            m.set_texture("World", self.albedo.clone());
            m.set_texture("WorldData", self.data_tex.clone());
            m.set_texture("Lights", self.lights.clone());
            gl_use_material(m);
            draw_rectangle(0.0, 0.0, w, h, WHITE);
            gl_use_default_material();
        } else {
            clear_background(Color::from_rgba(5, 10, 18, 255));
            draw_circle(self.center().x, self.center().y, self.radius(), Color::from_rgba(20, 60, 95, 255));
            draw_circle_lines(self.center().x, self.center().y, self.radius(), 2.0, Color::from_rgba(70, 170, 220, 255));
        }
    }
}
