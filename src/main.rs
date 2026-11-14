#![allow(dead_code)]
use macroquad::prelude::*;
use std::env;

mod data;
mod globe;
mod ui;
mod base;
mod tactical;
mod dogfight;
mod audio;
mod sky;
mod gfx;

use data::*;
use globe::{Globe, ZoomDir};
use ui::{Ui, BG, PANEL, PANEL_2, CYAN, CYAN_DIM, AMBER, TEXT, DIM, Btn, button, panel, glow, wrap, bar, vignette, scanlines};
use dogfight::DogfightState;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen { Title, CampaignSelect, SituationMap, Base, Editor, Tactical, Debrief, Event, End, AssemblyReport, Dogfight }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SiteKind { Crash, Landed, Raid, Underwater, AlienBase, Final, BaseDefense }

#[derive(Clone)]
pub struct Site {
    pub id: usize,
    pub name: String,
    pub lat: f32,
    pub lon: f32,
    pub kind: SiteKind,
    pub underwater: bool,
    pub raid: bool,
    pub expires: i32,
    pub aliens: Vec<u8>,
    pub tier: u32,
    pub reward: i64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UfoKind { Spore, Burr, Thorn, Rhizome, Taproot }

#[derive(Clone)]
pub struct Ufo {
    pub id: usize,
    pub lat: f32,
    pub lon: f32,
    pub target_lat: f32,
    pub target_lon: f32,
    pub kind: UfoKind,
    pub underwater: bool,
    pub hp: i32,
    pub timer: i32,
    pub detected: bool,
    pub raid: bool,
    pub speed: f32,
}

#[derive(Clone, Debug)]
pub struct BaseFacility {
    pub ftype: FacilityType,
    pub x: usize,
    pub y: usize,
    pub days_left: i32,
}

#[derive(Clone, Debug)]
pub struct CraftInstance {
    pub name: String,
    pub ctype: CraftType,
    pub ready: bool,
    pub hp: i32,
    pub max_hp: i32,
    pub fuel: i32,
    pub max_fuel: i32,
    pub weapons: [Option<CraftWeaponType>; 2],
    pub ammo: [i32; 2],
}

#[derive(Clone, Debug)]
pub struct ActiveResearch {
    pub tech_id: String,
    pub scientists: i32,
    pub progress: i32,
    pub total_points: i32,
}

#[derive(Clone, Debug)]
pub struct ActiveProduction {
    pub blueprint_id: String,
    pub engineers: i32,
    pub progress: i32,
    pub total_hours: i32,
}

#[derive(Clone)]
pub struct Base {
    pub name: String,
    pub lat: f32,
    pub lon: f32,
    pub water: bool,
    pub radar: f32,
    pub facilities: Vec<BaseFacility>,
    pub grid: [[Option<usize>; 6]; 6],
    pub craft: Vec<CraftInstance>,
    pub scientists: i32,
    pub engineers: i32,
    pub active_research: Option<ActiveResearch>,
    pub active_production: Option<ActiveProduction>,
    pub lattice: i32,
    pub electrolyte: i32,
    pub nacre: i32,
    pub brine: i32,
    pub captured_aliens: Vec<CapturedAlienSpec>,
    pub items: Vec<(String, i32)>,
}

impl Base {
    pub fn add_item(&mut self, id: &str, count: i32) {
        if let Some(entry) = self.items.iter_mut().find(|(name, _)| name == id) {
            entry.1 += count;
        } else {
            self.items.push((id.to_string(), count));
        }
    }

    pub fn remove_item(&mut self, id: &str, count: i32) -> bool {
        if let Some(entry) = self.items.iter_mut().find(|(name, _)| name == id) {
            if entry.1 >= count {
                entry.1 -= count;
                return true;
            }
        }
        false
    }

    pub fn item_count(&self, id: &str) -> i32 {
        self.items.iter().find(|(name, _)| name == id).map(|(_, c)| *c).unwrap_or(0)
    }

    pub fn new(name: String, lat: f32, lon: f32, water: bool) -> Self {
        let mut b = Self {
            name, lat, lon, water, radar: 1.0,
            facilities: Vec::new(),
            grid: [[None; 6]; 6],
            craft: Vec::new(),
            scientists: 10,
            engineers: 10,
            active_research: None,
            active_production: None,
            lattice: if water { 0 } else { 30 },
            electrolyte: if water { 0 } else { 15 },
            nacre: if water { 30 } else { 0 },
            brine: if water { 15 } else { 0 },
            captured_aliens: Vec::new(),
            items: if water {
                vec![
                    ("DIVE RIFLE".into(), 10),
                    ("RIFLE BOLTS".into(), 24),
                    ("HEAVY DIVE RIFLE".into(), 4),
                    ("HEAVY BOLTS".into(), 12),
                    ("DIVE CHARGE".into(), 12),
                    ("TRAUMA KIT".into(), 6),
                ]
            } else {
                vec![
                    ("SERVICE RIFLE".into(), 12),
                    ("5.56 MAGAZINE".into(), 30),
                    ("SERVICE PISTOL".into(), 6),
                    ("9MM MAGAZINE".into(), 18),
                    ("HEAVY RIFLE".into(), 3),
                    (".50 MAGAZINE".into(), 10),
                    ("FRAG GRENADE".into(), 16),
                    ("SMOKE GRENADE".into(), 10),
                    ("TRAUMA KIT".into(), 8),
                ]
            },
        };
        // Standard starting base facilities
        b.add_facility_direct(FacilityType::Entry, 2, 2, 0);
        b.add_facility_direct(FacilityType::Hangar, 0, 0, 0);
        b.add_facility_direct(FacilityType::Hangar, 4, 0, 0);
        b.add_facility_direct(FacilityType::Barracks, 2, 3, 0);
        b.add_facility_direct(FacilityType::SupplyBay, 2, 1, 0);
        b.add_facility_direct(FacilityType::Laboratory, 1, 2, 0);
        b.add_facility_direct(FacilityType::Workshop, 3, 2, 0);
        b.add_facility_direct(FacilityType::SearchSensor, 3, 3, 0);

        if water {
            b.craft.push(CraftInstance {
                name: "SKUA-1".into(),
                ctype: CraftType::Skua,
                ready: true,
                hp: 125, max_hp: 125,
                fuel: 850, max_fuel: 850,
                weapons: [Some(CraftWeaponType::LightTorpedo), Some(CraftWeaponType::HeavyTorpedo)],
                ammo: [8, 4],
            });
            b.craft.push(CraftInstance {
                name: "GANNET-1".into(),
                ctype: CraftType::Gannet,
                ready: true,
                hp: 150, max_hp: 150,
                fuel: 1500, max_fuel: 1500,
                weapons: [None, None],
                ammo: [0, 0],
            });
        } else {
            b.craft.push(CraftInstance {
                name: "SWIFT-1".into(),
                ctype: CraftType::Swift,
                ready: true,
                hp: 110, max_hp: 110,
                fuel: 900, max_fuel: 900,
                weapons: [Some(CraftWeaponType::ShortRangeMissile), Some(CraftWeaponType::LongRangeMissile)],
                ammo: [6, 4],
            });
            b.craft.push(CraftInstance {
                name: "HERON-1".into(),
                ctype: CraftType::Heron,
                ready: true,
                hp: 140, max_hp: 140,
                fuel: 1600, max_fuel: 1600,
                weapons: [None, None],
                ammo: [0, 0],
            });
        }
        b
    }

    pub fn add_facility_direct(&mut self, ftype: FacilityType, x: usize, y: usize, days: i32) {
        let def = facility_def(ftype);
        let idx = self.facilities.len();
        self.facilities.push(BaseFacility { ftype, x, y, days_left: days });
        for dy in 0..def.size {
            for dx in 0..def.size {
                if x + dx < 6 && y + dy < 6 {
                    self.grid[y + dy][x + dx] = Some(idx);
                }
            }
        }
    }

    pub fn holding_cap(&self) -> usize {
        self.facilities.iter().filter(|f| f.ftype == FacilityType::Holding && f.days_left == 0).count() * 8
    }

    pub fn can_build(&self, ftype: FacilityType, x: usize, y: usize) -> Result<(), &'static str> {
        let def = facility_def(ftype);
        if x + def.size > 6 || y + def.size > 6 { return Err("OUT OF BOUNDS"); }
        for dy in 0..def.size {
            for dx in 0..def.size {
                if self.grid[y + dy][x + dx].is_some() { return Err("CELL OCCUPIED"); }
            }
        }
        let mut adjacent = false;
        for dy in 0..def.size {
            for dx in 0..def.size {
                let cx = x + dx;
                let cy = y + dy;
                let neighbors = [
                    if cx > 0 { Some((cx - 1, cy)) } else { None },
                    if cx + 1 < 6 { Some((cx + 1, cy)) } else { None },
                    if cy > 0 { Some((cx, cy - 1)) } else { None },
                    if cy + 1 < 6 { Some((cx, cy + 1)) } else { None },
                ];
                for n in neighbors.into_iter().flatten() {
                    if self.grid[n.1][n.0].is_some() {
                        adjacent = true;
                        break;
                    }
                }
                if adjacent { break; }
            }
            if adjacent { break; }
        }
        if !adjacent && !self.facilities.is_empty() { return Err("MUST TOUCH EXISTING FACILITY"); }
        Ok(())
    }

    pub fn start_build(&mut self, ftype: FacilityType, x: usize, y: usize) {
        let def = facility_def(ftype);
        self.add_facility_direct(ftype, x, y, def.build_days);
    }

    pub fn dismantle(&mut self, fac_idx: usize) -> Result<(), &'static str> {
        if fac_idx >= self.facilities.len() { return Err("INVALID FACILITY"); }
        if self.facilities[fac_idx].ftype == FacilityType::Entry {
            return Err("THE ENTRY CANNOT BE DISMANTLED");
        }
        for row in &mut self.grid {
            for cell in row {
                if *cell == Some(fac_idx) { *cell = None; }
                else if let Some(idx) = cell {
                    if *idx > fac_idx { *idx -= 1; }
                }
            }
        }
        self.facilities.remove(fac_idx);
        Ok(())
    }

    pub fn living_cap(&self) -> i32 {
        self.facilities.iter().filter(|f| f.days_left == 0).map(|f| facility_def(f.ftype).living_cap).sum()
    }
    pub fn lab_cap(&self) -> i32 {
        self.facilities.iter().filter(|f| f.days_left == 0).map(|f| facility_def(f.ftype).lab_cap).sum()
    }
    pub fn workshop_cap(&self) -> i32 {
        self.facilities.iter().filter(|f| f.days_left == 0).map(|f| facility_def(f.ftype).workshop_cap).sum()
    }
    pub fn stores_cap(&self) -> i32 {
        self.facilities.iter().filter(|f| f.days_left == 0).map(|f| facility_def(f.ftype).store_cap).sum()
    }
    pub fn craft_cap(&self) -> i32 {
        self.facilities.iter().filter(|f| f.days_left == 0).map(|f| facility_def(f.ftype).craft_cap).sum()
    }
    pub fn radar_range(&self) -> i32 {
        self.facilities.iter().filter(|f| f.days_left == 0).map(|f| facility_def(f.ftype).radar_range).max().unwrap_or(0)
    }
    pub fn defense_rating(&self) -> i32 {
        self.facilities.iter().filter(|f| f.days_left == 0).map(|f| facility_def(f.ftype).defense_val).sum()
    }
    pub fn has_sigint(&self) -> bool {
        self.facilities.iter().any(|f| f.days_left == 0 && f.ftype == FacilityType::SigintStation)
    }
    pub fn monthly_upkeep(&self) -> i64 {
        self.facilities.iter().map(|f| facility_def(f.ftype).upkeep).sum()
    }
}

#[derive(Clone)]
pub struct LogLine {
    pub text: String,
    pub color: Color,
    pub age: f32,
}

#[derive(Clone)]
pub struct MissionSpec {
    pub name: String,
    pub lat: f32,
    pub lon: f32,
    pub underwater: bool,
    pub raid: bool,
    pub biome: u8,
    pub aliens: Vec<u8>,
    pub tier: u32,
    pub site: Option<usize>,
    pub reward: i64,
    pub final_mission: bool,
    pub month: Month,
    pub minute_of_day: i32,
}

pub struct Game {
    pub data: GameData,
    pub rng: Rng,
    pub ui: Ui,
    pub audio: audio::SoundEffects,
    pub globe: Globe,
    pub screen: Screen,
    pub date: Date,
    pub speed: usize,
    pub paused: bool,
    pub funds: i64,
    pub score: i32,
    pub missions_won: u32,
    pub water_missions: u32,
    pub water_unlocked: bool,
    pub campaign_type: u8, // 0 = Landfall (surface), 1 = Undertow (underwater)
    pub soldiers: Vec<Soldier>,
    pub bases: Vec<Base>,
    pub ufos: Vec<Ufo>,
    pub sites: Vec<Site>,
    pub nations: Vec<NationState>,
    pub researched_techs: Vec<String>,
    pub last_report: Option<MonthlyReport>,
    pub selected_facility_cell: Option<(usize, usize)>,
    pub building_catalog_open: bool,
    pub base_architecture_mode: bool,
    pub inspector_gear_tab: usize,
    pub selected_blueprint: Option<usize>,
    pub selected_tech_topic: Option<usize>,
    pub selected_craft_idx: Option<usize>,
    pub craft_weapon_modal: Option<(usize, usize)>,
    pub finance_scroll: usize,
    pub research_scroll: usize,
    pub workshop_scroll: usize,
    pub minutes_accumulator: i32,
    pub selected_base: usize,
    pub selected_soldier: usize,
    pub selected_site: Option<usize>,
    pub selected_ufo: Option<usize>,
    pub battle: Option<tactical::Battle>,
    pub dogfight: Option<DogfightState>,
    pub log: Vec<LogLine>,
    pub next_id: usize,
    pub spawn_timer: i32,
    pub elapsed: f32,
    pub toast: Option<(String, f32)>,
    pub flash: f32,
    pub base_tab: usize,
    pub editor_return: Screen,
    pub base_return_zoom: f32,
    pub placing_base: bool,
    pub show_help: bool,
    pub self_test: Option<tactical::SelfTest>,
    pub screenshot: Option<String>,
    pub frame: u64,
    pub last_month: Month,
    pub last_funding: i64,
    pub event_title: String,
    pub event_body: String,
    pub event_next: Screen,
    pub title_pulse: f32,
    pub campaign_started: bool,
    pub game_over: bool,
    pub won: bool,
    pub pending_final: bool,
    pub first_campaign_done: bool,
    pub roster_scroll: usize,
    pub equip_scroll: usize,
    pub demo: bool,
    pub last_kills: u32,
    pub last_lost: u32,
    pub last_shots: u32,
    pub last_hits: u32,
    pub last_mission: String,
    pub last_reward: i64,
    /// Lighting pipeline for the battle view; made the first time it is needed.
    pub gfx: Option<gfx::Gfx>,
    /// Debug: fixed clock for battle lighting and the globe (`--clock`).
    pub dbg_clock: Option<sky::Moment>,
    /// Debug: weather override, 0 clear, 1 rain, 2 snow, 3 dust (`--weather`).
    pub dbg_weather: Option<u8>,
    /// Debug: draw the frame-time readout (`--frametime`).
    pub dbg_frametime: bool,
    /// Debug: force the battlefield biome (`--biome`).
    pub dbg_biome: Option<u8>,
    /// Debug: put the battle at this latitude and longitude (`--site`).
    pub dbg_site: Option<(f32, f32)>,
    /// Debug: extra seed mixed into map generation (`--seed`).
    pub dbg_seed: Option<u64>,
    /// Debug: battle camera preset, 0 default, 1 wide, 2 tele, 3 low (`--view`).
    pub dbg_view: u8,
    /// Debug: seconds to run before `--screenshot` saves (`--t`).
    pub dbg_t: f32,
}

const SPEEDS: [i32; 5] = [1, 4, 12, 60, 240];
const SPEED_LABEL: [&str; 5] = ["1", "2", "3", "4", "5"];

impl Game {
    pub fn new(data: GameData, ui: Ui, audio: audio::SoundEffects) -> Self {
        let mut globe = Globe::new(&data);
        globe.yaw = 1.35;
        globe.pitch = 0.18;
        globe.zoom = 1.0;
        let mut g = Self {
            data, rng: Rng::new(0x5EED_1234), ui, audio, globe,
            screen: Screen::Title,
            date: Date::new(),
            speed: 1, paused: true,
            funds: 6_000_000, score: 0,
            missions_won: 0, water_missions: 0, water_unlocked: false,
            campaign_type: 0,
            soldiers: Vec::new(), bases: Vec::new(), ufos: Vec::new(), sites: Vec::new(),
            nations: Vec::new(), researched_techs: Vec::new(), last_report: None,
            selected_facility_cell: None, building_catalog_open: false,
            base_architecture_mode: true, inspector_gear_tab: 0,
            selected_blueprint: None, selected_tech_topic: None, selected_craft_idx: None,
            craft_weapon_modal: None, finance_scroll: 0, research_scroll: 0, workshop_scroll: 0,
            minutes_accumulator: 0,
            selected_base: 0, selected_soldier: 0, selected_site: None, selected_ufo: None,
            battle: None, dogfight: None, log: Vec::new(), next_id: 1,
            spawn_timer: 40, elapsed: 0.0, toast: None, flash: 0.0,
            base_tab: 0, editor_return: Screen::Base, base_return_zoom: 1.0,
            placing_base: false, show_help: false,
            self_test: None, screenshot: None, frame: 0,
            last_month: Month::Jan, last_funding: 6_000_000,
            event_title: String::new(), event_body: String::new(), event_next: Screen::SituationMap,
            title_pulse: 0.0, campaign_started: false, game_over: false, won: false,
            pending_final: false,
            first_campaign_done: false,
            roster_scroll: 0,
            equip_scroll: 0,
            demo: false,
            last_kills: 0, last_lost: 0, last_shots: 0, last_hits: 0, last_mission: String::new(), last_reward: 0,
            gfx: None, dbg_clock: None, dbg_weather: None, dbg_frametime: false, dbg_biome: None, dbg_site: None, dbg_seed: None, dbg_view: 0, dbg_t: 0.0,
        };
        g.rng = Rng::new((macroquad::time::get_time() * 1000.0) as u64 ^ 0xA5A5_5A5A);
        g
    }

    pub fn log(&mut self, text: impl Into<String>, color: Color) {
        self.log.push(LogLine { text: text.into(), color, age: 0.0 });
        if self.log.len() > 60 { self.log.remove(0); }
    }
    pub fn toast(&mut self, text: impl Into<String>) { self.toast = Some((text.into(), 3.5)); }

    pub fn new_campaign(&mut self, campaign_type: u8) {
        self.campaign_type = campaign_type;
        self.soldiers.clear();
        self.bases.clear();
        self.ufos.clear();
        self.sites.clear();
        self.log.clear();
        self.speed = 1;
        self.paused = false;
        self.funds = 6_000_000;
        self.score = 0;
        self.missions_won = 0;
        self.water_missions = 0;
        self.selected_base = 0;
        self.selected_soldier = 0;
        self.selected_site = None;
        self.selected_ufo = None;
        self.battle = None;
        self.dogfight = None;
        self.game_over = false;
        self.won = false;
        self.campaign_started = true;
        self.pending_final = false;
        self.first_campaign_done = false;
        self.last_month = Month::Jan;
        self.spawn_timer = 60;
        self.last_kills = 0; self.last_lost = 0; self.last_shots = 0; self.last_hits = 0; self.last_mission.clear(); self.last_reward = 0;
        self.minutes_accumulator = 0;
        self.researched_techs.clear();
        self.last_report = None;
        self.selected_facility_cell = None;
        self.building_catalog_open = false;
        self.selected_blueprint = None;
        self.selected_tech_topic = None;
        self.selected_craft_idx = None;
        self.craft_weapon_modal = None;

        // Initialize Funding Nations
        self.nations.clear();
        if campaign_type == 0 {
            self.date = Date { year: 2027, month: Month::Jan, day: 1, minute: 0 };
            self.water_unlocked = false;
            for n in &NATIONS_SURFACE {
                self.nations.push(NationState {
                    name: n.name.to_string(),
                    game: 0,
                    lat: n.lat,
                    lon: n.lon,
                    funding: n.base_funding,
                    satisfaction: 85,
                    pact_signed: false,
                    last_delta: 0,
                    defence_activity: 0,
                    alien_activity: 0,
                    panic: 15,
                });
            }

            // Home base: land base in North America / Europe
            let (lat, lon) = self.find_land(42.0, -96.0);
            self.bases.push(Base::new("HEARTH ALPHA".to_string(), lat, lon, false));
            for i in 0..8 {
                let mut s = self.make_soldier();
                s.assigned = i < 8;
                s.weapon = if i < 5 { self.weapon_index("SERVICE RIFLE") } else if i < 7 { self.weapon_index("HEAVY RIFLE") } else { self.weapon_index("SERVICE PISTOL") };
                s.ammo = self.data.clip_size(&self.data.weapons[s.weapon]);
                s.spare = 2;
                self.soldiers.push(s);
            }
            // First gentle crash site
            let (slat, slon) = self.find_land(lat + 8.0, lon + 12.0);
            let sid = self.next_id();
            self.sites.push(Site {
                id: sid, name: "CRASH SITE // SPORE-CLASS".to_string(),
                lat: slat, lon: slon, kind: SiteKind::Crash, underwater: false, raid: false,
                expires: 2880, aliens: vec![0, 0, 0, 0], tier: 1, reward: 180_000,
            });
            self.next_id += 1;
            self.log("COMMANDER, THE FIRST ALIEN CRAFT IS DOWN.", CYAN);
            self.log("SELECT THE CRASH SITE ON THE GLOBE AND LAUNCH A SQUAD.", TEXT);
            self.log("HEARTH ALPHA IS ONLINE. THE CHARTER STATES ARE WATCHING.", AMBER);
            self.log("SEARCH RADAR UP. SWIFT AND HERON READY.", DIM);
            self.log("ASSEMBLY FUNDING: $6,000,000. FIRST MONTHLY REVIEW PENDING.", TEXT);
            self.log("DEFENSE ROSTER: 8 SOLDIERS EQUIPPED AND ON STANDBY.", DIM);
            self.globe.yaw = 1.35;
            self.globe.pitch = 0.18;
            self.toast("LANDFALL CAMPAIGN STARTED — 2027");
        } else {
            self.date = Date { year: 2027, month: Month::Jan, day: 1, minute: 0 };
            self.water_unlocked = true;
            for b in &BLOCS_WATER {
                self.nations.push(NationState {
                    name: b.name.to_string(),
                    game: 1,
                    lat: b.lat,
                    lon: b.lon,
                    funding: b.base_funding,
                    satisfaction: 85,
                    pact_signed: false,
                    last_delta: 0,
                    defence_activity: 0,
                    alien_activity: 0,
                    panic: 15,
                });
            }

            // Home base: water base in the Atlantic / Pacific
            let (lat, lon) = self.find_water(28.0, -42.0);
            self.bases.push(Base::new("HEARTH DEEP 1".to_string(), lat, lon, true));
            for i in 0..8 {
                let mut s = self.make_soldier();
                s.assigned = i < 8;
                s.weapon = if i < 5 { self.weapon_index("DIVE RIFLE") } else if i < 7 { self.weapon_index("HEAVY DIVE RIFLE") } else { self.weapon_index("DIVE PISTOL") };
                s.ammo = self.data.clip_size(&self.data.weapons[s.weapon]);
                s.spare = 2;
                self.soldiers.push(s);
            }
            // Submerged USO crash site
            let (slat, slon) = self.find_water(lat + 6.0, lon + 10.0);
            let sid = self.next_id();
            self.sites.push(Site {
                id: sid, name: "USO DOWN // SPORE-CLASS".to_string(),
                lat: slat, lon: slon, kind: SiteKind::Underwater, underwater: true, raid: false,
                expires: 2880, aliens: vec![0, 0, 0, 0], tier: 1, reward: 220_000,
            });
            self.next_id += 1;
            self.log("COMMANDER, SUBMERSIBLE CONTACT CRIPPLED IN OCEAN TRENCH.", CYAN);
            self.log("SEND THE GANNET DIVE TEAM TO SECURE THE USO.", TEXT);
            self.log("HEARTH DEEP 1 IS OPERATIONAL ON THE SEA FLOOR.", CYAN);
            self.log("SEARCH SONAR UP. SKUA AND GANNET READY.", DIM);
            self.log("SEA-LANE GROUP FUNDING: $6,000,000. FIRST MONTHLY REVIEW PENDING.", TEXT);
            self.log("DIVE TEAM: 8 DIVERS ARMED AND READY.", DIM);
            self.globe.yaw = 0.85;
            self.globe.pitch = 0.32;
            self.toast("UNDERTOW CAMPAIGN STARTED — 2027");
        }

        self.screen = Screen::SituationMap;
        self.globe.zoom = 1.0;
    }

    pub fn find_land(&self, lat: f32, lon: f32) -> (f32, f32) {
        if !self.data.is_water(lat, lon) { return (lat, lon); }
        for r in 1..40 {
            for i in 0..16 {
                let a = i as f32 / 16.0 * std::f32::consts::TAU;
                let la = (lat + a.cos() * r as f32 * 2.0).clamp(-80.0, 84.0);
                let lo = lon + a.sin() * r as f32 * 2.0;
                if !self.data.is_water(la, lo) { return (la, lo); }
            }
        }
        (lat, lon)
    }
    pub fn find_water(&self, lat: f32, lon: f32) -> (f32, f32) {
        if self.data.is_water(lat, lon) { return (lat, lon); }
        for r in 1..50 {
            for i in 0..16 {
                let a = i as f32 / 16.0 * std::f32::consts::TAU;
                let la = (lat + a.cos() * r as f32 * 2.0).clamp(-80.0, 84.0);
                let lo = lon + a.sin() * r as f32 * 2.0;
                if self.data.is_water(la, lo) { return (la, lo); }
            }
        }
        (lat, lon)
    }

    fn next_id(&mut self) -> usize { let v = self.next_id; self.next_id += 1; v }

    pub fn weapon_index(&self, name: &str) -> usize {
        self.data.weapons.iter().position(|w| w.name == name).unwrap_or(0)
    }
    pub fn make_soldier(&mut self) -> Soldier {
        let look = Look::random(&mut self.rng);
        let name = look.name(&mut self.rng);
        let stats = Stats::recruit(&mut self.rng);
        let hp = stats.health;
        let id = self.next_id();
        Soldier {
            id, name, look, stats, rank: 0, missions: 0, kills: 0,
            weapon: self.weapon_index("SERVICE RIFLE"), ammo: 20, spare: 2, armor: 0,
            hp, stun: 0, wounds: 0, dead: false, wounded_days: 0, assigned: true,
            grenades: 2, smoke_grenades: 1, medkits: 1, tu: 0,
        }
    }

    pub fn campaign_progress(&self) -> f32 { (self.missions_won as f32 / 12.0).min(1.0) }

    pub fn available_weapons(&self) -> Vec<usize> {
        let prog = self.missions_won;
        self.data.weapons.iter().enumerate().filter(|(_, w)| {
            if w.game == 1 && !self.water_unlocked { return false; }
            w.unlock <= prog
        }).map(|(i, _)| i).collect()
    }

    pub fn best_weapon(&self, underwater: bool) -> usize {
        let avail = self.available_weapons();
        let mut best = avail[0];
        let mut best_score = -1;
        for i in avail {
            let w = &self.data.weapons[i];
            if w.cat == Cat::Grenade || w.cat == Cat::Melee || w.cat == Cat::Heal { continue; }
            if underwater && w.env == Env::Surface { continue; }
            let dmg = self.data.weapon_damage(w);
            let acc = w.item;
            let _ = acc;
            let score = dmg * 10 + self.data.item(w.game, w.item).acc[2] as i32;
            if score > best_score { best_score = score; best = i; }
        }
        best
    }

    pub fn equip_default(&mut self, idx: usize, underwater: bool) {
        let w = self.best_weapon(underwater);
        let s = &mut self.soldiers[idx];
        s.weapon = w;
        s.ammo = self.data.clip_size(&self.data.weapons[w]);
        s.spare = 2;
        let armor = if underwater {
            if self.water_missions >= 2 { 5 } else { 3 }
        } else if self.missions_won >= 5 { 2 } else if self.missions_won >= 2 { 1 } else { 0 };
        s.armor = armor;
    }

    pub fn ready_soldiers(&self) -> usize {
        self.soldiers.iter().filter(|s| s.assigned && !s.dead && s.wounded_days == 0).count()
    }

    pub fn select_soldier_for_edit(&mut self, idx: usize) {
        if idx < self.soldiers.len() { self.selected_soldier = idx; }
    }

    pub fn selected_site_ref(&self) -> Option<&Site> {
        self.selected_site.and_then(|i| self.sites.get(i))
    }

    pub fn select_site(&mut self, i: usize) {
        self.selected_site = Some(i);
        self.selected_ufo = None;
        if let Some(s) = self.sites.get(i) {
            self.globe.auto_rotate = 8.0;
            self.globe.yaw = -s.lon.to_radians();
            self.globe.pitch = s.lat.to_radians();
        }
    }

    pub fn launch_selected_site(&mut self) {
        let Some(i) = self.selected_site else { self.toast("NO TARGET SELECTED"); return; };
        if self.ready_soldiers() == 0 { self.toast("NO READY SOLDIERS — VISIT THE BASE"); return; }
        let site = self.sites[i].clone();
        let biome = self.data.terrain_at(site.lat, site.lon);
        let underwater = site.underwater || (self.data.is_water(site.lat, site.lon) && self.water_unlocked);
        let spec = MissionSpec {
            name: site.name.clone(), lat: site.lat, lon: site.lon,
            underwater, raid: site.raid, biome: if self.data.is_water(site.lat, site.lon) { 13 } else { biome },
            aliens: site.aliens.clone(), tier: site.tier, site: Some(i), reward: site.reward,
            final_mission: site.kind == SiteKind::Final,
            month: self.date.month,
            minute_of_day: self.date.minute,
        };
        tactical::start(self, spec);
    }

    pub fn spawn_ufo(&mut self) {
        let underwater = self.campaign_type == 1;
        let lat = self.rng.range(-55, 60) as f32;
        let lon = self.rng.range(0, 359) as f32;
        let (lat, lon) = if underwater { self.find_water(lat, lon) } else {
            let (la, lo) = self.find_land(lat, lon);
            if self.rng.chance(15) { (lat, lon) } else { (la, lo) }
        };
        let kind = if underwater {
            match self.rng.range(0, 3) { 0 => UfoKind::Rhizome, 1 => UfoKind::Rhizome, 2 => UfoKind::Taproot, _ => UfoKind::Rhizome }
        } else {
            match self.rng.range(0, 9) {
                0..=4 => UfoKind::Spore, 5..=7 => UfoKind::Burr, _ => UfoKind::Thorn,
            }
        };
        let raid = kind == UfoKind::Thorn;
        let (tlat, tlon) = if raid {
            let b = &self.bases[self.rng.range(0, self.bases.len() as i32 - 1) as usize];
            (b.lat, b.lon)
        } else {
            (lat + self.rng.range(-12, 12) as f32, lon + self.rng.range(-20, 20) as f32)
        };
        let hp = match kind { UfoKind::Spore => 60, UfoKind::Burr => 110, UfoKind::Thorn => 150, UfoKind::Rhizome => 120, UfoKind::Taproot => 220 };
        let u = Ufo { id: self.next_id(), lat, lon, target_lat: tlat, target_lon: tlon, kind, underwater, hp, timer: 720, detected: true, raid, speed: 0.4 };
        self.ufos.push(u);
        self.log(if underwater { "SONAR CONTACT — UNIDENTIFIED SUBMERSIBLE." } else { "RADAR CONTACT — UNIDENTIFIED CRAFT." }, AMBER);
        self.toast("NEW CONTACT DETECTED");
    }

    pub fn scramble_interceptor(&mut self, i: usize) {
        if i >= self.ufos.len() { return; }
        let u = &self.ufos[i];
        let is_sub = u.underwater;

        let mut best_craft: Option<(usize, usize)> = None;
        for (b_idx, base) in self.bases.iter().enumerate() {
            if is_sub != base.water { continue; }
            for (c_idx, c) in base.craft.iter().enumerate() {
                let def = craft_def(c.ctype);
                if def.weapon_mounts > 0 && c.ready && c.hp > 20 && c.fuel > 100 {
                    best_craft = Some((b_idx, c_idx));
                    break;
                }
            }
            if best_craft.is_some() { break; }
        }

        if best_craft.is_none() {
            for (b_idx, base) in self.bases.iter().enumerate() {
                for (c_idx, c) in base.craft.iter().enumerate() {
                    let def = craft_def(c.ctype);
                    if def.weapon_mounts > 0 && c.ready && c.hp > 20 && c.fuel > 100 {
                        best_craft = Some((b_idx, c_idx));
                        break;
                    }
                }
                if best_craft.is_some() { break; }
            }
        }

        if let Some((b_idx, c_idx)) = best_craft {
            self.dogfight = Some(DogfightState::new(self, i, b_idx, c_idx));
            self.screen = Screen::Dogfight;
            self.selected_ufo = None;
            self.toast("INTERCEPT VECTOR ENGAGED");
        } else {
            self.toast("NO OPERATIONAL COMBAT CRAFT");
            self.log("CANNOT INTERCEPT — NO READY COMBAT CRAFT IN HANGARS.", ui::RED);
        }
    }

    pub fn find_nearest_nation(&self, lat: f32, lon: f32, underwater: bool) -> Option<usize> {
        let target_game = if underwater { 1 } else { 0 };
        self.nations.iter().enumerate()
            .filter(|(_, n)| n.game == target_game)
            .min_by(|(_, a), (_, b)| {
                let da = (a.lat - lat).powi(2) + (a.lon - lon).powi(2);
                let db = (b.lat - lat).powi(2) + (b.lon - lon).powi(2);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
    }

    pub fn intercept_ufo(&mut self, i: usize) {
        if i >= self.ufos.len() { return; }
        let u = self.ufos[i].clone();
        let chance = match u.kind { UfoKind::Spore => 82, UfoKind::Burr => 68, UfoKind::Thorn => 58, UfoKind::Rhizome => 64, UfoKind::Taproot => 44 };
        if self.rng.chance(chance) {
            let (lat, lon) = if u.underwater { self.find_water(u.lat, u.lon) } else { self.find_land(u.lat, u.lon) };
            let tier = self.missions_won / 3 + 1;
            let n = self.rng.range(4, 6) as usize;
            let mut aliens = Vec::new();
            for _ in 0..n {
                aliens.push(self.roll_alien(u.underwater, tier));
            }
            let site = Site {
                id: self.next_id(),
                name: if u.underwater { "SUNKEN CRAFT".to_string() } else { "CRASH SITE".to_string() },
                lat, lon, kind: SiteKind::Crash, underwater: u.underwater, raid: false,
                expires: 2400, aliens, tier, reward: 160_000 + tier as i64 * 20_000,
            };
            self.sites.push(site);
            self.log(format!("{} DESTROYED. CRASH SITE MARKED.", if u.underwater { "SUBMERSIBLE" } else { "CRAFT" }), ui::GREEN);
            self.ufos.remove(i);
            self.selected_ufo = None;
            self.flash = 0.7;
        } else {
            self.ufos.remove(i);
            self.selected_ufo = None;
            self.log("INTERCEPTION FAILED — CONTACT ESCAPED.", ui::RED);
        }
    }

    pub fn roll_alien(&mut self, underwater: bool, tier: u32) -> u8 {
        if underwater {
            let pool: &[u8] = match tier { 0..=1 => &[0, 0, 1, 1, 5], 2..=3 => &[1, 2, 2, 5, 6], _ => &[2, 2, 3, 4, 7, 8] };
            *self.rng.pick(pool)
        } else {
            let pool: &[u8] = match tier { 0..=1 => &[0, 0, 1, 5], 2..=3 => &[1, 2, 2, 6], 4..=5 => &[2, 3, 3, 7], _ => &[3, 4, 4, 8, 9] };
            *self.rng.pick(pool)
        }
    }

    pub fn make_final_mission(&mut self, underwater: bool) {
        // Only one final site of each kind may be on the globe at a time.
        if self.sites.iter().any(|s| s.kind == SiteKind::Final && s.underwater == underwater) { return; }
        let (lat, lon) = if underwater { self.find_water(-47.0, -126.0) } else { self.find_land(10.0, 160.0) };
        let name = if underwater { "VERITY DEEP // FINAL ASSAULT".to_string() } else { "LUNAR FAR SIDE // FINAL ASSAULT".to_string() };
        let n = 8usize;
        let mut aliens = Vec::new();
        for _ in 0..n { aliens.push(self.roll_alien(underwater, 6)); }
        let sid = self.next_id();
        self.sites.push(Site {
            id: sid, name, lat, lon, kind: SiteKind::Final, underwater,
            raid: false, expires: 100000, aliens, tier: 6, reward: 1_500_000,
        });
        self.selected_site = Some(self.sites.len() - 1);
        self.log("FINAL ASSAULT SITE LOCATED. THIS IS THE END.", AMBER);
    }

    pub fn finish_battle(&mut self, victory: bool, kills: u32, lost: u32, site: Option<usize>, reward: i64, final_mission: bool, live_captured: Vec<CapturedAlienSpec>) {
        self.last_kills = kills;
        self.last_lost = lost;
        self.last_reward = if victory { reward } else { 0 };
        let final_underwater = site.and_then(|i| self.sites.get(i)).map(|s| s.underwater).unwrap_or(false);
        let event_shown = false;
        if victory {
            self.missions_won += 1;
            if self.water_unlocked { self.water_missions += 1; }
            self.funds += reward;
            self.score += 100 + kills as i32 * 20 - lost as i32 * 15;
            self.log(format!("MISSION COMPLETE. {} HOSTILES NEUTRALISED. +${}", kills, fmt_money(reward)), ui::GREEN);

            // Handle live captured alien specimens
            if !live_captured.is_empty() {
                let mut admitted = 0;
                let mut died = 0;
                if let Some(b) = self.bases.get_mut(self.selected_base) {
                    let cap = b.holding_cap();
                    let current = b.captured_aliens.len();
                    let free_slots = cap.saturating_sub(current);
                    for alien in live_captured {
                        if admitted < free_slots {
                            b.captured_aliens.push(alien);
                            admitted += 1;
                        } else {
                            died += 1;
                        }
                    }
                }
                if admitted > 0 {
                    self.log(format!("LIVE CAPTIVES RECOVERED: {} MOVED TO HOLDING.", admitted), ui::GREEN);
                }
                if died > 0 {
                    self.log("LIVE CAPTIVE LOST: NO ROOM IN HOLDING.", ui::RED);
                }
            }
            // Surviving soldiers gain combat stat increases based on survival and kills!
            for s in self.soldiers.iter_mut() {
                if s.assigned && !s.dead {
                    s.stats.health = (s.stats.health + 1).min(80);
                    s.stats.tu = (s.stats.tu + 1).min(80);
                    s.stats.stamina = (s.stats.stamina + 2).min(100);
                    s.stats.reactions = (s.stats.reactions + 1).min(100);
                    s.stats.firing = (s.stats.firing + 2).min(120);
                    s.stats.throwing = (s.stats.throwing + 1).min(120);
                    s.stats.strength = (s.stats.strength + 1).min(70);
                    s.stats.bravery = (s.stats.bravery + 5).min(100);
                }
            }
            // Materials and enemy tools brought back go into the active base
            if let Some(b) = self.bases.get_mut(self.selected_base) {
                b.add_item("ALIEN CORPSE", kills as i32);
                if final_underwater {
                    b.nacre += 25;
                    b.brine += 10;
                    b.add_item("PRESSURE RAM", (kills / 3).max(1) as i32);
                    b.add_item("PRESSURE BORE", (kills / 2).max(1) as i32);
                } else {
                    b.lattice += 25;
                    b.electrolyte += 10;
                    b.add_item("ARC TORCH", (kills / 3).max(1) as i32);
                    b.add_item("ARC CUTTER", (kills / 2).max(1) as i32);
                    b.add_item("CUTTER CELL", kills as i32);
                }
            }

            // Award country activity and reassure funding nation
            if let Some(s_idx) = site {
                if let Some(s) = self.sites.get(s_idx) {
                    if let Some(n_idx) = self.find_nearest_nation(s.lat, s.lon, s.underwater) {
                        let boost = if s.raid { 280 } else { 130 };
                        self.nations[n_idx].defence_activity += boost;
                        self.nations[n_idx].panic = (self.nations[n_idx].panic - 25).max(0);
                        self.nations[n_idx].satisfaction = (self.nations[n_idx].satisfaction + 15).min(100);
                        self.log(format!("{} DEFENDED: +{} HEARTH ACTIVITY, PANIC REDUCED.", self.nations[n_idx].name, boost), ui::GREEN);
                    }
                }
            }
            // Monthly promotion check
            let awards = evaluate_force_promotions(&mut self.soldiers);
            for a in awards {
                self.log(format!("PROMOTION: {} -> {}", a.name, RANKS[a.new_rank as usize]), CYAN);
            }
            if final_mission && final_underwater {
                // Verity Deep: UNDERTOW is won.
                self.won = true;
                self.game_over = true;
                self.screen = Screen::End;
                return;
            }
            if final_mission && !final_underwater {
                // Lunar far side: LANDFALL is won.
                self.won = true;
                self.game_over = true;
                self.screen = Screen::End;
                return;
            }
            if self.campaign_type == 1 && self.water_missions >= 6 && !self.won {
                self.make_final_mission(true);
            }
            if self.campaign_type == 0 && self.missions_won >= 8 && !self.won {
                self.make_final_mission(false);
            }
        } else {
            self.score -= 60;
            if let Some(s_idx) = site {
                if let Some(s) = self.sites.get(s_idx) {
                    if let Some(n_idx) = self.find_nearest_nation(s.lat, s.lon, s.underwater) {
                        self.nations[n_idx].alien_activity += 180;
                        self.nations[n_idx].panic = (self.nations[n_idx].panic + 25).min(100);
                        self.nations[n_idx].satisfaction = (self.nations[n_idx].satisfaction - 20).max(0);
                        self.log(format!("MISSION FAILED IN {}: PANIC IS SPREADING.", self.nations[n_idx].name), ui::RED);
                    }
                }
            }
            self.log("MISSION FAILED. THE SQUAD IS LOST.", ui::RED);
            self.toast("MISSION FAILED");
        }
        if let Some(i) = site {
            if i < self.sites.len() { self.sites.remove(i); }
            self.selected_site = None;
        }
        if !event_shown { self.screen = Screen::Debrief; }
        self.paused = true;
    }

    pub fn monthly_tick(&mut self) {
        if self.date.month != self.last_month {
            let mut report = MonthlyReport::default();
            report.month = self.last_month;
            report.year = self.date.year;
            report.score = self.score;

            // 1. National Funding
            let mut total_inc = 0i64;
            for n in self.nations.iter_mut() {
                if n.pact_signed {
                    report.nation_changes.push((n.name.clone(), 0, -n.funding));
                    continue;
                }
                let net_act = n.defence_activity - n.alien_activity;
                // Funding change per state: a random 0–14 percent plus 5 percent
                let pct = (5 + self.rng.range(0, 16)) as f64 / 100.0;
                let delta = if net_act > 30 || n.satisfaction >= 80 {
                    n.satisfaction = (n.satisfaction + 8).min(100);
                    n.panic = (n.panic - 15).max(0);
                    (n.funding as f64 * pct) as i64
                } else if net_act < -30 || n.panic > 50 {
                    n.satisfaction = (n.satisfaction - 15).max(0);
                    n.panic = (n.panic + 18).min(100);
                    -((n.funding as f64 * pct) as i64)
                } else {
                    0
                };
                if n.satisfaction < 15 && n.panic >= 70 {
                    n.pact_signed = true;
                    n.funding = 0;
                    n.last_delta = -n.funding;
                    report.nation_changes.push((n.name.clone(), 0, -n.funding));
                    continue;
                }
                n.funding = (n.funding + delta).max(40_000);
                n.last_delta = delta;
                n.defence_activity = 0;
                n.alien_activity = 0;
                total_inc += n.funding;
                report.nation_changes.push((n.name.clone(), n.funding, delta));
            }
            report.total_funding = total_inc;

            // 2. Expenses
            let mut fac_maint = 0i64;
            let mut craft_maint = 0i64;
            let mut total_sci = 0i32;
            let mut total_eng = 0i32;
            for b in &self.bases {
                fac_maint += b.monthly_upkeep();
                for c in &b.craft {
                    craft_maint += craft_def(c.ctype).upkeep;
                }
                total_sci += b.scientists;
                total_eng += b.engineers;
            }
            let sold_sal = self.soldiers.iter().filter(|s| !s.dead).count() as i64 * 20_000;
            let sci_sal = total_sci as i64 * 30_000;
            let eng_sal = total_eng as i64 * 25_000;

            report.facility_maint = fac_maint;
            report.craft_maint = craft_maint;
            report.soldier_salaries = sold_sal;
            report.scientist_salaries = sci_sal;
            report.engineer_salaries = eng_sal;
            report.total_expenses = fac_maint + craft_maint + sold_sal + sci_sal + eng_sal;
            report.net_balance = report.total_funding - report.total_expenses;

            self.funds += report.net_balance;
            self.last_funding = report.total_funding;

            // 3. Assembly rating
            report.rating = if self.score > 500 {
                "EXCELLENT"
            } else if self.score > 0 {
                "GOOD"
            } else if self.score > -300 {
                "SATISFACTORY"
            } else if self.score > -600 {
                "POOR"
            } else {
                "TERRIBLE"
            };

            // Debt and termination rules
            if self.funds < -1_000_000 {
                self.log("WARNING: THE ASSEMBLY HAS SERVED A DEFICIT NOTICE. DEBT EXCEEDS $1,000,000.", AMBER);
            }

            // 4. Force-wide promotions
            let awards = evaluate_force_promotions(&mut self.soldiers);
            for a in awards {
                report.promotions.push(format!("{} PROMOTED TO {}", a.name, RANKS[a.new_rank as usize]));
                self.log(format!("PROMOTION: {} -> {}", a.name, RANKS[a.new_rank as usize]), CYAN);
            }

            self.last_month = self.date.month;
            self.last_report = Some(report);
            self.screen = Screen::AssemblyReport;
            self.paused = true;
            self.log("ASSEMBLY REVIEW RECEIVED. OPENING THE ACCOUNTS.", ui::GREEN);
        }
    }

    pub fn daily_tick(&mut self) {
        for b_idx in 0..self.bases.len() {
            // 1. Facilities construction
            let mut completed_facs = Vec::new();
            {
                let b = &mut self.bases[b_idx];
                for f in &mut b.facilities {
                    if f.days_left > 0 {
                        f.days_left -= 1;
                        if f.days_left == 0 {
                            completed_facs.push((f.ftype, b.water));
                        }
                    }
                }
            }
            for (ftype, water) in completed_facs {
                let name = if water { facility_def(ftype).name_water } else { facility_def(ftype).name_surface };
                self.log(format!("FACILITY ONLINE: {}", name), ui::GREEN);
            }

            // 2. Active Research
            let mut tech_finished = None;
            {
                let b = &mut self.bases[b_idx];
                if let Some(res) = &mut b.active_research {
                    res.progress += res.scientists * 24;
                    if res.progress >= res.total_points {
                        tech_finished = Some(res.tech_id.clone());
                    }
                }
                if tech_finished.is_some() {
                    b.active_research = None;
                }
            }
            if let Some(tid) = tech_finished {
                if !self.researched_techs.contains(&tid) {
                    self.researched_techs.push(tid.clone());
                }
                if tid.starts_with("STUDY_") {
                    self.score += 150;
                    self.log(format!("LIVE STUDY COMPLETE: {}. The subject did not survive it.", tid), ui::GREEN);
                    if let Some(b) = self.bases.get_mut(b_idx) {
                        let key = tid.trim_start_matches("STUDY_").replace('_', " ");
                        if let Some(pos) = b.captured_aliens.iter().position(|a| a.name.to_uppercase().contains(&key) && !a.studied) {
                            b.captured_aliens[pos].studied = true;
                        }
                    }
                } else {
                    self.log(format!("RESEARCH COMPLETE: {}", tid), CYAN);
                }

                if tid == "SEED_LOCATION" {
                    self.make_final_mission(false);
                    self.log("THE SEED IS FOUND: LUNAR FAR SIDE ASSAULT AVAILABLE ON THE SITUATION MAP.", AMBER);
                    self.toast("FAR SIDE ASSAULT UNLOCKED");
                } else if tid == "VERITY_DEEP" {
                    self.make_final_mission(true);
                    self.log("THE ROOT IS FOUND: VERITY DEEP ASSAULT AVAILABLE ON THE SITUATION MAP.", CYAN);
                    self.toast("VERITY DEEP ASSAULT UNLOCKED");
                } else {
                    self.toast("TECHNOLOGY RESEARCHED");
                }
            }

            // 3. Active Production
            let mut prod_finished = None;
            {
                let b = &mut self.bases[b_idx];
                if let Some(prod) = &mut b.active_production {
                    prod.progress += prod.engineers * 24;
                    if prod.progress >= prod.total_hours {
                        prod_finished = Some(prod.blueprint_id.clone());
                    }
                }
                if prod_finished.is_some() {
                    b.active_production = None;
                }
            }
            if let Some(bpid) = prod_finished {
                let bp_def = WORKSHOP_BLUEPRINTS.iter().find(|bp| bp.id == bpid);
                let bp_name = bp_def.map(|bp| bp.name).unwrap_or(&bpid);
                self.log(format!("MANUFACTURING COMPLETE: {}", bp_name), ui::GREEN);
                self.toast("PRODUCTION FINISHED");

                let b = &mut self.bases[b_idx];
                match bpid.as_str() {
                    "PEREGRINE" => {
                        let cname = format!("PEREGRINE-{}", b.craft.len() + 1);
                        b.craft.push(CraftInstance {
                            name: cname,
                            ctype: CraftType::Peregrine,
                            ready: true,
                            hp: 240,
                            max_hp: 240,
                            fuel: 24,
                            max_fuel: 24,
                            weapons: [Some(CraftWeaponType::ArcProjector), None],
                            ammo: [80, 0],
                        });
                        self.log("PEREGRINE INTERCEPTOR DELIVERED TO HANGAR.", CYAN);
                    }
                    "ALBATROSS" => {
                        let cname = format!("ALBATROSS-{}", b.craft.len() + 1);
                        b.craft.push(CraftInstance {
                            name: cname,
                            ctype: CraftType::Albatross,
                            ready: true,
                            hp: 480,
                            max_hp: 480,
                            fuel: 64,
                            max_fuel: 64,
                            weapons: [Some(CraftWeaponType::ArcProjector), Some(CraftWeaponType::BreacherMissile)],
                            ammo: [80, 2],
                        });
                        self.log("ALBATROSS TRANSPORT DELIVERED TO HANGAR.", AMBER);
                    }
                    "CORMORANT" => {
                        let cname = format!("CORMORANT-{}", b.craft.len() + 1);
                        b.craft.push(CraftInstance {
                            name: cname,
                            ctype: CraftType::Cormorant,
                            ready: true,
                            hp: 260,
                            max_hp: 260,
                            fuel: 28,
                            max_fuel: 28,
                            weapons: [Some(CraftWeaponType::CavCannon), None],
                            ammo: [60, 0],
                        });
                        self.log("CORMORANT ATTACK SUB DELIVERED TO WET DOCK.", CYAN);
                    }
                    "EMPEROR" => {
                        let cname = format!("EMPEROR-{}", b.craft.len() + 1);
                        b.craft.push(CraftInstance {
                            name: cname,
                            ctype: CraftType::Emperor,
                            ready: true,
                            hp: 520,
                            max_hp: 520,
                            fuel: 64,
                            max_fuel: 64,
                            weapons: [Some(CraftWeaponType::CavCannon), Some(CraftWeaponType::CavCannon)],
                            ammo: [60, 60],
                        });
                        self.log("EMPEROR TRANSPORT SUB DELIVERED TO WET DOCK.", AMBER);
                    }
                    other => {
                        let item_name = bp_def.map(|bp| bp.name).unwrap_or(other);
                        b.add_item(item_name, 1);
                        self.log(format!("{} TRANSFERRED TO ARMORY STORES.", item_name), ui::GREEN);
                    }
                }
            }

            // 4. Craft maintenance & repair
            let b = &mut self.bases[b_idx];
            for c in &mut b.craft {
                if c.hp < c.max_hp { c.hp = (c.hp + 20).min(c.max_hp); }
                if c.fuel < c.max_fuel { c.fuel = (c.fuel + 150).min(c.max_fuel); }
                c.ready = c.hp >= c.max_hp * 8 / 10 && c.fuel >= c.max_fuel / 2;
            }
        }

        // 5. Soldier wound recovery
        let mut recovered_names = Vec::new();
        for s in &mut self.soldiers {
            if s.wounded_days > 0 {
                s.wounded_days -= 1;
                if s.wounded_days == 0 {
                    s.hp = s.stats.health as i32;
                    s.wounds = 0;
                    recovered_names.push(s.name.clone());
                }
            }
        }
        for name in recovered_names {
            self.log(format!("SOLDIER RECOVERED: {} IS COMBAT READY.", name), ui::GREEN);
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.frame += 1;
        self.elapsed += dt;
        self.title_pulse += dt;
        if let Some((_, t)) = &mut self.toast { *t -= dt; if *t <= 0.0 { self.toast = None; } }
        self.flash = (self.flash - dt * 1.8).max(0.0);
        for l in &mut self.log { l.age += dt; }
        if self.screen == Screen::Tactical {
            tactical::update(self, dt);
            return;
        }
        if matches!(self.screen, Screen::Title | Screen::Event | Screen::Debrief | Screen::End | Screen::AssemblyReport) {
            // still animate the globe
            self.globe.time += dt;
            return;
        }
        // ---- situation map time
        if !self.paused && self.screen == Screen::SituationMap {
            let mins = (dt * SPEEDS[self.speed] as f32) as i32;
            if mins > 0 {
                self.date.add_minutes(mins);
                self.spawn_timer -= mins;
                if self.spawn_timer <= 0 {
                    self.spawn_ufo();
                    self.spawn_timer = self.rng.range(600, 1600);
                }
                for s in self.sites.iter_mut() { s.expires -= mins; }
                let expired: Vec<usize> = self.sites.iter().enumerate().filter(|(_, s)| s.expires <= 0 && s.kind != SiteKind::Final).map(|(i, _)| i).collect();
                for i in expired.into_iter().rev() {
                    let s = self.sites.remove(i);
                    self.score -= 30;
                    self.log(format!("{} WENT COLD. THE ALIENS ESCAPED.", s.name), ui::RED);
                }
                self.update_ufos(mins as f32);

                self.minutes_accumulator += mins;
                while self.minutes_accumulator >= 1440 {
                    self.minutes_accumulator -= 1440;
                    self.daily_tick();
                }

                self.monthly_tick();
            }
        }
        if self.screen == Screen::SituationMap {
            self.update_situation_map_input(dt);
        }
        if self.screen == Screen::Base { base::update(self, dt); }
        if self.screen == Screen::Editor { base::update_editor(self, dt); }
        if self.screen == Screen::Dogfight { dogfight::update(self, dt); }
    }

    fn update_ufos(&mut self, mins: f32) {
        let mut remove = Vec::new();
        for (i, u) in self.ufos.iter_mut().enumerate() {
            u.timer -= mins as i32;
            let dlat = u.target_lat - u.lat;
            let mut dlon = u.target_lon - u.lon;
            if dlon > 180.0 { dlon -= 360.0; }
            if dlon < -180.0 { dlon += 360.0; }
            let dist = (dlat * dlat + dlon * dlon).sqrt();
            if dist < 2.0 || u.timer <= 0 {
                remove.push((i, u.clone()));
            } else {
                let step = (u.speed * mins / 120.0).min(dist);
                u.lat += dlat / dist * step;
                u.lon += dlon / dist * step;
            }
        }
        for (i, u) in remove.into_iter().rev() {
            if u.raid {
                let (lat, lon) = self.find_land(u.lat, u.lon);
                let tier = self.missions_won / 3 + 1;
                let mut aliens = Vec::new();
                for _ in 0..self.rng.range(5, 7) { aliens.push(self.roll_alien(u.underwater, tier)); }
                let sid = self.next_id();
                self.sites.push(Site {
                    id: sid, name: "CITY RAID".into(), lat, lon,
                    kind: SiteKind::Raid, underwater: u.underwater, raid: true,
                    expires: 3600, aliens, tier, reward: 240_000,
                });
                self.log("CITY RAID — CIVILIANS ARE DYING.", ui::RED);
                self.toast("CITY RAID REPORTED");
            } else {
                let (lat, lon) = if u.underwater { self.find_water(u.lat, u.lon) } else { self.find_land(u.lat, u.lon) };
                let tier = self.missions_won / 3 + 1;
                let mut aliens = Vec::new();
                for _ in 0..self.rng.range(4, 6) { aliens.push(self.roll_alien(u.underwater, tier)); }
                let sid = self.next_id();
                self.sites.push(Site {
                    id: sid, name: if u.underwater { "ALIEN ACTIVITY // DEEP".into() } else { "LANDING SITE".into() },
                    lat, lon, kind: SiteKind::Crash, underwater: u.underwater, raid: false,
                    expires: 3000, aliens, tier, reward: 150_000,
                });
                self.log("CONTACT HAS LANDED. ASSAULT TEAM READY.", AMBER);
            }
            self.ufos.remove(i);
            self.selected_ufo = None;
        }
    }

    fn update_situation_map_input(&mut self, dt: f32) {
        let w = screen_width(); let h = screen_height();
        let panel_w = (w * 0.34).clamp(250.0, 360.0);
        let blocked = {
            let (mx, my) = mouse_position();
            mx > w - panel_w || my < 46.0 || my > h - 58.0
        };
        // placing a base takes over the click
        if self.placing_base {
            let (mx, my) = mouse_position();
            if is_mouse_button_released(MouseButton::Left) && my > 46.0 && my < h - 58.0 {
                if let Some((lat, lon)) = self.globe.screen_to_latlon(vec2(mx, my)) {
                    let water = self.data.is_water(lat, lon);
                    if self.campaign_type == 1 {
                        if water {
                            if self.funds >= 1_500_000 {
                                self.funds -= 1_500_000;
                                self.add_base(lat, lon, true);
                            } else {
                                self.toast("INSUFFICIENT FUNDS ($1.5M REQUIRED)");
                            }
                        } else {
                            self.toast("UNDERTOW: STATIONS MUST BE PLACED ON THE SEA FLOOR");
                        }
                    } else {
                        if !water {
                            if self.funds >= 1_000_000 {
                                self.funds -= 1_000_000;
                                self.add_base(lat, lon, false);
                            } else {
                                self.toast("INSUFFICIENT FUNDS ($1.0M REQUIRED)");
                            }
                        } else {
                            self.toast("LANDFALL: BASES MUST BE PLACED ON LAND");
                        }
                    }
                }
                self.placing_base = false;
            }
            if is_key_pressed(KeyCode::Escape) { self.placing_base = false; }
            return;
        }
        let clicked = self.globe.handle_input(dt, blocked, true);
        let anim_done = self.globe.update_anim(dt);
        // top bar time buttons are handled in draw_situation_map (immediate mode)
        if let Some(mp) = clicked {
            // marker hit tests, nearest first
            let mut hit: Option<(u8, usize, f32)> = None;
            for (i, b) in self.bases.iter().enumerate() {
                if let Some((p, _)) = self.globe.project(b.lat, b.lon) {
                    let d = p.distance(mp);
                    if d < 20.0 { hit = Some((0, i, d)); }
                }
            }
            for (i, u) in self.ufos.iter().enumerate() {
                if let Some((p, _)) = self.globe.project(u.lat, u.lon) {
                    let d = p.distance(mp);
                    if d < 20.0 && hit.map(|x| d < x.2).unwrap_or(true) { hit = Some((1, i, d)); }
                }
            }
            for (i, s) in self.sites.iter().enumerate() {
                if let Some((p, _)) = self.globe.project(s.lat, s.lon) {
                    let d = p.distance(mp);
                    if d < 22.0 && hit.map(|x| d < x.2).unwrap_or(true) { hit = Some((2, i, d)); }
                }
            }
            match hit {
                Some((0, i, _)) => {
                    self.selected_base = i;
                    self.base_return_zoom = self.globe.zoom;
                    let b = self.bases[i].clone();
                    self.globe.start_zoom(ZoomDir::In, b.lat, b.lon, 1.0);
                    self.flash = 0.4;
                }
                Some((1, i, _)) => { self.selected_ufo = Some(i); self.selected_site = None; }
                Some((2, i, _)) => { self.select_site(i); }
                _ => {}
            }
        }
        // finish the Google-Maps-style dive into the base
        if let Some(ZoomDir::In) = anim_done {
            self.screen = Screen::Base;
            self.paused = true;
            self.base_tab = 0;
            self.selected_soldier = self.selected_soldier.min(self.soldiers.len().saturating_sub(1));
        }
    }

    pub fn add_base(&mut self, lat: f32, lon: f32, water: bool) {
        let cost = if water { 1_500_000 } else { 1_000_000 };
        if self.funds < cost { self.toast("INSUFFICIENT FUNDS"); return; }
        self.funds -= cost;
        let idx = self.bases.len();
        let name = if water { format!("HEARTH DEEP {}", idx + 1) } else { format!("HEARTH {}", ["ALPHA","BRAVO","CHARLIE","DELTA","ECHO","FOXTROT","GOLF","HOTEL"][idx.min(7)]) };
        self.bases.push(Base::new(name.clone(), lat, lon, water));
        self.selected_base = idx;
        self.log(format!("{} COMMISSIONED.", name), ui::GREEN);
        self.toast("BASE ONLINE");
        self.flash = 0.5;
        self.base_return_zoom = 1.0;
        let b = self.bases[idx].clone();
        self.globe.start_zoom(ZoomDir::In, b.lat, b.lon, 1.0);
    }

    pub fn return_to_globe(&mut self) {
        if let Some(b) = self.bases.get(self.selected_base) {
            let back = self.base_return_zoom.max(0.85);
            self.globe.start_zoom(ZoomDir::Out, b.lat, b.lon, back);
        }
        self.screen = Screen::SituationMap;
        self.paused = false;
        self.flash = 0.35;
    }

    // ================================================================ draw
    pub fn draw(&mut self) {
        clear_background(BG);
        // The globe's sun follows the campaign clock (or `--clock`).
        let now = self.dbg_clock.unwrap_or_else(|| sky::Moment::new(self.date.year, self.date.month as i32 + 1, self.date.day, self.date.minute as f32));
        self.globe.sun = Vec3::from(sky::globe_sun_dir(now));
        match self.screen {
            Screen::Title => self.draw_title(),
            Screen::CampaignSelect => self.draw_campaign_select(),
            Screen::SituationMap => self.draw_situation_map(),
            Screen::Base => base::draw(self),
            Screen::Editor => base::draw_editor(self),
            Screen::Tactical => tactical::draw(self),
            Screen::Debrief => { self.draw_situation_map(); self.draw_debrief(); }
            Screen::Event => { self.draw_situation_map(); self.draw_event(); }
            Screen::End => { self.draw_situation_map(); self.draw_end(); }
            Screen::AssemblyReport => { self.draw_situation_map(); self.draw_assembly_report(); }
            Screen::Dogfight => dogfight::draw(self),
        }
        vignette();
        scanlines(0.035);
        if let Some((text, t)) = &self.toast {
            let a = (t / 0.5).clamp(0.0, 1.0);
            let w = self.ui.width(14.0, text) + 44.0;
            let ty = if self.screen == Screen::Tactical { screen_height() - 150.0 } else { screen_height() - 92.0 };
            let r = Rect::new((screen_width() - w) * 0.5, ty, w, 30.0);
            panel(r, Color { a: 0.9 * a, ..PANEL_2 }, Color { a, ..AMBER });
            self.ui.center(r.x + r.w * 0.5, r.y + 20.0, 14.0, Color { a, ..AMBER }, text);
        }
        if self.flash > 0.01 {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color { r: 0.7, g: 0.9, b: 1.0, a: self.flash * 0.25 });
        }
    }

    fn draw_title(&mut self) {
        let t = self.elapsed;
        self.globe.time = t;
        self.globe.yaw += 0.0006;
        self.globe.draw(t);
        let w = screen_width(); let h = screen_height();
        // dark title scrim
        for i in 0..24 {
            let f = i as f32 / 24.0;
            draw_rectangle(0.0, 0.0, w * (0.58 - f * 0.30), h, Color { r: 0.01, g: 0.02, b: 0.03, a: 0.035 });
        }
        let x = w * 0.07;
        let y = h * 0.30;
        self.ui.label(x, y - 34.0, 12.0, AMBER, "JOINT TASK FORCE HEARTH // 2027");
        self.ui.bold(x, y + 18.0, 46.0, TEXT, "DEFENDING");
        self.ui.bold(x, y + 62.0, 46.0, CYAN, "EARTH");
        draw_rectangle(x, y + 78.0, 260.0, 2.0, CYAN);
        self.ui.text(x, y + 106.0, 15.0, TEXT, "Squad tactics for a war that starts tomorrow.");
        self.ui.text(x, y + 128.0, 12.0, DIM, "Every shot, every roll, every consequence.");
        let r = Rect::new(x, y + 160.0, 230.0, 44.0);
        if button(&self.ui, r, "NEW CAMPAIGN", Btn::Primary, true) {
            self.screen = Screen::CampaignSelect;
        }
        let r2 = Rect::new(x, y + 212.0, 230.0, 32.0);
        if button(&self.ui, r2, "QUIT", Btn::Normal, true) { std::process::exit(0); }
        let r3 = Rect::new(x, y + 252.0, 230.0, 28.0);
        if button(&self.ui, r3, "CONTROLS & HELP", Btn::Ghost, true) { self.show_help = !self.show_help; }
        if self.show_help {
            let hr = Rect::new(w * 0.05, h * 0.62, w * 0.40, h * 0.30);
            panel(hr, PANEL, CYAN_DIM);
            self.ui.bold(hr.x + 14.0, hr.y + 24.0, 13.0, CYAN, "SITUATION MAP");
            self.ui.text(hr.x + 14.0, hr.y + 44.0, 11.0, TEXT, "Drag globe to rotate. Wheel to zoom.");
            self.ui.text(hr.x + 14.0, hr.y + 60.0, 11.0, TEXT, "Click a base to dive in; wheel back out.");
            self.ui.text(hr.x + 14.0, hr.y + 76.0, 11.0, TEXT, "Click sites and contacts to act on them.");
            self.ui.bold(hr.x + 14.0, hr.y + 100.0, 13.0, CYAN, "FIELD VIEW");
            self.ui.text(hr.x + 14.0, hr.y + 120.0, 11.0, TEXT, "LMB select / move. RMB fire. 1-2-3 shot mode.");
            self.ui.text(hr.x + 14.0, hr.y + 136.0, 11.0, TEXT, "K kneel. R reload. G grenade. H trauma kit. SPACE end turn.");
            self.ui.text(hr.x + 14.0, hr.y + 152.0, 11.0, TEXT, "Q/E rotate view, wheel zoom, RMB drag pan.");
        }
        self.ui.text(x, h - 24.0, 10.0, DIM, "DEFENDING EARTH 0.9 // DEVELOPMENT BUILD");
    }

    fn draw_campaign_select(&mut self) {
        let t = self.elapsed;
        self.globe.time = t;
        self.globe.yaw += 0.0006;
        self.globe.draw(t);
        let w = screen_width(); let h = screen_height();

        // Dark background scrim
        draw_rectangle(0.0, 0.0, w, h, Color { r: 0.01, g: 0.02, b: 0.035, a: 0.88 });

        // Header
        let cy = 50.0;
        self.ui.label(w * 0.5 - self.ui.width(13.0, "STRATEGIC DEFENSE PROTOCOL // CHOOSE YOUR CAMPAIGN") * 0.5, cy, 13.0, AMBER, "STRATEGIC DEFENSE PROTOCOL // CHOOSE YOUR CAMPAIGN");
        self.ui.bcenter(w * 0.5, cy + 32.0, 32.0, TEXT, "SELECT OPERATIONAL THEATER");
        draw_rectangle(w * 0.5 - 180.0, cy + 46.0, 360.0, 2.0, CYAN);

        let card_w = (w * 0.42).clamp(340.0, 500.0);
        let card_h = (h * 0.68).clamp(420.0, 560.0);
        let gap = 36.0;
        let left_x = w * 0.5 - card_w - gap * 0.5;
        let right_x = w * 0.5 + gap * 0.5;
        let card_y = cy + 66.0;

        // ---------- CARD 1: LANDFALL (2027) ----------
        let r1 = Rect::new(left_x, card_y, card_w, card_h);
        let hover1 = r1.contains(vec2(mouse_position().0, mouse_position().1));
        panel(r1, if hover1 { Color { r: 0.02, g: 0.08, b: 0.07, a: 0.94 } } else { Color { r: 0.015, g: 0.045, b: 0.04, a: 0.90 } }, if hover1 { ui::GREEN } else { Color { r: 0.2, g: 0.6, b: 0.35, a: 0.6 } });

        let mut y = r1.y + 26.0;
        self.ui.label(r1.x + 22.0, y, 11.0, AMBER, "OPERATION LANDFALL // 2027");
        y += 24.0;
        self.ui.bold(r1.x + 22.0, y, 24.0, ui::GREEN, "LANDFALL");
        y += 10.0;
        draw_rectangle(r1.x + 22.0, y, r1.w - 44.0, 1.0, Color { a: 0.4, ..ui::GREEN });
        y += 18.0;

        self.ui.bold(r1.x + 22.0, y, 12.0, TEXT, "THEATRE: Land and Air");
        y += 18.0;
        self.ui.text(r1.x + 22.0, y, 10.5, DIM, "Unidentified craft are coming down over land, and they are not here to talk.");
        y += 15.0;
        self.ui.text(r1.x + 22.0, y, 10.5, DIM, "They take people and livestock, raid cities, and lean on governments to stand aside.");
        y += 15.0;
        self.ui.text(r1.x + 22.0, y, 10.5, DIM, "Run Joint Task Force HEARTH from bases on land.");
        y += 26.0;

        self.ui.bold(r1.x + 22.0, y, 11.5, CYAN, "COMMAND SPECIFICATIONS:");
        y += 18.0;
        self.ui.text(r1.x + 22.0, y, 10.5, TEXT, "• AIRCRAFT: Heron twin-rotor transport, Swift interceptor");
        y += 16.0;
        self.ui.text(r1.x + 22.0, y, 10.5, TEXT, "• ARSENAL: Service rifles and machine guns now; cored and arc weapons later");
        y += 16.0;
        self.ui.text(r1.x + 22.0, y, 10.5, TEXT, "• FACILITIES: Search radar, barracks, labs, holding block");
        y += 16.0;
        self.ui.text(r1.x + 22.0, y, 10.5, TEXT, "• FUNDING: 16 Charter states ($6,000,000/mo to start)");
        y += 16.0;
        self.ui.text(r1.x + 22.0, y, 10.5, TEXT, "• FINAL OBJECTIVE: The Seed, far side of the Moon");

        let btn_rect1 = Rect::new(r1.x + 22.0, r1.y + r1.h - 52.0, r1.w - 44.0, 38.0);
        if button(&self.ui, btn_rect1, "BEGIN LANDFALL [2027]", Btn::Normal, true) {
            self.new_campaign(0);
        }

        // ---------- CARD 2: UNDERTOW (2027) ----------
        let r2 = Rect::new(right_x, card_y, card_w, card_h);
        let hover2 = r2.contains(vec2(mouse_position().0, mouse_position().1));
        panel(r2, if hover2 { Color { r: 0.02, g: 0.06, b: 0.10, a: 0.94 } } else { Color { r: 0.015, g: 0.035, b: 0.06, a: 0.90 } }, if hover2 { CYAN } else { Color { r: 0.15, g: 0.45, b: 0.65, a: 0.6 } });

        let mut y2 = r2.y + 26.0;
        self.ui.label(r2.x + 22.0, y2, 11.0, AMBER, "OPERATION UNDERTOW // 2027");
        y2 += 24.0;
        self.ui.bold(r2.x + 22.0, y2, 24.0, CYAN, "UNDERTOW");
        y2 += 10.0;
        draw_rectangle(r2.x + 22.0, y2, r2.w - 44.0, 1.0, Color { a: 0.4, ..CYAN });
        y2 += 18.0;

        self.ui.bold(r2.x + 22.0, y2, 12.0, TEXT, "THEATRE: Sea Floor and Trenches");
        y2 += 18.0;
        self.ui.text(r2.x + 22.0, y2, 10.5, DIM, "The same landings, seen from below: craft that enter the sea and do not come back up.");
        y2 += 15.0;
        self.ui.text(r2.x + 22.0, y2, 10.5, DIM, "Contacts are working the deep trenches, cutting cables and shipping lanes above.");
        y2 += 15.0;
        self.ui.text(r2.x + 22.0, y2, 10.5, DIM, "Command sea-floor stations with pressure-rated kit and dive weapons.");
        y2 += 26.0;

        self.ui.bold(r2.x + 22.0, y2, 11.5, CYAN, "COMMAND SPECIFICATIONS:");
        y2 += 18.0;
        self.ui.text(r2.x + 22.0, y2, 10.5, TEXT, "• CRAFT: Gannet transport sub, Skua attack sub");
        y2 += 16.0;
        self.ui.text(r2.x + 22.0, y2, 10.5, TEXT, "• ARSENAL: Dive rifles and bolt guns now; cav and pressure weapons later");
        y2 += 16.0;
        self.ui.text(r2.x + 22.0, y2, 10.5, TEXT, "• FACILITIES: Wet docks, search sonar, wet labs, holding tank");
        y2 += 16.0;
        self.ui.text(r2.x + 22.0, y2, 10.5, TEXT, "• FUNDING: 13 sea-lane groups ($6,000,000/mo to start)");
        y2 += 16.0;
        self.ui.text(r2.x + 22.0, y2, 10.5, TEXT, "• FINAL OBJECTIVE: The Root, Verity Deep, South Pacific");

        let btn_rect2 = Rect::new(r2.x + 22.0, r2.y + r2.h - 52.0, r2.w - 44.0, 38.0);
        if button(&self.ui, btn_rect2, "BEGIN UNDERTOW [2027]", Btn::Primary, true) {
            self.new_campaign(1);
        }

        // Return button
        let back_w = 180.0;
        let back_r = Rect::new(w * 0.5 - back_w * 0.5, h - 48.0, back_w, 32.0);
        if button(&self.ui, back_r, "RETURN TO TITLE", Btn::Ghost, true) {
            self.screen = Screen::Title;
        }
    }

    fn draw_situation_map(&mut self) {
        let t = self.elapsed;
        self.globe.time = t;
        self.globe.draw(t);
        let w = screen_width(); let h = screen_height();
        let panel_w = (w * 0.34).clamp(250.0, 360.0);

        let (mx, my) = mouse_position();
        let mouse_on_globe = mx < w - panel_w && my > 44.0 && my < h - 44.0 && !self.placing_base;
        let hovered_nation_idx = if mouse_on_globe {
            if let Some((mlat, mlon)) = self.globe.screen_to_latlon(vec2(mx, my)) {
                let is_sea = self.data.is_water(mlat, mlon);
                self.find_nearest_nation(mlat, mlon, is_sea && self.water_unlocked)
            } else {
                None
            }
        } else {
            None
        };
        let hovered_nation_name = hovered_nation_idx.map(|idx| self.nations[idx].name.clone());

        // -------- country & sea-lane group borders
        for border in COUNTRY_BORDERS {
            let is_hovered = hovered_nation_name.as_deref() == Some(border.name);
            let nation_match = self.nations.iter().find(|n| n.name == border.name);
            let is_pact = nation_match.map(|n| n.pact_signed).unwrap_or(false);

            let (col, thickness) = if is_hovered {
                (AMBER, 2.0)
            } else if is_pact {
                (Color { r: 0.9, g: 0.2, b: 0.2, a: 0.75 }, 1.5)
            } else if border.game == 1 {
                (Color { r: 0.15, g: 0.55, b: 0.75, a: 0.40 }, 1.0)
            } else {
                (Color { r: 0.15, g: 0.65, b: 0.35, a: 0.45 }, 1.0)
            };

            for i in 0..border.points.len() {
                let next_i = (i + 1) % border.points.len();
                let (la1, lo1) = border.points[i];
                let (la2, lo2) = border.points[next_i];
                let steps = 3;
                let mut prev_proj: Option<Vec2> = None;
                for s in 0..=steps {
                    let f = s as f32 / steps as f32;
                    let la = la1 + (la2 - la1) * f;
                    let lo = lo1 + (lo2 - lo1) * f;
                    if let Some((p, z)) = self.globe.project(la, lo) {
                        if z > 0.05 {
                            if let Some(pp) = prev_proj {
                                draw_line(pp.x, pp.y, p.x, p.y, thickness, col);
                            }
                            prev_proj = Some(p);
                        } else {
                            prev_proj = None;
                        }
                    } else {
                        prev_proj = None;
                    }
                }
            }
        }

        // -------- markers
        let mut hover_text: Option<(Vec2, String, Color)> = None;
        for (i, b) in self.bases.iter().enumerate() {
            if let Some((p, _)) = self.globe.project(b.lat, b.lon) {
                let selected = i == self.selected_base;
                glow(p.x, p.y, if selected { 22.0 } else { 15.0 }, if b.water { CYAN } else { ui::GREEN }, 0.9);
                draw_poly(p.x, p.y, 6, if selected { 8.0 } else { 6.0 }, 0.0, if b.water { Color { a: 0.9, ..CYAN } } else { Color { a: 0.9, ..ui::GREEN } });
                draw_circle(p.x, p.y, 2.5, ui::WHITE);
                if p.distance(vec2(mouse_position().0, mouse_position().1)) < 18.0 {
                    hover_text = Some((p + vec2(12.0, -10.0), b.name.clone(), CYAN));
                }
                self.ui.text(p.x - self.ui.width(9.0, &b.name) * 0.5, p.y + 20.0, 9.0, Color { a: 0.8, ..TEXT }, &b.name);
            }
        }
        for (i, s) in self.sites.iter().enumerate() {
            if let Some((p, _)) = self.globe.project(s.lat, s.lon) {
                let col = match s.kind { SiteKind::Raid => ui::RED, SiteKind::Final => AMBER, SiteKind::Underwater => CYAN, _ => AMBER };
                let pulse = 1.0 + (t * 3.0 + i as f32).sin() * 0.15;
                glow(p.x, p.y, 18.0 * pulse, col, 1.0);
                draw_circle_lines(p.x, p.y, 8.0 * pulse, 1.5, col);
                draw_line(p.x - 11.0, p.y, p.x - 4.0, p.y, 1.5, col);
                draw_line(p.x + 4.0, p.y, p.x + 11.0, p.y, 1.5, col);
                draw_line(p.x, p.y - 11.0, p.x, p.y - 4.0, 1.5, col);
                draw_line(p.x, p.y + 4.0, p.x, p.y + 11.0, 1.5, col);
                if self.selected_site == Some(i) {
                    draw_circle_lines(p.x, p.y, 14.0, 1.0, AMBER);
                }
                if p.distance(vec2(mouse_position().0, mouse_position().1)) < 20.0 {
                    hover_text = Some((p + vec2(12.0, -10.0), s.name.clone(), col));
                }
            }
        }
        for (i, u) in self.ufos.iter().enumerate() {
            if let Some((p, _)) = self.globe.project(u.lat, u.lon) {
                let col = if u.underwater { CYAN } else { ui::RED };
                let pulse = 1.0 + (t * 4.0 + i as f32).sin() * 0.2;
                glow(p.x, p.y, 14.0 * pulse, col, 0.8);
                draw_poly(p.x, p.y, 3, 6.0, -std::f32::consts::FRAC_PI_2, col);
                draw_circle_lines(p.x, p.y, 12.0 * pulse, 1.0, Color { a: 0.6, ..col });
                if p.distance(vec2(mouse_position().0, mouse_position().1)) < 18.0 {
                    hover_text = Some((p + vec2(12.0, -10.0), if u.underwater { "SUBMERSIBLE CONTACT".into() } else { "UNIDENTIFIED CRAFT".into() }, col));
                }
            }
        }
        if self.placing_base {
            let (mx, my) = mouse_position();
            if let Some((lat, lon)) = self.globe.screen_to_latlon(vec2(mx, my)) {
                let water = self.data.is_water(lat, lon);
                let ok = !water || self.water_unlocked;
                let col = if ok { ui::GREEN } else { ui::RED };
                draw_circle_lines(mx, my, 26.0, 1.5, col);
                draw_line(mx - 34.0, my, mx + 34.0, my, 1.0, col);
                draw_line(mx, my - 34.0, mx, my + 34.0, 1.0, col);
                self.ui.text(mx + 34.0, my, 11.0, col, if water { "WATER BASE — $1.5M" } else { "LAND BASE — $1.0M" });
            }
        }

        // -------- top bar
        draw_rectangle(0.0, 0.0, w, 44.0, Color { r: 0.015, g: 0.035, b: 0.050, a: 0.96 });
        draw_rectangle(0.0, 44.0, w, 1.0, Color { a: 0.5, ..CYAN });
        self.ui.label(14.0, 28.0, 13.0, CYAN, "SITUATION MAP");
        self.ui.text(140.0, 27.0, 12.0, TEXT, &self.date.pretty());
        self.ui.text(276.0, 27.0, 12.0, DIM, &self.date.hhmm());
        self.ui.text(330.0, 27.0, 12.0, ui::GREEN, &format!("${}", fmt_money(self.funds)));
        self.ui.text(440.0, 27.0, 11.0, DIM, "SCORE");
        self.ui.text(482.0, 27.0, 12.0, if self.score >= 0 { ui::GREEN } else { ui::RED }, &format!("{}", self.score));
        // speed buttons
        let mut bx = w - panel_w - 152.0;
        if button(&self.ui, Rect::new(bx, 8.0, 28.0, 26.0), if self.paused { ">" } else { "||" }, Btn::Normal, true) {
            self.paused = !self.paused;
        }
        bx += 32.0;
        for i in 0..5 {
            let sel = self.speed == i && !self.paused;
            let r = Rect::new(bx, 8.0, 22.0, 26.0);
            let col = if sel { AMBER } else { DIM };
            draw_rectangle(r.x, r.y, r.w, r.h, if sel { Color { r: 0.16, g: 0.12, b: 0.05, a: 1.0 } } else { Color { a: 0.6, ..PANEL } });
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, col);
            self.ui.bcenter(r.x + r.w * 0.5, r.y + 18.0, 11.0, col, SPEED_LABEL[i]);
            if r.contains(vec2(mouse_position().0, mouse_position().1)) && is_mouse_button_released(MouseButton::Left) {
                self.speed = i; self.paused = false;
            }
            bx += 24.0;
        }

        // -------- campaign progress (left)
        let px = 12.0;
        let py = 56.0;
        let pr = Rect::new(px, py, 268.0, 64.0);
        panel(pr, Color { a: 0.94, ..PANEL }, Color { a: 0.65, ..CYAN });
        self.ui.label(pr.x + 10.0, pr.y + 16.0, 9.5, CYAN, "CAMPAIGN 1 // LANDFALL");
        bar(Rect::new(pr.x + 10.0, pr.y + 24.0, pr.w - 20.0, 5.0), self.missions_won as f32, 12.0, CYAN, Color { a: 0.25, ..BLACK });
        self.ui.text(pr.x + 10.0, pr.y + 47.0, 9.5, TEXT, &format!("{} / 12 OPS", self.missions_won.min(12)));
        let front = if self.water_unlocked { "UNDERTOW OPEN" } else { "UNDERTOW LOCKED" };
        let fcol = if self.water_unlocked { AMBER } else { DIM };
        self.ui.label(pr.right() - self.ui.bwidth(9.5, front) - 10.0, pr.y + 47.0, 9.5, fcol, front);

        // -------- event log (left, below progress). Newest first so the
        // bottom of the panel can never cut a line in half.
        let log_r = Rect::new(12.0, 126.0, 268.0, h - 196.0);
        panel(log_r, Color { a: 0.82, ..PANEL }, Color { a: 0.45, ..CYAN_DIM });
        self.ui.label(log_r.x + 10.0, log_r.y + 18.0, 10.0, CYAN, "OPERATIONS LOG");
        let max_lines = ((log_r.h - 44.0) / 13.0).max(1.0) as usize;
        let mut lines: Vec<(String, Color, f32)> = Vec::new();
        'outer: for l in self.log.iter().rev() {
            for line in wrap(&self.ui, &l.text, 10.0, log_r.w - 20.0) {
                lines.push((line, l.color, l.age));
                if lines.len() >= max_lines { break 'outer; }
            }
        }
        let mut yy = log_r.y + 38.0;
        for (line, col, age) in &lines {
            let a = (1.1 - age / 60.0).clamp(0.3, 1.0);
            self.ui.text(log_r.x + 10.0, yy, 10.0, Color { a, ..*col }, line);
            yy += 13.0;
        }

        // -------- right panel
        let rx = w - panel_w;
        draw_rectangle(rx, 44.0, panel_w, h - 44.0, Color { r: 0.012, g: 0.028, b: 0.040, a: 0.94 });
        draw_rectangle(rx, 44.0, 1.0, h - 44.0, Color { a: 0.5, ..CYAN });
        let mut y = 56.0;
        // selected contact
        if let Some(ui) = self.selected_ufo {
            if let Some(u) = self.ufos.get(ui).cloned() {
                self.ui.label(rx + 14.0, y, 11.0, ui::RED, "HOSTILE CONTACT");
                y += 20.0;
                self.ui.bold(rx + 14.0, y, 16.0, TEXT, if u.underwater { "SUBMERSIBLE" } else { "UNIDENTIFIED CRAFT" });
                y += 18.0;
                self.ui.text(rx + 14.0, y, 11.0, DIM, &format!("CLASS {}", match u.kind { UfoKind::Spore => "SPORE", UfoKind::Burr => "BURR", UfoKind::Thorn => "THORN", UfoKind::Rhizome => "RHIZOME", UfoKind::Taproot => "TAPROOT" }));
                y += 22.0;
                if button(&self.ui, Rect::new(rx + 14.0, y, panel_w - 28.0, 34.0), "SCRAMBLE INTERCEPTOR", Btn::Danger, true) {
                    self.scramble_interceptor(ui);
                }
                y += 42.0;
                if button(&self.ui, Rect::new(rx + 14.0, y, panel_w - 28.0, 28.0), "CANCEL TRACK", Btn::Ghost, true) {
                    self.selected_ufo = None;
                }
            }
        } else if let Some(si) = self.selected_site {
            if let Some(s) = self.sites.get(si).cloned() {
                let col = if s.raid { ui::RED } else { AMBER };
                self.ui.label(rx + 14.0, y, 11.0, col, if s.raid { "CITY RAID" } else { "MISSION TARGET" });
                y += 22.0;
                self.ui.bold(rx + 14.0, y, 16.0, TEXT, &s.name);
                y += 20.0;
                self.ui.text(rx + 14.0, y, 11.0, DIM, if s.underwater { "UNDERWATER // DIVE GEAR REQUIRED" } else { "SURFACE // STANDARD GEAR" });
                y += 16.0;
                self.ui.text(rx + 14.0, y, 11.0, DIM, &format!("HOSTILES: APPROX {}", s.aliens.len()));
                y += 16.0;
                self.ui.text(rx + 14.0, y, 11.0, DIM, &format!("RECOVERY VALUE: ${}", fmt_money(s.reward)));
                y += 22.0;
                let ready = self.ready_soldiers();
                self.ui.text(rx + 14.0, y, 11.0, if ready > 0 { ui::GREEN } else { ui::RED }, &format!("SQUAD READY: {}/8", ready));
                y += 16.0;
                if button(&self.ui, Rect::new(rx + 14.0, y, panel_w - 28.0, 38.0), "LAUNCH MISSION", Btn::Primary, ready > 0) {
                    self.launch_selected_site();
                }
                y += 46.0;
                if button(&self.ui, Rect::new(rx + 14.0, y, panel_w - 28.0, 26.0), "ENTER BASE", Btn::Normal, true) {
                    self.selected_base = 0;
                    self.base_return_zoom = self.globe.zoom;
                    let b = self.bases[0].clone();
                    self.globe.start_zoom(ZoomDir::In, b.lat, b.lon, 1.0);
                }
            }
        } else {
            // base overview
            let b = self.bases.get(self.selected_base).cloned();
            if let Some(b) = b {
                self.ui.label(rx + 14.0, y, 11.0, CYAN, "COMMAND BASE");
                y += 22.0;
                self.ui.bold(rx + 14.0, y, 17.0, TEXT, &b.name);
                y += 18.0;
                self.ui.text(rx + 14.0, y, 11.0, DIM, &format!("{:.1}°{}  {:.1}°{}", b.lat.abs(), if b.lat >= 0.0 { "N" } else { "S" }, b.lon.rem_euclid(360.0), "E"));
                y += 22.0;
                self.ui.text(rx + 14.0, y, 11.0, TEXT, &format!("PERSONNEL {}   READY {}", self.soldiers.len(), self.ready_soldiers()));
                y += 16.0;
                self.ui.text(rx + 14.0, y, 11.0, TEXT, &format!("CONTACTS {}   SITES {}", self.ufos.len(), self.sites.len()));
                y += 24.0;
                if button(&self.ui, Rect::new(rx + 14.0, y, panel_w - 28.0, 38.0), "ENTER BASE", Btn::Primary, true) {
                    self.base_return_zoom = self.globe.zoom;
                    self.globe.start_zoom(ZoomDir::In, b.lat, b.lon, 1.0);
                }
                y += 46.0;
                if button(&self.ui, Rect::new(rx + 14.0, y, panel_w - 28.0, 28.0), "BUILD NEW BASE", Btn::Normal, true) {
                    self.placing_base = true;
                    self.toast("CLICK THE GLOBE TO PLACE A BASE");
                }
            }
        }
        // contacts / sites list
        let mut ly = (h * 0.40).max(y + 20.0);
        self.ui.label(rx + 14.0, ly, 10.0, DIM, "ACTIVE SITES");
        ly += 14.0;
        let site_rows: Vec<(usize, String, bool, bool)> = self.sites.iter().enumerate().take(4).map(|(i, s)| (i, s.name.clone(), s.raid, s.underwater)).collect();
        for (i, name, raid, underwater) in site_rows {
            let r = Rect::new(rx + 10.0, ly, panel_w - 20.0, 30.0);
            let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
            draw_rectangle(r.x, r.y, r.w, r.h, if hover { Color { a: 0.18, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
            let col = if raid { ui::RED } else { AMBER };
            draw_rectangle(r.x, r.y, 2.0, r.h, col);
            self.ui.text(r.x + 8.0, r.y + 13.0, 10.0, TEXT, &name);
            self.ui.text(r.x + 8.0, r.y + 25.0, 9.0, DIM, if underwater { "UNDERWATER" } else { "SURFACE" });
            if hover && is_mouse_button_released(MouseButton::Left) { self.select_site(i); }
            ly += 34.0;
        }
        // hostile contacts
        ly += 12.0;
        self.ui.label(rx + 14.0, ly, 10.0, DIM, "HOSTILE CONTACTS");
        ly += 14.0;
        let contact_rows: Vec<(usize, bool, bool)> = self.ufos.iter().enumerate().take(3).map(|(i, u)| (i, u.underwater, u.raid)).collect();
        if contact_rows.is_empty() {
            self.ui.text(rx + 14.0, ly + 10.0, 9.0, Color { a: 0.5, ..DIM }, "NO ACTIVE CONTACTS");
            ly += 22.0;
        }
        for (i, underwater, raid) in contact_rows {
            let r = Rect::new(rx + 10.0, ly, panel_w - 20.0, 26.0);
            let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
            draw_rectangle(r.x, r.y, r.w, r.h, if self.selected_ufo == Some(i) { Color { r: 0.16, g: 0.05, b: 0.05, a: 1.0 } } else if hover { Color { a: 0.15, ..PANEL_2 } } else { Color { a: 0.05, ..PANEL } });
            draw_rectangle(r.x, r.y, 2.0, r.h, if raid { ui::RED } else if underwater { CYAN } else { ui::RED });
            self.ui.text(r.x + 8.0, r.y + 17.0, 9.5, TEXT, if underwater { "SUBMERSIBLE CONTACT" } else if raid { "RAID CRAFT" } else { "UNIDENTIFIED CRAFT" });
            if hover && is_mouse_button_released(MouseButton::Left) { self.selected_ufo = Some(i); self.selected_site = None; }
            ly += 29.0;
        }
        // bases
        ly += 10.0;
        self.ui.label(rx + 14.0, ly, 10.0, DIM, "BASES");
        ly += 14.0;
        let base_rows: Vec<(usize, String, bool, f32, f32)> = self.bases.iter().enumerate().map(|(i, b)| (i, b.name.clone(), b.water, b.lat, b.lon)).collect();
        for (i, name, water, lat, lon) in base_rows {
            let r = Rect::new(rx + 10.0, ly, panel_w - 20.0, 26.0);
            let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
            draw_rectangle(r.x, r.y, r.w, r.h, if hover { Color { a: 0.15, ..PANEL_2 } } else { Color { a: 0.05, ..PANEL } });
            self.ui.text(r.x + 8.0, r.y + 17.0, 10.0, if water { CYAN } else { TEXT }, &name);
            if hover && is_mouse_button_released(MouseButton::Left) {
                self.selected_base = i;
                self.base_return_zoom = self.globe.zoom;
                self.globe.start_zoom(ZoomDir::In, lat, lon, 1.0);
            }
            ly += 29.0;
        }

        // -------- bottom bar
        draw_rectangle(0.0, h - 44.0, w, 44.0, Color { r: 0.012, g: 0.028, b: 0.040, a: 0.94 });
        draw_rectangle(0.0, h - 44.0, w, 1.0, Color { a: 0.4, ..CYAN_DIM });
        self.ui.text(14.0, h - 18.0, 10.0, DIM, "DRAG ROTATE   WHEEL ZOOM   CLICK BASE TO ENTER   CLICK SITE/CONTACT TO ACT");
        let mp = mouse_position();
        if let Some((lat, lon)) = self.globe.screen_to_latlon(vec2(mp.0, mp.1)) {
            let lon = (lon + 540.0) % 360.0 - 180.0;
            self.ui.text(w * 0.44, h - 18.0, 9.0, Color { a: 0.6, ..DIM },
                &format!("{:.1}°{}  {:.1}°{}", lat.abs(), if lat >= 0.0 { "N" } else { "S" }, lon.abs(), if lon >= 0.0 { "E" } else { "W" }));
        }
        self.ui.text(w - panel_w + 14.0, h - 18.0, 10.0, DIM, if self.water_unlocked { "UNDERTOW ACTIVE" } else { "LANDFALL ACTIVE" });

        // -------- hovered territory dossier HUD
        if let Some(hn_idx) = hovered_nation_idx {
            let n = &self.nations[hn_idx];
            let box_w = 360.0;
            let box_h = 76.0;
            let box_x = 18.0;
            let box_y = h - 44.0 - box_h - 12.0;
            panel(Rect::new(box_x, box_y, box_w, box_h), Color { a: 0.92, ..PANEL_2 }, if n.pact_signed { ui::RED } else { CYAN });
            self.ui.bold(box_x + 12.0, box_y + 18.0, 13.0, if n.pact_signed { ui::RED } else { AMBER }, &format!("TERRITORY: {}", n.name));
            self.ui.text(box_x + 12.0, box_y + 34.0, 10.5, TEXT, &format!("FUNDING: ${}/MO  //  PANIC: {}%  //  SAT: {}%", fmt_money(n.funding), n.panic, n.satisfaction));
            let status_str = if n.pact_signed {
                "STATUS: COMPROMISED // INFILTRATED BY ALIEN FORCES"
            } else {
                &format!("ACTIVITY: +{} HEARTH / +{} ALIEN", n.defence_activity, n.alien_activity)
            };
            self.ui.text(box_x + 12.0, box_y + 50.0, 9.5, if n.pact_signed { ui::RED } else { ui::GREEN }, status_str);
            let zone_type = if n.game == 1 { "SEA-LANE GROUP WATERS" } else { "CHARTER STATE AIRSPACE" };
            self.ui.text(box_x + 12.0, box_y + 64.0, 8.5, DIM, zone_type);
        }

        // help overlay
        if self.show_help {
            let hr = Rect::new(w * 0.16, h * 0.20, w * 0.55, h * 0.58);
            panel(hr, PANEL, CYAN);
            self.ui.bold(hr.x + 20.0, hr.y + 30.0, 18.0, CYAN, "SITUATION MAP CONTROLS");
            let lines = [
                "Drag the globe to rotate it. Use the wheel to zoom in and out.",
                "Click a base marker to dive all the way into the base.",
                "Mouse wheel back out from a base to return to orbit.",
                "Click a crash site or city raid, then LAUNCH MISSION.",
                "Click a hostile contact, then SCRAMBLE INTERCEPTOR.",
                "Speeds 1-5 advance time. Pause stops the world.",
                "Each campaign is chosen from the campaign screen.",
            ];
            let mut yy = hr.y + 58.0;
            for l in lines { self.ui.text(hr.x + 20.0, yy, 12.0, TEXT, l); yy += 22.0; }
            if button(&self.ui, Rect::new(hr.x + hr.w - 130.0, hr.y + hr.h - 46.0, 110.0, 30.0), "CLOSE", Btn::Normal, true) { self.show_help = false; }
        }
        if let Some((p, text, col)) = hover_text {
            let tw = self.ui.width(10.0, &text) + 16.0;
            let rr = Rect::new((p.x).min(w - panel_w - tw - 6.0).max(6.0), p.y - 26.0, tw, 20.0);
            draw_rectangle(rr.x, rr.y, rr.w, rr.h, Color { a: 0.9, ..BG });
            draw_rectangle_lines(rr.x, rr.y, rr.w, rr.h, 1.0, Color { a: 0.6, ..col });
            self.ui.text(rr.x + 8.0, rr.y + 14.0, 10.0, col, &text);
        }
    }

    fn draw_debrief(&mut self) {
        let w = screen_width(); let h = screen_height();
        draw_rectangle(0.0, 0.0, w, h, Color { a: 0.58, ..BLACK });
        let r = Rect::new(w * 0.16, h * 0.13, w * 0.68, h * 0.72);
        panel(r, PANEL, CYAN);
        self.ui.label(r.x + 24.0, r.y + 28.0, 10.0, DIM, "AFTER-ACTION REPORT");
        self.ui.bold(r.x + 24.0, r.y + 52.0, 24.0, CYAN, "MISSION DEBRIEF");
        self.ui.bold(r.x + 24.0, r.y + 78.0, 13.0, TEXT, &self.ui.fit(13.0, &self.last_mission, r.w - 48.0, true));
        draw_rectangle(r.x + 24.0, r.y + 90.0, r.w - 48.0, 1.0, Color { a: 0.45, ..CYAN_DIM });
        let stats = [
            ("HOSTILES DOWN", format!("{}", self.last_kills), ui::GREEN),
            ("SOLDIERS LOST", format!("{}", self.last_lost), if self.last_lost > 0 { ui::RED } else { TEXT }),
            ("SHOTS FIRED", format!("{}", self.last_shots), TEXT),
            ("ACCURACY", if self.last_shots > 0 { format!("{}%", self.last_hits * 100 / self.last_shots) } else { "—".into() }, CYAN),
            ("RECOVERY", format!("${}", fmt_money(self.last_reward)), AMBER),
            ("SCORE", format!("{}", self.score), if self.score >= 0 { ui::GREEN } else { ui::RED }),
        ];
        for (i, (label, value, col)) in stats.iter().enumerate() {
            let col_i = i % 3;
            let row_i = i / 3;
            let sx = r.x + 28.0 + col_i as f32 * ((r.w - 56.0) / 3.0);
            let sy = r.y + 126.0 + row_i as f32 * 58.0;
            self.ui.label(sx, sy, 8.0, DIM, label);
            self.ui.bold(sx, sy + 22.0, 20.0, *col, value);
        }
        let last = self.log.iter().rev().take(3).map(|l| l.text.clone()).collect::<Vec<_>>();
        let mut y = r.y + 252.0;
        for l in last.iter().rev() {
            for line in wrap(&self.ui, l, 11.0, r.w - 56.0) { self.ui.text(r.x + 28.0, y, 11.0, TEXT, &line); y += 17.0; }
        }
        self.ui.text(r.x + 28.0, r.y + r.h - 62.0, 9.0, DIM, &format!("FUNDS ${}   OPERATIONS {}   DATE {}", fmt_money(self.funds), self.missions_won, self.date.pretty()));
        if button(&self.ui, Rect::new(r.x + r.w - 264.0, r.y + r.h - 54.0, 240.0, 36.0), "RETURN TO SITUATION MAP", Btn::Primary, true) {
            self.screen = Screen::SituationMap;
            self.paused = false;
            if !self.won && self.missions_won >= 12 && self.water_unlocked && self.sites.iter().any(|s| s.kind == SiteKind::Final) {
                self.selected_site = self.sites.iter().position(|s| s.kind == SiteKind::Final);
            }
        }
    }

    fn draw_event(&mut self) {
        let w = screen_width(); let h = screen_height();
        draw_rectangle(0.0, 0.0, w, h, Color { a: 0.65, ..BLACK });
        let r = Rect::new(w * 0.16, h * 0.12, w * 0.68, h * 0.74);
        panel(r, PANEL, AMBER);
        self.ui.label(r.x + 24.0, r.y + 30.0, 11.0, AMBER, "PRIORITY TRANSMISSION");
        self.ui.bold(r.x + 24.0, r.y + 62.0, 26.0, TEXT, &self.event_title);
        draw_rectangle(r.x + 24.0, r.y + 74.0, 260.0, 2.0, AMBER);
        let mut y = r.y + 104.0;
        for line in self.event_body.lines() {
            for wl in wrap(&self.ui, line, 12.0, r.w - 48.0) { self.ui.text(r.x + 24.0, y, 12.0, TEXT, &wl); y += 18.0; }
            y += 4.0;
        }
        if button(&self.ui, Rect::new(r.x + r.w - 174.0, r.y + r.h - 54.0, 152.0, 36.0), "ACKNOWLEDGE", Btn::Amber, true) {
            let next = self.event_next;
            self.screen = next;
            self.paused = false;
        }
    }

    fn draw_end(&mut self) {
        let w = screen_width(); let h = screen_height();
        draw_rectangle(0.0, 0.0, w, h, Color { a: 0.88, ..BLACK });
        let title_text = if self.won {
            if self.campaign_type == 1 { "THE DEEP IS SILENT" } else { "THE MOON IS SILENT" }
        } else {
            "EARTH HAS FALLEN"
        };
        let body_text = if self.won {
            if self.campaign_type == 1 {
                "The Root at Verity Deep is dead. The sea lanes are open again."
            } else {
                "The Seed on the far side is dead. Nothing more is coming from the Moon."
            }
        } else {
            "The Charter Assembly has dissolved. There is no one left to pay for the defence."
        };
        self.ui.bcenter(w * 0.5, h * 0.32, 40.0, if self.won { AMBER } else { ui::RED }, title_text);
        self.ui.center(w * 0.5, h * 0.42, 15.0, TEXT, body_text);
        self.ui.center(w * 0.5, h * 0.50, 12.0, DIM, &format!("FINAL SCORE {}   OPERATIONS {}", self.score, if self.campaign_type == 1 { self.water_missions } else { self.missions_won }));
        if button(&self.ui, Rect::new(w * 0.5 - 110.0, h * 0.62, 220.0, 42.0), "NEW CAMPAIGN", Btn::Primary, true) {
            self.screen = Screen::CampaignSelect;
        }
    }

    fn draw_assembly_report(&mut self) {
        let w = screen_width(); let h = screen_height();
        draw_rectangle(0.0, 0.0, w, h, Color { a: 0.94, ..BG });

        let Some(rep) = self.last_report.clone() else {
            self.screen = Screen::SituationMap;
            return;
        };

        let card_w = (w * 0.76).clamp(600.0, 1100.0);
        let card_h = (h * 0.84).clamp(500.0, 800.0);
        let cx = (w - card_w) * 0.5;
        let cy = (h - card_h) * 0.5;
        let r = Rect::new(cx, cy, card_w, card_h);
        panel(r, PANEL, CYAN);

        // Header
        self.ui.label(r.x + 24.0, r.y + 24.0, 11.0, CYAN, "JTF HEARTH // MONTHLY REVIEW // RESTRICTED");
        self.ui.bold(r.x + 24.0, r.y + 50.0, 22.0, TEXT, "CHARTER ASSEMBLY // FUNDING STATES");
        self.ui.text(r.x + r.w - 240.0, r.y + 50.0, 12.0, DIM, &format!("{} {}", rep.month.name(), rep.year));
        draw_rectangle(r.x + 24.0, r.y + 64.0, r.w - 48.0, 1.0, Color { a: 0.5, ..CYAN_DIM });

        // Rating Banner
        let (rating_col, rating_text) = match rep.rating {
            "EXCELLENT" => (ui::GREEN, "EXCELLENT: The Assembly is satisfied on every count. Several states are raising their contributions."),
            "GOOD" => (CYAN, "GOOD: Delegates are content with the month's work. Interceptions are keeping pace with the contacts."),
            "SATISFACTORY" => (TEXT, "SATISFACTORY: The Assembly notes activity that went unanswered. Hold the line and confidence will hold with it."),
            "POOR" => (AMBER, "POOR: Too many contacts went unchallenged and too many people were hurt. Contributions are being cut."),
            _ => (ui::RED, "TERRIBLE: The month was a failure. Panic is spreading faster than the task force can answer it. Fix this now."),
        };
        let b_r = Rect::new(r.x + 24.0, r.y + 74.0, r.w - 48.0, 48.0);
        draw_rectangle(b_r.x, b_r.y, b_r.w, b_r.h, Color { a: 0.15, ..rating_col });
        draw_rectangle_lines(b_r.x, b_r.y, b_r.w, b_r.h, 1.0, rating_col);
        self.ui.bold(b_r.x + 16.0, b_r.y + 20.0, 13.0, rating_col, &format!("OVERALL RATING: {}", rep.rating));
        self.ui.text(b_r.x + 16.0, b_r.y + 36.0, 10.0, TEXT, rating_text);

        // Two columns: Left = Financial Ledger, Right = Personnel & Promotions / Directives
        let col_w = (r.w - 64.0) * 0.5;

        // Left: Financial Statement
        let fx = r.x + 24.0;
        let mut fy = r.y + 140.0;
        self.ui.label(fx, fy, 11.0, CYAN, "MONTHLY FINANCIAL STATEMENT");
        fy += 16.0;
        draw_rectangle(fx, fy, col_w, 1.0, Color { a: 0.35, ..CYAN_DIM });
        fy += 12.0;

        let statement = [
            ("TOTAL ASSEMBLY FUNDING", format!("+${}", fmt_money(rep.total_funding)), ui::GREEN),
            ("BASE FACILITIES UPKEEP", format!("-${}", fmt_money(rep.facility_maint)), AMBER),
            ("CRAFT FLEET MAINTENANCE", format!("-${}", fmt_money(rep.craft_maint)), AMBER),
            ("SOLDIER SALARIES", format!("-${}", fmt_money(rep.soldier_salaries)), DIM),
            ("SCIENTIST SALARIES", format!("-${}", fmt_money(rep.scientist_salaries)), DIM),
            ("ENGINEER SALARIES", format!("-${}", fmt_money(rep.engineer_salaries)), DIM),
            ("TOTAL EXPENDITURES", format!("-${}", fmt_money(rep.total_expenses)), ui::RED),
            ("NET MONTHLY BALANCE", format!("{}${ }", if rep.net_balance >= 0 { "+" } else { "-" }, fmt_money(rep.net_balance.abs())), if rep.net_balance >= 0 { ui::GREEN } else { ui::RED }),
        ];

        for (item, val, col) in statement {
            self.ui.text(fx + 6.0, fy + 12.0, 10.0, TEXT, item);
            self.ui.bold(fx + col_w - self.ui.bwidth(11.0, &val) - 6.0, fy + 12.0, 11.0, col, &val);
            fy += 24.0;
        }

        // Right Column: Field Promotions & Directives
        let rx = r.x + 24.0 + col_w + 16.0;
        let mut ry = r.y + 140.0;
        self.ui.label(rx, ry, 11.0, CYAN, "PERSONNEL CITATIONS & FIELD PROMOTIONS");
        ry += 16.0;
        draw_rectangle(rx, ry, col_w, 1.0, Color { a: 0.35, ..CYAN_DIM });
        ry += 12.0;

        if rep.promotions.is_empty() {
            self.ui.text(rx + 6.0, ry + 12.0, 10.0, DIM, "NO OFFICER PROMOTIONS THIS CYCLE.");
            ry += 24.0;
        } else {
            for p in &rep.promotions {
                draw_circle(rx + 12.0, ry + 8.0, 3.5, ui::GREEN);
                self.ui.bold(rx + 24.0, ry + 12.0, 10.5, TEXT, p);
                ry += 22.0;
            }
        }

        ry += 16.0;
        self.ui.label(rx, ry, 11.0, AMBER, "INTELLIGENCE BRIEFING // GEOPOLITICAL ADVISORY");
        ry += 16.0;
        draw_rectangle(rx, ry, col_w, 1.0, Color { a: 0.35, ..CYAN_DIM });
        ry += 12.0;
        let geopolitical = if self.score < -200 {
            "Some Charter states are being offered separate terms, and the unhappy ones are listening. Protect their cities or expect them to sign."
        } else {
            "No Charter state is known to be in talks with the enemy. Keep intercepting Spore and Burr craft before they can land."
        };
        for line in wrap(&self.ui, geopolitical, 10.0, col_w - 12.0) {
            self.ui.text(rx + 6.0, ry + 12.0, 10.0, DIM, &line);
            ry += 16.0;
        }

        // Button to proceed
        let btn_w = 220.0;
        let btn_h = 36.0;
        if button(&self.ui, Rect::new(r.x + r.w - btn_w - 24.0, r.y + r.h - btn_h - 18.0, btn_w, btn_h), "BACK TO SITUATION MAP", Btn::Primary, true) || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Space) {
            self.screen = Screen::SituationMap;
            self.paused = false;
        }
    }
}

pub fn fmt_money(v: i64) -> String {
    let neg = v < 0;
    let s = v.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 { out.push(','); }
        out.push(c);
    }
    if neg { format!("-{}", out) } else { out }
}

#[macroquad::main(window_conf)]
async fn main() {
    macroquad::window::gl_set_drawcall_buffer_capacity(65_535, 400_000);
    let data = GameData::load();
    let ui = Ui::load();
    let audio = audio::SoundEffects::load().await;
    let mut game = Game::new(data, ui, audio);
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    let mut self_test_path: Option<String> = None;
    let mut demo_mode = false;
    // Debug flags are read first so they apply to whatever `--demo` starts.
    for k in 1..args.len() {
        let val = args.get(k + 1).cloned().unwrap_or_default();
        match args[k].as_str() {
            // local solar time at the site, e.g. --clock "2027-06-21 8am"
            "--clock" => { game.dbg_clock = sky::parse_clock(&val); if game.dbg_clock.is_none() { eprintln!("--clock: cannot read '{}'", val); } }
            // the last three are the heavy forms (bit 16)
            "--weather" => { game.dbg_weather = match val.as_str() { "clear" => Some(0), "rain" => Some(1), "snow" => Some(2), "dust" => Some(3), "storm" => Some(17), "blizzard" => Some(18), "duststorm" => Some(19), _ => None }; }
            // latitude,longitude in degrees, e.g. --site 64,-150; the biome follows the Earth map
            "--site" => { let mut it = val.split(',').map(|v| v.trim().parse::<f32>()); if let (Some(Ok(la)), Some(Ok(lo))) = (it.next(), it.next()) { game.dbg_site = Some((la.clamp(-89.0, 89.0), lo)); } else { eprintln!("--site: cannot read '{}'", val); } }
            "--biome" => { game.dbg_biome = val.parse().ok(); }
            "--seed" => { game.dbg_seed = val.parse().ok(); }
            "--view" => { game.dbg_view = match val.as_str() { "wide" => 1, "tele" => 2, "low" => 3, _ => 0 }; }
            "--t" => { game.dbg_t = val.parse().unwrap_or(0.0); }
            "--frametime" => { game.dbg_frametime = true; }
            _ => {}
        }
    }
    while i < args.len() {
        if args[i] == "--screenshot" && i + 1 < args.len() { game.screenshot = Some(args[i + 1].clone()); }
        else if args[i] == "--self-test" && i + 1 < args.len() { self_test_path = Some(args[i + 1].clone()); }
        else if args[i] == "--demo" && i + 1 < args.len() {
            demo_mode = true;
            game.demo = true;
            // A "sea-" prefix shows the same screen in the underwater campaign.
            let (demo_name, sea) = match args[i + 1].strip_prefix("sea-") {
                Some(rest) => (rest, true),
                None => (args[i + 1].as_str(), false),
            };
            game.new_campaign(if sea { 1 } else { 0 });
            match demo_name {
                "title" => { game.screen = Screen::Title; }
                "map" => { game.screen = Screen::SituationMap; }
                "campaign" => { game.screen = Screen::CampaignSelect; }
                "base" | "equip" => { game.screen = Screen::Base; game.base_tab = 0; }
                "facilities" => { game.screen = Screen::Base; game.base_tab = 1; }
                "research" => { game.screen = Screen::Base; game.base_tab = 2; }
                "workshop" => { game.screen = Screen::Base; game.base_tab = 3; }
                "market" => { game.screen = Screen::Base; game.base_tab = 4; }
                "fleet" => { game.screen = Screen::Base; game.base_tab = 5; }
                "finances" => { game.screen = Screen::Base; game.base_tab = 6; }
                "launch" => { game.screen = Screen::Base; game.base_tab = 7; }
                "assembly" => {
                    game.last_month = Month::Dec;
                    game.monthly_tick();
                }
                "editor" => { game.screen = Screen::Editor; game.selected_soldier = 0; }
                "dogfight" | "interception" => {
                    if game.ufos.is_empty() {
                        game.ufos.push(Ufo {
                            id: 99, lat: 28.0, lon: -40.0, target_lat: 30.0, target_lon: -35.0,
                            kind: UfoKind::Burr, underwater: sea, hp: 500, timer: 720, detected: true, raid: false, speed: 2800.0,
                        });
                    }
                    game.dogfight = Some(DogfightState::new(&game, 0, 0, 0));
                    game.screen = Screen::Dogfight;
                }
                "debrief" => {
                    game.last_kills = 4; game.last_lost = 1; game.last_shots = 17; game.last_hits = 9;
                    game.last_mission = "CRASH SITE // SPORE-CLASS".to_string(); game.last_reward = 180_000;
                    game.score = 225; game.log("MISSION COMPLETE. 4 HOSTILES NEUTRALISED. +$180", ui::GREEN);
                    game.log("SPECIALIST RIVERA WAS KILLED IN ACTION.", ui::RED);
                    game.log("RECOVERED 2 ALIEN WEAPON COMPONENTS.", CYAN);
                    game.screen = Screen::Debrief;
                }
                "tactical" => {
                    game.selected_site = Some(0);
                    game.launch_selected_site();
                    if let Some(b) = &mut game.battle {
                        b.cam_zoom = 0.62;
                        if let Some(a) = b.units.iter().find(|u| u.side == tactical::Side::Alien) {
                            b.hover = (a.x.round() as i32, a.y.round() as i32);
                        }
                    }
                }
                "water" => {
                    game.new_campaign(1);
                    for i in 0..game.soldiers.len() { game.equip_default(i, true); }
                    let spec = MissionSpec {
                        name: "SUNKEN LINER // DEEP CONTACT".into(), lat: 20.0, lon: -160.0,
                        underwater: true, raid: false, biome: 13, aliens: vec![2,2,3,1,0], tier: 4,
                        site: None, reward: 300_000, final_mission: false,
                        month: Month::May, minute_of_day: 860,
                    };
                    tactical::start(&mut game, spec);
                }
                _ => {}
            }
            game.paused = true;
        }
        else if args[i] == "--ai-test" {
            game.new_campaign(0);
            game.paused = false;
            game.selected_site = Some(0);
            game.launch_selected_site();
            for tick in 0..2400 {
                if tick % 200 == 0 { tactical::force_end_turn(&mut game); }
                game.update(0.05);
            }
            if let Some(b) = &game.battle {
                println!("AI TEST phase={:?} turn={} players={} aliens={} shots={} hits={} reactions={}",
                    b.phase, b.turn, b.alive_players(), b.alive_aliens(), b.shots_fired, b.hits, b.reaction_shots);
            } else {
                println!("AI TEST finished screen={:?} score={}", game.screen, game.score);
            }
            std::process::exit(0);
        }
        else if args[i] == "--dump-map" {
            let mut img = Image::gen_image_color(game.data.world_w as u16, game.data.world_h as u16, BLACK);
            for y in 0..game.data.world_h { for x in 0..game.data.world_w {
                let t = game.data.biome[y * game.data.world_w + x];
                let land = game.data.world[y * game.data.world_w + x] != 0;
                let c = if !land { Color::new(0.02, 0.05, 0.1, 1.0) } else { Color::new(0.1 + t as f32 / 14.0, 0.7, 0.3, 1.0) };
                img.set_pixel(x as u32, y as u32, c);
            }}
            img.export_png("shots/worldmap.png");
            std::process::exit(0);
        }
        i += 1;
    }
    if let Some(output_path) = self_test_path {
        game.new_campaign(0);
        game.paused = true;
        game.screen = Screen::Tactical;
        tactical::start_selftest(&mut game);
        game.self_test = Some(tactical::SelfTest::new(output_path));
    } else if game.screenshot.is_some() && !demo_mode {
        game.new_campaign(0);
        game.screen = Screen::SituationMap;
        game.paused = true;
        game.selected_site = Some(0);
        game.globe.zoom = 1.0;
    }
    game.globe.time = 0.0;
    let (mut ft_sum, mut ft_n, mut ft_show, mut elapsed) = (0.0f32, 0u32, 0.0f32, 0.0f32);
    loop {
        let raw_dt = get_frame_time();
        let dt = raw_dt.min(0.05);
        elapsed += raw_dt;
        game.update(dt);
        game.draw();
        if game.frame > 30 { ft_sum += raw_dt; ft_n += 1; }
        if game.dbg_frametime {
            if ft_n % 30 == 1 { ft_show = ft_sum / ft_n.max(1) as f32; }
            let txt = format!("{:.2} ms  ({:.0} fps)", ft_show * 1000.0, 1.0 / ft_show.max(1e-5));
            draw_rectangle(8.0, 126.0, 220.0, 28.0, Color::new(0.0, 0.0, 0.0, 0.6));
            draw_text(&txt, 16.0, 146.0, 24.0, WHITE);
        }
        if let Some(path) = game.screenshot.clone() {
            if game.frame >= 15 && elapsed >= game.dbg_t {
                if game.dbg_frametime && ft_n > 0 { println!("FRAMETIME avg {:.2} ms over {} frames", ft_sum / ft_n as f32 * 1000.0, ft_n); }
                let img = get_screen_data();
                if let Some(parent) = std::path::Path::new(&path).parent() { let _ = std::fs::create_dir_all(parent); }
                img.export_png(&path);
                std::process::exit(0);
            }
        }
        next_frame().await;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Defending Earth".to_owned(),
        window_width: 1920,
        window_height: 1080,
        high_dpi: false,
        sample_count: 4,
        window_resizable: false,
        platform: macroquad::miniquad::conf::Platform {
            swap_interval: None,
            linux_wm_class: "defending-earth",
            ..Default::default()
        },
        ..Default::default()
    }
}
