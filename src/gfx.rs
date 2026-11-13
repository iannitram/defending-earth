//! Lighting pipeline: shadow map, HDR scene pass, lens pass.
//!
//! Frame order (see docs/ART_BIBLE.md):
//!   1. `shadow_begin` .. draw casters
//!   2. `scene_begin`  .. draw the world (lit, shadowed, hazed) into HDR
//!   3. `finish`       .. meter, narrow bloom, lens (exposure, shoulder,
//!                        vignette, chromatic aberration, gain noise) to screen
//! The HUD is drawn after `finish`, untouched by the lens.

#![allow(dead_code)]

use macroquad::camera::Camera;
use macroquad::miniquad as mq;
use macroquad::prelude::*;
use mq::{FilterMode, TextureFormat, TextureId, TextureParams, TextureWrap};

// ---------------------------------------------------------------- geometry

/// Surface classes the scene shader knows how to light.
pub mod mat {
    pub const GROUND: f32 = 0.0; // soil, grass, crops: rough, broken up by world-space noise
    pub const BUILT: f32 = 1.0; // walls, roofs, concrete: rough, stained
    pub const FOLIAGE: f32 = 2.0; // leaves: light passes through
    pub const METAL: f32 = 3.0; // painted or bare metal: tighter highlight
    pub const SKIN: f32 = 4.0; // cloth, skin, kit
    pub const EMISSIVE: f32 = 5.0; // light sources: albedo is radiance
    pub const WATER: f32 = 6.0;
    pub const HULL: f32 = 7.0; // craft skin: like BUILT, but snow never settles on it
}

/// Builds lit meshes, splitting into chunks so 16-bit indices never wrap.
pub struct Geo {
    pub chunks: Vec<Mesh>,
    v: Vec<Vertex>,
    i: Vec<u16>,
}

impl Default for Geo {
    fn default() -> Self {
        Self::new()
    }
}

impl Geo {
    pub fn new() -> Self {
        Self { chunks: Vec::new(), v: Vec::new(), i: Vec::new() }
    }

    fn room(&mut self, verts: usize) {
        if self.v.len() + verts > 60_000 {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if !self.v.is_empty() {
            self.chunks.push(Mesh {
                vertices: std::mem::take(&mut self.v),
                indices: std::mem::take(&mut self.i),
                texture: None,
            });
        }
    }

    pub fn finish(mut self) -> Vec<Mesh> {
        self.flush();
        self.chunks
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.v.clear();
        self.i.clear();
    }

    fn vert(p: Vec3, n: Vec3, c: Color, ao: f32, m: f32) -> Vertex {
        Vertex {
            position: p,
            uv: vec2(0.0, 0.0),
            color: [
                (c.r.clamp(0.0, 1.0) * 255.0) as u8,
                (c.g.clamp(0.0, 1.0) * 255.0) as u8,
                (c.b.clamp(0.0, 1.0) * 255.0) as u8,
                (ao.clamp(0.0, 1.0) * 255.0) as u8,
            ],
            normal: vec4(n.x, n.y, n.z, m),
        }
    }

    /// Quad a-b-c-d (counter-clockwise seen from the normal side).
    pub fn quad(&mut self, p: [Vec3; 4], n: Vec3, c: Color, ao: [f32; 4], m: f32) {
        self.room(4);
        let b = self.v.len() as u16;
        for k in 0..4 {
            self.v.push(Self::vert(p[k], n, c, ao[k], m));
        }
        self.i.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }

    pub fn tri(&mut self, p: [Vec3; 3], n: [Vec3; 3], c: Color, ao: [f32; 3], m: f32) {
        self.room(3);
        let b = self.v.len() as u16;
        for k in 0..3 {
            self.v.push(Self::vert(p[k], n[k], c, ao[k], m));
        }
        self.i.extend_from_slice(&[b, b + 1, b + 2]);
    }

    /// Axis-aligned box from `lo` to `hi`. `ao_base` darkens the foot of the
    /// walls (contact occlusion until baked maps replace it).
    pub fn boxy(&mut self, lo: Vec3, hi: Vec3, c: Color, m: f32, ao_base: f32) {
        let (x0, y0, z0, x1, y1, z1) = (lo.x, lo.y, lo.z, hi.x, hi.y, hi.z);
        let a = ao_base;
        self.quad([vec3(x0, y1, z0), vec3(x0, y1, z1), vec3(x1, y1, z1), vec3(x1, y1, z0)], Vec3::Y, c, [1.0; 4], m);
        self.quad([vec3(x0, y0, z1), vec3(x1, y0, z1), vec3(x1, y1, z1), vec3(x0, y1, z1)], Vec3::Z, c, [a, a, 1.0, 1.0], m);
        self.quad([vec3(x1, y0, z0), vec3(x0, y0, z0), vec3(x0, y1, z0), vec3(x1, y1, z0)], -Vec3::Z, c, [a, a, 1.0, 1.0], m);
        self.quad([vec3(x1, y0, z1), vec3(x1, y0, z0), vec3(x1, y1, z0), vec3(x1, y1, z1)], Vec3::X, c, [a, a, 1.0, 1.0], m);
        self.quad([vec3(x0, y0, z0), vec3(x0, y0, z1), vec3(x0, y1, z1), vec3(x0, y1, z0)], -Vec3::X, c, [a, a, 1.0, 1.0], m);
    }

    /// Smooth-shaded blob (stretched sphere), used for tree crowns and bushes.
    pub fn blob(&mut self, centre: Vec3, r: Vec3, c: Color, m: f32, seed: u32) {
        const SEG: usize = 9;
        const RINGS: usize = 6;
        self.room((SEG + 1) * (RINGS + 1));
        let b = self.v.len() as u16;
        for j in 0..=RINGS {
            let v = j as f32 / RINGS as f32;
            let phi = v * std::f32::consts::PI;
            for k in 0..=SEG {
                let u = k as f32 / SEG as f32;
                let th = u * std::f32::consts::TAU;
                let n = vec3(phi.sin() * th.cos(), phi.cos(), phi.sin() * th.sin());
                // lumpy outline so crowns are not perfect ellipsoids
                let h = hash3(seed, (k % SEG) as u32, j as u32);
                let lump = 1.0 + (h - 0.5) * 0.34;
                let p = centre + n * r * lump;
                // underside of a crown sits in its own shade
                let ao = 0.45 + 0.55 * (n.y * 0.5 + 0.5);
                self.v.push(Self::vert(p, n, c, ao, m));
            }
        }
        let w = (SEG + 1) as u16;
        for j in 0..RINGS as u16 {
            for k in 0..SEG as u16 {
                let a = b + j * w + k;
                self.i.extend_from_slice(&[a, a + w, a + w + 1, a, a + w + 1, a + 1]);
            }
        }
    }

    /// Upright tapered cylinder (trunks, posts, limbs).
    pub fn column(&mut self, base: Vec3, top: Vec3, r0: f32, r1: f32, c: Color, m: f32) {
        const SEG: usize = 7;
        self.room((SEG + 1) * 2);
        let b = self.v.len() as u16;
        let axis = (top - base).normalize_or_zero();
        let side = if axis.y.abs() > 0.99 { Vec3::X } else { axis.cross(Vec3::Y).normalize() };
        let fwd = axis.cross(side);
        for k in 0..=SEG {
            let th = k as f32 / SEG as f32 * std::f32::consts::TAU;
            let n = side * th.cos() + fwd * th.sin();
            self.v.push(Self::vert(base + n * r0, n, c, 0.55, m));
            self.v.push(Self::vert(top + n * r1, n, c, 1.0, m));
        }
        for k in 0..SEG as u16 {
            let a = b + k * 2;
            self.i.extend_from_slice(&[a, a + 2, a + 3, a, a + 3, a + 1]);
        }
    }
}

pub fn hash3(a: u32, b: u32, c: u32) -> f32 {
    let mut h = a.wrapping_mul(0x9E37_79B1) ^ b.wrapping_mul(0x85EB_CA77) ^ c.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h & 0x00FF_FFFF) as f32 / 16_777_216.0
}

// ------------------------------------------------------------------- light

/// Everything the scene shader needs to know about the sky at this moment.
/// Radiance units are relative: a clear noon sun on a white card is about 10.
#[derive(Clone, Copy, Debug)]
pub struct SkyLook {
    /// Colour and strength of the direct light (sun, or moon at night).
    pub sun: Vec3,
    pub zenith: Vec3,
    pub horizon: Vec3,
    /// Extra horizon radiance on the side of the sky the sun is on.
    pub glow: Vec3,
    /// Haze extinction per metre.
    pub haze: f32,
    /// How diffuse the direct light is: 0 crisp shadows, 1 none (overcast).
    pub diffusion: f32,
}

impl SkyLook {
    pub fn lerp(self, o: SkyLook, t: f32) -> SkyLook {
        SkyLook {
            sun: self.sun.lerp(o.sun, t),
            zenith: self.zenith.lerp(o.zenith, t),
            horizon: self.horizon.lerp(o.horizon, t),
            glow: self.glow.lerp(o.glow, t),
            haze: self.haze + (o.haze - self.haze) * t,
            diffusion: self.diffusion + (o.diffusion - self.diffusion) * t,
        }
    }

    pub fn radiance(&self, d: Vec3, sun_dir: Vec3, ground: Vec3) -> Vec3 {
        if d.y >= 0.0 {
            let up = d.y.powf(0.45);
            let mut c = self.horizon.lerp(self.zenith, up);
            let dh = vec3(d.x, 0.0, d.z).normalize_or_zero();
            let sh = vec3(sun_dir.x, 0.0, sun_dir.z).normalize_or_zero();
            let toward = dh.dot(sh).max(0.0);
            c += self.glow * toward.powi(3) * (1.0 - d.y).powi(4);
            c
        } else {
            // the ground, lit by sun and sky, seen from above it
            let e_sun = self.sun * sun_dir.y.max(0.0) * (1.0 - self.diffusion * 0.6) * 0.75;
            let e_sky = (self.zenith + self.horizon) * 0.5 * std::f32::consts::PI;
            ground * (e_sun + e_sky) / std::f32::consts::PI
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Light {
    /// Unit vector toward the direct light.
    pub sun_dir: Vec3,
    pub look: SkyLook,
    /// Average ground colour (linear), for bounce light.
    pub ground: Vec3,
    /// Irradiance/π as 9 spherical-harmonic coefficients.
    pub sh: [Vec3; 9],
    /// 0 dry .. 1 soaked.
    pub wet: f32,
    /// Share of the ground under snow, 0..1.
    pub snow: f32,
    /// 1 where open ground is sand instead of grass.
    pub sand: f32,
    /// 1 while rain is falling (rings in the puddles).
    pub rain: f32,
    /// Seconds, for anything that moves on the ground.
    pub time: f32,
}

impl Light {
    pub fn new(sun_dir: Vec3, look: SkyLook, ground: Vec3) -> Light {
        let mut l = Light { sun_dir: sun_dir.normalize(), look, ground, sh: [Vec3::ZERO; 9], wet: 0.0, snow: 0.0, sand: 0.0, rain: 0.0, time: 0.0 };
        l.sh = l.project_sh();
        l
    }

    /// Project the sky (and lit ground below the horizon) onto SH9, already
    /// convolved with the cosine lobe and divided by π, so the shader gets
    /// "radiance leaving a white diffuse surface" straight from the normal.
    fn project_sh(&self) -> [Vec3; 9] {
        let mut c = [Vec3::ZERO; 9];
        const NT: usize = 24;
        const NP: usize = 48;
        let mut wsum = 0.0;
        for i in 0..NT {
            let ct = 1.0 - 2.0 * (i as f32 + 0.5) / NT as f32;
            let st = (1.0 - ct * ct).sqrt();
            for j in 0..NP {
                let ph = (j as f32 + 0.5) / NP as f32 * std::f32::consts::TAU;
                let d = vec3(st * ph.cos(), ct, st * ph.sin());
                let l = self.look.radiance(d, self.sun_dir, self.ground);
                let b = sh_basis(d);
                for k in 0..9 {
                    c[k] += l * b[k];
                }
                wsum += 1.0;
            }
        }
        let w = 4.0 * std::f32::consts::PI / wsum;
        // cosine-lobe factors / π
        const A: [f32; 9] = [1.0, 2.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0, 0.25, 0.25, 0.25, 0.25, 0.25];
        for k in 0..9 {
            c[k] *= w * A[k];
        }
        c
    }
}

fn sh_basis(d: Vec3) -> [f32; 9] {
    [
        0.282095,
        0.488603 * d.y,
        0.488603 * d.z,
        0.488603 * d.x,
        1.092548 * d.x * d.y,
        1.092548 * d.y * d.z,
        0.315392 * (3.0 * d.z * d.z - 1.0),
        1.092548 * d.x * d.z,
        0.546274 * (d.x * d.x - d.y * d.y),
    ]
}

/// Lens and operator settings for `finish`.
#[derive(Clone, Copy, Debug)]
pub struct Lens {
    /// Vignette strength in stops at the corner (0.4 at 70 mm, 0.2 at 200 mm).
    pub vignette_stops: f32,
    /// Lateral chromatic aberration at the corner, in pixels.
    pub ca_px: f32,
    /// Operator's exposure compensation in stops.
    pub ev_bias: f32,
    /// Seconds for the auto-exposure to settle; 0 = instant (for stills).
    pub adapt_secs: f32,
    /// Extra full-frame flash (lightning), in scene radiance.
    pub flash: f32,
}

impl Default for Lens {
    fn default() -> Self {
        Self { vignette_stops: 0.3, ca_px: 1.0, ev_bias: 0.0, adapt_secs: 1.5, flash: 0.0 }
    }
}

// ---------------------------------------------------------------- pipeline

struct MatCam(Mat4);
impl Camera for MatCam {
    fn matrix(&self) -> Mat4 {
        self.0
    }
    fn depth_enabled(&self) -> bool {
        true
    }
    fn render_pass(&self) -> Option<macroquad::texture::RenderPass> {
        None
    }
    fn viewport(&self) -> Option<(i32, i32, i32, i32)> {
        None
    }
}

#[derive(Clone)]
struct Target {
    pass: mq::RenderPass,
    tex: Texture2D,
    w: u32,
    h: u32,
}

/// A repeating surface-grain texture with mipmaps (so it does not sparkle
/// from the air). Packed by source_assets/polyhaven/make_ground.py.
fn grain_tex(png: &[u8]) -> Texture2D {
    let t = Texture2D::from_file_with_format(png, None);
    let gl = unsafe { get_internal_gl() };
    let id = t.raw_miniquad_id();
    gl.quad_context.texture_set_wrap(id, TextureWrap::Repeat, TextureWrap::Repeat);
    gl.quad_context.texture_generate_mipmaps(id);
    gl.quad_context.texture_set_min_filter(id, FilterMode::Linear, mq::MipmapFilterMode::Linear);
    t
}

fn make_target(w: u32, h: u32, format: TextureFormat, depth: Option<TextureId>, filter: FilterMode) -> Target {
    let gl = unsafe { get_internal_gl() };
    let id = gl.quad_context.new_render_texture(TextureParams {
        width: w,
        height: h,
        format,
        wrap: TextureWrap::Clamp,
        min_filter: filter,
        mag_filter: filter,
        ..Default::default()
    });
    let pass = gl.quad_context.new_render_pass_mrt(&[id], None, depth);
    Target { pass, tex: Texture2D::from_miniquad_texture(id), w, h }
}

fn bind(pass: Option<mq::RenderPass>) {
    let mut gl = unsafe { get_internal_gl() };
    gl.flush();
    gl.quad_gl.render_pass(pass);
}

pub struct Gfx {
    pub w: u32,
    pub h: u32,
    hdr: Target,
    shadow: Target,
    shadow_depth: Texture2D,
    bloom_a: Target,
    bloom_b: Target,
    meter: [Target; 2],
    meter_i: usize,
    first: bool,
    frame: u32,
    scene_mat: Material,
    overlay_mat: Material,
    shadow_mat: Material,
    meter_mat: Material,
    bright_mat: Material,
    blur_mat: Material,
    lens_mat: Material,
    light_vp: Mat4,
    shadow_texel_m: f32,
    cam_pos: Vec3,
    cam_fwd: Vec3,
    ground_map: Texture2D,
    ground_rect: Vec4,
    ground_tex: [Texture2D; 5],
}

pub const SHADOW_RES: u32 = 4096;

impl Gfx {
    /// `w`×`h` is the internal scene resolution (the HUD stays at native).
    pub fn new(w: u32, h: u32) -> Gfx {
        let gl = unsafe { get_internal_gl() };
        let scene_depth = gl.quad_context.new_render_texture(TextureParams {
            width: w,
            height: h,
            format: TextureFormat::Depth,
            ..Default::default()
        });
        let hdr = make_target(w, h, TextureFormat::RGBA16F, Some(scene_depth), FilterMode::Linear);

        let sd = gl.quad_context.new_render_texture(TextureParams {
            width: SHADOW_RES,
            height: SHADOW_RES,
            format: TextureFormat::Depth32,
            wrap: TextureWrap::Clamp,
            min_filter: FilterMode::Nearest,
            mag_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let shadow = make_target(SHADOW_RES, SHADOW_RES, TextureFormat::RGBA8, Some(sd), FilterMode::Nearest);

        let bloom_a = make_target(w / 2, h / 2, TextureFormat::RGBA16F, None, FilterMode::Linear);
        let bloom_b = make_target(w / 2, h / 2, TextureFormat::RGBA16F, None, FilterMode::Linear);
        let meter = [
            make_target(1, 1, TextureFormat::RGBA16F, None, FilterMode::Nearest),
            make_target(1, 1, TextureFormat::RGBA16F, None, FilterMode::Nearest),
        ];

        let scene_mat = load_material(
            ShaderSource::Glsl { vertex: SCENE_VS, fragment: SCENE_FS },
            MaterialParams {
                pipeline_params: PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    cull_face: mq::CullFace::Nothing,
                    ..Default::default()
                },
                uniforms: vec![
                    UniformDesc::new("light_vp", UniformType::Mat4),
                    UniformDesc::new("sun_dir", UniformType::Float3),
                    UniformDesc::new("sun_col", UniformType::Float3),
                    UniformDesc::new("sh", UniformType::Float3).array(9),
                    UniformDesc::new("cam_pos", UniformType::Float3),
                    UniformDesc::new("haze_sun", UniformType::Float3),
                    UniformDesc::new("haze_away", UniformType::Float3),
                    UniformDesc::new("zenith", UniformType::Float3),
                    UniformDesc::new("horizon", UniformType::Float3),
                    UniformDesc::new("params", UniformType::Float4),
                    UniformDesc::new("shadow_px", UniformType::Float2),
                    UniformDesc::new("ground_rect", UniformType::Float4),
                    UniformDesc::new("surf", UniformType::Float4),
                ],
                textures: ["ShadowMap", "GroundMap", "GrassA", "GrassB", "Soil", "Mud", "Gravel"].iter().map(|s| s.to_string()).collect(),
            },
        )
        .expect("scene shader");

        let overlay_mat = load_material(
            ShaderSource::Glsl { vertex: OVERLAY_VS, fragment: OVERLAY_FS },
            MaterialParams {
                pipeline_params: PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: false,
                    cull_face: mq::CullFace::Nothing,
                    color_blend: Some(mq::BlendState::new(
                        mq::Equation::Add,
                        mq::BlendFactor::Value(mq::BlendValue::SourceAlpha),
                        mq::BlendFactor::OneMinusValue(mq::BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
                uniforms: vec![
                    UniformDesc::new("cam_pos", UniformType::Float3),
                    UniformDesc::new("haze_col", UniformType::Float3),
                    UniformDesc::new("level", UniformType::Float4),
                ],
                textures: vec![],
            },
        )
        .expect("overlay shader");

        let shadow_mat = load_material(
            ShaderSource::Glsl { vertex: SHADOW_VS, fragment: SHADOW_FS },
            MaterialParams {
                pipeline_params: PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    cull_face: mq::CullFace::Nothing,
                    depth_write_offset: Some((1.5, 3.0)),
                    ..Default::default()
                },
                uniforms: vec![],
                textures: vec![],
            },
        )
        .expect("shadow shader");

        let post = |fs: &str, uniforms: Vec<UniformDesc>, textures: Vec<&str>| {
            load_material(
                ShaderSource::Glsl { vertex: POST_VS, fragment: fs },
                MaterialParams {
                    pipeline_params: PipelineParams::default(),
                    uniforms,
                    textures: textures.into_iter().map(|s| s.to_string()).collect(),
                },
            )
            .expect("post shader")
        };
        let meter_mat = post(METER_FS, vec![UniformDesc::new("k", UniformType::Float1)], vec!["Scene", "Prev"]);
        let bright_mat = post(
            BRIGHT_FS,
            vec![UniformDesc::new("inv", UniformType::Float2), UniformDesc::new("ev_bias", UniformType::Float1)],
            vec!["Scene", "Meter"],
        );
        let blur_mat = post(
            BLUR_FS,
            vec![UniformDesc::new("inv", UniformType::Float2), UniformDesc::new("dir", UniformType::Float2)],
            vec!["Src"],
        );
        let lens_mat = post(
            LENS_FS,
            vec![
                UniformDesc::new("inv", UniformType::Float2),
                UniformDesc::new("lens", UniformType::Float4),
                UniformDesc::new("frame", UniformType::Float1),
            ],
            vec!["Scene", "Bloom", "Meter"],
        );

        Gfx {
            w,
            h,
            hdr,
            shadow,
            shadow_depth: Texture2D::from_miniquad_texture(sd),
            bloom_a,
            bloom_b,
            meter,
            meter_i: 0,
            first: true,
            frame: 0,
            scene_mat,
            overlay_mat,
            shadow_mat,
            meter_mat,
            bright_mat,
            blur_mat,
            lens_mat,
            light_vp: Mat4::IDENTITY,
            shadow_texel_m: 0.02,
            cam_pos: Vec3::ZERO,
            cam_fwd: Vec3::NEG_Z,
            ground_map: Texture2D::from_rgba8(1, 1, &[0, 0, 0, 0]),
            ground_rect: Vec4::ZERO,
            ground_tex: [
                grain_tex(include_bytes!("../assets/ground/leafy_grass.png")),
                grain_tex(include_bytes!("../assets/ground/sparse_grass.png")),
                grain_tex(include_bytes!("../assets/ground/farm_soil.png")),
                grain_tex(include_bytes!("../assets/ground/dry_mud_field_001.png")),
                grain_tex(include_bytes!("../assets/ground/gravel_road.png")),
            ],
        }
    }

    /// Give the ground its colours and surface types as a small map laid
    /// over the site: RGB = albedo, A = type (0 grass, 0.33 ploughed soil,
    /// 0.66 crop rows, 1 packed earth or paving). The shader samples it with
    /// warped coordinates so borders wander instead of following the grid.
    /// `origin` and `size` are the map's world-space x/z rectangle.
    pub fn set_ground_map(&mut self, w: u16, h: u16, rgba: &[u8], origin: Vec2, size: Vec2) {
        let t = Texture2D::from_rgba8(w, h, rgba);
        t.set_filter(FilterMode::Linear);
        self.ground_map = t;
        self.ground_rect = vec4(origin.x, origin.y, 1.0 / size.x, 1.0 / size.y);
    }

    /// Start the shadow pass. `centre` and `radius` bound everything that can
    /// cast or receive a shadow.
    pub fn shadow_begin(&mut self, light: &Light, centre: Vec3, radius: f32) {
        let l = light.sun_dir;
        let up = if l.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
        let dist = radius * 2.0;
        let view = Mat4::look_at_rh(centre + l * dist, centre, up);
        let proj = Mat4::orthographic_rh_gl(-radius, radius, -radius, radius, dist - radius * 1.6, dist + radius * 1.6);
        self.light_vp = proj * view;
        self.shadow_texel_m = 2.0 * radius / SHADOW_RES as f32;
        set_camera(&MatCam(self.light_vp));
        bind(Some(self.shadow.pass));
        clear_background(WHITE);
        gl_use_material(&self.shadow_mat);
    }

    /// Start the lit scene pass.
    pub fn scene_begin(&mut self, cam: &Camera3D, light: &Light) {
        let aspect = self.w as f32 / self.h as f32;
        let m = Mat4::perspective_rh_gl(cam.fovy, aspect, cam.z_near, cam.z_far) * Mat4::look_at_rh(cam.position, cam.target, cam.up);
        self.cam_pos = cam.position;
        self.cam_fwd = (cam.target - cam.position).normalize();
        set_camera(&MatCam(m));
        bind(Some(self.hdr.pass));

        // The backdrop is what the lens sees past the edge of the site: haze.
        let lk = &light.look;
        let (haze_sun, haze_away) = haze_colours(light);
        let toward = self.cam_fwd.dot(light.sun_dir).max(0.0).powi(4);
        let bg = haze_away.lerp(haze_sun, toward);
        clear_background(Color::new(bg.x, bg.y, bg.z, 1.0));

        let mt = &self.scene_mat;
        mt.set_uniform("light_vp", self.light_vp);
        mt.set_uniform("sun_dir", light.sun_dir);
        mt.set_uniform("sun_col", lk.sun);
        mt.set_uniform_array("sh", &light.sh[..]);
        mt.set_uniform("cam_pos", cam.position);
        mt.set_uniform("haze_sun", haze_sun);
        mt.set_uniform("haze_away", haze_away);
        mt.set_uniform("zenith", lk.zenith);
        mt.set_uniform("horizon", lk.horizon);
        mt.set_uniform("params", vec4(lk.haze, lk.diffusion, light.wet, self.frame as f32));
        mt.set_uniform("shadow_px", vec2(1.0 / SHADOW_RES as f32, self.shadow_texel_m));
        mt.set_uniform("ground_rect", self.ground_rect);
        mt.set_uniform("surf", vec4(light.snow, light.sand, light.time, light.rain));
        mt.set_texture("ShadowMap", self.shadow_depth.clone());
        mt.set_texture("GroundMap", self.ground_map.clone());
        for (name, t) in ["GrassA", "GrassB", "Soil", "Mud", "Gravel"].iter().zip(self.ground_tex.iter()) {
            mt.set_texture(name, t.clone());
        }
        gl_use_material(mt);
    }

    /// After the lit scene: see-through things (smoke, markers, tracers,
    /// anything drawn with macroquad's own 3D calls). Vertex alpha is opacity.
    /// `glow` 0 = lit like a matte surface under this light (smoke, markers);
    /// above 0 = gives off its own light at that radiance, whatever the hour
    /// (noon sunlit white is about 11; fire 6, a tracer 20).
    pub fn overlay_begin(&mut self, light: &Light, glow: f32) {
        let lk = &light.look;
        let amb = light.sh[0] * 0.282095 + light.sh[1] * 0.488603;
        let lit = lk.sun * light.sun_dir.y.max(0.0) * (1.0 - lk.diffusion * 0.6) + amb;
        let (hs, ha) = haze_colours(light);
        let mt = &self.overlay_mat;
        mt.set_uniform("cam_pos", self.cam_pos);
        mt.set_uniform("haze_col", (hs + ha) * 0.5);
        let l = if glow > 0.0 { Vec3::splat(glow) } else { lit };
        mt.set_uniform("level", vec4(l.x, l.y, l.z, lk.haze));
        gl_use_material(mt);
    }

    fn fullscreen(&self, mt: &Material, to: Option<&Target>) {
        set_default_camera();
        bind(to.map(|t| t.pass));
        gl_use_material(mt);
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), WHITE);
        let mut gl = unsafe { get_internal_gl() };
        gl.flush();
        gl_use_default_material();
    }

    /// Meter, bloom and lens; leaves the default camera and material active
    /// with the finished picture on screen.
    pub fn finish(&mut self, dt: f32, lens: &Lens) {
        gl_use_default_material();
        let inv = vec2(1.0 / self.w as f32, 1.0 / self.h as f32);

        // 1. metering with operator lag
        let prev = self.meter[self.meter_i].clone();
        let next = self.meter[1 - self.meter_i].clone();
        let k = if self.first || lens.adapt_secs <= 0.0 { 1.0 } else { 1.0 - (-dt / lens.adapt_secs).exp() };
        self.meter_mat.set_uniform("k", k);
        self.meter_mat.set_texture("Scene", self.hdr.tex.clone());
        self.meter_mat.set_texture("Prev", prev.tex.clone());
        self.fullscreen(&self.meter_mat, Some(&next));
        self.meter_i = 1 - self.meter_i;
        self.first = false;

        // 2. narrow bloom from values that clip after exposure
        let half = vec2(2.0 * inv.x, 2.0 * inv.y);
        self.bright_mat.set_uniform("inv", half);
        self.bright_mat.set_uniform("ev_bias", lens.ev_bias);
        self.bright_mat.set_texture("Scene", self.hdr.tex.clone());
        self.bright_mat.set_texture("Meter", next.tex.clone());
        self.fullscreen(&self.bright_mat, Some(&self.bloom_a));
        self.blur_mat.set_uniform("inv", half);
        self.blur_mat.set_uniform("dir", vec2(1.0, 0.0));
        self.blur_mat.set_texture("Src", self.bloom_a.tex.clone());
        self.fullscreen(&self.blur_mat, Some(&self.bloom_b));
        self.blur_mat.set_uniform("dir", vec2(0.0, 1.0));
        self.blur_mat.set_texture("Src", self.bloom_b.tex.clone());
        self.fullscreen(&self.blur_mat, Some(&self.bloom_a));

        // 3. lens, to the screen
        let screen_inv = vec2(1.0 / screen_width(), 1.0 / screen_height());
        self.lens_mat.set_uniform("inv", screen_inv * (1.0 / screen_dpi_scale()));
        self.lens_mat.set_uniform("lens", vec4(lens.vignette_stops, lens.ca_px, lens.ev_bias, lens.flash));
        self.lens_mat.set_uniform("frame", self.frame as f32);
        self.lens_mat.set_texture("Scene", self.hdr.tex.clone());
        self.lens_mat.set_texture("Bloom", self.bloom_a.tex.clone());
        self.lens_mat.set_texture("Meter", next.tex.clone());
        self.fullscreen(&self.lens_mat, None);
        self.frame = self.frame.wrapping_add(1);
    }
}

/// Haze brightness looking toward and away from the sun.
fn haze_colours(light: &Light) -> (Vec3, Vec3) {
    let lk = &light.look;
    let lit = lk.sun * light.sun_dir.y.max(0.0).powf(0.6) * 0.035;
    let away = lk.horizon * 0.50 + lit * 0.4;
    let sun = lk.horizon * 0.60 + lk.glow * 0.35 + lit * 1.8;
    (sun, away)
}

// ----------------------------------------------------------------- shaders

const SCENE_VS: &str = r#"#version 330
in vec3 position;
in vec2 texcoord;
in vec4 color0;
in vec4 normal;
uniform mat4 Model;
uniform mat4 Projection;
uniform mat4 light_vp;
uniform vec3 sun_dir;
uniform vec2 shadow_px;
out vec3 v_pos;
out vec3 v_n;
out vec4 v_col;
out float v_mat;
out vec4 v_lp;
void main() {
    vec4 w = Model * vec4(position, 1.0);
    v_pos = w.xyz;
    v_n = normal.xyz;
    v_mat = normal.w;
    v_col = color0 / 255.0;
    // look the shadow up a little off the surface, more at grazing angles
    float graze = 1.0 - clamp(dot(normalize(normal.xyz), sun_dir), 0.0, 1.0);
    v_lp = light_vp * vec4(w.xyz + normalize(normal.xyz) * shadow_px.y * (1.5 + 5.0 * graze), 1.0);
    gl_Position = Projection * w;
}
"#;

const SCENE_FS: &str = r#"#version 330
in vec3 v_pos;
in vec3 v_n;
in vec4 v_col;
in float v_mat;
in vec4 v_lp;
uniform sampler2D ShadowMap;
uniform sampler2D GroundMap;
uniform sampler2D GrassA;
uniform sampler2D GrassB;
uniform sampler2D Soil;
uniform sampler2D Mud;
uniform sampler2D Gravel;
uniform vec4 ground_rect;
uniform vec4 surf;        // snow cover, sand, seconds, raining
uniform vec3 sun_dir;
uniform vec3 sun_col;
uniform vec3 sh[9];
uniform vec3 cam_pos;
uniform vec3 haze_sun;
uniform vec3 haze_away;
uniform vec3 zenith;
uniform vec3 horizon;
uniform vec4 params;      // haze per metre, diffusion, wetness, frame
uniform vec2 shadow_px;   // 1/resolution, metres per texel
out vec4 frag;

float hash(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}
float vnoise(vec2 p) {
    vec2 i = floor(p), f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1, 0)), u.x),
               mix(hash(i + vec2(0, 1)), hash(i + vec2(1, 1)), u.x), u.y);
}
float fbm(vec2 p) {
    float a = 0.5, s = 0.0;
    for (int i = 0; i < 4; i++) { s += a * vnoise(p); p = p * 2.03 + 17.7; a *= 0.5; }
    return s;
}
vec3 to_lin(vec3 c) { return pow(c, vec3(2.2)); }

vec3 irradiance(vec3 n) {
    return max(vec3(0.0),
        sh[0] * 0.282095 +
        sh[1] * 0.488603 * n.y + sh[2] * 0.488603 * n.z + sh[3] * 0.488603 * n.x +
        sh[4] * 1.092548 * n.x * n.y + sh[5] * 1.092548 * n.y * n.z +
        sh[6] * 0.315392 * (3.0 * n.z * n.z - 1.0) +
        sh[7] * 1.092548 * n.x * n.z + sh[8] * 0.546274 * (n.x * n.x - n.y * n.y));
}

float shadow(vec3 n, float ndl) {
    vec3 p = v_lp.xyz / v_lp.w * 0.5 + 0.5;
    if (p.x < 0.0 || p.x > 1.0 || p.y < 0.0 || p.y > 1.0 || p.z > 1.0) return 1.0;
    float bias = mix(0.0009, 0.00025, clamp(ndl, 0.0, 1.0));
    float ang = hash(gl_FragCoord.xy) * 6.2831853;
    mat2 rot = mat2(cos(ang), -sin(ang), sin(ang), cos(ang));
    // penumbra: sun's half-degree disc, widened by sky diffusion
    float rad = (1.6 + params.y * 40.0) * shadow_px.x;
    float s = 0.0;
    const vec2 taps[12] = vec2[12](
        vec2(-0.326, -0.406), vec2(-0.840, -0.074), vec2(-0.696, 0.457), vec2(-0.203, 0.621),
        vec2(0.962, -0.195), vec2(0.473, -0.480), vec2(0.519, 0.767), vec2(0.185, -0.893),
        vec2(0.507, 0.064), vec2(0.896, 0.412), vec2(-0.322, -0.933), vec2(-0.792, -0.598));
    for (int i = 0; i < 12; i++) {
        float d = texture(ShadowMap, p.xy + rot * taps[i] * rad).r;
        s += (p.z - bias > d) ? 0.0 : 1.0;
    }
    vec2 e = min(p.xy, 1.0 - p.xy);
    return mix(1.0, s / 12.0, smoothstep(0.0, 0.04, min(e.x, e.y)));
}

// Photographed surface grain without visible repeats: the same texture is
// laid down twice, at different sizes and angles, and a slow noise decides
// which one shows where. Returns (brightness around 1, normal x, normal z, roughness).
vec4 grain(sampler2D t, vec2 p, float per_m) {
    vec4 a = texture(t, p * per_m);
    vec4 b = texture(t, mat2(0.8, 0.6, -0.6, 0.8) * p * per_m * 0.43 + 0.37);
    vec2 na = a.gb * 2.0 - 1.0;
    vec2 nb = mat2(0.8, -0.6, 0.6, 0.8) * (b.gb * 2.0 - 1.0);
    float k = smoothstep(0.38, 0.62, fbm(p * 0.19 + 7.0));
    vec2 nn = mix(na, nb, k);
    return vec4(mix(a.r, b.r, k) * 2.0, nn.x, -nn.y, mix(a.a, b.a, k));
}

// Where snow lies: patchy at low cover, drifts joining up as it rises.
float snow_mask(vec3 p, float amt) {
    if (amt <= 0.0) return 0.0;
    float q = fbm(p.xz * 0.16 + 19.0) * 0.6 + fbm(p.xz * 1.3 + 3.0) * 0.3 + vnoise(p.xz * 6.0) * 0.1;
    float th = 1.0 - amt;
    return smoothstep(th - 0.08, th + 0.10, q + (amt - 0.5) * 0.3);
}

void main() {
    vec3 n = normalize(v_n);
    vec3 view = cam_pos - v_pos;
    float dist = length(view);
    vec3 v = view / dist;
    if (dot(n, v) < 0.0 && v_mat > 0.5 && v_mat < 2.5) n = -n;

    vec3 alb = to_lin(v_col.rgb);
    float ao = v_col.a;
    int m = int(v_mat + 0.5);
    bool hull = (m == 7);
    if (hull) m = 1;
    float sand = surf.y;
    float rough = 0.9;
    float f0 = 0.03;
    float trans = 0.0;
    float wet = params.z;

    if (m == 0) {
        // ground: colours and types come from the site map, sampled with
        // warped coordinates so borders wander; detail is world-space, no tiling
        float big = fbm(v_pos.xz * 0.045);
        float mid = fbm(v_pos.xz * 0.37 + 11.0);
        float fine = vnoise(v_pos.xz * 5.3) * 0.6 + vnoise(v_pos.xz * 17.0) * 0.4;
        float gtype = 0.0;
        float flat_ = step(0.7, v_n.y);
        if (ground_rect.z > 0.0 && flat_ > 0.5) {
            vec2 warp = vec2(fbm(v_pos.xz * 0.8 + 3.0), fbm(v_pos.xz * 0.8 + 47.0)) - 0.5;
            warp += (vec2(vnoise(v_pos.xz * 4.0), vnoise(v_pos.xz * 4.0 + 9.0)) - 0.5) * 0.25;
            vec2 gp = (v_pos.xz + warp * 1.5 - ground_rect.xy) * ground_rect.zw;
            vec2 e = min(gp, 1.0 - gp);
            float inside = smoothstep(0.0, 0.01, min(e.x, e.y));
            vec4 gm = texture(GroundMap, gp);
            alb = mix(alb, to_lin(gm.rgb), inside);
            gtype = gm.a * inside;
        }
        float w_soil = smoothstep(0.17, 0.25, gtype) * (1.0 - smoothstep(0.42, 0.5, gtype));
        float w_crop = smoothstep(0.5, 0.58, gtype) * (1.0 - smoothstep(0.76, 0.84, gtype));
        float w_hard = smoothstep(0.84, 0.92, gtype);
        float w_grass = clamp(1.0 - w_soil - w_crop - w_hard, 0.0, 1.0);

        // grass: clover and dry patches, tufts
        vec3 g_alb = alb * (0.74 + 0.30 * big + 0.22 * (mid - 0.5) + 0.10 * (fine - 0.5));
        g_alb = mix(g_alb, g_alb * vec3(1.12, 1.0, 0.80), smoothstep(0.45, 0.75, mid) * 0.5);
        // worn, dry ground shows through in broad patches
        float thin = 1.0 - smoothstep(0.35, 0.65, big * 0.6 + mid * 0.4);
        g_alb = mix(g_alb, g_alb * vec3(1.30, 1.10, 0.72), thin * 0.45);
        // sand: slow dunes, wind ripples a hand apart, stony lag in the hollows
        float dune = fbm(v_pos.xz * 0.11 + 5.0);
        float rp = (v_pos.x * 0.83 + v_pos.z * 0.56) + (mid - 0.5) * 1.6 + dune * 3.0;
        if (sand > 0.0) {
            float rippled = smoothstep(0.30, 0.55, big);
            vec3 sa = alb * (0.86 + 0.20 * dune + 0.10 * (mid - 0.5) + 0.04 * sin(rp * 21.0) * rippled);
            sa *= 1.0 - 0.22 * smoothstep(0.60, 0.70, vnoise(v_pos.xz * 9.0)) * (1.0 - smoothstep(0.30, 0.50, dune));
            g_alb = mix(g_alb, sa, sand);
        }
        // ploughed soil: furrows 0.7 m apart, wandering slightly, clods
        float fw = (v_pos.x * 0.94 + v_pos.z * 0.34) + (mid - 0.5) * 0.5;
        float furrow = sin(fw * 8.976);
        vec3 s_alb = alb * (0.80 + 0.14 * furrow + 0.22 * (fine - 0.5) + 0.20 * (big - 0.5));
        // crop: rows 0.5 m apart with gaps where plants failed
        float cw = (v_pos.z * 0.97 - v_pos.x * 0.24);
        float row = smoothstep(-0.2, 0.5, sin(cw * 12.566));
        float gap = smoothstep(0.22, 0.50, vnoise(v_pos.xz * 0.33 + 5.0));
        vec3 c_alb = mix(alb * vec3(0.76, 0.66, 0.52), alb, row * gap) * (0.86 + 0.2 * (fine - 0.5) + 0.16 * (big - 0.5));
        // packed earth: wheel ruts and stones
        vec3 h_alb = alb * (0.86 + 0.16 * (mid - 0.5) + 0.22 * (fine - 0.5));
        h_alb *= 1.0 - 0.10 * smoothstep(0.55, 0.6, vnoise(v_pos.xz * 11.0));
        alb = g_alb * w_grass + s_alb * w_soil + c_alb * w_crop + h_alb * w_hard;

        // photographed grain on top: brightness, small relief, roughness
        vec4 gr = vec4(0.0);
        if (w_grass > 0.003) {
            float lush = smoothstep(0.35, 0.65, big * 0.6 + mid * 0.4);
            if (sand > 0.5) gr += grain(Soil, v_pos.xz, 0.70) * w_grass;
            else gr += mix(grain(GrassB, v_pos.xz, 0.55), grain(GrassA, v_pos.xz, 0.45), lush) * w_grass;
        }
        if (w_soil > 0.003) gr += grain(Soil, v_pos.xz, 0.40) * w_soil;
        if (w_crop > 0.003) gr += grain(Mud, v_pos.xz, 0.45) * w_crop;
        if (w_hard > 0.003) gr += grain(Gravel, v_pos.xz, 0.35) * w_hard;
        gr /= max(w_grass + w_soil + w_crop + w_hard, 0.003);
        float gk = clamp(gr.r, 0.25, 2.2);
        float soft = sand * w_grass;
        alb *= mix(1.0, gk, mix(0.85, 0.30, soft));
        // sunlit blade tips and dry stalks run yellower than the shaded base
        alb *= mix(vec3(1.0), vec3(1.10, 1.0, 0.78), smoothstep(1.0, 1.9, gk) * w_grass * 0.6 * (1.0 - sand));

        // micro relief so the sun rakes across it
        float e = 0.06;
        float hx = fbm((v_pos.xz + vec2(e, 0)) * 2.9) - fbm((v_pos.xz - vec2(e, 0)) * 2.9);
        float hz = fbm((v_pos.xz + vec2(0, e)) * 2.9) - fbm((v_pos.xz - vec2(0, e)) * 2.9);
        vec3 bump = vec3(-hx, 0.0, -hz) * mix(1.3, 0.5, w_hard);
        bump += vec3(0.94, 0.0, 0.34) * cos(fw * 8.976) * -0.55 * w_soil;
        bump += vec3(-0.24, 0.0, 0.97) * cos(cw * 12.566) * -0.35 * w_crop * gap;
        bump += vec3(gr.g, 0.0, gr.b) * mix(0.9, 0.55, w_grass) * (1.0 - 0.6 * soft);
        bump += vec3(0.83, 0.0, 0.56) * cos(rp * 21.0) * -0.30 * soft * smoothstep(0.30, 0.55, big);
        // a low sun rakes every bump into a glint and the field reads as
        // water; real grass shades itself, so flatten the relief as the sun drops
        bump *= mix(1.0, mix(0.28, 1.0, smoothstep(0.08, 0.55, sun_dir.y)), w_grass * (1.0 - sand));
        if (n.y > 0.7) n = normalize(n + bump);
        rough = mix(0.92, clamp(gr.a, 0.3, 1.0), 0.7);
        rough = mix(rough, max(rough, 0.82), w_grass);
        // standing water collects in the low frequencies when wet
        float low = fbm(v_pos.xz * 0.55 + 31.0) * 0.55 + fbm(v_pos.xz * 1.9 + 9.0) * 0.25 + mid * 0.2;
        float puddle = smoothstep(0.585, 0.66, low + (wet - 1.0) * 0.12) * smoothstep(0.25, 0.6, wet) * step(0.9, v_n.y);
        alb *= mix(1.0, 0.72, wet);
        rough = mix(rough, 0.06, puddle);
        rough = mix(rough, 0.55, wet * (1.0 - puddle));
        f0 = mix(0.03, 0.08, puddle);
        // shallow muddy water: the bed shows through, a little paler
        alb = mix(alb, alb * 0.55 + vec3(0.012, 0.011, 0.008), puddle);
        if (puddle > 0.5) {
            n = normalize(mix(n, vec3(0, 1, 0), 0.95));
            if (surf.w > 0.0) {
                // rain rings: one drop per hand-sized cell, each on its own beat
                vec2 cp = v_pos.xz * 5.0;
                vec2 cell = floor(cp);
                vec2 f = fract(cp) - 0.5;
                float hsh = hash(cell);
                float ph = fract(surf.z * 1.6 + hsh * 7.0);
                float r = length(f);
                float ring = smoothstep(0.10, 0.0, abs(r - ph * 0.45)) * (1.0 - ph);
                n = normalize(n + vec3(f.x, 0.0, f.y) / max(r, 0.02) * ring * 0.55 * surf.w);
            }
        }
        // snow lying on the ground: drifts and wind crust
        float sn = snow_mask(v_pos, surf.x) * step(0.7, v_n.y);
        if (sn > 0.0) {
            float drift = fbm(v_pos.xz * 0.5 + 40.0);
            // wind crust: broken ridges, never a regular stripe
            float sa = (v_pos.x * 0.9 + v_pos.z * 0.43) * 2.2 + drift * 17.0 + fbm(v_pos.xz * 1.9 + 7.0) * 9.0;
            float crust = smoothstep(0.35, 0.75, fbm(v_pos.xz * 0.23 + 60.0));
            float hol = fbm(v_pos.xz * 0.09 + 80.0);
            vec3 s_col = mix(vec3(0.74, 0.81, 0.92), vec3(0.90, 0.91, 0.92), smoothstep(0.30, 0.65, hol)) * (0.86 + 0.16 * drift + 0.025 * sin(sa) * crust);
            // trodden, thin snow lets the ground colour through at its edges
            alb = mix(alb, s_col, sn);
            rough = mix(rough, 0.78, sn);
            f0 = mix(f0, 0.03, sn);
            vec3 sn_n = normalize(vec3(0, 1, 0) + vec3(0.9, 0.0, 0.43) * cos(sa) * 0.07 * crust + vec3(drift - 0.45, 0.0, vnoise(v_pos.xz * 0.9) - 0.5) * 0.35);
            n = normalize(mix(n, sn_n, sn));
        }
    } else if (m == 1) {
        // built: streaks down walls, patches on roofs
        vec2 q = abs(n.y) > 0.5 ? v_pos.xz : vec2(v_pos.x + v_pos.z, v_pos.y * 0.35);
        float stain = fbm(q * 0.9);
        float fine = vnoise(q * 9.0);
        alb *= 0.80 + 0.28 * stain + 0.10 * (fine - 0.5);
        rough = 0.78 + 0.15 * stain;
        alb *= mix(1.0, 0.7, wet);
        rough = mix(rough, 0.35, wet);
        if (!hull && surf.x > 0.0) {
            float sn = snow_mask(v_pos, min(1.0, surf.x * 1.25)) * smoothstep(0.6, 0.9, normalize(v_n).y);
            alb = mix(alb, vec3(0.80, 0.84, 0.88), sn);
            rough = mix(rough, 0.78, sn);
        }
    } else if (m == 2) {
        float leaf = vnoise(v_pos.xz * 7.0 + v_pos.y * 5.0);
        float clump = vnoise(v_pos.xz * 1.7 + v_pos.y * 1.1);
        alb *= 0.62 + 0.45 * leaf + 0.30 * (clump - 0.5);
        ao *= 0.70 + 0.30 * clump;
        n = normalize(n + (vec3(leaf, clump, vnoise(v_pos.zx * 6.0)) - 0.5) * 0.9);
        rough = 0.6;
        trans = 0.35;
        rough = mix(rough, 0.3, wet);
        if (surf.x > 0.0) {
            float sn = snow_mask(v_pos * 3.0, surf.x) * smoothstep(0.3, 0.9, normalize(v_n).y) * 0.85;
            alb = mix(alb, vec3(0.80, 0.84, 0.88), sn);
            trans *= 1.0 - sn;
        }
    } else if (m == 3) {
        rough = 0.42; f0 = 0.5;
        alb *= 0.9 + 0.2 * vnoise(v_pos.xz * 6.0 + v_pos.y * 4.0);
    } else if (m == 4) {
        rough = 0.85;
        alb *= 0.9 + 0.2 * vnoise(v_pos.xz * 23.0 + v_pos.y * 19.0);
    } else if (m == 6) {
        rough = 0.08; f0 = 0.02;
    }

    vec3 col;
    if (m == 5) {
        col = alb * 40.0;
    } else {
        float ndl = dot(n, sun_dir);
        float geo_ndl = dot(normalize(v_n), sun_dir);
        float sh_ = (sun_col.r + sun_col.g + sun_col.b > 0.0001) ? shadow(n, geo_ndl) : 0.0;
        // overcast: the "sun" is a bright patch of cloud, so shadows go faint
        float direct_vis = mix(sh_, mix(sh_, 1.0, 0.75), params.y);
        vec3 direct = sun_col * max(ndl, 0.0) * direct_vis * step(0.0, geo_ndl + trans);
        // light through leaves
        direct += sun_col * trans * max(-ndl, 0.0) * 0.6 * sh_ * vec3(0.8, 1.0, 0.4);

        vec3 amb = irradiance(n) * ao;
        col = alb * (direct / 3.14159265 + amb);

        // specular: sun highlight plus a sky reflection, both rough-aware
        vec3 h = normalize(v + sun_dir);
        float a = rough * rough;
        float ndh = max(dot(n, h), 0.0);
        float d = a * a / (3.14159265 * pow(ndh * ndh * (a * a - 1.0) + 1.0, 2.0));
        float fres = f0 + (1.0 - f0) * pow(1.0 - max(dot(n, v), 0.0), 5.0);
        col += sun_col * d * fres * max(ndl, 0.0) * direct_vis * 0.25;
        vec3 r = reflect(-v, n);
        vec3 skyr = mix(horizon, zenith, pow(max(r.y, 0.0), 0.45));
        skyr += haze_sun * pow(max(dot(normalize(vec3(r.x, 0.0, r.z) + 1e-4), normalize(vec3(sun_dir.x, 0.0, sun_dir.z) + 1e-4)), 0.0), 3.0) * pow(1.0 - max(r.y, 0.0), 4.0);
        col += skyr * fres * (1.0 - rough) * (1.0 - rough) * ao;
    }

    // aerial perspective along the view ray
    float t = 1.0 - exp(-dist * params.x);
    float toward = pow(max(dot(-v, sun_dir), 0.0), 4.0);
    col = mix(col, mix(haze_away, haze_sun, toward), t);
    frag = vec4(col, 1.0);
}
"#;

const OVERLAY_VS: &str = r#"#version 330
in vec3 position;
in vec4 color0;
uniform mat4 Model;
uniform mat4 Projection;
out vec4 v_col;
out vec3 v_pos;
void main() {
    vec4 w = Model * vec4(position, 1.0);
    v_pos = w.xyz;
    v_col = color0 / 255.0;
    gl_Position = Projection * w;
}
"#;

const OVERLAY_FS: &str = r#"#version 330
in vec4 v_col;
in vec3 v_pos;
uniform vec3 cam_pos;
uniform vec3 haze_col;
uniform vec4 level;
out vec4 frag;
void main() {
    vec3 c = pow(v_col.rgb, vec3(2.2)) * level.rgb;
    float t = 1.0 - exp(-distance(v_pos, cam_pos) * level.w);
    frag = vec4(mix(c, haze_col, t), v_col.a);
}
"#;

const SHADOW_VS: &str = r#"#version 330
in vec3 position;
in vec2 texcoord;
in vec4 color0;
uniform mat4 Model;
uniform mat4 Projection;
void main() { gl_Position = Projection * Model * vec4(position, 1.0); }
"#;

const SHADOW_FS: &str = r#"#version 330
out vec4 frag;
void main() { frag = vec4(1.0); }
"#;

const POST_VS: &str = r#"#version 330
in vec3 position;
in vec2 texcoord;
in vec4 color0;
uniform mat4 Model;
uniform mat4 Projection;
void main() { gl_Position = Projection * Model * vec4(position, 1.0); }
"#;

// Centre-weighted average luminance, eased toward over time (operator lag).
const METER_FS: &str = r#"#version 330
uniform sampler2D Scene;
uniform sampler2D Prev;
uniform float k;
out vec4 frag;
void main() {
    float sum = 0.0, wsum = 0.0;
    for (int y = 0; y < 16; y++) for (int x = 0; x < 24; x++) {
        vec2 uv = (vec2(x, y) + 0.5) / vec2(24.0, 16.0);
        vec3 c = texture(Scene, uv).rgb;
        float l = dot(c, vec3(0.2126, 0.7152, 0.0722));
        vec2 d = uv - 0.5;
        float w = exp(-dot(d, d) * 5.0);
        sum += log(max(l, 1e-5)) * w;
        wsum += w;
    }
    float cur = exp(sum / wsum);
    float prev = texture(Prev, vec2(0.5)).r;
    frag = vec4(mix(prev, cur, k), 0.0, 0.0, 1.0);
}
"#;

// Shared exposure rule: aim mid-grey, within what aperture and gain allow.
const EXPOSE: &str = "";

const BRIGHT_FS: &str = r#"#version 330
uniform sampler2D Scene;
uniform sampler2D Meter;
uniform vec2 inv;
uniform float ev_bias;
out vec4 frag;
float exposure() {
    float l = texture(Meter, vec2(0.5)).r;
    return clamp(0.16 / max(l, 1e-5), 0.02, 22.0) * exp2(ev_bias);
}
void main() {
    vec2 uv = gl_FragCoord.xy * inv;
    vec3 c = vec3(0.0);
    c += texture(Scene, uv + inv * vec2(-0.5, -0.5)).rgb;
    c += texture(Scene, uv + inv * vec2(0.5, -0.5)).rgb;
    c += texture(Scene, uv + inv * vec2(-0.5, 0.5)).rgb;
    c += texture(Scene, uv + inv * vec2(0.5, 0.5)).rgb;
    c *= 0.25 * exposure();
    // only what the sensor could not hold
    vec3 over = max(c - 1.6, 0.0);
    frag = vec4(min(over, vec3(12.0)), 1.0);
}
"#;

const BLUR_FS: &str = r#"#version 330
uniform sampler2D Src;
uniform vec2 inv;
uniform vec2 dir;
out vec4 frag;
void main() {
    vec2 uv = gl_FragCoord.xy * inv;
    vec3 c = texture(Src, uv).rgb * 0.227027;
    c += (texture(Src, uv + dir * inv * 1.3846).rgb + texture(Src, uv - dir * inv * 1.3846).rgb) * 0.316216;
    c += (texture(Src, uv + dir * inv * 3.2308).rgb + texture(Src, uv - dir * inv * 3.2308).rgb) * 0.070270;
    frag = vec4(c, 1.0);
}
"#;

const LENS_FS: &str = r#"#version 330
uniform sampler2D Scene;
uniform sampler2D Bloom;
uniform sampler2D Meter;
uniform vec2 inv;
uniform vec4 lens;   // vignette stops, CA px, ev bias, flash
uniform float frame;
out vec4 frag;

float hash(vec3 p) {
    p = fract(p * 0.1031);
    p += dot(p, p.zyx + 31.32);
    return fract((p.x + p.y) * p.z);
}
// broadcast camera: straight line, then a soft knee into white
vec3 shoulder(vec3 x) {
    const float knee = 0.72;
    vec3 hi = knee + (1.0 - knee) * (1.0 - exp(-(x - knee) / (1.0 - knee)));
    return mix(x, hi, step(knee, x));
}
vec3 oetf(vec3 c) {
    return mix(12.92 * c, 1.055 * pow(c, vec3(1.0 / 2.4)) - 0.055, step(0.0031308, c));
}
void main() {
    vec2 uv = gl_FragCoord.xy * inv;
    vec2 d = uv - 0.5;
    float aspect = inv.y / inv.x;
    vec2 dn = d * vec2(aspect, 1.0) / (0.5 * length(vec2(aspect, 1.0)));   // 1.0 at the corner
    float r2 = dot(dn, dn);

    // lateral chromatic aberration: red and blue land at slightly different radii
    vec2 ca = d * r2 * lens.y * inv * 2.0;
    vec3 c;
    c.r = texture(Scene, uv + ca).r;
    c.g = texture(Scene, uv).g;
    c.b = texture(Scene, uv - ca).b;
    c += lens.w;

    float l = texture(Meter, vec2(0.5)).r;
    float want = 0.14 / max(l, 1e-5);
    // aperture opens to here cleanly; past it the operator adds gain
    const float open = 2.2;
    float ex = clamp(want, 0.02, 22.0) * exp2(lens.z);
    float gain = max(ex / open, 1.0);
    c *= ex;
    c += texture(Bloom, uv).rgb * 0.55;

    // optical falloff
    float k = pow(2.0, lens.x * 0.5) - 1.0;
    c *= 1.0 / pow(1.0 + k * r2, 2.0);

    // gain drains colour and adds noise, worst in the shadows
    float g = clamp(log2(gain) / 4.5, 0.0, 1.0);
    float lum = dot(c, vec3(0.2126, 0.7152, 0.0722));
    c = mix(c, vec3(lum), g * 0.55);
    vec3 nz = vec3(hash(vec3(gl_FragCoord.xy, frame)),
                   hash(vec3(gl_FragCoord.xy + 37.0, frame + 1.0)),
                   hash(vec3(gl_FragCoord.xy + 91.0, frame + 2.0))) - 0.5;
    float amt = 0.006 + g * 0.11;
    c += (mix(vec3(nz.g), nz, 0.35)) * amt * (0.35 + sqrt(max(lum, 0.0)));

    // camera picture profile: a little contrast about mid grey, deeper blacks
    c = 0.18 * pow(max(c, 0.0) / 0.18, vec3(1.16));
    c = shoulder(max(c, 0.0));
    frag = vec4(oetf(clamp(c, 0.0, 1.0)), 1.0);
}
"#;

// -------------------------------------------------------------- sky states

use crate::sky::{self, SkyState};

/// Sun and sky colours for the six clear states, measured from Blender's
/// atmosphere renders (`source_assets/blender/sky/`). Night and the two cloud
/// states are still set by hand below.
fn baked_look(name: &str) -> Option<[Vec3; 4]> {
    static TABLE: std::sync::OnceLock<Vec<(String, [Vec3; 4])>> = std::sync::OnceLock::new();
    let t = TABLE.get_or_init(|| {
        include_str!("../assets/sky/looks.txt")
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .filter_map(|l| {
                let mut it = l.split_whitespace();
                let name = it.next()?.to_string();
                let n: Vec<f32> = it.filter_map(|x| x.parse().ok()).collect();
                if n.len() != 12 {
                    return None;
                }
                let v = |i: usize| vec3(n[i], n[i + 1], n[i + 2]);
                Some((name, [v(0), v(3), v(6), v(9)]))
            })
            .collect()
    });
    t.iter().find(|(n, _)| n == name).map(|(_, v)| *v)
}

pub fn state_look(s: SkyState, moon_lit: f32) -> SkyLook {
    let v = |r: f32, g: f32, b: f32, k: f32| vec3(r, g, b) * k;
    let key = match s {
        SkyState::Dawn => "dawn",
        SkyState::Morning => "morning",
        SkyState::Noon => "noon",
        SkyState::Afternoon => "afternoon",
        SkyState::GoldenHour => "golden",
        SkyState::Dusk => "dusk",
        _ => "",
    };
    let hand = state_look_hand(s, moon_lit);
    if let Some([sun, zenith, horizon, glow]) = baked_look(key) {
        return SkyLook { sun, zenith, horizon, glow, ..hand };
    }
    let _ = v;
    hand
}

fn state_look_hand(s: SkyState, moon_lit: f32) -> SkyLook {
    let v = |r: f32, g: f32, b: f32, k: f32| vec3(r, g, b) * k;
    match s {
        SkyState::Dawn => SkyLook { sun: v(1.0, 0.45, 0.22, 0.35), zenith: v(0.16, 0.22, 0.42, 0.22), horizon: v(0.62, 0.50, 0.50, 0.30), glow: v(1.0, 0.52, 0.26, 0.9), haze: 0.00010, diffusion: 0.35 },
        SkyState::Morning => SkyLook { sun: v(1.0, 0.90, 0.76, 8.5), zenith: v(0.17, 0.36, 0.82, 0.85), horizon: v(0.62, 0.74, 0.92, 1.05), glow: v(1.0, 0.86, 0.66, 0.9), haze: 0.00014, diffusion: 0.0 },
        SkyState::Noon => SkyLook { sun: v(1.0, 0.97, 0.91, 11.0), zenith: v(0.15, 0.33, 0.86, 1.0), horizon: v(0.56, 0.70, 0.95, 1.15), glow: v(1.0, 0.95, 0.85, 0.5), haze: 0.00010, diffusion: 0.0 },
        SkyState::Afternoon => SkyLook { sun: v(1.0, 0.91, 0.74, 9.0), zenith: v(0.17, 0.35, 0.80, 0.9), horizon: v(0.66, 0.74, 0.88, 1.1), glow: v(1.0, 0.84, 0.60, 1.0), haze: 0.00015, diffusion: 0.0 },
        SkyState::GoldenHour => SkyLook { sun: v(1.0, 0.58, 0.27, 4.2), zenith: v(0.17, 0.27, 0.56, 0.5), horizon: v(0.80, 0.62, 0.50, 0.7), glow: v(1.0, 0.55, 0.22, 2.2), haze: 0.00020, diffusion: 0.05 },
        SkyState::Dusk => SkyLook { sun: v(1.0, 0.36, 0.16, 0.18), zenith: v(0.12, 0.16, 0.36, 0.16), horizon: v(0.60, 0.38, 0.40, 0.24), glow: v(1.0, 0.40, 0.18, 0.8), haze: 0.00009, diffusion: 0.4 },
        SkyState::Night => SkyLook { sun: v(0.62, 0.72, 1.0, 0.006 + 0.05 * moon_lit), zenith: v(0.25, 0.32, 0.60, 0.004 + 0.004 * moon_lit), horizon: v(0.30, 0.34, 0.50, 0.005 + 0.004 * moon_lit), glow: Vec3::ZERO, haze: 0.00016, diffusion: 0.1 },
        SkyState::Overcast => SkyLook { sun: v(1.0, 0.98, 0.95, 1.1), zenith: v(0.80, 0.84, 0.90, 1.5), horizon: v(0.70, 0.74, 0.80, 0.95), glow: v(1.0, 1.0, 1.0, 0.15), haze: 0.00040, diffusion: 1.0 },
        SkyState::Storm => SkyLook { sun: v(0.9, 0.95, 1.0, 0.25), zenith: v(0.42, 0.46, 0.54, 0.42), horizon: v(0.36, 0.40, 0.46, 0.30), glow: Vec3::ZERO, haze: 0.0011, diffusion: 1.0 },
    }
}

/// The light at a site at a moment. `cover` 0 clear .. 1 overcast, `storm`
/// 0..1 on top of that.
pub fn light_at(m: sky::Moment, lat: f32, lon: f32, cover: f32, storm: f32, ground: Vec3) -> Light {
    let sun = m.sun_at(lat, lon);
    let moon = m.moon_at(lat, lon);
    let moon_up = (moon.elevation.to_degrees() / 10.0).clamp(0.0, 1.0);
    let moon_lit = m.moon_illumination() * moon_up;
    let (a, b, t) = sky::clear_sky_blend(sun, m.solar_minute(lon));
    let mut look = state_look(a, moon_lit).lerp(state_look(b, moon_lit), t);
    // night scales everything under cloud; day cross-fades to the cloud states
    let day = ((sun.elevation.to_degrees() + 6.0) / 12.0).clamp(0.0, 1.0);
    let dim = look.zenith.max_element().max(look.sun.max_element() * 0.1);
    let fade = |cl: SkyLook, k: f32, look: SkyLook| {
        let mut c = cl;
        let s = day + (1.0 - day) * (dim / 1.0).min(1.0) * 0.5;
        c.sun *= s;
        c.zenith *= s;
        c.horizon *= s;
        c.glow *= s;
        look.lerp(c, k)
    };
    look = fade(state_look(SkyState::Overcast, moon_lit), cover.clamp(0.0, 1.0), look);
    look = fade(state_look(SkyState::Storm, moon_lit), storm.clamp(0.0, 1.0), look);
    // the direct light comes from the sun while it is up, else the moon
    let s = sun.dir();
    let mo = moon.dir();
    let dir = if sun.elevation.to_degrees() > -4.0 {
        // keep a usable shadow direction through sunset
        let mut d = vec3(s[0], s[1].max(0.045), s[2]);
        d = d.normalize();
        d
    } else if moon_up > 0.0 {
        vec3(mo[0], mo[1].max(0.08), mo[2]).normalize()
    } else {
        look.sun = Vec3::ZERO;
        vec3(0.3, 0.9, 0.2).normalize()
    };
    Light::new(dir, look, ground)
}
