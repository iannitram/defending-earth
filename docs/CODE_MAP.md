# Defending Earth — Code Map

Written 2026-11-13, 3pm, and checked against the code again later that day,
after graphics phase 2. Checked a third time that day, after the on-screen
renames (SPINE, GLOW, REACH, BREACHER BATTERY, the Moon), and the two new
sections were then checked once more against `LORE.md`. For whoever
(person or Claude session) picks the work up next. Line numbers are as of
that last check and drift; find things again with
`grep -n "^fn \|^pub fn \|^pub struct " src/<file>.rs`.

Two sections near the end are new: "On screen / in code" (the words the
player sees against the words the code uses) and "Where the planned systems
will go" (the work that is decided in `LORE.md` but not built yet).

Read first: `STATUS.md` (where the work stands), `ROADMAP.md` (what comes
next and how), `ART_BIBLE.md` (the look), `LORE.md` (story, co-designed with
the user), `NAMES.md` (every name in the game).

## Ground rules that are not in the code

- Engine stays Rust + macroquad 0.4.16 (miniquad 0.4.11). Anything that cannot
  be done in it gets raised with the user before swapping anything.
- The code lives in a public git repository, https://github.com/iannitram/defending-earth
  (created 2026-11-13 at the user's request, earlier than first planned).
  The project folder is the repository. Commit before a risky refactor so there is a
  state to go back to.
- Original work only. The old investigation material was
  deleted on 2026-11-13.
- All questions to the user go through the question tool; times are written
  7am / 4:45pm. The user is building their first game: explain plainly.
- Creature lore is decided with the user, not for them.
- The game opens a real window: scripts need `DISPLAY=:0`, and every launch
  flashes on the user's screen.
- Downloads: Poly Haven only works with the user's Tailscale exit node on
  (the user switches it; we do not touch network settings). CC0 files go in
  `source_assets/polyhaven/` with a `SOURCE.md`.
- Python helpers: `python3 -P` for our own scripts (Pillow is installed),
  `python3 -I` for anything that reads downloaded files.

## Commands

```
cargo build --release
DISPLAY=:0 ./target/release/defending_earth                      # play
DISPLAY=:0 ./target/release/defending_earth --self-test out.json # scripted battle, must pass
DISPLAY=:0 ./target/release/defending_earth --demo tactical --screenshot shots/x.png
DISPLAY=:0 ./target/release/defending_earth --ai-test            # headless-ish AI run, prints counters
cargo build --release --example lightlab
DISPLAY=:0 ./target/release/examples/lightlab --view tele --clock "2026-06-21 8am" --frames 40 --screenshot out.png
```

`--demo <name>`: title, map (situation map), campaign, base (also `equip`),
facilities, research, workshop, market, fleet, finances, launch, assembly,
editor, dogfight (also `interception`), debrief, tactical, water; prefix
`sea-` for the underwater campaign. `--screenshot <file>` saves a picture
at frame 15 (or after `--t` seconds, if that is later) and exits; without
`--demo` it shows the situation map. `--self-test` needs a path argument.
`--dump-map` writes the decoded world map to `shots/worldmap.png` and exits.

Debug flags (read first, so they can go anywhere on the line): `--clock
"2027-06-21 8am"` (local solar time at the battle site; UTC for the globe),
`--weather clear|rain|snow|dust|storm|blizzard|duststorm` (the last three
are the heavy forms), `--site <lat,lon>` (moves the battle to that place:
biome from the world map there, season from the latitude and the month),
`--biome <n>` (forces the ground type; see the biome list under
`tactical.rs`), `--seed <n>`, `--view wide|tele|low`, `--t <seconds before
the screenshot>`, `--frametime` (readout on screen and `FRAMETIME avg`
printed at exit). `--biome` and `--weather` do nothing under water, and
`--site` there only moves the place.

The lab has its own flags: `--view wide|tele|site|low|top`, `--clock`,
`--lat`, `--lon`, `--cover` (cloud, 0–1), `--storm` (0–1), `--wet` (0–1),
`--seed`, `--frames <n>`, `--speed <game minutes per second>`,
`--screenshot <file>`.

Two limits to remember: one drawn mesh holds at most 60,000 corner points
(`MB_MAX` in `tactical.rs`; `Geo` in `gfx.rs` splits at the same number),
and the draw buffer set in `main.rs` with
`gl_set_drawcall_buffer_capacity(65_535, 400_000)` must keep its first
number at 65,535 or lower, otherwise big meshes merge and scramble (this
caused a full white screen in blizzards). `data::world_index` reads the
world map with its left edge at 180° west. Before phase 2 it was half a
world off, so every site in the game now gets different (correct) terrain.

## Layout

```
Defending Earth/
  Cargo.toml            macroquad 0.4, serde, serde_json
  src/                  the game (below)
  examples/lightlab.rs  the lighting test scene; the reference for how to drive gfx.rs
  assets/               shipped files: earth_albedo/data/lights.png, fonts
                        (ui.ttf, ui_bold.ttf), selftest.json,
                        ground/*.png (5 packed Poly Haven grains), sky/looks.txt
  source_assets/        originals and bake scripts: blender/ (sky/make_looks.py),
                        polyhaven/ (make_ground.py), nasa_black_marble/
  docs/                 the five documents above, this file, lore_refs/ (user's pictures)
  checks/               self-test outputs
  shots/                screenshots (phase 1 and the last round of phase 2)
  MECHANICS.md, mechanics.json   rules reference
  README.md, CREDITS.md          front page and asset credits
  target/               build output (made by cargo; never edit)
```

## Source files

| File | Lines | What it is |
|---|---|---|
| `main.rs` | 2474 | `Game` struct (all campaign state, plus the `dbg_*` debug settings), `Screen` enum, `Base`, `Site`, `Ufo`, `MissionSpec`; `Game::update` (1318) and `Game::draw` (1534, one big match on screen, situation map drawn here); daily/monthly ticks; argument parsing and main loop (2288–2456), which starts by setting the draw buffer size; `window_conf` (1920×1080, 4× MSAA, no vsync). |
| `data.rs` | 1256 | Tables and rules: `Rng` (seeded), item tables (surface/water), `WeaponDef`, `GameData::load` (decodes `earth_data.png` into land/biome/coast maps), `world_index` (318, turns latitude and longitude into a place in those maps; used by `terrain_at`, `coast_at`, `is_water`), soldiers (`Look`, `Stats`, `Soldier`, name lists), armour, `Month`/`Date` (starts 2027-01-01; `minute` is minute of day, UTC), hit chance, damage tables, facilities, craft, research, workshop, nations, borders, promotions. |
| `tactical.rs` | 4071 | The battle: simulation AND drawing in one file. See below. |
| `base.rs` | 1981 | Base screens (soldiers, facilities, research, workshop, market, fleet, finances, launch) and the soldier editor. 2D, uses `ui.rs`. |
| `dogfight.rs` | 798 | Interception minigame. |
| `globe.rs` | 463 | Situation-map globe: one full-screen GLSL 100 shader (ray-sphere), `Globe::project`, `screen_to_latlon`, zoom animation. Sun comes from `Globe.sun`, set every frame in `Game::draw` from the campaign clock (`sky::globe_sun_dir`); city lights from `assets/earth_lights.png`, brighter places switching on first as night deepens. A shader that fails to compile prints `globe shader failed` and falls back to a flat disc. |
| `ui.rs` | 386 | Colours, fonts, panels, buttons, `glow`, `grad_v`, `vignette`, `scanlines`, soldier `portrait`. |
| `audio.rs` | 424 | Every sound is generated in code (`gen_*` → WAV bytes). |
| `gfx.rs` | 1355 | The lighting pipeline. Used by `tactical::draw` since 2026-11-13 (`Game.gfx`). See below. |
| `examples/lightlab.rs` | 203 | The lighting test scene, a separate small program. Builds its own test site and drives `gfx.rs` and `sky.rs` directly. |
| `sky.rs` | 417 | Sun/moon position from date, time, place; sky-state blending; clock parsing. Used by the battle light and the globe sun. |

### tactical.rs in detail

Simulation (keep behaviour identical; `--self-test` guards it):
- 7–9: `TW = TH = 26` tiles, `TILE = 1.65` m. Tile height step is 0.62 m
  (written as a literal `0.62` in many places).
- 18 `Tile` (h, kind = ground biome, obstacle, obstacle_hp, smoke, fire,
  scorched, visible, explored). Obstacles: 1 rock, 2 tree, 3 stone block,
  4 wall with windows, 10 our craft's hull, 12 enemy hull, 13 enemy power
  plant, other = crystal.
- 31 `Unit`, 86 `Effect` (tracers, blasts), 118 `Battle` (everything about one
  fight, including camera: `cam_yaw`, `cam_zoom`, `cam_pan`, `view_level`).
- 211 `biome_ground(kind, x, y, seed)` ground colour per biome.
- Biome numbers. The world map (`assets/earth_data.png`) holds seven land
  biomes: 0 temperate, 1 farmland, 5 mountain, 6 rain forest, 7 desert,
  9 ice, 11 northern forest. The code also accepts 2 (drawn like 6) and 4
  and 8 (drawn like 7); any other number draws as temperate with its own
  ground colour. 13 is the sea floor used by the `water` demo.
- 237 `season_at(month, lat)`: 0 winter, 1 spring, 2 summer, 3 autumn; the
  tropics count as summer all year. 244 `weather_heavy` (true for downpour,
  blizzard, storm). 249 `snow_amount` (how much of the ground lies under
  snow; drawing only).
- 263 `compute_environment(spec, rng)` picks the weather (0 clear, 1 rain,
  2 snow, 3 dust, 4 underwater drift) from the biome, the season and the
  hour, on its own dice. The old weather picker still runs inside it, only
  so the mission's random numbers stay the same; its answer is thrown
  away. It also builds the header text (`weather_desc`: region, season or
  WET SEASON / DRY SEASON in the tropics, MORNING / MIDDAY / AFTERNOON /
  EVENING / NIGHT, the time, the weather name). The heavy forms are told
  apart only by that name. The old-style sun/ambient/fog values it returns
  are no longer used for lighting; `scene_light` and `gfx::light_at` do
  that.
- 377–1753 `impl Battle`: map generation, movement, line of sight, shooting,
  reaction fire, alien turn, inventory, and the Reach attacks. `Battle::new`
  (517) applies the debug flags `--site`, `--biome`, `--clock`, `--seed`,
  `--weather`.
- Inside it, the ones the planned work touches most: 383 `gen_map`, 630
  `compute_visibility`, 650 `los` (line of sight), 689 `walk_cost`, 810
  `check_unit_panic`, 825 `on_unit_death`, 865 `tick_smoke_and_fire`, 920
  `start_player_turn`, 975 `fire`, 1088 `check_end`, 1200 `explode`, 1354
  `throw_grenade`, 1400 `update_ai` (the enemy turn), 1502
  `opportunity_fire` (reaction fire), 1550 `handle_input`, 1697
  `update_selftest`.
- Reach on screen is "psi" in the code. The three attacks are 1127
  `execute_psi_probe` (shown as REACH READ), 1143 `execute_psi_panic` (REACH
  ROUT) and 1172 `execute_psi_control` (REACH TAKE). The battle button reads
  "REACH"; the key is still P (1592).
- 1767 `alien_def`: the creature table. One row per creature kind, 0–10 on
  land and 0–8 under water.
- 1826 `start`, 1832 `start_selftest`, 1859 `update`, 1893 `finalize`.

Drawing as of phase 2 (2026-11-13). The second list further down
describes the old code and is kept for the function names; what changed:
- `MB` now has three buffers. `lit` (unlit colour, real normals, material in
  `mb.mat`, ambient occlusion in alpha) is drawn in the shadow and scene
  passes; `over` gets anything with alpha < 0.99; `glow` gets anything
  whose `bx` boost is 1.5 or more (colour × `mb.glow_gain`). `finish()`
  returns `SceneMeshes`, split at 60,000 vertices (`MB_MAX`). `fan` adds a
  soft-edged see-through patch to `over` (used for blowing snow and dust).
- `shade` and `fogged` return the colour unchanged; `cast_shadow` does
  nothing; `sky()` is no longer called; the 2D weather and the grading
  rectangles (`draw_surface_fx`) are gone.
- `make_camera_view(b, view)`: 14° lens, 3.3× further back (`LENS_BACK`);
  `--view` picks wide/tele/low.
- `ground_site` (3339) gives each tile its colour and surface type from the
  biome, the season and where the tile lies. `ground_albedo`,
  `ground_average` and `ground_map` are built on it and feed
  `set_ground_map` and the 300 m apron.
- `scene_light` (3414) builds the `gfx::Light`: sun and sky from
  `gfx::light_at`, then cloud, haze and tint from the weather and biome,
  then `wet`, `snow`, `sand`, `rain`, `time` for the ground shader.
  Underwater is a hand-made look.
- `draw_weather` (2673), called at the end of `build_scene`: rain, snow and
  dust as real shapes in the air, in the `over` buffer, so they are lit and
  things in front hide them. Rain is streaks as long as a drop falls in
  1/30 s, plus small splashes; snow is flakes plus low drifting sheets in a
  blizzard; dust is drifting sheets plus hopping grains. `wind_dir` (2664)
  sets the direction. Puddles, lying snow and sand ripples are not here;
  the ground shader in `gfx.rs` draws them.
- `dress(biome)` (2471) picks rock, wall and roof colours per biome;
  `draw_prop` (2504) draws rocks, trees, blocks and walls.
- `draw` (3465): shadow → scene → overlay(0) `over` (smoke, markers,
  weather) → overlay(glow) `glow` + ring → overlay(tracers) → `finish` →
  2D glows, underwater drift, name tags → HUD. Glow brightness is held to
  `GLOW_PER_LIT` × the scene light so lamps do not burn white at night.
  The shadow map covers 34 m around the map centre. The lens is given
  `ev_bias` = snow share × 0.5, so snow scenes come out bright, not grey.
- Craft roofs are skipped while a living soldier of ours is under them.
- Walls use `mat::BUILT`; craft hulls use `mat::HULL` (the same look, but
  snow never settles on it); `mat::METAL` mirrors the sky and reads as pale
  plastic on big painted surfaces, so keep it for small parts.

Drawing (old description, line numbers brought up to date), from 1922:
- `MB` mesh builder: `quad`, `tri`, `bx(centre, size, yaw, colour, top_boost)`.
  Old: colours were final screen colours with shading baked in, in one
  `Mesh`. Now: see above. Indices are still 16-bit and everything is
  rebuilt every frame.
- `shade`, `fogged`, `cast_shadow` (flat blob shadows; all three now do
  nothing), `face_quad` (cliff sides).
- 2048 `make_camera` (calls `make_camera_view`, 2053): looks at a point
  between the selected soldier and map centre. Old lens was 44°; now 14°.
  `project`, `unproject`, `screen_to_tile` (mouse picking) all call it, so
  any camera change must happen here only.
- 2127 `sky`: 2D painted backdrop (gradient, sun disc, clouds, ridges).
  Still in the file, no longer called.
- 2221 `build_scene`: ground plane, tiles, grass blades, cliffs, scorch,
  obstacles, smoke puffs and fire (see-through boxes), blue move tiles,
  landing craft (`draw_landing_craft`, 2779), corpses, soldiers
  (`draw_soldier`, 2888), aliens (`draw_alien`, 3067), then `draw_weather`.
  The selection ring is `draw_selection_ring` (2454, `draw_line_3d`),
  called from `draw`.
- 3115 `draw_effects_3d` (tracer lines), 3142 `draw_effects` (2D glows),
  3182 `draw_unit_overlays` (health bars, names), 3222 `draw_underwater_fx`.
  `draw_surface_fx` (the old 2D rain/snow/dust) was removed in phase 2.
- 3465 `pub fn draw(g)`: order as listed above, then HUD (`draw_hud`, 3594),
  inventory (3722), the Reach panel (`draw_tactical_psi`, 3978).

### gfx.rs in detail (the lighting pipeline)

Frame order, exactly as `examples/lightlab.rs` does it:

```rust
let mut light = gfx::light_at(moment, lat, lon, cover, storm, ground_albedo);
light.wet = wet;                              // 0 dry .. 1 soaked
// the battle also sets light.snow, .sand, .rain, .time (see below)
gfx.shadow_begin(&light, centre, radius);     // draw everything that casts
gfx.scene_begin(&cam, &light);                // draw the same meshes again, lit
gfx.overlay_begin(&light, 0.0);               // see-through, lit, matte (smoke, markers)
gfx.overlay_begin(&light, 6.0);               // glowing (fire 6, tracer 20)
gfx.finish(dt, &Lens { .. });                 // exposure, bloom, lens → screen
// then HUD with ordinary macroquad 2D calls
```

- Lit vertices: `Vertex { position, uv: (0,0), color: [r,g,b, ao·255],
  normal: vec4(nx,ny,nz, material) }`. Colour is **unlit surface colour**
  (albedo), not a shaded colour. Alpha is ambient occlusion, not opacity.
- Materials (`gfx::mat`): GROUND 0, BUILT 1, FOLIAGE 2, METAL 3, SKIN 4,
  EMISSIVE 5 (colour is radiance), WATER 6, HULL 7 (craft skin: like BUILT,
  but snow never settles on it).
- Ground cover. `Light` carries `wet` (0 dry to 1 soaked), `snow` (share of
  the ground under snow, 0–1), `sand` (1 where open ground is sand, not
  grass), `rain` (1 while rain falls) and `time` (seconds). `scene_begin`
  passes the last four to the scene shader as one uniform (a value handed
  to a shader) called `surf` = (snow, sand, seconds, raining). The shader
  uses it to lay snow on ground, roofs and tree tops, ripple the sand, and
  put rain rings in puddles. `wet` travels in the `params` uniform.
- `Geo` builder: `quad(p, n, c, ao, m)`, `tri`, `boxy(lo, hi, c, m, ao)`
  (axis-aligned only), `blob`, `column`, `finish() -> Vec<Mesh>` (splits at
  60,000 vertices).
- Overlay pass: plain macroquad vertices/3D calls; vertex alpha is opacity.
- `set_ground_map(w, h, rgba, origin, size)`: one texel per ground patch;
  RGB = ground colour, A = type (0 open ground: grass, or sand when
  `light.sand` is 1; 85 ploughed; 168 crop rows; 255 packed earth, gravel,
  rock). The shader lays photographed grain over it.
- `Lens { vignette_stops, ca_px, ev_bias, adapt_secs, flash }`;
  `adapt_secs: 0.0` for screenshots so exposure does not drift. `ev_bias`
  makes the whole picture brighter or darker, in camera stops (+1 is twice
  as bright).
- Sky colours: `state_look` reads measured values from `assets/sky/looks.txt`
  for dawn…dusk; night, overcast and storm are hand-set in
  `state_look_hand` (1302) until Poly Haven HDRIs replace them.
- Limits: miniquad allows 12 texture slots, the scene material uses 7.
  Shadow map 4096 (`SHADOW_RES`) covering `radius` metres around `centre`
  (lab uses 36, the battle 34).
- Lab camera for the look: range 420 m, field of view 3.4–19.5°,
  `z_near = range − 160`, `z_far = range + 420`.

### sky.rs in detail

`Moment::new(year, month 1–12, day, minute_of_day_utc)`, `plus_seconds`,
`sun_at(lat, lon)`, `moon_at`, `moon_illumination`, `solar_minute(lon)`,
`parse_clock("2026-06-21 8am")`, `clock_text(minute)`, `globe_sun_dir(m)`
(sun direction in the globe shader's frame), `SkyState` (nine states) and
`clear_sky_blend` (which two states to mix, and how much).

## Couplings that bite

- Weapons are looked up by display name; creature armour class by name
  substring; two final missions by research id; four craft by blueprint id;
  stores keyed by item name. Rename these together (see `NAMES.md`).
- The words on screen and the words in the code no longer match for the
  things renamed on 2026-11-13. See "On screen / in code" below before
  searching the code for SPINE, GLOW or REACH.
- The land final site is still placed on Earth's globe
  (`make_final_mission`, `main.rs` 889, `find_land(10.0, 160.0)`). Only its
  label says the Moon.
- `MissionSpec.minute_of_day` is UTC; local solar time is derived with
  `lon / 15 * 60`. `Game.date` is the campaign clock.
- Simulation and drawing share `tactical.rs` and the `Battle` struct. 3.0
  needs them apart; do not add new drawing state to simulation code paths.
- `draw_line_3d` and other macroquad immediate 3D calls only work between
  `gfx.overlay_begin` and `gfx.finish`.
- `gl_set_drawcall_buffer_capacity(65_535, 400_000)` at the top of `main`
  must stay, and its first number must never go above 65,535 (see
  Commands). `examples/lightlab.rs` (line 144) sets the same two numbers.
- The heavy weather forms are not a separate number. `Battle.weather` is
  still 1, 2 or 3; `weather_heavy` looks for DOWNPOUR, BLIZZARD or STORM in
  `weather_desc`. Renaming a weather label can switch heavy weather on or
  off.
- `world_index` decides land or water and the biome for every site. A
  change there changes terrain game-wide.

## On screen / in code

On 2026-11-13 only the text the player reads was changed. The names inside
the code were left alone, on purpose: research ids and blueprint ids are
used as keys, and renaming fields touches hundreds of lines for no gain in
play. So the code still says lattice, electrolyte and psi. When you search
the code, use the right-hand column.

| On screen | In code |
|---|---|
| SPINE (material, item slot 72, research topic) | `Base.lattice` (`main.rs` 111); research id `"LATTICE"`; workshop cost field `ProductionBlueprint.structure` |
| SPINE CARRIER (armour) | research id and blueprint id `"LATTICE_CARRIER"`; `ARMORS[1]` |
| GLOW (material, item slot 73, research topic, craft fuel) | `Base.electrolyte` (`main.rs` 112); research id `"ELECTROLYTE"`; workshop cost field `ProductionBlueprint.power` |
| REACH (the stat, the battle button, log and panel text) | `psi_strength`, `psi_skill` on `Stats`, `Unit` and `AlienKind`; `Battle.psi_open`, `psi_target`, `psi_clicked`; `draw_tactical_psi`; sound `play_psi` / `gen_psi` |
| REACH READ, REACH ROUT, REACH TAKE | `execute_psi_probe`, `execute_psi_panic`, `execute_psi_control` |
| REACH DAMPER | `FacilityType::PsiDamper` |
| REACH WING | `FacilityType::PsiWing` |
| REACH STUDIES (research topic, category "Reach") | research id `"PSI_WING"` |
| BREACHER BATTERY | `FacilityType::DerivedBattery` |
| THE FARSIDE SIGNAL (research topic) | research id `"SEED_LOCATION"`; finishing it calls `make_final_mission` (`main.rs` 1189) |
| LUNAR FAR SIDE // FINAL ASSAULT (map label) | `SiteKind::Final`, built in `make_final_mission` (`main.rs` 889) |
| THE MOON IS SILENT (win screen) | `draw_end` (`main.rs` 2143) |

Three of these names are shared with the sea campaign, so they changed
there too: REACH DAMPER, REACH WING and BREACHER BATTERY are one table row
each for both campaigns, and the sea research topic with id
`"PSI_WING_DEEP"` now also reads REACH STUDIES. The sea materials (NACRE,
BRINE) and the rest of the sea tables were not renamed.

## Where the planned systems will go

This section is a plan. None of it is built. It lists, for each piece of
work decided in `LORE.md`, the files, structs and tables it will touch, and
the fixed-order limits to watch. "Struct" means a named bundle of values,
such as `Soldier`. "Flag" means a yes-or-no value. "Dice" means the game's
random numbers. Line numbers were checked against the code on 2026-11-13.

Item slot numbers, field names and table names given for new things are
suggestions for whoever builds them. They are not decisions. What is
decided is in `LORE.md`.

### How things stand today

- **There is no save game.** Nothing is written to disk except the
  self-test result (`tactical.rs` 1745–1747). So nothing here can break a
  save. Keep every new record small and plain (numbers and short lists), so
  the planned save slot and version 3.0 can store it.
- **`Soldier.missions` never goes up.** The soldier screens show it
  (`base.rs` 414, 439, 440, 1832) but it is always 0. The only counters
  that rise are for the whole campaign: `Game.missions_won` and
  `Game.water_missions` (`main.rs` 913–914).
- **`wounded_days` is never set above 0.** It is only counted down
  (`main.rs` 1304–1306). Wounded soldiers never actually rest.
- **Loot is flat.** Every land win gives +25 Spine and +10 Glow (`main.rs`
  965–966) and a generic "ALIEN CORPSE" (958). Nothing is read from the
  battlefield.
- **All enemies behave the same.** `update_ai` (`tactical.rs` 1400–1467)
  runs one routine for every creature kind.
- **There are two sides only.** `Side` is `Player` or `Alien` (`tactical.rs`
  15). There are no civilians.
- **Tiles hold no pool.** `Tile` (`tactical.rs` 18–28) has smoke and fire.
- **A mission has no kind and no gravity.** See `MissionSpec` (`main.rs`
  341–355). `gen_map` (`tactical.rs` 383) always builds the same sort of
  layout with one height layer.
- **Land maps have no water tiles.** Every tile gets the mission's biome as
  its `kind`. The extra walking cost for kind 9 (`walk_cost`, `tactical.rs`
  699) is the ice biome, not water.
- **Rain is a number, heavy rain is a word.** `Battle.weather` is 1 for
  rain. Heavy rain is only known from the word DOWNPOUR in `weather_desc`.

### Fixed-order limits to watch

These are the tables where position matters. Breaking one moves other
things without any error message.

| Table | Where | The limit |
|---|---|---|
| Land items `SURFACE_ITEMS` | `data.rs` 71–123 | 80 numbered slots (`item_table`, 58). A slot number never changes. Free land slots: 6, 7, 10, 11, 14, 15, 21, 27, 29–33, 40–49, 74–79. |
| Corpse and live items | same table | Corpse = slot 50 + creature kind. Live captive = slot 61 + creature kind. |
| Creature kinds | `alien_def`, `tactical.rs` 1767 | Land kinds 0–10. The number is stored in `Site.aliens` and `CapturedAlienSpec.kind`. Never renumber. |
| Weapon list | `data.rs` 241–281 | A list that can grow, found by name. Soldiers store a position in it, so add new rows at the very end (after the two NATURAL WEAPON rows). Position 0 is the fallback when a name is not found. |
| Damage table `DAMAGE_MODS` | `data.rs` 604–622 | 8 damage types down, 14 armour classes across. Keep every column. |
| Armour `ARMORS` | `data.rs` 480–487 | Exactly 6. 0–2 land, 3–5 dive. |
| Buildings `FACILITY_CATALOG` | `data.rs` 705–723 | Exactly 17. The count is written in the type, so adding a row means changing the number too. |
| Research `RESEARCH_SURFACE` | `data.rs` 843–864 | Exactly 20. Same rule. Ids are keys. |
| Workshop `WORKSHOP_BLUEPRINTS` | `data.rs` 903–919 | Exactly 14 (8 land, 6 sea). Same rule. |
| Self-test dice | `tactical.rs` 1697–1752, 1832–1857 | It uses soldier 0, a snap shot and one Tender (kind 0). Any new dice roll on that path changes its result. Then the stored results in `checks/` and `assets/selftest.json` must be re-made on purpose and compared. |

Who does what follows the roadmap table: Fable does design, lore and art.
Opus does rules and behaviour. Sonnet does table wiring, menus and keeping
the documents in step, checked by Opus.

### Package 1. Groundwork: honest records, and removing the Trestle

Goes with graphics phase 3 (lights). No art needed. Everything else
depends on 1a.

**1a. Mission records** (medium, Opus)
- `Soldier` (`data.rs` 442–464) gains a small `history` record: missions,
  times wounded, times panicked, friends lost, kills by creature kind.
- `MissionSpec` (`main.rs` 341–355) gains `kind` (normal, signpost, moon)
  and `gravity`. It is built by hand in three places: `main.rs` 755,
  `main.rs` 2376 and `start_selftest` (`tactical.rs` 1833). Each needs the
  new fields.
- `Game` (`main.rs` 357–450) gains a `story_flags` list and a
  `lost_soldiers` list (abducted, mind-taken, left on the field).
- `finalize` (`tactical.rs` 1893–1920) writes back who fought, who was
  wounded, who was left behind, and corpses per creature kind.
- `finish_battle` (`main.rs` 906–1031) adds 1 to `missions` and sets
  `wounded_days` from wounds.
- Limits: none. Self-test: unchanged if no new dice are rolled. Setting
  `wounded_days` changes who can deploy (`tactical.rs` 555), so later
  campaign missions play differently. That is intended.

**1b. Remove the Trestle** (small, Sonnet checked by Opus)
- Take 8 out of the top land pool `[3, 4, 4, 8, 9]` in `roll_alien`
  (`main.rs` 884). Leave the sea pool on line 881 alone; its 8 is a sea
  creature.
- Mark as RESERVED, do not delete: the kind 8 row in `alien_def`
  (`tactical.rs` 1793), item 58 "TRESTLE WRECK" (`data.rs` 107) and item 69
  "DISABLED TRESTLE" (`data.rs` 118).
- Leave armour class 11 (the "Mach" column of `DAMAGE_MODS`) in place. The
  Lathe may want it later.
- Limits: creature kinds and item slots. **Do not renumber.** Reserving is
  the whole fix.
- Other documents name the Trestle too (`MECHANICS.md`, `mechanics.json`,
  `NAMES.md`, `ART_BIBLE.md`). Change them in the same step.
- Self-test: not affected. It uses kind 0.

**1c. One behaviour hook per creature** (medium, Opus)
- A hook is one place in the code where each creature's own rule plugs in.
- `Unit` (`tactical.rs` 31–83) gains a small status block: dazed, marked,
  melt, buried, fullness, turns working.
- Split `update_ai` (1400–1467) so it calls one function per creature
  kind. At first every kind keeps today's behaviour.
- Replace the stand-in weapon at line 1440 (`g.data.weapons[0]`, the
  service pistol) with the creature's own natural weapon.
- Limits: the self-test must give exactly the same result file as before.
  That is the pass mark for this step.

### Package 2. SHOTGUN, KNIFE, SLAG and real loot

Goes with graphics phase 6 (weapons, items, materials). Glow pools go with
phase 3.

**2a. SHOTGUN** (small to medium; Opus for the rule, Sonnet for the rows)
- Item slot 6 (gun) and slot 7 (shells) in `SURFACE_ITEMS`. One new
  `add(...)` row at the end of the weapon list. Add to the starting stock
  in `Base::new` (`main.rs` 167–177).
- Rule in `fire` (`tactical.rs` 975–1048): several pellets, strong close
  up, weak far away. Either a pellet count on `WeaponDef` (`data.rs`
  187–198) or a new `Cat::Shotgun` (`data.rs` 184). The pellet count is the
  smaller change. A new `Cat` would also touch `best_weapon` (`main.rs`
  698–713) and the right-click fire code in `handle_input` (`tactical.rs`
  1638–1667).
- Limits: weapon list (add at the end), item slots 6 and 7. Self-test is
  safe if pellets are only rolled for shotguns.

**2b. KNIFE** (medium, Opus)
- Item slot 10. Model it on the DIVE KNIFE row (`data.rs` 145: damage 24,
  cut, 14% time units). One weapon row, `Cat::Melee`.
- Soldiers have no close-combat action today. Right-click only shoots
  (`tactical.rs` 1638–1667). Add one: the target must be next to the
  soldier, no ammunition, the knife is carried as a second weapon. That
  needs a `knife` flag or side-arm slot on `Soldier` and on `Unit`.
- Limits: weapon list, item slot 10. `best_weapon` already skips `Melee`,
  which is right.

**2c. SLAG** (small, Sonnet)
- New `slag` field on `Base`, beside `lattice` and `electrolyte` (`main.rs`
  111–112).
- New `slag` cost on `ProductionBlueprint` (`data.rs` 891–901). All 14 rows
  of `WORKSHOP_BLUEPRINTS` need the new value, zero for now.
- Screens in `base.rs`: workshop cost text, check and spend (1214–1226),
  stock readout (1286–1288), market sell list (1413–1423, 1464–1465).
- Limits: none for the material itself. Materials are fields, not item
  slots. Until the Maul rule exists (package 4), Slag can only be handed
  out by a debug flag.
- What Slag builds (`LORE.md`): the best heat and blast protection, and a
  late close-range magma weapon. Neither is designed yet. When they are: a
  new suit is a 7th row in `ARMORS`, which is exactly 6 today, and a new
  weapon goes at the end of the weapon list.

**2d. Loot from the battlefield** (medium, Opus)
- `finalize` hands back a loot record: pool tiles left at the end, corpses
  per kind, how full each Maul was.
- `finish_battle` stops giving the flat +25 and +10 (`main.rs` 965–966) and
  the generic corpse (958). Until pools exist, keep a base amount so the
  player does not run dry.
- Limits: uses the corpse slots 50–60 that already exist. The balance
  changes, so `MECHANICS.md` section 20 must be updated.
- Needs 1a.

### Package 3. Soldier traits, fears, earned traits, changing faces

Goes with graphics phase 4 (soldiers). Large. Fable writes the trait sheet
and the portrait stages, Opus the rules, Sonnet the soldier file screen.
Needs 1a: without it the mission count is always 0.

**3a. Trait data**
- New `TraitDef` table in `data.rs`, beside `RANKS` (466). Each row: name,
  strength or flaw, a trigger (when it shows itself), an effect.
- `Soldier` gains `strength`, `flaw`, `strength_seen`, `flaw_seen` and a
  list of `earned` traits.
- `make_soldier` (`main.rs` 674–686) draws one strength and one flaw.
- The eight starters: BRAVE, COLD, EMPATH, PROTECTIVE, RECKLESS, STEADY,
  SUPERSTITIOUS, LONER. Which are strengths and which are flaws is not
  decided.

**3b. Fears** (AFRAID OF WATER, AFRAID OF SHARKS, and more of the kind)
- A fear is a flaw that only bites where the thing is present. So each
  fear row is "a thing" plus "how the game can tell it is here". Two kinds
  of trigger cover all of them: **a place or condition**, and **a creature
  kind in sight**.
- AFRAID OF WATER, using what the code can already tell:
  - An underwater mission: `Battle.underwater` exists.
  - Rain: `Battle.weather == 1`. Heavy rain is only a word in
    `weather_desc`. Give the battle a proper "heavy" flag before a fear
    depends on it.
  - Standing next to water: **not possible today.** Land maps have no water
    tiles. This trigger waits until a map can hold water (the taker's
    pools in package 4 are the first liquid on a tile).
- AFRAID OF SHARKS is a "creature kind in sight" fear for a sea creature.
  The sea campaign is deferred, so this fear stays asleep on land. It needs
  no new sea design, only the trigger.
- Further fears use the same two triggers and are for the user to pick.
- Effect: a morale hit at the start of the soldier's turn while the thing
  is present, fed into `check_unit_panic` (`tactical.rs` 810–823).

**3c. Where the trait rules hook in**
- STEADY: `hit_chance` (`data.rs` 581–590), where wounds cut accuracy.
- BRAVE, COLD, EMPATH, LONER, SUPERSTITIOUS: the team morale loss in
  `on_unit_death` (`tactical.rs` 848–854) and `check_unit_panic`.
- PROTECTIVE and RECKLESS: `opportunity_fire` (`tactical.rs` 1502–1531) and
  `start_player_turn` (920–944).
- Hidden until shown: the first time a trigger fires, set the `seen` flag
  and write a log line. The soldier file (`base.rs` 414, 439–440, 1832)
  shows "UNKNOWN" until then.

**3d. Earned traits**
- A rule list in `data.rs`. Each rule is a test on the `history` record
  from 1a and the trait it grants.
- Checked once per soldier at the end of `finish_battle`, beside
  `evaluate_force_promotions` (`data.rs` 1171–1234; called at `main.rs`
  986).
- Each earned trait is a boost or a penalty, and is either permanent or
  wears off. So each one stored on a soldier needs a "missions left"
  number as well as its name.

**3e. Faces that change with service**
- `ui::portrait` (`ui.rs` 205) takes only a `Look`. Add a "service stage"
  number worked out from `missions`: 0 is new and jittery, 50 is the
  thousand-yard stare.
- Callers to update: `tactical.rs` 3605 and 3753; `base.rs` 410, 429, 1819.
- Keep the stage a plain number. No drawing state goes into the
  simulation.

**Limits for the package**
- No fixed table is touched.
- **Self-test at risk.** Drawing traits in `make_soldier` adds dice rolls
  and shifts every roll after it. Give traits their own set of dice,
  started from the soldier's id number, so the main run of dice does not
  move.

### Package 4. Creature rules

Goes with graphics phase 5 (creatures). The taker's fluid and the Cinder's
fires go with phase 3. Large overall; each creature is medium. Fable writes
a design sheet where `LORE.md` leaves gaps, Opus writes the rule, Sonnet
adds one self-test case per creature.

All of these use the hook and the status block from 1c. Build one creature
at a time, in this order: Bloat, Kite, Gorger, Maul, Cinder, Brooder, Lathe.
No new creature kinds are needed.

- **Bloat (kind 9).** Bursts into a thick cloud when shot or when it
  reaches a target. The cloud blocks sight (the Kite's too) and chokes.
  - Reuse the smoke part of `explode` (`tactical.rs` 1200–1352), called
    from `on_unit_death` (825–863). Sight already fails once smoke along
    the line adds up to 18 (`los`, 665–666).
  - Choking is new: stun each turn in `tick_smoke_and_fire` (865–918).
    Tag the cloud so ordinary smoke grenades do not choke.
- **Kite (kind 1).** Marks a soldier; its shot dazes.
  - Dazed: half time units and no reaction fire for one turn. Touches
    `fire`, `opportunity_fire` and `start_player_turn`.
  - Marked: a Lash may shoot a marked soldier it cannot see, for as long
    as the Kite can see that soldier. That is an exception in the sight
    test (`los`, 650) for the Lash only. Overhead cover, smoke above, or
    killing the Kite ends the mark.
- **Gorger (kind 7).** A buried ambush.
  - New `buried` flag. Hidden from the player until a soldier is next to
    it or it attacks. Note that `compute_visibility` (630) lights tiles,
    not units, so check how enemies on lit tiles are drawn first.
  - Its corpse (item slot 57) becomes refugee food in package 5.
- **Maul (kind 3).** An animal with no weapon.
  - Its table row (`tactical.rs` 1788) names "ARC TORCH". Replace that
    with a magma blast: heat damage (type 1) through `explode`.
  - It walks to corpses and dropped kit and eats them, and eats unconscious
    soldiers. Each meal raises `fullness`.
  - Its corpse gives Slag by fullness (ties to 2c and 2d).
  - It cannot be freed: block it in the capture step of `finalize`
    (1898–1906) or at the freeing step in package 6.
  - Research id `"STUDY_MAUL"` stays (`data.rs` 862).
- **Cinder (kind 10).** The Maul's young.
  - It is never spawned today: no pool in `roll_alien` holds a 10. By
    `LORE.md` it breaks off a full Maul or comes out of a Maul's remains.
    So it is added to the battle while it runs, not rolled at the start.
  - Fire does not spread at the moment. The Cinder sets fires directly
    through `Tile.fire`.
  - Growing into a Maul means changing `alien_kind` and the unit's numbers
    in the middle of a battle.
- **Brooder, the taker (kind 5).** A hovering machine that spits fluid.
  - `Tile` gains a `pool` amount. `Side` gains `Civilian`. `Battle::new`
    (`tactical.rs` 517–623) places civilians on missions that suit them.
  - A civilian melts in one spit and then runs at soldiers. Soldiers take
    several, because armour soaks it up.
  - Pools hurt units standing in them. Acid, damage type 7, is already a
    row in `DAMAGE_MODS`.
  - Neutralising spray (decided): item slot 14 plus one research row.
  - Its cube core as loot is a suggestion only, not decided. If wanted:
    item slot 15.
  - Its table row (1790) must change. Today it is a close-combat creature
    that does not float.
  - Adding `Side::Civilian` means re-reading every "other side" test:
    `check_end` (1088–1099), `opportunity_fire` (1511), the right-click
    target test (1642), and the two helpers at 625–628.
- **Lathe, the builder (kind 6).** Repairs, makes cover, and builds one new
  taker after several undisturbed turns beside a pool.
  - Uses `turns_working`. Heavily armoured while travelling, exposed while
    working: swap its armour values by state.
  - Its sphere core as loot is a suggestion only, not decided. If wanted:
    item slot 21.
  - The drawing code picks its shape by name: `"Lathe" || "Lamp"`
    (`tactical.rs` 3090). Remember this if the name ever changes.

**Limits for the package**
- Item slot 14, and slots 15 and 21 if the cores become loot.
- `RESEARCH_SURFACE` is exactly 20 rows. The spray's research row means
  changing that number.
- Self-test is safe as long as the Tender (kind 0) path is untouched.
- Order: Brooder before Lathe. Maul before Cinder. 2c before Slag drops.

### Package 5. MESS HALL, SCRUB ROOM, REFUGEE QUARTERS, infection tax

Goes with graphics phase 7 (base modules and base people). Medium. Opus
for the rules, Sonnet for the rows and screens, Fable for the icons.

- Three new `FacilityType` values (`data.rs` 665–683).
- `FACILITY_CATALOG` grows from 17 to 20 rows (705–723).
- `FacilityDef` (686–703) gains `mess_cap`, `scrub_cap`, `refugee_cap`.
  Every existing row needs the three new values.
- `MonthlyReport` (`data.rs` 1238–1254) gains food and scrubbing lines.
- `Base` gains an `infection` number.
- Capacity sums go beside the others (`main.rs` 304–330). Food use and
  scrubbing go in `daily_tick` (1138–1316). Costs go in the expenses part
  of `monthly_tick` (1077–1100).
- Infection tax: the more alien-derived equipment a base builds and uses,
  the higher its infection. The Scrub Room lowers it, at a cost in power
  and resources, and never to zero. Refugees add nothing: by `LORE.md` a
  freed alien is clean.
- The Mess Hall is also where stims and healing medicines are ordered once
  researched. That is a new order list on a base screen.
- New research rows are needed: the early stims and medicines topic, the
  scrubbing line, and "alien food" (opens after the research that lets us
  free captives).
- Limits:
  - Two lists in `base.rs` name every building: tile colours (587–598) and
    icons (610–801). The build fails until all three new ones are added to
    both. That is a useful safety net.
  - A third list (913–921) has a catch-all line, so new buildings quietly
    get the default colour there. Add them by hand.
  - The Barracks text already says "Bunks, mess and showers" (`data.rs`
    707). Reword it.
  - `RESEARCH_SURFACE` is exactly 20 rows. Each new research row means
    changing that number.
- Needs 2d for Gorger meat as food. Refugee Quarters is only useful with
  package 6.

### Package 6. Freed aliens ("refugees") as soldiers

Goes with graphics phases 4 and 5. Large. Opus, with Fable for the species
sheets. Needs packages 1a, 3 and 5.

- `Soldier` gains `species` (human, Regent, Tender, Lash).
- `CapturedAlienSpec` (`data.rs` 491–496) gains `freed`.
- A stat template per species.
- One new research row for the line that removes the control hardware
  (`RESEARCH_SURFACE` is exactly 20 rows; change the number).
- A "free" button on the Holding panel (`base.rs` 1038).
- Freeing moves a captive from `Base.captured_aliens` to `Game.soldiers`,
  if Refugee Quarters has room.
- `Battle::new` (`tactical.rs` 554–593) must build a player unit from a
  creature template.
- `draw_soldier` (2888) or `draw_alien` (3067) must be chosen by species,
  not by side.
- Limits: `ARMORS` is exactly 6, all cut for humans. Whether refugees wear
  armour at all is not decided (open question 6). A suit of their own
  would be a new row and a new count. Self-test is safe; its soldier is
  human.

### Package 7. Signpost missions and the human worker

Goes with graphics phase 8 (environment, civilians). Medium for the
framework, small per extra signpost. Opus for the framework, Fable for the
scripts and the sound design. Needs 1a. The civilian side from the Brooder
work helps.

- A `SignpostDef` table: id, trigger, mission script. It uses
  `story_flags`, `lost_soldiers` and `MissionSpec.kind` from 1a.
- Triggers are checked in `daily_tick` and `finish_battle`: first live
  capture, first soldier taken, first Regent studied, share of
  alien-derived kit past a threshold.
- "The same face" builds its enemies from the highest-ranked entry in
  `lost_soldiers`.
- The worker is a new unit that does not fight, with a `voice_stage`
  number. Its "duck calls" are generated sounds in `audio.rs` (new `gen_*`
  functions). They move closer to speech as `voice_stage` rises.
- Limits: `SiteKind` (`main.rs` 24) has values that are never used. Add
  `Signpost`; do not reuse those.

### Package 8. The Moon finale

Goes with graphics phase 9 (craft and the final sites). Large. Fable for
the ending and the maps, Opus for the rules. Needs package 6 (Tenders),
package 7 (the script system) and graphics phase 3 (dark-side light).

- `MissionSpec.gravity`, and a map source for the lunar surface.
- A height-map loader. `gen_map` makes one sort of layout with a single
  height layer, so real lunar maps need a new map path.
- A `dark_side` light flag for `scene_light`.
- `make_final_mission` (`main.rs` 889–904) still puts the site on Earth's
  globe. It needs a Moon target.
- Low gravity: longer throws (`throw_grenade`, `tactical.rs` 1354, range 12
  tiles today) and cheaper climbs in `walk_cost` (689).
- Victory cannot be "no living aliens" (`check_end`, 1088–1099), because
  the Registrar cannot be killed. It needs a scripted goal.
- The freed Tenders' vote and merge depends on how many Tenders the player
  freed.
- The last word, "there are many", goes on the end screen (`draw_end`,
  `main.rs` 2143). The win flags are set at `main.rs` 997–1003.
- Limits: the final mission is found by research id `"SEED_LOCATION"`
  (`main.rs` 1189). Keep that id. Self-test is safe if gravity defaults to
  normal.

### Package 9. Multiplayer: join or be wiped out

Version 3.0. A design note only for now. No code and no art yet. Large.
Opus for the split, Fable for the design.

- It needs the plain, save-ready records from packages 1, 3 and 6.
- It needs the simulation split from the drawing. Both live in
  `tactical.rs` and share the `Battle` struct today.

### Order

1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9. Packages 2 and 3 can run side by side
after 1. Package 5 can start once 2d is in.

### Still open (the user decides; ask through the question tool)

1. Which of the eight starter traits are strengths and which are flaws.
2. Which fears beyond water and sharks.
3. Maul numbers: Slag yield, eating speed, how the Registrar controls it.
4. What happens when the mess runs out, how many Gorgers a refugee needs,
   and whether the Gorgers must be dead or alive.
5. Infection tax and scrubbing numbers, and whether there is an attack
   from inside.
6. What each freed species can do.
7. The human worker's look and first appearance.
8. More signposts and their triggers.
9. What a joined multiplayer player controls.
