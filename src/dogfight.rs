#![allow(dead_code)]
use macroquad::prelude::*;
use crate::data::*;
use crate::ui::{self, PANEL, PANEL_2, CYAN, CYAN_DIM, AMBER, TEXT, DIM, Btn, button, panel};
use crate::{Game, Screen, Site, SiteKind, UfoKind};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DogfightStance {
    Shadow,
    LongShot,
    Press,
    CloseIn,
    BreakOff,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DogfightPhase {
    Approaching,
    Combat,
    UfoDown,
    UfoVaporized,
    CraftDown,
    Disengaged,
}

#[derive(Clone, Debug)]
pub struct ActiveCraftWeapon {
    pub wtype: CraftWeaponType,
    pub name: &'static str,
    pub range_km: f32,
    pub damage: i32,
    pub accuracy: i32,
    pub ammo: i32,
    pub max_ammo: i32,
    pub cooldown: f32,
    pub cooldown_max: f32,
}

#[derive(Clone, Debug)]
pub struct Projectile {
    pub from_craft: bool,
    pub progress: f32,
    pub speed: f32,
    pub color: Color,
    pub is_missile: bool,
    pub damage: i32,
    pub accuracy: i32,
    pub trail: Vec<Vec2>,
}

#[derive(Clone, Debug)]
pub struct Spark {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub size: f32,
}

#[derive(Clone, Debug)]
pub struct CombatLogEntry {
    pub text: String,
    pub color: Color,
    pub time: f32,
}

#[derive(Clone)]
pub struct DogfightState {
    pub ufo_idx: usize,
    pub ufo_kind: UfoKind,
    pub ufo_name: String,
    pub underwater: bool,
    pub ufo_hp: i32,
    pub ufo_max_hp: i32,
    pub ufo_weapon_name: &'static str,
    pub ufo_weapon_range: f32,
    pub ufo_weapon_damage: i32,
    pub ufo_weapon_accuracy: i32,
    pub ufo_weapon_cooldown: f32,
    pub ufo_weapon_cooldown_max: f32,
    pub ufo_speed: f32,

    pub base_idx: usize,
    pub craft_idx: usize,
    pub craft_name: String,
    pub craft_type: CraftType,
    pub craft_hp: i32,
    pub craft_max_hp: i32,
    pub craft_speed: f32,
    pub weapons: [Option<ActiveCraftWeapon>; 2],

    pub distance_km: f32,
    pub stance: DogfightStance,
    pub phase: DogfightPhase,
    pub phase_timer: f32,
    pub radar_angle: f32,
    pub screen_shake: f32,

    pub projectiles: Vec<Projectile>,
    pub sparks: Vec<Spark>,
    pub logs: Vec<CombatLogEntry>,
    pub elapsed: f32,
}

impl DogfightState {
    pub fn new(game: &Game, ufo_idx: usize, base_idx: usize, craft_idx: usize) -> Self {
        let u = &game.ufos[ufo_idx];
        let base = &game.bases[base_idx];
        let craft = &base.craft[craft_idx];
        let craft_d = craft_def(craft.ctype);

        let (ufo_name, ufo_max_hp, u_range, u_dmg, u_acc, u_wname, u_cool, u_spd) = match u.kind {
            UfoKind::Spore => (
                if u.underwater { "SPORE-CLASS USO" } else { "SPORE-CLASS UFO" },
                110, 14.0, 22, 50, "ARC NEEDLE", 2.4, 3000.0,
            ),
            UfoKind::Burr => (
                if u.underwater { "BURR-CLASS USO" } else { "BURR-CLASS UFO" },
                460, 22.0, 46, 60, "ARC LANCE", 2.1, 2600.0,
            ),
            UfoKind::Thorn => (
                if u.underwater { "THORN-CLASS USO" } else { "THORN-CLASS UFO" },
                1100, 32.0, 80, 68, "ARC VOLLEY", 1.9, 3600.0,
            ),
            UfoKind::Rhizome => (
                "RHIZOME-CLASS USO",
                600, 25.0, 55, 62, "PRESSURE LANCE", 2.1, 3200.0,
            ),
            UfoKind::Taproot => (
                if u.underwater { "TAPROOT-CLASS USO" } else { "TAPROOT-CLASS UFO" },
                2600, 42.0, 130, 76, "MAIN DISCHARGE", 1.6, 4400.0,
            ),
        };

        let mut weapons: [Option<ActiveCraftWeapon>; 2] = [None, None];
        for i in 0..2 {
            if let Some(wtype) = craft.weapons[i] {
                let wdef = craft_weapon_def(wtype);
                let cool_max = match wtype {
                    CraftWeaponType::GunPod | CraftWeaponType::BoltCannon => 0.35,
                    CraftWeaponType::CoredLance | CraftWeaponType::CavCannon => 1.8,
                    CraftWeaponType::ArcProjector | CraftWeaponType::PressureProjector => 2.0,
                    CraftWeaponType::ShortRangeMissile | CraftWeaponType::LightTorpedo => 2.6,
                    CraftWeaponType::LongRangeMissile | CraftWeaponType::HeavyTorpedo => 3.4,
                    CraftWeaponType::BreacherMissile | CraftWeaponType::BreacherTorpedo => 4.0,
                };
                weapons[i] = Some(ActiveCraftWeapon {
                    wtype,
                    name: wdef.name,
                    range_km: wdef.range_km as f32,
                    damage: wdef.damage,
                    accuracy: wdef.accuracy,
                    ammo: craft.ammo[i],
                    max_ammo: wdef.max_ammo,
                    cooldown: 0.5,
                    cooldown_max: cool_max,
                });
            }
        }

        let mut st = Self {
            ufo_idx,
            ufo_kind: u.kind,
            ufo_name: ufo_name.to_string(),
            underwater: u.underwater,
            ufo_hp: ufo_max_hp,
            ufo_max_hp,
            ufo_weapon_name: u_wname,
            ufo_weapon_range: u_range,
            ufo_weapon_damage: u_dmg,
            ufo_weapon_accuracy: u_acc,
            ufo_weapon_cooldown: 1.5,
            ufo_weapon_cooldown_max: u_cool,
            ufo_speed: u_spd,

            base_idx,
            craft_idx,
            craft_name: craft.name.clone(),
            craft_type: craft.ctype,
            craft_hp: craft.hp,
            craft_max_hp: craft_d.max_hp,
            craft_speed: craft_d.speed as f32,
            weapons,

            distance_km: 62.0,
            stance: DogfightStance::LongShot,
            phase: DogfightPhase::Approaching,
            phase_timer: 0.0,
            radar_angle: 0.0,
            screen_shake: 0.0,

            projectiles: Vec::new(),
            sparks: Vec::new(),
            logs: Vec::new(),
            elapsed: 0.0,
        };

        st.add_log(format!("TACTICAL INTERCEPT: {} VS {}", craft.name, ufo_name), CYAN);
        st.add_log("RADAR LOCK CONFIRMED. RANGE 62.0 KM.", TEXT);
        st.add_log("ENGAGEMENT STANCE: LONG SHOT", AMBER);
        st
    }

    pub fn add_log(&mut self, text: impl Into<String>, color: Color) {
        self.logs.push(CombatLogEntry {
            text: text.into(),
            color,
            time: self.elapsed,
        });
        if self.logs.len() > 14 {
            self.logs.remove(0);
        }
    }

    pub fn max_effective_range(&self) -> f32 {
        let mut max_r = 15.0f32;
        for w in self.weapons.iter().flatten() {
            if (w.max_ammo == 0 || w.ammo > 0) && w.range_km > max_r {
                max_r = w.range_km;
            }
        }
        max_r
    }

    pub fn set_stance(&mut self, stance: DogfightStance) {
        if self.phase != DogfightPhase::Approaching && self.phase != DogfightPhase::Combat {
            return;
        }
        self.stance = stance;
        let name = match stance {
            DogfightStance::Shadow => "SHADOW (HOLD RANGE)",
            DogfightStance::LongShot => "LONG SHOT (MAX RANGE)",
            DogfightStance::Press => "PRESS (MEDIUM RANGE)",
            DogfightStance::CloseIn => "CLOSE IN (POINT BLANK)",
            DogfightStance::BreakOff => "BREAKING OFF — RETURN TO BASE",
        };
        self.add_log(format!("COMMAND: {}", name), AMBER);
    }

    pub fn tick(&mut self, game: &mut Game, dt: f32) {
        self.elapsed += dt;
        self.radar_angle = (self.radar_angle + dt * 2.8) % (std::f32::consts::PI * 2.0);
        if self.screen_shake > 0.0 {
            self.screen_shake = (self.screen_shake - dt * 2.5).max(0.0);
        }

        // Input keyboard controls
        if is_key_pressed(KeyCode::Key1) { self.set_stance(DogfightStance::Shadow); }
        if is_key_pressed(KeyCode::Key2) { self.set_stance(DogfightStance::LongShot); }
        if is_key_pressed(KeyCode::Key3) { self.set_stance(DogfightStance::Press); }
        if is_key_pressed(KeyCode::Key4) { self.set_stance(DogfightStance::CloseIn); }
        if is_key_pressed(KeyCode::Escape) { self.set_stance(DogfightStance::BreakOff); }

        match self.phase {
            DogfightPhase::Approaching | DogfightPhase::Combat => {
                self.update_combat(game, dt);
            }
            DogfightPhase::UfoDown | DogfightPhase::UfoVaporized | DogfightPhase::CraftDown | DogfightPhase::Disengaged => {
                self.phase_timer += dt;
                // Allow space / enter or wait 3 seconds to return
                if self.phase_timer >= 3.2 || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    self.finish_combat(game);
                }
            }
        }

        // Update sparks
        for s in &mut self.sparks {
            s.pos += s.vel * dt;
            s.life -= dt;
        }
        self.sparks.retain(|s| s.life > 0.0);
    }

    fn update_combat(&mut self, game: &mut Game, dt: f32) {
        // Target distance based on stance
        let target_dist = match self.stance {
            DogfightStance::Shadow => 65.0,
            DogfightStance::LongShot => self.max_effective_range().min(60.0),
            DogfightStance::Press => 22.0,
            DogfightStance::CloseIn => 7.5,
            DogfightStance::BreakOff => 85.0,
        };

        // Distance closure velocity
        let speed_ratio = (self.craft_speed / 2000.0).clamp(0.6, 2.5);
        let dist_delta = (target_dist - self.distance_km) * 1.2 * speed_ratio * dt;
        self.distance_km += dist_delta;

        if self.stance == DogfightStance::BreakOff && self.distance_km >= 75.0 {
            self.phase = DogfightPhase::Disengaged;
            self.phase_timer = 0.0;
            self.add_log("CONTACT BROKEN. RETURNING TO BASE.", CYAN);
            return;
        }

        if self.distance_km <= self.max_effective_range() + 2.0 {
            self.phase = DogfightPhase::Combat;
        }

        // Weapon cooldowns and firing
        for i in 0..2 {
            let mut fired_proj = None;
            let mut log_entry = None;
            if let Some(w) = &mut self.weapons[i] {
                if w.cooldown > 0.0 {
                    w.cooldown -= dt;
                }
                let has_ammo = w.max_ammo == 0 || w.ammo > 0;
                if has_ammo && w.cooldown <= 0.0 && self.distance_km <= w.range_km && self.stance != DogfightStance::Shadow && self.stance != DogfightStance::BreakOff {
                    // Fire weapon!
                    if w.max_ammo > 0 {
                        w.ammo -= 1;
                    }
                    w.cooldown = w.cooldown_max;
                    let is_missile = matches!(w.wtype, CraftWeaponType::ShortRangeMissile | CraftWeaponType::LongRangeMissile | CraftWeaponType::BreacherMissile | CraftWeaponType::LightTorpedo | CraftWeaponType::HeavyTorpedo | CraftWeaponType::BreacherTorpedo);
                    let color = match w.wtype {
                        CraftWeaponType::CoredLance => CYAN,
                        CraftWeaponType::ArcProjector | CraftWeaponType::PressureProjector => Color { r: 0.3, g: 0.9, b: 0.5, a: 1.0 },
                        CraftWeaponType::BreacherMissile | CraftWeaponType::BreacherTorpedo => Color { r: 0.9, g: 0.3, b: 0.9, a: 1.0 },
                        _ => AMBER,
                    };
                    match w.wtype {
                        CraftWeaponType::CoredLance => game.audio.play_cored(),
                        CraftWeaponType::ArcProjector | CraftWeaponType::PressureProjector => game.audio.play_arc(),
                        _ => game.audio.play_rifle(),
                    }
                    fired_proj = Some(Projectile {
                        from_craft: true,
                        progress: 0.0,
                        speed: if is_missile { 0.75 } else { 1.6 },
                        color,
                        is_missile,
                        damage: w.damage,
                        accuracy: w.accuracy,
                        trail: Vec::new(),
                    });
                    log_entry = Some((format!("{}: FIRED", w.name), color));
                }
            }
            if let Some(p) = fired_proj {
                self.projectiles.push(p);
            }
            if let Some((msg, col)) = log_entry {
                self.add_log(msg, col);
            }
        }

        // UFO Weapon firing
        if self.ufo_weapon_cooldown > 0.0 {
            self.ufo_weapon_cooldown -= dt;
        }
        if self.ufo_weapon_cooldown <= 0.0 && self.distance_km <= self.ufo_weapon_range && self.ufo_hp > 0 {
            self.ufo_weapon_cooldown = self.ufo_weapon_cooldown_max;
            game.audio.play_arc();
            self.projectiles.push(Projectile {
                from_craft: false,
                progress: 0.0,
                speed: 1.3,
                color: Color { r: 1.0, g: 0.2, b: 0.3, a: 1.0 },
                is_missile: false,
                damage: self.ufo_weapon_damage,
                accuracy: self.ufo_weapon_accuracy,
                trail: Vec::new(),
            });
            self.add_log(format!("{}: CONTACT IS SHOOTING BACK.", self.ufo_weapon_name), ui::RED);
        }

        // Update projectiles
        let mut resolved_hits = Vec::new();
        for p in &mut self.projectiles {
            p.progress += p.speed * dt;
            if p.progress >= 1.0 {
                resolved_hits.push((p.from_craft, p.damage, p.accuracy, p.color));
            }
        }
        self.projectiles.retain(|p| p.progress < 1.0);

        // Resolve hits
        for (from_craft, dmg, acc, col) in resolved_hits {
            if from_craft {
                let stance_idx = match self.stance {
                    DogfightStance::LongShot => 1,
                    DogfightStance::Press => 2,
                    DogfightStance::CloseIn => 3,
                    _ => 2,
                };
                let acc_half = acc / 2;
                let stance_div = (5 - stance_idx).max(1);
                let effective_acc = (acc_half + (acc_half / stance_div) * 3).clamp(5, 100);

                let hit = game.rng.rand(100) < effective_acc;
                if hit {
                    game.audio.play_explosion();
                    let dmg_roll = game.rng.rand(dmg);
                    let actual_dmg = ((dmg_roll + dmg) / 2).max(1);
                    self.ufo_hp = (self.ufo_hp - actual_dmg).max(0);
                    self.spawn_sparks(vec2(760.0, 360.0), col, 22);
                    self.screen_shake = 0.35;
                    self.add_log(format!("HIT ON CONTACT (-{} HP, {} LEFT)", actual_dmg, self.ufo_hp), col);

                    if self.ufo_hp <= 0 {
                        if actual_dmg >= 140 {
                            self.phase = DogfightPhase::UfoVaporized;
                            self.phase_timer = 0.0;
                            self.add_log("CONTACT BROKE UP. NOTHING TO RECOVER.", ui::GREEN);
                        } else {
                            self.phase = DogfightPhase::UfoDown;
                            self.phase_timer = 0.0;
                            self.add_log("CONTACT IS GOING DOWN. MARKING THE SITE.", ui::GREEN);
                        }
                        self.spawn_sparks(vec2(760.0, 360.0), ui::GREEN, 50);
                        break;
                    }
                } else {
                    self.add_log("SHOT MISSED TARGET.", DIM);
                }
            } else {
                let hit = game.rng.rand(100) > 40;
                if hit {
                    game.audio.play_explosion();
                    let dmg_roll = game.rng.rand(self.ufo_weapon_damage).max(10);
                    self.craft_hp = (self.craft_hp - dmg_roll).max(0);
                    self.spawn_sparks(vec2(280.0, 360.0), ui::RED, 25);
                    self.screen_shake = 0.6;
                    self.add_log(format!("WE ARE HIT (-{} HP, HULL {})", dmg_roll, self.craft_hp), ui::RED);

                    if self.craft_hp <= 0 {
                        self.phase = DogfightPhase::CraftDown;
                        self.phase_timer = 0.0;
                        self.add_log("CRAFT LOST.", ui::RED);
                        self.spawn_sparks(vec2(280.0, 360.0), ui::RED, 60);
                        break;
                    }
                } else {
                    self.add_log("CONTACT FIRED WIDE.", DIM);
                }
            }
        }
    }

    fn spawn_sparks(&mut self, center: Vec2, color: Color, count: usize) {
        for _ in 0..count {
            let ang = rand::gen_range(0.0, std::f32::consts::PI * 2.0);
            let spd = rand::gen_range(40.0, 180.0);
            let vel = vec2(ang.cos() * spd, ang.sin() * spd);
            self.sparks.push(Spark {
                pos: center,
                vel,
                life: rand::gen_range(0.3, 0.8),
                max_life: 0.8,
                color,
                size: rand::gen_range(2.0, 4.5),
            });
        }
    }

    pub fn finish_combat(&self, game: &mut Game) {
        // Sync craft state back to base
        if self.base_idx < game.bases.len() && self.craft_idx < game.bases[self.base_idx].craft.len() {
            let craft = &mut game.bases[self.base_idx].craft[self.craft_idx];
            craft.hp = self.craft_hp;
            for i in 0..2 {
                if let Some(w) = &self.weapons[i] {
                    craft.ammo[i] = w.ammo;
                }
            }
            if self.phase == DogfightPhase::CraftDown {
                game.bases[self.base_idx].craft.remove(self.craft_idx);
                game.log(format!("{} WAS DESTROYED IN COMBAT.", self.craft_name), ui::RED);
            }
        }

        // UFO outcome
        match self.phase {
            DogfightPhase::UfoDown => {
                if self.ufo_idx < game.ufos.len() {
                    let u = game.ufos[self.ufo_idx].clone();
                    let (lat, lon) = if u.underwater {
                        game.find_water(u.lat, u.lon)
                    } else {
                        game.find_land(u.lat, u.lon)
                    };
                    let tier = game.missions_won / 3 + 1;
                    let n = game.rng.range(4, 6) as usize;
                    let mut aliens = Vec::new();
                    for _ in 0..n {
                        aliens.push(game.roll_alien(u.underwater, tier));
                    }
                    let site = Site {
                        id: game.next_id(),
                        name: if u.underwater { "SUNKEN CRAFT".to_string() } else { "CRASH SITE".to_string() },
                        lat, lon, kind: SiteKind::Crash, underwater: u.underwater, raid: false,
                        expires: 2400, aliens, tier, reward: 160_000 + tier as i64 * 20_000,
                    };
                    game.sites.push(site);
                    game.log(format!("{} SHOT DOWN. CRASH SITE CONFIRMED.", if u.underwater { "SUBMERSIBLE" } else { "UFO" }), ui::GREEN);
                    if let Some(n_idx) = game.find_nearest_nation(u.lat, u.lon, u.underwater) {
                        game.nations[n_idx].defence_activity += 40;
                        game.nations[n_idx].panic = (game.nations[n_idx].panic - 8).max(0);
                        game.nations[n_idx].satisfaction = (game.nations[n_idx].satisfaction + 4).min(100);
                        game.log(format!("{} DEFENDED: AIRSPACE SECURED (+40 ACTIVITY).", game.nations[n_idx].name), ui::GREEN);
                    }
                    game.ufos.remove(self.ufo_idx);
                }
                game.score += 25;
            }
            DogfightPhase::UfoVaporized => {
                if self.ufo_idx < game.ufos.len() {
                    let u = game.ufos.remove(self.ufo_idx);
                    game.log(format!("{} DESTROYED BY DIRECT CONCUSSION HIT.", if u.underwater { "SUBMERSIBLE" } else { "UFO" }), ui::GREEN);
                    if let Some(n_idx) = game.find_nearest_nation(u.lat, u.lon, u.underwater) {
                        game.nations[n_idx].defence_activity += 60;
                        game.nations[n_idx].panic = (game.nations[n_idx].panic - 12).max(0);
                        game.nations[n_idx].satisfaction = (game.nations[n_idx].satisfaction + 6).min(100);
                    }
                }
                game.score += 50;
            }
            DogfightPhase::Disengaged => {
                game.log(format!("{} BROKE OFF INTERCEPTION. RETURNING TO RE-ARM.", self.craft_name), AMBER);
            }
            _ => {}
        }

        game.selected_ufo = None;
        game.dogfight = None;
        game.screen = Screen::SituationMap;
        game.toast("INTERCEPTION MISSION CONCLUDED");
    }

    // ================================================================ RENDER
    pub fn render(&mut self, game: &mut Game) {
        let sw = screen_width();
        let sh = screen_height();

        // Screen shake offset
        let shake_ox = if self.screen_shake > 0.0 {
            (self.elapsed * 60.0).sin() * self.screen_shake * 12.0
        } else {
            0.0
        };
        let shake_oy = if self.screen_shake > 0.0 {
            (self.elapsed * 45.0).cos() * self.screen_shake * 10.0
        } else {
            0.0
        };

        // Dark cockpit / CRT background
        draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.02, g: 0.04, b: 0.06, a: 1.0 });

        // Header bar
        draw_rectangle(0.0, 0.0, sw, 44.0, Color { r: 0.03, g: 0.08, b: 0.12, a: 0.95 });
        draw_rectangle(0.0, 44.0, sw, 1.0, Color { a: 0.6, ..CYAN });
        game.ui.label(18.0 + shake_ox, 28.0 + shake_oy, 12.0, CYAN, if self.underwater { "SEA DEFENSE TACTICAL COMMAND" } else { "AIR DEFENSE TACTICAL COMMAND" });
        game.ui.bold(260.0 + shake_ox, 30.0 + shake_oy, 16.0, TEXT, "INTERCEPTION RADAR // WEAPON TELEMETRY");

        let status_str = match self.phase {
            DogfightPhase::Approaching => "APPROACHING TARGET",
            DogfightPhase::Combat => "ENGAGEMENT IN PROGRESS",
            DogfightPhase::UfoDown => "CONTACT DOWN — SITE MARKED",
            DogfightPhase::UfoVaporized => "CONTACT DESTROYED",
            DogfightPhase::CraftDown => "CRAFT LOST",
            DogfightPhase::Disengaged => "CONTACT BROKEN",
        };
        let status_col = match self.phase {
            DogfightPhase::Approaching => AMBER,
            DogfightPhase::Combat => CYAN,
            DogfightPhase::UfoDown | DogfightPhase::UfoVaporized => ui::GREEN,
            DogfightPhase::CraftDown => ui::RED,
            DogfightPhase::Disengaged => DIM,
        };
        game.ui.bold(sw - 320.0 + shake_ox, 28.0 + shake_oy, 14.0, status_col, status_str);

        // Main radar scope area (center)
        let radar_cx = sw * 0.45 + shake_ox;
        let radar_cy = sh * 0.48 + shake_oy;
        let radar_r = (sh * 0.38).min(sw * 0.35);

        // Circular CRT Radar Display
        draw_circle(radar_cx, radar_cy, radar_r, Color { r: 0.015, g: 0.05, b: 0.04, a: 0.92 });
        draw_circle_lines(radar_cx, radar_cy, radar_r, 2.0, Color { r: 0.1, g: 0.45, b: 0.35, a: 0.8 });

        // Concentric range rings: 10, 20, 30, 40, 50, 60 km
        for k in 1..=6 {
            let r_ring = radar_r * (k as f32 / 6.0);
            draw_circle_lines(radar_cx, radar_cy, r_ring, 1.0, Color { r: 0.05, g: 0.25, b: 0.18, a: 0.45 });
            game.ui.text(radar_cx + 4.0, radar_cy - r_ring + 12.0, 8.0, Color { r: 0.1, g: 0.45, b: 0.3, a: 0.7 }, &format!("{} KM", k * 10));
        }

        // Crosshairs & angular ticks
        draw_line(radar_cx - radar_r, radar_cy, radar_cx + radar_r, radar_cy, 1.0, Color { r: 0.05, g: 0.25, b: 0.18, a: 0.35 });
        draw_line(radar_cx, radar_cy - radar_r, radar_cx, radar_cy + radar_r, 1.0, Color { r: 0.05, g: 0.25, b: 0.18, a: 0.35 });

        // Sweeping radar beam
        let sweep_x = radar_cx + self.radar_angle.cos() * radar_r;
        let sweep_y = radar_cy + self.radar_angle.sin() * radar_r;
        draw_line(radar_cx, radar_cy, sweep_x, sweep_y, 1.5, Color { r: 0.2, g: 0.8, b: 0.5, a: 0.55 });

        // Distance & Blips positions
        // Interceptor stays at center-left, UFO at distance along horizontal line
        let dist_fraction = (self.distance_km / 70.0).clamp(0.08, 0.96);
        let interceptor_pos = vec2(radar_cx - radar_r * 0.72, radar_cy);
        let ufo_pos = vec2(radar_cx - radar_r * 0.72 + radar_r * 1.44 * dist_fraction, radar_cy);

        // Range line between them
        draw_line(interceptor_pos.x, interceptor_pos.y, ufo_pos.x, ufo_pos.y, 1.5, Color { r: 0.1, g: 0.4, b: 0.5, a: 0.35 });

        // Draw Interceptor blip
        draw_circle(interceptor_pos.x, interceptor_pos.y, 6.0, CYAN);
        draw_circle_lines(interceptor_pos.x, interceptor_pos.y, 10.0 + (self.elapsed * 4.0).sin().abs() * 3.0, 1.0, Color { a: 0.6, ..CYAN });
        game.ui.bold(interceptor_pos.x - 30.0, interceptor_pos.y + 22.0, 10.0, CYAN, &self.craft_name);

        // Draw UFO blip
        if self.phase != DogfightPhase::UfoVaporized {
            let ufo_col = if self.ufo_hp <= 0 { ui::GREEN } else { ui::RED };
            draw_circle(ufo_pos.x, ufo_pos.y, 7.0, ufo_col);
            draw_circle_lines(ufo_pos.x, ufo_pos.y, 12.0 + (self.elapsed * 5.0).cos().abs() * 5.0, 1.5, Color { a: 0.8, ..ufo_col });
            game.ui.bold(ufo_pos.x - 20.0, ufo_pos.y - 14.0, 10.0, ufo_col, &self.ufo_name);
        }

        // Draw weapon range threshold markers on the radar
        for w in self.weapons.iter().flatten() {
            let w_frac = (w.range_km / 70.0).clamp(0.08, 0.96);
            let w_marker_x = interceptor_pos.x + radar_r * 1.44 * w_frac;
            draw_line(w_marker_x, radar_cy - 14.0, w_marker_x, radar_cy + 14.0, 1.0, Color { a: 0.5, ..AMBER });
            game.ui.text(w_marker_x - 12.0, radar_cy - 18.0, 7.0, AMBER, &format!("{:.0}k", w.range_km));
        }

        // Draw flying projectiles
        for p in &self.projectiles {
            let (start, end) = if p.from_craft {
                (interceptor_pos, ufo_pos)
            } else {
                (ufo_pos, interceptor_pos)
            };
            let cur = start.lerp(end, p.progress);
            if p.is_missile {
                draw_circle(cur.x, cur.y, 3.5, p.color);
                // Smoke trail
                let back = cur - (end - start).normalize_or_zero() * 12.0;
                draw_line(back.x, back.y, cur.x, cur.y, 2.0, Color { a: 0.7, ..p.color });
            } else {
                let tail = cur - (end - start).normalize_or_zero() * 16.0;
                draw_line(tail.x, tail.y, cur.x, cur.y, 2.5, p.color);
            }
        }

        // Draw hit sparks
        for s in &self.sparks {
            let a = s.life / s.max_life;
            draw_circle(s.pos.x, s.pos.y, s.size * a, Color { a: a.clamp(0.0, 1.0), ..s.color });
        }

        // Range display banner above radar
        let range_col = if self.distance_km <= self.ufo_weapon_range {
            ui::RED
        } else if self.distance_km <= self.max_effective_range() {
            CYAN
        } else {
            AMBER
        };
        let range_panel = Rect::new(radar_cx - 150.0, radar_cy - radar_r - 46.0, 300.0, 36.0);
        panel(range_panel, Color { a: 0.85, ..PANEL }, Color { a: 0.6, ..CYAN_DIM });
        game.ui.center(range_panel.x + 150.0, range_panel.y + 24.0, 16.0, range_col, &format!("ENGAGEMENT RANGE: {:.1} KM", self.distance_km));

        // ==================== LEFT PANEL: CRAFT TELEMETRY
        let lp = Rect::new(20.0, 56.0, 240.0, sh - 140.0);
        panel(lp, Color { a: 0.9, ..PANEL }, Color { a: 0.45, ..CYAN_DIM });
        let mut ly = lp.y + 24.0;
        game.ui.bold(lp.x + 16.0, ly, 14.0, CYAN, &self.craft_name);
        ly += 16.0;
        game.ui.text(lp.x + 16.0, ly, 10.0, DIM, &format!("SPEED: {} KTS // MAX HP: {}", self.craft_speed as i32, self.craft_max_hp));
        ly += 20.0;

        // Interceptor Hull Bar
        game.ui.text(lp.x + 16.0, ly, 10.0, TEXT, "HULL INTEGRITY");
        game.ui.text(lp.x + lp.w - 70.0, ly, 10.0, if self.craft_hp < 30 { ui::RED } else { CYAN }, &format!("{}/{}", self.craft_hp, self.craft_max_hp));
        ly += 8.0;
        let hp_bar = Rect::new(lp.x + 16.0, ly, lp.w - 32.0, 12.0);
        draw_rectangle(hp_bar.x, hp_bar.y, hp_bar.w, hp_bar.h, Color { r: 0.05, g: 0.1, b: 0.15, a: 1.0 });
        let hp_frac = (self.craft_hp as f32 / self.craft_max_hp as f32).clamp(0.0, 1.0);
        let hp_fill_col = if hp_frac < 0.3 { ui::RED } else if hp_frac < 0.6 { AMBER } else { CYAN };
        draw_rectangle(hp_bar.x, hp_bar.y, hp_bar.w * hp_frac, hp_bar.h, hp_fill_col);
        draw_rectangle_lines(hp_bar.x, hp_bar.y, hp_bar.w, hp_bar.h, 1.0, CYAN_DIM);
        ly += 28.0;

        // Weapon Systems bays
        game.ui.bold(lp.x + 16.0, ly, 12.0, TEXT, "WEAPON SYSTEMS");
        ly += 18.0;
        for i in 0..2 {
            let wp_rect = Rect::new(lp.x + 12.0, ly, lp.w - 24.0, 68.0);
            panel(wp_rect, Color { a: 0.5, ..PANEL_2 }, Color { a: 0.3, ..CYAN_DIM });
            if let Some(w) = &self.weapons[i] {
                game.ui.bold(wp_rect.x + 8.0, wp_rect.y + 16.0, 10.0, CYAN, w.name);
                game.ui.text(wp_rect.x + 8.0, wp_rect.y + 30.0, 9.0, DIM, &format!("RANGE: {:.0} KM  DMG: {}", w.range_km, w.damage));

                // Ammo status
                let ammo_str = if w.max_ammo == 0 {
                    "UNLIMITED".to_string()
                } else {
                    format!("AMMO: {} / {}", w.ammo, w.max_ammo)
                };
                let ammo_col = if w.max_ammo > 0 && w.ammo == 0 { ui::RED } else { TEXT };
                game.ui.text(wp_rect.x + 8.0, wp_rect.y + 44.0, 9.0, ammo_col, &ammo_str);

                // Reload / Cooldown progress bar
                let cool_frac = 1.0 - (w.cooldown / w.cooldown_max).clamp(0.0, 1.0);
                let cool_rect = Rect::new(wp_rect.x + 8.0, wp_rect.y + 52.0, wp_rect.w - 16.0, 6.0);
                draw_rectangle(cool_rect.x, cool_rect.y, cool_rect.w, cool_rect.h, Color { r: 0.05, g: 0.1, b: 0.15, a: 1.0 });
                let cool_col = if cool_frac >= 1.0 { ui::GREEN } else { AMBER };
                draw_rectangle(cool_rect.x, cool_rect.y, cool_rect.w * cool_frac, cool_rect.h, cool_col);
            } else {
                game.ui.text(wp_rect.x + 12.0, wp_rect.y + 36.0, 10.0, DIM, "POD EMPTY");
            }
            ly += 76.0;
        }

        // ==================== RIGHT PANEL: TELEMETRY FEED & UFO STATUS
        let rp = Rect::new(sw - 330.0, 56.0, 310.0, sh - 140.0);
        panel(rp, Color { a: 0.9, ..PANEL }, Color { a: 0.45, ..CYAN_DIM });
        let mut ry = rp.y + 24.0;

        // UFO Telemetry Card
        game.ui.bold(rp.x + 16.0, ry, 14.0, if self.ufo_hp <= 0 { ui::GREEN } else { ui::RED }, &self.ufo_name);
        ry += 16.0;
        game.ui.text(rp.x + 16.0, ry, 10.0, DIM, &format!("SPEED: {} KTS // WEAPON: {}", self.ufo_speed as i32, self.ufo_weapon_name));
        ry += 20.0;

        // UFO Hull Bar
        game.ui.text(rp.x + 16.0, ry, 10.0, TEXT, "TARGET INTEGRITY");
        game.ui.text(rp.x + rp.w - 80.0, ry, 10.0, if self.ufo_hp < 100 { ui::GREEN } else { ui::RED }, &format!("{}/{}", self.ufo_hp, self.ufo_max_hp));
        ry += 8.0;
        let u_bar = Rect::new(rp.x + 16.0, ry, rp.w - 32.0, 12.0);
        draw_rectangle(u_bar.x, u_bar.y, u_bar.w, u_bar.h, Color { r: 0.1, g: 0.05, b: 0.05, a: 1.0 });
        let u_frac = (self.ufo_hp as f32 / self.ufo_max_hp as f32).clamp(0.0, 1.0);
        draw_rectangle(u_bar.x, u_bar.y, u_bar.w * u_frac, u_bar.h, if u_frac < 0.25 { ui::GREEN } else { ui::RED });
        draw_rectangle_lines(u_bar.x, u_bar.y, u_bar.w, u_bar.h, 1.0, Color { a: 0.6, ..ui::RED });
        ry += 26.0;

        // Running Combat Feed Log
        game.ui.bold(rp.x + 16.0, ry, 12.0, TEXT, "COMBAT TELEMETRY LOG");
        ry += 14.0;
        let feed_box = Rect::new(rp.x + 10.0, ry, rp.w - 20.0, rp.h - (ry - rp.y) - 10.0);
        panel(feed_box, Color { r: 0.015, g: 0.035, b: 0.045, a: 0.95 }, Color { a: 0.35, ..CYAN_DIM });

        let mut log_y = feed_box.y + 18.0;
        for entry in &self.logs {
            game.ui.text(feed_box.x + 8.0, log_y, 9.0, entry.color, &entry.text);
            log_y += 15.0;
            if log_y > feed_box.y + feed_box.h - 10.0 { break; }
        }

        // ==================== BOTTOM PANEL: TACTICAL STANCE CONTROLS
        let bp = Rect::new(20.0, sh - 74.0, sw - 40.0, 58.0);
        panel(bp, Color { a: 0.95, ..PANEL }, Color { a: 0.5, ..CYAN_DIM });

        let btn_w = (bp.w - 36.0) / 5.0;
        let stances = [
            (DogfightStance::Shadow, "[1] SHADOW", Btn::Ghost),
            (DogfightStance::LongShot, "[2] LONG SHOT", Btn::Primary),
            (DogfightStance::Press, "[3] PRESS", Btn::Primary),
            (DogfightStance::CloseIn, "[4] CLOSE IN", Btn::Danger),
            (DogfightStance::BreakOff, "[ESC] BREAK OFF", Btn::Ghost),
        ];

        let mut picked_stance = None;
        for (i, (st, label, style)) in stances.iter().enumerate() {
            let r = Rect::new(bp.x + 10.0 + i as f32 * (btn_w + 4.0), bp.y + 12.0, btn_w, 36.0);
            let active = self.stance == *st;
            let b_style = if active { Btn::Amber } else { *style };
            if button(&game.ui, r, label, b_style, true) {
                picked_stance = Some(*st);
            }
        }
        if let Some(st) = picked_stance {
            game.audio.play_click();
            self.set_stance(st);
        }
    }
}

pub fn update(g: &mut Game, dt: f32) {
    let Some(mut df) = g.dogfight.take() else { return; };
    df.tick(g, dt);
    if g.screen == Screen::Dogfight {
        g.dogfight = Some(df);
    }
}

pub fn draw(g: &mut Game) {
    let Some(mut df) = g.dogfight.take() else { return; };
    df.render(g);
    if g.screen == Screen::Dogfight {
        g.dogfight = Some(df);
    }
}
