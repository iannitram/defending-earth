//! Game data tables: items, weapons, armour, facilities, craft, research, nations.
//! Every name and number here is this project's own. See docs/NAMES.md.
use macroquad::prelude::*;
use crate::dice::Dice;

// ---------------------------------------------------------------- random
/// Seeded pseudo-random generator. One seed gives one repeatable sequence, so a mission can be replayed exactly.
#[derive(Clone)]
pub struct Rng(pub u32);

impl Rng {
    pub fn new(seed: u64) -> Self { Self(seed as u32) }
    /// Linear congruential step, returning a 15-bit value.
    pub fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(0x343FD).wrapping_add(0x269EC3);
        (self.0 >> 16) & 0x7FFF
    }
    /// Uniform integer in 0..=n.
    pub fn rand(&mut self, n: i32) -> i32 {
        if n < 0 { return 0; }
        (self.next_u32() % (n as u32 + 1)) as i32
    }
    /// Uniform integer in lo..=hi.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo { return lo; }
        lo + self.rand(hi - lo)
    }
    /// Percentage roll: true with the given chance out of 100.
    pub fn chance(&mut self, percent: i32) -> bool {
        self.rand(100) < percent.max(0).min(100)
    }
    pub fn pct(&mut self) -> i32 { self.rand(100) }
    pub fn unit(&mut self) -> f32 { self.next_u32() as f32 / 32768.0 }
    pub fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T { &v[(self.next_u32() as usize) % v.len()] }
}

// ---------------------------------------------------------------- weapons
#[derive(Clone, Default)]
pub struct ItemRec {
    pub name: String,
    pub damage: u8,
    pub damage_type: u8,
    pub acc: [u8; 3],   // auto, snap, aimed
    pub tu: [u8; 3],    // percent of base TUs
    pub clip_size: u8,
    pub weight: u8,
    pub ammo: [u8; 3],
    pub is_ranged: bool,
    pub is_ammo: bool,
    pub two_handed: bool,
    pub grenade_type: u8,
    pub score: u8,
}

/// (index, name, damage, damage_type, acc [auto, snap, aimed], tu% [auto, snap, aimed], clip, weight)
type ItemRow = (usize, &'static str, u8, u8, [u8; 3], [u8; 3], u8, u8);

fn item_table(rows: &[ItemRow]) -> Vec<ItemRec> {
    let mut v: Vec<ItemRec> = (0..80)
        .map(|_| ItemRec { name: "RESERVED".into(), ..Default::default() })
        .collect();
    for r in rows {
        v[r.0] = ItemRec {
            name: r.1.into(), damage: r.2, damage_type: r.3, acc: r.4, tu: r.5,
            clip_size: r.6, weight: r.7, ..Default::default()
        };
    }
    v
}

/// Land item table. Indices are fixed; any index not listed is a RESERVED row.
const SURFACE_ITEMS: &[ItemRow] = &[
    (0, "SERVICE PISTOL", 0, 0, [0, 58, 74], [0, 16, 28], 0, 2),
    (1, "9MM MAGAZINE", 22, 0, [0, 0, 0], [0, 0, 0], 15, 1),
    (2, "SERVICE RIFLE", 0, 0, [42, 65, 100], [40, 22, 65], 0, 4),
    (3, "5.56 MAGAZINE", 27, 0, [0, 0, 0], [0, 0, 0], 30, 1),
    (4, "HEAVY RIFLE", 0, 0, [0, 55, 115], [0, 34, 78], 0, 13),
    (5, ".50 MAGAZINE", 62, 0, [0, 0, 0], [0, 0, 0], 10, 2),
    (8, "MACHINE GUN", 0, 0, [48, 52, 78], [38, 30, 72], 0, 11),
    (9, "7.62 BELT", 34, 0, [0, 0, 0], [0, 0, 0], 50, 3),
    (12, "AT LAUNCHER", 0, 0, [0, 50, 108], [0, 42, 74], 0, 7),
    (13, "HE ROCKET", 90, 2, [0, 0, 0], [0, 0, 0], 1, 3),
    (16, "CORED PISTOL", 42, 3, [30, 62, 82], [26, 18, 46], 0, 3),
    (17, "CORED RIFLE", 56, 3, [44, 64, 98], [32, 24, 52], 0, 5),
    (18, "CORED HEAVY GUN", 78, 3, [0, 48, 88], [0, 32, 72], 0, 12),
    (19, "FRAG GRENADE", 64, 2, [0, 65, 85], [0, 24, 50], 0, 1),
    (20, "SMOKE GRENADE", 48, 5, [0, 65, 85], [0, 24, 50], 0, 1),
    (22, "DEMO CHARGE", 105, 2, [0, 55, 75], [0, 30, 55], 0, 3),
    (23, "TRIP MINE", 70, 2, [0, 0, 0], [0, 0, 0], 0, 2),
    (24, "TRAUMA KIT", 0, 0, [0, 0, 0], [0, 0, 0], 0, 2),
    (25, "HANDHELD RADAR", 0, 0, [0, 0, 0], [0, 0, 0], 0, 2),
    (26, "STUN BATON", 60, 5, [0, 100, 0], [0, 28, 0], 0, 2),
    (28, "NATURAL WEAPON", 58, 6, [0, 90, 0], [0, 22, 0], 0, 0),
    (34, "ARC TORCH", 0, 0, [46, 70, 108], [36, 30, 62], 0, 9),
    (35, "TORCH CELL", 104, 4, [0, 0, 0], [0, 0, 0], 36, 2),
    (36, "ARC CUTTER", 0, 0, [52, 78, 96], [34, 28, 58], 0, 6),
    (37, "CUTTER CELL", 72, 4, [0, 0, 0], [0, 0, 0], 30, 1),
    (38, "ARC STYLUS", 0, 0, [45, 62, 82], [28, 20, 48], 0, 2),
    (39, "STYLUS CELL", 48, 4, [0, 0, 0], [0, 0, 0], 24, 1),
    (50, "TENDER CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 20),
    (51, "KITE CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 22),
    (52, "LASH CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 40),
    (53, "MAUL CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 55),
    (54, "REGENT CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 28),
    (55, "BROODER CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 45),
    (56, "LATHE WRECK", 0, 0, [0, 0, 0], [0, 0, 0], 0, 60),
    (57, "GORGER CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 80),
    (58, "TRESTLE WRECK", 0, 0, [0, 0, 0], [0, 0, 0], 0, 90),
    (59, "BLOAT CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 18),
    (60, "CINDER CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 70),
    (61, "LIVE TENDER", 0, 0, [0, 0, 0], [0, 0, 0], 0, 20),
    (62, "LIVE KITE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 22),
    (63, "LIVE LASH", 0, 0, [0, 0, 0], [0, 0, 0], 0, 40),
    (64, "LIVE MAUL", 0, 0, [0, 0, 0], [0, 0, 0], 0, 55),
    (65, "LIVE REGENT", 0, 0, [0, 0, 0], [0, 0, 0], 0, 28),
    (66, "LIVE BROODER", 0, 0, [0, 0, 0], [0, 0, 0], 0, 45),
    (67, "DISABLED LATHE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 60),
    (68, "LIVE GORGER", 0, 0, [0, 0, 0], [0, 0, 0], 0, 80),
    (69, "DISABLED TRESTLE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 90),
    (70, "LIVE BLOAT", 0, 0, [0, 0, 0], [0, 0, 0], 0, 18),
    (71, "LIVE CINDER", 0, 0, [0, 0, 0], [0, 0, 0], 0, 70),
    (72, "SPINE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 3),
    (73, "GLOW", 0, 0, [0, 0, 0], [0, 0, 0], 0, 1),
];

/// Underwater item table. Indices are fixed; any index not listed is a RESERVED row.
const WATER_ITEMS: &[ItemRow] = &[
    (0, "DIVE PISTOL", 0, 0, [0, 54, 70], [0, 17, 30], 0, 2),
    (1, "PISTOL BOLTS", 20, 0, [0, 0, 0], [0, 0, 0], 5, 1),
    (2, "DIVE RIFLE", 0, 0, [36, 58, 92], [38, 24, 66], 0, 5),
    (3, "RIFLE BOLTS", 30, 0, [0, 0, 0], [0, 0, 0], 26, 1),
    (4, "HEAVY DIVE RIFLE", 0, 0, [0, 52, 105], [0, 35, 76], 0, 12),
    (5, "HEAVY BOLTS", 54, 0, [0, 0, 0], [0, 0, 0], 8, 2),
    (8, "DIVE MG", 0, 0, [44, 48, 72], [40, 30, 70], 0, 12),
    (9, "BOLT BELT", 36, 0, [0, 0, 0], [0, 0, 0], 40, 3),
    (12, "TORPEDO TUBE", 0, 0, [0, 48, 100], [0, 44, 76], 0, 8),
    (13, "MINI TORPEDO", 96, 2, [0, 0, 0], [0, 0, 0], 1, 4),
    (16, "CAV PISTOL", 0, 0, [28, 60, 80], [28, 19, 44], 0, 3),
    (17, "CAV RIFLE", 0, 0, [40, 62, 96], [34, 25, 54], 0, 6),
    (18, "CAV HEAVY GUN", 0, 0, [0, 46, 90], [0, 33, 74], 0, 13),
    (19, "DIVE CHARGE", 70, 2, [0, 60, 80], [0, 26, 52], 0, 2),
    (21, "IMPLOSION POD", 100, 2, [0, 62, 82], [0, 26, 52], 0, 2),
    (24, "TRAUMA KIT", 0, 0, [0, 0, 0], [0, 0, 0], 0, 2),
    (25, "HANDHELD SONAR", 0, 0, [0, 0, 0], [0, 0, 0], 0, 2),
    (26, "STUN PROD", 66, 5, [0, 100, 0], [0, 30, 0], 0, 2),
    (28, "DIVE KNIFE", 24, 6, [0, 110, 0], [0, 14, 0], 0, 1),
    (29, "NATURAL WEAPON (DEEP)", 62, 6, [0, 90, 0], [0, 22, 0], 0, 0),
    (34, "PRESSURE RAM", 0, 0, [0, 66, 104], [0, 32, 64], 0, 10),
    (35, "RAM CHARGE", 112, 4, [0, 0, 0], [0, 0, 0], 16, 2),
    (36, "PRESSURE BORE", 0, 0, [40, 74, 98], [36, 29, 60], 0, 7),
    (37, "BORE CHARGE", 76, 4, [0, 0, 0], [0, 0, 0], 24, 1),
    (38, "PRESSURE AWL", 0, 0, [0, 60, 84], [0, 21, 50], 0, 2),
    (39, "AWL CHARGE", 50, 4, [0, 0, 0], [0, 0, 0], 20, 1),
    (42, "SNARE PROJECTOR", 0, 0, [0, 55, 95], [0, 40, 70], 0, 9),
    (43, "SNARE CHARGE", 90, 5, [0, 0, 0], [0, 0, 0], 1, 2),
    (47, "CAV PISTOL MAG", 44, 3, [0, 0, 0], [0, 0, 0], 12, 1),
    (48, "CAV RIFLE MAG", 60, 3, [0, 0, 0], [0, 0, 0], 18, 1),
    (49, "CAV HEAVY MAG", 82, 3, [0, 0, 0], [0, 0, 0], 10, 2),
    (50, "FATHOM CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 26),
    (51, "WADER CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 34),
    (52, "MANTLE CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 65),
    (53, "DEACON CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 48),
    (54, "BELL CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 75),
    (55, "SILT CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 16),
    (56, "LAMP WRECK", 0, 0, [0, 0, 0], [0, 0, 0], 0, 58),
    (57, "WREATH CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 38),
    (58, "HULK CORPSE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 95),
    (61, "LIVE FATHOM", 0, 0, [0, 0, 0], [0, 0, 0], 0, 26),
    (62, "LIVE WADER", 0, 0, [0, 0, 0], [0, 0, 0], 0, 34),
    (63, "LIVE MANTLE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 65),
    (64, "LIVE DEACON", 0, 0, [0, 0, 0], [0, 0, 0], 0, 48),
    (65, "LIVE BELL", 0, 0, [0, 0, 0], [0, 0, 0], 0, 75),
    (66, "LIVE SILT", 0, 0, [0, 0, 0], [0, 0, 0], 0, 16),
    (67, "DISABLED LAMP", 0, 0, [0, 0, 0], [0, 0, 0], 0, 58),
    (68, "LIVE WREATH", 0, 0, [0, 0, 0], [0, 0, 0], 0, 38),
    (69, "LIVE HULK", 0, 0, [0, 0, 0], [0, 0, 0], 0, 95),
    (72, "NACRE PLATE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 3),
    (73, "LIVE BRINE", 0, 0, [0, 0, 0], [0, 0, 0], 0, 1),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Env { Surface, Underwater, Both }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cat { Pistol, Rifle, Heavy, Cannon, Launcher, Grenade, Melee, Heal }

#[derive(Clone)]
pub struct WeaponDef {
    pub name: &'static str,
    pub short: &'static str,
    pub game: u8,          // 0 surface, 1 underwater
    pub item: u8,
    pub ammo_item: Option<u8>,
    pub env: Env,
    pub cat: Cat,
    pub tracer: [f32; 3],
    pub unlock: u32,       // campaign progress (missions won) needed
    pub infinite: bool,
}

pub struct GameData {
    pub surface_items: Vec<ItemRec>,
    pub water_items: Vec<ItemRec>,
    pub weapons: Vec<WeaponDef>,
    /// Land mask per pixel: 0 = sea, 1 = land. Sampled from Natural Earth 1:50m.
    pub world: Vec<u8>,
    /// Biome id per pixel (matches tactical.rs ground colours).
    pub biome: Vec<u8>,
    /// Distance-to-coast, 255 at the shoreline falling to 0 inland / offshore.
    pub coast: Vec<u8>,
    pub world_w: usize,
    pub world_h: usize,
    pub albedo_png: &'static [u8],
    pub data_png: &'static [u8],
}

/// Decode the packed Earth raster: R = land, G = biome * 17, B = coastal falloff.
fn decode_earth(bytes: &'static [u8]) -> (usize, usize, Vec<u8>, Vec<u8>, Vec<u8>) {
    let img = Image::from_file_with_format(bytes, Some(ImageFormat::Png))
        .expect("earth_data.png must be a valid PNG");
    let w = img.width as usize;
    let h = img.height as usize;
    let mut world = vec![0u8; w * h];
    let mut biome = vec![0u8; w * h];
    let mut coast = vec![0u8; w * h];
    for i in 0..w * h {
        let px = &img.bytes[i * 4..i * 4 + 4];
        world[i] = if px[0] >= 128 { 1 } else { 0 };
        biome[i] = (px[1] as u32 * 15 / 255) as u8;
        coast[i] = px[2];
    }
    (w, h, world, biome, coast)
}

impl GameData {
    pub fn load() -> Self {
        let surface_items = item_table(SURFACE_ITEMS);
        let water_items = item_table(WATER_ITEMS);
        let albedo_png: &'static [u8] = include_bytes!("../assets/earth_albedo.png");
        let data_png: &'static [u8] = include_bytes!("../assets/earth_data.png");
        let (w, h, world, biome, coast) = decode_earth(data_png);
        let mut weapons = Vec::new();
        let mut add = |name: &'static str, short: &'static str, game: u8, item: u8, ammo: Option<u8>,
                       env: Env, cat: Cat, tracer: [f32; 3], unlock: u32, infinite: bool| {
            weapons.push(WeaponDef { name, short, game, item, ammo_item: ammo, env, cat, tracer, unlock, infinite });
        };
        // --- Surface arsenal
        add("SERVICE PISTOL", "PST", 0, 0, Some(1), Env::Surface, Cat::Pistol, [1.0, 0.82, 0.45], 0, false);
        add("SERVICE RIFLE", "RIF", 0, 2, Some(3), Env::Surface, Cat::Rifle, [1.0, 0.78, 0.42], 0, false);
        add("HEAVY RIFLE", "HVR", 0, 4, Some(5), Env::Surface, Cat::Cannon, [1.0, 0.62, 0.28], 0, false);
        add("MACHINE GUN", "MG", 0, 8, Some(9), Env::Surface, Cat::Cannon, [1.0, 0.68, 0.30], 0, false);
        add("AT LAUNCHER", "ATL", 0, 12, Some(13), Env::Surface, Cat::Launcher, [1.0, 0.5, 0.2], 0, false);
        add("CORED PISTOL", "CRP", 0, 16, None, Env::Surface, Cat::Pistol, [0.5, 0.95, 1.0], 3, true);
        add("CORED RIFLE", "CRR", 0, 17, None, Env::Surface, Cat::Rifle, [0.45, 0.95, 1.0], 3, true);
        add("CORED HEAVY GUN", "CRH", 0, 18, None, Env::Surface, Cat::Heavy, [0.6, 0.9, 1.0], 3, true);
        add("ARC STYLUS", "AST", 0, 38, Some(39), Env::Surface, Cat::Pistol, [0.55, 1.0, 0.55], 6, false);
        add("ARC CUTTER", "ACT", 0, 36, Some(37), Env::Surface, Cat::Rifle, [0.55, 1.0, 0.55], 6, false);
        add("ARC TORCH", "ATR", 0, 34, Some(35), Env::Surface, Cat::Heavy, [0.7, 1.0, 0.45], 6, false);
        add("FRAG GRENADE", "FRG", 0, 19, None, Env::Surface, Cat::Grenade, [1.0, 0.55, 0.2], 0, true);
        add("DEMO CHARGE", "DMO", 0, 22, None, Env::Surface, Cat::Grenade, [1.0, 0.4, 0.15], 0, true);
        add("STUN BATON", "BTN", 0, 26, None, Env::Surface, Cat::Melee, [0.6, 0.8, 1.0], 0, true);
        // --- Underwater arsenal
        add("DIVE PISTOL", "DPS", 1, 0, Some(1), Env::Underwater, Cat::Pistol, [0.65, 0.9, 0.8], 0, false);
        add("DIVE RIFLE", "DRF", 1, 2, Some(3), Env::Underwater, Cat::Rifle, [0.7, 0.95, 0.75], 0, false);
        add("HEAVY DIVE RIFLE", "HDR", 1, 4, Some(5), Env::Underwater, Cat::Cannon, [0.8, 1.0, 0.6], 0, false);
        add("DIVE MG", "DMG", 1, 8, Some(9), Env::Underwater, Cat::Cannon, [0.45, 0.85, 1.0], 0, false);
        add("TORPEDO TUBE", "TRP", 1, 12, Some(13), Env::Underwater, Cat::Launcher, [0.5, 0.8, 1.0], 0, false);
        add("DIVE CHARGE", "CHG", 1, 19, None, Env::Underwater, Cat::Grenade, [0.55, 0.9, 1.0], 0, true);
        add("STUN PROD", "PRD", 1, 26, None, Env::Underwater, Cat::Melee, [0.7, 0.9, 1.0], 0, true);
        add("DIVE KNIFE", "KNF", 1, 28, None, Env::Underwater, Cat::Melee, [0.75, 0.85, 1.0], 0, true);
        add("CAV PISTOL", "CVP", 1, 16, Some(47), Env::Underwater, Cat::Pistol, [0.6, 0.9, 1.0], 11, false);
        add("CAV RIFLE", "CVR", 1, 17, Some(48), Env::Underwater, Cat::Rifle, [0.55, 0.9, 1.0], 11, false);
        add("CAV HEAVY GUN", "CVH", 1, 18, Some(49), Env::Underwater, Cat::Heavy, [0.5, 0.85, 1.0], 11, false);
        add("PRESSURE AWL", "AWL", 1, 38, Some(39), Env::Underwater, Cat::Pistol, [1.0, 0.9, 0.45], 13, false);
        add("PRESSURE BORE", "BOR", 1, 36, Some(37), Env::Underwater, Cat::Rifle, [1.0, 0.85, 0.4], 13, false);
        add("PRESSURE RAM", "RAM", 1, 34, Some(35), Env::Underwater, Cat::Heavy, [1.0, 0.8, 0.35], 13, false);
        add("SNARE PROJECTOR", "SNR", 1, 42, Some(43), Env::Underwater, Cat::Heavy, [0.7, 0.85, 1.0], 13, false);
        add("IMPLOSION POD", "IMP", 1, 21, None, Env::Underwater, Cat::Grenade, [0.8, 0.7, 1.0], 13, true);
        // --- Creature-only natural weapons (never offered to soldiers)
        add("NATURAL WEAPON", "NAT", 0, 28, None, Env::Both, Cat::Melee, [0.75, 0.85, 1.0], u32::MAX, true);
        add("NATURAL WEAPON (DEEP)", "NAT", 1, 29, None, Env::Both, Cat::Melee, [0.75, 0.85, 1.0], u32::MAX, true);
        Self { surface_items, water_items, weapons, world, biome, coast, world_w: w, world_h: h, albedo_png, data_png }
    }

    pub fn item(&self, game: u8, id: u8) -> &ItemRec {
        let v = if game == 0 { &self.surface_items } else { &self.water_items };
        &v[id as usize % v.len()]
    }

    pub fn weapon_damage(&self, w: &WeaponDef) -> i32 {
        match w.ammo_item {
            Some(a) => self.item(w.game, a).damage as i32,
            None => self.item(w.game, w.item).damage as i32,
        }
    }
    pub fn weapon_damage_type(&self, w: &WeaponDef) -> u8 {
        match w.ammo_item {
            Some(a) => self.item(w.game, a).damage_type,
            None => self.item(w.game, w.item).damage_type,
        }
    }
    pub fn clip_size(&self, w: &WeaponDef) -> i32 {
        match w.ammo_item {
            Some(a) => self.item(w.game, a).clip_size as i32,
            None => {
                let rec = self.item(w.game, w.item);
                if w.infinite { 0 } else { rec.clip_size as i32 }
            }
        }
    }
    pub fn ammo_name(&self, w: &WeaponDef) -> &str {
        match w.ammo_item { Some(a) => &self.item(w.game, a).name, None => "NO MAGAZINE" }
    }
    pub fn weight(&self, w: &WeaponDef) -> i32 {
        let mut n = self.item(w.game, w.item).weight as i32;
        if let Some(a) = w.ammo_item { n += self.item(w.game, a).weight as i32; }
        n
    }
    fn world_index(&self, lat: f32, lon: f32) -> usize {
        // the raster's left edge is 180° west
        let x = (((lon + 180.0).rem_euclid(360.0)) / 360.0 * self.world_w as f32) as usize % self.world_w;
        let y = (((90.0 - lat) / 180.0) * self.world_h as f32) as usize;
        y.min(self.world_h - 1) * self.world_w + x
    }
    pub fn terrain_at(&self, lat: f32, lon: f32) -> u8 { self.biome[self.world_index(lat, lon)] }
    pub fn coast_at(&self, lat: f32, lon: f32) -> u8 { self.coast[self.world_index(lat, lon)] }
    pub fn is_water(&self, lat: f32, lon: f32) -> bool { self.world[self.world_index(lat, lon)] == 0 }

    /// Albedo (4096x2048 Natural Earth shaded relief) and the packed land/biome/coast map.
    pub fn globe_textures(&self) -> (Texture2D, Texture2D) {
        let alb = Image::from_file_with_format(self.albedo_png, Some(ImageFormat::Png))
            .expect("earth_albedo.png must be a valid PNG");
        let dat = Image::from_file_with_format(self.data_png, Some(ImageFormat::Png))
            .expect("earth_data.png must be a valid PNG");
        let a = Texture2D::from_image(&alb);
        a.set_filter(FilterMode::Linear);
        let d = Texture2D::from_image(&dat);
        d.set_filter(FilterMode::Linear);
        (a, d)
    }
}

// ---------------------------------------------------------------- soldiers
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gender { Male, Female }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ethnicity { Anglo, Nordic, Slavic, Hispanic, African, Arab, SouthAsian, EastAsian, Pacific }

impl Ethnicity {
    pub const ALL: [Ethnicity; 9] = [Ethnicity::Anglo, Ethnicity::Nordic, Ethnicity::Slavic, Ethnicity::Hispanic,
        Ethnicity::African, Ethnicity::Arab, Ethnicity::SouthAsian, Ethnicity::EastAsian, Ethnicity::Pacific];
    pub fn label(self) -> &'static str {
        match self { Ethnicity::Anglo => "Anglo", Ethnicity::Nordic => "Nordic", Ethnicity::Slavic => "Slavic",
            Ethnicity::Hispanic => "Hispanic", Ethnicity::African => "African", Ethnicity::Arab => "Arab",
            Ethnicity::SouthAsian => "South Asian", Ethnicity::EastAsian => "East Asian", Ethnicity::Pacific => "Pacific" }
    }
    pub fn skin_range(self) -> (u8, u8) {
        match self {
            Ethnicity::Nordic => (0, 1), Ethnicity::Anglo => (0, 2), Ethnicity::Slavic => (1, 2),
            Ethnicity::Hispanic => (2, 4), Ethnicity::Arab => (3, 4), Ethnicity::SouthAsian => (3, 4),
            Ethnicity::EastAsian => (2, 3), Ethnicity::African => (4, 5), Ethnicity::Pacific => (4, 5),
        }
    }
    pub fn names(self) -> (&'static [&'static str], &'static [&'static str], &'static [&'static str], &'static [&'static str]) {
        match self {
            Ethnicity::Anglo => (&ANGLO_M, &ANGLO_F, &ANGLO_LAST, &ANGLO_LAST),
            Ethnicity::Nordic => (&NORDIC_M, &NORDIC_F, &NORDIC_LAST, &NORDIC_LAST),
            Ethnicity::Slavic => (&SLAVIC_M, &SLAVIC_F, &SLAVIC_LAST, &SLAVIC_LAST),
            Ethnicity::Hispanic => (&HISP_M, &HISP_F, &HISP_LAST, &HISP_LAST),
            Ethnicity::African => (&AFRIC_M, &AFRIC_F, &AFRIC_LAST, &AFRIC_LAST),
            Ethnicity::Arab => (&ARAB_M, &ARAB_F, &ARAB_LAST, &ARAB_LAST),
            Ethnicity::SouthAsian => (&SASIA_M, &SASIA_F, &SASIA_LAST, &SASIA_LAST),
            Ethnicity::EastAsian => (&EASIA_M, &EASIA_F, &EASIA_LAST, &EASIA_LAST),
            Ethnicity::Pacific => (&PAC_M, &PAC_F, &PAC_LAST, &PAC_LAST),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Look {
    pub gender: Gender,
    pub ethnicity: Ethnicity,
    pub skin: u8,
    pub face: u8,
    pub jaw: u8,
    pub nose: u8,
    pub brow: u8,
    pub eyes: u8,
    pub eye_color: u8,
    pub hair: u8,
    pub hair_color: u8,
    pub facial_hair: u8,
}

impl Look {
    pub fn random(rng: &mut Rng) -> Self {
        let ethnicity = *rng.pick(&Ethnicity::ALL);
        let gender = if rng.chance(50) { Gender::Male } else { Gender::Female };
        let (lo, hi) = ethnicity.skin_range();
        let skin = rng.range(lo as i32, hi as i32) as u8;
        let hair = if gender == Gender::Female { rng.range(0, 5) as u8 } else { rng.range(0, 4) as u8 };
        let facial_hair = if gender == Gender::Male && rng.chance(38) { rng.range(1, 3) as u8 } else { 0 };
        Self {
            gender, ethnicity,
            skin,
            face: rng.range(0, 3) as u8,
            jaw: rng.range(0, 3) as u8,
            nose: rng.range(0, 3) as u8,
            brow: rng.range(0, 2) as u8,
            eyes: rng.range(0, 2) as u8,
            eye_color: rng.range(0, 4) as u8,
            hair,
            hair_color: if skin >= 4 { 0 } else { rng.range(0, 6) as u8 },
            facial_hair,
        }
    }
    pub fn name(&self, rng: &mut Rng) -> String {
        let (m, f, last, _) = self.ethnicity.names();
        let first = match self.gender { Gender::Male => *rng.pick(m), Gender::Female => *rng.pick(f) };
        format!("{} {}", first, rng.pick(last))
    }
}

#[derive(Clone)]
pub struct Stats {
    pub tu: i32, pub stamina: i32, pub health: i32, pub bravery: i32, pub reactions: i32,
    pub firing: i32, pub throwing: i32, pub strength: i32, pub psi_skill: i32, pub psi_strength: i32,
}

impl Stats {
    pub fn recruit(rng: &mut Rng) -> Self {
        Self {
            tu: rng.range(48, 62), stamina: rng.range(45, 75), health: rng.range(32, 44),
            bravery: rng.range(20, 70), reactions: rng.range(32, 58), firing: rng.range(38, 66),
            throwing: rng.range(45, 75), strength: rng.range(25, 55), psi_skill: 0,
            psi_strength: rng.range(5, 95),
        }
    }
}

#[derive(Clone)]
pub struct Soldier {
    pub id: usize,
    pub name: String,
    pub look: Look,
    pub stats: Stats,
    pub rank: u8,
    pub missions: u32,
    pub kills: u32,
    pub weapon: usize,
    pub ammo: i32,
    pub spare: i32,
    pub armor: u8,
    pub hp: i32,
    pub stun: i32,
    pub wounds: i32,
    pub dead: bool,
    pub wounded_days: u32,
    pub assigned: bool,
    pub grenades: u8,
    pub smoke_grenades: u8,
    pub medkits: u8,
    pub tu: i32,
}

pub const RANKS: [&str; 7] = ["RECRUIT", "PRIVATE", "CORPORAL", "SERGEANT", "LIEUTENANT", "CAPTAIN", "MAJOR"];

#[derive(Clone)]
pub struct ArmorDef {
    pub name: &'static str,
    pub front: i32, pub side: i32, pub rear: i32, pub under: i32,
    pub armor_class: u8,
    pub weight: i32,
    pub underwater: bool,
    pub unlock: u32,
    pub color: [f32; 3],
}

/// Soldier suits. Index order is fixed: 0..2 land, 3..5 dive.
pub const ARMORS: [ArmorDef; 6] = [
    ArmorDef { name: "COMBAT UNIFORM", front: 8, side: 6, rear: 5, under: 2, armor_class: 1, weight: 0, underwater: false, unlock: 0, color: [0.30, 0.34, 0.30] },
    ArmorDef { name: "SPINE CARRIER", front: 46, side: 30, rear: 36, under: 20, armor_class: 2, weight: 9, underwater: false, unlock: 2, color: [0.34, 0.42, 0.38] },
    ArmorDef { name: "SEALED FRAME", front: 90, side: 72, rear: 64, under: 50, armor_class: 3, weight: 18, underwater: false, unlock: 5, color: [0.40, 0.52, 0.58] },
    ArmorDef { name: "DIVE RIG", front: 40, side: 28, rear: 24, under: 16, armor_class: 2, weight: 14, underwater: true, unlock: 9, color: [0.14, 0.34, 0.42] },
    ArmorDef { name: "NACRE DIVE RIG", front: 84, side: 60, rear: 56, under: 40, armor_class: 3, weight: 17, underwater: true, unlock: 11, color: [0.12, 0.28, 0.40] },
    ArmorDef { name: "DEEP FRAME", front: 118, side: 88, rear: 80, under: 60, armor_class: 3, weight: 22, underwater: true, unlock: 13, color: [0.20, 0.26, 0.36] },
];

// ---------------------------------------------------------------- aliens
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedAlienSpec {
    pub kind: usize,
    pub name: String,
    pub underwater: bool,
    pub studied: bool,
}

#[derive(Clone)]
pub struct AlienKind {
    pub name: &'static str,
    pub hp: i32,
    pub tu: i32,
    pub reactions: i32,
    pub acc: i32,
    pub armor: i32,
    pub armor_locs: [i32; 5],
    pub armor_class: u8,
    pub bravery_mod: i32,
    pub psi_strength: i32,
    pub psi_skill: i32,
    pub weapon: usize,
    pub underwater: bool,
    pub body: [f32; 3],
    pub accent: [f32; 3],
    pub scale: f32,
    pub melee: i32,
    pub score: i32,
    pub float: bool,
}

// ---------------------------------------------------------------- names
const ANGLO_M: [&str; 14] = ["James","John","Michael","David","Robert","Daniel","Thomas","Andrew","Peter","Mark","Steven","Paul","Richard","Christopher"];
const ANGLO_F: [&str; 12] = ["Sarah","Emma","Claire","Anna","Lucy","Rachel","Helen","Kate","Laura","Jane","Alice","Sophie"];
const ANGLO_LAST: [&str; 14] = ["Harper","Coleman","Bennett","Foster","Hayes","Sullivan","Walker","Reed","Palmer","Brooks","Ellis","Grant","Ward","Bishop"];
const NORDIC_M: [&str; 10] = ["Lars","Erik","Magnus","Bjorn","Anders","Nils","Soren","Henrik","Gustav","Olav"];
const NORDIC_F: [&str; 9] = ["Astrid","Ingrid","Freya","Sigrid","Karin","Helga","Liv","Solveig","Maja"];
const NORDIC_LAST: [&str; 11] = ["Nilsson","Johansson","Lund","Berg","Hansen","Larsen","Eriksen","Dahl","Holm","Sorensen","Vik"];
const SLAVIC_M: [&str; 12] = ["Ivan","Dmitri","Sergei","Alexei","Nikolai","Viktor","Pavel","Andrei","Mikhail","Yuri","Oleg","Boris"];
const SLAVIC_F: [&str; 10] = ["Natasha","Olga","Irina","Svetlana","Anya","Katya","Elena","Tatiana","Vera","Mila"];
const SLAVIC_LAST: [&str; 12] = ["Volkov","Petrov","Sokolov","Ivanov","Kozlov","Novak","Dvorak","Kovac","Rusak","Morozov","Zajac","Horvath"];
const HISP_M: [&str; 12] = ["Carlos","Miguel","Javier","Diego","Rafael","Luis","Andres","Mateo","Pablo","Alejandro","Jorge","Emilio"];
const HISP_F: [&str; 10] = ["Maria","Carmen","Lucia","Elena","Sofia","Isabel","Valentina","Rosa","Paula","Camila"];
const HISP_LAST: [&str; 12] = ["Garcia","Rodriguez","Martinez","Lopez","Gonzalez","Perez","Sanchez","Ramirez","Torres","Flores","Rivera","Vargas"];
const AFRIC_M: [&str; 14] = ["Kwame","Kofi","Chidi","Tunde","Sipho","Mpho","Jabari","Malik","Ade","Omar","Thabo","Kito","Zuri","Baraka"];
const AFRIC_F: [&str; 12] = ["Amara","Nia","Zuri","Ayo","Thandi","Makena","Lindiwe","Nadifa","Sade","Imani","Ayanna","Esme"];
const AFRIC_LAST: [&str; 14] = ["Mensah","Okafor","Nkosi","Adeyemi","Mbeki","Kamara","Diallo","Ndlovu","Owusu","Beko","Jabari","Molefe","Sesay","Kone"];
const ARAB_M: [&str; 12] = ["Omar","Yusuf","Karim","Tariq","Sami","Rashid","Nabil","Faisal","Adel","Hassan","Zaid","Ibrahim"];
const ARAB_F: [&str; 10] = ["Layla","Amira","Yasmin","Noor","Rania","Salma","Farah","Huda","Mariam","Dina"];
const ARAB_LAST: [&str; 12] = ["Haddad","Khalil","Nasser","Saleh","Mansour","Aziz","Farouk","Sultan","Hamdan","Rahman","Darwish","Qureshi"];
const SASIA_M: [&str; 12] = ["Arjun","Rahul","Vikram","Sanjay","Aditya","Rohan","Kiran","Amit","Dev","Nikhil","Ravi","Karan"];
const SASIA_F: [&str; 10] = ["Priya","Ananya","Meera","Kavya","Asha","Neha","Divya","Anjali","Pooja","Shreya"];
const SASIA_LAST: [&str; 12] = ["Sharma","Patel","Singh","Gupta","Iyer","Nair","Reddy","Mehta","Das","Chowdhury","Bose","Rao"];
const EASIA_M: [&str; 12] = ["Wei","Jun","Hiro","Ren","Minho","Jin","Kaito","Dae","Tao","Kenji","Sung","Chen"];
const EASIA_F: [&str; 10] = ["Mei","Yuna","Hana","Sora","Ling","Xiu","Ji-eun","Aiko","Mina","Yumi"];
const EASIA_LAST: [&str; 12] = ["Chen","Wang","Tanaka","Kim","Lee","Sato","Park","Nguyen","Zhang","Suzuki","Choi","Watanabe"];
const PAC_M: [&str; 10] = ["Tane","Koa","Mano","Ikaika","Rangi","Ari","Tui","Kai","Mateo","Niko"];
const PAC_F: [&str; 9] = ["Leilani","Moana","Aroha","Kiri","Sina","Alana","Mele","Hina","Talia"];
const PAC_LAST: [&str; 10] = ["Kealoha","Ngata","Tupua","Faleolo","Kailani","Mahuika","Te Rangi","Vaka","Alofa","Rangi"];

// ---------------------------------------------------------------- calendar
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Month { #[default] Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct, Nov, Dec }
impl Month {
    pub fn name(self) -> &'static str { ["JANUARY","FEBRUARY","MARCH","APRIL","MAY","JUNE","JULY","AUGUST","SEPTEMBER","OCTOBER","NOVEMBER","DECEMBER"][self as usize] }
    pub fn days(self, year: i32) -> i32 {
        match self {
            Month::Jan|Month::Mar|Month::May|Month::Jul|Month::Aug|Month::Oct|Month::Dec => 31,
            Month::Apr|Month::Jun|Month::Sep|Month::Nov => 30,
            Month::Feb => if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 { 29 } else { 28 },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Date { pub year: i32, pub month: Month, pub day: i32, pub minute: i32 }
impl Date {
    pub fn new() -> Self { Self { year: 2027, month: Month::Jan, day: 1, minute: 0 } }
    pub fn add_minutes(&mut self, m: i32) {
        self.minute += m;
        while self.minute >= 1440 {
            self.minute -= 1440; self.day += 1;
            if self.day > self.month.days(self.year) { self.day = 1; self.month = next_month(self.month); if self.month == Month::Jan { self.year += 1; } }
        }
    }
    pub fn hhmm(&self) -> String { format!("{:02}:{:02}", self.minute / 60, self.minute % 60) }
    pub fn pretty(&self) -> String { format!("{} {:02} {}", self.month.name(), self.day, self.year) }
}
fn next_month(m: Month) -> Month { match m { Month::Jan=>Month::Feb, Month::Feb=>Month::Mar, Month::Mar=>Month::Apr, Month::Apr=>Month::May, Month::May=>Month::Jun, Month::Jun=>Month::Jul, Month::Jul=>Month::Aug, Month::Aug=>Month::Sep, Month::Sep=>Month::Oct, Month::Oct=>Month::Nov, Month::Nov=>Month::Dec, Month::Dec=>Month::Jan } }

/// Chance to hit in percent, from the shooter's firing skill, the weapon's accuracy for the fire mode, and stance, wounds and morale.
pub fn hit_chance(fa: i32, weapon_acc: i32, kneeling: bool, health: i32, max_health: i32,
                  wounds: i32, two_handed_offhand: bool) -> i32 {
    let max_hp = max_health.max(1);
    let mut acc = fa;
    if kneeling { acc = (acc * 115) / 100; }
    if two_handed_offhand { acc = (acc * 80) / 100; }
    let health_mod = (75 + (25 * health.max(0)) / max_hp - wounds.max(0) * 10).max(10);
    let unit_acc = (health_mod * acc) / 100;
    (unit_acc * weapon_acc) / 100
}

/// Percent-TU cost uses the soldier's BASE time units (not current), floored.
pub fn tu_cost(base_tu: i32, percent: u8) -> i32 { (base_tu * percent as i32) / 100 }

/// Damage roll: one logged roll, 50 to 150 percent of the weapon's power, for
/// every kind of hit (owner, 2026-11-13, R8; it replaced 0 to 200 percent for
/// non-explosives). Callers apply the modifier table and subtract location armour.
pub fn roll_damage(dice: &mut Dice, who: &str, what: &'static str, base: i32) -> i32 {
    if base <= 0 { return 0; }
    dice.range(who, what, 0, base) + base / 2
}

/// Damage modifier in percent on land: rows are damage types 0..7, columns are armour classes 0..13.
pub const DAMAGE_MODS: [[i32; 14]; 8] = [
    //  Bare Soft Plat Seal Memb Scal Insu Heav Mine Slik Hide Mach Dron Pres
    // 0: Kinetic
    [100, 100,  90,  80, 110,  90, 100,  80,  50, 100,  90,  60,  80,  70],
    // 1: Heat
    [120, 110,  90,  50, 150, 100,  60,  90,  20, 130, 110,  40,  70,  80],
    // 2: Blast
    [120, 110, 100,  80, 130, 100, 100,  80, 120, 100,  90,  70, 110,  60],
    // 3: Core
    [100, 100, 100,  90, 100, 110,  70, 100,  80, 110, 100, 130, 140,  90],
    // 4: Arc
    [110, 110, 100,  80, 100,  90,  60,  90,  70, 100, 100,  80,  90,  80],
    // 5: Stun
    [110, 100,  90,  70, 110, 100,  80,  80,  40,  90,  90,   0,   0,  60],
    // 6: Cut
    [120, 110,  80,  60, 140,  90, 100,  80,  30, 110,  90,  40,  60,  50],
    // 7: Acid
    [130, 120, 110,  90, 120,  90, 100,  90, 140,  70, 100, 120, 110,  80],
];

/// Damage modifier in percent under water: same rows and columns.
pub const DAMAGE_MODS_WATER: [[i32; 14]; 8] = [
    //  Bare Soft Plat Seal Memb Scal Insu Heav Mine Slik Hide Mach Dron Pres
    // 0: Kinetic
    [100, 100,  90,  70, 100,  80, 100,  70,  60, 110,  80,  70,  80,  40],
    // 1: Heat
    [100,  90,  80,  40, 120,  90,  70,  80,  30, 110,  90,  50,  70,  60],
    // 2: Blast
    [130, 120, 110,  90, 140, 110, 100,  90, 110, 120, 100,  80, 120,  50],
    // 3: Cav
    [100, 100, 100,  90, 110, 100,  90,  90,  90, 110, 100, 110, 120,  80],
    // 4: Pressure
    [120, 110, 100,  80, 120, 100,  90,  90,  80, 110,  90,  90, 100,  30],
    // 5: Stun
    [110, 100,  90,  80, 110, 100,  70,  90,  40, 100,  90,   0,   0,  70],
    // 6: Cut
    [120, 110,  80,  60, 150,  90, 100,  80,  30, 120,  90,  40,  60,  40],
    // 7: Acid
    [110, 100,  90,  80, 100,  80,  90,  80, 120,  60,  90, 100,  90,  70],
];

/// Modifier in percent for a damage type against an armour class. `game` picks the land or the underwater table.
pub fn damage_modifier(game: u8, damage_type: u8, armour_class: u8) -> i32 {
    if damage_type > 7 { return 100; }
    let col = (armour_class as usize).min(13);
    if game == 1 {
        DAMAGE_MODS_WATER[damage_type as usize][col]
    } else {
        DAMAGE_MODS[damage_type as usize][col]
    }
}

pub const WALK_TU: i32 = 4;

pub fn promotion(missions: u32, kills: u32) -> u8 {
    let score = missions + kills / 3;
    match score { 0..=1 => 0, 2..=4 => 1, 5..=8 => 2, 9..=13 => 3, 14..=19 => 4, 20..=27 => 5, _ => 6 }
}

// ---------------------------------------------------------------- BASE FACILITIES
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FacilityType {
    Entry,
    Barracks,
    Laboratory,
    Workshop,
    SearchSensor,
    LongRangeSensor,
    MissileBattery,
    SupplyBay,
    Holding,
    AdaptedBattery,
    CapturedBattery,
    DerivedBattery,
    DefenceScreen,
    PsiDamper,
    PsiWing,
    SigintStation,
    Hangar,
}

#[derive(Clone)]
pub struct FacilityDef {
    pub ftype: FacilityType,
    pub name_surface: &'static str,
    pub name_water: &'static str,
    pub cost: i64,
    pub build_days: i32,
    pub upkeep: i64,
    pub size: usize, // 1 for 1x1, 2 for 2x2
    pub living_cap: i32,
    pub lab_cap: i32,
    pub workshop_cap: i32,
    pub store_cap: i32,
    pub alien_cap: i32,
    pub craft_cap: i32,
    pub radar_range: i32,
    pub defense_val: i32,
    pub desc: &'static str,
}

pub const FACILITY_CATALOG: [FacilityDef; 17] = [
    FacilityDef { ftype: FacilityType::Entry, name_surface: "ENTRY SHAFT", name_water: "MOON POOL", cost: 250_000, build_days: 2, upkeep: 5_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 0, defense_val: 0, desc: "Shaft and lock to the outside. Every other module must connect back to it." },
    FacilityDef { ftype: FacilityType::Barracks, name_surface: "BARRACKS", name_water: "BERTHING", cost: 350_000, build_days: 14, upkeep: 8_000, size: 1, living_cap: 48, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 0, defense_val: 0, desc: "Bunks, mess and showers for 48 people: soldiers, scientists and engineers." },
    FacilityDef { ftype: FacilityType::Laboratory, name_surface: "RESEARCH LAB", name_water: "WET LAB", cost: 700_000, build_days: 28, upkeep: 28_000, size: 1, living_cap: 0, lab_cap: 40, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 0, defense_val: 0, desc: "Bench space for 40 scientists working on recovered material." },
    FacilityDef { ftype: FacilityType::Workshop, name_surface: "MACHINE SHOP", name_water: "MACHINE SHOP", cost: 750_000, build_days: 28, upkeep: 32_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 40, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 0, defense_val: 0, desc: "Floor space and tooling for 40 engineers building craft and kit." },
    FacilityDef { ftype: FacilityType::SearchSensor, name_surface: "SEARCH RADAR", name_water: "SEARCH SONAR", cost: 450_000, build_days: 14, upkeep: 9_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 750, defense_val: 0, desc: "Rotating search set. Picks up contacts out to 750 nm." },
    FacilityDef { ftype: FacilityType::LongRangeSensor, name_surface: "LONG-RANGE RADAR", name_water: "TOWED ARRAY", cost: 850_000, build_days: 21, upkeep: 17_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 1500, defense_val: 0, desc: "Large fixed array. Picks up contacts out to 1,500 nm." },
    FacilityDef { ftype: FacilityType::MissileBattery, name_surface: "SAM BATTERY", name_water: "TORPEDO BATTERY", cost: 250_000, build_days: 14, upkeep: 6_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 0, defense_val: 450, desc: "Issue point-defence battery. The first answer to an attack on the base." },
    FacilityDef { ftype: FacilityType::SupplyBay, name_surface: "SUPPLY BAY", name_water: "SUPPLY BAY", cost: 200_000, build_days: 7, upkeep: 4_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 60, alien_cap: 0, craft_cap: 0, radar_range: 0, defense_val: 0, desc: "Racked storage for 60 units of kit and recovered material." },
    FacilityDef { ftype: FacilityType::Holding, name_surface: "HOLDING BLOCK", name_water: "HOLDING TANK", cost: 450_000, build_days: 21, upkeep: 14_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 8, craft_cap: 0, radar_range: 0, defense_val: 0, desc: "Sealed cells for up to 8 live captives awaiting study." },
    FacilityDef { ftype: FacilityType::AdaptedBattery, name_surface: "CORED BATTERY", name_water: "CAV BATTERY", cost: 450_000, build_days: 21, upkeep: 9_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 0, defense_val: 650, desc: "Point defence rebuilt around recovered cores. Fast and accurate." },
    FacilityDef { ftype: FacilityType::CapturedBattery, name_surface: "ARC BATTERY", name_water: "PRESSURE BATTERY", cost: 650_000, build_days: 28, upkeep: 13_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 0, defense_val: 950, desc: "Enemy tools on a human mount. Hits hard, runs hot." },
    FacilityDef { ftype: FacilityType::DerivedBattery, name_surface: "BREACHER BATTERY", name_water: "BREACHER BATTERY", cost: 900_000, build_days: 35, upkeep: 18_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 0, defense_val: 1300, desc: "Our own design, built their way. The last line, and the heaviest." },
    FacilityDef { ftype: FacilityType::DefenceScreen, name_surface: "FARADAY SCREEN", name_water: "BAFFLE SCREEN", cost: 1_100_000, build_days: 35, upkeep: 16_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 0, defense_val: 350, desc: "Grounded mesh over the site. Bleeds off incoming fire and slows the attackers' approach." },
    FacilityDef { ftype: FacilityType::PsiDamper, name_surface: "REACH DAMPER", name_water: "REACH DAMPER", cost: 1_250_000, build_days: 35, upkeep: 18_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 0, defense_val: 0, desc: "Keeps the base quiet to whatever they listen with. Scouts pass it by." },
    FacilityDef { ftype: FacilityType::PsiWing, name_surface: "REACH WING", name_water: "REACH WING", cost: 800_000, build_days: 21, upkeep: 16_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 0, defense_val: 0, desc: "Testing and training rooms for soldiers who show Reach ability." },
    FacilityDef { ftype: FacilityType::SigintStation, name_surface: "SIGINT STATION", name_water: "SIGINT STATION", cost: 1_500_000, build_days: 28, upkeep: 32_000, size: 1, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 0, radar_range: 2600, defense_val: 0, desc: "Reads the enemy's traffic: every contact in range, with class, crew and heading." },
    FacilityDef { ftype: FacilityType::Hangar, name_surface: "HANGAR", name_water: "WET DOCK", cost: 250_000, build_days: 21, upkeep: 22_000, size: 2, living_cap: 0, lab_cap: 0, workshop_cap: 0, store_cap: 0, alien_cap: 0, craft_cap: 1, radar_range: 0, defense_val: 0, desc: "2x2 bay for one craft, with fuel, arming and a way out." },
];

pub fn facility_def(ftype: FacilityType) -> &'static FacilityDef {
    FACILITY_CATALOG.iter().find(|f| f.ftype == ftype).unwrap()
}

// ---------------------------------------------------------------- SHIPS / CRAFT
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CraftType {
    Heron,
    Swift,
    Peregrine,
    Ibis,
    Albatross,
    Gannet,
    Skua,
    Cormorant,
    Razorbill,
    Emperor,
}

#[derive(Clone)]
pub struct CraftDef {
    pub ctype: CraftType,
    pub name: &'static str,
    pub game: u8, // 0 surface, 1 underwater
    pub speed: i32,
    pub max_fuel: i32,
    pub fuel_name: &'static str,
    pub soldier_cap: usize,
    pub weapon_mounts: usize,
    pub max_hp: i32,
    pub cost: i64,
    pub upkeep: i64,
    pub desc: &'static str,
}

pub const CRAFT_CATALOG: [CraftDef; 10] = [
    // Surface
    CraftDef { ctype: CraftType::Heron, name: "HERON", game: 0, speed: 700, max_fuel: 1600, fuel_name: "JET FUEL", soldier_cap: 12, weapon_mounts: 0, max_hp: 140, cost: 550_000, upkeep: 36_000, desc: "Twin-rotor transport helicopter. Slow, long-legged, lands anywhere." },
    CraftDef { ctype: CraftType::Swift, name: "SWIFT", game: 0, speed: 1900, max_fuel: 900, fuel_name: "JET FUEL", soldier_cap: 0, weapon_mounts: 2, max_hp: 110, cost: 650_000, upkeep: 12_000, desc: "Twin-engine interceptor with two hardpoints. What an air force has today." },
    CraftDef { ctype: CraftType::Peregrine, name: "PEREGRINE", game: 0, speed: 3800, max_fuel: 24, fuel_name: "GLOW", soldier_cap: 0, weapon_mounts: 2, max_hp: 240, cost: 1_300_000, upkeep: 18_000, desc: "Adapted interceptor: a human airframe built around a recovered power plant." },
    CraftDef { ctype: CraftType::Ibis, name: "IBIS", game: 0, speed: 2900, max_fuel: 32, fuel_name: "GLOW", soldier_cap: 10, weapon_mounts: 1, max_hp: 280, cost: 1_500_000, upkeep: 24_000, desc: "Adapted assault transport. Carries a squad and one weapon." },
    CraftDef { ctype: CraftType::Albatross, name: "ALBATROSS", game: 0, speed: 5000, max_fuel: 64, fuel_name: "GLOW", soldier_cap: 20, weapon_mounts: 2, max_hp: 480, cost: 2_800_000, upkeep: 34_000, desc: "Derived long-range transport. The only craft that can reach the far side of the Moon." },
    // Underwater
    CraftDef { ctype: CraftType::Gannet, name: "GANNET", game: 1, speed: 720, max_fuel: 1500, fuel_name: "DIESEL-ELECTRIC", soldier_cap: 12, weapon_mounts: 0, max_hp: 150, cost: 550_000, upkeep: 36_000, desc: "Transport submersible with a lock-out chamber for a full dive team." },
    CraftDef { ctype: CraftType::Skua, name: "SKUA", game: 1, speed: 2200, max_fuel: 850, fuel_name: "DIESEL-ELECTRIC", soldier_cap: 0, weapon_mounts: 2, max_hp: 125, cost: 650_000, upkeep: 12_000, desc: "Small attack submarine with two torpedo stations." },
    CraftDef { ctype: CraftType::Cormorant, name: "CORMORANT", game: 1, speed: 4100, max_fuel: 28, fuel_name: "LIVE BRINE", soldier_cap: 0, weapon_mounts: 2, max_hp: 260, cost: 1_400_000, upkeep: 18_000, desc: "Adapted attack sub: a human hull around a recovered brine drive." },
    CraftDef { ctype: CraftType::Razorbill, name: "RAZORBILL", game: 1, speed: 3600, max_fuel: 40, fuel_name: "LIVE BRINE", soldier_cap: 10, weapon_mounts: 1, max_hp: 320, cost: 1_600_000, upkeep: 24_000, desc: "Adapted assault sub. Carries a dive team and one weapon." },
    CraftDef { ctype: CraftType::Emperor, name: "EMPEROR", game: 1, speed: 5000, max_fuel: 64, fuel_name: "LIVE BRINE", soldier_cap: 20, weapon_mounts: 2, max_hp: 520, cost: 2_900_000, upkeep: 38_000, desc: "Derived deep transport. The only craft rated for Verity Deep." },
];

pub fn craft_def(ctype: CraftType) -> &'static CraftDef {
    CRAFT_CATALOG.iter().find(|c| c.ctype == ctype).unwrap()
}

// ---------------------------------------------------------------- CRAFT WEAPONS
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CraftWeaponType {
    ShortRangeMissile,
    LongRangeMissile,
    GunPod,
    CoredLance,
    ArcProjector,
    BreacherMissile,
    LightTorpedo,
    HeavyTorpedo,
    BoltCannon,
    CavCannon,
    PressureProjector,
    BreacherTorpedo,
}

#[derive(Clone)]
pub struct CraftWeaponDef {
    pub wtype: CraftWeaponType,
    pub name: &'static str,
    pub game: u8,
    pub range_km: i32,
    pub damage: i32,
    pub accuracy: i32,
    pub max_ammo: i32,
    pub cost: i64,
    pub desc: &'static str,
}

pub const CRAFT_WEAPONS: [CraftWeaponDef; 12] = [
    // Surface
    CraftWeaponDef { wtype: CraftWeaponType::ShortRangeMissile, name: "SHORT-RANGE MISSILE", game: 0, range_km: 28, damage: 65, accuracy: 72, max_ammo: 6, cost: 18_000, desc: "Issue heat-seeking missile. Six to a rail set." },
    CraftWeaponDef { wtype: CraftWeaponType::LongRangeMissile, name: "LONG-RANGE MISSILE", game: 0, range_km: 58, damage: 95, accuracy: 78, max_ammo: 4, cost: 24_000, desc: "Issue radar-guided missile. Reaches further, carries fewer." },
    CraftWeaponDef { wtype: CraftWeaponType::GunPod, name: "GUN POD (20MM)", game: 0, range_km: 9, damage: 12, accuracy: 30, max_ammo: 250, cost: 11_000, desc: "Issue rotary cannon. Short reach, deep magazine." },
    CraftWeaponDef { wtype: CraftWeaponType::CoredLance, name: "CORED LANCE", game: 0, range_km: 24, damage: 75, accuracy: 40, max_ammo: 0, cost: 170_000, desc: "Adapted. A sealed core behind a long barrel. Never needs rearming." },
    CraftWeaponDef { wtype: CraftWeaponType::ArcProjector, name: "ARC PROJECTOR", game: 0, range_km: 48, damage: 130, accuracy: 82, max_ammo: 80, cost: 240_000, desc: "An enemy arc tool scaled up and bolted to a human pylon." },
    CraftWeaponDef { wtype: CraftWeaponType::BreacherMissile, name: "BREACHER MISSILE", game: 0, range_km: 62, damage: 215, accuracy: 95, max_ammo: 2, cost: 310_000, desc: "Derived. Two rounds, each built to open a capital craft." },
    // Underwater
    CraftWeaponDef { wtype: CraftWeaponType::LightTorpedo, name: "LIGHT TORPEDO", game: 1, range_km: 30, damage: 62, accuracy: 68, max_ammo: 8, cost: 17_000, desc: "Issue homing torpedo. Eight to a load." },
    CraftWeaponDef { wtype: CraftWeaponType::HeavyTorpedo, name: "HEAVY TORPEDO", game: 1, range_km: 52, damage: 105, accuracy: 76, max_ammo: 4, cost: 25_000, desc: "Issue wire-guided torpedo. Slower to reload, harder hit." },
    CraftWeaponDef { wtype: CraftWeaponType::BoltCannon, name: "BOLT CANNON", game: 1, range_km: 7, damage: 14, accuracy: 28, max_ammo: 240, cost: 12_000, desc: "Issue rapid-fire bolt gun for knife-fight range." },
    CraftWeaponDef { wtype: CraftWeaponType::CavCannon, name: "CAV CANNON", game: 1, range_km: 22, damage: 85, accuracy: 42, max_ammo: 60, cost: 175_000, desc: "Adapted. Brine-driven slugs that travel in their own vapour pocket." },
    CraftWeaponDef { wtype: CraftWeaponType::PressureProjector, name: "PRESSURE PROJECTOR", game: 1, range_km: 50, damage: 140, accuracy: 80, max_ammo: 90, cost: 250_000, desc: "An enemy pressure tool scaled up and mounted in a human turret." },
    CraftWeaponDef { wtype: CraftWeaponType::BreacherTorpedo, name: "BREACHER TORPEDO", game: 1, range_km: 58, damage: 205, accuracy: 94, max_ammo: 2, cost: 320_000, desc: "Derived. Two rounds, each built to open a capital hull." },
];

pub fn craft_weapon_def(wtype: CraftWeaponType) -> &'static CraftWeaponDef {
    CRAFT_WEAPONS.iter().find(|w| w.wtype == wtype).unwrap()
}

// ---------------------------------------------------------------- RESEARCH TECH TREES
#[derive(Clone, Debug)]
pub struct ResearchTopic {
    pub id: &'static str,
    pub name: &'static str,
    pub game: u8, // 0 surface, 1 underwater
    pub points: i32, // scientist-days
    pub prereqs: &'static [&'static str],
    pub category: &'static str,
    pub description: &'static str,
    pub unlocks: &'static str,
}

pub const RESEARCH_SURFACE: [ResearchTopic; 20] = [
    ResearchTopic { id: "CORED_WEAPONS", name: "CORED WEAPONS", game: 0, points: 140, prereqs: &[], category: "Weapons", description: "A recovered core holds a charge no battery can. Put one behind a barrel and it throws a discharge instead of a bullet.", unlocks: "Cored Pistol, Cored Rifle, Cored Heavy Gun" },
    ResearchTopic { id: "CORED_PISTOL", name: "CORED PISTOL", game: 0, points: 90, prereqs: &["CORED_WEAPONS"], category: "Weapons", description: "A service pistol frame rebuilt around a sealed core. No magazine to change.", unlocks: "Armoury: Cored Pistol" },
    ResearchTopic { id: "CORED_RIFLE", name: "CORED RIFLE", game: 0, points: 180, prereqs: &["CORED_WEAPONS"], category: "Weapons", description: "The service rifle's layout with a core where the magazine well was. Accurate, and it never runs dry.", unlocks: "Armoury: Cored Rifle" },
    ResearchTopic { id: "CORED_HEAVY_GUN", name: "CORED HEAVY GUN", game: 0, points: 240, prereqs: &["CORED_WEAPONS"], category: "Weapons", description: "A crew-served mount for the largest core one soldier can carry. Heavy, slow and worth it.", unlocks: "Armoury: Cored Heavy Gun" },
    ResearchTopic { id: "CORED_LANCE", name: "CORED LANCE", game: 0, points: 320, prereqs: &["CORED_HEAVY_GUN"], category: "Craft", description: "The heavy gun's core scaled up for an aircraft pylon.", unlocks: "Aircraft weapon: Cored Lance" },
    ResearchTopic { id: "LATTICE", name: "SPINE", game: 0, points: 220, prereqs: &[], category: "Materials", description: "The printed frame under every creature and hull. Cut from their wrecks and their dead. Lighter than aluminium, and it mends itself when fed.", unlocks: "Spine Carrier, enemy craft studies" },
    ResearchTopic { id: "LATTICE_CARRIER", name: "SPINE CARRIER", game: 0, points: 160, prereqs: &["LATTICE"], category: "Armor", description: "The issue plate carrier with its ceramic plates swapped for cut Spine.", unlocks: "Workshop: Spine Carrier" },
    ResearchTopic { id: "SEALED_FRAME", name: "SEALED FRAME", game: 0, points: 320, prereqs: &["LATTICE_CARRIER", "ELECTROLYTE"], category: "Armor", description: "A powered, sealed shell grown on a Spine frame and driven by Glow.", unlocks: "Workshop: Sealed Frame" },
    ResearchTopic { id: "ARC_TOOLS", name: "ARC TOOLS", game: 0, points: 380, prereqs: &[], category: "Weapons", description: "They do not carry guns. They carry cutting tools and point them at us. Learning the grips and the cells.", unlocks: "Arc Stylus, Arc Cutter, Arc Torch" },
    ResearchTopic { id: "ELECTROLYTE", name: "GLOW", game: 0, points: 260, prereqs: &[], category: "Propulsion", description: "The shining fluid every enemy machine runs on, gathered from the pools they leave. Handling it without losing the charge.", unlocks: "Sealed Frame, enemy power plant study" },
    ResearchTopic { id: "CRAFT_POWER_PLANT", name: "ENEMY POWER PLANT", game: 0, points: 340, prereqs: &["ELECTROLYTE"], category: "Propulsion", description: "How a craft with no intakes and no exhaust stays in the air.", unlocks: "Enemy airframe study" },
    ResearchTopic { id: "CRAFT_GUIDANCE", name: "ENEMY GUIDANCE", game: 0, points: 310, prereqs: &["LATTICE"], category: "Propulsion", description: "The craft are not flown. They are told where to be. Reading what tells them.", unlocks: "SIGINT Station, enemy airframe study" },
    ResearchTopic { id: "CRAFT_AIRFRAME", name: "ENEMY AIRFRAME", game: 0, points: 440, prereqs: &["CRAFT_POWER_PLANT", "CRAFT_GUIDANCE"], category: "Craft", description: "Putting a recovered power plant and guidance into an airframe a human can fly.", unlocks: "Workshop: Peregrine" },
    ResearchTopic { id: "SIGINT_STATION", name: "SIGINT STATION", game: 0, points: 420, prereqs: &["CRAFT_GUIDANCE"], category: "Facilities", description: "A listening post tuned to the traffic between enemy craft.", unlocks: "Facility: SIGINT Station" },
    ResearchTopic { id: "PSI_WING", name: "REACH STUDIES", game: 0, points: 360, prereqs: &[], category: "Reach", description: "Some creatures push on minds. Some soldiers can push back. Finding out which ones.", unlocks: "Facility: Reach Wing" },
    ResearchTopic { id: "SEED_LOCATION", name: "THE FARSIDE SIGNAL", game: 0, points: 520, prereqs: &["CRAFT_AIRFRAME"], category: "Story", description: "Every craft we have opened was told to come here by the same source. It is not on Earth. It is on the side of the Moon we never see.", unlocks: "Objective: the far side of the Moon" },
    ResearchTopic { id: "STUDY_TENDER", name: "LIVE STUDY: TENDER", game: 0, points: 140, prereqs: &[], category: "Live Study", description: "A living Tender, observed until it stops.", unlocks: "Tender file" },
    ResearchTopic { id: "STUDY_LASH", name: "LIVE STUDY: LASH", game: 0, points: 170, prereqs: &[], category: "Live Study", description: "A living Lash, observed until it stops.", unlocks: "Lash file" },
    ResearchTopic { id: "STUDY_MAUL", name: "LIVE STUDY: MAUL", game: 0, points: 190, prereqs: &[], category: "Live Study", description: "A living Maul, observed until it stops.", unlocks: "Maul file" },
    ResearchTopic { id: "STUDY_REGENT", name: "LIVE STUDY: REGENT", game: 0, points: 310, prereqs: &[], category: "Live Study", description: "A living Regent, observed until it stops. Keep the Reach-sensitive staff away.", unlocks: "Regent file" },
];

pub const RESEARCH_WATER: [ResearchTopic; 20] = [
    ResearchTopic { id: "NACRE", name: "NACRE PLATE", game: 1, points: 230, prereqs: &[], category: "Materials", description: "The layered shell the sea-going builds grow instead of lattice. It takes pressure the way steel takes paint.", unlocks: "Nacre Dive Rig, enemy sea guidance study" },
    ResearchTopic { id: "NACRE_DIVE_RIG", name: "NACRE DIVE RIG", game: 1, points: 190, prereqs: &["NACRE"], category: "Armor", description: "The issue dive rig with nacre plate over the chest, back and helmet.", unlocks: "Workshop: Nacre Dive Rig" },
    ResearchTopic { id: "DEEP_FRAME", name: "DEEP FRAME", game: 1, points: 330, prereqs: &["NACRE_DIVE_RIG", "LIVE_BRINE"], category: "Armor", description: "A powered pressure suit grown in nacre and driven by live brine.", unlocks: "Workshop: Deep Frame" },
    ResearchTopic { id: "DEEP_FRAME_THRUST", name: "DEEP FRAME THRUST PACK", game: 1, points: 410, prereqs: &["DEEP_FRAME", "SEA_GUIDANCE"], category: "Armor", description: "The deep frame with a guided thrust pack, so a diver can hold station in a current.", unlocks: "Workshop: Deep Frame Thrust Pack" },
    ResearchTopic { id: "CAV_WEAPONS", name: "CAV WEAPONS", game: 1, points: 170, prereqs: &[], category: "Weapons", description: "A slug that travels inside its own vapour pocket keeps its speed under water. Building guns that fire one.", unlocks: "Cav Pistol, Cav Rifle, Cav Heavy Gun" },
    ResearchTopic { id: "CAV_RIFLE", name: "CAV RIFLE", game: 1, points: 140, prereqs: &["CAV_WEAPONS"], category: "Weapons", description: "The dive rifle's layout around a cav barrel and magazine.", unlocks: "Workshop: Cav Rifle" },
    ResearchTopic { id: "CAV_HEAVY_GUN", name: "CAV HEAVY GUN", game: 1, points: 230, prereqs: &["CAV_WEAPONS"], category: "Weapons", description: "The largest cav barrel a diver can brace and fire.", unlocks: "Armoury: Cav Heavy Gun" },
    ResearchTopic { id: "PRESSURE_TOOLS", name: "PRESSURE TOOLS", game: 1, points: 400, prereqs: &[], category: "Weapons", description: "Their sea tools do not cut. They squeeze. Learning the grips and the charges.", unlocks: "Pressure Awl, Pressure Bore, Pressure Ram" },
    ResearchTopic { id: "LIVE_BRINE", name: "LIVE BRINE", game: 1, points: 260, prereqs: &[], category: "Propulsion", description: "The charged fluid the sea-going builds run on. It stays live only while it is kept cold and dark.", unlocks: "Deep Frame, brine drive study" },
    ResearchTopic { id: "SEA_GUIDANCE", name: "ENEMY SEA GUIDANCE", game: 1, points: 310, prereqs: &["NACRE"], category: "Propulsion", description: "How a craft with no sonar finds its way along a trench in the dark.", unlocks: "SIGINT Station, enemy pressure hull study" },
    ResearchTopic { id: "BRINE_DRIVE", name: "BRINE DRIVE", game: 1, points: 370, prereqs: &["LIVE_BRINE"], category: "Propulsion", description: "A drive with no screw and no wake. Making one turn for us.", unlocks: "Enemy pressure hull study" },
    ResearchTopic { id: "SEA_HULL", name: "ENEMY PRESSURE HULL", game: 1, points: 470, prereqs: &["BRINE_DRIVE", "SEA_GUIDANCE"], category: "Craft", description: "Putting a brine drive and their guidance into a hull a human crew can survive in.", unlocks: "Workshop: Cormorant" },
    ResearchTopic { id: "SIGINT_STATION_DEEP", name: "SIGINT STATION", game: 1, points: 420, prereqs: &["SEA_GUIDANCE"], category: "Facilities", description: "A listening post tuned to the traffic between enemy craft under water.", unlocks: "Facility: SIGINT Station" },
    ResearchTopic { id: "PSI_WING_DEEP", name: "REACH STUDIES", game: 1, points: 370, prereqs: &[], category: "Reach", description: "Some creatures push on minds. Some divers can push back. Finding out which ones.", unlocks: "Facility: Reach Wing" },
    ResearchTopic { id: "ROOT_SIGNAL", name: "THE ROOT SIGNAL", game: 1, points: 460, prereqs: &["SEA_HULL"], category: "Story", description: "Every craft we have opened was told where to go by one source, and it is under the South Pacific.", unlocks: "Verity Deep study" },
    ResearchTopic { id: "VERITY_DEEP", name: "VERITY DEEP", game: 1, points: 560, prereqs: &["ROOT_SIGNAL", "SEA_HULL"], category: "Story", description: "Charting the basin the signal comes from, and what it would take to get a dive team to the bottom of it.", unlocks: "Objective: Verity Deep" },
    ResearchTopic { id: "STUDY_FATHOM", name: "LIVE STUDY: FATHOM", game: 1, points: 150, prereqs: &[], category: "Live Study", description: "A living Fathom, observed until it stops.", unlocks: "Fathom file" },
    ResearchTopic { id: "STUDY_WADER", name: "LIVE STUDY: WADER", game: 1, points: 170, prereqs: &[], category: "Live Study", description: "A living Wader, observed until it stops.", unlocks: "Wader file" },
    ResearchTopic { id: "STUDY_MANTLE", name: "LIVE STUDY: MANTLE", game: 1, points: 230, prereqs: &[], category: "Live Study", description: "A living Mantle, observed until it stops.", unlocks: "Mantle file" },
    ResearchTopic { id: "STUDY_DEACON", name: "LIVE STUDY: DEACON", game: 1, points: 330, prereqs: &[], category: "Live Study", description: "A living Deacon, observed until it stops. Keep the Reach-sensitive staff away.", unlocks: "Deacon file" },
];

// ---------------------------------------------------------------- WORKSHOP BLUEPRINTS
#[derive(Clone, Debug)]
pub struct ProductionBlueprint {
    pub id: &'static str,
    pub name: &'static str,
    pub game: u8,
    pub tech_req: &'static str,
    pub engineer_hours: i32,
    pub cost: i64,
    pub structure: i32,
    pub power: i32,
    pub desc: &'static str,
}

pub const WORKSHOP_BLUEPRINTS: [ProductionBlueprint; 14] = [
    ProductionBlueprint { id: "LATTICE_CARRIER", name: "SPINE CARRIER", game: 0, tech_req: "LATTICE_CARRIER", engineer_hours: 700, cost: 20_000, structure: 4, power: 0, desc: "Plate carrier with cut Spine plates." },
    ProductionBlueprint { id: "SEALED_FRAME", name: "SEALED FRAME", game: 0, tech_req: "SEALED_FRAME", engineer_hours: 1100, cost: 48_000, structure: 6, power: 4, desc: "Powered sealed shell on a Spine frame." },
    ProductionBlueprint { id: "CORED_PISTOL", name: "CORED PISTOL", game: 0, tech_req: "CORED_PISTOL", engineer_hours: 250, cost: 9_000, structure: 1, power: 0, desc: "Service pistol frame with a sealed core." },
    ProductionBlueprint { id: "CORED_RIFLE", name: "CORED RIFLE", game: 0, tech_req: "CORED_RIFLE", engineer_hours: 450, cost: 18_000, structure: 2, power: 0, desc: "Service rifle layout with a sealed core." },
    ProductionBlueprint { id: "CORED_HEAVY_GUN", name: "CORED HEAVY GUN", game: 0, tech_req: "CORED_HEAVY_GUN", engineer_hours: 750, cost: 36_000, structure: 4, power: 0, desc: "Heavy mount for the largest carried core." },
    ProductionBlueprint { id: "CORED_LANCE", name: "CORED LANCE", game: 0, tech_req: "CORED_LANCE", engineer_hours: 1100, cost: 170_000, structure: 6, power: 0, desc: "Aircraft weapon. Never needs rearming." },
    ProductionBlueprint { id: "PEREGRINE", name: "PEREGRINE INTERCEPTOR", game: 0, tech_req: "CRAFT_AIRFRAME", engineer_hours: 4200, cost: 1_300_000, structure: 22, power: 14, desc: "Adapted interceptor around a recovered power plant." },
    ProductionBlueprint { id: "ALBATROSS", name: "ALBATROSS TRANSPORT", game: 0, tech_req: "SEED_LOCATION", engineer_hours: 8500, cost: 2_800_000, structure: 42, power: 32, desc: "Derived long-range transport. Reaches the Moon." },
    // Underwater
    ProductionBlueprint { id: "NACRE_DIVE_RIG", name: "NACRE DIVE RIG", game: 1, tech_req: "NACRE_DIVE_RIG", engineer_hours: 750, cost: 22_000, structure: 4, power: 0, desc: "Dive rig with nacre plate." },
    ProductionBlueprint { id: "DEEP_FRAME", name: "DEEP FRAME", game: 1, tech_req: "DEEP_FRAME", engineer_hours: 1150, cost: 50_000, structure: 6, power: 4, desc: "Powered pressure suit grown in nacre." },
    ProductionBlueprint { id: "DEEP_FRAME_THRUST", name: "DEEP FRAME THRUST PACK", game: 1, tech_req: "DEEP_FRAME_THRUST", engineer_hours: 1500, cost: 62_000, structure: 8, power: 7, desc: "Deep frame with a guided thrust pack." },
    ProductionBlueprint { id: "CAV_RIFLE", name: "CAV RIFLE", game: 1, tech_req: "CAV_RIFLE", engineer_hours: 450, cost: 19_000, structure: 2, power: 0, desc: "Dive rifle layout with a cav barrel." },
    ProductionBlueprint { id: "CORMORANT", name: "CORMORANT ATTACK SUB", game: 1, tech_req: "SEA_HULL", engineer_hours: 4400, cost: 1_400_000, structure: 24, power: 14, desc: "Adapted attack sub around a recovered brine drive." },
    ProductionBlueprint { id: "EMPEROR", name: "EMPEROR TRANSPORT SUB", game: 1, tech_req: "VERITY_DEEP", engineer_hours: 9200, cost: 2_900_000, structure: 46, power: 36, desc: "Derived deep transport. Rated for Verity Deep." },
];

// ---------------------------------------------------------------- FUNDING NATIONS & BLOCS
#[derive(Clone, Debug)]
pub struct NationDef {
    pub name: &'static str,
    pub game: u8,
    pub lat: f32,
    pub lon: f32,
    pub base_funding: i64,
}

pub const NATIONS_SURFACE: [NationDef; 16] = [
    NationDef { name: "USA", game: 0, lat: 38.0, lon: -97.0, base_funding: 850_000 },
    NationDef { name: "RUSSIA", game: 0, lat: 55.0, lon: 37.0, base_funding: 420_000 },
    NationDef { name: "UNITED KINGDOM", game: 0, lat: 51.5, lon: -0.1, base_funding: 320_000 },
    NationDef { name: "FRANCE", game: 0, lat: 48.8, lon: 2.3, base_funding: 300_000 },
    NationDef { name: "GERMANY", game: 0, lat: 52.5, lon: 13.4, base_funding: 360_000 },
    NationDef { name: "ITALY", game: 0, lat: 41.9, lon: 12.5, base_funding: 240_000 },
    NationDef { name: "SPAIN", game: 0, lat: 40.4, lon: -3.7, base_funding: 200_000 },
    NationDef { name: "CHINA", game: 0, lat: 39.9, lon: 116.4, base_funding: 400_000 },
    NationDef { name: "JAPAN", game: 0, lat: 35.7, lon: 139.7, base_funding: 480_000 },
    NationDef { name: "INDIA", game: 0, lat: 28.6, lon: 77.2, base_funding: 280_000 },
    NationDef { name: "BRAZIL", game: 0, lat: -15.8, lon: -47.9, base_funding: 260_000 },
    NationDef { name: "AUSTRALIA", game: 0, lat: -35.3, lon: 149.1, base_funding: 280_000 },
    NationDef { name: "NIGERIA", game: 0, lat: 9.0, lon: 7.5, base_funding: 180_000 },
    NationDef { name: "SOUTH AFRICA", game: 0, lat: -25.7, lon: 28.2, base_funding: 220_000 },
    NationDef { name: "EGYPT", game: 0, lat: 30.0, lon: 31.2, base_funding: 200_000 },
    NationDef { name: "CANADA", game: 0, lat: 45.4, lon: -75.7, base_funding: 340_000 },
];

pub const BLOCS_WATER: [NationDef; 13] = [
    NationDef { name: "NORTH ATLANTIC COMPACT", game: 1, lat: 44.0, lon: -40.0, base_funding: 880_000 },
    NationDef { name: "EUROPEAN SHELF GROUP", game: 1, lat: 50.0, lon: 10.0, base_funding: 750_000 },
    NationDef { name: "EAST ASIA SEA LANES", game: 1, lat: 32.0, lon: 110.0, base_funding: 720_000 },
    NationDef { name: "CORAL SEA GROUP", game: 1, lat: -25.0, lon: 135.0, base_funding: 420_000 },
    NationDef { name: "SOUTH ATLANTIC COMPACT", game: 1, lat: -10.0, lon: -55.0, base_funding: 340_000 },
    NationDef { name: "GULF OF GUINEA GROUP", game: 1, lat: 0.0, lon: 20.0, base_funding: 310_000 },
    NationDef { name: "ARABIAN SEA GROUP", game: 1, lat: 24.0, lon: 45.0, base_funding: 380_000 },
    NationDef { name: "RED SEA AUTHORITY", game: 1, lat: 26.0, lon: 30.0, base_funding: 260_000 },
    NationDef { name: "NORTH PACIFIC COMPACT", game: 1, lat: 36.0, lon: 138.0, base_funding: 510_000 },
    NationDef { name: "NORDIC AND BALTIC GROUP", game: 1, lat: 60.0, lon: 15.0, base_funding: 350_000 },
    NationDef { name: "ARCTIC ROUTE AUTHORITY", game: 1, lat: 56.0, lon: 60.0, base_funding: 480_000 },
    NationDef { name: "TASMAN GROUP", game: 1, lat: -40.0, lon: 174.0, base_funding: 250_000 },
    NationDef { name: "CARIBBEAN AND CANAL AUTHORITY", game: 1, lat: 15.0, lon: -90.0, base_funding: 220_000 },
];

#[derive(Clone, Debug)]
pub struct NationState {
    pub name: String,
    pub game: u8,
    pub lat: f32,
    pub lon: f32,
    pub funding: i64,
    pub satisfaction: i32, // 0..100
    pub pact_signed: bool,
    pub last_delta: i64,
    pub defence_activity: i32,
    pub alien_activity: i32,
    pub panic: i32, // 0..100
}

// ---------------------------------------------------------------- COUNTRY & SECTOR GEOGRAPHIC BOUNDARIES
pub struct CountryBorder {
    pub name: &'static str,
    pub game: u8,
    pub points: &'static [(f32, f32)],
}

pub const COUNTRY_BORDERS: &[CountryBorder] = &[
    CountryBorder {
        name: "USA",
        game: 0,
        points: &[
            (48.5, -124.5), (49.0, -117.0), (49.0, -95.0), (48.0, -89.5), (46.0, -84.0),
            (45.0, -75.0), (45.0, -71.0), (44.5, -67.0), (41.0, -70.0), (37.0, -76.0),
            (31.0, -81.0), (25.0, -80.5), (29.0, -84.5), (29.0, -89.5), (29.5, -94.0),
            (26.0, -97.0), (29.0, -103.5), (31.8, -106.5), (31.3, -111.0), (32.5, -117.0),
            (37.0, -122.5), (42.0, -124.5), (46.0, -124.0), (48.5, -124.5),
        ],
    },
    CountryBorder {
        name: "CANADA",
        game: 0,
        points: &[
            (49.0, -123.0), (49.0, -95.0), (48.0, -89.5), (45.0, -75.0), (45.0, -67.0),
            (52.0, -56.0), (60.0, -64.0), (62.0, -77.0), (60.0, -94.0), (70.0, -130.0),
            (69.0, -141.0), (60.0, -141.0), (54.0, -130.0), (49.0, -123.0),
        ],
    },
    CountryBorder {
        name: "RUSSIA",
        game: 0,
        points: &[
            (55.0, 28.0), (60.0, 30.0), (68.0, 32.0), (70.0, 60.0), (73.0, 80.0),
            (76.0, 110.0), (72.0, 140.0), (66.0, 170.0), (60.0, 165.0), (52.0, 142.0),
            (43.0, 132.0), (48.0, 130.0), (50.0, 120.0), (50.0, 87.0), (55.0, 82.0),
            (51.0, 55.0), (45.0, 48.0), (50.0, 40.0), (55.0, 28.0),
        ],
    },
    CountryBorder {
        name: "UNITED KINGDOM",
        game: 0,
        points: &[
            (50.0, -5.0), (50.5, 1.5), (53.0, 0.5), (56.0, -2.0), (58.5, -3.5),
            (58.0, -5.5), (55.0, -5.0), (53.5, -3.0), (51.5, -3.0), (50.0, -5.0),
        ],
    },
    CountryBorder {
        name: "FRANCE",
        game: 0,
        points: &[
            (43.5, -1.5), (46.0, -1.2), (48.5, -4.5), (50.0, 1.5), (51.0, 2.5),
            (49.0, 6.0), (47.5, 7.5), (46.0, 6.0), (43.5, 7.0), (43.0, 3.0),
            (42.5, 3.0), (43.0, 0.0), (43.5, -1.5),
        ],
    },
    CountryBorder {
        name: "GERMANY",
        game: 0,
        points: &[
            (47.5, 7.5), (49.0, 6.0), (51.0, 6.0), (53.5, 7.0), (54.8, 9.0),
            (54.0, 14.0), (51.0, 15.0), (50.0, 12.5), (47.5, 13.0), (47.5, 7.5),
        ],
    },
    CountryBorder {
        name: "ITALY",
        game: 0,
        points: &[
            (46.0, 7.0), (46.5, 13.0), (44.0, 12.5), (41.0, 16.0), (40.0, 18.0),
            (38.0, 16.0), (38.0, 13.0), (41.0, 14.0), (44.0, 8.5), (46.0, 7.0),
        ],
    },
    CountryBorder {
        name: "SPAIN",
        game: 0,
        points: &[
            (36.0, -5.5), (37.0, -8.0), (42.0, -9.0), (43.5, -8.0), (43.5, -1.5),
            (42.5, 3.0), (41.0, 1.0), (38.0, 0.0), (36.5, -2.0), (36.0, -5.5),
        ],
    },
    CountryBorder {
        name: "CHINA",
        game: 0,
        points: &[
            (22.0, 108.0), (22.0, 114.0), (25.0, 119.0), (31.0, 122.0), (38.0, 119.0),
            (40.0, 124.0), (43.0, 131.0), (48.0, 134.0), (53.0, 123.0), (49.0, 117.0),
            (42.0, 100.0), (48.0, 87.0), (40.0, 75.0), (31.0, 79.0), (28.0, 89.0),
            (28.0, 97.0), (22.0, 101.0), (22.0, 108.0),
        ],
    },
    CountryBorder {
        name: "JAPAN",
        game: 0,
        points: &[
            (31.0, 130.5), (33.5, 130.0), (35.5, 133.0), (41.0, 140.5), (45.5, 142.0),
            (43.5, 145.5), (36.0, 140.5), (34.0, 136.0), (31.0, 130.5),
        ],
    },
    CountryBorder {
        name: "INDIA",
        game: 0,
        points: &[
            (8.0, 77.5), (13.0, 80.0), (20.0, 86.0), (22.0, 89.0), (26.0, 89.0),
            (27.0, 96.0), (28.0, 89.0), (31.0, 79.0), (35.0, 75.0), (31.0, 70.0),
            (24.0, 68.5), (16.0, 73.5), (8.0, 77.5),
        ],
    },
    CountryBorder {
        name: "BRAZIL",
        game: 0,
        points: &[
            (-30.0, -50.0), (-24.0, -46.0), (-13.0, -39.0), (-5.0, -35.0), (-2.0, -44.0),
            (4.0, -51.0), (3.0, -60.0), (-4.0, -70.0), (-10.0, -69.0), (-18.0, -58.0),
            (-22.0, -58.0), (-30.0, -57.0), (-33.0, -53.0), (-30.0, -50.0),
        ],
    },
    CountryBorder {
        name: "AUSTRALIA",
        game: 0,
        points: &[
            (-38.0, 145.0), (-33.0, 151.0), (-24.0, 153.0), (-14.0, 144.0), (-11.0, 142.0),
            (-15.0, 136.0), (-12.0, 131.0), (-17.0, 122.0), (-22.0, 114.0), (-32.0, 115.0),
            (-35.0, 118.0), (-32.0, 129.0), (-35.0, 136.0), (-38.0, 141.0), (-38.0, 145.0),
        ],
    },
    CountryBorder {
        name: "EGYPT",
        game: 0,
        points: &[
            (22.0, 25.0), (31.5, 25.0), (31.5, 34.0), (29.5, 35.0), (28.0, 33.5),
            (22.0, 36.8), (22.0, 25.0),
        ],
    },
    CountryBorder {
        name: "NIGERIA",
        game: 0,
        points: &[
            (4.5, 6.0), (4.5, 8.5), (6.5, 9.5), (10.0, 12.0), (13.0, 14.0),
            (13.8, 12.0), (13.8, 4.0), (10.0, 3.5), (6.5, 2.7), (4.5, 6.0),
        ],
    },
    CountryBorder {
        name: "SOUTH AFRICA",
        game: 0,
        points: &[
            (-34.5, 20.0), (-34.0, 25.5), (-29.0, 32.0), (-27.0, 32.8), (-22.2, 30.0),
            (-22.2, 29.0), (-25.0, 25.0), (-27.0, 20.0), (-28.5, 16.5), (-33.0, 18.0),
            (-34.5, 20.0),
        ],
    },
    // Sea-lane groups (underwater campaign)
    CountryBorder {
        name: "NORTH ATLANTIC COMPACT",
        game: 1,
        points: &[
            (20.0, -60.0), (65.0, -60.0), (65.0, -10.0), (20.0, -10.0), (20.0, -60.0),
        ],
    },
    CountryBorder {
        name: "EUROPEAN SHELF GROUP",
        game: 1,
        points: &[
            (35.0, -10.0), (60.0, -10.0), (60.0, 35.0), (35.0, 35.0), (35.0, -10.0),
        ],
    },
    CountryBorder {
        name: "EAST ASIA SEA LANES",
        game: 1,
        points: &[
            (10.0, 95.0), (45.0, 95.0), (45.0, 145.0), (10.0, 145.0), (10.0, 95.0),
        ],
    },
    CountryBorder {
        name: "CORAL SEA GROUP",
        game: 1,
        points: &[
            (-45.0, 110.0), (-10.0, 110.0), (-10.0, 165.0), (-45.0, 165.0), (-45.0, 110.0),
        ],
    },
];

// ---------------------------------------------------------------- PROMOTION LOGIC
#[derive(Clone, Debug)]
pub struct PromotionAward {
    pub soldier_id: usize,
    pub name: String,
    pub old_rank: u8,
    pub new_rank: u8,
}

/// Monthly promotions. A Recruit with at least one mission becomes a Private; higher ranks are filled from the best-scoring soldiers, limited by how many soldiers the force has.
pub fn evaluate_force_promotions(soldiers: &mut [Soldier]) -> Vec<PromotionAward> {
    let mut awards = Vec::new();
    let living_indices: Vec<usize> = soldiers.iter().enumerate().filter(|(_, s)| !s.dead).map(|(i, _)| i).collect();
    let n = living_indices.len();
    if n == 0 { return awards; }

    // First promote recruits who completed a mission
    for &idx in &living_indices {
        if soldiers[idx].rank == 0 && soldiers[idx].missions >= 1 {
            soldiers[idx].rank = 1;
            awards.push(PromotionAward {
                soldier_id: soldiers[idx].id,
                name: soldiers[idx].name.clone(),
                old_rank: 0,
                new_rank: 1,
            });
        }
    }

    let quota_corporals = n / 5;
    let quota_sergeants = n / 11;
    let quota_lieutenants = n / 23;
    let quota_captains = n / 30;
    let quota_majors = if n >= 30 { 1 } else { 0 };

    let merit = |s: &Soldier| -> i32 {
        // Score: (missions + kills) * 10 plus a blend of the soldier's attributes
        (s.missions as i32 + s.kills as i32) * 10
            + s.stats.tu + s.stats.health + s.stats.stamina + s.stats.reactions
            + s.stats.firing + s.stats.throwing + s.stats.strength + s.stats.bravery
    };

    // Promotion passes upwards, one rank at a time
    let try_promote = |soldiers: &mut [Soldier], from_rank: u8, to_rank: u8, quota: usize, awards: &mut Vec<PromotionAward>, bravery_bonus: i32| {
        let current_at_or_above = soldiers.iter().filter(|s| !s.dead && s.rank >= to_rank).count();
        if current_at_or_above < quota {
            let mut candidates: Vec<usize> = soldiers.iter().enumerate()
                .filter(|(_, s)| !s.dead && s.rank == from_rank)
                .map(|(i, _)| i)
                .collect();
            candidates.sort_by_key(|&i| -merit(&soldiers[i]));
            let needed = quota - current_at_or_above;
            for &c_idx in candidates.iter().take(needed) {
                soldiers[c_idx].rank = to_rank;
                // A promoted soldier gains bravery
                soldiers[c_idx].stats.bravery = (soldiers[c_idx].stats.bravery + bravery_bonus).min(100);
                awards.push(PromotionAward {
                    soldier_id: soldiers[c_idx].id,
                    name: soldiers[c_idx].name.clone(),
                    old_rank: from_rank,
                    new_rank: to_rank,
                });
            }
        }
    };

    try_promote(soldiers, 1, 2, quota_corporals, &mut awards, 10);
    try_promote(soldiers, 2, 3, quota_sergeants, &mut awards, 20);
    try_promote(soldiers, 3, 4, quota_lieutenants, &mut awards, 30);
    try_promote(soldiers, 4, 5, quota_captains, &mut awards, 30);
    try_promote(soldiers, 5, 6, quota_majors, &mut awards, 40);

    awards
}

// ---------------------------------------------------------------- MONTHLY REPORT
#[derive(Clone, Debug, Default)]
pub struct MonthlyReport {
    pub month: Month,
    pub year: i32,
    pub total_funding: i64,
    pub nation_changes: Vec<(String, i64, i64)>, // name, current, delta
    pub facility_maint: i64,
    pub craft_maint: i64,
    pub soldier_salaries: i64,
    pub scientist_salaries: i64,
    pub engineer_salaries: i64,
    pub total_expenses: i64,
    pub net_balance: i64,
    pub score: i32,
    pub rating: &'static str,
    pub promotions: Vec<String>,
    pub pact_warnings: Vec<String>,
}


