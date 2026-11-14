use macroquad::prelude::*;
use crate::ui::{self, *};
use crate::{Game, Screen, CraftInstance, ActiveResearch, ActiveProduction};
use crate::data::*;

pub fn update(g: &mut Game, dt: f32) {
    // mouse wheel back out to orbit, exactly like Google Maps
    if mouse_wheel().1 < -0.5 && !is_key_down(KeyCode::LeftShift) {
        g.return_to_globe();
        return;
    }
    if is_key_pressed(KeyCode::Escape) {
        g.return_to_globe();
        return;
    }
    if is_key_pressed(KeyCode::Key1) { g.base_tab = 0; }
    if is_key_pressed(KeyCode::Key2) { g.base_tab = 1; }
    if is_key_pressed(KeyCode::Key3) { g.base_tab = 2; }
    if is_key_pressed(KeyCode::Key4) { g.base_tab = 3; }
    if is_key_pressed(KeyCode::Key5) { g.base_tab = 4; }
    if is_key_pressed(KeyCode::Key6) { g.base_tab = 5; }
    if is_key_pressed(KeyCode::Key7) { g.base_tab = 6; }
    if is_key_pressed(KeyCode::Key8) { g.base_tab = 7; }

    // scroll the active list with the wheel when shift is held
    if is_key_down(KeyCode::LeftShift) {
        let w = mouse_wheel().1;
        if w.abs() > 0.1 {
            let max = g.soldiers.len().saturating_sub(1);
            let cur = match g.base_tab {
                0 => &mut g.roster_scroll,
                2 => &mut g.research_scroll,
                3 => &mut g.workshop_scroll,
                4 => &mut g.equip_scroll,
                6 => &mut g.finance_scroll,
                _ => &mut g.equip_scroll,
            };
            if w < 0.0 { *cur = (*cur + 1).min(max); } else { *cur = cur.saturating_sub(1); }
        }
    }
    let _ = dt;
}

fn header(g: &mut Game, title: &str, sub: &str) {
    let w = screen_width();
    draw_rectangle(0.0, 0.0, w, 48.0, Color { r: 0.012, g: 0.030, b: 0.042, a: 0.96 });
    draw_rectangle(0.0, 48.0, w, 1.0, Color { a: 0.6, ..CYAN });
    g.ui.label(16.0, 29.0, 12.0, CYAN, "BASE");
    g.ui.bold(78.0, 31.0, 17.0, TEXT, title);
    g.ui.text(16.0, 64.0, 10.0, DIM, sub);
    g.ui.text(w - 500.0, 29.0, 11.0, DIM, &g.date.pretty());
    g.ui.text(w - 340.0, 29.0, 11.0, ui::GREEN, &format!("${}", crate::fmt_money(g.funds)));
    if button(&g.ui, Rect::new(w - 150.0, 10.0, 134.0, 28.0), "ORBIT  [ESC]", Btn::Normal, true) {
        g.return_to_globe();
    }
}

fn nav(g: &mut Game, r: Rect) {
    let tabs = ["SOLDIERS", "FACILITIES", "RESEARCH", "WORKSHOP", "MARKET", "FLEET & CRAFT", "FINANCES", "LAUNCH"];
    let mut y = r.y + 6.0;
    for (i, t) in tabs.iter().enumerate() {
        let b = Rect::new(r.x + 4.0, y, r.w - 8.0, 34.0);
        let active = g.base_tab == i;
        let hover = b.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(b.x, b.y, b.w, b.h, if active { Color { r: 0.05, g: 0.18, b: 0.22, a: 1.0 } } else if hover { Color { a: 0.15, ..PANEL_2 } } else { Color { a: 0.30, ..PANEL } });
        draw_rectangle(b.x, b.y, 3.0, b.h, if active { CYAN } else { Color { a: 0.25, ..CYAN_DIM } });
        g.ui.bold(b.x + 10.0, b.y + 22.0, 10.0, if active { CYAN } else { TEXT }, t);
        if hover && is_mouse_button_released(MouseButton::Left) { g.audio.play_click(); g.base_tab = i; }
        y += 38.0;
    }

    // Base vital metrics block
    if let Some(b) = g.bases.get(g.selected_base) {
        let my = y + 14.0;
        draw_rectangle(r.x + 6.0, my - 6.0, r.w - 12.0, 1.0, Color { a: 0.3, ..CYAN_DIM });
        g.ui.label(r.x + 8.0, my + 8.0, 9.0, CYAN, "BASE STATUS");

        let living_used = g.soldiers.len() as i32 + b.scientists + b.engineers;
        let living_cap = b.living_cap();
        let lab_cap = b.lab_cap();
        let work_cap = b.workshop_cap();
        let stores_cap = b.stores_cap();
        let craft_cap = b.craft_cap();

        let metrics = [
            ("LIVING", format!("{}/{}", living_used, living_cap), living_used <= living_cap),
            ("LABS", format!("{}/{}", b.scientists, lab_cap), b.scientists <= lab_cap),
            ("WORKSHOP", format!("{}/{}", b.engineers, work_cap), b.engineers <= work_cap),
            ("STORES", format!("{} CAP", stores_cap), true),
            ("HANGARS", format!("{}/{}", b.craft.len(), craft_cap), b.craft.len() <= craft_cap as usize),
            ("RADAR", format!("{} NM", b.radar_range()), true),
            ("DEFENSE", format!("{} PTS", b.defense_rating()), true),
        ];

        let mut py = my + 24.0;
        for (label, val, ok) in metrics {
            g.ui.text(r.x + 8.0, py, 8.5, DIM, label);
            g.ui.bold(r.x + r.w - g.ui.bwidth(8.5, &val) - 10.0, py, 8.5, if ok { TEXT } else { ui::RED }, &val);
            py += 17.0;
        }

        let upkeep = b.monthly_upkeep();
        g.ui.text(r.x + 8.0, py + 4.0, 8.5, AMBER, &format!("UPKEEP -${}/MO", crate::fmt_money(upkeep)));
    }
}

pub fn draw(g: &mut Game) {
    let w = screen_width(); let h = screen_height();
    base_background(g);
    let bsel = g.bases.get(g.selected_base).cloned();
    let bname = bsel.as_ref().map(|b| b.name.clone()).unwrap_or_else(|| "HEARTH".into());
    let sub = match &bsel {
        Some(b) if b.water => "SEA-FLOOR STATION // PRESSURE HULL 1",
        _ => "UNDERGROUND COMMAND COMPLEX // SURFACE COMMAND",
    };
    header(g, &bname, sub);
    let nav_r = Rect::new(16.0, 88.0, 170.0, h - 150.0);
    draw_rectangle(nav_r.x, nav_r.y, nav_r.w, nav_r.h, Color { a: 0.45, ..PANEL });
    nav(g, nav_r);
    let content = Rect::new(200.0, 80.0, w - 216.0, h - 132.0);
    draw_rectangle(content.x, content.y, content.w, content.h, Color { a: 0.35, ..PANEL });
    draw_rectangle_lines(content.x, content.y, content.w, content.h, 1.0, Color { a: 0.35, ..CYAN_DIM });

    match g.base_tab {
        0 => draw_soldiers(g, content),
        1 => draw_facilities(g, content),
        2 => draw_research(g, content),
        3 => draw_workshop(g, content),
        4 => draw_market(g, content),
        5 => draw_fleet(g, content),
        6 => draw_finances(g, content),
        _ => draw_launch(g, content),
    }

    draw_rectangle(0.0, h - 44.0, w, 44.0, Color { r: 0.012, g: 0.028, b: 0.040, a: 0.96 });
    let env = if bsel.as_ref().map(|b| b.water).unwrap_or(false) { "SEA-FLOOR STATION" } else { "LAND BASE" };
    g.ui.text(16.0, h - 18.0, 10.0, DIM, &format!("WHEEL DOWN TO ZOOM BACK TO ORBIT   ESC ORBIT   [1-8] TABS   SHIFT+WHEEL SCROLL   //   {}", env));
    g.ui.text(w - 300.0, h - 18.0, 10.0, DIM, "CLICK A FACILITY OR TAB TO ACCESS");
}

fn base_background(g: &Game) {
    let w = screen_width(); let h = screen_height();
    grad_v(Rect::new(0.0, 0.0, w, h), Color { r: 0.018, g: 0.045, b: 0.058, a: 1.0 }, Color { r: 0.004, g: 0.010, b: 0.016, a: 1.0 });
    let t = g.elapsed;
    for i in 0..12 {
        let y = h * 0.58 + i as f32 * 12.0;
        draw_rectangle(0.0, y, w, 1.0, Color { a: 0.05 - i as f32 * 0.003, ..CYAN });
    }
    let horizon = h * 0.56;
    for i in 0..18 {
        let f = i as f32 / 18.0;
        let y = horizon + (h - horizon) * f * f;
        draw_line(0.0, y, w, y, 1.0, Color { a: 0.08, ..CYAN });
    }
    for i in -10..=10 {
        let x = w * 0.5 + i as f32 * 90.0;
        draw_line(w * 0.5, horizon, x, h, 1.0, Color { a: 0.055, ..CYAN });
    }
    for i in 0..7 {
        let x = 30.0 + i as f32 * (w - 60.0) / 6.0;
        let pulse = 0.5 + 0.5 * (t * 0.8 + i as f32).sin();
        glow(x, 52.0, 26.0, CYAN, 0.28 * pulse);
        draw_rectangle(x - 22.0, 50.0, 44.0, 3.0, Color { a: 0.5, ..CYAN });
    }
    for i in 0..34 {
        let seed = i as f32 * 37.7;
        let x = (seed.sin() * 0.5 + 0.5) * w;
        let y = ((seed.cos() * 0.5 + 0.5) + t * 0.004 * (i as f32 % 3.0 + 1.0)) % 1.0 * h;
        draw_circle(x, y, 1.0, Color { a: 0.10, ..CYAN });
    }
}

// ---------------------------------------------------------------- 0: SOLDIERS & LOADOUT
fn draw_soldier_3d_preview(r: Rect, s: &Soldier, g: &Game) {
    draw_rectangle(r.x, r.y, r.w, r.h, Color { a: 0.85, ..BG });
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, Color { a: 0.45, ..CYAN_DIM });
    g.ui.label(r.x + 6.0, r.y + 12.0, 7.5, CYAN, "3D LOADOUT FIGURE");

    let cx = r.x + r.w * 0.5;
    let cy = r.y + r.h * 0.62;
    let yaw = (g.elapsed * 0.9) % std::f32::consts::TAU;
    let (sy, cy_rot) = (yaw.sin(), yaw.cos());

    let armored = s.armor > 0;
    let armor_def = &ARMORS[s.armor as usize % ARMORS.len()];
    let uniform = if armored {
        Color::new(armor_def.color[0], armor_def.color[1], armor_def.color[2], 1.0)
    } else {
        Color::new(0.18, 0.26, 0.20, 1.0)
    };
    let skin = skin_color(s.look.skin);
    let scale = 36.0;

    let project = |x: f32, y: f32, z: f32| -> Vec2 {
        let rx = x * cy_rot + z * sy;
        let rz = -x * sy + z * cy_rot;
        vec2(cx + rx * scale, cy - y * scale + rz * scale * 0.35)
    };

    // Floor shadow
    draw_circle(cx, cy + 3.0, 16.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.35 });

    // Legs
    for side in [-1.0f32, 1.0] {
        let p0 = project(side * 0.16, 0.0, 0.0);
        let p1 = project(side * 0.16, 0.65, 0.0);
        draw_line(p0.x, p0.y, p1.x, p1.y, 5.0, Color { r: uniform.r * 0.7, g: uniform.g * 0.7, b: uniform.b * 0.7, a: 1.0 });
        draw_circle(p0.x, p0.y, 3.5, Color::new(0.08, 0.08, 0.08, 1.0));
        if s.armor >= 1 {
            let pk = project(side * 0.16, 0.35, 0.08);
            draw_circle(pk.x, pk.y, 3.0, Color { r: uniform.r * 1.25, g: uniform.g * 1.25, b: uniform.b * 1.25, a: 1.0 });
        }
    }

    // Torso
    let t0 = project(0.0, 0.65, 0.0);
    let t1 = project(0.0, 1.20, 0.0);
    draw_line(t0.x, t0.y, t1.x, t1.y, 12.0, uniform);
    let cp = project(0.0, 0.95, 0.12);
    draw_rectangle(cp.x - 5.0, cp.y - 6.0, 10.0, 12.0, Color { r: uniform.r * 1.25, g: uniform.g * 1.25, b: uniform.b * 1.25, a: 1.0 });

    // Sealed frame core housing
    if s.armor == 2 || s.armor == 4 {
        let rc = project(0.0, 1.00, 0.14);
        draw_circle(rc.x, rc.y, 3.0, Color::new(0.2, 0.9, 1.0, 0.95));
        draw_circle(rc.x, rc.y, 1.5, ui::WHITE);
    }
    // Dive rig thruster pack
    if s.armor >= 3 {
        for side in [-1.0f32, 1.0] {
            let tp = project(side * 0.22, 1.10, -0.16);
            draw_rectangle(tp.x - 3.0, tp.y - 7.0, 6.0, 14.0, Color::new(0.25, 0.28, 0.32, 1.0));
            let ep = project(side * 0.22, 0.88, -0.16);
            draw_circle(ep.x, ep.y, 2.5, Color::new(0.2, 0.8, 1.0, 0.9));
        }
    }

    // Arms & Shoulders
    for side in [-1.0f32, 1.0] {
        let sp = project(side * 0.30, 1.15, 0.0);
        let ep = project(side * 0.32, 0.82, 0.12);
        draw_line(sp.x, sp.y, ep.x, ep.y, 4.5, uniform);
        if armored {
            draw_circle(sp.x, sp.y, 4.5, Color { r: uniform.r * 1.25, g: uniform.g * 1.25, b: uniform.b * 1.25, a: 1.0 });
        }
    }

    // Head / Helmet
    let hp = project(0.0, 1.40, 0.0);
    if armored {
        draw_circle(hp.x, hp.y, 7.5, Color { r: uniform.r * 1.15, g: uniform.g * 1.15, b: uniform.b * 1.15, a: 1.0 });
        let vp = project(0.0, 1.40, 0.16);
        draw_rectangle(vp.x - 4.5, vp.y - 2.0, 9.0, 4.0, Color::new(0.25, 0.75, 0.85, 1.0));
    } else {
        draw_circle(hp.x, hp.y, 7.0, skin);
        let hair_c = hair_color(s.look.hair_color);
        let hair_p = project(0.0, 1.48, -0.02);
        draw_circle(hair_p.x, hair_p.y, 7.0, hair_c);
    }

    // Tactical items
    if s.grenades > 0 {
        let gp = project(-0.12, 1.08, 0.14);
        draw_circle(gp.x, gp.y, 2.5, Color::new(0.25, 0.35, 0.22, 1.0));
    }
    if s.smoke_grenades > 0 {
        let sp = project(0.20, 0.72, 0.0);
        draw_rectangle(sp.x - 2.0, sp.y - 3.5, 4.0, 7.0, Color::new(0.40, 0.40, 0.42, 1.0));
        draw_circle(sp.x, sp.y - 3.5, 1.8, AMBER);
    }
    if s.medkits > 0 {
        let mp = project(-0.22, 0.70, 0.0);
        draw_rectangle(mp.x - 3.5, mp.y - 3.5, 7.0, 7.0, ui::WHITE);
        draw_rectangle(mp.x - 1.0, mp.y - 2.5, 2.0, 5.0, ui::RED);
        draw_rectangle(mp.x - 2.5, mp.y - 1.0, 5.0, 2.0, ui::RED);
    }

    // Weapon
    let wdef = &g.data.weapons[s.weapon % g.data.weapons.len()];
    let wp0 = project(0.16, 0.90, 0.12);
    let wp1 = project(0.20, 0.98, 0.42);
    let w_col = match wdef.cat {
        Cat::Heavy | Cat::Cannon | Cat::Launcher => Color::new(0.20, 0.22, 0.20, 1.0),
        Cat::Pistol => Color::new(0.16, 0.17, 0.18, 1.0),
        _ => Color::new(0.22, 0.24, 0.22, 1.0),
    };
    draw_line(wp0.x, wp0.y, wp1.x, wp1.y, if wdef.cat == Cat::Heavy { 5.5 } else { 3.5 }, w_col);
    let tc = Color::new(wdef.tracer[0], wdef.tracer[1], wdef.tracer[2], 0.9);
    draw_circle(wp1.x, wp1.y, 2.2, tc);

    g.ui.center(r.x + r.w * 0.5, r.bottom() - 8.0, 7.5, DIM, &format!("SUIT: {}", armor_def.name));
}

fn draw_equipment_inspector_3d(r: Rect, g: &mut Game, s: &Soldier) {
    draw_rectangle(r.x, r.y, r.w, r.h, Color { a: 0.85, ..PANEL });
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, Color { a: 0.4, ..CYAN_DIM });

    let tab_w = (r.w - 12.0) / 3.0;
    let tabs = ["WEAPON", "ARMOR", "GEAR"];
    for (i, &tname) in tabs.iter().enumerate() {
        let tr = Rect::new(r.x + 6.0 + i as f32 * tab_w, r.y + 4.0, tab_w - 2.0, 18.0);
        let sel = g.inspector_gear_tab == i;
        let hover = tr.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(tr.x, tr.y, tr.w, tr.h, if sel { Color { a: 0.4, ..CYAN } } else if hover { Color { a: 0.18, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
        draw_rectangle_lines(tr.x, tr.y, tr.w, tr.h, 1.0, if sel { CYAN } else { Color { a: 0.3, ..CYAN_DIM } });
        g.ui.center(tr.x + tr.w * 0.5, tr.y + 12.0, 7.5, if sel { ui::WHITE } else { TEXT }, tname);
        if hover && is_mouse_button_released(MouseButton::Left) { g.inspector_gear_tab = i; }
    }

    let cx = r.x + 45.0;
    let cy = r.y + 80.0;
    let time = g.elapsed;
    let rot = time * 1.5;

    match g.inspector_gear_tab {
        0 => {
            let wdef = &g.data.weapons[s.weapon % g.data.weapons.len()];
            g.ui.bold(r.x + 95.0, r.y + 30.0, 9.5, CYAN, wdef.name);
            let item = g.data.item(wdef.game, wdef.item);
            g.ui.text(r.x + 95.0, r.y + 44.0, 8.0, DIM, &format!("ENV: {:?} // WT: {}kg", wdef.env, item.weight));
            let dmg = g.data.weapon_damage(wdef);
            g.ui.text(r.x + 95.0, r.y + 56.0, 8.0, TEXT, &format!("PWR: {} // {}", dmg, { let labels: [&str; 8] = if wdef.game == 0 { ["KIN", "HEAT", "BLAST", "CORE", "ARC", "STUN", "CUT", "ACID"] } else { ["KIN", "HEAT", "BLAST", "CAV", "PRES", "STUN", "CUT", "ACID"] }; labels[(g.data.weapon_damage_type(wdef) as usize).min(7)] }));
            g.ui.text(r.x + 95.0, r.y + 68.0, 7.5, DIM, &format!("SNAP: {}% ({})", item.acc[1], item.tu[1]));
            g.ui.text(r.x + 95.0, r.y + 80.0, 7.5, DIM, &format!("AUTO: {}% ({})", item.acc[0], item.tu[0]));
            g.ui.text(r.x + 95.0, r.y + 92.0, 7.5, DIM, &format!("AIM:  {}% ({})", item.acc[2], item.tu[2]));

            let w_col = Color::new(wdef.tracer[0], wdef.tracer[1], wdef.tracer[2], 0.95);
            let p0 = vec2(cx - rot.cos() * 24.0, cy - rot.sin() * 10.0);
            let p1 = vec2(cx + rot.cos() * 24.0, cy + rot.sin() * 10.0);
            draw_line(p0.x, p0.y, p1.x, p1.y, 6.0, Color::new(0.25, 0.28, 0.25, 1.0));
            draw_line(p0.x, p0.y - 2.5, p1.x, p1.y - 2.5, 2.0, w_col);
            draw_circle(p1.x, p1.y, 3.5, w_col);
            let mp = vec2(cx - rot.cos() * 6.0, cy + 10.0);
            draw_rectangle(mp.x - 2.5, mp.y - 5.0, 5.0, 10.0, Color::new(0.18, 0.18, 0.20, 1.0));
        }
        1 => {
            let arm = &ARMORS[s.armor as usize % ARMORS.len()];
            g.ui.bold(r.x + 95.0, r.y + 30.0, 9.5, CYAN, arm.name);
            g.ui.text(r.x + 95.0, r.y + 44.0, 8.0, DIM, &format!("CLS: {} // WT: {}kg", arm.armor_class, arm.weight));
            g.ui.text(r.x + 95.0, r.y + 56.0, 8.0, TEXT, &format!("F:{} S:{} R:{} U:{}", arm.front, arm.side, arm.rear, arm.under));
            g.ui.text(r.x + 95.0, r.y + 70.0, 8.0, if arm.underwater { CYAN } else { AMBER }, if arm.underwater { "PRESSURE-SEALED" } else { "SURFACE-RATED" });

            let arm_col = Color::new(arm.color[0], arm.color[1], arm.color[2], 1.0);
            draw_circle(cx, cy - 8.0, 13.0, arm_col);
            draw_rectangle(cx - rot.cos() * 9.0 - 3.0, cy - 10.0, 11.0, 5.0, CYAN);
            draw_rectangle(cx - 13.0, cy + 4.0, 26.0, 16.0, arm_col);
            if s.armor == 2 || s.armor == 4 {
                draw_circle(cx, cy + 12.0, 4.0, Color::new(0.2, 0.9, 1.0, 0.95));
            }
        }
        _ => {
            g.ui.bold(r.x + 95.0, r.y + 30.0, 9.5, CYAN, "TACTICAL ORDNANCE");
            g.ui.text(r.x + 95.0, r.y + 44.0, 8.0, TEXT, &format!("FRAG GRENADES: {}", s.grenades));
            g.ui.text(r.x + 95.0, r.y + 56.0, 8.0, TEXT, &format!("SMOKE CANISTERS: {}", s.smoke_grenades));
            g.ui.text(r.x + 95.0, r.y + 68.0, 8.0, TEXT, &format!("TRAUMA KITS: {}", s.medkits));
            g.ui.text(r.x + 95.0, r.y + 80.0, 7.5, DIM, "WEIGHT BURDENS TU");

            let gx = cx - 14.0; let mx = cx + 14.0;
            draw_circle(gx, cy, 8.0, Color::new(0.3, 0.4, 0.25, 1.0));
            draw_rectangle(gx - 2.5, cy - 11.0, 5.0, 5.0, Color::new(0.5, 0.5, 0.55, 1.0));
            draw_rectangle(mx - 8.0, cy - 8.0, 16.0, 16.0, ui::WHITE);
            draw_rectangle(mx - 2.0, cy - 6.0, 4.0, 12.0, ui::RED);
            draw_rectangle(mx - 6.0, cy - 2.0, 12.0, 4.0, ui::RED);
        }
    }
}

fn draw_soldiers(g: &mut Game, c: Rect) {
    let list_w = c.w * 0.38;
    let list = Rect::new(c.x + 8.0, c.y + 8.0, list_w, c.h - 16.0);
    draw_rectangle(list.x, list.y, list.w, list.h, Color { a: 0.35, ..PANEL });
    let phead = format!("PERSONNEL // {} ACTIVE", g.soldiers.iter().filter(|s| !s.dead).count());
    g.ui.label(list.x + 10.0, list.y + 20.0, 9.5, CYAN, &phead);

    let bsel = g.bases.get(g.selected_base).cloned();
    let living_ok = bsel.as_ref().map(|b| (g.soldiers.len() as i32 + b.scientists + b.engineers) < b.living_cap()).unwrap_or(true);

    if button(&g.ui, Rect::new(list.x + list.w - 96.0, list.y + 10.0, 86.0, 22.0), "HIRE $25K", Btn::Amber, living_ok) {
        if g.funds >= 25_000 && living_ok {
            g.funds -= 25_000;
            let mut s = g.make_soldier();
            s.weapon = g.weapon_index("SERVICE RIFLE");
            s.ammo = g.data.clip_size(&g.data.weapons[s.weapon]);
            s.spare = 2;
            g.soldiers.push(s);
            g.selected_soldier = g.soldiers.len() - 1;
            g.log("NEW RECRUIT ARRIVED AT BASE.", CYAN);
        } else { g.toast(if !living_ok { "BARRACKS FULL" } else { "INSUFFICIENT FUNDS" }); }
    }

    let rows_top = list.y + 42.0;
    let row_h = 37.0;
    let max_rows = ((list.h - 50.0) / row_h).max(1.0) as usize;
    let max_scroll = g.soldiers.len().saturating_sub(max_rows);
    g.roster_scroll = g.roster_scroll.min(max_scroll);
    if max_scroll > 0 {
        if button(&g.ui, Rect::new(list.x + list.w - 54.0, list.y + list.h - 24.0, 22.0, 18.0), "<", Btn::Ghost, g.roster_scroll > 0) { g.roster_scroll = g.roster_scroll.saturating_sub(1); }
        if button(&g.ui, Rect::new(list.x + list.w - 28.0, list.y + list.h - 24.0, 22.0, 18.0), ">", Btn::Ghost, g.roster_scroll < max_scroll) { g.roster_scroll = (g.roster_scroll + 1).min(max_scroll); }
    }
    let start = g.roster_scroll;
    for (vis, idx) in (start..g.soldiers.len()).take(max_rows).enumerate() {
        let s = g.soldiers[idx].clone();
        let r = Rect::new(list.x + 6.0, rows_top + vis as f32 * row_h, list.w - 12.0, row_h - 3.0);
        let sel = idx == g.selected_soldier;
        let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(r.x, r.y, r.w, r.h, if sel { Color { r: 0.06, g: 0.18, b: 0.20, a: 1.0 } } else if hover { Color { a: 0.14, ..PANEL_2 } } else { Color { a: 0.10, ..PANEL } });
        if sel { draw_rectangle(r.x, r.y, 2.0, r.h, CYAN); }
        let pr = Rect::new(r.x + 3.0, r.y + 2.0, 28.0, 30.0);
        draw_rectangle(pr.x, pr.y, pr.w, pr.h, Color { a: 0.8, ..BG });
        ui::portrait(&g.ui, pr, s.look, g.elapsed, false);
        let col = if s.dead { ui::RED } else if s.wounded_days > 0 { AMBER } else { TEXT };
        let name = g.ui.fit(10.0, &s.name, r.w - 36.0 - 74.0, true);
        g.ui.bold(r.x + 36.0, r.y + 12.0, 10.0, col, &name);
        let sub = g.ui.fit(8.0, &format!("{} // {} M / {} K", RANKS[s.rank as usize], s.missions, s.kills), r.w - 36.0 - 74.0, false);
        g.ui.text(r.x + 36.0, r.y + 23.0, 8.0, DIM, &sub);
        g.ui.text(r.x + 36.0, r.y + 33.0, 8.0, if s.assigned { ui::GREEN } else { DIM }, if s.assigned { "ASSIGNED" } else { "STANDBY" });
        g.ui.text(r.x + r.w - 62.0, r.y + 11.0, 8.0, TEXT, &format!("HP {}/{}", s.hp, s.stats.health));
        g.ui.text(r.x + r.w - 62.0, r.y + 22.0, 8.0, TEXT, &format!("TU {}", s.stats.tu));
        g.ui.text(r.x + r.w - 62.0, r.y + 33.0, 8.0, DIM, &format!("FA {}", s.stats.firing));
        if hover && is_mouse_button_released(MouseButton::Left) { g.selected_soldier = idx; }
    }

    // Detail panel
    let d = Rect::new(c.x + list_w + 14.0, c.y + 8.0, c.w - list_w - 22.0, c.h - 16.0);
    draw_rectangle(d.x, d.y, d.w, d.h, Color { a: 0.30, ..PANEL });
    let s = g.soldiers.get(g.selected_soldier).cloned();
    let Some(s) = s else { return; };
    let pr = Rect::new(d.x + 10.0, d.y + 12.0, 108.0, 160.0);
    ui::portrait(&g.ui, pr, s.look, g.elapsed, true);

    let fig_r = Rect::new(d.x + 124.0, d.y + 12.0, 122.0, 160.0);
    draw_soldier_3d_preview(fig_r, &s, g);

    let ix = d.x + 254.0;
    let avail = d.x + d.w - ix - 12.0;
    g.ui.label(ix, d.y + 18.0, 8.5, CYAN, "SERVICE RECORD & PROMOTION PROFILE");
    let name = g.ui.fit(15.0, &s.name, avail, true);
    g.ui.bold(ix, d.y + 36.0, 15.0, TEXT, &name);
    g.ui.bold(ix, d.y + 52.0, 10.5, AMBER, &format!("RANK: {}  //  MERIT: {}", RANKS[s.rank as usize], s.missions * 10 + s.kills * 5 + s.stats.bravery as u32 / 2));
    g.ui.text(ix, d.y + 66.0, 8.5, DIM, &format!("ID {:04}  M {}  K {}  ORIGIN: {}", s.id, s.missions, s.kills, s.look.ethnicity.label()));

    let stats = [
        ("TU", s.stats.tu, 80), ("STA", s.stats.stamina, 100), ("HP", s.stats.health, 60),
        ("BRA", s.stats.bravery, 100), ("REA", s.stats.reactions, 100), ("FA", s.stats.firing, 120),
        ("TA", s.stats.throwing, 120), ("STR", s.stats.strength, 70),
    ];
    let yy = d.y + 86.0;
    for (i, (sname, val, max)) in stats.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;
        let sx = ix + col as f32 * 120.0;
        let sy = yy + row as f32 * 18.0;
        g.ui.text(sx, sy, 8.5, DIM, sname);
        g.ui.text(sx + 30.0, sy, 8.5, TEXT, &format!("{}", val));
        bar(Rect::new(sx + 54.0, sy - 5.0, 50.0, 4.0), *val as f32, *max as f32, if *val as f32 / *max as f32 > 0.66 { ui::GREEN } else if *val as f32 / *max as f32 > 0.33 { AMBER } else { ui::RED }, Color { a: 0.25, ..BLACK });
    }

    // Equipment and kit block in the bottom half of the soldier detail
    let eq_y = d.y + 196.0;
    draw_rectangle(d.x + 14.0, eq_y - 8.0, d.w - 28.0, 1.0, Color { a: 0.35, ..CYAN_DIM });
    g.ui.label(d.x + 14.0, eq_y + 8.0, 10.0, CYAN, "TACTICAL LOADOUT");

    let wsel = g.data.weapons[s.weapon].clone();
    let clip_size = g.data.clip_size(&wsel);
    g.ui.text(d.x + 14.0, eq_y + 26.0, 9.5, DIM, "WEAPON:");
    g.ui.bold(d.x + 74.0, eq_y + 26.0, 11.0, TEXT, wsel.name);
    g.ui.text(d.x + 230.0, eq_y + 26.0, 9.5, DIM, &format!("AMMO {}/{} (x{} SPARE)", s.ammo, clip_size, s.spare));

    // Right side of tactical loadout: 3D Equipment Inspector
    let r_insp = Rect::new(d.x + d.w - 236.0, eq_y + 4.0, 224.0, 150.0);
    draw_equipment_inspector_3d(r_insp, g, &s);

    // Left side: Field loadout (frag grenades, smoke grenades, trauma kits)
    let idx = g.selected_soldier;
    let (gren, smoke, med) = (g.soldiers[idx].grenades, g.soldiers[idx].smoke_grenades, g.soldiers[idx].medkits);

    let w_wt = g.data.item(wsel.game, wsel.item).weight as i32;
    let arm_wt = ARMORS[s.armor as usize % ARMORS.len()].weight;
    let total_wt = w_wt + arm_wt + (gren as i32 * 3) + (smoke as i32 * 2) + (med as i32 * 5) + (s.spare * 2);
    let max_str = s.stats.strength;
    let encumbered = total_wt > max_str;

    // Weight and encumbrance bar
    let wt_col = if encumbered { ui::RED } else if total_wt * 4 > max_str * 3 { AMBER } else { ui::GREEN };
    g.ui.text(d.x + 14.0, eq_y + 44.0, 8.5, DIM, &format!("WEIGHT: {}kg / {}kg (STR {})", total_wt, max_str, max_str));
    g.ui.text(d.x + 195.0, eq_y + 44.0, 8.5, wt_col, if encumbered { "OVERBURDENED (-TU)" } else { "OPTIMAL MOBILITY" });
    bar(Rect::new(d.x + 14.0, eq_y + 50.0, d.w - 260.0, 4.0), total_wt as f32, max_str as f32, wt_col, Color { a: 0.35, ..BLACK });

    let ky = eq_y + 60.0;
    g.ui.text(d.x + 14.0, ky + 14.0, 8.5, DIM, "FRAG:");
    if button(&g.ui, Rect::new(d.x + 64.0, ky + 2.0, 20.0, 18.0), "-", Btn::Ghost, gren > 0) { g.audio.play_equip(); g.soldiers[idx].grenades = gren.saturating_sub(1); }
    g.ui.bold(d.x + 88.0, ky + 14.0, 9.5, TEXT, &format!("{}", gren));
    if button(&g.ui, Rect::new(d.x + 100.0, ky + 2.0, 20.0, 18.0), "+", Btn::Ghost, gren < 4) { g.audio.play_equip(); g.soldiers[idx].grenades = (gren + 1).min(4); }

    g.ui.text(d.x + 130.0, ky + 14.0, 8.5, DIM, "SMOKE:");
    if button(&g.ui, Rect::new(d.x + 172.0, ky + 2.0, 20.0, 18.0), "-", Btn::Ghost, smoke > 0) { g.audio.play_equip(); g.soldiers[idx].smoke_grenades = smoke.saturating_sub(1); }
    g.ui.bold(d.x + 196.0, ky + 14.0, 9.5, TEXT, &format!("{}", smoke));
    if button(&g.ui, Rect::new(d.x + 208.0, ky + 2.0, 20.0, 18.0), "+", Btn::Ghost, smoke < 4) { g.audio.play_equip(); g.soldiers[idx].smoke_grenades = (smoke + 1).min(4); }

    g.ui.text(d.x + 236.0, ky + 14.0, 8.5, DIM, "TRAUMA KIT:");
    if button(&g.ui, Rect::new(d.x + 284.0, ky + 2.0, 20.0, 18.0), "-", Btn::Ghost, med > 0) { g.audio.play_equip(); g.soldiers[idx].medkits = med.saturating_sub(1); }
    g.ui.bold(d.x + 308.0, ky + 14.0, 9.5, TEXT, &format!("{}", med));
    if button(&g.ui, Rect::new(d.x + 320.0, ky + 2.0, 20.0, 18.0), "+", Btn::Ghost, med < 3) { g.audio.play_equip(); g.soldiers[idx].medkits = (med + 1).min(3); }

    // Role Presets
    let pry = ky + 26.0;
    g.ui.label(d.x + 14.0, pry + 12.0, 8.0, DIM, "ROLE PRESETS:");
    let presets = [("RIFLEMAN", 0), ("HEAVY", 1), ("MEDIC", 2), ("SCOUT", 3)];
    let pw = 68.0;
    for (pi, (pname, ptype)) in presets.iter().enumerate() {
        let pr = Rect::new(d.x + 90.0 + pi as f32 * (pw + 4.0), pry + 2.0, pw, 18.0);
        let phover = pr.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(pr.x, pr.y, pr.w, pr.h, if phover { Color { a: 0.25, ..PANEL_2 } } else { Color { a: 0.12, ..PANEL } });
        draw_rectangle_lines(pr.x, pr.y, pr.w, pr.h, 1.0, Color { a: 0.4, ..CYAN_DIM });
        g.ui.center(pr.x + pr.w * 0.5, pr.y + 12.0, 7.0, if phover { CYAN } else { TEXT }, pname);
        if phover && is_mouse_button_released(MouseButton::Left) {
            g.audio.play_equip();
            match ptype {
                0 => { g.soldiers[idx].grenades = 2; g.soldiers[idx].smoke_grenades = 1; g.soldiers[idx].medkits = 1; }
                1 => { g.soldiers[idx].grenades = 3; g.soldiers[idx].smoke_grenades = 0; g.soldiers[idx].medkits = 0; }
                2 => { g.soldiers[idx].grenades = 1; g.soldiers[idx].smoke_grenades = 2; g.soldiers[idx].medkits = 3; }
                _ => { g.soldiers[idx].grenades = 2; g.soldiers[idx].smoke_grenades = 3; g.soldiers[idx].medkits = 1; }
            }
        }
    }

    // Weapon choice row
    let water_base = g.bases.get(g.selected_base).map(|b| b.water).unwrap_or(false);
    let avail: Vec<usize> = g.available_weapons().into_iter().filter(|i| {
        let w = &g.data.weapons[*i];
        if water_base { w.env != Env::Surface } else { w.env != Env::Underwater }
    }).collect();

    let mut wx = d.x + 14.0;
    let mut wy = pry + 24.0;
    g.ui.label(d.x + 14.0, wy + 8.0, 8.5, DIM, "REARM WEAPON:");
    wy += 16.0;
    for &wi in avail.iter().take(8) {
        let wdef = &g.data.weapons[wi];
        let r = Rect::new(wx, wy, 84.0, 22.0);
        let sel = wi == s.weapon;
        let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(r.x, r.y, r.w, r.h, if sel { Color { r: 0.05, g: 0.20, b: 0.22, a: 1.0 } } else if hover { Color { a: 0.15, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, if sel { CYAN } else { Color { a: 0.3, ..CYAN_DIM } });
        g.ui.center(r.x + r.w * 0.5, r.y + 14.0, 7.5, if sel { CYAN } else { TEXT }, wdef.short);
        if hover && is_mouse_button_released(MouseButton::Left) {
            g.soldiers[idx].weapon = wi;
            g.soldiers[idx].ammo = g.data.clip_size(wdef);
            g.soldiers[idx].spare = 2;
            g.audio.play_equip();
        }
        wx += 88.0;
        if wx + 84.0 > d.x + d.w - 248.0 { wx = d.x + 14.0; wy += 26.0; }
    }

    let by = d.y + d.h - 76.0;
    if button(&g.ui, Rect::new(d.x + 10.0, by, d.w - 20.0, 30.0), "EDIT SOLDIER // CUSTOMIZE", Btn::Primary, true) {
        g.select_soldier_for_edit(g.selected_soldier);
        g.editor_return = Screen::Base;
        g.screen = Screen::Editor;
    }
    let half = (d.w - 26.0) * 0.5;
    if button(&g.ui, Rect::new(d.x + 10.0, by + 36.0, half, 26.0), if s.assigned { "STANDBY" } else { "ASSIGN" }, Btn::Normal, true) {
        g.soldiers[g.selected_soldier].assigned = !s.assigned;
    }
    if button(&g.ui, Rect::new(d.x + 16.0 + half, by + 36.0, half, 26.0), "AUTO-EQUIP", Btn::Normal, true) {
        let underwater = g.bases.get(g.selected_base).map(|b| b.water).unwrap_or(false);
        g.equip_default(g.selected_soldier, underwater);
    }
}

// ---------------------------------------------------------------- 1: BASE FACILITIES (6x6 GRID)
fn draw_facility_blueprint_tile(
    fr: Rect,
    ftype: FacilityType,
    water: bool,
    is_sel: bool,
    hover: bool,
    days_left: i32,
    build_days: i32,
    upkeep: i64,
    time: f32,
    ui: &Ui,
    fname: &str,
) {
    let (base_col, border_col) = match ftype {
        FacilityType::Entry => (Color::new(0.65, 0.40, 0.12, 1.0), AMBER),
        FacilityType::Barracks => (Color::new(0.15, 0.35, 0.65, 1.0), Color::new(0.35, 0.65, 0.95, 1.0)),
        FacilityType::Laboratory => (Color::new(0.10, 0.55, 0.68, 1.0), CYAN),
        FacilityType::Workshop => (Color::new(0.70, 0.40, 0.15, 1.0), Color::new(0.95, 0.65, 0.25, 1.0)),
        FacilityType::SearchSensor | FacilityType::LongRangeSensor => (Color::new(0.12, 0.60, 0.72, 1.0), Color::new(0.30, 0.85, 0.95, 1.0)),
        FacilityType::MissileBattery | FacilityType::AdaptedBattery | FacilityType::CapturedBattery | FacilityType::DerivedBattery => (Color::new(0.65, 0.18, 0.18, 1.0), ui::RED),
        FacilityType::SupplyBay => (Color::new(0.38, 0.38, 0.45, 1.0), Color::new(0.60, 0.60, 0.70, 1.0)),
        FacilityType::Holding => (Color::new(0.52, 0.18, 0.68, 1.0), Color::new(0.78, 0.35, 0.95, 1.0)),
        FacilityType::DefenceScreen | FacilityType::PsiDamper => (Color::new(0.38, 0.22, 0.72, 1.0), Color::new(0.65, 0.45, 0.95, 1.0)),
        FacilityType::PsiWing => (Color::new(0.28, 0.48, 0.75, 1.0), Color::new(0.50, 0.75, 0.98, 1.0)),
        FacilityType::SigintStation => (Color::new(0.18, 0.70, 0.58, 1.0), Color::new(0.40, 0.95, 0.80, 1.0)),
        FacilityType::Hangar => (Color::new(0.25, 0.42, 0.52, 1.0), Color::new(0.45, 0.70, 0.85, 1.0)),
    };

    let bg_a = if is_sel { 0.50 } else if hover { 0.38 } else { 0.24 };
    draw_rectangle(fr.x, fr.y, fr.w, fr.h, Color { a: bg_a, ..base_col });
    draw_rectangle_lines(fr.x, fr.y, fr.w, fr.h, if is_sel { 2.0 } else { 1.0 }, if is_sel { ui::WHITE } else { border_col });

    let pad = Rect::new(fr.x + 3.0, fr.y + 16.0, fr.w - 6.0, fr.h - 32.0);
    draw_rectangle(pad.x, pad.y, pad.w, pad.h, Color { r: 0.02, g: 0.05, b: 0.08, a: 0.75 });
    let cx = pad.x + pad.w * 0.5;
    let cy = pad.y + pad.h * 0.5;

    match ftype {
        FacilityType::Entry => {
            draw_line(pad.x, pad.y, pad.right(), pad.bottom(), 1.0, Color { a: 0.25, ..AMBER });
            draw_line(pad.x, pad.bottom(), pad.right(), pad.y, 1.0, Color { a: 0.25, ..AMBER });
            let ew = (pad.w * 0.45).min(26.0);
            let eh = (pad.h * 0.45).min(26.0);
            let er = Rect::new(cx - ew * 0.5, cy - eh * 0.5, ew, eh);
            draw_rectangle(er.x, er.y, er.w, er.h, Color { a: 0.8, ..PANEL_2 });
            draw_rectangle_lines(er.x, er.y, er.w, er.h, 1.0, AMBER);
            for i in 0..3 {
                let off = i as f32 * 6.0;
                draw_line(er.x + off, er.bottom(), er.x + off + 6.0, er.y, 1.0, Color { a: 0.6, ..BLACK });
            }
            draw_line(cx, pad.y, cx, er.y, 1.5, Color { a: 0.5, ..TEXT });
            let pulse = (time * 6.0).sin().abs();
            draw_circle(cx, cy, 3.0, Color { r: 1.0, g: 0.8 * pulse + 0.2, b: 0.1, a: 0.95 });
        }
        FacilityType::Barracks => {
            let hw = pad.w * 0.5;
            let hh = pad.h * 0.5;
            draw_line(cx, pad.y, cx, pad.bottom(), 1.0, Color { a: 0.4, ..CYAN_DIM });
            draw_line(pad.x, cy, pad.right(), cy, 1.0, Color { a: 0.4, ..CYAN_DIM });
            for (qx, qy) in [(-0.5f32, -0.5f32), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
                let bx = cx + qx * hw * 0.6 - 6.0;
                let by = cy + qy * hh * 0.6 - 4.0;
                draw_rectangle(bx, by, 12.0, 8.0, Color::new(0.20, 0.35, 0.50, 0.8));
                draw_rectangle(bx + 1.0, by + 1.0, 3.0, 6.0, Color::new(0.85, 0.85, 0.90, 0.85));
            }
        }
        FacilityType::Laboratory => {
            for i in 1..3 {
                let gx = pad.x + pad.w * (i as f32 / 3.0);
                draw_line(gx, pad.y, gx, pad.bottom(), 1.0, Color { a: 0.15, ..CYAN });
            }
            draw_circle_lines(cx, cy, 13.0, 1.0, Color { a: 0.5, ..CYAN });
            for i in 0..5 {
                let t = time * 2.5 + i as f32 * 0.9;
                let hx = cx + t.cos() * 9.0;
                let hy = cy + (i as f32 - 2.0) * 3.5;
                let hx2 = cx - t.cos() * 9.0;
                draw_circle(hx, hy, 1.8, CYAN);
                draw_circle(hx2, hy, 1.8, ui::GREEN);
                draw_line(hx, hy, hx2, hy, 1.0, Color { a: 0.35, ..CYAN });
            }
        }
        FacilityType::Workshop => {
            draw_rectangle(pad.x + 4.0, pad.bottom() - 3.0, pad.w - 8.0, 2.0, AMBER);
            let tw = (pad.w * 0.6).min(30.0);
            draw_rectangle(cx - tw * 0.5, cy, tw, 10.0, Color::new(0.35, 0.32, 0.28, 0.9));
            let ax0 = cx - 10.0; let ay0 = cy - 2.0;
            let ax1 = cx - 3.0;  let ay1 = cy - 12.0;
            let ax2 = cx + 2.0;  let ay2 = cy - 2.0;
            draw_line(ax0, ay0, ax1, ay1, 2.0, AMBER);
            draw_line(ax1, ay1, ax2, ay2, 1.5, AMBER);
            let spark = (time * 14.0).sin().abs() > 0.4;
            if spark {
                draw_circle(ax2, ay2, 3.0, Color::new(1.0, 0.95, 0.60, 0.95));
                draw_line(ax2, ay2, ax2 + 3.0, ay2 - 4.0, 1.0, ui::WHITE);
                draw_line(ax2, ay2, ax2 - 2.0, ay2 - 4.0, 1.0, AMBER);
            }
        }
        FacilityType::SearchSensor => {
            draw_circle_lines(cx, cy, 13.0, 1.0, Color { a: 0.4, ..CYAN });
            draw_circle(cx, cy, 3.5, border_col);
            if water {
                for r_ring in 0..3 {
                    let rad = ((time * 0.8 + r_ring as f32 * 0.33) % 1.0) * 14.0;
                    let a = (1.0 - rad / 14.0).max(0.0);
                    draw_circle_lines(cx, cy, rad, 1.0, Color { a, ..CYAN });
                }
            } else {
                let ang = time * 3.0;
                draw_line(cx, cy, cx + ang.cos() * 13.0, cy + ang.sin() * 13.0, 2.0, border_col);
            }
        }
        FacilityType::LongRangeSensor => {
            let qw = pad.w * 0.25;
            let qh = pad.h * 0.25;
            for (qx, qy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let px = cx + qx * qw;
                let py = cy + qy * qh;
                draw_circle_lines(px, py, 6.0, 1.0, Color { a: 0.6, ..CYAN });
                draw_circle(px, py, 2.0, border_col);
            }
            draw_circle(cx, cy, 3.0, AMBER);
        }
        FacilityType::MissileBattery => {
            let sw = 9.0; let sh = 16.0;
            for side in [-1.0f32, 1.0] {
                let mx = cx + side * 8.0;
                draw_rectangle(mx - sw * 0.5, cy - sh * 0.5, sw, sh, Color::new(0.25, 0.25, 0.28, 0.9));
                draw_rectangle_lines(mx - sw * 0.5, cy - sh * 0.5, sw, sh, 1.0, ui::RED);
                draw_rectangle(mx - 2.0, cy - sh * 0.5 + 4.0, 4.0, sh - 5.0, Color::new(0.8, 0.8, 0.85, 1.0));
                draw_triangle(vec2(mx - 2.5, cy - sh * 0.5 + 4.0), vec2(mx + 2.5, cy - sh * 0.5 + 4.0), vec2(mx, cy - sh * 0.5 - 2.0), ui::RED);
            }
        }
        FacilityType::SupplyBay => {
            for (gx, gy, w, h, col) in [
                (cx - 14.0, cy - 7.0, 10.0, 8.0, Color::new(0.60, 0.45, 0.25, 0.9)),
                (cx - 2.0, cy - 7.0, 12.0, 8.0, Color::new(0.55, 0.40, 0.22, 0.9)),
                (cx + 6.0, cy + 1.0, 8.0, 10.0, Color::new(0.25, 0.40, 0.25, 0.9)),
                (cx - 12.0, cy + 3.0, 14.0, 8.0, Color::new(0.40, 0.40, 0.45, 0.9)),
            ] {
                draw_rectangle(gx, gy, w, h, col);
                draw_rectangle_lines(gx, gy, w, h, 1.0, Color { a: 0.5, ..BLACK });
                draw_line(gx, gy, gx + w, gy + h, 1.0, Color { a: 0.3, ..BLACK });
            }
        }
        FacilityType::Holding => {
            for side in [-1.0f32, 1.0] {
                let tx = cx + side * 10.0;
                let tw = 10.0; let th = 18.0;
                let tr = Rect::new(tx - tw * 0.5, cy - th * 0.5, tw, th);
                draw_rectangle(tr.x, tr.y, tr.w, tr.h, Color::new(0.08, 0.35, 0.20, 0.75));
                draw_rectangle_lines(tr.x, tr.y, tr.w, tr.h, 1.0, Color::new(0.3, 0.9, 0.4, 0.8));
                let bob = (time * 2.0 + side).sin() * 1.5;
                draw_circle(tx, cy - 2.0 + bob, 2.5, Color::new(0.1, 0.15, 0.1, 0.9));
                draw_rectangle(tx - 1.5, cy + bob, 3.0, 5.0, Color::new(0.1, 0.15, 0.1, 0.9));
                let bub_y = tr.bottom() - ((time * 15.0 + side * 8.0) % tr.h);
                draw_circle(tx + 1.0, bub_y, 0.9, Color::new(0.7, 1.0, 0.7, 0.8));
            }
        }
        FacilityType::AdaptedBattery => {
            draw_circle_lines(cx, cy, 12.0, 2.0, Color::new(0.4, 0.4, 0.45, 1.0));
            draw_circle(cx, cy, 7.0, Color::new(0.25, 0.25, 0.30, 1.0));
            for side in [-1.0f32, 1.0] {
                let bx = cx + side * 3.5;
                draw_rectangle(bx - 1.5, cy - 15.0, 3.0, 12.0, Color::new(0.7, 0.7, 0.75, 1.0));
                draw_circle(bx, cy - 15.0, 1.8, ui::RED);
            }
        }
        FacilityType::CapturedBattery => {
            draw_circle(cx, cy, 10.0, Color::new(0.2, 0.25, 0.22, 1.0));
            draw_circle_lines(cx, cy, 10.0, 1.5, Color::new(0.3, 0.9, 0.4, 1.0));
            draw_rectangle(cx - 2.5, cy - 16.0, 5.0, 13.0, Color::new(0.25, 0.30, 0.25, 1.0));
            for r in 0..3 {
                let py = cy - 6.0 - r as f32 * 3.5;
                draw_line(cx - 3.5, py, cx + 3.5, py, 1.5, Color::new(0.2, 1.0, 0.4, 0.9));
            }
        }
        FacilityType::DerivedBattery => {
            draw_circle_lines(cx, cy, 12.0, 2.5, Color::new(0.7, 0.4, 0.1, 1.0));
            let f_pulse = (time * 8.0).sin().abs() * 0.3 + 0.7;
            draw_circle(cx, cy, 6.0, Color::new(1.0, 0.65 * f_pulse, 0.1, 0.95));
            for side in [-1.0f32, 1.0] {
                draw_rectangle(cx + side * 7.0 - 1.5, cy - 15.0, 3.0, 10.0, Color::new(0.6, 0.6, 0.65, 1.0));
            }
        }
        FacilityType::DefenceScreen => {
            for r_idx in 0..3 {
                let rad = ((time * 0.7 + r_idx as f32 * 0.33) % 1.0) * 14.0;
                let a = (1.0 - rad / 14.0).max(0.0);
                draw_circle_lines(cx, cy, rad, 1.5, Color { a, ..CYAN });
            }
            draw_circle(cx, cy, 4.5, CYAN);
        }
        FacilityType::PsiDamper => {
            let col = Color::new(0.8, 0.3, 0.95, 1.0);
            for r_idx in 0..3 {
                let rad = ((time * 0.6 + r_idx as f32 * 0.33) % 1.0) * 13.0;
                let a = (1.0 - rad / 13.0).max(0.0);
                draw_circle_lines(cx, cy, rad, 1.0, Color { a, ..col });
            }
            draw_triangle(vec2(cx, cy - 7.0), vec2(cx - 4.5, cy + 3.5), vec2(cx + 4.5, cy + 3.5), col);
        }
        FacilityType::PsiWing => {
            for side in [-1.0f32, 1.0] {
                let px = cx + side * 10.0;
                draw_rectangle(px - 4.0, cy - 9.0, 8.0, 18.0, Color::new(0.2, 0.35, 0.55, 0.9));
                draw_circle(px, cy - 2.0, 2.0, CYAN);
            }
            for step in 0..10 {
                let x0 = cx - 5.0 + step as f32;
                let y0 = cy + ((step as f32 * 0.8 + time * 6.0).sin() * 3.5);
                let x1 = x0 + 1.0;
                let y1 = cy + (((step + 1) as f32 * 0.8 + time * 6.0).sin() * 3.5);
                draw_line(x0, y0, x1, y1, 1.0, ui::GREEN);
            }
        }
        FacilityType::SigintStation => {
            draw_circle_lines(cx, cy, 13.0, 1.0, Color { a: 0.6, ..CYAN });
            draw_line(cx - 12.0, cy - 4.0, cx + 12.0, cy - 4.0, 1.0, Color { a: 0.3, ..CYAN });
            draw_line(cx - 13.0, cy, cx + 13.0, cy, 1.0, Color { a: 0.45, ..CYAN });
            draw_line(cx - 12.0, cy + 4.0, cx + 12.0, cy + 4.0, 1.0, Color { a: 0.3, ..CYAN });
            let rx = (time * 1.5).cos().abs() * 11.0;
            for ang_i in 0..12 {
                let a0 = ang_i as f32 * 0.52;
                let a1 = (ang_i + 1) as f32 * 0.52;
                draw_line(cx + a0.cos() * rx, cy + a0.sin() * 12.0, cx + a1.cos() * rx, cy + a1.sin() * 12.0, 1.0, ui::GREEN);
            }
        }
        FacilityType::Hangar => {
            for civ in 0..3 {
                let vy = pad.y + 10.0 + civ as f32 * 16.0;
                draw_line(cx - 16.0, vy + 6.0, cx, vy, 2.0, Color { a: 0.45, ..AMBER });
                draw_line(cx, vy, cx + 16.0, vy + 6.0, 2.0, Color { a: 0.45, ..AMBER });
            }
            let craft_col = if water { Color::new(0.2, 0.6, 0.7, 0.85) } else { Color::new(0.35, 0.55, 0.35, 0.85) };
            draw_rectangle(cx - 4.0, cy - 20.0, 8.0, 40.0, craft_col);
            draw_triangle(vec2(cx, cy - 6.0), vec2(cx - 26.0, cy + 12.0), vec2(cx, cy + 12.0), craft_col);
            draw_triangle(vec2(cx, cy - 6.0), vec2(cx + 26.0, cy + 12.0), vec2(cx, cy + 12.0), craft_col);
            draw_rectangle(cx - 13.0, cy + 4.0, 5.0, 12.0, Color::new(0.2, 0.2, 0.25, 0.9));
            draw_rectangle(cx + 8.0, cy + 4.0, 5.0, 12.0, Color::new(0.2, 0.2, 0.25, 0.9));
            draw_circle(cx - 10.5, cy + 16.0, 2.2, Color::new(1.0, 0.6, 0.2, 0.9));
            draw_circle(cx + 10.5, cy + 16.0, 2.2, Color::new(1.0, 0.6, 0.2, 0.9));
        }
    }

    // Header & status text
    ui.bold(fr.x + 5.0, fr.y + 13.0, 8.5, if is_sel { ui::WHITE } else { TEXT }, &ui.fit(8.5, fname, fr.w - 10.0, true));
    if days_left > 0 {
        ui.bold(fr.x + 5.0, fr.bottom() - 14.0, 7.5, AMBER, &format!("BUILD ({}d)", days_left));
        bar(Rect::new(fr.x + 5.0, fr.bottom() - 6.0, fr.w - 10.0, 3.5), (build_days - days_left) as f32, build_days as f32, AMBER, Color { a: 0.3, ..BLACK });
    } else {
        ui.text(fr.x + 5.0, fr.bottom() - 14.0, 7.5, ui::GREEN, "ONLINE");
        ui.text(fr.x + 5.0, fr.bottom() - 5.0, 7.0, DIM, &format!("-${}k/mo", upkeep / 1000));
    }
}

fn draw_facilities(g: &mut Game, c: Rect) {
    let grid_w = c.w * 0.56;
    let grid_panel = Rect::new(c.x + 10.0, c.y + 10.0, grid_w, c.h - 20.0);
    draw_rectangle(grid_panel.x, grid_panel.y, grid_panel.w, grid_panel.h, Color { a: 0.30, ..PANEL });

    let bsel = g.bases.get(g.selected_base).cloned();
    let Some(b) = bsel else { return; };

    g.ui.label(grid_panel.x + 14.0, grid_panel.y + 24.0, 11.0, CYAN, if b.water { "SEA-FLOOR STATION GRID (6 x 6)" } else { "BASE GRID (6 x 6)" });
    g.ui.text(grid_panel.x + 14.0, grid_panel.y + 38.0, 8.5, DIM, "CLICK VACANT CELL TO CONSTRUCT. CLICK FACILITY TO INSPECT OR DISMANTLE.");

    // Mode toggle button
    let mode_btn_r = Rect::new(grid_panel.right() - 176.0, grid_panel.y + 12.0, 166.0, 22.0);
    if button(&g.ui, mode_btn_r, if g.base_architecture_mode { "GRID: ARCHITECTURE" } else { "GRID: SCHEMATIC" }, Btn::Normal, true) {
        g.base_architecture_mode = !g.base_architecture_mode;
    }

    let cell_size = ((grid_panel.w - 48.0) / 6.0).min((grid_panel.h - 78.0) / 6.0).floor();
    let gx = grid_panel.x + 28.0;
    let gy = grid_panel.y + 64.0;

    // Architectural excavation border
    if g.base_architecture_mode {
        let pit_w = cell_size * 6.0 + 8.0;
        let pit_h = cell_size * 6.0 + 8.0;
        let pit_r = Rect::new(gx - 4.0, gy - 4.0, pit_w, pit_h);
        let border_soil = if b.water { Color::new(0.06, 0.16, 0.24, 0.95) } else { Color::new(0.24, 0.18, 0.12, 0.95) };
        draw_rectangle_lines(pit_r.x - 2.0, pit_r.y - 2.0, pit_r.w + 4.0, pit_r.h + 4.0, 3.0, border_soil);
        draw_rectangle_lines(pit_r.x, pit_r.y, pit_r.w, pit_r.h, 1.0, Color { a: 0.35, ..CYAN_DIM });

        // Coordinate headers: A..F across top, 1..6 down left
        for col in 0..6 {
            let col_name = format!("{}", (b'A' + col as u8) as char);
            g.ui.center(gx + col as f32 * cell_size + cell_size * 0.5, gy - 8.0, 9.0, CYAN_DIM, &col_name);
        }
        for row in 0..6 {
            let row_name = format!("{}", row + 1);
            g.ui.center(gx - 14.0, gy + row as f32 * cell_size + cell_size * 0.5 + 4.0, 9.0, CYAN_DIM, &row_name);
        }

        // Draw structural power and transport conduits between adjacent facilities
        for row in 0..6 {
            for col in 0..6 {
                if b.grid[row][col].is_some() {
                    let cx0 = gx + col as f32 * cell_size + cell_size * 0.5;
                    let cy0 = gy + row as f32 * cell_size + cell_size * 0.5;
                    if col + 1 < 6 && b.grid[row][col + 1].is_some() {
                        let cx1 = gx + (col + 1) as f32 * cell_size + cell_size * 0.5;
                        draw_line(cx0, cy0, cx1, cy0, 3.0, Color::new(0.8, 0.65, 0.2, 0.55));
                    }
                    if row + 1 < 6 && b.grid[row + 1][col].is_some() {
                        let cy1 = gy + (row + 1) as f32 * cell_size + cell_size * 0.5;
                        draw_line(cx0, cy0, cx0, cy1, 3.0, Color::new(0.8, 0.65, 0.2, 0.55));
                    }
                }
            }
        }
    }

    // Draw 6x6 cells
    let mut rendered_facs = Vec::new();
    for row in 0..6 {
        for col in 0..6 {
            let cx = gx + col as f32 * cell_size;
            let cy = gy + row as f32 * cell_size;
            let cr = Rect::new(cx, cy, cell_size - 2.0, cell_size - 2.0);

            let cell_fac = b.grid[row][col];
            if let Some(f_idx) = cell_fac {
                if !rendered_facs.contains(&f_idx) {
                    rendered_facs.push(f_idx);
                    let fac = &b.facilities[f_idx];
                    let fdef = facility_def(fac.ftype);
                    let span_w = cell_size * fdef.size as f32 - 2.0;
                    let span_h = cell_size * fdef.size as f32 - 2.0;
                    let fr = Rect::new(cx, cy, span_w, span_h);
                    let hover = fr.contains(vec2(mouse_position().0, mouse_position().1));
                    let is_sel = g.selected_facility_cell == Some((fac.x, fac.y));
                    let fname = if b.water { fdef.name_water } else { fdef.name_surface };

                    if g.base_architecture_mode {
                        draw_facility_blueprint_tile(fr, fac.ftype, b.water, is_sel, hover, fac.days_left, fdef.build_days, fdef.upkeep, g.elapsed, &g.ui, fname);
                    } else {
                        let theme_col = match fac.ftype {
                            FacilityType::Entry => AMBER,
                            FacilityType::Barracks => Color::new(0.2, 0.4, 0.7, 1.0),
                            FacilityType::Laboratory => CYAN,
                            FacilityType::Workshop => Color::new(0.8, 0.5, 0.2, 1.0),
                            FacilityType::SearchSensor | FacilityType::LongRangeSensor => Color::new(0.1, 0.7, 0.8, 1.0),
                            FacilityType::MissileBattery | FacilityType::AdaptedBattery | FacilityType::CapturedBattery | FacilityType::DerivedBattery => ui::RED,
                            FacilityType::SupplyBay => Color::new(0.4, 0.4, 0.5, 1.0),
                            FacilityType::Holding => Color::new(0.6, 0.2, 0.7, 1.0),
                            FacilityType::Hangar => Color::new(0.3, 0.5, 0.6, 1.0),
                            _ => CYAN_DIM,
                        };
                        draw_rectangle(fr.x, fr.y, fr.w, fr.h, Color { a: if is_sel { 0.45 } else if hover { 0.35 } else { 0.22 }, ..theme_col });
                        draw_rectangle_lines(fr.x, fr.y, fr.w, fr.h, if is_sel { 2.0 } else { 1.0 }, if is_sel { ui::WHITE } else { theme_col });
                        g.ui.bold(fr.x + 6.0, fr.y + 16.0, 9.0, if is_sel { ui::WHITE } else { TEXT }, &g.ui.fit(9.0, fname, fr.w - 12.0, true));
                        if fac.days_left > 0 {
                            g.ui.bold(fr.x + 6.0, fr.y + 32.0, 8.5, AMBER, &format!("BUILD ({}d)", fac.days_left));
                            bar(Rect::new(fr.x + 6.0, fr.y + fr.h - 10.0, fr.w - 12.0, 4.0), (fdef.build_days - fac.days_left) as f32, fdef.build_days as f32, AMBER, Color { a: 0.3, ..BLACK });
                        } else {
                            g.ui.text(fr.x + 6.0, fr.y + 32.0, 8.0, ui::GREEN, "ONLINE");
                            g.ui.text(fr.x + 6.0, fr.y + fr.h - 8.0, 7.5, DIM, &format!("-${}k/mo", fdef.upkeep / 1000));
                        }
                    }

                    if hover && is_mouse_button_released(MouseButton::Left) {
                        g.selected_facility_cell = Some((fac.x, fac.y));
                        g.building_catalog_open = false;
                    }
                }
            } else {
                let hover = cr.contains(vec2(mouse_position().0, mouse_position().1));
                let is_sel = g.selected_facility_cell == Some((col, row));
                draw_rectangle(cr.x, cr.y, cr.w, cr.h, if is_sel { Color { a: 0.25, ..CYAN } } else if hover { Color { a: 0.12, ..PANEL_2 } } else { Color { a: 0.05, ..PANEL } });
                draw_rectangle_lines(cr.x, cr.y, cr.w, cr.h, 1.0, if is_sel { CYAN } else if hover { Color { a: 0.4, ..CYAN_DIM } } else { Color { a: 0.15, ..CYAN_DIM } });

                if g.base_architecture_mode {
                    // Blueprint grid corner ticks and coordinate label
                    let coord_str = format!("{}{}", (b'A' + col as u8) as char, row + 1);
                    g.ui.text(cr.x + 4.0, cr.y + 12.0, 7.5, Color { a: 0.35, ..CYAN_DIM }, &coord_str);
                    g.ui.center(cr.x + cr.w * 0.5, cr.y + cr.h * 0.5 + 4.0, 9.0, if hover { CYAN } else { Color { a: 0.35, ..DIM } }, if hover { "+ BUILD" } else { "[VACANT]" });
                } else {
                    g.ui.center(cr.x + cr.w * 0.5, cr.y + cr.h * 0.5 + 4.0, 10.0, if hover { CYAN } else { Color { a: 0.3, ..DIM } }, "+ VACANT");
                }

                if hover && is_mouse_button_released(MouseButton::Left) {
                    g.selected_facility_cell = Some((col, row));
                    g.building_catalog_open = true;
                }
            }
        }
    }

    // Right side: Construction Catalog or Facility Inspector
    let info_w = c.w - grid_w - 30.0;
    let info_panel = Rect::new(grid_panel.right() + 10.0, c.y + 10.0, info_w, c.h - 20.0);
    draw_rectangle(info_panel.x, info_panel.y, info_panel.w, info_panel.h, Color { a: 0.30, ..PANEL });

    if g.building_catalog_open {
        if let Some((cx, cy)) = g.selected_facility_cell {
            g.ui.label(info_panel.x + 12.0, info_panel.y + 24.0, 11.0, CYAN, &format!("CONSTRUCTION CATALOG // CELL ({}, {})", cx + 1, cy + 1));
            let mut fy = info_panel.y + 44.0;

            for fdef in &FACILITY_CATALOG {
                if fdef.ftype == FacilityType::Entry { continue; }
                let name = if b.water { fdef.name_water } else { fdef.name_surface };
                let r = Rect::new(info_panel.x + 10.0, fy, info_panel.w - 20.0, 36.0);
                let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
                draw_rectangle(r.x, r.y, r.w, r.h, if hover { Color { a: 0.18, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, Color { a: 0.3, ..CYAN_DIM });

                g.ui.bold(r.x + 8.0, r.y + 15.0, 9.5, TEXT, name);
                g.ui.text(r.x + 8.0, r.y + 27.0, 8.0, DIM, &format!("${}  //  {}d  //  {}x{}", crate::fmt_money(fdef.cost), fdef.build_days, fdef.size, fdef.size));

                let can_afford = g.funds >= fdef.cost;
                if button(&g.ui, Rect::new(r.x + r.w - 68.0, r.y + 6.0, 60.0, 24.0), "BUILD", Btn::Normal, can_afford) {
                    if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                        match base_mut.can_build(fdef.ftype, cx, cy) {
                            Ok(()) => {
                                g.funds -= fdef.cost;
                                base_mut.start_build(fdef.ftype, cx, cy);
                                g.toast(format!("CONSTRUCTION STARTED: {}", name));
                                g.building_catalog_open = false;
                            }
                            Err(e) => { g.toast(e); }
                        }
                    }
                }
                fy += 40.0;
                if fy > info_panel.y + info_panel.h - 50.0 { break; }
            }
        }
    } else if let Some((cx, cy)) = g.selected_facility_cell {
        if let Some(f_idx) = b.grid[cy][cx] {
            let fac = &b.facilities[f_idx];
            let fdef = facility_def(fac.ftype);
            let fname = if b.water { fdef.name_water } else { fdef.name_surface };

            g.ui.label(info_panel.x + 12.0, info_panel.y + 24.0, 11.0, CYAN, "FACILITY INTELLIGENCE");
            g.ui.bold(info_panel.x + 12.0, info_panel.y + 50.0, 16.0, TEXT, fname);
            let status_str = if fac.days_left > 0 { format!("UNDER CONSTRUCTION: {} DAYS REMAINING", fac.days_left) } else { "OPERATIONAL STATUS: ONLINE".to_string() };
            g.ui.text(info_panel.x + 12.0, info_panel.y + 70.0, 10.0, if fac.days_left > 0 { AMBER } else { ui::GREEN }, &status_str);

            let mut dy = info_panel.y + 100.0;
            for line in wrap(&g.ui, fdef.desc, 9.5, info_panel.w - 24.0) {
                g.ui.text(info_panel.x + 12.0, dy, 9.5, TEXT, &line);
                dy += 16.0;
            }

            dy += 12.0;
            let details = [
                ("DIMENSIONS", format!("{} x {} TILES", fdef.size, fdef.size)),
                ("CONSTRUCTION COST", format!("${}", crate::fmt_money(fdef.cost))),
                ("MONTHLY UPKEEP", format!("${}/MONTH", crate::fmt_money(fdef.upkeep))),
                ("LIVING CAPACITY", format!("+{}", fdef.living_cap)),
                ("LAB CAPACITY", format!("+{}", fdef.lab_cap)),
                ("WORKSHOP CAPACITY", format!("+{}", fdef.workshop_cap)),
                ("RADAR DETECTION", format!("{} NM", fdef.radar_range)),
                ("BASE DEFENSE POWER", format!("{} PTS", fdef.defense_val)),
            ];

            for (label, val) in details {
                g.ui.text(info_panel.x + 12.0, dy, 9.0, DIM, label);
                g.ui.bold(info_panel.x + info_panel.w - g.ui.bwidth(9.0, &val) - 16.0, dy, 9.0, TEXT, &val);
                dy += 18.0;
            }

            if fac.ftype == FacilityType::Holding {
                dy += 8.0;
                let cap = b.holding_cap();
                let held = b.captured_aliens.len();
                g.ui.bold(info_panel.x + 12.0, dy, 10.0, AMBER, &format!("HOLDING: {} / {} CAPTIVES", held, cap));
                dy += 16.0;
                for alien in b.captured_aliens.iter().take(5) {
                    let status_str = if alien.studied { "STUDY COMPLETE // NECROPSY FILED" } else { "HELD LIVE // AWAITING STUDY" };
                    g.ui.text(info_panel.x + 16.0, dy, 8.5, if alien.studied { DIM } else { CYAN }, &format!("• {} // {}", alien.name, status_str));
                    dy += 15.0;
                }
                if b.captured_aliens.is_empty() {
                    g.ui.text(info_panel.x + 16.0, dy, 8.5, DIM, "Holding is empty. Stun a hostile in the field and bring it back alive.");
                }
            }

            if fac.ftype != FacilityType::Entry {
                if button(&g.ui, Rect::new(info_panel.x + 14.0, info_panel.y + info_panel.h - 50.0, info_panel.w - 28.0, 34.0), "DISMANTLE FACILITY", Btn::Danger, true) {
                    if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                        if let Ok(()) = base_mut.dismantle(f_idx) {
                            g.toast("FACILITY DISMANTLED");
                            g.selected_facility_cell = None;
                        }
                    }
                }
            }
        }
    } else {
        g.ui.label(info_panel.x + 12.0, info_panel.y + 24.0, 11.0, CYAN, "COMMAND ADVISORY");
        g.ui.text(info_panel.x + 12.0, info_panel.y + 48.0, 10.0, DIM, "Select an empty tile on the 6x6 grid to build a new installation, or select a facility to view status or dismantle.");
    }
}

// ---------------------------------------------------------------- 2: RESEARCH & DEVELOPMENT
fn draw_research(g: &mut Game, c: Rect) {
    let left_w = c.w * 0.55;
    let left = Rect::new(c.x + 10.0, c.y + 10.0, left_w, c.h - 20.0);
    draw_rectangle(left.x, left.y, left.w, left.h, Color { a: 0.30, ..PANEL });
    g.ui.label(left.x + 14.0, left.y + 24.0, 11.0, CYAN, "RESEARCH & DEVELOPMENT // TECHNOLOGIES");

    let bsel = g.bases.get(g.selected_base).cloned();
    let Some(b) = bsel else { return; };

    // List available topics based on game mode & prereqs
    let topics = if b.water { &RESEARCH_WATER[..] } else { &RESEARCH_SURFACE[..] };
    let available: Vec<&ResearchTopic> = topics.iter().filter(|t| {
        if g.researched_techs.contains(&t.id.to_string()) { return false; }
        if !t.prereqs.iter().all(|p| g.researched_techs.contains(&p.to_string())) { return false; }
        if t.category == "Live Study" {
            let species_match = t.id.trim_start_matches("STUDY_").replace('_', " ");
            b.captured_aliens.iter().any(|a| a.name.to_uppercase().contains(&species_match))
        } else {
            true
        }
    }).collect();

    let mut y = left.y + 46.0;
    for (i, t) in available.iter().enumerate().take(8) {
        let r = Rect::new(left.x + 10.0, y, left.w - 20.0, 44.0);
        let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
        let sel = g.selected_tech_topic == Some(i);
        draw_rectangle(r.x, r.y, r.w, r.h, if sel { Color { r: 0.06, g: 0.18, b: 0.22, a: 1.0 } } else if hover { Color { a: 0.15, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, if sel { CYAN } else { Color { a: 0.3, ..CYAN_DIM } });

        g.ui.bold(r.x + 10.0, r.y + 16.0, 10.5, if sel { CYAN } else { TEXT }, t.name);
        g.ui.label(r.x + 10.0, r.y + 30.0, 8.5, AMBER, t.category);
        g.ui.text(r.x + 90.0, r.y + 30.0, 8.5, DIM, &format!("{} SCIENTIST-DAYS  //  UNLOCKS: {}", t.points, t.unlocks));

        if button(&g.ui, Rect::new(r.x + r.w - 80.0, r.y + 10.0, 72.0, 24.0), "ASSIGN", Btn::Normal, b.scientists > 0 && b.active_research.is_none()) {
            if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                base_mut.active_research = Some(ActiveResearch {
                    tech_id: t.id.to_string(),
                    scientists: base_mut.scientists.min(10),
                    progress: 0,
                    total_points: t.points * 24,
                });
                g.log(format!("RESEARCH PROJECT INITIATED: {}", t.name), CYAN);
            }
        }
        if hover && is_mouse_button_released(MouseButton::Left) { g.selected_tech_topic = Some(i); }
        y += 48.0;
    }

    // Right side: Active Project or Tech Archives
    let right_w = c.w - left_w - 30.0;
    let right = Rect::new(left.right() + 10.0, c.y + 10.0, right_w, c.h - 20.0);
    draw_rectangle(right.x, right.y, right.w, right.h, Color { a: 0.30, ..PANEL });

    if let Some(active) = &b.active_research {
        let topic_def = topics.iter().find(|t| t.id == active.tech_id);
        g.ui.label(right.x + 14.0, right.y + 24.0, 11.0, CYAN, "ACTIVE RESEARCH PROJECT");
        g.ui.bold(right.x + 14.0, right.y + 50.0, 16.0, TEXT, topic_def.map(|t| t.name).unwrap_or(&active.tech_id));

        let progress_pct = (active.progress as f32 / active.total_points as f32).clamp(0.0, 1.0);
        g.ui.text(right.x + 14.0, right.y + 76.0, 10.0, DIM, &format!("PROGRESS: {} / {} PTS ({}%)", active.progress, active.total_points, (progress_pct * 100.0) as i32));
        bar(Rect::new(right.x + 14.0, right.y + 84.0, right.w - 28.0, 8.0), active.progress as f32, active.total_points as f32, CYAN, Color { a: 0.3, ..BLACK });

        let days_left = if active.scientists > 0 { (active.total_points - active.progress) / (active.scientists * 24) } else { 999 };
        g.ui.bold(right.x + 14.0, right.y + 114.0, 11.0, AMBER, &format!("ESTIMATED COMPLETION: {} DAYS", days_left));

        // Scientist Allocation
        g.ui.text(right.x + 14.0, right.y + 144.0, 10.0, TEXT, "SCIENTISTS ASSIGNED:");
        let sc = active.scientists;
        if button(&g.ui, Rect::new(right.x + 170.0, right.y + 130.0, 26.0, 24.0), "-", Btn::Ghost, sc > 1) {
            if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                if let Some(res) = &mut base_mut.active_research { res.scientists = res.scientists.saturating_sub(1); }
            }
        }
        g.ui.bold(right.x + 208.0, right.y + 146.0, 12.0, CYAN, &format!("{}", sc));
        if button(&g.ui, Rect::new(right.x + 230.0, right.y + 130.0, 26.0, 24.0), "+", Btn::Ghost, sc < b.scientists) {
            if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                if let Some(res) = &mut base_mut.active_research { res.scientists = (res.scientists + 1).min(base_mut.scientists); }
            }
        }

        if let Some(td) = topic_def {
            let mut ty = right.y + 180.0;
            g.ui.label(right.x + 14.0, ty, 9.5, DIM, "BREAKTHROUGH PROFILE");
            ty += 16.0;
            for line in wrap(&g.ui, td.description, 9.5, right.w - 28.0) {
                g.ui.text(right.x + 14.0, ty, 9.5, TEXT, &line);
                ty += 16.0;
            }
            ty += 8.0;
            g.ui.bold(right.x + 14.0, ty, 10.0, ui::GREEN, &format!("UNLOCKS: {}", td.unlocks));
        }

        if button(&g.ui, Rect::new(right.x + 14.0, right.y + right.h - 50.0, right.w - 28.0, 32.0), "ABORT RESEARCH", Btn::Danger, true) {
            if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                base_mut.active_research = None;
                g.toast("RESEARCH ABORTED");
            }
        }
    } else {
        g.ui.label(right.x + 14.0, right.y + 24.0, 11.0, CYAN, "COMPLETED TECHNOLOGIES");
        let mut y = right.y + 50.0;
        if g.researched_techs.is_empty() {
            g.ui.text(right.x + 14.0, y, 10.0, DIM, "No technologies researched yet. Assign scientists to begin.");
        }
        for tid in &g.researched_techs {
            draw_circle(right.x + 18.0, y - 4.0, 3.5, ui::GREEN);
            g.ui.bold(right.x + 28.0, y, 10.0, TEXT, tid);
            y += 20.0;
            if y > right.y + right.h - 40.0 { break; }
        }
    }
}

// ---------------------------------------------------------------- 3: WORKSHOP & MANUFACTURING
fn draw_workshop(g: &mut Game, c: Rect) {
    let left_w = c.w * 0.55;
    let left = Rect::new(c.x + 10.0, c.y + 10.0, left_w, c.h - 20.0);
    draw_rectangle(left.x, left.y, left.w, left.h, Color { a: 0.30, ..PANEL });
    g.ui.label(left.x + 14.0, left.y + 24.0, 11.0, CYAN, "ENGINEERING WORKSHOP // PRODUCTION BLUEPRINTS");

    let bsel = g.bases.get(g.selected_base).cloned();
    let Some(b) = bsel else { return; };

    let blueprints: Vec<&ProductionBlueprint> = WORKSHOP_BLUEPRINTS.iter().filter(|bp| {
        (b.water == (bp.game == 1)) &&
        (bp.tech_req.is_empty() || g.researched_techs.contains(&bp.tech_req.to_string()))
    }).collect();

    let mut y = left.y + 46.0;
    if blueprints.is_empty() {
        g.ui.text(left.x + 14.0, y + 12.0, 10.0, DIM, "No blueprints yet. Finished research adds production orders here.");
    }
    for (i, bp) in blueprints.iter().enumerate().take(8) {
        let r = Rect::new(left.x + 10.0, y, left.w - 20.0, 44.0);
        let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
        let sel = g.selected_blueprint == Some(i);
        draw_rectangle(r.x, r.y, r.w, r.h, if sel { Color { r: 0.06, g: 0.18, b: 0.22, a: 1.0 } } else if hover { Color { a: 0.15, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, if sel { CYAN } else { Color { a: 0.3, ..CYAN_DIM } });

        g.ui.bold(r.x + 10.0, r.y + 16.0, 10.5, if sel { CYAN } else { TEXT }, bp.name);
        let mat_str = if bp.game == 1 { format!("${} // {} NACRE // {} BRINE", crate::fmt_money(bp.cost), bp.structure, bp.power) }
                      else { format!("${} // {} SPINE // {} GLOW", crate::fmt_money(bp.cost), bp.structure, bp.power) };
        g.ui.text(r.x + 10.0, r.y + 30.0, 8.5, DIM, &format!("{} ENG-HOURS  //  {}", bp.engineer_hours, mat_str));

        let can_build = g.funds >= bp.cost &&
            (if bp.game == 1 { b.nacre >= bp.structure && b.brine >= bp.power }
             else { b.lattice >= bp.structure && b.electrolyte >= bp.power }) &&
            b.engineers > 0 && b.active_production.is_none();

        if button(&g.ui, Rect::new(r.x + r.w - 80.0, r.y + 10.0, 72.0, 24.0), "BUILD", Btn::Normal, can_build) {
            if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                g.funds -= bp.cost;
                if bp.game == 1 { base_mut.nacre -= bp.structure; base_mut.brine -= bp.power; }
                else { base_mut.lattice -= bp.structure; base_mut.electrolyte -= bp.power; }
                base_mut.active_production = Some(ActiveProduction {
                    blueprint_id: bp.id.to_string(),
                    engineers: base_mut.engineers.min(10),
                    progress: 0,
                    total_hours: bp.engineer_hours,
                });
                g.log(format!("MANUFACTURING ORDER INITIATED: {}", bp.name), ui::GREEN);
            }
        }
        if hover && is_mouse_button_released(MouseButton::Left) { g.selected_blueprint = Some(i); }
        y += 48.0;
    }

    // Right side: Active Production & Materials Vault
    let right_w = c.w - left_w - 30.0;
    let right = Rect::new(left.right() + 10.0, c.y + 10.0, right_w, c.h - 20.0);
    draw_rectangle(right.x, right.y, right.w, right.h, Color { a: 0.30, ..PANEL });

    if let Some(active) = &b.active_production {
        let bp_def = WORKSHOP_BLUEPRINTS.iter().find(|bp| bp.id == active.blueprint_id);
        g.ui.label(right.x + 14.0, right.y + 24.0, 11.0, CYAN, "ACTIVE PRODUCTION ORDER");
        g.ui.bold(right.x + 14.0, right.y + 50.0, 16.0, TEXT, bp_def.map(|bp| bp.name).unwrap_or(&active.blueprint_id));

        let progress_pct = (active.progress as f32 / active.total_hours as f32).clamp(0.0, 1.0);
        g.ui.text(right.x + 14.0, right.y + 76.0, 10.0, DIM, &format!("PROGRESS: {} / {} HRS ({}%)", active.progress, active.total_hours, (progress_pct * 100.0) as i32));
        bar(Rect::new(right.x + 14.0, right.y + 84.0, right.w - 28.0, 8.0), active.progress as f32, active.total_hours as f32, ui::GREEN, Color { a: 0.3, ..BLACK });

        let days_left = if active.engineers > 0 { (active.total_hours - active.progress) / (active.engineers * 24) } else { 999 };
        g.ui.bold(right.x + 14.0, right.y + 114.0, 11.0, AMBER, &format!("ESTIMATED COMPLETION: {} DAYS", days_left));

        // Engineer Allocation
        g.ui.text(right.x + 14.0, right.y + 144.0, 10.0, TEXT, "ENGINEERS ALLOCATED:");
        let ec = active.engineers;
        if button(&g.ui, Rect::new(right.x + 170.0, right.y + 130.0, 26.0, 24.0), "-", Btn::Ghost, ec > 1) {
            if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                if let Some(prod) = &mut base_mut.active_production { prod.engineers = prod.engineers.saturating_sub(1); }
            }
        }
        g.ui.bold(right.x + 208.0, right.y + 146.0, 12.0, CYAN, &format!("{}", ec));
        if button(&g.ui, Rect::new(right.x + 230.0, right.y + 130.0, 26.0, 24.0), "+", Btn::Ghost, ec < b.engineers) {
            if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                if let Some(prod) = &mut base_mut.active_production { prod.engineers = (prod.engineers + 1).min(base_mut.engineers); }
            }
        }

        if button(&g.ui, Rect::new(right.x + 14.0, right.y + 190.0, right.w - 28.0, 32.0), "HALT PRODUCTION", Btn::Danger, true) {
            if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                base_mut.active_production = None;
                g.toast("PRODUCTION HALTED");
            }
        }
    }

    // Material Vault inventory
    let vy = right.y + right.h - 180.0;
    draw_rectangle(right.x + 14.0, vy - 10.0, right.w - 28.0, 1.0, Color { a: 0.35, ..CYAN_DIM });
    g.ui.label(right.x + 14.0, vy + 4.0, 10.5, CYAN, "BASE STORES VAULT // MATERIALS");

    let mats = if b.water {
        [("NACRE PLATE", b.nacre), ("LIVE BRINE", b.brine), ("SPINE (LAND)", b.lattice), ("GLOW (LAND)", b.electrolyte)]
    } else {
        [("SPINE", b.lattice), ("GLOW", b.electrolyte), ("NACRE (SEA)", b.nacre), ("BRINE (SEA)", b.brine)]
    };

    let mut my = vy + 24.0;
    for (mname, count) in mats {
        g.ui.text(right.x + 14.0, my, 9.5, DIM, mname);
        g.ui.bold(right.x + right.w - g.ui.bwidth(9.5, &format!("{} UNITS", count)) - 16.0, my, 9.5, TEXT, &format!("{} UNITS", count));
        my += 22.0;
    }
}

// ---------------------------------------------------------------- 4: MARKET (PROCUREMENT & SALVAGE)
fn draw_market(g: &mut Game, c: Rect) {
    let left_w = c.w * 0.52;
    let left = Rect::new(c.x + 10.0, c.y + 10.0, left_w, c.h - 20.0);
    draw_rectangle(left.x, left.y, left.w, left.h, Color { a: 0.30, ..PANEL });
    g.ui.label(left.x + 14.0, left.y + 24.0, 11.0, CYAN, "MARKET PROCUREMENT // REQUISITIONS & RECRUITS");

    let bsel = g.bases.get(g.selected_base).cloned();
    let Some(b) = bsel else { return; };

    let living_used = g.soldiers.len() as i32 + b.scientists + b.engineers;
    let living_cap = b.living_cap();
    let craft_cap = b.craft_cap();

    // Requisitions catalogue
    struct BuyOption {
        name: &'static str,
        category: &'static str,
        cost: i64,
        is_person: u8, // 1: soldier, 2: sci, 3: eng, 4: craft, 0: item
        craft_type: Option<CraftType>,
        water: Option<bool>,
    }

    let catalog = [
        BuyOption { name: "RECRUIT SOLDIER", category: "PERSONNEL", cost: 40_000, is_person: 1, craft_type: None, water: None },
        BuyOption { name: "HIRE SCIENTIST", category: "PERSONNEL", cost: 60_000, is_person: 2, craft_type: None, water: None },
        BuyOption { name: "HIRE ENGINEER", category: "PERSONNEL", cost: 50_000, is_person: 3, craft_type: None, water: None },
        BuyOption { name: "SERVICE RIFLE", category: "SURFACE ARMS", cost: 2_600, is_person: 0, craft_type: None, water: Some(false) },
        BuyOption { name: "5.56 MAGAZINE", category: "AMMO", cost: 300, is_person: 0, craft_type: None, water: Some(false) },
        BuyOption { name: "HEAVY RIFLE", category: "SURFACE ARMS", cost: 7_200, is_person: 0, craft_type: None, water: Some(false) },
        BuyOption { name: "DIVE RIFLE", category: "DIVE ARMS", cost: 2_900, is_person: 0, craft_type: None, water: Some(true) },
        BuyOption { name: "RIFLE BOLTS", category: "AMMO", cost: 340, is_person: 0, craft_type: None, water: Some(true) },
        BuyOption { name: "HEAVY DIVE RIFLE", category: "DIVE ARMS", cost: 7_600, is_person: 0, craft_type: None, water: Some(true) },
        BuyOption { name: "FRAG GRENADE", category: "TACTICAL", cost: 250, is_person: 0, craft_type: None, water: Some(false) },
        BuyOption { name: "DIVE CHARGE", category: "TACTICAL", cost: 280, is_person: 0, craft_type: None, water: Some(true) },
        BuyOption { name: "TRAUMA KIT", category: "TACTICAL", cost: 800, is_person: 0, craft_type: None, water: None },
        BuyOption { name: "HANDHELD RADAR", category: "TACTICAL", cost: 1_500, is_person: 0, craft_type: None, water: Some(false) },
        BuyOption { name: "HANDHELD SONAR", category: "TACTICAL", cost: 1_500, is_person: 0, craft_type: None, water: Some(true) },
        BuyOption { name: "SWIFT INTERCEPTOR", category: "CRAFT", cost: 650_000, is_person: 4, craft_type: Some(CraftType::Swift), water: Some(false) },
        BuyOption { name: "HERON TRANSPORT", category: "CRAFT", cost: 550_000, is_person: 4, craft_type: Some(CraftType::Heron), water: Some(false) },
        BuyOption { name: "SKUA ATTACK SUB", category: "CRAFT", cost: 650_000, is_person: 4, craft_type: Some(CraftType::Skua), water: Some(true) },
        BuyOption { name: "GANNET TRANSPORT SUB", category: "CRAFT", cost: 550_000, is_person: 4, craft_type: Some(CraftType::Gannet), water: Some(true) },
    ];
    let filtered_catalog: Vec<&BuyOption> = catalog.iter().filter(|opt| opt.water.is_none() || opt.water == Some(b.water)).collect();

    let mut y = left.y + 44.0;
    let buy_rows = (((left.h - 54.0) / 40.0) as usize).max(10);
    for (_i, opt) in filtered_catalog.iter().enumerate().take(buy_rows) {
        let r = Rect::new(left.x + 10.0, y, left.w - 20.0, 36.0);
        let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(r.x, r.y, r.w, r.h, if hover { Color { a: 0.15, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, Color { a: 0.25, ..CYAN_DIM });

        g.ui.bold(r.x + 10.0, r.y + 14.0, 9.5, TEXT, opt.name);
        g.ui.text(r.x + 10.0, r.y + 26.0, 8.0, DIM, &format!("{}  //  COST: ${}", opt.category, crate::fmt_money(opt.cost)));

        let can_afford = g.funds >= opt.cost;
        let can_living = opt.is_person == 0 || opt.is_person == 4 || living_used < living_cap;
        let can_craft = opt.is_person != 4 || b.craft.len() < craft_cap as usize;
        let eligible = can_afford && can_living && can_craft;

        if button(&g.ui, Rect::new(r.x + r.w - 74.0, r.y + 6.0, 66.0, 24.0), "BUY", Btn::Normal, eligible) {
            g.audio.play_click();
            g.funds -= opt.cost;
            match opt.is_person {
                1 => {
                    let mut s = g.make_soldier();
                    s.assigned = true;
                    g.soldiers.push(s);
                    g.toast("SOLDIER ENLISTED");
                    g.log("NEW SOLDIER COMMISSIONED TO BARRACKS.", ui::GREEN);
                }
                2 => {
                    if let Some(base_mut) = g.bases.get_mut(g.selected_base) { base_mut.scientists += 1; }
                    g.toast("SCIENTIST HIRED");
                }
                3 => {
                    if let Some(base_mut) = g.bases.get_mut(g.selected_base) { base_mut.engineers += 1; }
                    g.toast("ENGINEER HIRED");
                }
                4 => {
                    if let Some(ctype) = opt.craft_type {
                        let cdef = craft_def(ctype);
                        let cname = format!("{}-{}", cdef.name, g.next_id());
                        if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                            base_mut.craft.push(CraftInstance {
                                name: cname.clone(), ctype, ready: true, hp: cdef.max_hp, max_hp: cdef.max_hp,
                                fuel: cdef.max_fuel, max_fuel: cdef.max_fuel, weapons: [None, None], ammo: [0, 0],
                            });
                        }
                        g.log(format!("CRAFT DELIVERED: {}", cname), ui::GREEN);
                        g.toast("CRAFT PURCHASED");
                    }
                }
                _ => {
                    if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                        base_mut.add_item(opt.name, 1);
                    }
                    g.toast("EQUIPMENT ACQUIRED");
                }
            }
        }
        y += 40.0;
    }

    // Right side: Salvage Liquidation & Alien Bounties (SELL)
    let right_w = c.w - left_w - 30.0;
    let right = Rect::new(left.right() + 10.0, c.y + 10.0, right_w, c.h - 20.0);
    draw_rectangle(right.x, right.y, right.w, right.h, Color { a: 0.30, ..PANEL });
    g.ui.label(right.x + 14.0, right.y + 24.0, 11.0, CYAN, "SALVAGE LIQUIDATION // ARTIFACT BOUNTIES");

    struct SellEntry {
        name: String,
        kind: u8, // 0: lattice, 1: electrolyte, 2: nacre, 3: brine, 4: captive, 5: item
        count: i32,
        price: i64,
        item_key: String,
    }

    let mut sellables: Vec<SellEntry> = Vec::new();
    if b.lattice > 0 { sellables.push(SellEntry { name: "SPINE".into(), kind: 0, count: b.lattice, price: 6_000, item_key: "".into() }); }
    if b.electrolyte > 0 { sellables.push(SellEntry { name: "GLOW".into(), kind: 1, count: b.electrolyte, price: 5_500, item_key: "".into() }); }
    if b.nacre > 0 { sellables.push(SellEntry { name: "NACRE PLATE".into(), kind: 2, count: b.nacre, price: 7_000, item_key: "".into() }); }
    if b.brine > 0 { sellables.push(SellEntry { name: "LIVE BRINE".into(), kind: 3, count: b.brine, price: 6_500, item_key: "".into() }); }
    let captives_count = b.captured_aliens.len() as i32;
    if captives_count > 0 { sellables.push(SellEntry { name: "LIVE ALIEN CAPTIVE".into(), kind: 4, count: captives_count, price: 45_000, item_key: "".into() }); }

    for (iname, icount) in &b.items {
        if *icount > 0 {
            let price = match iname.as_str() {
                "ARC TORCH" => 160_000,
                "ARC CUTTER" => 115_000,
                "CUTTER CELL" => 8_000,
                "PRESSURE RAM" => 175_000,
                "PRESSURE BORE" => 130_000,
                "ALIEN CORPSE" => 18_000,
                "SERVICE RIFLE" => 1_300,
                "DIVE RIFLE" => 1_450,
                // Anything the market sells is bought back at half its price, so stock cannot be flipped for profit.
                other => catalog.iter().find(|o| o.is_person == 0 && o.name == other).map(|o| o.cost / 2).unwrap_or(700),
            };
            sellables.push(SellEntry { name: iname.clone(), kind: 5, count: *icount, price, item_key: iname.clone() });
        }
    }

    let mut sy = right.y + 44.0;
    if sellables.is_empty() {
        g.ui.text(right.x + 14.0, sy + 20.0, 10.0, DIM, "NO SALVAGE ARTIFACTS IN STORES. RECOVER MATERIALS FROM BATTLE.");
    } else {
        let sell_rows = (((right.h - 54.0) / 40.0) as usize).max(9);
        for s in sellables.iter().take(sell_rows) {
            let r = Rect::new(right.x + 10.0, sy, right.w - 20.0, 36.0);
            let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
            draw_rectangle(r.x, r.y, r.w, r.h, if hover { Color { a: 0.15, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, Color { a: 0.25, ..CYAN_DIM });

            g.ui.bold(r.x + 10.0, r.y + 14.0, 9.5, TEXT, &s.name);
            g.ui.text(r.x + 10.0, r.y + 26.0, 8.0, DIM, &format!("HELD: {} UNITS  //  BOUNTY: +${}", s.count, crate::fmt_money(s.price)));

            if button(&g.ui, Rect::new(r.x + r.w - 74.0, r.y + 6.0, 66.0, 24.0), "SELL", Btn::Ghost, s.count > 0) {
                g.audio.play_click();
                g.funds += s.price;
                if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                    match s.kind {
                        0 => base_mut.lattice = base_mut.lattice.saturating_sub(1),
                        1 => base_mut.electrolyte = base_mut.electrolyte.saturating_sub(1),
                        2 => base_mut.nacre = base_mut.nacre.saturating_sub(1),
                        3 => base_mut.brine = base_mut.brine.saturating_sub(1),
                        4 => { if !base_mut.captured_aliens.is_empty() { base_mut.captured_aliens.remove(0); } },
                        _ => { base_mut.remove_item(&s.item_key, 1); },
                    }
                }
                g.log(format!("LIQUIDATED 1x {} // +${}", s.name, crate::fmt_money(s.price)), ui::GREEN);
            }
            sy += 40.0;
        }
    }
}

// ---------------------------------------------------------------- 5: FLEET & CRAFT
fn draw_fleet(g: &mut Game, c: Rect) {
    let left_w = c.w * 0.48;
    let left = Rect::new(c.x + 10.0, c.y + 10.0, left_w, c.h - 20.0);
    draw_rectangle(left.x, left.y, left.w, left.h, Color { a: 0.30, ..PANEL });
    g.ui.label(left.x + 14.0, left.y + 24.0, 11.0, CYAN, "BASE FLEET ROSTER // INTERCEPTORS & TRANSPORTS");

    let bsel = g.bases.get(g.selected_base).cloned();
    let Some(b) = bsel else { return; };

    let mut y = left.y + 44.0;
    for (i, craft) in b.craft.iter().enumerate() {
        let r = Rect::new(left.x + 10.0, y, left.w - 20.0, 56.0);
        let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
        let sel = g.selected_craft_idx == Some(i);
        draw_rectangle(r.x, r.y, r.w, r.h, if sel { Color { r: 0.06, g: 0.18, b: 0.22, a: 1.0 } } else if hover { Color { a: 0.15, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, if sel { CYAN } else { Color { a: 0.3, ..CYAN_DIM } });

        g.ui.bold(r.x + 10.0, r.y + 16.0, 11.0, if sel { CYAN } else { TEXT }, &craft.name);
        let cdef = craft_def(craft.ctype);
        g.ui.text(r.x + 10.0, r.y + 30.0, 9.0, DIM, &format!("{}  //  {} KNOTS  //  CAP: {} TROOPS", cdef.name, cdef.speed, cdef.soldier_cap));
        g.ui.text(r.x + 10.0, r.y + 46.0, 8.5, if craft.ready { ui::GREEN } else { AMBER }, if craft.ready { "STATUS: READY FOR DEPLOYMENT" } else { "STATUS: MAINTENANCE / REFUELING" });

        g.ui.text(r.x + r.w - 110.0, r.y + 16.0, 8.5, TEXT, &format!("HULL {}/{}", craft.hp, craft.max_hp));
        bar(Rect::new(r.x + r.w - 110.0, r.y + 22.0, 100.0, 4.0), craft.hp as f32, craft.max_hp as f32, ui::GREEN, Color { a: 0.3, ..BLACK });

        g.ui.text(r.x + r.w - 110.0, r.y + 38.0, 8.5, DIM, &format!("FUEL {}/{}", craft.fuel, craft.max_fuel));
        bar(Rect::new(r.x + r.w - 110.0, r.y + 44.0, 100.0, 4.0), craft.fuel as f32, craft.max_fuel as f32, CYAN, Color { a: 0.3, ..BLACK });

        if hover && is_mouse_button_released(MouseButton::Left) { g.selected_craft_idx = Some(i); }
        y += 62.0;
    }

    // Purchase New Craft button
    let can_order = b.craft.len() < b.craft_cap() as usize;
    if button(&g.ui, Rect::new(left.x + 14.0, left.y + left.h - 46.0, left.w - 28.0, 32.0), "PURCHASE NEW CRAFT", Btn::Primary, can_order) {
        if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
            let cost = 650_000;
            if g.funds >= cost {
                g.funds -= cost;
                let cnum = base_mut.craft.len() + 1;
                if base_mut.water {
                    base_mut.craft.push(CraftInstance {
                        name: format!("SKUA-{}", cnum),
                        ctype: CraftType::Skua,
                        ready: true,
                        hp: 125, max_hp: 125,
                        fuel: 850, max_fuel: 850,
                        weapons: [Some(CraftWeaponType::LightTorpedo), None],
                        ammo: [8, 0],
                    });
                } else {
                    base_mut.craft.push(CraftInstance {
                        name: format!("SWIFT-{}", cnum),
                        ctype: CraftType::Swift,
                        ready: true,
                        hp: 110, max_hp: 110,
                        fuel: 900, max_fuel: 900,
                        weapons: [Some(CraftWeaponType::ShortRangeMissile), None],
                        ammo: [6, 0],
                    });
                }
                g.log("NEW COMBAT CRAFT DELIVERED TO HANGAR.", ui::GREEN);
            } else { g.toast("INSUFFICIENT FUNDS ($650K NEEDED)"); }
        }
    }

    // Right side: Craft Weapons & Systems Config
    let right_w = c.w - left_w - 30.0;
    let right = Rect::new(left.right() + 10.0, c.y + 10.0, right_w, c.h - 20.0);
    draw_rectangle(right.x, right.y, right.w, right.h, Color { a: 0.30, ..PANEL });

    let sel_idx = g.selected_craft_idx.unwrap_or(0);
    if let Some(craft) = b.craft.get(sel_idx).cloned() {
        let cdef = craft_def(craft.ctype);
        g.ui.label(right.x + 14.0, right.y + 24.0, 11.0, CYAN, "CRAFT SYSTEMS CONFIGURATION");
        g.ui.bold(right.x + 14.0, right.y + 50.0, 17.0, TEXT, &craft.name);
        g.ui.text(right.x + 14.0, right.y + 70.0, 9.5, DIM, &format!("TYPE: {}  //  MAX SPEED: {} KNOTS  //  FUEL: {}", cdef.name, cdef.speed, cdef.fuel_name));

        let mut py = right.y + 100.0;
        for pod_i in 0..cdef.weapon_mounts {
            let pod_box = Rect::new(right.x + 14.0, py, right.w - 28.0, 68.0);
            draw_rectangle(pod_box.x, pod_box.y, pod_box.w, pod_box.h, Color { a: 0.20, ..PANEL_2 });
            draw_rectangle_lines(pod_box.x, pod_box.y, pod_box.w, pod_box.h, 1.0, Color { a: 0.35, ..CYAN_DIM });

            g.ui.label(pod_box.x + 10.0, pod_box.y + 16.0, 9.0, CYAN, &format!("WEAPON MOUNT // POD {}", pod_i + 1));
            let wopt = craft.weapons[pod_i];
            let wname = wopt.map(|w| craft_weapon_def(w).name).unwrap_or("EMPTY / UNARMED");
            g.ui.bold(pod_box.x + 10.0, pod_box.y + 36.0, 11.0, TEXT, wname);

            if let Some(wtype) = wopt {
                let wdef = craft_weapon_def(wtype);
                g.ui.text(pod_box.x + 10.0, pod_box.y + 52.0, 8.5, DIM, &format!("RANGE {} KM  //  DMG {}  //  ACC {}%  //  AMMO {}/{}", wdef.range_km, wdef.damage, wdef.accuracy, craft.ammo[pod_i], wdef.max_ammo));
            }

            if button(&g.ui, Rect::new(pod_box.x + pod_box.w - 84.0, pod_box.y + 12.0, 76.0, 24.0), "ARM...", Btn::Normal, true) {
                g.craft_weapon_modal = Some((sel_idx, pod_i));
            }

            py += 78.0;
        }

        // Modal for weapon selection
        if let Some((c_idx, pod_i)) = g.craft_weapon_modal {
            let modal_r = Rect::new(right.x + 10.0, py + 10.0, right.w - 20.0, 180.0);
            panel(modal_r, PANEL, CYAN);
            g.ui.label(modal_r.x + 10.0, modal_r.y + 18.0, 9.5, CYAN, "SELECT WEAPON SYSTEM");

            let avail_w: Vec<&CraftWeaponDef> = CRAFT_WEAPONS.iter().filter(|w| w.game == if b.water { 1 } else { 0 }).collect();
            let mut my = modal_r.y + 32.0;
            for wdef in avail_w {
                let r = Rect::new(modal_r.x + 8.0, my, modal_r.w - 16.0, 22.0);
                let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
                draw_rectangle(r.x, r.y, r.w, r.h, if hover { Color { a: 0.25, ..PANEL_2 } } else { Color { a: 0.10, ..PANEL } });
                g.ui.text(r.x + 6.0, r.y + 14.0, 8.5, TEXT, &format!("{} (${}k)", wdef.name, wdef.cost / 1000));
                if hover && is_mouse_button_released(MouseButton::Left) {
                    if let Some(base_mut) = g.bases.get_mut(g.selected_base) {
                        if let Some(c_mut) = base_mut.craft.get_mut(c_idx) {
                            c_mut.weapons[pod_i] = Some(wdef.wtype);
                            c_mut.ammo[pod_i] = wdef.max_ammo;
                            g.craft_weapon_modal = None;
                        }
                    }
                }
                my += 24.0;
            }
        }
    }
}

// ---------------------------------------------------------------- 5: FINANCES & ASSEMBLY
fn draw_finances(g: &mut Game, c: Rect) {
    let left_w = c.w * 0.52;
    let left = Rect::new(c.x + 10.0, c.y + 10.0, left_w, c.h - 20.0);
    draw_rectangle(left.x, left.y, left.w, left.h, Color { a: 0.30, ..PANEL });
    g.ui.label(left.x + 14.0, left.y + 24.0, 11.0, CYAN, "CHARTER ASSEMBLY // FUNDING STATES AND SEA-LANE GROUPS");

    let mut y = left.y + 44.0;
    let start = g.finance_scroll.min(g.nations.len().saturating_sub(10));
    for n in g.nations.iter().skip(start).take(10) {
        let r = Rect::new(left.x + 10.0, y, left.w - 20.0, 36.0);
        let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(r.x, r.y, r.w, r.h, if hover { Color { a: 0.18, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, Color { a: 0.3, ..CYAN_DIM });

        let col = if n.pact_signed { ui::RED } else { TEXT };
        g.ui.bold(r.x + 10.0, r.y + 15.0, 9.5, col, &n.name);
        g.ui.text(r.x + 10.0, r.y + 28.0, 8.0, DIM, if n.game == 0 { "CHARTER STATE" } else { "SEA-LANE GROUP" });

        let fund_str = if n.pact_signed { "SEPARATE TERMS (LOST)".to_string() } else { format!("${}/MO", crate::fmt_money(n.funding)) };
        g.ui.bold(r.x + r.w - 150.0, r.y + 15.0, 9.5, if n.pact_signed { ui::RED } else { ui::GREEN }, &fund_str);

        // Satisfaction bar
        bar(Rect::new(r.x + r.w - 150.0, r.y + 22.0, 140.0, 4.0), n.satisfaction as f32, 100.0, if n.satisfaction > 60 { ui::GREEN } else if n.satisfaction > 30 { AMBER } else { ui::RED }, Color { a: 0.3, ..BLACK });

        y += 40.0;
    }

    // Scroll buttons
    if g.nations.len() > 10 {
        if button(&g.ui, Rect::new(left.x + 14.0, left.y + left.h - 40.0, 80.0, 24.0), "PAGE UP", Btn::Ghost, start > 0) {
            g.finance_scroll = g.finance_scroll.saturating_sub(5);
        }
        if button(&g.ui, Rect::new(left.x + 104.0, left.y + left.h - 40.0, 80.0, 24.0), "PAGE DOWN", Btn::Ghost, start + 10 < g.nations.len()) {
            g.finance_scroll = (g.finance_scroll + 5).min(g.nations.len().saturating_sub(10));
        }
    }

    // Right side: Financial Statement & Ledger
    let right_w = c.w - left_w - 30.0;
    let right = Rect::new(left.right() + 10.0, c.y + 10.0, right_w, c.h - 20.0);
    draw_rectangle(right.x, right.y, right.w, right.h, Color { a: 0.30, ..PANEL });
    g.ui.label(right.x + 14.0, right.y + 24.0, 11.0, CYAN, "MONTHLY FINANCIAL STATEMENT");

    let total_funding: i64 = g.nations.iter().filter(|n| !n.pact_signed).map(|n| n.funding).sum();
    let mut fac_maint = 0i64;
    let mut craft_maint = 0i64;
    let mut total_sci = 0i32;
    let mut total_eng = 0i32;
    for b in &g.bases {
        fac_maint += b.monthly_upkeep();
        for cr in &b.craft { craft_maint += craft_def(cr.ctype).upkeep; }
        total_sci += b.scientists;
        total_eng += b.engineers;
    }
    let sold_sal = g.soldiers.iter().filter(|s| !s.dead).count() as i64 * 20_000;
    let sci_sal = total_sci as i64 * 30_000;
    let eng_sal = total_eng as i64 * 25_000;
    let total_expenses = fac_maint + craft_maint + sold_sal + sci_sal + eng_sal;
    let net = total_funding - total_expenses;

    let mut fy = right.y + 54.0;
    let statement = [
        ("TOTAL ASSEMBLY INCOME", format!("+${}", crate::fmt_money(total_funding)), ui::GREEN),
        ("BASE FACILITIES UPKEEP", format!("-${}", crate::fmt_money(fac_maint)), AMBER),
        ("CRAFT FLEET UPKEEP", format!("-${}", crate::fmt_money(craft_maint)), AMBER),
        ("SOLDIER SALARIES ($20K)", format!("-${}", crate::fmt_money(sold_sal)), DIM),
        ("SCIENTIST SALARIES ($30K)", format!("-${}", crate::fmt_money(sci_sal)), DIM),
        ("ENGINEER SALARIES ($25K)", format!("-${}", crate::fmt_money(eng_sal)), DIM),
        ("TOTAL MONTHLY EXPENSES", format!("-${}", crate::fmt_money(total_expenses)), ui::RED),
        ("PROJECTED NET BALANCE", format!("{}${ }", if net >= 0 { "+" } else { "-" }, crate::fmt_money(net.abs())), if net >= 0 { ui::GREEN } else { ui::RED }),
    ];

    for (item, val, col) in statement {
        g.ui.text(right.x + 14.0, fy, 9.5, TEXT, item);
        g.ui.bold(right.x + right.w - g.ui.bwidth(10.0, &val) - 16.0, fy, 10.0, col, &val);
        fy += 24.0;
    }

    fy += 16.0;
    let rating = if g.score >= 800 { "EXCELLENT" } else if g.score >= 400 { "GOOD" } else if g.score >= 0 { "SATISFACTORY" } else if g.score >= -400 { "POOR" } else { "TERRIBLE" };
    g.ui.bold(right.x + 14.0, fy, 11.0, AMBER, &format!("ASSEMBLY RATING: {}", rating));

    if button(&g.ui, Rect::new(right.x + 14.0, right.y + right.h - 50.0, right.w - 28.0, 34.0), "OPEN ASSEMBLY REVIEW", Btn::Primary, true) {
        g.screen = Screen::AssemblyReport;
    }
}

// ---------------------------------------------------------------- 6: LAUNCH OPERATIONS
fn draw_launch(g: &mut Game, c: Rect) {
    if g.selected_site.is_none() && !g.sites.is_empty() { g.selected_site = Some(0); }
    let left = Rect::new(c.x + 10.0, c.y + 10.0, c.w * 0.55, c.h - 20.0);
    draw_rectangle(left.x, left.y, left.w, left.h, Color { a: 0.30, ..PANEL });
    g.ui.label(left.x + 12.0, left.y + 22.0, 11.0, CYAN, "ACTIVE OPERATIONS");
    if g.sites.is_empty() {
        g.ui.text(left.x + 12.0, left.y + 56.0, 11.0, DIM, "No active sites. Advance time on the situation map and intercept contacts.");
    }
    let mut y = left.y + 40.0;
    let rows: Vec<(usize, String, bool, bool, usize)> = g.sites.iter().enumerate().map(|(i, s)| (i, s.name.clone(), s.underwater, s.raid, s.aliens.len())).collect();
    for (i, name, underwater, raid, n) in rows.into_iter().take(6) {
        let r = Rect::new(left.x + 10.0, y, left.w - 20.0, 48.0);
        let sel = g.selected_site == Some(i);
        let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(r.x, r.y, r.w, r.h, if sel { Color { r: 0.06, g: 0.18, b: 0.20, a: 1.0 } } else if hover { Color { a: 0.14, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
        draw_rectangle(r.x, r.y, 2.0, r.h, if raid { ui::RED } else if underwater { CYAN } else { AMBER });
        g.ui.bold(r.x + 10.0, r.y + 18.0, 11.0, TEXT, &name);
        g.ui.text(r.x + 10.0, r.y + 32.0, 9.0, DIM, &format!("{}  //  {} HOSTILES", if underwater { "UNDERWATER" } else { "SURFACE" }, n));
        if hover && is_mouse_button_released(MouseButton::Left) { g.selected_site = Some(i); }
        y += 54.0;
    }

    if let Some(site) = g.selected_site.and_then(|i| g.sites.get(i)).cloned() {
        let by = (y + 8.0).min(left.y + left.h - 250.0);
        let br = Rect::new(left.x + 10.0, by, left.w - 20.0, 200.0);
        draw_rectangle(br.x, br.y, br.w, br.h, Color { a: 0.35, ..PANEL_2 });
        draw_rectangle_lines(br.x, br.y, br.w, br.h, 1.0, Color { a: 0.35, ..CYAN_DIM });

        g.ui.label(br.x + 12.0, br.y + 20.0, 9.0, CYAN, "MISSION BRIEFING");
        g.ui.bold(br.x + 12.0, br.y + 40.0, 14.0, TEXT, &g.ui.fit(14.0, &site.name, br.w - 24.0, true));
        let env = if site.underwater { "UNDERWATER" } else { "SURFACE" };
        let threat = match site.tier { 0..=1 => ("LOW", ui::GREEN), 2..=3 => ("MODERATE", AMBER), 4..=5 => ("HIGH", ui::RED), _ => ("EXTREME", ui::RED) };
        g.ui.text(br.x + 12.0, br.y + 58.0, 9.0, DIM, &format!("ENVIRONMENT: {}   THREAT: {}", env, threat.0));
        g.ui.text(br.x + 12.0, br.y + 74.0, 9.0, DIM, &format!("HOSTILES: APPROX {}   RECOVERY: ${}", site.aliens.len(), crate::fmt_money(site.reward)));
        let hours = site.expires / 60;
        g.ui.text(br.x + 12.0, br.y + 90.0, 9.0, DIM, &format!("WINDOW: {} H {} M REMAINING", hours, site.expires % 60));
        g.ui.text(br.x + 12.0, br.y + 114.0, 9.0, if site.underwater { CYAN } else { AMBER },
            if site.underwater { "DIVE GEAR ONLY. SURFACE WEAPONS AND ARMOUR STAY BEHIND." } else { "STANDARD GEAR. DIVE GEAR IS NOT USED ON LAND." });
        let ready = g.ready_soldiers();
        g.ui.text(br.x + 12.0, br.y + 164.0, 10.0, if ready > 0 { ui::GREEN } else { ui::RED }, &format!("{} SOLDIERS READY TO DEPLOY", ready));
        if button(&g.ui, Rect::new(br.x + br.w - 186.0, br.y + 150.0, 174.0, 34.0), "LAUNCH MISSION", Btn::Primary, ready > 0) {
            g.launch_selected_site();
        }
    }

    let right = Rect::new(c.x + c.w * 0.55 + 20.0, c.y + 10.0, c.w * 0.45 - 30.0, c.h - 20.0);
    draw_rectangle(right.x, right.y, right.w, right.h, Color { a: 0.30, ..PANEL });
    g.ui.label(right.x + 12.0, right.y + 22.0, 11.0, CYAN, "SQUAD MANIFEST");
    let ready = g.ready_soldiers();
    g.ui.text(right.x + 12.0, right.y + 42.0, 10.0, if ready > 0 { ui::GREEN } else { ui::RED }, &format!("{} / 8 READY", ready));

    let mut y = right.y + 72.0;
    for s in g.soldiers.iter() {
        if !s.assigned || s.dead { continue; }
        draw_circle(right.x + 18.0, y - 4.0, 4.0, if s.wounded_days > 0 { AMBER } else { ui::GREEN });
        let nw = right.w * 0.42 - 32.0;
        g.ui.text(right.x + 28.0, y, 9.0, TEXT, &g.ui.fit(9.0, &s.name, nw, false));
        g.ui.text(right.x + right.w * 0.42, y, 8.0, DIM, &g.ui.fit(8.0, &g.data.weapons[s.weapon].name, right.w * 0.26, false));
        g.ui.text(right.x + right.w * 0.70, y, 8.0, if ARMORS[s.armor as usize].underwater { CYAN } else { DIM }, &g.ui.fit(8.0, ARMORS[s.armor as usize].name, right.w * 0.22, false));
        g.ui.text(right.x + right.w - 54.0, y, 8.0, DIM, &format!("G{} M{}", s.grenades, s.medkits));
        y += 19.0;
        if y > right.y + right.h - 120.0 { break; }
    }

    let underwater = g.selected_site.and_then(|i| g.sites.get(i)).map(|s| s.underwater).unwrap_or(false);
    if underwater {
        g.ui.text(right.x + 12.0, right.y + right.h - 108.0, 9.0, ui::RED, "WATER OPERATION: ABOVE-GROUND GEAR");
        g.ui.text(right.x + 12.0, right.y + right.h - 96.0, 9.0, ui::RED, "WILL BE SWAPPED FOR DIVE GEAR ON DEPLOYMENT.");
    }
    if button(&g.ui, Rect::new(right.x + 12.0, right.y + right.h - 84.0, right.w - 24.0, 32.0), "AUTO-EQUIP SQUAD", Btn::Normal, true) {
        for i in 0..g.soldiers.len() { if g.soldiers[i].assigned { g.equip_default(i, underwater); } }
        g.toast("SQUAD EQUIPPED");
    }
    if button(&g.ui, Rect::new(right.x + 12.0, right.y + right.h - 46.0, right.w - 24.0, 34.0), "LAUNCH MISSION", Btn::Primary, ready > 0 && g.selected_site.is_some()) {
        g.launch_selected_site();
    }
}


pub fn update_editor(g: &mut Game, _dt: f32) {
    if is_key_pressed(KeyCode::Escape) {
        g.screen = g.editor_return;
    }
    if is_key_pressed(KeyCode::R) {
        let mut gen = g.dice.generator("recruit");
        let look = Look::random(&mut gen);
        let idx = g.selected_soldier;
        g.soldiers[idx].look = look;
        let name = g.soldiers[idx].look.name(&mut gen);
        g.soldiers[idx].name = name;
    }
}

fn choice_row(g: &mut Game, x: f32, y: f32, w: f32, label: &str, items: &[&str], selected: usize, on_pick: &mut dyn FnMut(&mut Game, usize)) {
    g.ui.label(x, y + 16.0, 9.0, DIM, label);
    let bx = x + 78.0;
    let avail = w - 78.0;
    let n = items.len();
    let bw = (avail - (n.saturating_sub(1)) as f32 * 4.0) / n as f32;
    for (i, it) in items.iter().enumerate() {
        let r = Rect::new(bx + i as f32 * (bw + 4.0), y, bw, 24.0);
        let sel = i == selected;
        let hover = r.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(r.x, r.y, r.w, r.h, if sel { Color { r: 0.06, g: 0.19, b: 0.22, a: 1.0 } } else if hover { Color { a: 0.14, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, if sel { CYAN } else { Color { a: 0.32, ..CYAN_DIM } });
        g.ui.center(r.x + r.w * 0.5, r.y + 16.0, 8.0, if sel { CYAN } else { TEXT }, it);
        if hover && is_mouse_button_released(MouseButton::Left) { on_pick(g, i); }
    }
}

pub fn draw_editor(g: &mut Game) {
    let w = screen_width(); let h = screen_height();
    base_background(g);
    draw_rectangle(0.0, 0.0, w, 42.0, Color { r: 0.012, g: 0.030, b: 0.042, a: 0.96 });
    draw_rectangle(0.0, 42.0, w, 1.0, Color { a: 0.6, ..CYAN });
    g.ui.label(14.0, 26.0, 11.0, CYAN, "PERSONNEL");
    g.ui.bold(104.0, 28.0, 15.0, TEXT, "EDIT SOLDIER // APPEARANCE");
    g.ui.text(w - 260.0, 26.0, 9.0, DIM, "ALL CHANGES ARE COSMETIC");

    let s = g.soldiers[g.selected_soldier].clone();
    // --- left: a framed portrait and the service record
    let pr = Rect::new(24.0, 62.0, 252.0, 300.0);
    ui::portrait(&g.ui, pr, s.look, g.elapsed, true);
    let name = g.ui.fit(20.0, &s.name, 252.0, true);
    g.ui.bold(24.0, 392.0, 20.0, TEXT, &name);
    g.ui.text(24.0, 410.0, 9.0, DIM, &format!("{} // {} // ID {:04}", RANKS[s.rank as usize], s.look.ethnicity.label(), s.id));
    g.ui.label(24.0, 434.0, 8.0, DIM, "SERVICE RECORD");
    let stats = [("TU", s.stats.tu), ("STA", s.stats.stamina), ("HP", s.stats.health), ("BRA", s.stats.bravery),
                 ("REA", s.stats.reactions), ("FA", s.stats.firing), ("TA", s.stats.throwing), ("STR", s.stats.strength)];
    for (i, (n, v)) in stats.iter().enumerate() {
        let x = 24.0 + (i % 4) as f32 * 62.0;
        let y = 452.0 + (i / 4) as f32 * 18.0;
        g.ui.text(x, y, 8.0, DIM, n);
        g.ui.bold(x + 26.0, y, 11.0, TEXT, &format!("{}", v));
    }
    g.ui.text(24.0, 500.0, 8.0, DIM, &format!("WOUNDS {}   MISSIONS {}   KILLS {}", s.wounds, s.missions, s.kills));

    // --- right: the appearance form
    let x = 300.0;
    let cw = w - x - 24.0;
    let half = (cw - 12.0) * 0.5;
    let row = if h < 620.0 { 26.0 } else { 32.0 };
    let mut y = 58.0;

    // name row
    g.ui.label(x, y + 16.0, 9.0, DIM, "NAME");
    let nr = Rect::new(x + 78.0, y, cw * 0.48 - 78.0, 30.0);
    draw_rectangle(nr.x, nr.y, nr.w, nr.h, Color { a: 0.55, ..PANEL_2 });
    draw_rectangle_lines(nr.x, nr.y, nr.w, nr.h, 1.0, Color { a: 0.45, ..CYAN_DIM });
    g.ui.text(nr.x + 9.0, nr.y + 20.0, 12.0, TEXT, &g.ui.fit(12.0, &g.soldiers[g.selected_soldier].name, nr.w - 18.0, false));
    if button(&g.ui, Rect::new(x + cw * 0.50, y, cw * 0.50, 30.0), "RANDOMIZE NAME", Btn::Normal, true) {
        let mut gen = g.dice.generator("name");
        let nm = g.soldiers[g.selected_soldier].look.name(&mut gen);
        g.soldiers[g.selected_soldier].name = nm;
    }
    y += 44.0;

    // gender (left half) and heritage (right half, 3 x 3)
    let gl = if g.soldiers[g.selected_soldier].look.gender == Gender::Male { 0 } else { 1 };
    choice_row(g, x, y, half - 8.0, "GENDER", &["MALE", "FEMALE"], gl, &mut |g, i| {
        g.soldiers[g.selected_soldier].look.gender = if i == 0 { Gender::Male } else { Gender::Female };
    });
    let eth_labels = ["ANGLO", "NORDIC", "SLAVIC", "HISPANIC", "AFRICAN", "ARAB", "S. ASIAN", "E. ASIAN", "PACIFIC"];
    let cur_eth = Ethnicity::ALL.iter().position(|e| *e == g.soldiers[g.selected_soldier].look.ethnicity).unwrap_or(0);
    let hx = x + half + 4.0;
    let hw = cw - half - 4.0;
    g.ui.label(hx, y + 16.0, 9.0, DIM, "HERITAGE");
    let hb_x = hx + 78.0;
    let hb_w = (hw - 82.0) / 3.0;
    for i in 0..9 {
        let bx = hb_x + (i % 3) as f32 * (hb_w + 2.0);
        let by = y + (i / 3) as f32 * 22.0;
        let rr = Rect::new(bx, by, hb_w, 20.0);
        let sel = i == cur_eth;
        let hov = rr.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(rr.x, rr.y, rr.w, rr.h, if sel { Color { r: 0.06, g: 0.19, b: 0.22, a: 1.0 } } else if hov { Color { a: 0.14, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
        draw_rectangle_lines(rr.x, rr.y, rr.w, rr.h, 1.0, if sel { CYAN } else { Color { a: 0.3, ..CYAN_DIM } });
        g.ui.center(rr.x + rr.w * 0.5, rr.y + 13.0, 7.5, if sel { CYAN } else { TEXT }, eth_labels[i]);
        if hov && is_mouse_button_released(MouseButton::Left) {
            g.soldiers[g.selected_soldier].look.ethnicity = Ethnicity::ALL[i];
            let (lo, hi) = Ethnicity::ALL[i].skin_range();
            if g.soldiers[g.selected_soldier].look.skin < lo || g.soldiers[g.selected_soldier].look.skin > hi {
                g.soldiers[g.selected_soldier].look.skin = lo;
            }
        }
    }
    y += 74.0;

    // skin swatches
    g.ui.label(x, y + 16.0, 9.0, DIM, "SKIN");
    for i in 0..6 {
        let rr = Rect::new(x + 78.0 + i as f32 * 40.0, y, 36.0, 22.0);
        draw_rectangle(rr.x, rr.y, rr.w, rr.h, skin_color(i as u8));
        draw_rectangle_lines(rr.x, rr.y, rr.w, rr.h, if g.soldiers[g.selected_soldier].look.skin == i as u8 { 2.0 } else { 1.0 },
            if g.soldiers[g.selected_soldier].look.skin == i as u8 { CYAN } else { Color { a: 0.4, ..BLACK } });
        if rr.contains(vec2(mouse_position().0, mouse_position().1)) && is_mouse_button_released(MouseButton::Left) {
            g.soldiers[g.selected_soldier].look.skin = i as u8;
        }
    }
    y += row;

    let idx = g.selected_soldier;
    choice_row(g, x, y, cw, "FACE SHAPE", &["OVAL", "LONG", "SQUARE", "ANGULAR"], g.soldiers[idx].look.face as usize, &mut |g, i| g.soldiers[g.selected_soldier].look.face = i as u8);
    y += row;
    choice_row(g, x, y, cw, "JAW", &["SOFT", "NORMAL", "STRONG", "HEAVY"], g.soldiers[idx].look.jaw as usize, &mut |g, i| g.soldiers[g.selected_soldier].look.jaw = i as u8);
    y += row;
    choice_row(g, x, y, cw, "NOSE", &["STRAIGHT", "WIDE", "HOOKED", "SHORT"], g.soldiers[idx].look.nose as usize, &mut |g, i| g.soldiers[g.selected_soldier].look.nose = i as u8);
    y += row;
    choice_row(g, x, y, cw, "BROW", &["SOFT", "STRAIGHT", "HEAVY"], g.soldiers[idx].look.brow as usize, &mut |g, i| g.soldiers[g.selected_soldier].look.brow = i as u8);
    y += row;
    choice_row(g, x, y, cw, "EYES", &["ROUND", "ALMOND", "NARROW"], g.soldiers[idx].look.eyes as usize, &mut |g, i| g.soldiers[g.selected_soldier].look.eyes = i as u8);
    y += row;

    // hair style
    g.ui.label(x, y + 16.0, 9.0, DIM, "HAIR");
    let styles = ["BUZZ", "SHORT", "LONG", "BUN", "BRAIDS", "CURLY"];
    let bw = (cw - 82.0 - 5.0 * 4.0) / 6.0;
    for (i, st) in styles.iter().enumerate() {
        let rr = Rect::new(x + 78.0 + i as f32 * (bw + 4.0), y, bw, 22.0);
        let sel = g.soldiers[idx].look.hair as usize == i;
        let hov = rr.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(rr.x, rr.y, rr.w, rr.h, if sel { Color { r: 0.06, g: 0.19, b: 0.22, a: 1.0 } } else if hov { Color { a: 0.14, ..PANEL_2 } } else { Color { a: 0.08, ..PANEL } });
        draw_rectangle_lines(rr.x, rr.y, rr.w, rr.h, 1.0, if sel { CYAN } else { Color { a: 0.3, ..CYAN_DIM } });
        g.ui.center(rr.x + rr.w * 0.5, rr.y + 14.0, 7.5, if sel { CYAN } else { TEXT }, st);
        if hov && is_mouse_button_released(MouseButton::Left) { g.soldiers[idx].look.hair = i as u8; }
    }
    y += row;

    // hair colour
    g.ui.label(x, y + 16.0, 9.0, DIM, "COLOR");
    for i in 0..8 {
        let rr = Rect::new(x + 78.0 + i as f32 * 34.0, y, 30.0, 22.0);
        draw_rectangle(rr.x, rr.y, rr.w, rr.h, hair_color(i as u8));
        draw_rectangle_lines(rr.x, rr.y, rr.w, rr.h, if g.soldiers[idx].look.hair_color == i as u8 { 2.0 } else { 1.0 },
            if g.soldiers[idx].look.hair_color == i as u8 { CYAN } else { Color { a: 0.4, ..BLACK } });
        if rr.contains(vec2(mouse_position().0, mouse_position().1)) && is_mouse_button_released(MouseButton::Left) {
            g.soldiers[idx].look.hair_color = i as u8;
        }
    }
    y += row;

    if g.soldiers[idx].look.gender == Gender::Male {
        choice_row(g, x, y, cw, "FACIAL HAIR", &["NONE", "STUBBLE", "BEARD", "MOUSTACHE"], g.soldiers[idx].look.facial_hair as usize, &mut |g, i| g.soldiers[g.selected_soldier].look.facial_hair = i as u8);
    } else {
        g.ui.label(x, y + 16.0, 9.0, DIM, "FACIAL HAIR");
        g.ui.text(x + 78.0, y + 17.0, 9.0, Color { a: 0.5, ..DIM }, "NOT AVAILABLE FOR THIS RECORD");
    }

    // actions, anchored to the bottom of the window
    let by = h - 52.0;
    if button(&g.ui, Rect::new(x, by, 160.0, 34.0), "RANDOMIZE", Btn::Normal, true) {
        let mut gen = g.dice.generator("recruit");
        let look = Look::random(&mut gen);
        g.soldiers[idx].look = look;
        let nm = g.soldiers[idx].look.name(&mut gen);
        g.soldiers[idx].name = nm;
    }
    if button(&g.ui, Rect::new(x + 170.0, by, 130.0, 34.0), "CANCEL", Btn::Ghost, true) {
        g.screen = g.editor_return;
    }
    if button(&g.ui, Rect::new(w - 214.0, by, 190.0, 34.0), "SAVE & RETURN", Btn::Primary, true) {
        g.screen = g.editor_return;
    }
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
fn hair_color(i: u8) -> Color {
    match i {
        0 => Color::new(0.07, 0.06, 0.06, 1.0),
        1 => Color::new(0.16, 0.11, 0.07, 1.0),
        2 => Color::new(0.29, 0.19, 0.10, 1.0),
        3 => Color::new(0.44, 0.27, 0.12, 1.0),
        4 => Color::new(0.62, 0.46, 0.24, 1.0),
        5 => Color::new(0.30, 0.30, 0.30, 1.0),
        6 => Color::new(0.78, 0.78, 0.76, 1.0),
        _ => Color::new(0.55, 0.16, 0.09, 1.0),
    }
}
