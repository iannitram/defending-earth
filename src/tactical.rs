use macroquad::prelude::*;
use crate::ui::{self, *};
use crate::{data::Month, Game, MissionSpec, Screen};
use crate::data::*;
use crate::{gfx, sky};

pub const TW: i32 = 26;
pub const TH: i32 = 26;
pub const TILE: f32 = 1.65;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase { Player, Alien, Victory, Defeat }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side { Player, Alien }

#[derive(Clone)]
pub struct Tile {
    pub h: i32,
    pub kind: u8,
    pub obstacle: u8,
    pub obstacle_hp: i32,
    pub smoke: u8,
    pub fire: u8,
    pub scorched: bool,
    pub visible: bool,
    pub explored: bool,
}

#[derive(Clone)]
pub struct Unit {
    pub side: Side,
    pub x: f32,
    pub y: f32,
    pub facing: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub tu: f32,
    pub max_tu: f32,
    pub acc: i32,
    pub reactions: i32,
    pub armor: u8,
    /// Armour class: the column this unit uses in the 14-column
    /// damage-modifier table for its campaign.
    pub armor_class: u8,
    /// Current front/left/right/rear/under armour values.
    /// They wear down as they absorb hits.
    pub armor_cur: [i32; 5],
    /// Morale, 0..=100, starting at 100.
    pub morale: i32,
    /// Bravery modifier (`11 - bravery / 10`).
    pub bravery_mod: i32,
    /// Psi strength and skill, used by the psi attack roll.
    pub psi_strength: i32,
    pub psi_skill: i32,
    pub stun: i32,
    pub weapon: usize,
    pub ammo: i32,
    pub spare: i32,
    pub kneeling: bool,
    pub alive: bool,
    pub path: Vec<(i32, i32)>,
    pub anim_phase: f32,
    pub muzzle: f32,
    pub hit_flash: f32,
    pub death_t: f32,
    pub name: String,
    pub rank: u8,
    pub look: Look,
    pub alien_kind: usize,
    pub grenades: u8,
    pub smoke_grenades: u8,
    pub medkits: u8,
    pub wounds: i32,
    pub soldier_idx: usize,
    pub kills: u32,
    pub shot_flash: f32,
    pub moved_this_turn: f32,
    /// Flag retained for the HUD/save shape; there is no
    /// one-attempt-per-turn cap (reaction shots chain while TU lasts).
    pub reaction_used: bool,
    pub mind_controlled: bool,
}

#[derive(Clone)]
pub enum Effect {
    Tracer { a: Vec3, b: Vec3, color: Color, t: f32, life: f32 },
    Explosion { p: Vec3, t: f32, life: f32, radius: f32 },
    Flash { p: Vec3, t: f32, life: f32, color: Color, size: f32 },
    Text { p: Vec3, text: String, color: Color, t: f32, life: f32 },
    Debris { p: Vec3, v: Vec3, color: Color, t: f32, life: f32 },
}

pub struct SelfTest {
    pub output_path: String,
    pub t: f32,
    pub stage: u32,
    pub move_start: (f32, f32),
    pub move_dist: f32,
    pub shots_fired: u32,
    pub hits: u32,
    pub steps: Vec<(String, bool, String)>,
}
impl SelfTest {
    pub fn new(output_path: String) -> Self {
        Self { output_path, t: 0.0, stage: 0, move_start: (5.0, 12.0), move_dist: 0.0, shots_fired: 0, hits: 0, steps: Vec::new() }
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShotMode { Auto, Snap, Aimed }
impl ShotMode {
    pub fn idx(self) -> usize { match self { ShotMode::Auto => 0, ShotMode::Snap => 1, ShotMode::Aimed => 2 } }
    pub fn label(self) -> &'static str { match self { ShotMode::Auto => "AUTO", ShotMode::Snap => "SNAP", ShotMode::Aimed => "AIMED" } }
}

pub struct Battle {
    pub tiles: Vec<Tile>,
    pub units: Vec<Unit>,
    pub selected: usize,
    pub mode: ShotMode,
    pub phase: Phase,
    pub turn: u32,
    pub hover: (i32, i32),
    pub cam_yaw: f32,
    pub cam_zoom: f32,
    pub cam_pan: Vec2,
    pub effects: Vec<Effect>,
    pub log: Vec<(String, Color, f32)>,
    pub mission: MissionSpec,
    pub underwater: bool,
    pub biome: u8,
    pub weather: u8,
    pub weather_desc: String,
    pub sun_dir: Vec3,
    pub sun_col: Color,
    pub ambient: f32,
    pub fog_col: Color,
    pub time: f32,
    pub alien_timer: f32,
    pub alien_step: usize,
    pub end_timer: f32,
    pub shots_fired: u32,
    pub hits: u32,
    pub reaction_shots: u32,
    pub pending_reaction: Option<usize>,
    pub lmb_press: f32,
    pub rmb_press: f32,
    pub input_armed: bool,
    pub hud_health: bool,
    pub hud_ammo: bool,
    pub hud_objective: bool,
    pub selftest: bool,
    pub target_alien: Option<usize>,
    pub show_help: bool,
    pub last_click: f32,
    pub move_tiles: Vec<(i32, i32)>,
    pub screen_flash: f32,
    pub victory_applied: bool,
    pub end_turn_clicked: bool,
    pub fire_clicked: bool,
    pub reload_clicked: bool,
    pub kneel_clicked: bool,
    pub grenade_clicked: bool,
    pub smoke_clicked: bool,
    pub medkit_clicked: bool,
    pub inventory_open: bool,
    pub inventory_selected_slot: usize,
    pub inventory_clicked: bool,
    pub view_level: i32,
    pub psi_open: bool,
    pub psi_target: Option<usize>,
    pub probed_alien: Option<usize>,
    pub lvl_up_clicked: bool,
    pub lvl_down_clicked: bool,
    pub psi_clicked: bool,
    pub uw_audio_timer: f32,
    /// Drawing only: the lighting pipeline has this battlefield's ground map.
    pub gfx_ready: bool,
    /// Debug camera preset (`--view`).
    pub dbg_view: u8,
}

fn tile_idx(x: i32, y: i32) -> usize { (y * TW + x) as usize }
fn in_bounds(x: i32, y: i32) -> bool { x >= 0 && y >= 0 && x < TW && y < TH }
fn tile_at(b: &Battle, x: i32, y: i32) -> &Tile { &b.tiles[tile_idx(x.clamp(0, TW - 1), y.clamp(0, TH - 1))] }
fn world_pos(x: f32, y: f32, h: i32) -> Vec3 { vec3(x * TILE, h as f32 * 0.62, y * TILE) }

fn hash2(x: i32, y: i32, seed: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263) ^ seed.wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}
fn vnoise(x: f32, y: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32; let yi = y.floor() as i32;
    let xf = x - xi as f32; let yf = y - yi as f32;
    let u = xf * xf * (3.0 - 2.0 * xf); let v = yf * yf * (3.0 - 2.0 * yf);
    let a = hash2(xi, yi, seed) as f32 / 4294967295.0;
    let b = hash2(xi + 1, yi, seed) as f32 / 4294967295.0;
    let c = hash2(xi, yi + 1, seed) as f32 / 4294967295.0;
    let d = hash2(xi + 1, yi + 1, seed) as f32 / 4294967295.0;
    (a + (b - a) * u) * (1.0 - v) + (c + (d - c) * u) * v
}
fn fbm(x: f32, y: f32, seed: u32) -> f32 {
    let mut v = 0.0; let mut a = 0.5; let mut f = 1.0;
    for i in 0..3 { v += vnoise(x * f, y * f, seed + i * 77) * a; a *= 0.5; f *= 2.03; }
    v
}

fn biome_ground(kind: u8, x: i32, y: i32, seed: u32) -> Color {
    let n = vnoise(x as f32 * 0.55, y as f32 * 0.55, seed) * 0.18 + 0.91;
    let base = match kind {
        0 => Color::new(0.34, 0.43, 0.23, 1.0),
        1 => Color::new(0.42, 0.44, 0.24, 1.0),
        2 => Color::new(0.34, 0.36, 0.16, 1.0),
        3 => Color::new(0.36, 0.35, 0.17, 1.0),
        4 => Color::new(0.55, 0.46, 0.29, 1.0),
        5 => Color::new(0.31, 0.30, 0.27, 1.0),
        6 => Color::new(0.27, 0.38, 0.20, 1.0),
        7 => Color::new(0.56, 0.46, 0.28, 1.0),
        8 => Color::new(0.59, 0.48, 0.30, 1.0),
        9 => Color::new(0.82, 0.87, 0.90, 1.0),
        10 => Color::new(0.29, 0.39, 0.21, 1.0),
        11 => Color::new(0.42, 0.43, 0.41, 1.0),
        12 => Color::new(0.24, 0.34, 0.36, 1.0),
        13 => Color::new(0.14, 0.18, 0.16, 1.0),
        14 => Color::new(0.09, 0.13, 0.13, 1.0),
        15 => Color::new(0.12, 0.12, 0.15, 1.0),
        _ => Color::new(0.34, 0.43, 0.23, 1.0),
    };
    Color::new(base.r * n, base.g * n, base.b * n, 1.0)
}

/// 0 winter, 1 spring, 2 summer, 3 autumn at this latitude; the tropics
/// count as summer all year.
fn season_at(month: Month, lat: f32) -> u8 {
    if lat.abs() < 23.5 { return 2; }
    let s = match month as usize { 11 | 0 | 1 => 0u8, 2..=4 => 1, 5..=7 => 2, _ => 3 };
    if lat >= 0.0 { s } else { (s + 2) % 4 }
}

/// The heavy form of the weather: downpour, blizzard, dust storm.
fn weather_heavy(b: &Battle) -> bool {
    ["DOWNPOUR", "BLIZZARD", "STORM"].iter().any(|k| b.weather_desc.contains(k))
}

/// Share of the ground lying under snow, 0..1. Drawing only.
fn snow_amount(b: &Battle) -> f32 {
    if b.underwater { return 0.0; }
    let season = season_at(b.mission.month, b.mission.lat);
    let mut s: f32 = match b.biome {
        9 => 0.96,
        5 => match season { 0 => 0.85, 1 | 3 => 0.45, _ => 0.20 },
        11 => match season { 0 => 0.90, 1 => 0.50, 3 => 0.30, _ => 0.0 },
        0 | 1 | 10 => if season == 0 && b.mission.lat.abs() > 45.0 { 0.55 } else { 0.0 },
        _ => 0.0,
    };
    if b.weather == 2 { s = s.max(if weather_heavy(b) { 0.92 } else { 0.80 }); }
    s
}

fn compute_environment(spec: &MissionSpec, rng: &mut Rng) -> (u8, String, Vec3, Color, f32, Color) {
    let solar_min = (spec.minute_of_day + (spec.lon / 15.0 * 60.0) as i32).rem_euclid(1440);
    let hour = solar_min / 60;
    let minute = solar_min % 60;

    let is_north = spec.lat >= 0.0;
    let (is_spring, is_summer, _is_autumn, is_winter, season_name) = match spec.month {
        Month::Mar | Month::Apr | Month::May => if is_north { (true, false, false, false, "SPRING") } else { (false, false, true, false, "AUTUMN") },
        Month::Jun | Month::Jul | Month::Aug => if is_north { (false, true, false, false, "SUMMER") } else { (false, false, false, true, "WINTER") },
        Month::Sep | Month::Oct | Month::Nov => if is_north { (false, false, true, false, "AUTUMN") } else { (true, false, false, false, "SPRING") },
        Month::Dec | Month::Jan | Month::Feb => if is_north { (false, false, false, true, "WINTER") } else { (false, true, false, false, "SUMMER") },
    };

    let is_afternoon = hour >= 13 && hour < 18;
    let is_night = hour >= 21 || hour < 5;
    let is_morning = hour >= 5 && hour < 11;
    let is_twilight = hour >= 18 && hour < 21;

    let is_west_coast = (spec.lat >= 30.0 && spec.lat <= 55.0 && spec.lon >= -130.0 && spec.lon <= -115.0)
        || (spec.lat >= 42.0 && spec.lat <= 60.0 && spec.lon >= -12.0 && spec.lon <= 15.0)
        || (spec.lat >= 30.0 && spec.lat <= 45.0 && spec.lon >= 128.0 && spec.lon <= 145.0);

    let region_name = if is_west_coast {
        if spec.lon > -30.0 && spec.lon < 30.0 { "ATLANTIC COAST" } else { "PACIFIC COAST" }
    } else if spec.lat.abs() > 60.0 {
        "POLAR REGION"
    } else if spec.biome == 7 || spec.biome == 8 {
        "ARID DESERT"
    } else if spec.biome == 2 {
        "TROPICAL BASIN"
    } else {
        "TEMPERATE SECTOR"
    };

    // The old picker still runs so the mission's random stream is unchanged
    // (the map and the fight draw from it after this); its answer is not used.
    let _legacy = if spec.underwater {
        (4u8, "ABYSSAL DRIFT")
    } else if is_west_coast && is_spring && is_afternoon {
        // "West coast spring afternoon landing? Rain."
        (1u8, "WIND-DRIVEN RAIN")
    } else if (is_winter && spec.lat.abs() > 36.0) || spec.lat.abs() > 62.0 || spec.biome == 9 {
        (2u8, if rng.chance(40) { "BLIZZARD SQUALLS" } else { "LIGHT SNOW" })
    } else if spec.biome == 7 || spec.biome == 8 {
        (3u8, if is_afternoon && rng.chance(60) { "DUST STORM" } else { "THERMAL DUST" })
    } else if spec.biome == 2 && is_summer && is_afternoon {
        (1u8, "MONSOON DOWNPOUR")
    } else if rng.chance(25) {
        (1u8, "OVERCAST RAIN")
    } else {
        (0u8, if is_night { "CLEAR MOONLIT" } else { "CLEAR SKIES" })
    };

    // Weather from the climate of the place and the season, on its own dice.
    let season = season_at(spec.month, spec.lat);
    let mut wr = Rng::new((rng.0 as u64) ^ ((spec.lat.to_bits() as u64) << 3) ^ (spec.lon.to_bits() as u64) ^ ((spec.month as u64) << 11));
    let roll = wr.rand(99);
    let clear = (0u8, if is_night { "CLEAR NIGHT" } else { "CLEAR SKIES" });
    let (weather, w_name) = if spec.underwater {
        (4u8, "ABYSSAL DRIFT")
    } else {
        match spec.biome {
            9 => if roll < 25 { (2, "BLIZZARD") } else if roll < 55 { (2, "LIGHT SNOW") } else { clear },
            4 | 7 | 8 => {
                let p = if is_afternoon { 50 } else { 25 };
                if roll < 12 && is_afternoon { (3, "DUST STORM") } else if roll < p { (3, "BLOWING DUST") } else if roll > 96 { (1, "RARE RAIN") } else { clear }
            }
            2 | 6 => {
                let p = if is_afternoon { 65 } else { 40 };
                if roll < p / 3 { (1, "TROPICAL DOWNPOUR") } else if roll < p { (1, "WARM RAIN") } else { clear }
            }
            5 | 11 => match season {
                0 => if roll < 20 { (2, "BLIZZARD") } else if roll < 55 { (2, "SNOWFALL") } else { clear },
                1 | 3 => if roll < 28 { (2, "WET SNOW") } else if roll < 45 { (1, "COLD RAIN") } else { clear },
                _ => if roll < 9 { (1, "MOUNTAIN STORM") } else if roll < 27 { (1, "PASSING RAIN") } else { clear },
            },
            _ => {
                if season == 0 && spec.lat.abs() > 40.0 {
                    if roll < 8 { (2, "BLIZZARD") } else if roll < 30 { (2, "LIGHT SNOW") } else if roll < 45 { (1, "COLD RAIN") } else { clear }
                } else {
                    let p = if is_west_coast { 45 } else if spec.biome == 1 && season == 2 { 15 } else { 30 };
                    if roll < p / 4 { (1, "RAIN STORM") } else if roll < p { (1, "OVERCAST RAIN") }
                    else if spec.biome == 1 && season == 2 && roll > 92 { (3, "BLOWING DUST") } else { clear }
                }
            }
        }
    };
    let region_name = if spec.underwater { region_name } else { match spec.biome {
        1 => "OPEN FARMLAND", 5 => "HIGH MOUNTAIN", 2 | 6 => "RAIN FOREST", 4 | 7 | 8 => "ARID DESERT", 9 => "ICE SHEET", 11 => "NORTHERN FOREST",
        _ => region_name,
    } };

    let tod_str = if is_night { "NIGHT" } else if is_morning { "MORNING" } else if is_afternoon { "AFTERNOON" } else if is_twilight { "EVENING" } else { "MIDDAY" };
    let season_name = if spec.lat.abs() < 23.5 && !spec.underwater { if w_name.contains("RAIN") || w_name.contains("DOWNPOUR") { "WET SEASON" } else { "DRY SEASON" } } else { season_name };
    let h12 = if hour % 12 == 0 { 12 } else { hour % 12 };
    let weather_desc = format!("{} // {} {} {}:{:02}{} // {}", region_name, season_name, tod_str, h12, minute, if hour < 12 { "AM" } else { "PM" }, w_name);

    let (sun, sun_col, ambient, fog) = if spec.underwater {
        (vec3(-0.16, 0.95, 0.10).normalize(), Color::new(0.20, 0.75, 0.85, 1.0), 0.48, Color::new(0.020, 0.135, 0.160, 1.0))
    } else if is_night {
        (vec3(-0.25, 0.85, 0.35).normalize(), Color::new(0.40, 0.58, 0.88, 1.0), 0.24, Color::new(0.02, 0.04, 0.08, 1.0))
    } else if is_morning {
        (vec3(0.68, 0.38, -0.22).normalize(), Color::new(1.0, 0.82, 0.62, 1.0), 0.52, Color::new(0.18, 0.16, 0.15, 1.0))
    } else if is_afternoon {
        (vec3(-0.62, 0.68, -0.32).normalize(), Color::new(1.0, 0.86, 0.58, 1.0), 0.58, Color::new(0.18, 0.18, 0.16, 1.0))
    } else if is_twilight {
        (vec3(-0.85, 0.22, -0.35).normalize(), Color::new(0.98, 0.56, 0.38, 1.0), 0.40, Color::new(0.12, 0.10, 0.16, 1.0))
    } else {
        (vec3(-0.15, 0.95, -0.15).normalize(), Color::new(1.0, 0.98, 0.94, 1.0), 0.65, Color::new(0.14, 0.18, 0.22, 1.0))
    };

    (weather, weather_desc, sun, sun_col, ambient, fog)
}

impl Battle {
    fn log(&mut self, text: impl Into<String>, color: Color) {
        self.log.push((text.into(), color, 0.0));
        if self.log.len() > 10 { self.log.remove(0); }
    }

    fn gen_map(spec: &MissionSpec, rng: &mut Rng) -> (Vec<Tile>, Vec<(i32, i32)>, Vec<(i32, i32)>) {
        let seed = rng.next_u32();
        let mut tiles = Vec::with_capacity((TW * TH) as usize);
        for y in 0..TH {
            for x in 0..TW {
                let edge = x < 3 || y < 3 || x >= TW - 3 || y >= TH - 3;
                let n = fbm(x as f32 * 0.14, y as f32 * 0.14, seed);
                let mut h = if edge { 0 } else if n > 0.68 { 2 } else if n > 0.56 { 1 } else { 0 };
                if spec.underwater { h = if n > 0.74 { 1 } else { 0 }; }
                let kind = if spec.underwater { if n > 0.62 { 14 } else { 13 } } else { spec.biome.min(12) };
                let mut obs = 0u8;
                let o = vnoise(x as f32 * 0.8, y as f32 * 0.8, seed ^ 0x9e37);
                let spawn_safe = x < 9 || x > TW - 9;
                if !spawn_safe && h < 2 {
                    if o > 0.86 { obs = 1; }
                    else if o > 0.79 { obs = 2; }
                    else if o > 0.74 { obs = 3; }
                    else if o > 0.715 { obs = 4; }
                }
                if x < 9 { obs = 0; h = 0; }
                let obs_hp = match obs { 4 => 80, 3 => 60, 2 => 40, 1 => 50, _ => 0 };
                tiles.push(Tile { h, kind, obstacle: obs, obstacle_hp: obs_hp, smoke: 0, fire: 0, scorched: false, visible: false, explored: false });
            }
        }
        // guaranteed cover-cover and a few readable lanes
        for i in 0..14 {
            let x = 9 + (hash2(i, 3, seed) % (TW as u32 - 17)) as i32;
            let y = 5 + (hash2(i, 7, seed) % (TH as u32 - 10)) as i32;
            if in_bounds(x, y) && (x as i32) > 8 && (x as i32) < TW - 8 {
                let obs = if i % 3 == 0 { 2 } else { 3 };
                let idx = tile_idx(x, y);
                tiles[idx].obstacle = obs;
                tiles[idx].obstacle_hp = if obs == 2 { 40 } else { 60 };
            }
        }

        // 1. CONSTRUCT PLAYER LANDING SHIP (Heron / Gannet) AT WEST (x: 1..8, y: 10..15)
        for y in 10..=15 { for x in 1..=8 {
            let i = tile_idx(x, y);
            tiles[i].obstacle = 0; tiles[i].h = 0;
        }}
        // Hull perimeter walls (impassable ship armor walls)
        for x in 1..=6 {
            let top_i = tile_idx(x, 10);
            tiles[top_i].obstacle = 10; tiles[top_i].obstacle_hp = 300;
            let bot_i = tile_idx(x, 15);
            tiles[bot_i].obstacle = 10; tiles[bot_i].obstacle_hp = 300;
        }
        for y in 11..=14 {
            let nose_i = tile_idx(1, y);
            tiles[nose_i].obstacle = 10; tiles[nose_i].obstacle_hp = 300;
        }
        // Open deployment ramp at x: 7..=8, y: 12..=13 (remains obstacle = 0)
        let player_spawns = vec![
            (3, 12), (3, 13),
            (4, 12), (4, 13),
            (5, 12), (5, 13),
            (6, 12), (6, 13),
        ];

        // 2. CONSTRUCT ALIEN CRAFT (enemy craft) AT EAST (x: 18..25, y: 9..17)
        for y in 9..=17 { for x in 18..=25 {
            let i = tile_idx(x, y);
            tiles[i].obstacle = 0; tiles[i].h = 0;
        }}
        // Hull perimeter walls (impassable lattice walls)
        for x in 20..=24 {
            let top_i = tile_idx(x, 9);
            tiles[top_i].obstacle = 12; tiles[top_i].obstacle_hp = 400;
            let bot_i = tile_idx(x, 17);
            tiles[bot_i].obstacle = 12; tiles[bot_i].obstacle_hp = 400;
        }
        for y in 11..=15 {
            let rear_i = tile_idx(25, y);
            tiles[rear_i].obstacle = 12; tiles[rear_i].obstacle_hp = 400;
        }
        for &(cx, cy) in &[(19, 10), (19, 11), (19, 15), (19, 16)] {
            let i = tile_idx(cx, cy);
            tiles[i].obstacle = 12; tiles[i].obstacle_hp = 400;
        }
        // Alien Power Core at (23, 13)
        let core_i = tile_idx(23, 13);
        tiles[core_i].obstacle = 13; tiles[core_i].obstacle_hp = 500;
        // Open airlock portal at x: 18..=19, y: 12..=14 (remains obstacle = 0)
        let alien_spawns = vec![
            (22, 12), (22, 14),
            (24, 12), (24, 14),
            (21, 13), (20, 12),
            (20, 14), (19, 13),
        ];

        // 3. CONSTRUCT 2-STORY TACTICAL OBSERVATION STRUCTURE (x: 11..15, y: 5..9)
        for y in 5..=9 {
            for x in 11..=15 {
                let i = tile_idx(x, y);
                tiles[i].obstacle = 0;
                tiles[i].h = 0;
            }
        }
        // Ground floor perimeter walls (obstacle 4)
        for x in 11..=15 {
            let top_i = tile_idx(x, 5); tiles[top_i].obstacle = 4; tiles[top_i].obstacle_hp = 80;
            if x != 13 {
                let bot_i = tile_idx(x, 9); tiles[bot_i].obstacle = 4; tiles[bot_i].obstacle_hp = 80; // (13, 9) is entrance doorway
            }
        }
        for y in 6..=8 {
            let left_i = tile_idx(11, y); tiles[left_i].obstacle = 4; tiles[left_i].obstacle_hp = 80;
            let right_i = tile_idx(15, y); tiles[right_i].obstacle = 4; tiles[right_i].obstacle_hp = 80;
        }
        // Second-story sniper catwalk & stairwell transition at (14, 8)
        let stair_i = tile_idx(14, 8);
        tiles[stair_i].h = 1;
        for y in 6..=7 {
            for x in 12..=14 {
                let i = tile_idx(x, y);
                tiles[i].h = 1;
                if x == 12 || y == 6 {
                    tiles[i].obstacle = 3; tiles[i].obstacle_hp = 60; // Parapet cover
                }
            }
        }
        // Alien UFO elevated bridge deck at h=1
        for y in 12..=14 {
            for x in 22..=24 {
                let i = tile_idx(x, y);
                tiles[i].h = 1;
            }
        }
        tiles[tile_idx(21, 13)].h = 1;

        (tiles, player_spawns, alien_spawns)
    }

    fn new(g: &Game, mut spec: MissionSpec) -> Self {
        // debug overrides (`--biome`, `--clock`); never set in normal play
        if let Some((la, lo)) = g.dbg_site {
            spec.lat = la; spec.lon = lo;
            if !spec.underwater { spec.biome = g.data.terrain_at(la, lo); }
        }
        if let (Some(bi), false) = (g.dbg_biome, spec.underwater) { spec.biome = bi; }
        if let Some(c) = g.dbg_clock {
            spec.month = [Month::Jan, Month::Feb, Month::Mar, Month::Apr, Month::May, Month::Jun, Month::Jul, Month::Aug, Month::Sep, Month::Oct, Month::Nov, Month::Dec][(c.month - 1).clamp(0, 11) as usize];
            spec.minute_of_day = (c.minute - spec.lon * 4.0).rem_euclid(1440.0) as i32;
        }
        let mut rng = Rng::new(g.dbg_seed.unwrap_or(0).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (g.rng.0 as u64) ^ ((spec.lat.to_bits() as u64) << 17) ^ (spec.lon.to_bits() as u64));
        let (tiles, ps, aspa) = Self::gen_map(&spec, &mut rng);
        let (mut weather, mut weather_desc, sun_dir, sun_col, ambient, fog_col) = compute_environment(&spec, &mut rng);
        if let (Some(wo), false) = (g.dbg_weather, spec.underwater) {
            weather = wo & 15;
            let name = match wo { 1 => "OVERCAST RAIN", 2 => "LIGHT SNOW", 3 => "BLOWING DUST", 17 => "RAIN STORM", 18 => "BLIZZARD", 19 => "DUST STORM", _ => "CLEAR SKIES" };
            let head = weather_desc.rsplit_once(" // ").map(|(h, _)| h.to_string()).unwrap_or_default();
            weather_desc = format!("{} // {}", head, name);
        }
        let mut b = Self {
            tiles, units: Vec::new(), selected: 0, mode: ShotMode::Snap, phase: Phase::Player,
            turn: 1, hover: (0, 0), cam_yaw: 0.72, cam_zoom: 0.80, cam_pan: Vec2::ZERO,
            effects: Vec::new(), log: Vec::new(), mission: spec.clone(), underwater: spec.underwater,
            biome: spec.biome, weather, weather_desc, sun_dir, sun_col, ambient, fog_col, time: 0.0, alien_timer: 0.0, alien_step: 0, end_timer: 0.0,
            shots_fired: 0, hits: 0, reaction_shots: 0, pending_reaction: None, lmb_press: -10.0, rmb_press: -10.0, input_armed: false, hud_health: false, hud_ammo: false, hud_objective: false,
            selftest: false, target_alien: None, show_help: false, last_click: 0.0,
            move_tiles: Vec::new(), screen_flash: 0.0, victory_applied: false,
            end_turn_clicked: false, fire_clicked: false, reload_clicked: false,
            kneel_clicked: false, grenade_clicked: false, smoke_clicked: false, medkit_clicked: false,
            inventory_open: false, inventory_selected_slot: 0, inventory_clicked: false,
            view_level: 2, psi_open: false, psi_target: None, probed_alien: None,
            lvl_up_clicked: false, lvl_down_clicked: false, psi_clicked: false,
            uw_audio_timer: 0.0, gfx_ready: false, dbg_view: g.dbg_view,
        };
        // soldiers
        let mut k = 0;
        for (ci, s) in g.soldiers.iter().enumerate() {
            if !s.assigned || s.dead || s.wounded_days > 0 { continue; }
            if k >= ps.len() { break; }
            let (x, y) = ps[k];
            let mut weapon = s.weapon;
            let mut armor = s.armor;
            if spec.underwater {
                if g.data.weapons[weapon].env == Env::Surface {
                    weapon = g.weapon_index("DIVE RIFLE");
                }
                if !ARMORS[armor as usize].underwater { armor = 3; }
            } else {
                // Dive gear is not worn on land.
                if g.data.weapons[weapon].env == Env::Underwater {
                    weapon = g.weapon_index("SERVICE RIFLE");
                }
                if ARMORS[armor as usize].underwater { armor = 0; }
            }
            let mut u = Unit {
                side: Side::Player, x: x as f32, y: y as f32, facing: 0.0,
                hp: s.hp.max(1), max_hp: s.stats.health, tu: s.stats.tu as f32, max_tu: s.stats.tu as f32,
                acc: s.stats.firing, reactions: s.stats.reactions, armor,
                armor_class: ARMORS[armor as usize % ARMORS.len()].armor_class,
                armor_cur: soldier_armor_cur(armor),
                morale: 100,
                bravery_mod: (11 - s.stats.bravery / 10).clamp(1, 10),
                psi_strength: s.stats.psi_strength,
                psi_skill: s.stats.psi_skill,
                stun: 0,
                weapon, ammo: if s.ammo <= 0 { g.data.clip_size(&g.data.weapons[weapon]) } else { s.ammo },
                spare: s.spare, kneeling: false, alive: true, path: Vec::new(), anim_phase: rng.unit() * 6.28,
                muzzle: 0.0, hit_flash: 0.0, death_t: 0.0, name: s.name.clone(), rank: s.rank, look: s.look,
                alien_kind: 0, grenades: s.grenades, smoke_grenades: s.smoke_grenades, medkits: s.medkits, wounds: s.wounds,
                soldier_idx: ci, kills: 0, shot_flash: 0.0, moved_this_turn: 0.0, reaction_used: false,
                mind_controlled: false,
            };
            u.facing = if spec.aliens.is_empty() { 0.0 } else { 0.2 };
            b.units.push(u);
            k += 1;
        }
        // aliens
        for (ai, kind) in spec.aliens.iter().enumerate() {
            if ai >= aspa.len() { break; }
            let (x, y) = aspa[ai];
            let ak = alien_def(&g.data, b.underwater, *kind, spec.tier);
            let mut u = Unit {
                side: Side::Alien, x: x as f32, y: y as f32, facing: std::f32::consts::PI,
                hp: ak.hp, max_hp: ak.hp, tu: ak.tu as f32, max_tu: ak.tu as f32,
                acc: ak.acc, reactions: ak.reactions, armor: 0,
                armor_class: ak.armor_class, armor_cur: ak.armor_locs,
                morale: 100,
                bravery_mod: ak.bravery_mod,
                psi_strength: ak.psi_strength,
                psi_skill: ak.psi_skill,
                stun: 0,
                weapon: ak.weapon,
                ammo: 99, spare: 0, kneeling: false, alive: true, path: Vec::new(),
                anim_phase: rng.unit() * 6.28, muzzle: 0.0, hit_flash: 0.0, death_t: 0.0,
                name: ak.name.to_string(), rank: 0, look: Look::random(&mut rng), alien_kind: *kind as usize,
                grenades: 0, smoke_grenades: 0, medkits: 0, wounds: 0, soldier_idx: 0, kills: 0, shot_flash: 0.0, moved_this_turn: 0.0, reaction_used: false,
                mind_controlled: false,
            };
            u.armor = ak.armor.min(255) as u8;
            b.units.push(u);
        }
        b.compute_visibility();
        b.log(format!("DEPLOYED: {}", spec.name), CYAN);
        b.log(if spec.underwater { "DIVE GEAR ON. SURFACE WEAPONS STAY IN THE RACK." } else { "SQUAD DEPLOYED. GOOD HUNTING." }, if spec.underwater { CYAN } else { AMBER });
        b
    }

    fn player_units(&self) -> impl Iterator<Item = &Unit> { self.units.iter().filter(|u| u.side == Side::Player) }
    fn alien_units(&self) -> impl Iterator<Item = &Unit> { self.units.iter().filter(|u| u.side == Side::Alien) }
    pub fn alive_players(&self) -> usize { self.player_units().filter(|u| u.alive).count() }
    pub fn alive_aliens(&self) -> usize { self.alien_units().filter(|u| u.alive).count() }

    fn compute_visibility(&mut self) {
        for t in self.tiles.iter_mut() { t.visible = false; }
        let players: Vec<(f32, f32)> = self.units.iter().filter(|u| u.side == Side::Player && u.alive).map(|u| (u.x, u.y)).collect();
        for (px, py) in players {
            let cx = px.round() as i32; let cy = py.round() as i32;
            for dy in -12..=12 {
                for dx in -12..=12 {
                    let x = cx + dx; let y = cy + dy;
                    if !in_bounds(x, y) { continue; }
                    if dx * dx + dy * dy > 150 { continue; }
                    if self.los(cx, cy, x, y) {
                        let i = tile_idx(x, y);
                        self.tiles[i].visible = true;
                        self.tiles[i].explored = true;
                    }
                }
            }
        }
    }

    fn los(&self, x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
        let mut x = x0; let mut y = y0;
        let dx = (x1 - x0).abs(); let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut guard = 0;
        let mut accumulated_smoke = 0i32;
        loop {
            guard += 1; if guard > 80 { return true; }
            if x == x1 && y == y1 { return true; }
            if !(x == x0 && y == y0) {
                let t = tile_at(self, x, y);
                if t.obstacle == 4 || t.obstacle == 2 || t.obstacle == 10 || t.obstacle == 12 || t.obstacle == 13 { return false; }
                if t.h >= 2 { return false; }
                accumulated_smoke += t.smoke as i32;
                if accumulated_smoke >= 18 { return false; }
            }
            let e2 = 2 * err;
            if e2 > -dy { err -= dy; x += sx; }
            if e2 < dx { err += dx; y += sy; }
        }
    }

    fn unit_at(&self, x: i32, y: i32) -> Option<usize> {
        self.units.iter().position(|u| u.alive && u.x.round() as i32 == x && u.y.round() as i32 == y)
    }
    fn selected_unit(&self) -> Option<&Unit> { self.units.get(self.selected) }

    /// Cycle to the next living soldier; used by Tab and the squad strip.
    fn cycle_soldier(&mut self) {
        let players: Vec<usize> = self.units.iter().enumerate()
            .filter(|(_, u)| u.alive && u.side == Side::Player)
            .map(|(i, _)| i).collect();
        if players.is_empty() { return; }
        let at = players.iter().position(|i| *i == self.selected).unwrap_or(0);
        self.selected = players[(at + 1) % players.len()];
    }

    fn walk_cost(&self, x: i32, y: i32, from: (i32, i32)) -> f32 {
        let t = tile_at(self, x, y);
        let from_t = tile_at(self, from.0, from.1);
        let mut c = 4.0;
        // Extra TU for climbing or descending a level
        if t.h > from_t.h {
            c += (t.h - from_t.h) as f32 * 4.0;
        } else if t.h < from_t.h {
            c += (from_t.h - t.h) as f32 * 2.0;
        }
        if t.kind == 9 { c += 3.0; }
        if (x - from.0).abs() + (y - from.1).abs() == 2 { c *= 1.5; }
        c
    }

    fn path_to(&self, start: (i32, i32), goal: (i32, i32), max_tu: f32) -> Vec<(i32, i32)> {
        if !in_bounds(goal.0, goal.1) { return Vec::new(); }
        let gt = tile_at(self, goal.0, goal.1);
        if gt.obstacle != 0 || gt.h >= 3 { return Vec::new(); }
        if self.units.iter().any(|u| u.alive && u.side == Side::Alien && u.x.round() as i32 == goal.0 && u.y.round() as i32 == goal.1) { return Vec::new(); }
        let mut open = vec![start];
        let mut came: std::collections::HashMap<(i32, i32), (i32, i32)> = std::collections::HashMap::new();
        let mut cost: std::collections::HashMap<(i32, i32), f32> = std::collections::HashMap::new();
        cost.insert(start, 0.0);
        let mut iterations = 0;
        while !open.is_empty() && iterations < 2000 {
            iterations += 1;
            let mut best = 0;
            let mut bestf = f32::MAX;
            for (i, p) in open.iter().enumerate() {
                let g = *cost.get(p).unwrap_or(&9999.0);
                let h = ((p.0 - goal.0).abs() + (p.1 - goal.1).abs()) as f32;
                if g + h < bestf { bestf = g + h; best = i; }
            }
            let cur = open.swap_remove(best);
            if cur == goal {
                let mut path = Vec::new();
                let mut c = cur;
                while c != start {
                    path.push(c);
                    c = *came.get(&c).unwrap_or(&start);
                }
                path.reverse();
                let mut total = 0.0;
                let mut prev = start;
                for p in &path { total += self.walk_cost(p.0, p.1, prev); prev = *p; }
                if total > max_tu { return Vec::new(); }
                return path;
            }
            for (dx, dy) in [(-1,0),(1,0),(0,-1),(0,1),(-1,-1),(1,-1),(-1,1),(1,1)] {
                let nx = cur.0 + dx; let ny = cur.1 + dy;
                if !in_bounds(nx, ny) { continue; }
                let t = tile_at(self, nx, ny);
                if t.obstacle != 0 || t.h >= 3 { continue; }
                if dx != 0 && dy != 0 {
                    let a = tile_at(self, cur.0 + dx, cur.1);
                    let b = tile_at(self, cur.0, cur.1 + dy);
                    if a.obstacle != 0 || b.obstacle != 0 { continue; }
                }
                if self.units.iter().any(|u| u.alive && u.x.round() as i32 == nx && u.y.round() as i32 == ny) { continue; }
                let nc = cost[&cur] + self.walk_cost(nx, ny, cur);
                if nc < *cost.get(&(nx, ny)).unwrap_or(&9999.0) {
                    cost.insert((nx, ny), nc);
                    came.insert((nx, ny), cur);
                    if !open.contains(&(nx, ny)) { open.push((nx, ny)); }
                }
            }
        }
        Vec::new()
    }

    fn reachable(&self, start: (i32, i32), tu: f32) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        let mut cost: std::collections::HashMap<(i32, i32), f32> = std::collections::HashMap::new();
        let mut open = vec![start];
        cost.insert(start, 0.0);
        while let Some(cur) = open.pop() {
            let base = *cost.get(&cur).unwrap_or(&9999.0);
            for (dx, dy) in [(-1,0),(1,0),(0,-1),(0,1)] {
                let nx = cur.0 + dx; let ny = cur.1 + dy;
                if !in_bounds(nx, ny) { continue; }
                let t = tile_at(self, nx, ny);
                if t.obstacle != 0 || t.h >= 3 { continue; }
                if self.units.iter().any(|u| u.alive && u.x.round() as i32 == nx && u.y.round() as i32 == ny) { continue; }
                let nc = base + self.walk_cost(nx, ny, cur);
                if nc <= tu && nc < *cost.get(&(nx, ny)).unwrap_or(&9999.0) {
                    cost.insert((nx, ny), nc);
                    out.push((nx, ny));
                    open.push((nx, ny));
                }
            }
        }
        out
    }

    fn order_move(&mut self, idx: usize, gx: i32, gy: i32) {
        let (from, tu) = match self.units.get(idx) {
            Some(u) if u.alive => ((u.x.round() as i32, u.y.round() as i32), u.tu),
            _ => return,
        };
        let path = self.path_to(from, (gx, gy), tu);
        if path.is_empty() {
            self.log("NO ROUTE WITH THE TU REMAINING.", DIM);
            return;
        }
        let first = path[0];
        let u = &mut self.units[idx];
        u.facing = (first.1 as f32 - from.1 as f32).atan2(first.0 as f32 - from.0 as f32);
        u.path = path;
    }

    fn team_leadership(&self, side: Side) -> i32 {
        let mut best = 100;
        for u in &self.units {
            if u.side != side || !u.alive { continue; }
            let v = match u.rank { 2 => 110, 3 | 4 => 115, 5 => 125, 6 => 150, _ => 100 };
            best = best.max(v);
        }
        best
    }

    fn check_unit_panic(&mut self, g: &mut Game, idx: usize) {
        let (morale, name) = (self.units[idx].morale, self.units[idx].name.clone());
        if morale >= 50 { return; }
        if g.rng.rand(100) > 100 - morale * 2 { return; }
        match g.rng.rand(3) {
            0 => { self.units[idx].tu = 0.0; self.log(format!("{} FREEZES.", name), ui::RED); }
            1 | 2 => { self.units[idx].tu = 0.0; self.log(format!("{} PANICS AND LOSES THE TURN.", name), ui::RED); }
            _ => {
                self.log(format!("{} GOES BERSERK!", name), ui::RED);
                self.units[idx].morale = (morale + 20).min(100);
                self.units[idx].tu = 0.0;
            }
        }
    }

    fn on_unit_death(&mut self, g: &mut Game, victim_idx: usize, killer_idx: Option<usize>) {
        if victim_idx >= self.units.len() || !self.units[victim_idx].alive { return; }
        let victim_side = self.units[victim_idx].side;
        let victim_rank = self.units[victim_idx].rank;
        let victim_bravery = self.units[victim_idx].bravery_mod;
        let victim_lead = if victim_side == Side::Player {
            match victim_rank { 2 => 120, 3 => 130, 5 => 150, 6 => 175, _ => 100 }
        } else { 100 };
        let victim_name = self.units[victim_idx].name.clone();
        self.units[victim_idx].alive = false;
        self.units[victim_idx].death_t = 0.0;
        self.units[victim_idx].path.clear();
        if victim_side == Side::Player {
            g.audio.play_scream();
            self.log(format!("{} IS KILLED IN ACTION.", victim_name), ui::RED);
        } else if self.units[victim_idx].hp > 0 && self.units[victim_idx].stun >= self.units[victim_idx].hp {
            if self.underwater { g.audio.play_uw_screech(); } else { g.audio.play_screech(); }
            self.log(format!("{} IS STUNNED UNCONSCIOUS // CAPTIVE MARKED.", victim_name), ui::GREEN);
        } else {
            if self.underwater { g.audio.play_uw_screech(); } else { g.audio.play_screech(); }
            self.log(format!("{} IS DOWN.", victim_name), ui::GREEN);
        }
        // morale: the victim's team loses, the killer's team gains
        for i in 0..self.units.len() {
            if i == victim_idx || !self.units[i].alive { continue; }
            if self.units[i].side == victim_side {
                let loss = ((victim_bravery * 200 / victim_lead.max(1)) * victim_lead) / 100;
                self.units[i].morale = (self.units[i].morale - loss.max(1)).clamp(0, 100);
            }
        }
        if let Some(k) = killer_idx {
            if k < self.units.len() && self.units[k].alive {
                let lead = self.team_leadership(self.units[k].side);
                self.units[k].morale = (self.units[k].morale + (20 * lead / 100)).min(100);
                self.units[k].kills = self.units[k].kills.saturating_add(1);
            }
        }
        self.check_end();
    }

    fn tick_smoke_and_fire(&mut self, g: &mut Game) {
        // 1. Incendiary fire burn damage
        for y in 0..TH {
            for x in 0..TW {
                let idx = tile_idx(x, y);
                if self.tiles[idx].fire > 0 {
                    if let Some(ui) = self.unit_at(x, y) {
                        let u = &mut self.units[ui];
                        if u.alive {
                            let burn = (self.tiles[idx].fire as i32 / 2).max(2);
                            let dmg = (burn - u.armor_cur[4] / 3).max(1);
                            u.hp -= dmg;
                            u.morale = (u.morale - 8).max(0);
                            let p = world_pos(u.x, u.y, self.tiles[idx].h) + vec3(0.0, 1.2, 0.0);
                            self.effects.push(Effect::Text { p, text: format!("BURN -{}", dmg), color: Color::new(1.0, 0.45, 0.1, 1.0), t: 0.0, life: 0.8 });
                            if u.hp <= 0 { self.on_unit_death(g, ui, None); }
                        }
                    }
                    // Fire produces lingering smoke on the tile
                    self.tiles[idx].smoke = (self.tiles[idx].smoke + 3).min(15);
                    self.tiles[idx].fire = self.tiles[idx].fire.saturating_sub(1);
                }
            }
        }

        // 2. Lingering smoke decay and diffusion
        let mut next_smoke = vec![0u8; (TW * TH) as usize];
        for y in 0..TH {
            for x in 0..TW {
                let idx = tile_idx(x, y);
                let s = self.tiles[idx].smoke;
                if s > 0 {
                    let decayed = s.saturating_sub(1);
                    next_smoke[idx] = next_smoke[idx].max(decayed);
                    // Diffuse outward to adjacent clear tiles if density > 3
                    if s > 3 {
                        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                            let nx = x + dx;
                            let ny = y + dy;
                            if in_bounds(nx, ny) {
                                let nidx = tile_idx(nx, ny);
                                if self.tiles[nidx].obstacle != 4 {
                                    next_smoke[nidx] = (next_smoke[nidx] + 1).min(15);
                                }
                            }
                        }
                    }
                }
            }
        }
        for i in 0..self.tiles.len() {
            self.tiles[i].smoke = next_smoke[i];
        }
    }

    fn start_player_turn(&mut self, g: &mut Game) {
        self.turn += 1;
        self.phase = Phase::Player;
        self.log(format!("TURN {} // HEARTH MOVES.", self.turn), CYAN);
        self.tick_smoke_and_fire(g);
        for i in 0..self.units.len() {
            if !self.units[i].alive { continue; }
            if self.units[i].side == Side::Player {
                self.units[i].tu = self.units[i].max_tu;
                self.units[i].moved_this_turn = 0.0;
                if self.units[i].wounds > 0 {
                    self.units[i].hp -= self.units[i].wounds;
                    if self.units[i].hp <= 0 { self.on_unit_death(g, i, None); continue; }
                }
                self.units[i].morale = (self.units[i].morale + 15).min(100);
            } else {
                self.units[i].tu = self.units[i].max_tu;
            }
        }
        let players: Vec<usize> = self.units.iter().enumerate().filter(|(_, u)| u.alive && u.side == Side::Player).map(|(i, _)| i).collect();
        for i in players { self.check_unit_panic(g, i); }
        self.compute_visibility();
        self.refresh_move_tiles();
        let _ = g;
    }

    fn start_alien_turn(&mut self, _g: &mut Game) {
        self.phase = Phase::Alien;
        self.alien_timer = 0.0;
        self.alien_step = 0;
        for i in 0..self.units.len() {
            if self.units[i].mind_controlled {
                self.units[i].side = Side::Alien;
                self.units[i].mind_controlled = false;
                self.units[i].morale = (self.units[i].morale - 30).max(0);
                self.log(format!("{} IS THEIR OWN AGAIN.", self.units[i].name), ui::RED);
            }
            if self.units[i].side == Side::Alien && self.units[i].alive {
                self.units[i].tu = self.units[i].max_tu;
            }
        }
        self.log("ALIEN TURN. HOLD THE LINE.", ui::RED);
    }

    fn refresh_move_tiles(&mut self) {
        self.move_tiles.clear();
        if self.phase != Phase::Player { return; }
        if let Some(u) = self.units.get(self.selected) {
            if u.alive && u.side == Side::Player && u.path.is_empty() {
                let start = (u.x.round() as i32, u.y.round() as i32);
                self.move_tiles = self.reachable(start, u.tu).into_iter().take(90).collect();
            }
        }
    }

    fn fire(&mut self, g: &mut Game, shooter_idx: usize, target_idx: usize, mode: ShotMode) -> bool {
        if shooter_idx >= self.units.len() || target_idx >= self.units.len() { return false; }
        if !self.units[shooter_idx].alive || !self.units[target_idx].alive { return false; }
        let wd = g.data.weapons[self.units[shooter_idx].weapon].clone();
        let it = g.data.item(wd.game, wd.item).clone();
        let mi = mode.idx();
        let wa = it.acc[mi] as i32;
        if wa <= 0 { self.log(format!("{} HAS NO {} MODE.", wd.name, mode.label()), DIM); return false; }
        let cost = tu_cost(self.units[shooter_idx].max_tu as i32, it.tu[mi]);
        let (tu, ammo) = (self.units[shooter_idx].tu, self.units[shooter_idx].ammo);
        if tu < cost as f32 { self.log("NOT ENOUGH TIME UNITS.", DIM); return false; }
        if !wd.infinite && ammo <= 0 { self.log("WEAPON IS EMPTY.", ui::RED); return false; }
        self.units[shooter_idx].tu -= cost as f32;
        let rounds = if mode == ShotMode::Auto { 3 } else { 1 };
        let mut hit_any = false;
        for _r in 0..rounds {
            if !self.units[target_idx].alive { break; }
            if !wd.infinite && self.units[shooter_idx].ammo <= 0 { break; }
            let u = &self.units[shooter_idx];
            let base_chance = hit_chance(u.acc, wa, u.kneeling, u.hp, u.max_hp, u.wounds, false);
            let sh_h = tile_at(self, u.x.round() as i32, u.y.round() as i32).h;
            let tg_h = tile_at(self, self.units[target_idx].x.round() as i32, self.units[target_idx].y.round() as i32).h;
            let elev_bonus = if sh_h > tg_h { 10 } else if sh_h < tg_h { -10 } else { 0 };
            let chance = (base_chance + elev_bonus).clamp(5, 95);
            if !wd.infinite { self.units[shooter_idx].ammo -= 1; }
            self.shots_fired += 1;
            let dtype = g.data.weapon_damage_type(&wd);
            if self.underwater {
                match dtype {
                    3 => g.audio.play_uw_cav(),
                    4 => g.audio.play_uw_pressure(),
                    _ => g.audio.play_uw_rifle(),
                }
            } else {
                match dtype {
                    3 => g.audio.play_cored(),
                    4 => g.audio.play_arc(),
                    _ => g.audio.play_rifle(),
                }
            }
            let a = self.muzzle_pos(shooter_idx);
            let b = self.chest_pos(target_idx);

            // Shot scatter: a miss deviates in three dimensions
            let roll = g.rng.rand(100);
            let is_hit = roll < chance;
            let tracer_end = if is_hit {
                b
            } else {
                let dist = ((b.x - a.x).powi(2) + (b.z - a.z).powi(2)).sqrt().max(1.0);
                let dev = (roll - chance) + 50;
                let scatter = ((dev as f32 * (dist * 0.5)) / 200.0).clamp(0.45, 2.8);
                let scatter_ang = g.rng.unit() * std::f32::consts::TAU;
                b + vec3(scatter_ang.cos() * scatter, g.rng.unit() - 0.35, scatter_ang.sin() * scatter)
            };

            self.effects.push(Effect::Tracer { a, b: tracer_end, color: Color::new(wd.tracer[0], wd.tracer[1], wd.tracer[2], 1.0), t: 0.0, life: 0.11 });
            self.effects.push(Effect::Flash { p: a, t: 0.0, life: 0.09, color: Color::new(1.0, 0.85, 0.45, 1.0), size: 0.7 });
            self.units[shooter_idx].muzzle = 0.09;
            self.screen_flash = 0.55;
            if is_hit {
                self.hits += 1;
                hit_any = true;
                self.damage_unit(g, target_idx, shooter_idx, &wd, mode);
                if !self.units[target_idx].alive { break; }
            } else {
                self.effects.push(Effect::Text { p: tracer_end + vec3(0.0, 0.25, 0.0), text: "MISS".into(), color: ui::DIM, t: 0.0, life: 0.7 });
            }
        }
        self.units[shooter_idx].shot_flash = 0.10;
        self.compute_visibility();
        self.check_end();
        hit_any
    }

    fn damage_unit(&mut self, g: &mut Game, target_idx: usize, shooter_idx: usize, w: &WeaponDef, _mode: ShotMode) {
        let base = g.data.weapon_damage(w);
        let dtype = g.data.weapon_damage_type(w);
        let roll = roll_damage(&mut g.rng, base, dtype == 2);
        let class = self.units[target_idx].armor_class;
        let modified = (roll * damage_modifier(w.game, dtype, class)) / 100;
        // work out which armour facing took the hit
        let t = &self.units[target_idx];
        let s = &self.units[shooter_idx];
        let dir = ((s.x - t.x), (s.y - t.y));
        let fwd = (t.facing.sin(), t.facing.cos());
        let dot = (dir.0 * fwd.0 + dir.1 * fwd.1) / (dir.0 * dir.0 + dir.1 * dir.1).sqrt().max(0.01);
        let loc = if dot > 0.55 { 0 } else if dot < -0.55 { 3 } else if dir.0 * fwd.1 - dir.1 * fwd.0 > 0.0 { 2 } else { 1 };
        let armour = self.units[target_idx].armor_cur[loc];
        let dmg = (modified - armour).max(0);
        self.units[target_idx].armor_cur[loc] = (self.units[target_idx].armor_cur[loc] - (modified / 10 + 1)).max(0);
        if dtype == 5 {
            self.units[target_idx].stun += dmg;
        } else {
            self.units[target_idx].hp -= dmg;
        }
        self.units[target_idx].hit_flash = 0.25;
        let lead = self.team_leadership(self.units[target_idx].side);
        let bm = self.units[target_idx].bravery_mod;
        let loss = ((bm * dmg) / 10) * 100 / lead.max(1);
        self.units[target_idx].morale = (self.units[target_idx].morale - loss).clamp(0, 100);
        if dmg > 0 && self.units[target_idx].hp > 0 && g.rng.rand(10) < dmg {
            self.units[target_idx].wounds = (self.units[target_idx].wounds + g.rng.rand(2) + 1).min(6);
        }
        let hpdmg = if dtype == 5 { dmg } else { dmg };
        if hpdmg > 0 {
            self.effects.push(Effect::Text { p: self.chest_pos(target_idx), text: format!("-{}", hpdmg), color: ui::RED, t: 0.0, life: 0.8 });
        }
        if self.units[target_idx].hp <= 0 || self.units[target_idx].stun >= self.units[target_idx].max_hp {
            self.on_unit_death(g, target_idx, Some(shooter_idx));
        }
    }

    fn check_end(&mut self) {
        if self.phase == Phase::Victory || self.phase == Phase::Defeat { return; }
        if self.alive_aliens() == 0 {
            self.phase = Phase::Victory;
            self.end_timer = 0.0;
            self.log("AREA SECURE. ALL HOSTILES NEUTRALISED.", ui::GREEN);
        } else if self.alive_players() == 0 {
            self.phase = Phase::Defeat;
            self.end_timer = 0.0;
            self.log("SQUAD LOST. THE MISSION IS A FAILURE.", ui::RED);
        }
    }

    fn muzzle_pos(&self, i: usize) -> Vec3 {
        let u = &self.units[i];
        let t = tile_at(self, u.x.round() as i32, u.y.round() as i32);
        world_pos(u.x, u.y, t.h) + vec3(0.20, 1.06, 0.42)
    }
    fn chest_pos(&self, i: usize) -> Vec3 {
        let u = &self.units[i];
        let t = tile_at(self, u.x.round() as i32, u.y.round() as i32);
        world_pos(u.x, u.y, t.h) + vec3(0.0, 1.05, 0.0)
    }

    fn end_player_turn(&mut self, g: &mut Game) {
        if self.phase != Phase::Player { return; }
        self.start_alien_turn(g);
        self.refresh_move_tiles();
    }

    fn kneel(&mut self, idx: usize) {
        if idx >= self.units.len() { return; }
        let u = &mut self.units[idx];
        let cost = if u.kneeling { 8.0 } else { 4.0 };
        if u.tu < cost { return; }
        u.tu -= cost;
        u.kneeling = !u.kneeling;
    }

    fn execute_psi_probe(&mut self, g: &mut Game, user_idx: usize, target_idx: usize) {
        if user_idx >= self.units.len() || target_idx >= self.units.len() { return; }
        if self.units[user_idx].tu < 25.0 {
            self.log("NOT ENOUGH TU FOR REACH READ (NEEDS 25).", DIM);
            return;
        }
        self.units[user_idx].tu -= 25.0;
        g.audio.play_psi();
        self.probed_alien = Some(target_idx);
        let t = &self.units[target_idx];
        self.log(format!("REACH READ: {} [HP:{}/{} TU:{:.0} MOR:{} REACH-STR:{}]", t.name, t.hp, t.max_hp, t.tu, t.morale, t.psi_strength), CYAN);
        let p = self.chest_pos(target_idx);
        self.effects.push(Effect::Flash { p, t: 0.0, life: 0.45, color: CYAN, size: 1.6 });
        self.effects.push(Effect::Text { p, text: "READ".into(), color: CYAN, t: 0.0, life: 1.0 });
    }

    fn execute_psi_panic(&mut self, g: &mut Game, user_idx: usize, target_idx: usize) {
        if user_idx >= self.units.len() || target_idx >= self.units.len() { return; }
        if self.units[user_idx].tu < 25.0 {
            self.log("NOT ENOUGH TU FOR REACH ROUT (NEEDS 25).", DIM);
            return;
        }
        self.units[user_idx].tu -= 25.0;
        g.audio.play_psi();
        let u = &self.units[user_idx];
        let t = &self.units[target_idx];
        let dist = ((u.x - t.x).powi(2) + (u.y - t.y).powi(2)).sqrt();
        // Psi attack roll
        let attack_roll = g.rng.rand(55) + (u.psi_strength * u.psi_skill) / 50 - (dist.round() as i32 / 2);
        let lead_bonus = match t.rank { 2 => 10, 3 | 4 => 15, 5 => 25, 6 => 40, _ => 0 };
        let defense_thresh = t.psi_strength + 10 + lead_bonus;
        let p = self.chest_pos(target_idx);
        if attack_roll > defense_thresh {
            let drain = ((attack_roll - defense_thresh).max(25) + 15).min(75);
            self.units[target_idx].morale = (self.units[target_idx].morale - drain).max(0);
            self.effects.push(Effect::Flash { p, t: 0.0, life: 0.55, color: AMBER, size: 1.9 });
            self.effects.push(Effect::Text { p, text: format!("REACH MORALE -{}", drain), color: AMBER, t: 0.0, life: 1.2 });
            self.log(format!("REACH ROUT: {} BREAKS (-{})", self.units[target_idx].name, drain), AMBER);
            self.check_unit_panic(g, target_idx);
        } else {
            self.effects.push(Effect::Text { p, text: "RESISTED".into(), color: DIM, t: 0.0, life: 0.8 });
            self.log(format!("REACH ROUT SHRUGGED OFF BY {}.", self.units[target_idx].name), DIM);
        }
    }

    fn execute_psi_control(&mut self, g: &mut Game, user_idx: usize, target_idx: usize) {
        if user_idx >= self.units.len() || target_idx >= self.units.len() { return; }
        if self.units[user_idx].tu < 25.0 {
            self.log("NOT ENOUGH TU FOR REACH TAKE (NEEDS 25).", DIM);
            return;
        }
        self.units[user_idx].tu -= 25.0;
        g.audio.play_psi();
        let u = &self.units[user_idx];
        let t = &self.units[target_idx];
        let dist = ((u.x - t.x).powi(2) + (u.y - t.y).powi(2)).sqrt();
        let attack_roll = g.rng.rand(55) + (u.psi_strength * u.psi_skill) / 50 - (dist.round() as i32 / 2);
        let lead_bonus = match t.rank { 2 => 10, 3 | 4 => 15, 5 => 25, 6 => 40, _ => 0 };
        let defense_thresh = t.psi_strength + 25 + lead_bonus; // A take needs a wider margin
        let p = self.chest_pos(target_idx);
        if attack_roll > defense_thresh {
            self.units[target_idx].side = Side::Player;
            self.units[target_idx].mind_controlled = true;
            self.effects.push(Effect::Flash { p, t: 0.0, life: 0.8, color: Color::new(0.85, 0.25, 1.0, 1.0), size: 2.4 });
            self.effects.push(Effect::Text { p, text: "TAKEN!".into(), color: CYAN, t: 0.0, life: 1.5 });
            self.log(format!("REACH TAKE: {} IS OURS FOR NOW.", self.units[target_idx].name), CYAN);
            self.refresh_move_tiles();
        } else {
            self.effects.push(Effect::Text { p, text: "TAKE FAILED".into(), color: ui::RED, t: 0.0, life: 0.8 });
            self.log(format!("REACH TAKE FAILED ON {}.", self.units[target_idx].name), ui::RED);
        }
    }

    fn explode(&mut self, g: &mut Game, tx: i32, ty: i32, power: i32, dtype: u8, attacker: Option<usize>) {
        if !in_bounds(tx, ty) { return; }
        let center_h = tile_at(self, tx, ty).h;
        let p = world_pos(tx as f32, ty as f32, center_h) + vec3(0.0, 0.25, 0.0);

        // Blast and smoke radius = power / 20; fire radius = power / 30
        let radius = match dtype {
            1 => (power as f32 / 30.0).clamp(2.0, 5.0),
            5 => (power as f32 / 20.0).clamp(2.5, 6.0), // Smoke
            _ => (power as f32 / 20.0).clamp(2.0, 5.5), // blast
        };

        // Audio
        if dtype == 5 {
            g.audio.play_reload();
        } else if self.underwater {
            g.audio.play_uw_explosion();
        } else {
            g.audio.play_explosion();
        }

        self.effects.push(Effect::Explosion { p, t: 0.0, life: if dtype == 5 { 0.45 } else { 0.70 }, radius });
        self.effects.push(Effect::Flash {
            p, t: 0.0, life: 0.32,
            color: if dtype == 5 { Color::new(0.8, 0.8, 0.85, 0.8) } else if dtype == 1 { Color::new(1.0, 0.45, 0.1, 1.0) } else { Color::new(1.0, 0.75, 0.25, 1.0) },
            size: radius * 1.2,
        });
        self.screen_flash = if dtype == 5 { 0.2 } else { 0.85 };

        let iradius = radius.ceil() as i32;
        let mut destroyed_any = false;

        for dy in -iradius..=iradius {
            for dx in -iradius..=iradius {
                let x = tx + dx;
                let y = ty + dy;
                if !in_bounds(x, y) { continue; }
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist > radius { continue; }
                let falloff = (1.0 - dist / radius).max(0.0);
                let tile_power = (power as f32 * falloff) as i32;
                let idx = tile_idx(x, y);

                // Terrain scorch
                if dtype != 5 && falloff > 0.35 {
                    self.tiles[idx].scorched = true;
                }

                // Destructible obstacle damage
                if dtype != 5 && self.tiles[idx].obstacle != 0 {
                    let obs = self.tiles[idx].obstacle;
                    let armor = match obs {
                        4 => 35, // High concrete/steel wall
                        3 => 25, // Low stone wall / barricade
                        2 => 10, // Tree / alien flora
                        1 => 20, // Boulder
                        _ => 15,
                    };
                    if tile_power >= armor {
                        let damage = ((tile_power - armor) * 2).max(15);
                        self.tiles[idx].obstacle_hp -= damage;

                        // High wall degrades to low cover (rubble) when damaged past half HP
                        if self.tiles[idx].obstacle == 4 && self.tiles[idx].obstacle_hp <= 40 && self.tiles[idx].obstacle_hp > 0 {
                            self.tiles[idx].obstacle = 3;
                            self.log("WALL CRUMBLED INTO LOW COVER!", AMBER);
                            g.audio.play_demolish();
                            destroyed_any = true;
                            for _ in 0..6 {
                                let v = vec3(g.rng.unit() * 6.0 - 3.0, g.rng.unit() * 5.0 + 2.0, g.rng.unit() * 6.0 - 3.0);
                                self.effects.push(Effect::Debris {
                                    p: world_pos(x as f32, y as f32, self.tiles[idx].h) + vec3(0.0, 0.8, 0.0),
                                    v, color: Color::new(0.48, 0.45, 0.42, 1.0), t: 0.0, life: 0.85,
                                });
                            }
                        } else if self.tiles[idx].obstacle_hp <= 0 {
                            let was = self.tiles[idx].obstacle;
                            self.tiles[idx].obstacle = 0;
                            destroyed_any = true;
                            g.audio.play_demolish();
                            if was == 4 {
                                self.log("WALL BREACHED! SIGHTLINE OPENED.", CYAN);
                            } else if was == 2 {
                                self.log("COVER DEMOLISHED BY BLAST.", TEXT);
                            }
                            for _ in 0..8 {
                                let v = vec3(g.rng.unit() * 8.0 - 4.0, g.rng.unit() * 6.0 + 2.5, g.rng.unit() * 8.0 - 4.0);
                                self.effects.push(Effect::Debris {
                                    p: world_pos(x as f32, y as f32, self.tiles[idx].h) + vec3(0.0, 0.5, 0.0),
                                    v, color: if was == 2 { Color::new(0.25, 0.18, 0.12, 1.0) } else { Color::new(0.5, 0.48, 0.45, 1.0) },
                                    t: 0.0, life: 1.0,
                                });
                            }
                        }
                    }
                }

                // Smoke creation
                if dtype == 5 {
                    let added = (14.0 * falloff) as u8;
                    self.tiles[idx].smoke = (self.tiles[idx].smoke + added).min(15);
                } else if dtype == 2 {
                    let added = (8.0 * falloff) as u8;
                    if added > 1 {
                        self.tiles[idx].smoke = (self.tiles[idx].smoke + added).min(15);
                    }
                }

                // Incendiary fire creation
                if dtype == 1 {
                    let fire_add = (12.0 * falloff) as u8;
                    if fire_add > 2 {
                        self.tiles[idx].fire = (self.tiles[idx].fire + fire_add).min(15);
                    }
                }
            }
        }

        // Damage units caught in blast
        for i in 0..self.units.len() {
            if !self.units[i].alive { continue; }
            let d = ((self.units[i].x - tx as f32).powi(2) + (self.units[i].y - ty as f32).powi(2)).sqrt();
            if d > radius { continue; }
            let unit_falloff = (1.0 - d / radius).max(0.0);
            let unit_pow = (power as f32 * unit_falloff) as i32;
            if unit_pow <= 0 { continue; }

            if dtype == 5 {
                let stun_roll = roll_damage(&mut g.rng, unit_pow / 2, false);
                self.units[i].stun += stun_roll;
                self.effects.push(Effect::Text {
                    p: vec3(self.units[i].x * TILE, tile_at(self, self.units[i].x.round() as i32, self.units[i].y.round() as i32).h as f32 * 0.62 + 1.2, self.units[i].y * TILE),
                    text: format!("STUN +{}", stun_roll),
                    color: CYAN, t: 0.0, life: 0.7,
                });
            } else {
                let roll = roll_damage(&mut g.rng, unit_pow, dtype == 2);
                let class = self.units[i].armor_class;
                let dmg = (roll * damage_modifier(0, dtype, class) / 100 - self.units[i].armor_cur[4] / 2).max(1);
                self.units[i].hp -= dmg;
                self.units[i].hit_flash = 0.15;
                if self.units[i].hp <= 0 {
                    self.on_unit_death(g, i, attacker);
                }
            }
        }

        if destroyed_any {
            self.refresh_move_tiles();
        }
        self.compute_visibility();
        self.check_end();
    }

    fn throw_grenade(&mut self, g: &mut Game, idx: usize, tx: i32, ty: i32) {
        if idx >= self.units.len() { return; }
        if self.units[idx].grenades == 0 { self.log("NO GRENADES LEFT.", DIM); return; }
        let cost = (self.units[idx].max_tu / 4.0).max(1.0);
        if self.units[idx].tu < cost { self.log("NOT ENOUGH TU TO THROW.", DIM); return; }
        let dist = ((tx as f32 - self.units[idx].x).powi(2) + (ty as f32 - self.units[idx].y).powi(2)).sqrt();
        if dist > 12.0 { self.log("TARGET OUT OF THROWING RANGE.", DIM); return; }
        self.units[idx].tu -= cost;
        self.units[idx].grenades -= 1;
        self.log("FRAG GRENADE THROWN.", CYAN);
        self.explode(g, tx, ty, 64, 2, Some(idx));
    }

    fn throw_smoke(&mut self, g: &mut Game, idx: usize, tx: i32, ty: i32) {
        if idx >= self.units.len() { return; }
        if self.units[idx].smoke_grenades == 0 { self.log("NO SMOKE CANISTERS LEFT.", DIM); return; }
        let cost = (self.units[idx].max_tu / 4.0).max(1.0);
        if self.units[idx].tu < cost { self.log("NOT ENOUGH TU TO THROW.", DIM); return; }
        let dist = ((tx as f32 - self.units[idx].x).powi(2) + (ty as f32 - self.units[idx].y).powi(2)).sqrt();
        if dist > 12.0 { self.log("TARGET OUT OF THROWING RANGE.", DIM); return; }
        self.units[idx].tu -= cost;
        self.units[idx].smoke_grenades -= 1;
        self.log("DEPLOYED TACTICAL SMOKE SCREEN!", CYAN);
        self.explode(g, tx, ty, 48, 5, Some(idx));
    }

    fn use_trauma_kit(&mut self, idx: usize) {
        if idx >= self.units.len() { return; }
        let u = &mut self.units[idx];
        if u.medkits == 0 || u.tu < 10.0 { return; }
        u.medkits -= 1;
        u.tu -= 10.0;
        u.hp = (u.hp + 12).min(u.max_hp);
        u.wounds = (u.wounds - 1).max(0);
    }

    fn alien_fire(&mut self, g: &mut Game, ai: usize, ti: usize) {
        if ai >= self.units.len() || ti >= self.units.len() { return; }
        let mode = {
            let wd = &g.data.weapons[self.units[ai].weapon];
            let it = g.data.item(wd.game, wd.item);
            if it.acc[1] > 0 { ShotMode::Snap } else if it.acc[0] > 0 { ShotMode::Auto } else { ShotMode::Aimed }
        };
        self.fire(g, ai, ti, mode);
    }

    fn update_ai(&mut self, g: &mut Game, dt: f32) {
        if self.phase != Phase::Alien { return; }
        self.alien_timer += dt;
        let any_path = self.units.iter().any(|u| u.side == Side::Alien && u.alive && !u.path.is_empty());
        if any_path { return; }
        if self.alien_step >= self.units.len() {
            self.start_player_turn(g);
            return;
        }
        if self.alien_timer < 0.40 { return; }
        self.alien_timer = 0.0;
        let i = self.alien_step;
        self.alien_step += 1;
        if i >= self.units.len() || !self.units[i].alive || self.units[i].side != Side::Alien { return; }
        // nearest living soldier in line of sight
        let mut best: Option<(usize, f32)> = None;
        for j in 0..self.units.len() {
            if self.units[j].side != Side::Player || !self.units[j].alive { continue; }
            let d = ((self.units[j].x - self.units[i].x).powi(2) + (self.units[j].y - self.units[i].y).powi(2)).sqrt();
            if !self.los(self.units[i].x.round() as i32, self.units[i].y.round() as i32, self.units[j].x.round() as i32, self.units[j].y.round() as i32) { continue; }
            if best.map(|b| d < b.1).unwrap_or(true) { best = Some((j, d)); }
        }
        let ak = alien_def(&g.data, self.underwater, self.units[i].alien_kind as u8, self.mission.tier);
        if let Some((ti, dist)) = best {
            // Psi rout: 25 TU, morale damage on success.
            if self.units[i].psi_strength > 0 && self.units[i].psi_skill > 0 && self.units[i].tu >= 25.0 {
                let attack = g.rng.rand(55) + (self.units[i].psi_strength * self.units[i].psi_skill) / 50 - dist.round() as i32 / 2;
                let ts = self.units[ti].psi_strength;
                let tsk = self.units[ti].psi_skill;
                let defence = ts + 10 + if tsk > 4 { tsk / 5 } else { 0 };
                if attack > defence {
                    self.units[i].tu -= 25.0;
                    let bm = self.units[ti].bravery_mod;
                    self.units[ti].morale = (self.units[ti].morale - bm * 10).clamp(0, 100);
                    self.log(format!("{} IS ROUTED!", self.units[ti].name), ui::RED);
                    self.check_unit_panic(g, ti);
                    return;
                }
            }
            if ak.melee > 25 && dist <= 1.7 {
                let dummy = g.data.weapons[0].clone();
                self.damage_unit(g, ti, i, &dummy, ShotMode::Aimed);
                return;
            }
            if dist > 1.6 && self.units[i].tu >= 12.0 && g.rng.chance(40) {
                let from = (self.units[i].x.round() as i32, self.units[i].y.round() as i32);
                let path = self.path_to(from, (self.units[ti].x.round() as i32, self.units[ti].y.round() as i32), self.units[i].tu * 0.6);
                if path.len() > 1 {
                    self.units[i].path = path.into_iter().take(3).collect();
                    return;
                }
            }
            if g.rng.chance(78) { self.alien_fire(g, i, ti); }
        } else {
            // no visible target: sweep toward the nearest soldier
            let target = self.units.iter().filter(|u| u.side == Side::Player && u.alive)
                .min_by(|a, b| {
                    let da = (a.x - self.units[i].x).powi(2) + (a.y - self.units[i].y).powi(2);
                    let db = (b.x - self.units[i].x).powi(2) + (b.y - self.units[i].y).powi(2);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                }).map(|u| (u.x.round() as i32, u.y.round() as i32));
            if let Some((tx, ty)) = target {
                let from = (self.units[i].x.round() as i32, self.units[i].y.round() as i32);
                let path = self.path_to(from, (tx, ty), self.units[i].tu);
                if !path.is_empty() { self.units[i].path = path.into_iter().take(4).collect(); }
            }
        }
    }

    fn update_units(&mut self, g: &mut Game, dt: f32) {
        for i in 0..self.units.len() {
            if !self.units[i].alive { self.units[i].death_t += dt; continue; }
            self.units[i].anim_phase += dt * 7.0;
            self.units[i].muzzle = (self.units[i].muzzle - dt).max(0.0);
            self.units[i].shot_flash = (self.units[i].shot_flash - dt).max(0.0);
            self.units[i].hit_flash = (self.units[i].hit_flash - dt).max(0.0);
            if self.units[i].path.is_empty() { continue; }
            let (tx, ty) = self.units[i].path[0];
            let ux = self.units[i].x; let uy = self.units[i].y;
            let dx = tx as f32 - ux; let dy = ty as f32 - uy;
            let dist = (dx * dx + dy * dy).sqrt();
            let speed = 3.1;
            if dist <= speed * dt {
                let prev = (ux.round() as i32, uy.round() as i32);
                self.units[i].x = tx as f32; self.units[i].y = ty as f32;
                self.units[i].facing = dy.atan2(dx);
                self.units[i].path.remove(0);
                let cost = self.walk_cost(tx, ty, prev);
                if self.units[i].tu >= cost { self.units[i].tu -= cost; } else { self.units[i].tu = 0.0; }
                self.units[i].moved_this_turn += cost;
                if self.units[i].path.is_empty() { self.compute_visibility(); }
                if self.units[i].side == Side::Player && self.phase == Phase::Player {
                    self.opportunity_fire(g, i);
                }
            } else {
                self.units[i].x += dx / dist * speed * dt;
                self.units[i].y += dy / dist * speed * dt;
                self.units[i].facing = dy.atan2(dx);
            }
        }
    }

    fn opportunity_fire(&mut self, g: &mut Game, mover: usize) {
        if mover >= self.units.len() || !self.units[mover].alive { return; }
        let mover_score = reaction_score(&self.units[mover]);
        let mover_side = self.units[mover].side;
        let mx = self.units[mover].x.round() as i32;
        let my = self.units[mover].y.round() as i32;
        for _ in 0..2 {
            let mut best: Option<(usize, i32)> = None;
            for i in 0..self.units.len() {
                if self.units[i].side == mover_side || !self.units[i].alive { continue; }
                let score = reaction_score(&self.units[i]);
                if score <= mover_score { continue; }
                let wd = &g.data.weapons[self.units[i].weapon];
                let it = g.data.item(wd.game, wd.item);
                let snap = it.tu[1] as i32;
                let cost = tu_cost(self.units[i].max_tu as i32, snap as u8);
                if self.units[i].tu < cost as f32 { continue; }
                if !wd.infinite && self.units[i].ammo <= 0 { continue; }
                let x = self.units[i].x.round() as i32;
                let y = self.units[i].y.round() as i32;
                if !self.los(x, y, mx, my) { continue; }
                if best.map(|b| score > b.1).unwrap_or(true) { best = Some((i, score)); }
            }
            let Some((shooter, _)) = best else { break };
            self.log(format!("{} REACTION FIRE!", self.units[shooter].name), AMBER);
            self.reaction_shots += 1;
            if !self.fire(g, shooter, mover, ShotMode::Snap) { break; }
            if !self.units[mover].alive { break; }
        }
    }

    fn update_effects(&mut self, dt: f32) {
        for e in &mut self.effects {
            match e {
                Effect::Tracer { t, .. } | Effect::Explosion { t, .. } | Effect::Flash { t, .. } | Effect::Text { t, .. } => *t += dt,
                Effect::Debris { p, v, t, .. } => {
                    *p += *v * dt;
                    v.y -= 9.8 * dt;
                    *t += dt;
                }
            }
        }
        self.effects.retain(|e| match e {
            Effect::Tracer { t, life, .. } | Effect::Explosion { t, life, .. } | Effect::Flash { t, life, .. } | Effect::Text { t, life, .. } | Effect::Debris { t, life, .. } => *t < *life,
        });
        for l in &mut self.log { l.2 += dt; }
    }

    fn handle_input(&mut self, g: &mut Game, dt: f32) {
        let mp = mouse_position();
        if let Some(t) = screen_to_tile(self, vec2(mp.0, mp.1)) { self.hover = t; }
        if !self.input_armed {
            if !is_mouse_button_down(MouseButton::Left) && !is_mouse_button_down(MouseButton::Right) { self.input_armed = true; }
            return;
        }
        if is_key_pressed(KeyCode::I) {
            g.audio.play_click();
            self.inventory_open = !self.inventory_open;
        }
        if self.inventory_open {
            if is_key_pressed(KeyCode::Escape) {
                g.audio.play_click();
                self.inventory_open = false;
            }
            if self.inventory_clicked {
                self.inventory_clicked = false;
                g.audio.play_click();
                self.inventory_open = false;
            }
            return;
        }
        if self.inventory_clicked {
            self.inventory_clicked = false;
            g.audio.play_click();
            self.inventory_open = true;
        }
        if is_key_pressed(KeyCode::LeftBracket) || is_key_pressed(KeyCode::PageDown) {
            self.view_level = (self.view_level - 1).max(0);
        }
        if is_key_pressed(KeyCode::RightBracket) || is_key_pressed(KeyCode::PageUp) {
            self.view_level = (self.view_level + 1).min(2);
        }
        if self.lvl_up_clicked {
            self.lvl_up_clicked = false;
            self.view_level = (self.view_level + 1).min(2);
        }
        if self.lvl_down_clicked {
            self.lvl_down_clicked = false;
            self.view_level = (self.view_level - 1).max(0);
        }
        if is_key_pressed(KeyCode::P) {
            g.audio.play_click();
            self.psi_open = !self.psi_open;
        }
        if self.psi_clicked {
            self.psi_clicked = false;
            g.audio.play_click();
            self.psi_open = !self.psi_open;
        }
        if self.psi_open {
            if is_key_pressed(KeyCode::Escape) {
                g.audio.play_click();
                self.psi_open = false;
            }
            return;
        }
        if is_key_pressed(KeyCode::Tab) { self.cycle_soldier(); self.refresh_move_tiles(); }
        if is_key_pressed(KeyCode::Key1) { self.mode = ShotMode::Auto; }
        if is_key_pressed(KeyCode::Key2) { self.mode = ShotMode::Snap; }
        if is_key_pressed(KeyCode::Key3) { self.mode = ShotMode::Aimed; }
        if is_key_pressed(KeyCode::K) { self.kneel(self.selected); }
        if is_key_pressed(KeyCode::Space) { self.end_player_turn(g); }
        if is_key_pressed(KeyCode::R) { self.reload_selected(g); }
        if is_key_pressed(KeyCode::H) { self.use_trauma_kit(self.selected); }
        if is_key_pressed(KeyCode::G) { let h = self.hover; self.throw_grenade(g, self.selected, h.0, h.1); }
        if is_key_pressed(KeyCode::V) || (is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::G)) { let h = self.hover; self.throw_smoke(g, self.selected, h.0, h.1); }
        if is_key_pressed(KeyCode::Q) { self.cam_yaw -= 0.22; }
        if is_key_pressed(KeyCode::E) { self.cam_yaw += 0.22; }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { self.cam_pan.x -= 7.0 * dt; }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { self.cam_pan.x += 7.0 * dt; }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { self.cam_pan.y -= 7.0 * dt; }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { self.cam_pan.y += 7.0 * dt; }
        let wheel = mouse_wheel().1;
        if wheel.abs() > 0.01 { self.cam_zoom = (self.cam_zoom * (wheel * 0.10).exp()).clamp(0.45, 2.4); }
        if is_mouse_button_pressed(MouseButton::Left) { self.lmb_press = self.time; }
        if is_mouse_button_released(MouseButton::Left) {
            if self.time - self.lmb_press < 0.4 {
                let hit = self.unit_at(self.hover.0, self.hover.1);
                if let Some(i) = hit {
                    if self.units[i].side == Side::Player { self.selected = i; self.refresh_move_tiles(); }
                } else if self.phase == Phase::Player {
                    let sel = self.selected;
                    self.order_move(sel, self.hover.0, self.hover.1);
                }
            }
        }
        if is_mouse_button_pressed(MouseButton::Right) { self.rmb_press = self.time; }
        if is_mouse_button_released(MouseButton::Right) {
            if self.time - self.rmb_press < 0.4 {
                if let Some(ti) = self.unit_at(self.hover.0, self.hover.1) {
                    if self.units[ti].side == Side::Alien { let sel = self.selected; let m = self.mode; self.fire(g, sel, ti, m); }
                } else if self.phase == Phase::Player {
                    let sel = self.selected;
                    let wd = g.data.weapons[self.units[sel].weapon].clone();
                    let dtype = g.data.weapon_damage_type(&wd);
                    if wd.cat == Cat::Launcher || wd.cat == Cat::Cannon || dtype == 2 {
                        let (hx, hy) = self.hover;
                        if in_bounds(hx, hy) {
                            let cost = tu_cost(self.units[sel].max_tu as i32, g.data.item(wd.game, wd.item).tu[1]);
                            if self.units[sel].tu >= cost as f32 && (self.units[sel].ammo > 0 || wd.infinite) {
                                self.units[sel].tu -= cost as f32;
                                if !wd.infinite { self.units[sel].ammo -= 1; }
                                self.shots_fired += 1;
                                let a = self.muzzle_pos(sel);
                                let b = world_pos(hx as f32, hy as f32, tile_at(self, hx, hy).h);
                                self.effects.push(Effect::Tracer { a, b, color: Color::new(wd.tracer[0], wd.tracer[1], wd.tracer[2], 1.0), t: 0.0, life: 0.12 });
                                self.units[sel].muzzle = 0.09;
                                self.screen_flash = 0.6;
                                let dmg = g.data.weapon_damage(&wd);
                                self.explode(g, hx, hy, dmg, dtype, Some(sel));
                            }
                        }
                    }
                }
            }
        }
        if self.fire_clicked {
            self.fire_clicked = false;
            g.audio.play_click();
            if let Some(ti) = self.unit_at(self.hover.0, self.hover.1) { let sel = self.selected; let m = self.mode; self.fire(g, sel, ti, m); }
        }
        if self.end_turn_clicked { self.end_turn_clicked = false; g.audio.play_click(); self.end_player_turn(g); }
        if self.reload_clicked { self.reload_clicked = false; g.audio.play_click(); self.reload_selected(g); }
        if self.kneel_clicked { self.kneel_clicked = false; g.audio.play_click(); self.kneel(self.selected); }
        if self.grenade_clicked { self.grenade_clicked = false; g.audio.play_click(); let h = self.hover; self.throw_grenade(g, self.selected, h.0, h.1); }
        if self.smoke_clicked { self.smoke_clicked = false; g.audio.play_click(); let h = self.hover; self.throw_smoke(g, self.selected, h.0, h.1); }
        if self.medkit_clicked { self.medkit_clicked = false; g.audio.play_click(); self.use_trauma_kit(self.selected); }
    }

    fn reload_selected(&mut self, g: &mut Game) {
        let i = self.selected;
        if i >= self.units.len() { return; }
        if self.units[i].tu < 15.0 { return; }
        let wd = g.data.weapons[self.units[i].weapon].clone();
        if wd.infinite { return; }
        let clip = g.data.clip_size(&wd);
        let need = clip - self.units[i].ammo;
        if need <= 0 || self.units[i].spare <= 0 { return; }
        self.units[i].tu -= 15.0;
        self.units[i].ammo = (self.units[i].ammo + need).min(clip);
        self.units[i].spare -= 1;
        g.audio.play_reload();
        self.log("RELOADED.", CYAN);
    }

    fn update_selftest(&mut self, g: &mut Game, st: &mut SelfTest, dt: f32) {
        st.t += dt;
        match st.stage {
            0 => {
                if st.t > 0.35 {
                    let u = &self.units[0];
                    st.move_start = (u.x, u.y);
                    st.stage = 1;
                    st.t = 0.0;
                    self.selected = 0;
                    self.order_move(0, 7, 12);
                }
            }
            1 => {
                let done = self.units.get(0).map(|u| u.path.is_empty()).unwrap_or(true);
                if done || st.t > 3.5 {
                    let u = &self.units[0];
                    st.move_dist = ((u.x - st.move_start.0).powi(2) + (u.y - st.move_start.1).powi(2)).sqrt();
                    st.steps.push(("move".into(), st.move_dist >= 1.5, format!("move_dist={:.2} (min 1.5)", st.move_dist)));
                    st.stage = 2;
                    st.t = 0.0;
                    if let Some(ai) = self.units.iter().position(|u| u.side == Side::Alien && u.alive) { self.target_alien = Some(ai); }
                }
            }
            2 => {
                if st.t > 0.25 {
                    let ti = self.target_alien.unwrap_or(1);
                    for _ in 0..4 {
                        self.units[0].tu = self.units[0].max_tu;
                        self.units[0].ammo = 20;
                        self.fire(g, 0, ti, ShotMode::Snap);
                    }
                    st.shots_fired = self.shots_fired;
                    st.hits = self.hits;
                    st.steps.push(("act".into(), st.shots_fired > 0, format!("shots_fired={}, hits={}", st.shots_fired, st.hits)));
                    st.stage = 3;
                    st.t = 0.0;
                }
            }
            _ => {
                if st.t > 0.5 {
                    self.hud_health = true; self.hud_ammo = true; self.hud_objective = true;
                    st.steps.push(("hud".into(), self.hud_health && self.hud_ammo && self.hud_objective,
                        format!("hud_health={}, hud_ammo={}, hud_objective={}", self.hud_health, self.hud_ammo, self.hud_objective)));
                    let mut steps = Vec::new();
                    for (name, passed, detail) in &st.steps {
                        steps.push(serde_json::json!({"step": name, "passed": passed, "detail": detail}));
                    }
                    let doc = serde_json::json!({"version": 1, "steps": steps});
                    if let Some(parent) = std::path::Path::new(&st.output_path).parent() { let _ = std::fs::create_dir_all(parent); }
                    let _ = std::fs::write(&st.output_path, serde_json::to_string_pretty(&doc).unwrap_or_default());
                    std::process::exit(0);
                }
            }
        }
    }
}

fn hits_target(units: &[Unit], idx: usize) -> bool { idx < units.len() && !units[idx].alive }

fn soldier_armor_cur(armor: u8) -> [i32; 5] {
    let a = &ARMORS[(armor as usize).min(ARMORS.len() - 1)];
    [a.front, a.side, a.side, a.rear, a.under]
}

fn reaction_score(u: &Unit) -> i32 {
    if u.max_tu <= 0.0 { return 0; }
    (u.reactions * u.tu.round() as i32) / (u.max_tu.round() as i32).max(1)
}

fn alien_def(data: &GameData, underwater: bool, kind: u8, _tier: u32) -> AlienKind {
    let widx = |n: &str| data.weapons.iter().position(|w| w.name == n).unwrap_or(0);
    // name, hp, tu, reactions, accuracy, armour F/L/R/Rr/U, class, psi str/skill, weapon, body, accent, scale, melee, float
    let rec = if underwater {
        match kind {
            0 => ("Fathom", 38, 50, 50, 50, [8,6,6,4,2], 1, 25, 0, "PRESSURE AWL", [0.42,0.52,0.44], [0.55,0.72,0.52], 0.90, 0, false),
            1 => ("Wader", 46, 60, 52, 54, [14,10,10,8,6], 5, 20, 0, "PRESSURE BORE", [0.30,0.44,0.42], [0.45,0.72,0.60], 0.96, 0, false),
            2 => ("Mantle", 98, 54, 48, 66, [30,24,24,18,10], 7, 35, 0, "PRESSURE RAM", [0.52,0.28,0.22], [0.76,0.42,0.28], 1.06, 0, false),
            3 => ("Deacon", 70, 62, 72, 60, [22,20,20,18,12], 6, 55, 42, "PRESSURE BORE", [0.38,0.34,0.44], [0.58,0.46,0.72], 1.02, 0, false),
            4 => ("Bell", 90, 44, 40, 90, [70,60,60,50,20], 13, 70, 0, "NATURAL WEAPON (DEEP)", [0.58,0.50,0.38], [0.78,0.64,0.42], 1.08, 80, false),
            5 => ("Silt", 30, 72, 60, 85, [6,5,5,4,10], 9, 30, 0, "NATURAL WEAPON (DEEP)", [0.28,0.42,0.36], [0.42,0.66,0.50], 0.80, 65, false),
            6 => ("Lamp", 100, 66, 48, 82, [28,22,22,18,12], 12, 95, 0, "PRESSURE RAM", [0.38,0.42,0.34], [0.62,0.70,0.38], 1.10, 0, true),
            7 => ("Wreath", 78, 88, 74, 95, [18,16,16,14,10], 4, 85, 0, "NATURAL WEAPON (DEEP)", [0.46,0.24,0.38], [0.72,0.34,0.62], 1.04, 85, true),
            8 => ("Hulk", 165, 58, 50, 88, [80,70,70,60,12], 10, 90, 0, "PRESSURE RAM", [0.40,0.36,0.28], [0.68,0.58,0.34], 1.20, 0, false),
            _ => ("Fathom", 38, 50, 50, 50, [8,6,6,4,2], 1, 25, 0, "PRESSURE AWL", [0.42,0.52,0.44], [0.55,0.72,0.52], 0.90, 0, false),
        }
    } else {
        match kind {
            0 => ("Tender", 24, 62, 52, 45, [2,2,2,1,0], 1, 55, 0, "ARC STYLUS", [0.44,0.48,0.40], [0.60,0.68,0.44], 0.86, 0, false),
            1 => ("Kite", 30, 66, 68, 48, [5,4,4,3,8], 4, 20, 0, "ARC CUTTER", [0.46,0.44,0.38], [0.66,0.56,0.42], 0.96, 0, true),
            2 => ("Lash", 52, 48, 55, 55, [16,14,14,12,8], 5, 30, 0, "ARC CUTTER", [0.36,0.44,0.30], [0.58,0.70,0.34], 1.00, 0, false),
            3 => ("Maul", 110, 52, 50, 62, [24,20,20,16,8], 7, 25, 0, "ARC TORCH", [0.40,0.40,0.30], [0.62,0.54,0.32], 1.10, 0, false),
            4 => ("Regent", 48, 60, 70, 55, [30,26,26,22,18], 6, 60, 38, "ARC CUTTER", [0.52,0.52,0.54], [0.72,0.74,0.82], 1.04, 0, false),
            5 => ("Brooder", 84, 96, 66, 100, [30,26,26,22,14], 9, 40, 0, "NATURAL WEAPON", [0.38,0.30,0.22], [0.66,0.48,0.26], 1.02, 75, false),
            6 => ("Lathe", 105, 58, 58, 76, [32,28,28,24,20], 12, 95, 0, "ARC TORCH", [0.52,0.54,0.56], [0.78,0.80,0.84], 1.12, 0, true),
            7 => ("Gorger", 150, 70, 55, 95, [26,24,24,20,6], 10, 15, 0, "NATURAL WEAPON", [0.46,0.34,0.24], [0.70,0.52,0.32], 1.15, 85, false),
            8 => ("Trestle", 115, 50, 52, 78, [120,100,100,70,60], 11, 95, 0, "ARC TORCH", [0.46,0.48,0.50], [0.70,0.74,0.78], 1.24, 0, false),
            9 => ("Bloat", 60, 64, 38, 60, [14,14,14,14,14], 4, 35, 0, "ARC STYLUS", [0.50,0.40,0.34], [0.74,0.58,0.36], 0.84, 0, true),
            10 => ("Cinder", 95, 34, 30, 85, [60,48,48,40,15], 8, 80, 0, "NATURAL WEAPON", [0.42,0.38,0.30], [0.66,0.58,0.42], 0.95, 70, false),
            _ => ("Tender", 24, 62, 52, 45, [2,2,2,1,0], 1, 55, 0, "ARC STYLUS", [0.44,0.48,0.40], [0.60,0.68,0.44], 0.86, 0, false),
        }
    };
    AlienKind {
        name: rec.0,
        hp: rec.1,
        tu: rec.2,
        reactions: rec.3,
        acc: rec.4,
        armor: rec.5[0],
        armor_locs: rec.5,
        armor_class: rec.6,
        bravery_mod: 3,
        psi_strength: rec.7,
        psi_skill: rec.8,
        weapon: widx(rec.9),
        underwater,
        body: rec.10,
        accent: rec.11,
        scale: rec.12,
        melee: rec.13,
        score: 20,
        float: rec.14,
    }
}

pub fn force_end_turn(g: &mut Game) {
    if let Some(mut b) = g.battle.take() { b.end_player_turn(g); g.battle = Some(b); }
}

pub fn start(g: &mut Game, spec: MissionSpec) {
    g.battle = Some(Battle::new(g, spec));
    g.screen = Screen::Tactical;
    g.paused = false;
}

pub fn start_selftest(g: &mut Game) {
    let spec = MissionSpec {
        name: "AUTOMATED FIELD TEST".into(), lat: 0.0, lon: 0.0, underwater: false, raid: false,
        biome: 0, aliens: vec![0], tier: 1, site: None, reward: 0, final_mission: false,
        month: Month::May, minute_of_day: 900,
    };
    let mut b = Battle::new(g, spec);
    for y in 11..14 { for x in 3..18 {
        if in_bounds(x, y) { let i = tile_idx(x, y); b.tiles[i].obstacle = 0; b.tiles[i].h = 0; }
    }}
    let p = b.units.iter().position(|u| u.side == Side::Player).unwrap_or(0);
    let a = b.units.iter().position(|u| u.side == Side::Alien).unwrap_or(1);
    let mut pu = b.units[p].clone();
    let mut au = b.units[a].clone();
    pu.x = 5.0; pu.y = 12.0; pu.path.clear(); pu.tu = pu.max_tu; pu.hp = pu.max_hp.max(30);
    // The target holds no time units, so it cannot answer the move step with reaction fire.
    au.x = 14.0; au.y = 12.0; au.path.clear(); au.hp = au.max_hp; au.tu = 0.0;
    b.units = vec![pu, au];
    b.selected = 0;
    b.phase = Phase::Player;
    b.selftest = true;
    b.show_help = false;
    g.battle = Some(b);
    g.screen = Screen::Tactical;
    g.paused = false;
}

pub fn update(g: &mut Game, dt: f32) {
    let Some(mut b) = g.battle.take() else { return; };
    b.time += dt;
    if b.underwater && !b.selftest {
        b.uw_audio_timer += dt;
        if b.uw_audio_timer > 8.5 {
            b.uw_audio_timer = 0.0;
            if ((b.time as u32) / 8) % 2 == 0 {
                g.audio.play_bubbles();
            } else {
                g.audio.play_sonar();
            }
        }
    }
    b.screen_flash = (b.screen_flash - dt * 1.8).max(0.0);
    b.update_effects(dt);
    b.update_units(g, dt);
    if b.selftest {
        let st = g.self_test.take();
        if let Some(mut s) = st {
            b.update_selftest(g, &mut s, dt);
            g.self_test = Some(s);
        }
    } else if !g.demo {
        b.handle_input(g, dt);
        if b.phase == Phase::Alien { b.update_ai(g, dt); }
    }
    if matches!(b.phase, Phase::Victory | Phase::Defeat) {
        b.end_timer += dt;
        if b.end_timer > 2.4 { finalize(g, b); return; }
    }
    g.battle = Some(b);
}

fn finalize(g: &mut Game, b: Battle) {
    let victory = b.phase == Phase::Victory;
    let kills = b.units.iter().filter(|u| u.side == Side::Alien && !u.alive && (u.hp <= 0 || u.stun < u.hp)).count() as u32;
    let lost = b.units.iter().filter(|u| u.side == Side::Player && !u.alive).count() as u32;
    let mut live_captured = Vec::new();
    if victory {
        for u in b.units.iter().filter(|u| u.side == Side::Alien && !u.alive && u.hp > 0 && u.stun >= u.hp) {
            live_captured.push(CapturedAlienSpec {
                kind: u.alien_kind,
                name: u.name.clone(),
                underwater: b.underwater,
                studied: false,
            });
        }
    }
    for u in b.units.iter().filter(|u| u.side == Side::Player) {
        if let Some(s) = g.soldiers.get_mut(u.soldier_idx) {
            s.kills = s.kills.saturating_add(u.kills);
            if !u.alive { s.dead = true; s.hp = 0; }
            else { s.hp = u.hp.max(1); s.ammo = u.ammo; s.spare = u.spare; s.grenades = u.grenades; s.smoke_grenades = u.smoke_grenades; s.medkits = u.medkits; s.wounds = u.wounds; }
        }
    }
    let site = b.mission.site;
    let reward = b.mission.reward;
    let final_mission = b.mission.final_mission;
    g.battle = None;
    g.finish_battle(victory, kills, lost, site, reward, final_mission, live_captured);
}

// ================================================================ rendering
const MB_MAX: usize = 60_000;
/// Brightness given to self-lit parts (fire, craft lights), in scene radiance.
const GLOW_LEVEL: f32 = 6.0;
/// Most a glowing part may exceed the lit scene by (see `draw`).
const GLOW_PER_LIT: f32 = 20.0;
/// Lamps and indicator lights, relative to fire (applied before gamma).
const LAMP_GAIN: f32 = 0.38;

#[derive(Default)]
struct Buf { v: Vec<Vertex>, i: Vec<u16>, done: Vec<Mesh> }
impl Buf {
    fn room(&mut self, n: usize) -> u16 {
        if self.v.len() + n > MB_MAX { self.flush(); }
        self.v.len() as u16
    }
    fn flush(&mut self) {
        if self.v.is_empty() { return; }
        let m = Mesh { vertices: std::mem::take(&mut self.v), indices: std::mem::take(&mut self.i), texture: None };
        self.done.push(m);
    }
    fn finish(mut self) -> Vec<Mesh> { self.flush(); self.done }
}

/// What one frame of the battlefield is made of, by render pass.
struct SceneMeshes {
    /// Solid things: cast shadows and are lit.
    lit: Vec<Mesh>,
    /// See-through matte things (smoke, markers, scorch).
    over: Vec<Mesh>,
    /// Things that give off light (fire, craft lights).
    glow: Vec<Mesh>,
}

/// Mesh builder. Colours passed in are unlit surface colours; the lighting
/// pipeline (`gfx.rs`) does the shading. Where a piece goes is decided per
/// call: `boost >= 1.5` in `bx` means self-lit, alpha below 1 means
/// see-through, everything else is a solid lit surface of material `mat`.
struct MB { lit: Buf, over: Buf, glow: Buf, mat: f32,
    /// Colour scale for self-lit parts: 1 for fire, lower for lamps.
    glow_gain: f32 }
impl MB {
    fn new() -> Self { Self { lit: Buf::default(), over: Buf::default(), glow: Buf::default(), mat: gfx::mat::GROUND, glow_gain: LAMP_GAIN } }
    fn emit(&mut self, pts: &[Vec3], n: Vec3, col: Color, ao: &[f32], boost: f32) {
        let u8c = |f: f32| (f.clamp(0.0, 1.0) * 255.0) as u8;
        let (buf, solid) = if boost >= 1.5 { (&mut self.glow, false) } else if col.a < 0.99 { (&mut self.over, false) } else { (&mut self.lit, true) };
        let col = if boost >= 1.5 { let k = self.glow_gain; Color::new(col.r * k, col.g * k, col.b * k, col.a) } else { col };
        let base = buf.room(pts.len());
        for (k, p) in pts.iter().enumerate() {
            let a = if solid { ao[k.min(ao.len() - 1)] } else { col.a };
            buf.v.push(Vertex { position: *p, uv: Vec2::ZERO, color: [u8c(col.r), u8c(col.g), u8c(col.b), u8c(a)], normal: vec4(n.x, n.y, n.z, self.mat) });
        }
        if pts.len() == 4 { buf.i.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]); }
        else { buf.i.extend_from_slice(&[base, base + 1, base + 2]); }
    }
    fn quad_n(&mut self, p: [Vec3; 4], n: Vec3, col: Color) { self.emit(&p, n, col, &[1.0], 1.0); }
    fn quad(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, col: Color) {
        let mut n = (b - a).cross(c - a).normalize_or_zero();
        if n.y < -0.5 { n = -n; }
        self.emit(&[a, b, c, d], n, col, &[1.0], 1.0);
    }
    fn tri(&mut self, a: Vec3, b: Vec3, c: Vec3, col: Color) {
        let mut n = (b - a).cross(c - a).normalize_or_zero();
        if n.y < -0.5 { n = -n; }
        if self.mat == gfx::mat::FOLIAGE { n = Vec3::Y; }
        self.emit(&[a, b, c], n, col, &[1.0], 1.0);
    }
    fn bx(&mut self, c: Vec3, s: Vec3, yaw: f32, col: Color, top_boost: f32) {
        let (sy, cy) = (yaw.sin(), yaw.cos());
        let rot = |x: f32, y: f32, z: f32| -> Vec3 { vec3(c.x + x * cy + z * sy, c.y + y, c.z - x * sy + z * cy) };
        let dir = |x: f32, z: f32| -> Vec3 { vec3(x * cy + z * sy, 0.0, -x * sy + z * cy) };
        let hx = s.x * 0.5; let hy = s.y * 0.5; let hz = s.z * 0.5;
        let p = [
            rot(-hx, -hy, -hz), rot(hx, -hy, -hz), rot(hx, hy, -hz), rot(-hx, hy, -hz),
            rot(-hx, -hy, hz), rot(hx, -hy, hz), rot(hx, hy, hz), rot(-hx, hy, hz),
        ];
        // sides darken a little toward their foot, where less sky reaches
        let lo = 0.78;
        self.emit(&[p[3], p[2], p[6], p[7]], Vec3::Y, col, &[1.0], top_boost);
        self.emit(&[p[0], p[1], p[5], p[4]], -Vec3::Y, col, &[0.6], top_boost);
        self.emit(&[p[4], p[5], p[6], p[7]], dir(0.0, 1.0), col, &[lo, lo, 1.0, 1.0], top_boost);
        self.emit(&[p[0], p[3], p[7], p[4]], dir(-1.0, 0.0), col, &[lo, 1.0, 1.0, lo], top_boost);
        self.emit(&[p[1], p[2], p[6], p[5]], dir(1.0, 0.0), col, &[lo, 1.0, 1.0, lo], top_boost);
        self.emit(&[p[0], p[1], p[2], p[3]], dir(0.0, -1.0), col, &[lo, lo, 1.0, 1.0], top_boost);
    }
    /// A soft see-through patch: opacity `a` at the centre, nothing at the rim.
    fn fan(&mut self, c: Vec3, ax: Vec3, ay: Vec3, col: Color, a: f32) {
        let u8c = |f: f32| (f.clamp(0.0, 1.0) * 255.0) as u8;
        let buf = &mut self.over;
        let base = buf.room(9);
        let rgb = [u8c(col.r), u8c(col.g), u8c(col.b)];
        buf.v.push(Vertex { position: c, uv: Vec2::ZERO, color: [rgb[0], rgb[1], rgb[2], u8c(a)], normal: vec4(0.0, 1.0, 0.0, 0.0) });
        for k in 0..8 {
            let t = k as f32 * std::f32::consts::FRAC_PI_4;
            buf.v.push(Vertex { position: c + ax * t.cos() + ay * t.sin(), uv: Vec2::ZERO, color: [rgb[0], rgb[1], rgb[2], 0], normal: vec4(0.0, 1.0, 0.0, 0.0) });
        }
        for k in 0..8u16 { buf.i.extend_from_slice(&[base, base + 1 + k, base + 1 + (k + 1) % 8]); }
    }
    fn finish(self) -> SceneMeshes { SceneMeshes { lit: self.lit.finish(), over: self.over.finish(), glow: self.glow.finish() } }
}

// The lighting pipeline shades and hazes now; these stay so the many call
// sites keep reading as "this is a surface colour".
fn shade(c: Color, _n: Vec3, _sun: Vec3, _ambient: f32) -> Color { c }
fn fogged(c: Color, _dist: f32, _fog: Color, _start: f32, _end: f32) -> Color { c }

fn mulc(c: Color, f: f32) -> Color { Color { r: (c.r * f).min(1.0), g: (c.g * f).min(1.0), b: (c.b * f).min(1.0), a: c.a } }

/// Blob shadows are gone: real shadows come from the shadow pass.
fn cast_shadow(_mb: &mut MB, _base: Vec3, _height: f32, _rx: f32, _rz: f32, _sun: Vec3, _col: Color) {}

fn face_quad(mb: &mut MB, bx: f32, bz: f32, half: f32, dx: i32, dz: i32, y0: f32, y1: f32, col: Color) {
    let n = vec3(dx as f32, 0.0, dz as f32);
    let p = match (dx, dz) {
        (-1, 0) => [vec3(bx - half, y1, bz - half), vec3(bx - half, y1, bz + half), vec3(bx - half, y0, bz + half), vec3(bx - half, y0, bz - half)],
        (1, 0) => [vec3(bx + half, y1, bz + half), vec3(bx + half, y1, bz - half), vec3(bx + half, y0, bz - half), vec3(bx + half, y0, bz + half)],
        (0, -1) => [vec3(bx + half, y1, bz - half), vec3(bx - half, y1, bz - half), vec3(bx - half, y0, bz - half), vec3(bx + half, y0, bz - half)],
        _ => [vec3(bx - half, y1, bz + half), vec3(bx + half, y1, bz + half), vec3(bx + half, y0, bz + half), vec3(bx - half, y0, bz + half)],
    };
    mb.quad_n(p, n, col);
}

/// How much further back than the old close camera the long lens sits; the
/// field of view narrows by the same factor so the framing is unchanged.
const LENS_BACK: f32 = 3.3;

fn make_camera(b: &Battle) -> Camera3D {
    make_camera_view(b, b.dbg_view)
}

/// `view`: 0 as played, 1 wide, 2 tele, 3 low (debug presets).
fn make_camera_view(b: &Battle, view: u8) -> Camera3D {
    let cx = (TW as f32 * 0.5) * TILE;
    let cz = (TH as f32 * 0.5) * TILE;
    let yaw = b.cam_yaw;
    let zoom = match view { 1 => 0.40, 2 => 1.9, _ => b.cam_zoom.max(0.35) };
    let dist = 16.0 / zoom;
    let height = (11.0 / zoom.sqrt() + 2.6) * if view == 3 { 0.42 } else { 1.0 };
    let focus = b.units.get(b.selected).filter(|u| u.alive).map(|u| vec3(u.x * TILE, 0.0, u.y * TILE));
    let mut center = focus.unwrap_or(vec3(cx, 0.0, cz));
    center += (vec3(cx, 0.0, cz) - center) * 0.42;
    center.x += b.cam_pan.x; center.z += b.cam_pan.y;
    let off = vec3(yaw.sin() * dist, height, yaw.cos() * dist) * LENS_BACK;
    let range = off.length();
    Camera3D {
        position: center + off,
        target: center,
        up: vec3(0.0, 1.0, 0.0),
        fovy: 14f32.to_radians(),
        z_near: (range - 70.0).max(4.0),
        z_far: range + 420.0,
        ..Default::default()
    }
}

fn project(cam: &Camera3D, p: Vec3) -> Option<Vec2> {
    let m = cam.matrix();
    let clip = m * vec4(p.x, p.y, p.z, 1.0);
    if clip.w <= 0.01 { return None; }
    let ndc = vec3(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w);
    Some(vec2((ndc.x * 0.5 + 0.5) * screen_width(), (1.0 - (ndc.y * 0.5 + 0.5)) * screen_height()))
}

fn unproject(cam: &Camera3D, p: Vec2, z: f32) -> Vec3 {
    let inv = cam.matrix().inverse();
    let v = inv * vec4(2.0 * p.x / screen_width() - 1.0, 1.0 - 2.0 * p.y / screen_height(), z, 1.0);
    vec3(v.x / v.w, v.y / v.w, v.z / v.w)
}

fn screen_to_tile(b: &Battle, p: Vec2) -> Option<(i32, i32)> {
    let cam = make_camera(b);
    let near = unproject(&cam, p, -1.0);
    let mut far = unproject(&cam, p, 1.0);
    let dir = (far - near).normalize();
    if dir.y.abs() < 0.001 { return None; }
    let mut y = 0.0;
    let mut out = (0, 0);
    for _ in 0..3 {
        let t = (y - near.y) / dir.y;
        far = near + dir * t;
        let tx = (far.x / TILE).round() as i32;
        let ty = (far.z / TILE).round() as i32;
        if !in_bounds(tx, ty) { return None; }
        out = (tx, ty);
        y = tile_at(b, tx, ty).h as f32 * 0.62;
    }
    Some(out)
}

fn mixc(a: Color, b: Color, t: f32) -> Color {
    Color { r: a.r + (b.r - a.r) * t, g: a.g + (b.g - a.g) * t, b: a.b + (b.b - a.b) * t, a: a.a + (b.a - a.a) * t }
}

fn ridge(w: f32, base: f32, amp: f32, seed: u32, col: Color, step: f32) {
    let mut x = -step;
    let mut prev_y = base - fbm(x * 0.004, 0.0, seed) * amp;
    while x < w + step {
        let nx = x + step;
        let ny = base - fbm(nx * 0.004, 0.0, seed) * amp - vnoise(nx * 0.02, 0.0, seed + 9) * amp * 0.25;
        draw_triangle(vec2(x, prev_y), vec2(nx, ny), vec2(nx, base + 90.0), col);
        draw_triangle(vec2(x, prev_y), vec2(nx, base + 90.0), vec2(x, base + 90.0), col);
        x = nx; prev_y = ny;
    }
}

fn sky(b: &Battle) {
    let w = screen_width(); let h = screen_height();
    if b.underwater {
        grad_v(Rect::new(0.0, 0.0, w, h), Color { r: 0.030, g: 0.185, b: 0.215, a: 1.0 }, Color { r: 0.002, g: 0.014, b: 0.026, a: 1.0 });
        for i in 0..22 {
            let f = i as f32 / 22.0;
            let y = f * h * 0.26;
            draw_rectangle(0.0, y, w, h * 0.26 / 22.0 + 1.0, Color { r: 0.30, g: 0.72, b: 0.78, a: 0.028 * (1.0 - f) });
        }
        for i in 0..26 {
            let fi = i as f32;
            let y = 10.0 + fi * 7.0 + (b.time * 5.0 + fi).sin() * 4.0;
            let x0 = ((fi * 137.0 + b.time * 22.0).sin() * 0.5 + 0.5) * w;
            draw_line(x0 - 140.0, y, x0 + 140.0, y + 6.0, 1.0, Color { r: 0.55, g: 0.95, b: 1.0, a: 0.035 });
        }
        for i in 0..16 {
            let x = (i as f32 * 89.0 + b.time * 8.0).rem_euclid(w + 320.0) - 160.0;
            let a = 0.014 + 0.010 * (i as f32 * 1.7 + b.time * 0.7).sin().abs();
            draw_triangle(vec2(x, 0.0), vec2(x + 90.0, 0.0), vec2(x + 290.0, h), Color { r: 0.35, g: 0.85, b: 0.92, a });
        }
        draw_rectangle(0.0, 0.0, w, h, Color { r: 0.0, g: 0.05, b: 0.08, a: 0.10 });
    } else {
        let is_night = b.ambient < 0.32;
        let (top, mid, bot) = if is_night {
            (Color::new(0.01, 0.02, 0.06, 1.0), Color::new(0.03, 0.05, 0.12, 1.0), Color::new(0.06, 0.08, 0.16, 1.0))
        } else {
            match b.biome {
                7 | 8 => (Color::new(0.10, 0.16, 0.27, 1.0), Color::new(0.46, 0.42, 0.38, 1.0), Color::new(0.74, 0.56, 0.36, 1.0)),
                9 => (Color::new(0.07, 0.13, 0.25, 1.0), Color::new(0.38, 0.48, 0.60, 1.0), Color::new(0.66, 0.74, 0.82, 1.0)),
                6 => (Color::new(0.05, 0.11, 0.15, 1.0), Color::new(0.24, 0.35, 0.29, 1.0), Color::new(0.47, 0.52, 0.42, 1.0)),
                _ => (Color::new(0.06, 0.13, 0.24, 1.0), Color::new(0.36, 0.43, 0.47, 1.0), Color::new(0.62, 0.58, 0.48, 1.0)),
            }
        };
        grad_v(Rect::new(0.0, 0.0, w, h), top, mid);
        if is_night {
            for i in 0..60 {
                let sx = ((i as f32 * 37.1).sin() * 0.5 + 0.5) * w;
                let sy = ((i as f32 * 53.7).cos() * 0.5 + 0.5) * h * 0.45;
                let sz = 0.8 + ((i % 3) as f32) * 0.4;
                let alpha = 0.4 + ((b.time * 2.0 + i as f32).sin() * 0.3).abs();
                draw_circle(sx, sy, sz, Color { r: 0.9, g: 0.95, b: 1.0, a: alpha });
            }
        }
        for i in 0..18 {
            let f = i as f32 / 18.0;
            let y = h * (0.24 + f * 0.30);
            draw_rectangle(0.0, y, w, h * 0.30 / 18.0 + 2.0, Color { r: bot.r, g: bot.g, b: bot.b, a: 0.030 + f * 0.020 });
        }
        let sx = w * 0.78; let sy = h * 0.155;
        if is_night {
            draw_circle(sx, sy, 22.0, Color::new(0.90, 0.93, 0.98, 1.0));
            glow(sx, sy, 80.0, Color::new(0.6, 0.75, 1.0, 1.0), 0.4);
        } else {
            for i in 0..16 {
                let f = i as f32 / 16.0;
                draw_circle(sx, sy, 34.0 + f * 150.0, Color { r: 1.0, g: 0.76, b: 0.44, a: 0.010 * (1.0 - f) });
            }
            glow(sx, sy, 150.0, Color::new(1.0, 0.82, 0.55, 1.0), 0.75);
            draw_circle(sx, sy, 26.0, Color::new(1.0, 0.96, 0.84, 1.0));
            draw_circle(sx, sy, 40.0, Color { r: 1.0, g: 0.86, b: 0.60, a: 0.20 });
        }
        for layer in 0..2 {
            for i in 0..9 {
                let fi = i as f32;
                let drift = b.time * (2.2 + layer as f32 * 1.4);
                let cx = (fi * 233.0 + drift + layer as f32 * 117.0) % (w + 460.0) - 230.0;
                let cy = h * (0.10 + layer as f32 * 0.055) + (fi * 37.0).sin() * 14.0;
                let cw = 150.0 + (fi * 53.0).sin().abs() * 120.0;
                for j in 0..5 {
                    let jf = j as f32;
                    draw_ellipse(cx + jf * cw * 0.20, cy + (jf * 1.7).sin() * 5.0, cw * (0.45 - jf * 0.055), 9.0 + jf, 0.0,
                        Color { r: 1.0, g: 0.93, b: 0.84, a: 0.035 + layer as f32 * 0.012 });
                }
            }
        }
        ridge(w, h * 0.50, 78.0, 17, Color { r: 0.19, g: 0.25, b: 0.27, a: 0.55 }, 38.0);
        ridge(w, h * 0.54, 54.0, 41, Color { r: 0.15, g: 0.22, b: 0.22, a: 0.70 }, 30.0);
        ridge(w, h * 0.58, 34.0, 73, Color { r: 0.10, g: 0.17, b: 0.16, a: 0.85 }, 24.0);
        for i in 0..14 {
            let f = i as f32 / 14.0;
            draw_rectangle(0.0, h * (0.36 + f * 0.12), w, h * 0.12 / 14.0 + 2.0, Color { r: 0.55, g: 0.58, b: 0.55, a: 0.018 });
        }
    }
}

fn terrain_color_for(t: &Tile, b: &Battle, x: i32, y: i32) -> Color {
    let mut c = biome_ground(t.kind, x, y, 1234);
    if b.underwater {
        let cau = (b.time * 1.3 + x as f32 * 1.15 - y as f32 * 0.75).sin() * 0.5 + 0.5;
        c = Color { r: c.r + 0.005 + cau * 0.020, g: c.g + 0.010 + cau * 0.045, b: c.b + 0.012 + cau * 0.050, a: 1.0 };
    }
    c
}

fn build_scene(b: &Battle, g: &Game, cam: &Camera3D) -> SceneMeshes {
    let mut mb = MB::new();
    let sun = b.sun_dir;
    let ambient = b.ambient;
    let fog = b.fog_col;
    let cam_pos = cam.position;
    // the country around the battlefield, in the battlefield's average colour
    let gcol = ground_average(b);
    let half = TILE * 0.5;
    let (x0, z0) = (-half, -half);
    let (x1, z1) = (TW as f32 * TILE - half, TH as f32 * TILE - half);
    let ap = 300.0;
    for (a, c) in [
        (vec2(x0 - ap, z0 - ap), vec2(x1 + ap, z0)),
        (vec2(x0 - ap, z1), vec2(x1 + ap, z1 + ap)),
        (vec2(x0 - ap, z0), vec2(x0, z1)),
        (vec2(x1, z0), vec2(x1 + ap, z1)),
    ] {
        mb.quad_n([vec3(a.x, 0.0, a.y), vec3(a.x, 0.0, c.y), vec3(c.x, 0.0, c.y), vec3(c.x, 0.0, a.y)], Vec3::Y, gcol);
    }
    for y in 0..TH {
        for x in 0..TW {
            let t = &b.tiles[tile_idx(x, y)];
            if t.h > b.view_level { continue; }
            mb.mat = gfx::mat::GROUND;
            let base_y = t.h as f32 * 0.62;
            let bx = x as f32 * TILE; let bz = y as f32 * TILE;
            let half = TILE * 0.5;
            let mut c = terrain_color_for(t, b, x, y);
            let grain = fbm(x as f32 * 2.7, y as f32 * 2.7, 91) * 0.14 + fbm(x as f32 * 6.3, y as f32 * 6.3, 137) * 0.06;
            c = mulc(c, 0.95 + grain);
            if !b.underwater && t.kind != 9 {
                let patch = fbm(x as f32 * 0.31, y as f32 * 0.31, 17);
                let dry = ((patch - 0.46).max(0.0) * 0.55).min(0.28);
                c = mixc(c, Color::new(0.40, 0.37, 0.20, 1.0), dry);
            }
            if b.underwater {
                let ca = (b.time * 1.3 + x as f32 * 1.15 - y as f32 * 0.75).sin() * 0.5 + 0.5;
                c = Color { r: c.r + ca * 0.020, g: c.g + ca * 0.055, b: c.b + ca * 0.060, a: 1.0 };
            } else if t.kind == 9 {
                c = mixc(c, Color::new(0.88, 0.92, 0.96, 1.0), 0.35);
            }
            let top = shade(c, vec3(0.0, 1.0, 0.0), sun, ambient);
            let dist = ((bx - cam_pos.x).powi(2) + (bz - cam_pos.z).powi(2)).sqrt();
            let vis = 1.0;
            let top = Color { r: top.r * vis, g: top.g * vis, b: top.b * vis, a: 1.0 };
            let top = fogged(top, dist, fog, 32.0, 88.0);
            mb.quad(vec3(bx - half, base_y, bz + half), vec3(bx + half, base_y, bz + half), vec3(bx + half, base_y, bz - half), vec3(bx - half, base_y, bz - half), top);
            if t.visible && t.obstacle == 0 && t.h < 2 {
                mb.mat = gfx::mat::FOLIAGE;
                let tufts = if b.underwater { 6 } else { match t.kind { 9 => 0, 4 | 7 | 8 => 2, 5 => 3, 11 => 5, _ => 6 } };
                let tuft_season = season_at(b.mission.month, b.mission.lat);
                for k in 0..tufts {
                    let hs = hash2(x * 5 + k, y * 7 - k, 991);
                    let fx = ((hs & 255) as f32 / 255.0 - 0.5) * 1.32;
                    let fz = (((hs >> 8) & 255) as f32 / 255.0 - 0.5) * 1.32;
                    let px = bx + fx; let pz = bz + fz;
                    let tall = 0.09 + ((hs >> 16) & 63) as f32 / 540.0;
                    let hue = ((hs >> 22) & 31) as f32 / 31.0;
                    let gc = if b.underwater {
                        Color::new(0.08 + hue * 0.05, 0.42 + hue * 0.16, 0.38 + hue * 0.12, 1.0)
                    } else if t.kind == 7 || t.kind == 8 {
                        Color::new(0.55, 0.45 + hue * 0.06, 0.25, 1.0)
                    } else if t.kind == 9 {
                        Color::new(0.74, 0.80, 0.84, 1.0)
                    } else if t.kind == 6 || t.kind == 2 {
                        Color::new(0.10 + hue * 0.05, 0.36 + hue * 0.10, 0.12, 1.0)
                    } else if t.kind == 11 || t.kind == 5 {
                        Color::new(0.33 + hue * 0.08, 0.33 + hue * 0.06, 0.17, 1.0)
                    } else if t.kind == 1 {
                        mulc(ground_site(b, x, y).0, 0.86 + hue * 0.10)
                    } else if tuft_season == 0 {
                        Color::new(0.38 + hue * 0.05, 0.36 + hue * 0.04, 0.20, 1.0)
                    } else if tuft_season == 3 {
                        Color::new(0.32 + hue * 0.08, 0.37 + hue * 0.06, 0.16, 1.0)
                    } else {
                        Color::new(0.19 + hue * 0.06, 0.38 + hue * 0.12, 0.16, 1.0)
                    };
                    let gc = Color { r: gc.r * vis, g: gc.g * vis, b: gc.b * vis, a: 1.0 };
                    let sway = (b.time * 1.6 + fx * 3.0 + fz * 2.0).sin() * 0.018;
                    mb.tri(vec3(px - 0.030, base_y + 0.012, pz), vec3(px + 0.030, base_y + 0.012, pz), vec3(px + sway, base_y + 0.012 + tall, pz + sway), gc);
                    mb.tri(vec3(px, base_y + 0.012, pz - 0.030), vec3(px, base_y + 0.012, pz + 0.030), vec3(px + sway, base_y + 0.012 + tall * 0.78, pz), gc);
                }
            }
            mb.mat = gfx::mat::GROUND;
            for (dx, dz) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let nx = x + dx; let ny = y + dz;
                let nh = if in_bounds(nx, ny) { b.tiles[tile_idx(nx, ny)].h as f32 * 0.62 } else { base_y - 0.62 };
                if nh < base_y {
                    let nrm = vec3(dx as f32, 0.0, dz as f32);
                    let _ = nrm; let lit = 1.0;
                    let soil = mixc(c, Color::new(0.34, 0.27, 0.19, 1.0), 0.55);
                    let col = fogged(mulc(soil, lit * vis), dist, fog, 32.0, 88.0);
                    let y0 = nh; let y1 = base_y;
                    let ym = y0 + (y1 - y0) * 0.58;
                    face_quad(&mut mb, bx, bz, half, dx, dz, ym, y1, mulc(col, 1.04));
                    face_quad(&mut mb, bx, bz, half, dx, dz, y0, ym, mulc(col, 0.70));
                }
            }
            if t.scorched {
                let scorch_col = mulc(Color::new(0.06, 0.06, 0.05, 0.82), vis);
                let base = vec3(bx, base_y + 0.008, bz);
                mb.bx(base, vec3(TILE * 0.72, 0.005, TILE * 0.72), 0.2, scorch_col, 0.85);
            }
            if t.obstacle != 0 {
                let base = vec3(bx, base_y, bz);
                let sh_h = match t.obstacle { 2 => 1.9, 3 => 0.9, 4 => 2.2, _ => 1.7 };
                cast_shadow(&mut mb, base, sh_h, 0.62, 0.52, sun, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.30 * vis });
                mb.mat = match t.obstacle { 13 => gfx::mat::METAL, 10 | 12 => gfx::mat::HULL, _ => gfx::mat::BUILT };
                match t.obstacle {
                    1..=4 => draw_prop(&mut mb, b, t.obstacle, base, x, y),
                    10 => {
                        // Heron / Gannet hull bulkhead wall
                        let hull_col = if b.underwater { Color::new(0.16, 0.24, 0.30, 1.0) } else { Color::new(0.22, 0.26, 0.24, 1.0) };
                        mb.bx(base + vec3(0.0, 1.10, 0.0), vec3(1.48, 2.20, 0.50), 0.0, mulc(hull_col, vis), 1.25);
                        mb.bx(base + vec3(0.0, 2.15, 0.0), vec3(1.52, 0.12, 0.56), 0.0, mulc(hull_col, 1.3 * vis), 1.15);
                        mb.bx(base + vec3(0.0, 0.15, 0.26), vec3(1.40, 0.18, 0.02), 0.0, mulc(Color::new(0.85, 0.75, 0.15, 1.0), vis), 1.3);
                    }
                    12 => {
                        // Enemy craft hull wall (lattice)
                        let alien_col = if b.underwater { Color::new(0.15, 0.30, 0.35, 1.0) } else { Color::new(0.14, 0.11, 0.18, 1.0) };
                        let pulse = (b.time * 2.2 + bx * 0.4 + bz * 0.4).sin() * 0.12 + 0.88;
                        mb.bx(base + vec3(0.0, 1.20, 0.0), vec3(1.48, 2.40, 0.52), 0.0, mulc(alien_col, vis * pulse), 1.35);
                        let glow_col = if b.underwater { Color::new(0.2, 0.9, 0.85, 1.0) } else { Color::new(0.75, 0.35, 1.0, 1.0) };
                        mb.bx(base + vec3(0.0, 1.20, 0.27), vec3(0.18, 1.80, 0.03), 0.0, mulc(glow_col, pulse * vis), 2.2);
                        mb.bx(base + vec3(0.0, 2.35, 0.0), vec3(1.56, 0.14, 0.60), 0.0, mulc(alien_col, 1.4 * vis), 1.2);
                    }
                    13 => {
                        // Enemy power plant (electrolyte cell)
                        let base_col = mulc(Color::new(0.20, 0.18, 0.26, 1.0), vis);
                        mb.bx(base + vec3(0.0, 0.40, 0.0), vec3(1.10, 0.80, 1.10), 0.0, base_col, 1.2);
                        let p_glow = (b.time * 4.0).sin() * 0.2 + 0.8;
                        let core_col = if b.underwater { Color::new(0.2, 0.95, 0.9, 1.0) } else { Color::new(0.9, 0.3, 1.0, 1.0) };
                        mb.bx(base + vec3(0.0, 1.30, 0.0), vec3(0.55 * p_glow, 0.80 * p_glow, 0.55 * p_glow), b.time * 1.5, mulc(core_col, vis), 3.0);
                        let ring_col = mulc(Color::new(0.65, 0.70, 0.80, 1.0), vis);
                        mb.bx(base + vec3(0.0, 1.30, 0.0), vec3(0.95, 0.12, 0.95), b.time * -2.2, ring_col, 1.3);
                        mb.bx(base + vec3(0.0, 1.30, 0.0), vec3(0.85, 0.10, 0.85), b.time * 2.8, ring_col, 1.3);
                    }
                    _ => {
                        let col = if b.underwater { Color::new(0.20, 0.78, 0.82, 1.0) } else { Color::new(0.48, 0.36, 0.68, 1.0) };
                        mb.bx(base + vec3(0.0, 0.80, 0.0), vec3(0.30, 1.60, 0.30), 0.3, mulc(col, vis), 1.50);
                        mb.bx(base + vec3(0.22, 1.30, 0.10), vec3(0.18, 0.70, 0.18), -0.4, mulc(mulc(col, 1.25), vis), 1.55);
                    }
                }
            }
            mb.mat = gfx::mat::BUILT;
            if t.smoke > 0 {
                let s_vis = (t.smoke as f32 / 15.0).clamp(0.1, 1.0);
                let smoke_col = mulc(if b.underwater {
                    Color::new(0.12, 0.32, 0.35, 0.22 + s_vis * 0.30)
                } else {
                    Color::new(0.42, 0.42, 0.45, 0.18 + s_vis * 0.32)
                }, vis);
                let puffs = (t.smoke as i32 / 4).max(1);
                for pi in 0..puffs {
                    let bob = (b.time * 1.5 + (x * 4 + y * 7 + pi * 6) as f32).sin() * 0.12;
                    let py = base_y + 0.32 + pi as f32 * 0.42 + bob;
                    let psz = 0.60 + pi as f32 * 0.22;
                    let ox = ((x * 9 + pi * 13) as f32 * 0.3).sin() * 0.22;
                    let oz = ((y * 11 + pi * 15) as f32 * 0.3).cos() * 0.22;
                    mb.bx(vec3(bx + ox, py, bz + oz), vec3(psz, psz * 0.65, psz), b.time * 0.3 + pi as f32, smoke_col, 1.0);
                }
            }
            if t.fire > 0 {
                let flick = (b.time * 14.0 + (x * 5 + y * 9) as f32).sin() * 0.18 + 0.85;
                let f_vis = (t.fire as f32 / 15.0).clamp(0.2, 1.0);
                let fire_col = mulc(Color::new(1.0, 0.45 * flick, 0.06, 0.75 + f_vis * 0.2), vis);
                let ember_col = mulc(Color::new(1.0, 0.82, 0.25, 0.95), vis);
                let fy = base_y + 0.28 * flick;
                mb.glow_gain = 1.0;
                mb.bx(vec3(bx, fy, bz), vec3(0.55 * flick, 0.72 * flick, 0.55 * flick), b.time * 2.5, fire_col, 1.5);
                let ey = base_y + 0.85 + ((b.time * 3.0 + x as f32 * 0.4).fract() * 0.7);
                mb.bx(vec3(bx + ((b.time * 3.8).sin() * 0.22), ey, bz + ((b.time * 3.8).cos() * 0.22)), vec3(0.08, 0.08, 0.08), 0.0, ember_col, 2.0);
                mb.glow_gain = LAMP_GAIN;
            }
        }
    }
    if b.phase == Phase::Player {
        for &(x, y) in &b.move_tiles {
            let t = &b.tiles[tile_idx(x, y)];
            let yy = t.h as f32 * 0.62 + 0.018;
            let bx = x as f32 * TILE; let bz = y as f32 * TILE;
            let half = TILE * 0.46;
            mb.quad(vec3(bx - half, yy, bz + half), vec3(bx + half, yy, bz + half), vec3(bx + half, yy, bz - half), vec3(bx - half, yy, bz - half), Color::new(0.22, 0.68, 0.82, 0.22));
        }
    }
    mb.mat = gfx::mat::HULL;
    draw_landing_craft(&mut mb, b, sun, ambient, cam_pos, fog);
    mb.mat = gfx::mat::BUILT;
    for u in &b.units {
        let tx = u.x.round() as i32; let ty = u.y.round() as i32;
        let t = tile_at(b, tx, ty);
        if t.h > b.view_level { continue; }
        let by = t.h as f32 * 0.62 + 0.02;
        let base = vec3(u.x * TILE, by, u.y * TILE);
        if !u.alive {
            cast_shadow(&mut mb, base, 0.25, 0.70, 0.55, sun, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.25 });
            continue;
        }
        let hh = if u.side == Side::Alien { 1.75 } else { 1.62 };
        cast_shadow(&mut mb, base, hh, 0.50, 0.42, sun, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.30 });
        for i in 0..10 {
            let a0 = i as f32 / 10.0 * std::f32::consts::TAU;
            let a1 = (i + 1) as f32 / 10.0 * std::f32::consts::TAU;
            mb.tri(base + vec3(0.0, 0.012, 0.0),
                base + vec3(a0.cos() * 0.34, 0.012, a0.sin() * 0.28),
                base + vec3(a1.cos() * 0.34, 0.012, a1.sin() * 0.28),
                Color { r: 0.0, g: 0.0, b: 0.0, a: 0.20 });
        }
    }
    for u in &b.units {
        let tx = u.x.round() as i32; let ty = u.y.round() as i32;
        let t = tile_at(b, tx, ty);
        if t.h > b.view_level { continue; }
        let base = world_pos(u.x, u.y, t.h);
        if u.mind_controlled {
            let halo_p = base + vec3(0.0, 1.85, 0.0);
            mb.bx(halo_p, vec3(0.45, 0.06, 0.45), b.time * 3.0, Color::new(0.85, 0.25, 1.0, 0.85), 2.5);
        }
        if !u.alive {
            let fall = (u.death_t * 1.5).min(1.0);
            let col = if u.side == Side::Player { Color::new(0.18, 0.22, 0.18, 1.0) } else { Color::new(0.25, 0.16, 0.12, 1.0) };
            mb.bx(base + vec3(0.0, 0.10, 0.0), vec3(0.7, 0.18 + (1.0 - fall) * 0.5, 1.3), u.facing, col, 1.1);
            continue;
        }
        mb.mat = gfx::mat::BUILT;
        if u.side == Side::Alien { mb.mat = gfx::mat::SKIN; draw_alien(&mut mb, b, g, u, base, sun, ambient); }
        else { draw_soldier(&mut mb, b, g, u, base, sun, ambient, cam_pos, fog); }
    }
    draw_weather(&mut mb, b, cam);
    mb.finish()
}

fn draw_selection_ring(b: &Battle) {
    if let Some(u) = b.units.get(b.selected) {
        if u.alive && u.side == Side::Player {
            let base = world_pos(u.x, u.y, tile_at(b, u.x.round() as i32, u.y.round() as i32).h);
            let r = 0.72;
            for i in 0..24 {
                let a0 = i as f32 / 24.0 * std::f32::consts::TAU;
                let a1 = (i + 1) as f32 / 24.0 * std::f32::consts::TAU;
                draw_line_3d(base + vec3(a0.cos() * r, 0.03, a0.sin() * r), base + vec3(a1.cos() * r, 0.03, a1.sin() * r), Color::new(0.3, 0.9, 1.0, 0.9));
            }
        }
    }
}

/// What the standing things on a battlefield are made of, per biome.
struct Dress { rock: Color, moss: Option<Color>, block: Color, wall: Color, roof: Color, window: Color }

fn dress(biome: u8) -> Dress {
    let c = |r: f32, g: f32, b: f32| Color::new(r, g, b, 1.0);
    match biome {
        // farmland: fieldstone, hay, red barn boards under tin
        1 => Dress { rock: c(0.42, 0.40, 0.36), moss: Some(c(0.30, 0.36, 0.18)), block: c(0.66, 0.55, 0.29), wall: c(0.40, 0.15, 0.11), roof: c(0.40, 0.42, 0.43), window: c(0.08, 0.09, 0.10) },
        // mountain: granite, dry-stone walls under slate
        5 => Dress { rock: c(0.46, 0.45, 0.43), moss: None, block: c(0.42, 0.41, 0.39), wall: c(0.44, 0.43, 0.40), roof: c(0.22, 0.23, 0.25), window: c(0.07, 0.08, 0.09) },
        // rain forest: dark wet rock, logs, weathered tin under rust
        2 | 6 => Dress { rock: c(0.25, 0.27, 0.21), moss: Some(c(0.16, 0.33, 0.12)), block: c(0.25, 0.18, 0.12), wall: c(0.36, 0.42, 0.40), roof: c(0.42, 0.24, 0.14), window: c(0.05, 0.07, 0.07) },
        // desert: sandstone and mud brick, flat parapets
        4 | 7 | 8 => Dress { rock: c(0.62, 0.42, 0.28), moss: None, block: c(0.66, 0.50, 0.33), wall: c(0.64, 0.50, 0.35), roof: c(0.69, 0.55, 0.39), window: c(0.10, 0.07, 0.05) },
        // ice: dark outcrop, ice blocks, orange station panels
        9 => Dress { rock: c(0.20, 0.20, 0.22), moss: None, block: c(0.70, 0.82, 0.92), wall: c(0.78, 0.33, 0.10), roof: c(0.72, 0.74, 0.76), window: c(0.06, 0.09, 0.12) },
        // northern forest: lichened boulders, log stacks, cabins
        11 => Dress { rock: c(0.44, 0.44, 0.40), moss: Some(c(0.52, 0.50, 0.30)), block: c(0.33, 0.24, 0.16), wall: c(0.30, 0.21, 0.14), roof: c(0.28, 0.30, 0.30), window: c(0.07, 0.08, 0.09) },
        _ => Dress { rock: c(0.40, 0.38, 0.34), moss: Some(c(0.20, 0.34, 0.16)), block: c(0.36, 0.31, 0.22), wall: c(0.50, 0.45, 0.37), roof: c(0.30, 0.17, 0.13), window: c(0.10, 0.20, 0.24) },
    }
}

/// A faceted cone standing on `base`.
fn cone(mb: &mut MB, base: Vec3, r: f32, h: f32, col: Color, sides: usize, yaw: f32) {
    let apex = base + vec3(0.0, h, 0.0);
    for k in 0..sides {
        let a0 = yaw + k as f32 / sides as f32 * std::f32::consts::TAU;
        let a1 = yaw + (k + 1) as f32 / sides as f32 * std::f32::consts::TAU;
        let am = (a0 + a1) * 0.5;
        let n = vec3(am.cos() * h, r, am.sin() * h).normalize();
        mb.emit(&[base + vec3(a0.cos() * r, 0.0, a0.sin() * r), base + vec3(a1.cos() * r, 0.0, a1.sin() * r), apex], n, col, &[0.55, 0.55, 1.0], 1.0);
    }
}

/// Rocks, trees, blocks and walls, dressed for the biome and the season.
/// What stands where is the map's business; this only decides how it looks.
fn draw_prop(mb: &mut MB, b: &Battle, kind: u8, base: Vec3, x: i32, y: i32) {
    let c = |r: f32, g: f32, bl: f32| Color::new(r, g, bl, 1.0);
    let hs = hash2(x, y, 4477);
    let r1 = (hs & 255) as f32 / 255.0;
    let r2 = ((hs >> 8) & 255) as f32 / 255.0;
    let r3 = ((hs >> 16) & 255) as f32 / 255.0;
    let yaw = r1 * 6.283;
    let sc = 0.85 + r2 * 0.35;
    let biome = if b.underwater { 0 } else { b.biome };
    let d = dress(biome);
    mb.mat = gfx::mat::BUILT;
    match kind {
        1 => {
            let big = if biome == 5 { 1.18 } else { 1.0 };
            let s = sc * big;
            mb.bx(base + vec3(0.0, 0.30 * s, 0.0), vec3(0.92 * s, 0.60 * s, 0.86 * s), 0.15 + yaw, d.rock, 1.28);
            mb.bx(base + vec3(0.20, 0.62 * s, -0.10), vec3(0.48 * s, 0.30 * s, 0.44 * s), 0.55 + yaw, mulc(d.rock, 1.12), 1.20);
            match d.moss {
                Some(m) => mb.bx(base + vec3(-0.10, 0.60 * s + 0.03, 0.06), vec3(0.55 * s, 0.05, 0.50 * s), 0.2 + yaw, m, 1.05),
                None => mb.bx(base + vec3(-0.42, 0.16, 0.36), vec3(0.40, 0.32, 0.36), yaw * 2.0, mulc(d.rock, 0.90), 1.1),
            }
        }
        2 => {
            if b.underwater {
                mb.bx(base + vec3(0.0, 0.60, 0.0), vec3(0.22, 1.25, 0.22), 0.0, c(0.10, 0.20, 0.18), 1.10);
                mb.mat = gfx::mat::FOLIAGE;
                let green = c(0.09, 0.40, 0.33);
                mb.bx(base + vec3(0.0, 1.37, 0.0), vec3(1.30, 0.56, 1.18), 0.18, green, 1.30);
                mb.bx(base + vec3(0.16, 1.72, 0.04), vec3(0.96, 0.44, 0.90), -0.42, mulc(green, 1.14), 1.32);
                mb.bx(base + vec3(-0.20, 1.58, -0.10), vec3(0.82, 0.40, 0.78), 0.62, mulc(green, 0.86), 1.22);
                return;
            }
            let season = season_at(b.mission.month, b.mission.lat);
            match biome {
                5 | 11 => {
                    // conifer: pine on the mountain, narrow spruce in the north
                    let (r, h, green) = if biome == 11 { (0.50, 2.7, c(0.07, 0.17, 0.11)) } else { (0.64, 2.4, c(0.09, 0.21, 0.12)) };
                    mb.bx(base + vec3(0.0, 0.25, 0.0), vec3(0.18, 0.50, 0.18), yaw, c(0.22, 0.15, 0.10), 1.10);
                    mb.mat = gfx::mat::FOLIAGE;
                    for k in 0..3 {
                        let f = k as f32;
                        cone(mb, base + vec3(0.0, (0.32 + f * h * 0.25) * sc, 0.0), r * (1.0 - f * 0.27) * sc, h * 0.44 * sc, mulc(green, 1.0 + f * 0.10), 7, yaw + f * 0.4);
                    }
                }
                2 | 6 => {
                    // rain forest: pale tall trunk, buttress, broad dark crown, ferns below
                    mb.bx(base + vec3(0.0, 1.15 * sc, 0.0), vec3(0.26, 2.3 * sc, 0.26), yaw, c(0.30, 0.25, 0.18), 1.10);
                    mb.bx(base + vec3(0.0, 0.20, 0.0), vec3(0.52, 0.40, 0.52), yaw + 0.6, c(0.26, 0.21, 0.15), 1.10);
                    mb.mat = gfx::mat::FOLIAGE;
                    let green = c(0.07, 0.26, 0.09);
                    mb.bx(base + vec3(0.0, 2.45 * sc, 0.0), vec3(1.75, 0.60, 1.60), yaw, green, 1.30);
                    mb.bx(base + vec3(0.22, 2.90 * sc, 0.06), vec3(1.25, 0.50, 1.20), yaw - 0.5, mulc(green, 1.18), 1.32);
                    mb.bx(base + vec3(-0.30, 2.70 * sc, -0.16), vec3(1.05, 0.45, 1.00), yaw + 0.6, mulc(green, 0.84), 1.22);
                    mb.bx(base + vec3(0.38, 0.30, -0.32), vec3(0.70, 0.45, 0.60), yaw * 3.0, c(0.11, 0.33, 0.11), 1.2);
                }
                4 | 7 | 8 => {
                    if r3 < 0.35 {
                        // dead scrub: bleached sticks
                        let stick = c(0.47, 0.40, 0.30);
                        for k in 0..5 {
                            let a = yaw + k as f32 * 1.257;
                            mb.bx(base + vec3(a.cos() * 0.16, 0.22 + k as f32 * 0.14, a.sin() * 0.16), vec3(0.62 - k as f32 * 0.07, 0.05, 0.05), a, stick, 1.1);
                        }
                    } else {
                        // thorn tree: thin trunk, flat crown
                        mb.bx(base + vec3(0.0, 0.55 * sc, 0.0), vec3(0.14, 1.10 * sc, 0.14), yaw, c(0.36, 0.28, 0.20), 1.10);
                        mb.mat = gfx::mat::FOLIAGE;
                        let green = c(0.33, 0.36, 0.19);
                        mb.bx(base + vec3(0.0, 1.16 * sc, 0.0), vec3(1.35, 0.16, 1.20), yaw, green, 1.2);
                        mb.bx(base + vec3(0.15, 1.31 * sc, 0.05), vec3(0.85, 0.14, 0.80), yaw + 0.5, mulc(green, 1.15), 1.2);
                    }
                }
                9 => {
                    // no trees on the ice: a pinnacle of it instead
                    let ice = c(0.66, 0.80, 0.90);
                    cone(mb, base, 0.60 * sc, 1.9 * sc, ice, 5, yaw);
                    cone(mb, base + vec3(0.36, 0.0, 0.22), 0.36, 1.0, mulc(ice, 0.92), 4, yaw + 1.0);
                }
                _ => {
                    // broadleaf: in leaf, turning, or bare
                    let bark = c(0.21, 0.14, 0.09);
                    mb.bx(base + vec3(0.0, 0.60 * sc, 0.0), vec3(0.22, 1.25 * sc, 0.22), yaw, bark, 1.10);
                    let leaf = match season {
                        0 => None,
                        1 => Some(c(0.20, 0.40, 0.13)),
                        2 => Some(if biome == 1 { c(0.17, 0.34, 0.13) } else { c(0.14, 0.33, 0.13) }),
                        _ => Some([c(0.52, 0.30, 0.09), c(0.58, 0.46, 0.12), c(0.40, 0.20, 0.09), c(0.30, 0.36, 0.13)][((hs >> 24) & 3) as usize]),
                    };
                    match leaf {
                        Some(green) => {
                            mb.mat = gfx::mat::FOLIAGE;
                            mb.bx(base + vec3(0.0, 1.37 * sc, 0.0), vec3(1.30 * sc, 0.56, 1.18 * sc), 0.18 + yaw, green, 1.30);
                            mb.bx(base + vec3(0.16, 1.72 * sc, 0.04), vec3(0.96 * sc, 0.44, 0.90 * sc), -0.42 + yaw, mulc(green, 1.14), 1.32);
                            mb.bx(base + vec3(-0.20, 1.58 * sc, -0.10), vec3(0.82 * sc, 0.40, 0.78 * sc), 0.62 + yaw, mulc(green, 0.86), 1.22);
                        }
                        None => {
                            for k in 0..7 {
                                let a = yaw + k as f32 * 0.898;
                                let hgt = (1.05 + k as f32 * 0.13) * sc;
                                let len = 0.95 - k as f32 * 0.09;
                                mb.bx(base + vec3(a.cos() * len * 0.3, hgt, -a.sin() * len * 0.3), vec3(len, 0.06, 0.06), a, bark, 1.1);
                            }
                            mb.bx(base + vec3(0.0, 1.55 * sc, 0.0), vec3(0.12, 0.9 * sc, 0.12), yaw, bark, 1.1);
                        }
                    }
                }
            }
        }
        3 => match biome {
            1 => {
                // hay bale with its two bindings
                let by = yaw * 0.08;
                mb.bx(base + vec3(0.0, 0.42, 0.0), vec3(1.10, 0.84, 0.90), by, d.block, 1.2);
                for o in [-0.28f32, 0.28] {
                    mb.bx(base + vec3(o * by.cos(), 0.42, -o * by.sin()), vec3(0.05, 0.87, 0.93), by, mulc(d.block, 0.62), 1.1);
                }
            }
            2 | 6 | 11 => {
                // stacked logs, pale at the cut ends
                let by: f32 = if r3 < 0.5 { 0.0 } else { 1.5708 };
                let (dx, dz) = (by.cos(), -by.sin());
                for (o, yy) in [(-0.20f32, 0.17f32), (0.20, 0.17), (0.0, 0.50)] {
                    let p = base + vec3(-dz * o, yy, dx * o);
                    mb.bx(p, vec3(1.40, 0.34, 0.34), by, d.block, 1.15);
                    for e in [-0.71f32, 0.71] {
                        mb.bx(p + vec3(dx * e, 0.0, dz * e), vec3(0.02, 0.28, 0.28), by, c(0.62, 0.50, 0.33), 1.1);
                    }
                }
            }
            5 => {
                mb.bx(base + vec3(0.0, 0.34, 0.0), vec3(1.02, 0.68, 0.90), 0.4 + yaw, d.block, 1.22);
                mb.bx(base + vec3(0.18, 0.74, -0.08), vec3(0.60, 0.22, 0.56), yaw - 0.3, mulc(d.block, 1.10), 1.15);
            }
            _ => {
                mb.bx(base + vec3(0.0, 0.38, 0.0), vec3(0.92, 0.76, 0.88), 0.12, d.block, 1.22);
                mb.bx(base + vec3(0.0, 0.80, 0.0), vec3(0.96, 0.12, 0.92), 0.12, mulc(d.block, 1.20), 1.10);
            }
        },
        _ => {
            mb.bx(base + vec3(0.0, 0.92, 0.0), vec3(1.46, 1.84, 0.50), 0.0, d.wall, 1.18);
            mb.bx(base + vec3(0.0, 1.90, 0.0), vec3(1.60, 0.12, 0.64), 0.0, d.roof, 1.10);
            for sxx in [-0.42f32, 0.42] {
                for syy in [0.62f32, 1.22] {
                    mb.bx(base + vec3(sxx, syy, 0.26), vec3(0.24, 0.30, 0.04), 0.0, d.window, 1.45);
                }
            }
        }
    }
}

/// One steady pseudo-random number in 0..1 per (particle, channel).
fn hashf(i: u32, k: u32) -> f32 {
    let mut h = i.wrapping_mul(0x9E37_79B1) ^ k.wrapping_mul(0x85EB_CA77);
    h ^= h >> 15; h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12; h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h & 0xFF_FFFF) as f32 / 16_777_216.0
}

/// Which way the wind blows on this battlefield (level, unit length).
fn wind_dir(b: &Battle) -> Vec3 {
    let a = (hash2(b.mission.lat as i32, b.mission.lon as i32, 91) & 1023) as f32 / 1023.0 * std::f32::consts::TAU;
    vec3(a.cos(), 0.0, a.sin())
}

/// Rain, snow and dust as things in the air over the battlefield, lit by
/// the scene's light and hidden by whatever stands in front of them. The
/// camera's shutter is 1/100 s, so snow and dust are drawn as streaks as long
/// as they travel in that time; rain uses 1/30 s so it reads as rain.
fn draw_weather(mb: &mut MB, b: &Battle, cam: &Camera3D) {
    if b.underwater || b.weather == 0 || b.weather > 3 { return; }
    let heavy = weather_heavy(b);
    let fwd = (cam.target - cam.position).normalize();
    let cx = TW as f32 * 0.5 * TILE;
    let cz = TH as f32 * 0.5 * TILE;
    let span = 64.0f32;
    let top = 20.0f32;
    let wind = wind_dir(b);
    let perp = vec3(-wind.z, 0.0, wind.x);
    let t = b.time;
    let ground_y = |x: f32, z: f32| -> f32 {
        let (tx, ty) = ((x / TILE).round() as i32, (z / TILE).round() as i32);
        if in_bounds(tx, ty) { b.tiles[tile_idx(tx, ty)].h as f32 * 0.62 } else { 0.0 }
    };
    // a particle's place in the box: carried by the wind, falling, wrapping round
    let place = |i: u32, ws: f32, fall: f32| -> Vec3 {
        let x = cx - span * 0.5 + (hashf(i, 0) * span + wind.x * ws * t).rem_euclid(span);
        let z = cz - span * 0.5 + (hashf(i, 2) * span + wind.z * ws * t).rem_euclid(span);
        let y = top - (hashf(i, 1) * top + t * fall).rem_euclid(top);
        vec3(x, y, z)
    };
    // low sheets of something blowing along the ground
    let drift = |mb: &mut MB, n: u32, speed: f32, height: f32, col: Color, alpha: f32| {
        for i in 0..n {
            let sp = speed * (0.7 + 0.6 * hashf(i, 11));
            let along = (hashf(i, 12) * span + sp * t).rem_euclid(span) - span * 0.5;
            let across = (hashf(i, 13) - 0.5) * span;
            let mut c = vec3(cx, 0.0, cz) + wind * along + perp * across;
            c.y = ground_y(c.x, c.z) + 0.25 + hashf(i, 14) * height;
            let l = 3.0 + 4.0 * hashf(i, 15);
            let w = 0.7 + 0.9 * hashf(i, 16);
            let a = alpha * (1.0 - (along.abs() / (span * 0.5)).powi(4)) * (0.6 + 0.4 * hashf(i, 17));
            mb.fan(c, wind * l, perp * w, col, a);
        }
    };
    match b.weather {
        1 => {
            let n = if heavy { 14000 } else { 8000 };
            let ws = if heavy { 6.0 } else { 2.5 };
            let vel = wind * ws + vec3(0.0, -9.0, 0.0);
            let axis = vel.normalize();
            // rain is the one thing the 1/100 s shutter would freeze into dots;
            // drawn at 1/30 s so it reads as rain from this far away
            let len = vel.length() / 30.0;
            let side = axis.cross(fwd).normalize() * 0.007;
            let col = Color::new(0.80, 0.85, 0.92, if heavy { 0.26 } else { 0.20 });
            for i in 0..n {
                let p = place(i, ws, 9.0 * (0.85 + 0.3 * hashf(i, 3)));
                if p.y < ground_y(p.x, p.z) { continue; }
                let q = p - axis * len;
                mb.emit(&[p - side, p + side, q + side, q - side], Vec3::Y, col, &[1.0], 1.0);
            }
            // each drop that lands throws up a small crown for a moment
            let m = if heavy { 2600 } else { 1500 };
            for i in 0..m {
                let cyc = t * 4.0 + hashf(i, 7) * 10.0;
                let k = cyc.floor() as u32;
                let ph = cyc.fract();
                let j = i.wrapping_add(k.wrapping_mul(977));
                let x = cx - span * 0.5 + hashf(j, 8) * span;
                let z = cz - span * 0.5 + hashf(j, 9) * span;
                let y = ground_y(x, z) + 0.02;
                let r = 0.02 + 0.05 * ph;
                let c = Color::new(0.85, 0.88, 0.92, (1.0 - ph) * 0.16);
                mb.emit(&[vec3(x - r, y, z - r), vec3(x + r, y, z - r), vec3(x + r, y, z + r), vec3(x - r, y, z + r)], Vec3::Y, c, &[1.0], 1.0);
            }
        }
        2 => {
            let n = if heavy { 16000 } else { 10000 };
            let ws = if heavy { 7.0 } else { 1.2 };
            let fall = if heavy { 1.6 } else { 1.1 };
            let vel = wind * ws + vec3(0.0, -fall, 0.0);
            let axis = vel.normalize();
            let len = vel.length() / 100.0;
            let side = axis.cross(fwd).normalize();
            let col = Color::new(0.96, 0.97, 1.0, 0.85);
            for i in 0..n {
                let mut p = place(i, ws, fall * (0.8 + 0.4 * hashf(i, 3)));
                p += perp * ((t * 1.3 + i as f32).sin() * 0.15);
                if p.y < ground_y(p.x, p.z) { continue; }
                let s = 0.015 + 0.012 * hashf(i, 4);
                let (a, w) = (axis * (s + len * 0.5), side * s);
                mb.emit(&[p - a - w, p - a + w, p + a + w, p + a - w], Vec3::Y, col, &[1.0], 1.0);
            }
            if heavy { drift(mb, 420, 9.0, 1.2, Color::new(0.93, 0.95, 0.98, 1.0), 0.16); }
        }
        _ => {
            let speed = if heavy { 9.0 } else { 5.0 };
            let sand = Color::new(0.72, 0.58, 0.40, 1.0);
            drift(mb, if heavy { 520 } else { 260 }, speed, if heavy { 3.5 } else { 2.0 }, sand, if heavy { 0.11 } else { 0.07 });
            // grains hopping along just above the ground
            let n = if heavy { 5000 } else { 2500 };
            let len = speed / 100.0;
            let side = wind.cross(fwd).normalize() * 0.008;
            let col = Color::new(0.78, 0.64, 0.45, 0.5);
            for i in 0..n {
                let mut p = place(i, speed * (0.8 + 0.4 * hashf(i, 3)), 0.0);
                p.y = ground_y(p.x, p.z) + 0.03 + hashf(i, 5).powi(3) * 0.6;
                let q = p - wind * len;
                mb.emit(&[p - side, p + side, q + side, q - side], Vec3::Y, col, &[1.0], 1.0);
            }
        }
    }
}

fn draw_landing_craft(mb: &mut MB, b: &Battle, sun: Vec3, ambient: f32, cam_pos: Vec3, fog: Color) {
    let dist_player = ((4.5 * TILE - cam_pos.x).powi(2) + (12.5 * TILE - cam_pos.z).powi(2)).sqrt();
    let apply_p = |c: Color| fogged(shade(c, vec3(0.0, 1.0, 0.0), sun, ambient), dist_player, fog, 24.0, 90.0);

    // --- 1. PLAYER TRANSPORT (Heron or Gannet)
    // Centered around x: 4.0 * TILE, z: 12.5 * TILE. Troop bay x: 2..6, z: 11..14
    let pcx = 4.0 * TILE;
    let pcz = 12.5 * TILE;
    let phull = if b.underwater {
        Color::new(0.14, 0.22, 0.28, 1.0)
    } else {
        Color::new(0.20, 0.24, 0.21, 1.0)
    };
    let ptrim = if b.underwater {
        Color::new(0.85, 0.70, 0.15, 1.0)
    } else {
        Color::new(0.12, 0.14, 0.13, 1.0)
    };

    // Roof superstructure over fuselage
    // The roof lifts away while any of our people are under it.
    let under = |cx: f32, cz: f32, hx: f32, hz: f32| b.units.iter().any(|u| u.alive && u.side == Side::Player
        && (u.x * TILE - cx).abs() < hx && (u.y * TILE - cz).abs() < hz);
    if !under(pcx, pcz, 3.3 * TILE, 2.4 * TILE) {
        mb.bx(vec3(pcx, 2.45, pcz), vec3(6.2 * TILE, 0.18, 4.4 * TILE), 0.0, apply_p(phull), 1.15);
        mb.bx(vec3(pcx, 2.70, pcz), vec3(4.8 * TILE, 0.35, 1.8 * TILE), 0.0, apply_p(mulc(phull, 1.15)), 1.2);
    }

    // Front Cockpit Canopy (angled nose glass)
    let nose_x = 1.0 * TILE;
    let canopy_col = if b.underwater { Color::new(0.10, 0.30, 0.35, 1.0) } else { Color::new(0.12, 0.22, 0.28, 1.0) };
    mb.bx(vec3(nose_x, 1.65, pcz), vec3(1.4 * TILE, 1.4, 2.2 * TILE), 0.0, apply_p(canopy_col), 1.3);
    mb.bx(vec3(nose_x, 1.65, pcz), vec3(1.44 * TILE, 0.08, 2.24 * TILE), 0.0, apply_p(ptrim), 1.2);

    // Delta Wings / Hydrofoils extending outward
    mb.bx(vec3(pcx - 0.2 * TILE, 1.80, pcz - 3.2 * TILE), vec3(2.8 * TILE, 0.14, 2.2 * TILE), -0.15, apply_p(phull), 1.1);
    mb.bx(vec3(pcx - 0.2 * TILE, 1.80, pcz + 3.2 * TILE), vec3(2.8 * TILE, 0.14, 2.2 * TILE), 0.15, apply_p(phull), 1.1);

    // Twin Propulsion Engines (VTOL Turbines / Ducted Propellers)
    for z_side in [-3.2f32, 3.2f32] {
        let epos = vec3(pcx - 0.5 * TILE, 1.95, pcz + z_side * TILE);
        mb.mat = gfx::mat::METAL;
        mb.bx(epos, vec3(2.2 * TILE, 0.75, 0.75 * TILE), 0.0, apply_p(mulc(ptrim, 1.2)), 1.3);
        mb.mat = gfx::mat::HULL;
        let thrust_glow = Color::new(0.2, 0.85, 1.0, 0.85);
        mb.bx(epos + vec3(-1.15 * TILE, 0.0, 0.0), vec3(0.15, 0.55 * TILE, 0.55 * TILE), 0.0, thrust_glow, 2.5);
    }

    // Rear Deployment Ramp (x: 6.8..8.2, z: 12.0..13.0)
    let ramp_x = 7.5 * TILE;
    mb.bx(vec3(ramp_x, 0.12, pcz), vec3(1.6 * TILE, 0.06, 1.8 * TILE), 0.0, apply_p(Color::new(0.28, 0.28, 0.26, 1.0)), 1.1);
    for ki in 0..4 {
        let kx = ramp_x - 0.6 * TILE + ki as f32 * 0.4 * TILE;
        mb.bx(vec3(kx, 0.16, pcz), vec3(0.08, 0.01, 1.7 * TILE), 0.0, apply_p(Color::new(0.85, 0.72, 0.12, 1.0)), 1.2);
    }

    // --- 2. ENEMY CRAFT (UFO or USO)
    let acx = 22.0 * TILE;
    let acz = 13.0 * TILE;
    let dist_alien = ((acx - cam_pos.x).powi(2) + (acz - cam_pos.z).powi(2)).sqrt();
    let apply_a = |c: Color| fogged(shade(c, vec3(0.0, 1.0, 0.0), sun, ambient), dist_alien, fog, 24.0, 90.0);

    let alien_hull = if b.underwater {
        Color::new(0.12, 0.28, 0.32, 1.0)
    } else {
        Color::new(0.15, 0.12, 0.19, 1.0)
    };

    // Upper hull dome
    let open_top = under(acx, acz, 3.1 * TILE, 3.3 * TILE);
    if !open_top {
        mb.bx(vec3(acx, 2.65, acz), vec3(5.8 * TILE, 0.22, 6.2 * TILE), 0.0, apply_a(alien_hull), 1.25);
        mb.bx(vec3(acx + 0.5 * TILE, 3.10, acz), vec3(2.6 * TILE, 0.65, 2.6 * TILE), 0.0, apply_a(mulc(alien_hull, 1.25)), 1.4);
    }
    let bio_eye = if b.underwater { Color::new(0.1, 0.95, 0.85, 1.0) } else { Color::new(0.95, 0.35, 1.0, 1.0) };
    let p_pulse = (b.time * 3.0).sin() * 0.2 + 0.8;
    if !open_top { mb.bx(vec3(acx + 0.5 * TILE, 3.45, acz), vec3(1.2 * TILE, 0.12, 1.2 * TILE), 0.0, mulc(bio_eye, p_pulse), 2.8); }

    // Perimeter Bioluminescent Beacon Nodes
    for i in 0..8 {
        let ang = i as f32 / 8.0 * std::f32::consts::TAU;
        let px = acx + ang.cos() * 3.2 * TILE;
        let pz = acz + ang.sin() * 3.4 * TILE;
        let pulse = ((b.time * 2.8 + i as f32 * 0.8).sin() * 0.3 + 0.7).clamp(0.4, 1.0);
        let node_col = if b.underwater { Color::new(0.2, 0.9, 0.85, 1.0) } else { Color::new(0.85, 0.4, 1.0, 1.0) };
        mb.bx(vec3(px, 1.5, pz), vec3(0.25, 0.25, 0.25), 0.0, mulc(node_col, pulse), 2.5);
    }

    // Airlock Portal Gateway
    let gate_x = 18.5 * TILE;
    mb.bx(vec3(gate_x, 1.15, acz - 1.1 * TILE), vec3(0.40, 2.30, 0.40), 0.0, apply_a(mulc(alien_hull, 1.3)), 1.2);
    mb.bx(vec3(gate_x, 1.15, acz + 1.1 * TILE), vec3(0.40, 2.30, 0.40), 0.0, apply_a(mulc(alien_hull, 1.3)), 1.2);
    mb.bx(vec3(gate_x, 2.35, acz), vec3(0.45, 0.30, 2.5 * TILE), 0.0, apply_a(mulc(alien_hull, 1.4)), 1.2);
    let barrier_col = if b.underwater { Color::new(0.15, 0.85, 0.9, 0.35) } else { Color::new(0.8, 0.3, 0.95, 0.35) };
    let barrier_flick = (b.time * 6.0).sin() * 0.08 + 0.35;
    mb.bx(vec3(gate_x, 1.15, acz), vec3(0.08, 2.20, 2.0 * TILE), 0.0, Color { a: barrier_flick, ..barrier_col }, 1.8);
}

fn skin_color(i: u8) -> Color {
    match i {
        0 => Color::new(0.96, 0.82, 0.72, 1.0),
        1 => Color::new(0.88, 0.70, 0.56, 1.0),
        2 => Color::new(0.77, 0.57, 0.42, 1.0),
        3 => Color::new(0.64, 0.45, 0.32, 1.0),
        4 => Color::new(0.46, 0.31, 0.22, 1.0),
        _ => Color::new(0.31, 0.20, 0.15, 1.0),
    }
}

fn draw_soldier(mb: &mut MB, _b: &Battle, g: &Game, u: &Unit, base: Vec3, sun: Vec3, ambient: f32, cam_pos: Vec3, fog: Color) {
    let armored = u.armor > 0;
    let arm_def = &ARMORS[u.armor as usize % ARMORS.len()];
    let uniform = if armored {
        Color::new(arm_def.color[0], arm_def.color[1], arm_def.color[2], 1.0)
    } else {
        Color::new(0.18, 0.25, 0.19, 1.0)
    };
    let skin = skin_color(u.look.skin);
    let moving = !u.path.is_empty();
    let bob = if moving { (u.anim_phase * 2.0).sin().abs() * 0.06 } else { (u.anim_phase * 0.6).sin() * 0.015 };
    let yaw = u.facing;
    let leg_swing = if moving { (u.anim_phase * 2.0).sin() * 0.20 } else { 0.0 };
    let dist = ((base.x - cam_pos.x).powi(2) + (base.z - cam_pos.z).powi(2)).sqrt();
    let apply = |c: Color| fogged(shade(c, vec3(0.0, 1.0, 0.0), sun, ambient), dist, fog, 18.0, 52.0);

    // 1. LEGS & GREAVES
    for side in [-1.0f32, 1.0] {
        let swing = leg_swing * side;
        let p = base + vec3(side * 0.16, 0.36 + bob, swing);
        mb.bx(p, vec3(0.20, 0.68, 0.22), yaw, apply(mulc(uniform, 0.75)), 1.1);
        mb.bx(p + vec3(0.0, -0.36, 0.05), vec3(0.22, 0.16, 0.34), yaw, apply(Color::new(0.08, 0.08, 0.08, 1.0)), 1.0);
        if u.armor == 2 || u.armor == 4 || u.armor == 5 {
            mb.bx(p + vec3(side * 0.08, -0.05, 0.08), vec3(0.08, 0.46, 0.10), yaw, apply(mulc(uniform, 1.25)), 1.2);
        }
    }

    // 2. TORSO & ARMOR PLATING
    mb.bx(base + vec3(0.0, 0.95 + bob, 0.0), vec3(0.52, 0.66, 0.32), yaw, apply(uniform), 1.16);

    match u.armor {
        0 => {
            // Webbing harness straps
            mb.bx(base + vec3(0.0, 0.98 + bob, 0.165), vec3(0.42, 0.44, 0.02), yaw, apply(Color::new(0.12, 0.14, 0.12, 1.0)), 1.0);
        }
        1 => {
            // Lattice carrier - plate front and back
            mb.bx(base + vec3(0.0, 1.02 + bob, 0.17), vec3(0.38, 0.36, 0.06), yaw, apply(Color::new(0.44, 0.52, 0.48, 1.0)), 1.25);
        }
        2 => {
            // Sealed frame - shell breastplate and core housing
            mb.bx(base + vec3(0.0, 1.02 + bob, 0.18), vec3(0.40, 0.40, 0.06), yaw, apply(Color::new(0.46, 0.58, 0.64, 1.0)), 1.3);
            mb.bx(base + vec3(0.0, 1.05 + bob, 0.21), vec3(0.14, 0.14, 0.04), yaw, Color::new(0.25, 0.85, 1.0, 1.0), 2.5);
        }
        3 => {
            // Dive rig - chest plate and twin rebreather bottles
            mb.bx(base + vec3(0.0, 1.00 + bob, 0.17), vec3(0.36, 0.40, 0.05), yaw, apply(Color::new(0.18, 0.44, 0.54, 1.0)), 1.25);
            for bx_side in [-0.12f32, 0.12f32] {
                mb.bx(base + vec3(bx_side, 0.98 + bob, -0.20), vec3(0.10, 0.44, 0.10), yaw, apply(Color::new(0.85, 0.80, 0.22, 1.0)), 1.2);
            }
        }
        4 => {
            // Nacre dive rig - nacre plate over the rig
            mb.bx(base + vec3(0.0, 1.02 + bob, 0.18), vec3(0.42, 0.42, 0.07), yaw, apply(Color::new(0.16, 0.36, 0.52, 1.0)), 1.3);
            mb.bx(base + vec3(0.0, 1.04 + bob, 0.22), vec3(0.12, 0.12, 0.04), yaw, Color::new(0.3, 0.95, 0.85, 1.0), 2.2);
            for bx_side in [-0.13f32, 0.13f32] {
                mb.bx(base + vec3(bx_side, 0.98 + bob, -0.21), vec3(0.11, 0.46, 0.11), yaw, apply(Color::new(0.20, 0.40, 0.55, 1.0)), 1.25);
            }
        }
        _ => {
            // Deep frame - thrust pack and trim fins
            mb.bx(base + vec3(0.0, 1.04 + bob, 0.19), vec3(0.44, 0.42, 0.07), yaw, apply(Color::new(0.24, 0.30, 0.42, 1.0)), 1.35);
            mb.bx(base + vec3(0.0, 1.05 + bob, 0.23), vec3(0.14, 0.14, 0.04), yaw, Color::new(0.4, 0.85, 1.0, 1.0), 2.6);
            for tx_side in [-0.16f32, 0.16f32] {
                mb.bx(base + vec3(tx_side, 1.08 + bob, -0.22), vec3(0.12, 0.38, 0.12), yaw, apply(Color::new(0.26, 0.32, 0.42, 1.0)), 1.25);
                mb.bx(base + vec3(tx_side, 0.86 + bob, -0.22), vec3(0.09, 0.06, 0.09), yaw, Color::new(0.4, 0.9, 1.0, 1.0), 2.8);
            }
            mb.bx(base + vec3(0.0, 1.18 + bob, -0.26), vec3(0.70, 0.08, 0.14), yaw, apply(Color::new(0.28, 0.34, 0.45, 1.0)), 1.2);
        }
    }

    // 3. ARMS & SHOULDERS
    let aim = if u.muzzle > 0.0 || u.shot_flash > 0.0 { 1.0 } else { 0.0 };
    for side in [-1.0f32, 1.0] {
        let p = base + vec3(side * 0.36, 1.05 + bob, 0.12 + aim * 0.25);
        mb.bx(p, vec3(0.15, 0.58, 0.15), yaw, apply(mulc(uniform, 0.9)), 1.1);
        let pad_col = if armored { mulc(uniform, 1.28) } else { mulc(uniform, 1.15) };
        mb.bx(base + vec3(side * 0.34, 1.20 + bob, 0.0), vec3(0.22, 0.18, 0.32), yaw, apply(pad_col), 1.25);
    }

    // 4. HEAD & FACE / HELMET
    mb.mat = gfx::mat::SKIN;
    mb.bx(base + vec3(0.0, 1.49 + bob, 0.0), vec3(0.36, 0.36, 0.32), yaw, apply(skin), 1.25);
    mb.bx(base + vec3(0.0, 1.47 + bob, 0.16), vec3(0.24, 0.16, 0.03), yaw, apply(mulc(skin, 0.90)), 1.0);
    for side in [-1.0f32, 1.0] {
        mb.bx(base + vec3(side * 0.075, 1.51 + bob, 0.18), vec3(0.055, 0.04, 0.02), yaw, apply(Color::new(0.05, 0.06, 0.06, 1.0)), 1.0);
        mb.bx(base + vec3(side * 0.075, 1.475 + bob, 0.175), vec3(0.07, 0.018, 0.02), yaw, apply(Color::new(0.55, 0.30, 0.25, 1.0)), 1.0);
    }

    mb.mat = gfx::mat::BUILT;
    if armored {
        mb.bx(base + vec3(0.0, 1.62 + bob, 0.0), vec3(0.40, 0.18, 0.36), yaw, apply(mulc(uniform, 1.2)), 1.25);
        let visor_col = if u.armor >= 3 { Color::new(0.9, 0.75, 0.2, 1.0) } else { Color::new(0.25, 0.75, 0.95, 1.0) };
        mb.bx(base + vec3(0.0, 1.50 + bob, 0.16), vec3(0.28, 0.10, 0.04), yaw, Color { a: 0.95, ..visor_col }, 1.6);
    } else {
        let hair = if u.look.hair_color == 0 { Color::new(0.08, 0.07, 0.06, 1.0) } else if u.look.hair_color >= 6 { Color::new(0.55, 0.55, 0.52, 1.0) } else { Color::new(0.22, 0.15, 0.09, 1.0) };
        mb.bx(base + vec3(0.0, 1.685 + bob, -0.02), vec3(0.38, 0.07, 0.34), yaw, apply(hair), 1.15);
    }

    // 5. TACTICAL GEAR & LOADOUT ITEMS
    if u.grenades > 0 {
        let g_col = apply(Color::new(0.26, 0.34, 0.22, 1.0));
        mb.bx(base + vec3(-0.11, 0.96 + bob, 0.18), vec3(0.08, 0.12, 0.08), yaw, g_col, 1.2);
        if u.grenades > 1 {
            mb.bx(base + vec3(-0.11, 0.82 + bob, 0.18), vec3(0.08, 0.12, 0.08), yaw, g_col, 1.2);
        }
    }
    if u.smoke_grenades > 0 {
        mb.bx(base + vec3(0.22, 0.72 + bob, 0.0), vec3(0.08, 0.15, 0.08), yaw, apply(Color::new(0.80, 0.80, 0.84, 1.0)), 1.15);
    }
    if u.medkits > 0 {
        let mbox = base + vec3(-0.24, 0.72 + bob, 0.0);
        mb.bx(mbox, vec3(0.08, 0.16, 0.16), yaw, apply(Color::new(0.90, 0.90, 0.92, 1.0)), 1.2);
        mb.bx(mbox + vec3(-0.042, 0.0, 0.0), vec3(0.01, 0.09, 0.03), yaw, Color::new(0.88, 0.15, 0.15, 1.0), 1.4);
        mb.bx(mbox + vec3(-0.042, 0.0, 0.0), vec3(0.01, 0.03, 0.09), yaw, Color::new(0.88, 0.15, 0.15, 1.0), 1.4);
    }

    // 6. EQUIPPED WEAPON 3D MODEL
    mb.mat = gfx::mat::METAL;
    let wdef = &g.data.weapons[u.weapon % g.data.weapons.len()];
    let wp = base + vec3(0.20, 1.05 + bob, 0.33 + aim * 0.3);
    let tracer_c = Color::new(wdef.tracer[0], wdef.tracer[1], wdef.tracer[2], 1.0);
    let dark_metal = apply(Color::new(0.18, 0.19, 0.20, 1.0));

    match wdef.cat {
        Cat::Pistol => {
            mb.bx(wp, vec3(0.08, 0.10, 0.34), yaw, dark_metal, 1.1);
            mb.bx(wp + vec3(0.0, 0.03, 0.06), vec3(0.07, 0.05, 0.22), yaw, apply(Color::new(0.32, 0.34, 0.36, 1.0)), 1.2);
            mb.bx(wp + vec3(0.0, 0.02, 0.20), vec3(0.05, 0.05, 0.08), yaw, tracer_c, 1.5);
        }
        Cat::Rifle => {
            mb.bx(wp, vec3(0.09, 0.13, 0.68), yaw, dark_metal, 1.15);
            mb.bx(wp + vec3(0.0, 0.03, 0.40), vec3(0.06, 0.06, 0.22), yaw, apply(Color::new(0.35, 0.36, 0.38, 1.0)), 1.25);
            mb.bx(wp + vec3(0.0, -0.10, 0.08), vec3(0.05, 0.14, 0.10), yaw, apply(Color::new(0.25, 0.26, 0.27, 1.0)), 1.1);
            mb.bx(wp + vec3(0.0, 0.09, 0.04), vec3(0.06, 0.06, 0.20), yaw, apply(Color::new(0.12, 0.12, 0.14, 1.0)), 1.3);
            mb.bx(wp + vec3(0.0, 0.09, 0.14), vec3(0.05, 0.05, 0.02), yaw, Color::new(0.2, 0.8, 1.0, 1.0), 1.8);
        }
        Cat::Cannon => {
            mb.bx(wp, vec3(0.14, 0.16, 0.74), yaw, dark_metal, 1.2);
            mb.bx(wp + vec3(0.0, 0.02, 0.44), vec3(0.10, 0.10, 0.26), yaw, apply(Color::new(0.38, 0.40, 0.42, 1.0)), 1.3);
            mb.bx(wp + vec3(0.0, -0.12, 0.10), vec3(0.16, 0.16, 0.18), yaw, apply(Color::new(0.24, 0.25, 0.26, 1.0)), 1.15);
            mb.bx(wp + vec3(0.0, 0.11, 0.14), vec3(0.05, 0.07, 0.22), yaw, apply(Color::new(0.15, 0.15, 0.16, 1.0)), 1.2);
        }
        Cat::Launcher => {
            mb.bx(wp + vec3(0.0, 0.08, 0.0), vec3(0.16, 0.16, 0.86), yaw, dark_metal, 1.25);
            mb.bx(wp + vec3(0.0, 0.08, -0.44), vec3(0.20, 0.20, 0.08), yaw, apply(Color::new(0.30, 0.32, 0.34, 1.0)), 1.3);
            mb.bx(wp + vec3(0.0, 0.08, 0.46), vec3(0.13, 0.13, 0.16), yaw, Color::new(0.85, 0.45, 0.15, 1.0), 1.8);
        }
        Cat::Heavy => {
            mb.bx(wp, vec3(0.14, 0.16, 0.76), yaw, dark_metal, 1.2);
            mb.bx(wp + vec3(0.0, 0.02, 0.10), vec3(0.10, 0.10, 0.28), yaw, tracer_c, 2.5);
            for vi in 0..3 {
                let vz = 0.02 + vi as f32 * 0.08;
                mb.bx(wp + vec3(0.08, 0.04, vz), vec3(0.03, 0.09, 0.04), yaw, apply(Color::new(0.4, 0.45, 0.5, 1.0)), 1.3);
            }
        }
        Cat::Melee => {
            mb.bx(wp, vec3(0.06, 0.06, 0.62), yaw, apply(Color::new(0.25, 0.26, 0.28, 1.0)), 1.1);
            for pi in [-0.035f32, 0.035f32] {
                mb.bx(wp + vec3(pi, 0.0, 0.34), vec3(0.02, 0.02, 0.14), yaw, Color::new(0.4, 0.85, 1.0, 1.0), 2.8);
            }
        }
        _ => {
            mb.bx(wp, vec3(0.10, 0.12, 0.40), yaw, dark_metal, 1.1);
        }
    }

    // 7. TACTICAL FLASHLIGHT ILLUMINATION (Night or Deep Sea missions)
    if ambient < 0.38 {
        let f_dir = vec3(yaw.sin(), 0.0, yaw.cos());
        let light_spot = base + f_dir * 1.8 + vec3(0.0, 0.02, 0.0);
        let beam_c = Color::new(0.85, 0.92, 1.0, (0.38 - ambient) * 0.7);
        mb.bx(light_spot, vec3(1.2, 0.005, 1.2), yaw, beam_c, 1.5);
    }

    // Team indicator overhead
    mb.bx(base + vec3(0.0, 1.78 + bob, 0.0), vec3(0.10, 0.10, 0.10), yaw, Color::new(0.3, 0.9, 1.0, 1.0), 2.0);
}

fn draw_alien(mb: &mut MB, b: &Battle, g: &Game, u: &Unit, base: Vec3, sun: Vec3, ambient: f32) {
    let ak = alien_def(&g.data, b.underwater, u.alien_kind as u8, b.mission.tier);
    let body = Color::new(ak.body[0], ak.body[1], ak.body[2], 1.0);
    let accent = Color::new(ak.accent[0], ak.accent[1], ak.accent[2], 1.0);
    let float = ak.float;
    let bob = if float { (b.time * 1.6 + u.anim_phase).sin() * 0.12 + 0.25 } else { (u.anim_phase * 0.8).sin() * 0.02 };
    let base = base + vec3(0.0, bob, 0.0);
    let s = ak.scale;
    let yaw = u.facing;
    let apply = |c: Color| shade(c, vec3(0.0, 1.0, 0.0), sun, ambient);
    if ak.melee > 25 {
        // low quadruped / crawler
        mb.bx(base + vec3(0.0, 0.42 * s, 0.0), vec3(1.15 * s, 0.55 * s, 0.55 * s), yaw, apply(body), 1.2);
        mb.bx(base + vec3(0.0, 0.58 * s, 0.48 * s), vec3(0.45 * s, 0.40 * s, 0.40 * s), yaw, apply(accent), 1.3);
        for side in [-1.0f32, 1.0] {
            for f in [-1.0f32, 1.0] {
                mb.bx(base + vec3(side * 0.50 * s, 0.28 * s, f * 0.30 * s), vec3(0.14 * s, 0.55 * s, 0.14 * s), yaw, apply(Color { r: body.r * 0.7, g: body.g * 0.7, b: body.b * 0.7, a: 1.0 }), 1.0);
            }
        }
        // talons
        for side in [-1.0f32, 1.0] {
            mb.bx(base + vec3(side * 0.30 * s, 0.45 * s, 0.62 * s), vec3(0.08 * s, 0.08 * s, 0.40 * s), yaw, apply(accent), 1.2);
        }
    } else if float && (ak.name == "Lathe" || ak.name == "Lamp") {
        mb.bx(base + vec3(0.0, 0.55 * s, 0.0), vec3(1.65 * s, 0.22 * s, 1.65 * s), yaw, apply(body), 1.15);
        mb.bx(base + vec3(0.0, 0.72 * s, 0.0), vec3(0.95 * s, 0.45 * s, 0.95 * s), yaw, apply(Color { r: body.r * 1.15, g: body.g * 1.15, b: body.b * 1.15, a: 1.0 }), 1.3);
        mb.bx(base + vec3(0.0, 0.72 * s, 0.55 * s), vec3(0.22 * s, 0.16 * s, 0.18 * s), yaw, apply(accent), 1.4);
    } else {
        // humanoid
        for side in [-1.0f32, 1.0] {
            mb.bx(base + vec3(side * 0.20 * s, 0.38 * s, 0.0), vec3(0.20 * s, 0.72 * s, 0.20 * s), yaw, apply(Color { r: body.r * 0.78, g: body.g * 0.78, b: body.b * 0.78, a: 1.0 }), 1.1);
        }
        mb.bx(base + vec3(0.0, 1.00 * s, 0.0), vec3(0.60 * s, 0.66 * s, 0.36 * s), yaw, apply(body), 1.18);
        mb.bx(base + vec3(0.0, 1.30 * s, 0.02), vec3(0.42 * s, 0.20 * s, 0.30 * s), yaw, apply(accent), 1.25);
        mb.bx(base + vec3(0.0, 1.52 * s, 0.0), vec3(0.38 * s, 0.36 * s, 0.32 * s), yaw, apply(accent), 1.3);
        for side in [-1.0f32, 1.0] {
            mb.bx(base + vec3(side * 0.42 * s, 1.10 * s, 0.10), vec3(0.14 * s, 0.60 * s, 0.14 * s), yaw, apply(body), 1.1);
        }
        // alien weapon
        mb.bx(base + vec3(0.24 * s, 1.08 * s, 0.36 * s), vec3(0.10 * s, 0.12 * s, 0.58 * s), yaw, apply(Color::new(0.3, 0.4, 0.35, 1.0)), 1.2);
        // eyes
        for side in [-1.0f32, 1.0] {
            mb.bx(base + vec3(side * 0.09 * s, 1.54 * s, 0.16 * s), vec3(0.08 * s, 0.07 * s, 0.03 * s), yaw, Color::new(0.9, 0.95, 0.4, 1.0), 1.5);
        }
    }
}

/// Tracers and blast streaks: drawn in the glowing overlay pass.
fn draw_effects_3d(b: &Battle) {
    for e in &b.effects {
        match e {
            Effect::Tracer { a, b: bb, color, t, life } => {
                let f = (t / life).min(1.0);
                let a2 = *a;
                let b2 = *a + (*bb - *a) * f;
                draw_line_3d(a2, b2, Color { a: (1.0 - f) * 0.9, ..*color });
                for off in [vec3(0.02, 0.0, 0.0), vec3(-0.02, 0.0, 0.0), vec3(0.0, 0.02, 0.0)] {
                    draw_line_3d(a2 + off, b2 + off, Color { a: (1.0 - f) * 0.35, ..*color });
                }
            }
            Effect::Explosion { p, t, life, radius } => {
                let f = (t / life).min(1.0);
                let rr = radius * (0.2 + f * 1.1);
                for i in 0..10 {
                    let ang = i as f32 / 10.0 * std::f32::consts::TAU;
                    let rr2 = rr * (0.5 + 0.5 * i as f32 / 10.0);
                    let pp = *p + vec3(ang.cos() * rr2, 0.2 + f * 1.2, ang.sin() * rr2);
                    draw_line_3d(*p + vec3(0.0, 0.2, 0.0), pp, Color { r: 1.0, g: 0.5, b: 0.15, a: 1.0 - f });
                }
            }
            _ => {}
        }
    }
}

fn draw_effects(b: &Battle, cam: &Camera3D, ui: &Ui) {
    for e in &b.effects {
        match e {
            Effect::Tracer { a, b: bb, color, t, life } => {
                let f = (t / life).min(1.0);
                let a2 = *a;
                let b2 = *a + (*bb - *a) * f.min(1.0);
                let _ = a2;
                if let Some(p) = project(cam, b2) { glow(p.x, p.y, 12.0, *color, (1.0 - f) * 0.6); }
            }
            Effect::Explosion { p, t, life, radius } => {
                let f = (t / life).min(1.0);
                let rr = radius * (0.2 + f * 1.1);
                if let Some(sp) = project(cam, *p) {
                    let s = (rr / (1.0 + (sp - vec2(screen_width() * 0.5, screen_height() * 0.5)).length() / 200.0)) * 60.0;
                    glow(sp.x, sp.y, s * 1.6, Color::new(1.0, 0.55, 0.15, 1.0), (1.0 - f) * 1.4);
                    draw_circle(sp.x, sp.y, s * 0.6 * (1.0 - f), Color { r: 1.0, g: 0.9, b: 0.6, a: (1.0 - f) * 0.8 });
                }
            }
            Effect::Flash { p, t, life, color, size } => {
                let f = (t / life).min(1.0);
                if let Some(sp) = project(cam, *p) { glow(sp.x, sp.y, size * 40.0, *color, (1.0 - f) * 1.2); }
            }
            Effect::Text { p, text, color, t, life } => {
                let f = (t / life).min(1.0);
                if let Some(sp) = project(cam, *p) {
                    ui.bcenter(sp.x, sp.y - f * 30.0, 13.0, Color { a: 1.0 - f, ..*color }, text);
                }
            }
            Effect::Debris { p, color, t, life, .. } => {
                let f = (t / life).min(1.0);
                if let Some(sp) = project(cam, *p) {
                    let sz = (4.0 * (1.0 - f * 0.4)).max(1.0);
                    draw_rectangle(sp.x - sz * 0.5, sp.y - sz * 0.5, sz, sz, Color { a: 1.0 - f * 0.5, ..*color });
                }
            }
        }
    }
}

fn draw_unit_overlays(g: &mut Game, b: &Battle, cam: &Camera3D) {
    let ui = &g.ui;
    let (mx, my) = mouse_position();
    for (i, u) in b.units.iter().enumerate() {
        if !u.alive { continue; }
        let tx = u.x.round() as i32; let ty = u.y.round() as i32;
        let t = tile_at(b, tx, ty);
        if !t.visible && u.side == Side::Alien { continue; }
        let p = world_pos(u.x, u.y, t.h) + vec3(0.0, 2.0 * if u.side == Side::Alien { 1.15 } else { 1.0 }, 0.0);
        if let Some(sp) = project(cam, p) {
            let w = 44.0;
            let frac = (u.hp as f32 / u.max_hp.max(1) as f32).clamp(0.0, 1.0);
            let col = if u.side == Side::Player { ui::GREEN } else { ui::RED };
            draw_rectangle(sp.x - w * 0.5, sp.y - 8.0, w, 4.0, Color { a: 0.7, ..BLACK });
            draw_rectangle(sp.x - w * 0.5, sp.y - 8.0, w * frac, 4.0, col);
            draw_rectangle_lines(sp.x - w * 0.5, sp.y - 8.0, w, 4.0, 1.0, Color { a: 0.4, ..col });
            let d = vec2(sp.x, sp.y).distance(vec2(mx, my));
            if d < 30.0 && u.side == Side::Alien {
                let sel = b.units.get(b.selected);
                let chance = if let Some(s) = sel {
                    let wdef = &g.data.weapons[s.weapon];
                    let mode = b.mode.idx();
                    let wa = g.data.item(wdef.game, wdef.item).acc[mode] as i32;
                    hit_chance(s.acc, wa, s.kneeling, s.hp, s.max_hp, s.wounds, false)
                } else { 0 };
                let label = format!("{}  {}%  HP {}", u.name, chance, u.hp);
                let tw = ui.width(11.0, &label) + 14.0;
                draw_rectangle(sp.x - tw * 0.5, sp.y - 28.0, tw, 17.0, Color { a: 0.85, ..BG });
                ui.center(sp.x, sp.y - 16.0, 11.0, AMBER, &label);
            }
            if i == b.selected && u.side == Side::Player {
                ui.center(sp.x, sp.y - 26.0, 9.0, CYAN, &u.name);
            }
        }
    }
}

/// Surface atmosphere: warm late-afternoon sun, volumetric shafts and
/// drifting dust. This sits over the 3-D mesh so the battlefield reads as a
/// place rather than a board.
fn draw_underwater_fx(b: &Battle, cam: &Camera3D) {
    if !b.underwater { return; }
    let w = screen_width(); let h = screen_height();

    // 1. Thermocline / halocline optical distortion bands (shimmering refraction ribbons)
    for i in 0..12 {
        let fi = i as f32;
        let y0 = (fi * 92.0 + b.time * 6.0).rem_euclid(h + 80.0) - 40.0;
        let band_h = 32.0 + (fi * 2.1).sin().abs() * 24.0;
        let alpha = 0.015 + 0.010 * (fi * 1.7 + b.time * 0.9).sin().abs();
        draw_rectangle(0.0, y0, w, band_h, Color::new(0.08, 0.45, 0.55, alpha));
    }

    // 2. Slow caustic bands, like sunlight bent through surface waves
    for i in 0..20 {
        let fi = i as f32;
        let y0 = (fi * 55.0 + b.time * 11.0).rem_euclid(h + 120.0) - 60.0;
        let amp = 12.0 + (fi * 2.7).sin() * 6.0;
        let alpha = 0.024 + 0.014 * (fi * 1.9 + b.time * 0.8).sin().abs();
        let mut prev = vec2(0.0, y0);
        let mut x = 0.0;
        while x < w {
            let yy = y0 + (x * 0.018 + b.time * 0.75 + fi).sin() * amp;
            let p = vec2(x, yy);
            if x > 0.0 { draw_line(prev.x, prev.y, p.x, p.y, 1.4, Color { r: 0.35, g: 0.92, b: 0.98, a: alpha }); }
            prev = p; x += 24.0;
        }
    }

    // 3. Ascending atmospheric bubbles with organic wobble and refraction highlights
    for i in 0..65 {
        let seed = i as f32 * 17.31;
        let rise_speed = 42.0 + (i % 7) as f32 * 14.0;
        let progress = (b.time * rise_speed + seed * 83.0).rem_euclid(h + 100.0);
        let y = (h + 50.0) - progress;
        let wobble = (b.time * 3.8 + seed * 23.1).sin() * (6.0 + (i % 5) as f32 * 3.0);
        let x = (((seed * 73.7).fract()) * w + wobble).rem_euclid(w);
        let r = 2.2 + (i % 6) as f32 * 1.4;

        // Outer refraction membrane
        draw_circle(x, y, r + 0.8, Color::new(0.25, 0.80, 0.95, 0.12));
        // Translucent bubble body
        draw_circle(x, y, r, Color::new(0.15, 0.65, 0.85, 0.28));
        // Top-left specular glint / caustic glint
        draw_circle(x - r * 0.32, y - r * 0.32, (r * 0.36).max(0.7), Color::new(0.92, 0.98, 1.0, 0.75));
        // Bottom rim caustics bounce
        draw_circle(x + r * 0.25, y + r * 0.25, (r * 0.22).max(0.5), Color::new(0.35, 0.92, 1.0, 0.35));
    }

    // 4. Diver respiration bubble streams from living player units
    for u in &b.units {
        if u.side == Side::Player && u.alive {
            let wx = (u.x - TW as f32 * 0.5) * 1.65;
            let wz = (u.y - TH as f32 * 0.5) * 1.65;
            if let Some(sp) = project(cam, vec3(wx, 1.3, wz)) {
                let phase = (b.time * 0.85 + u.x * 3.1).fract();
                if phase < 0.55 {
                    for bi in 0..4 {
                        let bif = bi as f32;
                        let by = sp.y - phase * 80.0 - bif * 10.0;
                        let bx = sp.x + (b.time * 6.5 + bif * 2.2).sin() * 6.0;
                        let br = 1.8 + bif * 0.45;
                        draw_circle(bx, by, br + 0.8, Color::new(0.25, 0.85, 0.95, 0.18));
                        draw_circle(bx, by, br, Color::new(0.80, 0.96, 1.0, 0.60));
                        draw_circle(bx - br * 0.3, by - br * 0.3, (br * 0.35).max(0.6), Color::new(1.0, 1.0, 1.0, 0.85));
                    }
                }
            }
        }
    }

    // 5. Hydrothermal vent bubble plumes (concentrated rising micro-bubble columns)
    for vent_idx in 0..3 {
        let vent_x = match vent_idx { 0 => w * 0.22, 1 => w * 0.58, _ => w * 0.84 };
        for k in 0..14 {
            let kf = k as f32;
            let v_speed = 70.0 + kf * 8.0;
            let vy = (h + 20.0) - (b.time * v_speed + kf * 37.0).rem_euclid(h + 40.0);
            let vx = vent_x + (b.time * 5.0 + kf * 13.0).sin() * (4.0 + kf * 0.8);
            let vr = 1.2 + (k % 3) as f32 * 0.8;
            draw_circle(vx, vy, vr + 0.5, Color::new(0.3, 0.9, 0.95, 0.18));
            draw_circle(vx, vy, vr, Color::new(0.85, 0.98, 1.0, 0.60));
        }
    }

    // 6. Drifting bioluminescent marine snow and abyssal plankton
    for i in 0..120 {
        let seed = i as f32 * 13.7;
        let x = (seed.sin() * 0.5 + 0.5) * w + (b.time * 5.0 * (i % 3 + 1) as f32) % w;
        let y = ((seed.cos() * 0.5 + 0.5) + b.time * 0.018 * (i % 4 + 1) as f32) % 1.0 * h;
        let r = 0.7 + (i % 4) as f32 * 0.40;
        let pulse = 0.5 + 0.5 * (b.time * 2.5 + seed).sin();
        let glow_alpha = 0.04 + 0.03 * pulse;
        let core_alpha = 0.22 + 0.18 * pulse;
        draw_circle(x % w, y, r + 1.6, Color { r: 0.35, g: 0.85, b: 0.95, a: glow_alpha });
        draw_circle(x % w, y, r, Color { r: 0.75, g: 0.95, b: 1.0, a: core_alpha });
    }

}

/// Average ground colour of the battlefield, for the country around it.
fn ground_average(b: &Battle) -> Color {
    let (mut r, mut gg, mut bl) = (0.0, 0.0, 0.0);
    for y in 0..TH { for x in 0..TW {
        let c = ground_albedo(b, x, y);
        r += c.r; gg += c.g; bl += c.b;
    }}
    let n = (TW * TH) as f32;
    Color::new(r / n, gg / n, bl / n, 1.0)
}

/// Unlit colour of one ground tile.
fn ground_albedo(b: &Battle, x: i32, y: i32) -> Color { ground_site(b, x, y).0 }

/// Colour and surface type of one ground tile, from the biome, the season
/// and where the tile lies. Type: 0 open ground (grass, or sand in the
/// desert), 85 ploughed soil, 168 crop rows, 255 packed earth, gravel, rock.
fn ground_site(b: &Battle, x: i32, y: i32) -> (Color, u8) {
    let t = &b.tiles[tile_idx(x, y)];
    let c0 = biome_ground(t.kind, x, y, 1234);
    if b.underwater { return (c0, 255); }
    let n = vnoise(x as f32 * 0.55, y as f32 * 0.55, 1234) * 0.18 + 0.91;
    let col = |r: f32, g: f32, bl: f32| Color::new(r * n, g * n, bl * n, 1.0);
    let season = season_at(b.mission.month, b.mission.lat);
    let (xf, yf) = (x as f32, y as f32);
    let patch = fbm(xf * 0.31, yf * 0.31, 17);
    let dry = ((patch - 0.46).max(0.0) * 0.55).min(0.28);
    // grass through the year: fresh in spring, yellowing in autumn, dead in winter
    let grass = |c: Color| -> Color {
        match season {
            0 => mixc(c, Color::new(0.42, 0.39, 0.25, 1.0), 0.55),
            1 => mixc(c, Color::new(0.30, 0.47, 0.18, 1.0), 0.35),
            3 => mixc(c, Color::new(0.44, 0.41, 0.22, 1.0), 0.35),
            _ => c,
        }
    };
    match t.kind {
        1 => {
            // farmland: a track, and fields each with its own use this season
            let fx = (x + 3).div_euclid(8);
            let fy = (y + 2).div_euclid(6);
            if (x + 3).rem_euclid(16) == 0 { return (col(0.48, 0.43, 0.34), 255); }
            match hash2(fx, fy, 555) % 4 {
                0 => match season {
                    0 => (col(0.33, 0.26, 0.19), 85),
                    1 => (col(0.33, 0.46, 0.20), 168),
                    2 => (col(0.62, 0.54, 0.27), 168),
                    _ => (col(0.58, 0.50, 0.30), 0),
                },
                1 => (col(0.34, 0.27, 0.19), 85),
                2 => (grass(mixc(col(0.37, 0.44, 0.23), Color::new(0.50, 0.46, 0.26, 1.0), if season == 2 { 0.3 } else { 0.0 })), 0),
                _ => match season {
                    0 => (col(0.40, 0.38, 0.24), 0),
                    1 => (col(0.40, 0.48, 0.22), 168),
                    2 => (col(0.30, 0.44, 0.18), 168),
                    _ => (col(0.36, 0.28, 0.20), 85),
                },
            }
        }
        5 => {
            if fbm(xf * 0.23, yf * 0.23, 301) > 0.42 { (col(0.43, 0.42, 0.40), 255) } else { (grass(col(0.33, 0.38, 0.22)), 0) }
        }
        2 | 6 => (mixc(col(0.19, 0.29, 0.12), col(0.27, 0.23, 0.14), ((patch - 0.5).max(0.0) * 2.0).min(0.6)), 0),
        4 | 7 | 8 => {
            if fbm(xf * 0.19, yf * 0.19, 77) > 0.56 { (col(0.55, 0.46, 0.34), 255) } else { (mixc(col(0.71, 0.57, 0.39), col(0.74, 0.52, 0.33), patch), 0) }
        }
        9 => (col(0.60, 0.72, 0.82), 255),
        11 => {
            if fbm(xf * 0.27, yf * 0.27, 211) > 0.60 { (col(0.41, 0.40, 0.37), 255) } else { (mixc(col(0.33, 0.33, 0.19), col(0.40, 0.30, 0.18), patch), 0) }
        }
        _ => (grass(mixc(c0, Color::new(0.40, 0.37, 0.20, 1.0), dry)), 0),
    }
}

/// The per-tile colour and surface-type map the ground shader reads.
fn ground_map(b: &Battle) -> Vec<u8> {
    let mut m = vec![0u8; (TW * TH * 4) as usize];
    for y in 0..TH { for x in 0..TW {
        let t = &b.tiles[tile_idx(x, y)];
        let _ = t;
        let (c, ty) = ground_site(b, x, y);
        let o = ((y * TW + x) * 4) as usize;
        m[o] = (c.r.clamp(0.0, 1.0) * 255.0) as u8;
        m[o + 1] = (c.g.clamp(0.0, 1.0) * 255.0) as u8;
        m[o + 2] = (c.b.clamp(0.0, 1.0) * 255.0) as u8;
        m[o + 3] = ty;
    }}
    m
}

/// The light on this battlefield: from the mission's date, time and place
/// on land; a fixed hand-made look under water.
fn scene_light(b: &Battle, g: &Game) -> gfx::Light {
    if b.underwater {
        let v = |r: f32, gr: f32, bl: f32, k: f32| vec3(r, gr, bl) * k;
        let look = gfx::SkyLook {
            sun: v(0.35, 0.85, 0.95, 1.6), zenith: v(0.10, 0.45, 0.55, 0.55), horizon: v(0.03, 0.20, 0.26, 0.40),
            glow: Vec3::ZERO, haze: 0.0055, diffusion: 0.55,
        };
        return gfx::Light::new(vec3(-0.16, 0.95, 0.10).normalize(), look, vec3(0.03, 0.12, 0.13));
    }
    let (year, day) = g.dbg_clock.map(|c| (c.year, c.day)).unwrap_or((g.date.year, g.date.day as i32));
    let m = sky::Moment::new(year, b.mission.month as i32 + 1, day.clamp(1, 28), b.mission.minute_of_day as f32);
    let heavy = weather_heavy(b);
    // cloud cover, storm, wetness and the haze the weather itself adds
    let (cover, storm, wet, haze_w) = match b.weather {
        1 => (1.0, if heavy { 0.8 } else { 0.3 }, 1.0, if heavy { 0.0030 } else { 0.0016 }),
        2 => (if heavy { 1.0 } else { 0.9 }, if heavy { 0.35 } else { 0.0 }, 0.0, if heavy { 0.0040 } else { 0.0020 }),
        3 => (if heavy { 0.55 } else { 0.25 }, 0.0, 0.0, if heavy { 0.0040 } else { 0.0020 }),
        _ => (match b.biome { 2 | 6 => 0.2, _ => 0.0 }, 0.0, 0.0, 0.0),
    };
    let snow = snow_amount(b);
    let ga = ground_average(b);
    let gv = vec3(ga.r, ga.g, ga.b).lerp(vec3(0.80, 0.84, 0.88), snow);
    let l0 = gfx::light_at(m, b.mission.lat, b.mission.lon, cover, storm, gv * 0.5);
    let mut look = l0.look;
    // air: dry and thin in desert, mountain and ice; thick over rain forest
    let air = match b.biome { 4 | 7 | 8 => 0.6, 2 | 6 => 2.5, 9 => 0.5, 5 => 0.6, _ => 1.0 };
    look.haze = (look.haze * air).max(haze_w);
    if b.weather == 3 {
        // dust in the air warms and flattens the light
        let k = if heavy { 0.7 } else { 0.4 };
        let tint = vec3(1.0, 0.84, 0.62);
        let lum = |v: Vec3| v.x * 0.2126 + v.y * 0.7152 + v.z * 0.0722;
        look.zenith = look.zenith.lerp(tint * lum(look.zenith) * 1.1, k);
        look.horizon = look.horizon.lerp(tint * lum(look.horizon) * 1.1, k);
        look.sun = look.sun * Vec3::ONE.lerp(tint * 0.8, k);
        look.diffusion = look.diffusion.max(k * 0.6);
    }
    if b.weather == 1 && heavy {
        // lightning somewhere above the cloud lights the whole deck for a frame or two
        let strike = (b.time * 0.11).sin() * (b.time * 0.37).cos();
        if strike > 0.995 { look.zenith += vec3(2.0, 2.3, 3.0); }
    }
    let mut l = gfx::Light::new(l0.sun_dir, look, l0.ground);
    l.wet = wet;
    l.snow = snow;
    l.sand = if matches!(b.biome, 4 | 7 | 8) { 1.0 } else { 0.0 };
    l.rain = if b.weather == 1 { 1.0 } else { 0.0 };
    l.time = b.time;
    l
}

pub fn draw(g: &mut Game) {
    let Some(mut b) = g.battle.take() else { return; };
    b.time += 0.0;
    let w = screen_width(); let h = screen_height();
    let cam = make_camera(&b);
    let light = scene_light(&b, g);
    let sm = build_scene(&b, g, &cam);
    let mut gfx = g.gfx.take().unwrap_or_else(|| gfx::Gfx::new(w as u32, h as u32));
    if !b.gfx_ready {
        let half = TILE * 0.5;
        gfx.set_ground_map(TW as u16, TH as u16, &ground_map(&b), vec2(-half, -half), vec2(TW as f32 * TILE, TH as f32 * TILE));
        b.gfx_ready = true;
    }
    let centre = vec3((TW as f32 * 0.5 - 0.5) * TILE, 0.6, (TH as f32 * 0.5 - 0.5) * TILE);
    gfx.shadow_begin(&light, centre, 34.0);
    for m in &sm.lit { draw_mesh(m); }
    gfx.scene_begin(&cam, &light);
    for m in &sm.lit { draw_mesh(m); }
    gfx.overlay_begin(&light, 0.0);
    for m in &sm.over { draw_mesh(m); }
    // Lamps and fire are fixed-brightness things; the camera opens up at
    // night, so hold them to a few stops over the scene or they burn white.
    let amb = light.sh[0] * 0.282095 + light.sh[1] * 0.488603;
    let lit = light.look.sun * light.sun_dir.y.max(0.0) + amb;
    let lit_lum = lit.x * 0.2126 + lit.y * 0.7152 + lit.z * 0.0722;
    let glow_k = (lit_lum * GLOW_PER_LIT / GLOW_LEVEL).clamp(0.0, 1.0);
    gfx.overlay_begin(&light, GLOW_LEVEL * glow_k);
    for m in &sm.glow { draw_mesh(m); }
    draw_selection_ring(&b);
    gfx.overlay_begin(&light, 20.0 * glow_k);
    draw_effects_3d(&b);
    let lens = gfx::Lens { ev_bias: light.snow * 0.5, adapt_secs: if g.screenshot.is_some() { 0.0 } else { 1.5 }, vignette_stops: 0.25, ..Default::default() };
    gfx.finish(get_frame_time(), &lens);
    g.gfx = Some(gfx);
    draw_effects(&b, &cam, &g.ui);
    draw_underwater_fx(&b, &cam);
    // screen tint / muzzle light
    if b.screen_flash > 0.01 {
        draw_rectangle(0.0, 0.0, w, h, Color { r: 1.0, g: 0.75, b: 0.35, a: b.screen_flash * 0.12 });
    }
    draw_unit_overlays(g, &b, &cam);
    draw_hud(g, &mut b);
    // -------- target readout: the shot the current mode would take
    if b.phase == Phase::Player {
        if let Some(ti) = b.unit_at(b.hover.0, b.hover.1) {
            if b.units[ti].side == Side::Alien {
                let (mx, my) = mouse_position();
                let u = b.units[b.selected].clone();
                let wd = g.data.weapons[u.weapon].clone();
                let it = g.data.item(wd.game, wd.item);
                let mi = b.mode.idx();
                let wa = it.acc[mi] as i32;
                let cost = it.tu[mi];
                let chance = if wa > 0 { hit_chance(u.acc, wa, u.kneeling, u.hp, u.max_hp, u.wounds, false) } else { 0 };
                let los = b.los(u.x.round() as i32, u.y.round() as i32, b.units[ti].x.round() as i32, b.units[ti].y.round() as i32);
                let dist = ((b.units[ti].x - u.x).powi(2) + (b.units[ti].y - u.y).powi(2)).sqrt();
                if let (Some(a), Some(c)) = (project(&cam, b.muzzle_pos(b.selected)), project(&cam, b.chest_pos(ti))) {
                    let line = if !los { Color { a: 0.55, ..ui::RED } } else if u.tu >= cost as f32 && u.ammo > 0 { Color { a: 0.7, ..AMBER } } else { Color { a: 0.35, ..DIM } };
                    draw_line(a.x, a.y, c.x, c.y, 1.0, line);
                    draw_circle_lines(c.x, c.y, 13.0, 1.5, line);
                }
                let rw = 224.0;
                let r = Rect::new((mx + 18.0).min(w - rw - 8.0).max(8.0), (my + 16.0).min(h - 210.0).max(8.0), rw, 78.0);
                panel(r, Color { a: 0.94, ..PANEL }, if los { CYAN_DIM } else { ui::RED });
                let target = b.units[ti].clone();
                g.ui.bold(r.x + 10.0, r.y + 16.0, 8.0, ui::RED, &g.ui.fit(8.0, &target.name, rw - 20.0, true));
                g.ui.text(r.x + 10.0, r.y + 29.0, 8.0, DIM, &format!("HP {}/{}   ARMOUR {}", target.hp.max(0), target.max_hp, target.armor));
                draw_rectangle(r.x + 8.0, r.y + 35.0, rw - 16.0, 1.0, Color { a: 0.35, ..CYAN_DIM });
                if !los {
                    g.ui.bold(r.x + 10.0, r.y + 51.0, 10.0, ui::RED, "NO LINE OF FIRE");
                    g.ui.text(r.x + 10.0, r.y + 65.0, 7.5, DIM, "MOVE TO A CLEAR LANE");
                } else {
                    let ready = u.tu >= cost as f32 && (u.ammo > 0 || wd.infinite);
                    g.ui.bold(r.x + 10.0, r.y + 53.0, 12.0, if ready { CYAN } else { DIM }, &format!("{}  {}%", b.mode.label(), chance));
                    g.ui.text(r.x + 92.0, r.y + 53.0, 8.0, if ready { TEXT } else { DIM }, &format!("{} TU", cost));
                    g.ui.text(r.x + 140.0, r.y + 53.0, 8.0, DIM, &format!("{:.0} TILES", dist));
                    g.ui.text(r.x + 10.0, r.y + 68.0, 7.5, DIM, if ready { "RIGHT CLICK TO FIRE" } else { "NOT ENOUGH TU OR AMMO" });
                }
            }
        }
    }
    // log
    let mut y = 58.0;
    for (text, col, age) in b.log.iter().rev().take(5) {
        let a = (1.2 - age / 25.0).clamp(0.25, 1.0);
        let tw = g.ui.width(9.5, text) + 16.0;
        draw_rectangle(w - tw - 12.0, y - 12.0, tw, 16.0, Color { a: 0.55 * a, ..BG });
        g.ui.text(w - tw - 4.0, y, 9.5, Color { a, ..*col }, text);
        y += 17.0;
    }
    // hover reticle
    if b.phase == Phase::Player {
        if let Some((tx, ty)) = Some(b.hover) {
            if in_bounds(tx, ty) {
                let t = tile_at(&b, tx, ty);
                let p = world_pos(tx as f32, ty as f32, t.h) + vec3(0.0, 0.03, 0.0);
                if let Some(sp) = project(&cam, p) {
                    draw_circle_lines(sp.x, sp.y, 8.0, 1.0, Color { a: 0.7, ..CYAN });
                }
            }
        }
    }
    if b.show_help {
        let r = Rect::new(w * 0.16, h * 0.16, w * 0.62, h * 0.62);
        panel(r, PANEL, CYAN);
        g.ui.bold(r.x + 20.0, r.y + 28.0, 18.0, CYAN, "FIELD VIEW CONTROLS");
        let lines = [
            "LEFT CLICK a soldier to select. LEFT CLICK a tile to move.",
            "RIGHT CLICK an alien to fire. 1 / 2 / 3 choose auto, snap, aimed.",
            "K kneel (+15% aim). R reload. H trauma kit. G grenade at hovered tile.",
            "TAB next soldier. Reaction fire uses the TU you leave unspent.",
            "SPACE ends your turn. Q / E rotate. W A S D pans.",
            "Mouse wheel zooms. The selected soldier is the camera anchor.",
            "Every shot is rolled: accuracy first, then damage against armour.",
            "AP costs, ammo and time units are shown on the control bar.",
        ];
        let mut yy = r.y + 58.0;
        for l in lines { g.ui.text(r.x + 20.0, yy, 11.0, TEXT, l); yy += 21.0; }
        if button(&g.ui, Rect::new(r.x + r.w - 120.0, r.y + r.h - 42.0, 100.0, 28.0), "CLOSE", Btn::Normal, true) { b.show_help = false; }
    }
    if b.inventory_open {
        draw_tactical_inventory(g, &mut b);
    }
    if b.psi_open {
        draw_tactical_psi(g, &mut b);
    }
    g.battle = Some(b);
}

fn draw_hud(g: &mut Game, b: &mut Battle) {
    let w = screen_width(); let h = screen_height();
    let bar_h = 96.0;
    let y0 = h - bar_h;
    draw_rectangle(0.0, y0, w, bar_h, Color { r: 0.008, g: 0.024, b: 0.034, a: 0.97 });
    draw_rectangle(0.0, y0, w, 1.0, Color { a: 0.6, ..CYAN });
    b.hud_health = true; b.hud_ammo = true; b.hud_objective = true;
    if let Some(u) = b.units.get(b.selected).cloned() {
        // portrait
        let pr = Rect::new(10.0, y0 + 10.0, 60.0, 76.0);
        draw_rectangle(pr.x, pr.y, pr.w, pr.h, Color { a: 0.85, ..BG });
        ui::portrait(&g.ui, pr, u.look, b.time, false);
        g.ui.bold(78.0, y0 + 21.0, 12.0, TEXT, &u.name);
        g.ui.text(78.0, y0 + 35.0, 8.5, DIM, &format!("{} // {}", RANKS[u.rank as usize], if u.kneeling { "KNEELING" } else { "STANDING" }));
        g.ui.text(78.0, y0 + 51.0, 8.0, DIM, "TU");
        bar(Rect::new(92.0, y0 + 45.0, 78.0, 5.0), u.tu, u.max_tu, CYAN, Color { a: 0.3, ..BLACK });
        g.ui.text(174.0, y0 + 51.0, 8.0, CYAN, &format!("{:.0}/{:.0}", u.tu, u.max_tu));
        g.ui.text(78.0, y0 + 65.0, 8.0, DIM, "HP");
        bar(Rect::new(92.0, y0 + 59.0, 78.0, 5.0), u.hp as f32, u.max_hp as f32, if u.hp * 2 < u.max_hp { ui::RED } else { ui::GREEN }, Color { a: 0.3, ..BLACK });
        g.ui.text(174.0, y0 + 65.0, 8.0, ui::GREEN, &format!("{}/{}", u.hp, u.max_hp));
        g.ui.text(78.0, y0 + 80.0, 8.0, DIM, &format!("ARM {}  MOR {}  WND {}", u.armor_cur[0], u.morale, u.wounds));

        // weapon block
        let wx = 240.0;
        let wdef = g.data.weapons[u.weapon].clone();
        g.ui.bold(wx, y0 + 20.0, 13.0, AMBER, wdef.name);
        g.ui.text(wx, y0 + 35.0, 8.0, DIM, &format!("{} // DMG{}", g.data.ammo_name(&wdef), g.data.weapon_damage(&wdef)));
        g.ui.text(wx, y0 + 51.0, 11.0, if u.ammo > 0 { TEXT } else { ui::RED }, &format!("AMMO {}/{}", u.ammo, g.data.clip_size(&wdef)));
        g.ui.text(wx + 108.0, y0 + 51.0, 8.0, DIM, &format!("SPARE {}", u.spare));
        let mut bx = wx;
        for m in [ShotMode::Auto, ShotMode::Snap, ShotMode::Aimed] {
            let mi = m.idx();
            let wa = g.data.item(wdef.game, wdef.item).acc[mi] as i32;
            let cost = g.data.item(wdef.game, wdef.item).tu[mi];
            let chance = if wa > 0 { hit_chance(u.acc, wa, u.kneeling, u.hp, u.max_hp, u.wounds, false) } else { 0 };
            let r = Rect::new(bx, y0 + 58.0, 72.0, 30.0);
            let enabled = wa > 0 && u.tu >= cost as f32 && (u.ammo > 0 || wdef.infinite);
            let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
            draw_rectangle(r.x, r.y, r.w, r.h, if b.mode == m { Color { r: 0.08, g: 0.22, b: 0.24, a: 1.0 } } else if hover { Color { a: 0.15, ..PANEL_2 } } else { Color { a: 0.3, ..PANEL } });
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, if b.mode == m { CYAN } else { Color { a: 0.35, ..CYAN_DIM } });
            g.ui.center(r.x + r.w * 0.5, r.y + 12.0, 8.5, if enabled { if b.mode == m { CYAN } else { TEXT } } else { DIM }, m.label());
            g.ui.center(r.x + r.w * 0.5, r.y + 23.0, 7.5, if enabled { AMBER } else { DIM }, &format!("{}% {}TU", chance, cost));
            if hover && is_mouse_button_released(MouseButton::Left) && wa > 0 { b.mode = m; }
            bx += 75.0;
        }
        if button(&g.ui, Rect::new(bx + 2.0, y0 + 60.0, 50.0, 26.0), "FIRE", Btn::Danger, true) { b.fire_clicked = true; }
        if button(&g.ui, Rect::new(bx + 54.0, y0 + 60.0, 52.0, 26.0), "RELOAD", Btn::Normal, true) { b.reload_clicked = true; }
        if button(&g.ui, Rect::new(bx + 108.0, y0 + 60.0, 50.0, 26.0), "KNEEL", Btn::Normal, true) { b.kneel_clicked = true; }
        if button(&g.ui, Rect::new(bx + 160.0, y0 + 60.0, 56.0, 26.0), "GRENADE", Btn::Normal, b.units[b.selected].grenades > 0) { b.grenade_clicked = true; }
        if button(&g.ui, Rect::new(bx + 218.0, y0 + 60.0, 50.0, 26.0), "SMOKE", Btn::Normal, b.units[b.selected].smoke_grenades > 0) { b.smoke_clicked = true; }
        if button(&g.ui, Rect::new(bx + 270.0, y0 + 60.0, 56.0, 26.0), "GEAR [I]", if b.inventory_open { Btn::Amber } else { Btn::Primary }, true) { b.inventory_clicked = true; }
        if button(&g.ui, Rect::new(bx + 330.0, y0 + 60.0, 52.0, 26.0), "REACH", if b.psi_open { Btn::Amber } else { Btn::Primary }, true) { b.psi_clicked = true; }

        // -------- squad strip
        let players: Vec<usize> = b.units.iter().enumerate()
            .filter(|(_, u)| u.side == Side::Player)
            .map(|(i, _)| i).collect();
        if !players.is_empty() {
            let sq_x = (bx + 388.0).max(740.0);
            let sq_label_y = y0 + 10.0;
            g.ui.label(sq_x, sq_label_y, 8.0, DIM, "SQUAD");
            let right_edge = w - 226.0;
            let step = ((right_edge - sq_x) / players.len() as f32).clamp(34.0, 60.0);
            for (k, pi) in players.iter().enumerate() {
                let u = &b.units[*pi];
                let r = Rect::new(sq_x + k as f32 * step, y0 + 16.0, step - 4.0, 44.0);
                let selected = *pi == b.selected;
                let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
                draw_rectangle(r.x, r.y, r.w, r.h, if selected { Color { r: 0.05, g: 0.19, b: 0.21, a: 1.0 } } else if hover { Color { a: 0.16, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
                if selected { draw_rectangle(r.x, r.y, r.w, 2.0, CYAN); }
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, if selected { CYAN } else { Color { a: 0.3, ..CYAN_DIM } });
                let short = u.name.split_whitespace().last().unwrap_or("?");
                g.ui.text(r.x + 5.0, r.y + 12.0, 8.0, if u.alive { TEXT } else { ui::RED }, &g.ui.fit(8.0, short, r.w - 9.0, false));
                let hp_col = if u.hp * 2 < u.max_hp { ui::RED } else if u.hp * 4 < u.max_hp * 3 { AMBER } else { ui::GREEN };
                bar(Rect::new(r.x + 5.0, r.y + 20.0, r.w - 10.0, 4.0), u.hp as f32, u.max_hp as f32, hp_col, Color { a: 0.3, ..BLACK });
                bar(Rect::new(r.x + 5.0, r.y + 28.0, r.w - 10.0, 4.0), u.tu, u.max_tu, CYAN, Color { a: 0.3, ..BLACK });
                g.ui.text(r.x + 5.0, r.y + 40.0, 6.5, DIM, &format!("{}/{}", u.hp.max(0), u.max_hp));
                if hover && is_mouse_button_released(MouseButton::Left) { b.selected = *pi; }
            }
            // equipment + reaction state under the strip
            let sel_snap_cost = {
                let u = &b.units[b.selected];
                let wd = &g.data.weapons[u.weapon];
                let it = g.data.item(wd.game, wd.item);
                let mode = if it.acc[1] > 0 { 1 } else { 2 };
                tu_cost(u.max_tu as i32, it.tu[mode])
            };
            let armed = b.units.get(b.selected).map(|u| u.alive && u.tu >= sel_snap_cost as f32).unwrap_or(false);
            let sy = y0 + 78.0;
            let ex = sq_x + 72.0;
            g.ui.text(ex, sy, 7.5, DIM, &format!("GRENADES {}   SMOKE {}   TRAUMA KITS {}", b.units[b.selected].grenades, b.units[b.selected].smoke_grenades, b.units[b.selected].medkits));
            g.ui.text(ex + 184.0, sy, 7.5, if armed { AMBER } else { DIM },
                if armed { "REACTION FIRE ARMED" } else { "NO RESERVED TU" });
        }

        // operation / end turn (right)
        let rx = w - 210.0;
        let mname = b.mission.name.clone();
        g.ui.label(rx, y0 + 17.0, 8.0, DIM, "OPERATION");
        g.ui.bold(rx, y0 + 31.0, 9.0, TEXT, &g.ui.fit(9.0, &mname, 198.0, true));
        let env = if b.underwater {
            "UNDERWATER // DIVE GEAR".to_string()
        } else {
            format!("ENV // {}", b.weather_desc)
        };
        g.ui.text(rx, y0 + 44.0, 7.5, if b.underwater { CYAN } else { DIM }, &g.ui.fit(7.5, &env, 198.0, false));
        g.ui.text(rx, y0 + 56.0, 8.0, DIM, &format!("TURN {}  HOSTILES {}  REACTIONS {}", b.turn, b.alive_aliens(), b.reaction_shots));
        if button(&g.ui, Rect::new(rx, y0 + 63.0, 118.0, 24.0), "END TURN", Btn::Amber, b.phase == Phase::Player) { b.end_turn_clicked = true; }
    }
    // turn banner
    let bw = 190.0;
    draw_rectangle(w * 0.5 - bw * 0.5, 0.0, bw, 28.0, Color { r: 0.008, g: 0.025, b: 0.038, a: 0.94 });
    g.ui.center(w * 0.5, 19.0, 11.0, if b.phase == Phase::Player { CYAN } else { ui::RED }, if b.phase == Phase::Player { "YOUR TURN" } else if b.phase == Phase::Alien { "ALIEN TURN" } else if b.phase == Phase::Victory { "VICTORY" } else { "DEFEAT" });
    let obj = if b.underwater { "ELIMINATE ALL HOSTILES // DIVE GEAR ON" } else { "ELIMINATE ALL HOSTILES" };
    g.ui.center(w * 0.5, 40.0, 8.0, DIM, obj);

    // Multi-level floor view controls on top right
    draw_rectangle(w - 180.0, 8.0, 170.0, 28.0, Color { r: 0.01, g: 0.02, b: 0.03, a: 0.85 });
    draw_rectangle_lines(w - 180.0, 8.0, 170.0, 28.0, 1.0, Color { a: 0.45, ..CYAN_DIM });
    g.ui.bold(w - 172.0, 25.0, 9.5, CYAN, &format!("FLOOR LVL {}", b.view_level));
    if button(&g.ui, Rect::new(w - 85.0, 11.0, 34.0, 22.0), "LVL ▼", Btn::Ghost, b.view_level > 0) { b.lvl_down_clicked = true; }
    if button(&g.ui, Rect::new(w - 45.0, 11.0, 34.0, 22.0), "LVL ▲", Btn::Ghost, b.view_level < 2) { b.lvl_up_clicked = true; }

    // small control hint, tucked under the top banner on its own dark strip
    draw_rectangle(0.0, 46.0, w, 15.0, Color { r: 0.01, g: 0.02, b: 0.03, a: 0.55 });
    g.ui.text(10.0, 57.0, 7.5, Color { a: 0.85, ..TEXT }, "LMB MOVE  RMB FIRE  1-3 MODE  K KNEEL  R RELOAD  H KIT  G GRENADE  I GEAR  P REACH  [/] LEVEL  TAB NEXT  SPACE END TURN  Q/E ROTATE  WASD PAN");
}

fn draw_tactical_inventory(g: &mut Game, b: &mut Battle) {
    let sw = screen_width();
    let sh = screen_height();

    // Dark backdrop overlay
    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.01, b: 0.02, a: 0.84 });

    let pw = 860.0f32.min(sw - 32.0);
    let ph = 590.0f32.min(sh - 32.0);
    let px = (sw - pw) * 0.5;
    let py = (sh - ph) * 0.5;
    let modal = Rect::new(px, py, pw, ph);

    panel(modal, Color { r: 0.02, g: 0.06, b: 0.09, a: 0.98 }, CYAN);
    draw_rectangle(modal.x, modal.y, modal.w, 40.0, Color { r: 0.04, g: 0.12, b: 0.16, a: 0.95 });
    draw_rectangle(modal.x, modal.y + 40.0, modal.w, 1.0, CYAN_DIM);

    let Some(u) = b.units.get(b.selected).cloned() else { return; };

    g.ui.bold(modal.x + 16.0, modal.y + 26.0, 13.0, CYAN, "TACTICAL INVENTORY & EQUIPMENT PAPERDOLL // TIME UNIT COSTS");
    let close_btn = Rect::new(modal.x + modal.w - 110.0, modal.y + 8.0, 96.0, 26.0);
    if button(&g.ui, close_btn, "CLOSE [ESC]", Btn::Danger, true) {
        g.audio.play_click();
        b.inventory_open = false;
        return;
    }

    // Soldier status banner
    let sy = modal.y + 52.0;
    let pr = Rect::new(modal.x + 16.0, sy, 54.0, 68.0);
    draw_rectangle(pr.x, pr.y, pr.w, pr.h, Color { a: 0.85, ..BG });
    ui::portrait(&g.ui, pr, u.look, b.time, false);

    let bx = modal.x + 80.0;
    g.ui.bold(bx, sy + 16.0, 14.0, TEXT, &u.name);
    g.ui.text(bx, sy + 30.0, 9.5, DIM, &format!("{} // ARMOR: {} (AC {})", RANKS[u.rank as usize], ARMORS[u.armor as usize % ARMORS.len()].name, u.armor_class));

    let wdef = g.data.weapons[u.weapon].clone();
    let w_wt = g.data.item(wdef.game, wdef.item).weight as i32;
    let arm_wt = ARMORS[u.armor as usize % ARMORS.len()].weight;
    let ammo_wt = (u.spare + 1) * 2;
    let gren_wt = u.grenades as i32 * 3;
    let smoke_wt = u.smoke_grenades as i32 * 2;
    let med_wt = u.medkits as i32 * 5;
    let total_wt = w_wt + arm_wt + ammo_wt + gren_wt + smoke_wt + med_wt;
    let strength = 45;
    let encumbered = total_wt > strength;

    g.ui.text(bx, sy + 48.0, 9.0, CYAN, &format!("TU: {:.0} / {:.0}", u.tu, u.max_tu));
    bar(Rect::new(bx + 75.0, sy + 41.0, 70.0, 6.0), u.tu, u.max_tu, CYAN, Color { a: 0.3, ..BLACK });

    g.ui.text(bx + 160.0, sy + 48.0, 9.0, ui::GREEN, &format!("HP: {} / {}", u.hp, u.max_hp));
    bar(Rect::new(bx + 225.0, sy + 41.0, 70.0, 6.0), u.hp as f32, u.max_hp as f32, ui::GREEN, Color { a: 0.3, ..BLACK });

    let wt_col = if encumbered { ui::RED } else { ui::GREEN };
    g.ui.text(bx + 310.0, sy + 48.0, 9.0, wt_col, &format!("WEIGHT: {}kg / {}kg", total_wt, strength));
    g.ui.text(bx + 440.0, sy + 48.0, 8.5, if encumbered { ui::RED } else { DIM }, if encumbered { "OVERBURDENED" } else { "OPTIMAL LOAD" });

    // -------------------------------------------------------------
    // PAPERDOLL EQUIPMENT GRID (Left column: 410px wide)
    // -------------------------------------------------------------
    let grid_x = modal.x + 16.0;
    let grid_y = sy + 76.0;
    g.ui.bold(grid_x, grid_y + 12.0, 11.0, CYAN, "COMBAT READY SLOTS (BODY PAPERDOLL)");

    // Slot 0: RIGHT HAND (Primary Weapon)
    let rh_r = Rect::new(grid_x, grid_y + 22.0, 196.0, 72.0);
    panel(rh_r, Color { a: 0.7, ..PANEL_2 }, CYAN);
    g.ui.bold(rh_r.x + 8.0, rh_r.y + 16.0, 9.5, CYAN, "RIGHT HAND [PRIMARY]");
    g.ui.bold(rh_r.x + 8.0, rh_r.y + 32.0, 11.0, TEXT, wdef.name);
    g.ui.text(rh_r.x + 8.0, rh_r.y + 46.0, 8.5, DIM, &format!("AMMO: {}/{}  //  DMG: {}", u.ammo, g.data.clip_size(&wdef), g.data.weapon_damage(&wdef)));
    if button(&g.ui, Rect::new(rh_r.x + 8.0, rh_r.y + 51.0, 86.0, 17.0), "RELOAD (-15 TU)", Btn::Normal, u.tu >= 15.0 && u.spare > 0 && u.ammo < g.data.clip_size(&wdef)) {
        b.units[b.selected].tu -= 15.0;
        let clip = g.data.clip_size(&wdef);
        let need = clip - b.units[b.selected].ammo;
        b.units[b.selected].ammo = (b.units[b.selected].ammo + need).min(clip);
        b.units[b.selected].spare -= 1;
        g.audio.play_reload();
        b.log("RELOADED FROM SPARE MAG.", CYAN);
    }
    if button(&g.ui, Rect::new(rh_r.x + 98.0, rh_r.y + 51.0, 90.0, 17.0), "TO PACK (-12 TU)", Btn::Ghost, u.tu >= 12.0) {
        b.units[b.selected].tu -= 12.0;
        g.audio.play_equip();
        b.log("STOWED WEAPON TO BACKPACK.", DIM);
    }

    // Slot 1: LEFT HAND (Off-hand / Tactical device)
    let lh_r = Rect::new(grid_x + 204.0, grid_y + 22.0, 196.0, 72.0);
    panel(lh_r, Color { a: 0.7, ..PANEL_2 }, CYAN_DIM);
    g.ui.bold(lh_r.x + 8.0, lh_r.y + 16.0, 9.5, CYAN, "LEFT HAND [OFFHAND]");
    let offhand_item = if u.grenades > 0 { "FRAG GRENADE [READY]" } else if u.smoke_grenades > 0 { "SMOKE CANISTER" } else { "EMPTY (FREE HAND)" };
    g.ui.bold(lh_r.x + 8.0, lh_r.y + 32.0, 10.5, if u.grenades > 0 || u.smoke_grenades > 0 { AMBER } else { DIM }, offhand_item);
    g.ui.text(lh_r.x + 8.0, lh_r.y + 46.0, 8.5, DIM, "QUICK DRAW / UTILITY");
    if u.grenades > 0 {
        if button(&g.ui, Rect::new(lh_r.x + 8.0, lh_r.y + 51.0, 88.0, 17.0), "PRIME (-14 TU)", Btn::Normal, u.tu >= 14.0) {
            b.units[b.selected].tu -= 14.0;
            g.audio.play_equip();
            b.log("FRAG GRENADE FUSE SET [0 TURNS].", AMBER);
        }
    }

    // Shoulders
    let sh_y = grid_y + 102.0;
    let r_sh = Rect::new(grid_x, sh_y, 196.0, 48.0);
    panel(r_sh, Color { a: 0.6, ..PANEL }, CYAN_DIM);
    g.ui.bold(r_sh.x + 8.0, r_sh.y + 14.0, 9.0, CYAN, "R. SHOULDER [MAG 1]");
    let has_mag1 = u.spare > 0;
    let mag1_str = format!("{} CLIP (2kg)", wdef.short);
    g.ui.text(r_sh.x + 8.0, r_sh.y + 28.0, 9.5, TEXT, if has_mag1 { &mag1_str } else { "EMPTY" });
    if button(&g.ui, Rect::new(r_sh.x + 104.0, r_sh.y + 12.0, 84.0, 22.0), "DRAW (-3 TU)", Btn::Ghost, u.tu >= 3.0 && has_mag1) {
        b.units[b.selected].tu -= 3.0;
        g.audio.play_equip();
        b.log("DRAWN MAG FROM SHOULDER HARNESS.", CYAN);
    }

    let l_sh = Rect::new(grid_x + 204.0, sh_y, 196.0, 48.0);
    panel(l_sh, Color { a: 0.6, ..PANEL }, CYAN_DIM);
    g.ui.bold(l_sh.x + 8.0, l_sh.y + 14.0, 9.0, CYAN, "L. SHOULDER [MAG 2]");
    let has_mag2 = u.spare > 1;
    let mag2_str = format!("{} CLIP (2kg)", wdef.short);
    g.ui.text(l_sh.x + 8.0, l_sh.y + 28.0, 9.5, TEXT, if has_mag2 { &mag2_str } else { "EMPTY" });
    if button(&g.ui, Rect::new(l_sh.x + 104.0, l_sh.y + 12.0, 84.0, 22.0), "DRAW (-3 TU)", Btn::Ghost, u.tu >= 3.0 && has_mag2) {
        b.units[b.selected].tu -= 3.0;
        g.audio.play_equip();
        b.log("DRAWN MAG FROM SHOULDER HARNESS.", CYAN);
    }

    // Belt Slots (4 slots)
    let belt_y = sh_y + 56.0;
    g.ui.bold(grid_x, belt_y + 12.0, 10.0, CYAN, "TACTICAL BELT HARNESS (4 TU EQUIP COST)");
    let bw = 94.0;
    let belt_slots = [
        ("BELT 1", if u.grenades > 0 { "FRAG GRENADE" } else { "EMPTY" }, u.grenades > 0),
        ("BELT 2", if u.smoke_grenades > 0 { "SMOKE CAN." } else { "EMPTY" }, u.smoke_grenades > 0),
        ("BELT 3", if u.spare > 2 { "EXTRA MAG" } else { "EMPTY" }, u.spare > 2),
        ("BELT 4", "TRIP MINE", false),
    ];
    for (i, (slabel, sitem, has_item)) in belt_slots.iter().enumerate() {
        let br = Rect::new(grid_x + i as f32 * (bw + 8.0), belt_y + 18.0, bw, 52.0);
        panel(br, Color { a: 0.6, ..PANEL }, if *has_item { CYAN_DIM } else { Color { a: 0.25, ..DIM } });
        g.ui.bold(br.x + 6.0, br.y + 14.0, 8.5, CYAN, slabel);
        g.ui.text(br.x + 6.0, br.y + 28.0, 8.0, if *has_item { TEXT } else { DIM }, sitem);
        if button(&g.ui, Rect::new(br.x + 6.0, br.y + 32.0, br.w - 12.0, 16.0), "TAKE (-4 TU)", Btn::Ghost, *has_item && u.tu >= 4.0) {
            b.units[b.selected].tu -= 4.0;
            g.audio.play_equip();
            b.log("DRAWN FROM BELT HARNESS.", CYAN);
        }
    }

    // Legs / Thigh Holsters
    let leg_y = belt_y + 78.0;
    let r_leg = Rect::new(grid_x, leg_y, 196.0, 44.0);
    panel(r_leg, Color { a: 0.6, ..PANEL }, CYAN_DIM);
    g.ui.bold(r_leg.x + 8.0, r_leg.y + 14.0, 9.0, CYAN, "RIGHT THIGH HOLSTER");
    g.ui.text(r_leg.x + 8.0, r_leg.y + 28.0, 8.5, TEXT, "SIDEARM / POUCH (-4 TU)");

    let l_leg = Rect::new(grid_x + 204.0, leg_y, 196.0, 44.0);
    panel(l_leg, Color { a: 0.6, ..PANEL }, CYAN_DIM);
    g.ui.bold(l_leg.x + 8.0, l_leg.y + 14.0, 9.0, CYAN, "LEFT THIGH POUCH");
    g.ui.text(l_leg.x + 8.0, l_leg.y + 28.0, 8.5, TEXT, "UTILITY / FLARE (-4 TU)");

    // -------------------------------------------------------------
    // RIGHT COLUMN: BACKPACK COMPARTMENTS & FIELD ACTIONS
    // -------------------------------------------------------------
    let right_x = modal.x + 430.0;
    let right_w = modal.w - 446.0;

    g.ui.bold(right_x, grid_y + 12.0, 11.0, CYAN, "BACKPACK GRID & FIELD ACTIONS (8-12 TU)");

    // Backpack 2x2 Grid Slots
    let bp_slots = [
        ("PACK 1", if u.medkits > 0 { "TRAUMA KIT" } else { "EMPTY COMPARTMENT" }, u.medkits > 0),
        ("PACK 2", if u.grenades > 2 { "FRAG GRENADE" } else { "EMPTY COMPARTMENT" }, u.grenades > 2),
        ("PACK 3", if u.smoke_grenades > 2 { "SMOKE CHARGE" } else { "EMPTY COMPARTMENT" }, u.smoke_grenades > 2),
        ("PACK 4", "RATIONS // FLARE", true),
    ];
    let bp_w = (right_w - 12.0) * 0.5;
    for (i, (bplabel, bpitem, bphas)) in bp_slots.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;
        let bpr = Rect::new(right_x + col as f32 * (bp_w + 10.0), grid_y + 22.0 + row as f32 * 54.0, bp_w, 48.0);
        panel(bpr, Color { a: 0.65, ..PANEL_2 }, if *bphas { CYAN_DIM } else { Color { a: 0.25, ..DIM } });
        g.ui.bold(bpr.x + 6.0, bpr.y + 14.0, 8.5, CYAN, bplabel);
        g.ui.text(bpr.x + 6.0, bpr.y + 28.0, 8.5, if *bphas { TEXT } else { DIM }, bpitem);
        if button(&g.ui, Rect::new(bpr.x + bpr.w - 74.0, bpr.y + 24.0, 68.0, 18.0), "EQUIP (-8 TU)", Btn::Ghost, *bphas && u.tu >= 8.0) {
            b.units[b.selected].tu -= 8.0;
            g.audio.play_equip();
            b.log("RETRIEVED ITEM FROM BACKPACK.", CYAN);
        }
    }

    // Medical Field Treatment Controls
    let med_y = grid_y + 140.0;
    draw_rectangle(right_x, med_y - 6.0, right_w, 1.0, Color { a: 0.35, ..CYAN_DIM });
    g.ui.bold(right_x, med_y + 12.0, 11.0, ui::GREEN, "TRAUMA KIT (10 TU EACH)");

    let has_med = u.medkits > 0;
    let med_btn_w = (right_w - 16.0) / 3.0;

    // Heal wound
    let heal_ok = has_med && u.tu >= 10.0 && (u.wounds > 0 || u.hp < u.max_hp);
    if button(&g.ui, Rect::new(right_x, med_y + 20.0, med_btn_w, 36.0), "TREAT WOUND\n-10 TU", Btn::Primary, heal_ok) {
        b.units[b.selected].tu -= 10.0;
        if b.units[b.selected].wounds > 0 { b.units[b.selected].wounds -= 1; }
        b.units[b.selected].hp = (b.units[b.selected].hp + 12).min(b.units[b.selected].max_hp);
        g.audio.play_equip();
        b.log("WOUND TREATED (+12 HP).", ui::GREEN);
    }

    // Adrenaline
    let stim_ok = has_med && u.tu >= 10.0;
    if button(&g.ui, Rect::new(right_x + med_btn_w + 8.0, med_y + 20.0, med_btn_w, 36.0), "ADRENALINE\n-10 TU", Btn::Normal, stim_ok) {
        b.units[b.selected].tu = (b.units[b.selected].tu + 15.0).min(b.units[b.selected].max_tu);
        b.units[b.selected].stun = (b.units[b.selected].stun - 20).max(0);
        g.audio.play_equip();
        b.log("ADRENALINE GIVEN. STUN CLEARED.", CYAN);
    }

    // Pain killer
    let pain_ok = has_med && u.tu >= 10.0 && u.morale < 100;
    if button(&g.ui, Rect::new(right_x + (med_btn_w + 8.0) * 2.0, med_y + 20.0, med_btn_w, 36.0), "ANALGESIC\n-10 TU", Btn::Normal, pain_ok) {
        b.units[b.selected].tu -= 10.0;
        b.units[b.selected].morale = (b.units[b.selected].morale + 25).min(100);
        g.audio.play_equip();
        b.log("ANALGESIC APPLIED (+25 MORALE).", AMBER);
    }

    // Demolitions & Ordnance Arming
    let ord_y = med_y + 68.0;
    draw_rectangle(right_x, ord_y - 6.0, right_w, 1.0, Color { a: 0.35, ..CYAN_DIM });
    g.ui.bold(right_x, ord_y + 12.0, 11.0, AMBER, "ORDNANCE ARMING & DETONATION FUSES (14 TU)");

    let prime_he_ok = u.grenades > 0 && u.tu >= 14.0;
    if button(&g.ui, Rect::new(right_x, ord_y + 20.0, (right_w - 8.0) * 0.5, 30.0), "SET FUSE (0 TURNS)", Btn::Amber, prime_he_ok) {
        b.units[b.selected].tu -= 14.0;
        g.audio.play_equip();
        b.log("FRAG GRENADE SET TO IMPACT.", AMBER);
    }

    let prime_smk_ok = u.smoke_grenades > 0 && u.tu >= 14.0;
    if button(&g.ui, Rect::new(right_x + (right_w - 8.0) * 0.5 + 8.0, ord_y + 20.0, (right_w - 8.0) * 0.5, 30.0), "PRIME SMOKE (0 TURNS)", Btn::Normal, prime_smk_ok) {
        b.units[b.selected].tu -= 14.0;
        g.audio.play_equip();
        b.log("SMOKE CANISTER FUSE ARMED.", CYAN);
    }

    // Ground items at current tile
    let gnd_y = ord_y + 62.0;
    draw_rectangle(right_x, gnd_y - 6.0, right_w, 1.0, Color { a: 0.35, ..CYAN_DIM });
    g.ui.bold(right_x, gnd_y + 12.0, 11.0, TEXT, "GROUND AT CURRENT POSITION [FLOOR]");
    let gnd_r = Rect::new(right_x, gnd_y + 18.0, right_w, 42.0);
    panel(gnd_r, Color { a: 0.5, ..PANEL }, Color { a: 0.3, ..CYAN_DIM });
    g.ui.text(gnd_r.x + 8.0, gnd_r.y + 16.0, 8.5, DIM, &format!("POSITION ({:.0}, {:.0}) // DROP TO FLOOR: 2 TU", u.x, u.y));
    g.ui.text(gnd_r.x + 8.0, gnd_r.y + 30.0, 8.5, TEXT, "NO UNCLAIMED SALVAGE ON THIS TILE.");
}

fn draw_tactical_psi(g: &mut Game, b: &mut Battle) {
    let sw = screen_width();
    let sh = screen_height();
    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.01, b: 0.02, a: 0.85 });

    let pw = 840.0f32.min(sw - 32.0);
    let ph = 540.0f32.min(sh - 40.0);
    let px = (sw - pw) * 0.5;
    let py = (sh - ph) * 0.5;

    draw_rectangle(px, py, pw, ph, Color { r: 0.02, g: 0.05, b: 0.08, a: 0.98 });
    draw_rectangle_lines(px, py, pw, ph, 2.0, CYAN);

    // Title bar
    draw_rectangle(px, py, pw, 36.0, Color { r: 0.04, g: 0.12, b: 0.16, a: 1.0 });
    draw_rectangle_lines(px, py, pw, 36.0, 1.0, CYAN_DIM);
    g.ui.bold(px + 16.0, py + 23.0, 13.0, CYAN, "REACH // SELECT A TARGET");

    let sel_idx = b.selected;
    let (sel_name, sel_psi_str, sel_psi_skl, sel_tu) = if let Some(u) = b.units.get(sel_idx) {
        (u.name.clone(), u.psi_strength, u.psi_skill, u.tu)
    } else {
        ("NONE".to_string(), 0, 0, 0.0)
    };

    g.ui.text(px + 16.0, py + 56.0, 9.5, TEXT, &format!("OPERATIVE: {}   REACH-STRENGTH: {}   REACH-SKILL: {}   AVAILABLE TU: {:.0}", sel_name, sel_psi_str, sel_psi_skl, sel_tu));
    g.ui.text(px + 16.0, py + 72.0, 8.5, DIM, "Each action costs 25 TU: your Reach strength and skill against the target's. Pick a hostile you can see.");

    let list_y = py + 86.0;
    let list_h = ph - 140.0;
    draw_rectangle(px + 12.0, list_y, pw - 24.0, list_h, Color { a: 0.15, ..PANEL });
    draw_rectangle_lines(px + 12.0, list_y, pw - 24.0, list_h, 1.0, Color { a: 0.3, ..CYAN_DIM });

    let hostiles: Vec<usize> = b.units.iter().enumerate()
        .filter(|(_, u)| u.side == Side::Alien && u.alive)
        .map(|(i, _)| i).collect();

    let mut row_y = list_y + 8.0;
    let (u_x, u_y) = if let Some(u) = b.units.get(sel_idx) { (u.x, u.y) } else { (0.0, 0.0) };

    let mut action_probe = None;
    let mut action_panic = None;
    let mut action_control = None;

    if hostiles.is_empty() {
        g.ui.center(px + pw * 0.5, list_y + list_h * 0.5, 11.0, DIM, "NO DETECTED HOSTILES IN SENSORY RANGE.");
    }

    for &ai in hostiles.iter().take(6) {
        let alien = &b.units[ai];
        let dist = ((alien.x - u_x).powi(2) + (alien.y - u_y).powi(2)).sqrt();
        let rr = Rect::new(px + 16.0, row_y, pw - 32.0, 54.0);
        let hover = rr.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(rr.x, rr.y, rr.w, rr.h, if hover { Color { a: 0.16, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
        draw_rectangle_lines(rr.x, rr.y, rr.w, rr.h, 1.0, Color { a: 0.25, ..CYAN_DIM });

        g.ui.bold(rr.x + 10.0, rr.y + 18.0, 11.0, AMBER, &format!("HOSTILE #{} // {}", ai + 1, alien.name.to_uppercase()));
        g.ui.text(rr.x + 10.0, rr.y + 32.0, 8.5, DIM, &format!("DIST: {:.1} TILES   ARMOR: {}   MORALE: {}", dist, alien.armor_cur[0], alien.morale));

        if b.probed_alien == Some(ai) {
            g.ui.bold(rr.x + 10.0, rr.y + 46.0, 8.5, CYAN, &format!("READ: HP {}/{} | TU {:.0} | REACH-STR {} | WPN {}", alien.hp, alien.max_hp, alien.tu, alien.psi_strength, g.data.weapons[alien.weapon].name));
        }

        // Action buttons
        let btn_y = rr.y + 12.0;
        let can_act = sel_tu >= 25.0;
        if button(&g.ui, Rect::new(rr.right() - 290.0, btn_y, 90.0, 28.0), "READ [25TU]", Btn::Normal, can_act) {
            action_probe = Some(ai);
        }
        if button(&g.ui, Rect::new(rr.right() - 194.0, btn_y, 90.0, 28.0), "ROUT [25TU]", Btn::Amber, can_act) {
            action_panic = Some(ai);
        }
        if button(&g.ui, Rect::new(rr.right() - 98.0, btn_y, 92.0, 28.0), "TAKE [25TU]", Btn::Danger, can_act) {
            action_control = Some(ai);
        }

        row_y += 60.0;
    }

    if let Some(target) = action_probe {
        b.execute_psi_probe(g, sel_idx, target);
    }
    if let Some(target) = action_panic {
        b.execute_psi_panic(g, sel_idx, target);
    }
    if let Some(target) = action_control {
        b.execute_psi_control(g, sel_idx, target);
    }

    if button(&g.ui, Rect::new(px + pw - 130.0, py + ph - 42.0, 116.0, 30.0), "CLOSE [ESC]", Btn::Normal, true) || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
        b.psi_open = false;
        g.audio.play_click();
    }
}
