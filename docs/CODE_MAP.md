# Defending Earth — Code Map

Written 2026-11-13, 3pm, and checked against the code again later that day,
after graphics phase 2. For whoever (person or Claude session) picks the work
up next. Line numbers are as of that check and drift; find things again with
`grep -n "^fn \|^pub fn \|^pub struct " src/<file>.rs`.

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
  reaction fire, alien turn, inventory, psi. `Battle::new` (517) applies the
  debug flags `--site`, `--biome`, `--clock`, `--seed`, `--weather`.
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
  inventory (3722), psi (3978).

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
- `MissionSpec.minute_of_day` is UTC; local solar time is derived with
  `lon / 15 * 60`. `Game.date` is the campaign clock.
- Simulation and drawing share `tactical.rs` and the `Battle` struct. 3.0
  needs them apart; do not add new drawing state to simulation code paths.
- `draw_line_3d` and other macroquad immediate 3D calls only work between
  `gfx.overlay_begin` and `gfx.finish`.
- `gl_set_drawcall_buffer_capacity(65_535, 400_000)` at the top of `main`
  must stay, and its first number must never go above 65,535 (see
  Commands). `examples/lightlab.rs` still sets 200,000 there; do not copy
  that number into the game.
- The heavy weather forms are not a separate number. `Battle.weather` is
  still 1, 2 or 3; `weather_heavy` looks for DOWNPOUR, BLIZZARD or STORM in
  `weather_desc`. Renaming a weather label can switch heavy weather on or
  off.
- `world_index` decides land or water and the biome for every site. A
  change there changes terrain game-wide.
