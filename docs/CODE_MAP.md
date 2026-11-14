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
printed at exit), `--lights auto|on|off` (the soldiers' flashlights: `auto`
switches them on in the dark and is the default, `on` forces them on at any
hour, `off` forces them off and draws the battle as it was before
flashlights existed), `--fx <list>` (effects to show in a battle, separated
by commas: `flare` puts one burning hand flare on the ground four squares
from the middle of the map, toward the camera's starting side; `skyflare`
hangs one sky flare over the middle of the map, 45 m up and sinking; later
jobs add more names). `--biome`, `--weather` and `--fx` do nothing under
water, and `--site` there only moves the place.

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
  source_assets/        originals and bake scripts: blender/ (sky/make_looks.py,
                        creatures/<name>/ first models: build.py, render.py,
                        model.blend, sheet.md, rounds.md, size.json,
                        renders/contact.png; VERDICTS.md), scale/ (the scale
                        sheet: scale_sheet.json, scale_lib.py, size_chart.py),
                        polyhaven/ (make_ground.py), nasa_black_marble/
  docs/                 the five documents above, this file, HANDOFF.md (what
                        the next session does), briefs/ (the design job briefs),
                        lore_refs/ (user's pictures)
  reference/            the owner's creature reference pictures
  checks/               self-test outputs
  shots/                screenshots (phase 1 and the last round of phase 2)
  MECHANICS.md, mechanics.json   rules reference
  README.md, CREDITS.md          front page and asset credits
  target/               build output (made by cargo; never edit)
```

## Source files

| File | Lines | What it is |
|---|---|---|
| `main.rs` | 2508 | `Game` struct (all campaign state, plus the `dbg_*` debug settings), `Screen` enum, `Base`, `Site`, `Ufo`, `MissionSpec`; `Game::update` (1318) and `Game::draw` (1534, one big match on screen, situation map drawn here); daily/monthly ticks; argument parsing and main loop (2288–2456), which starts by setting the draw buffer size; `window_conf` (1920×1080, 4× MSAA, no vsync). |
| `dice.rs` | 177 | Rule dice. `Dice` logs every draw (`Roll`: who, what, odds, number, passed). `check` is a percent check on 0..=99. `range` and `index` are logged picks. `generator` hands out a plain `Rng` for building things (map, weather, faces, names) and logs one draw for it. `for_battle(campaign_seed, battle_number)` gives each battle its own dice. Has unit tests. A battle replays only from the same campaign seed, battle number, mission spec and roster: the campaign dice are drawn on clock and click timing, so the roster itself can differ between runs with the same `--seed`. |
| `data.rs` | 1257 | Tables and rules: `Rng` (seeded), item tables (surface/water), `WeaponDef`, `GameData::load` (decodes `earth_data.png` into land/biome/coast maps), `world_index` (318, turns latitude and longitude into a place in those maps; used by `terrain_at`, `coast_at`, `is_water`), soldiers (`Look`, `Stats`, `Soldier`, name lists), armour, `Month`/`Date` (starts 2027-01-01; `minute` is minute of day, UTC), hit chance, damage tables, facilities, craft, research, workshop, nations, borders, promotions. |
| `tactical.rs` | 5109 | The battle: simulation AND drawing in one file. See below. |
| `base.rs` | 1984 | Base screens (soldiers, facilities, research, workshop, market, fleet, finances, launch) and the soldier editor. 2D, uses `ui.rs`. |
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
- 380–766: shot odds, line of fire, and the shot rules' free functions and
  event types (see "Shot odds" and "Shot rules" below).
- 768–2428 `impl Battle`: map generation, movement, line of sight, shooting,
  reaction fire, alien turn, inventory, and the Reach attacks. `Battle::new`
  (908) applies the debug flags `--site`, `--biome`, `--clock`, `--seed`,
  `--weather`.
- Inside it, the ones the planned work touches most: 774 `gen_map`, 1029
  `compute_visibility`, 1050 `los` (line of sight), 1133 `walk_cost`, 1254
  `check_unit_panic`, 1271 `on_unit_death` (1304 `kill_unit` is its rule
  part), 1325 `tick_smoke_and_fire`, 1380 `start_player_turn`, 1437 `fire`
  (1451 `resolve_shot`, 1678 `play_shot`), 1773 `check_end`, 1887 `explode`,
  2027 `throw_grenade`, 2073 `update_ai` (the enemy turn), 2177
  `opportunity_fire` (reaction fire), 2225 `handle_input`, 2372
  `update_selftest`.
- Reach on screen is "psi" in the code. The three attacks are 1812
  `execute_psi_probe` (shown as REACH READ), 1828 `execute_psi_panic` (REACH
  ROUT) and 1857 `execute_psi_control` (REACH TAKE). The battle button reads
  "REACH"; the key is still P (2267).
- 2442 `alien_def`: the creature table. One row per creature kind, 0–10 on
  land and 0–8 under water.
- 2501 `start`, 2507 `start_selftest`, 2534 `update`, 2568 `finalize`.

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
gfx.set_lamps(&lamps, focus);                 // battle only: this frame's moving lights
gfx.set_air(scatter, Some(air_map));          // battle only: how thick the air is, for beams
gfx.shadow_begin(&light, centre, radius);     // draw everything that casts
gfx.lamp_shadows(|| draw_casters());          // battle only: same meshes, once per shadow lamp
gfx.scene_begin(&cam, &light);                // draw the same meshes again, lit
gfx.overlay_begin(&light, 0.0);               // see-through, lit, matte (smoke, markers)
gfx.beams(&light, any_smoke);                 // battle only: lamp beams in rain, dust, smoke
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
- Moving lights. A `Lamp` is one small light that moves: a flashlight
  now, later a fire or a muzzle flash. (`Light` already means the sun and
  sky, so the small ones are called lamps.) It holds `pos`, `dir`,
  `colour`, `strength` (how bright a white surface facing it is at 1 m),
  `reach` (metres; the light fades smoothly to nothing there), `inner` and
  `outer` (the cone's two angles, in radians: full brightness inside the
  first, nothing outside the second), `start` (metres in front of the lamp
  where the light begins), `shadows` and `beam` (switches). `Lamp::cone`
  makes a cone light, `Lamp::point` one that shines every way.
  `Lamp::falloff(d, angle)` is the same dimming rule the shader uses.
- `set_lamps(&lamps, focus)` keeps at most 24 (`MAX_LAMPS`), the ones
  nearest `focus` (the point the camera looks at). The 4 nearest cone
  lamps (`MAX_LAMP_SHADOWS`) cast shadows. `lamp_shadows(draw)` draws the
  casters once for each of them into one 2048×2048 depth picture, a
  1024×1024 quarter each. With no lamps it does nothing and costs nothing.
- The scene shader adds every lamp's light before exposure: it dims with
  the square of the distance, with the angle the light strikes the surface
  at, and it puts a soft highlight on wet ground. With no lamps the shader
  skips all of it.
- `set_air(scatter, map)` and `beams(&light, smoke)`: a lamp's beam shows
  as a soft cone only where something is in the air. `scatter` is the
  weather's thickness per metre (0 in clear air). `map` is one texel per
  tile: R = smoke, G = ground height (0 to 8 m). `beams` draws nothing
  when the air is clear and there is no smoke.
- The battle's flashlights are in `tactical.rs` under "flashlights":
  the numbers (`FLASH_*`), `is_dark`, `flashlight_on`, `flashlights`.
  Nothing there is read by a rule.
- Flares (drawing only, 2026-11-13). Two additions, both off unless asked
  for, so a frame without flares is drawn exactly as before:
  - `set_lamps_round(&lamps, focus, round)` is `set_lamps` with one more
    answer: `round` names one point lamp that casts shadows all round (a
    hand flare on the ground). It takes all four lamp depth pictures, one
    for each sideways quarter (+x, +z, −x, −z), so while it burns the
    flashlights cast no shadows. Things steeply above or below it (more
    than 45°) are not shadowed. A point lamp with `beam` set also glows in
    rain, snow, dust and smoke (`beams` draws a box round its reach).
  - `SkyLamp { pos, colour, strength, shadows }` and `set_sky_lamp(Some(..))`
    (call before `shadow_begin`): one light with no reach limit, for a
    flare hanging over the site. It dims with the square of the distance.
    With `shadows` set, `shadow_begin` draws the big shadow picture from
    the flare (a view that widens downward, not the sun's straight one),
    the flare's shadows stay hard, and moonlight falls unshadowed. The
    see-through pass (`overlay_begin(.., 0.0)`) adds the flare's light too.
  - The battle's side is in `tactical.rs` under "flares": every number
    (`FLARE_*` the hand flare, `SKY_*` the sky flare, `FX_*`), the lists
    `hand_flares` and `sky_flare` (only the `--fx` debug flares so far; a
    later rules job fills them), `draw_flare_solids` (body, white-hot spot,
    scorch, parachute; called from `build_scene`) and `flare_fx` (sparks and
    lit smoke, drawn in `draw` after the beams in a pass of their own at
    `FX_LEVEL`). Nothing there is read by a rule.
  - **Tuned by eye by the lead (2026-11-13); do not change without
    looking at a picture.** `FLASH_POOL` 0.07 (a strength from a brief
    must be divided by about 35 for this camera). Hand flare:
    `FLARE_LIGHT_UP` 0.8, `FLARE_COLOUR` (1.0, 0.13, 0.09), `FLARE_GROUND`
    0.5. Sky flare: `SKY_SHARE` 1/40, and `SKY_NIGHT_STOPS` -1.5, which
    holds the camera back while a sky flare lights the night (added to
    the lens `ev_bias` in the battle draw); without it the auto-exposure
    turns the flare-lit night into a dull day and burns the flashlights
    white.
- Limits: miniquad allows 12 texture slots, the scene material uses 8.
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

It was revised the same day for the third lore session (`LORE.md`, "Third
session (2026-11-13): the open questions"). That session changed packages
3 to 8 and added package 10, the soldier pool. Nothing from it is in the
code yet.

Item slot numbers, field names and table names given for new things are
suggestions for whoever builds them. They are not decisions. The same goes
for any rule here that says *how* a decided thing works in the code (how a
fear is tested, how a spike is hit, how food is counted). What is decided
is in `LORE.md`. Where this section goes further, it says "suggested" or
"not decided".

### How things stand today

- **There is no save game.** Nothing is written to disk except the
  self-test result (`tactical.rs` 2422), screenshots (`main.rs`
  2450–2451) and the `--dump-map` picture (`main.rs` 2413). So nothing here
  can break a save. Keep every new record small and plain (numbers and
  short lists), so the planned save slot and version 3.0 can store it. The
  soldier pool (package 10) will be the first real player file.
- **Nothing takes typed text.** The soldier editor (`base.rs` 1807–1958)
  only has "RANDOMIZE NAME" (1847).
- **A soldier can only be lost by dying.** Stun at or above full health
  counts as death (`hit_unit`, `tactical.rs` 1652, then `kill_unit`
  1304–1323), and `finalize` marks every fallen soldier dead (2586). There is no withdraw action. The enemy's
  Reach can only rout a soldier, never take one (`tactical.rs` 1424–1436).
  Nobody is abducted. So "a soldier the Registrar has read" cannot happen
  yet.
- **There is no map of the inside of a base.** `SiteKind::BaseDefense`
  exists (`main.rs` 24) and is never used.
- **The event screen is never fed.** `draw_event` (`main.rs` 2123–2141)
  reads `event_title` and `event_body`, but nothing sets them.
- **HANDHELD RADAR does nothing in battle.** It is item slot 25 (`data.rs`
  90) and is sold in the market (`base.rs` 1336).
- **Live study kills the captive** (the log line at `main.rs` 1178).
- **The final site also appears after 8 wins**, with no research (`main.rs`
  1007–1008).
- **`Soldier.missions` never goes up.** The soldier screens show it
  (`base.rs` 414, 439, 440, 1832) but it is always 0. The only counters
  that rise are for the whole campaign: `Game.missions_won` and
  `Game.water_missions` (`main.rs` 913–914).
- **`wounded_days` is never set above 0.** It is only counted down
  (`main.rs` 1304–1306). Wounded soldiers never actually rest.
- **Loot is flat.** Every land win gives +25 Spine and +10 Glow (`main.rs`
  965–966) and a generic "ALIEN CORPSE" (958). Nothing is read from the
  battlefield.
- **All enemies behave the same.** `update_ai` (`tactical.rs` 2073–2142)
  runs one routine for every creature kind.
- **There are two sides only.** `Side` is `Player` or `Alien` (`tactical.rs`
  15). There are no civilians and no wild creatures. The side also decides
  the shape that is drawn (`tactical.rs` 2447–2448), and a Reach-taken unit
  always goes back to `Alien` (951–953).
- **Morale climbs 15 every turn** (`tactical.rs` 934), and panic is only
  tested below 50 (812).
- **Tiles hold no pool.** `Tile` (`tactical.rs` 18–28) has smoke and fire.
- **A mission has no kind and no gravity.** See `MissionSpec` (`main.rs`
  341–355). `gen_map` (`tactical.rs` 774) always builds the same sort of
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
| Creature kinds | `alien_def`, `tactical.rs` 2442 | Land kinds 0–10. The number is stored in `Site.aliens` and `CapturedAlienSpec.kind`. Never renumber. The human worker (package 7) would be kind 11, added at the end. Its corpse and captive slots would then be 61 and 72. Both are taken: 61 is kind 0's live captive and 72 is SPINE. So the worker needs slots of its own, or none. |
| Weapon list | `data.rs` 241–281 | A list that can grow, found by name. Soldiers store a position in it, so add new rows at the very end (after the two NATURAL WEAPON rows). Position 0 is the fallback when a name is not found. |
| Damage table `DAMAGE_MODS` | `data.rs` 604–622 | 8 damage types down, 14 armour classes across. Keep every column. |
| Armour `ARMORS` | `data.rs` 480–487 | Exactly 6. 0–2 land, 3–5 dive. The three refugee harnesses (package 6) go at the end and change the count. |
| Buildings `FACILITY_CATALOG` | `data.rs` 705–723 | Exactly 17. The count is written in the type, so adding a row means changing the number too. |
| Research `RESEARCH_SURFACE` | `data.rs` 843–864 | Exactly 20. Same rule. Ids are keys. |
| Workshop `WORKSHOP_BLUEPRINTS` | `data.rs` 903–919 | Exactly 14 (8 land, 6 sea). Same rule. The three harnesses add three rows. |
| Player files | none yet | The soldier pool lives in the player's data folder, outside the repository. Its names and slider values must never be committed. See package 10. |
| Self-test dice | `tactical.rs` 2372–2427, 2507–2532 | It uses soldier 0, a snap shot and one Tender (kind 0). Any new dice roll on that path changes its result. The 2026-11-13 dice split changed the self-test's map and rolls once, on purpose. The self-test was run after the split and all three steps passed. Shot range and cover (same day) can change its shot odds if the Tender is far or behind cover; re-run it and compare. The shot rules slice (same day) adds rolls on every miss (stray angle, rolls for things a stray passes) and changes the damage roll to 50 to 150 percent, so the self-test's hits and rolls change. It has not been re-run since. |

Who does what follows the roadmap table: Fable does design, lore and art.
Opus does rules and behaviour. Sonnet does table wiring, menus and keeping
the documents in step, checked by Opus.

### Package 0. The test bench (decided 2026-11-13)

Not built. What follows was checked by searching `src/` on 2026-11-13.
- **Random numbers.** Three kinds, since 2026-11-13. `Rng` (`data.rs`) is the
  plain generator. Rule dice are `Dice` (`dice.rs`): `Game.dice` for the
  campaign and `Battle.dice` for one battle; every draw is logged. Visual
  dice are `Game.fx` and `Battle.fx`, plain `Rng`s seeded from the clock and
  never logged. `Game.campaign_seed` is set in `new_campaign` (from the
  clock, or `--seed N`). `Battle::new` adds one to `battles_started` and
  builds `Dice::for_battle(campaign_seed, battles_started)`, so the same
  seed and the same battle number give the same battle. Map, weather and
  alien looks use child generators from `dice.generator(..)`. When a battle
  ends, `finalize` moves its log into `Game.last_roll_log` (for a debrief
  screen that is not built). The only other randomness is the particle
  effects in `dogfight.rs`, which use macroquad's own `rand::gen_range`, and
  small fixed-seed generators for sounds (`audio.rs`) and face details
  (`ui.rs`). `roll_damage` (`data.rs`) now takes `&mut Dice`.
- **Save and load.** No save or load of game state exists. The only files
  the code writes are the self-test result (`tactical.rs` 2422) and
  screenshots and the world-map picture (`main.rs` 2413, 2450–2451).
  `serde` and `serde_json` are in `Cargo.toml`. The `load` functions found
  load fonts, sounds and shaders, not saves.
- **Rules and drawing.** Shooting and damage are cut free of the window
  (2026-11-13, see "Shot rules" below): `resolve_shot`, `hit_unit`,
  `kill_unit` and their helpers take `&GameData`, not `Game`, and are tested
  with no window. Other battle methods still take `g: &mut Game`, and `Game`
  also holds the interface and sound (`main.rs` 462).

**What the review found.** Line numbers are as of 2026-11-13.
- (Fixed 2026-11-13) The one dice stream and visual dice on the rule stream
  were split; see Random numbers above.
- (Fixed 2026-11-13) Shown odds differed from rolled odds. See "Shot odds"
  below.
- About 25 battle rules still take the whole `Game` (`src/tactical.rs` 1254-2428). Shooting and damage no longer do (2026-11-13): `resolve_shot` and everything it calls take `&GameData`. Still on `Game`: `fire`, `play_shot`, `on_unit_death` and `damage_unit` (thin wrappers that add sound and picture), `explode` and the grenades (slice 5 seam), movement, the alien turn, the Reach attacks, input and the self-test.
- The alien turn runs on frame time (`src/tactical.rs` 2073-2142).
- `Game` holds about 90 fields, of which roughly 25 are campaign state
  (`src/main.rs` 357-450).
- Campaign actions sit inside button clicks in the `src/base.rs` draw
  functions.
- The battle result passes as seven values (`src/tactical.rs` 2568-2600,
  `src/main.rs` 906).

### Shot odds (built 2026-11-13; range and cover added 2026-11-13)

One rule gives the number shown and the number rolled. It lives in
`tactical.rs`:
- `ShotOdds` holds `percent` (final, 0 to 100), `blocked`, and the parts:
  `base`, `height`, `range`, `cover`.
- `Battle::shot_odds(data, shooter, target, mode)` is the only place odds are
  worked out. It uses no dice and changes nothing.
- `Battle::aim_odds` is private. It is only the helper behind `base`.
  Nothing on screen shows it. `hit_chance` (`data.rs`) sits behind it.
- `combine_odds` adds `base + height + range + cover` and clamps to 0..=100.
  Nothing else limits the number. Height is +10 higher, -10 lower.
- Range: `comfortable_range(cat)`, `range_penalty(cat, squares)`. Starting
  numbers (owner, 2026-11-13; the test bench tunes them): Pistol 8, Rifle 16,
  Cannon and Heavy 14, Launcher 18 squares; minus 2 per square past it.
  Distance is the straight-line distance between the two squares, rounded.
  Grenade, Melee and Heal have no range effect (to settle with the owner).
  Constants are named `RANGE_*` and `COVER_*` above `comfortable_range`.
- Cover: `cover_penalty(light, heavy)`. Light is minus 10 each (a square with
  any smoke; a crystal obstacle). Heavy is minus 25 each (rock, stone block,
  a living unit of the shooter's own side on the line).
- One walk of the line: `Battle::line_of_fire(x0, y0, x1, y1, shooter_side)`
  gives `LineOfFire { blocked, light, heavy }`. It calls the free function
  `walk_line`, which takes a closure for "what is in this square". `los` is
  now `!line_of_fire(.., None).blocked`; sight gives the same answers as
  before. Blocking is unchanged: trees, walls, hulls, power plant, ground two
  levels up, and smoke adding to 18 or more.
- The start square, the end square and the first square after the start give
  no cover. The first square still blocks as before.
- Enemy and neutral units on the line have no effect (to settle with the
  owner).
- `line_of_fire` also lists the cover things in order from the shooter
  outward (`LineOfFire.cover`: kind and square; a friend carries its unit
  index). Within one square the order is obstacle, friend, smoke (to settle
  with the owner). `shot_detail` returns the odds and that list;
  `shot_odds` is `shot_detail(..).0`.
- A miss can hit things near its path. See "Shot rules" below.
- `fire` refuses a blocked shot (log line NO LINE OF FIRE.) before it spends
  time units, then rolls `dice.check_roll(.., "shot", percent)` each round.
  Aliens and reaction fire check `los` before calling it.
- Screen: the hover label and the target panel show `shot_odds`. The three
  fire-mode buttons show only the TU cost until an alien is under the
  pointer, then "N%" (Auto: "N% x3") and the cost.
- Unit tests are in the `odds_tests` module at the end of `tactical.rs`. They
  cover the range and cover arithmetic, the clamp, and `walk_line` with
  closures. The shot rules tests (below) also run `line_of_fire` and
  `shot_detail` on real tiles and units, through a small battle built in the
  test (`open_battle`) with `GameData::load()` and no window.

### Shot rules (slice 1 of the battle rules, built 2026-11-13)

The owner's decisions of the ninth session (R1 to R12, `LORE.md`). Numbers
are starting numbers; the test bench tunes them. Items marked "to confirm"
are the lead's reading and wait for the owner (T1 to T5).

The seam:
- `fire(g, shooter, target, mode)` is now three steps: `resolve_shot`
  (rules), `play_shot` (picture and sound), then sight and the end check.
  It returns true when any round hit the intended target, as before. It
  also refreshes the reachable squares when a shot broke scenery.
- `resolve_shot(&GameData, shooter, target, mode) -> Vec<ShotEvent>` holds
  all the rules. It changes rule state (time units, ammo, health, armour,
  morale, wounds, obstacles, counters, incidents) and draws only from
  `Battle.dice`. It never touches `Game`, sound, `effects`, `fx`, the battle
  log or screen flashes.
- `play_shot(g, &[ShotEvent])` makes the sound, tracers, flashes, floating
  numbers and log lines from the events. It never rolls rule dice and never
  changes rule state. It uses `fx` only for debris. It also plays the
  events of `kill_unit` and `hit_unit` for other callers.
- `ShotEvent` (`Refused`, `Fired`, `Round`, `PassRoll`, `UnitHit`,
  `SceneryHit`, `FriendlyMorale`, `Died`) holds everything the picture needs.
  A `Round` carries its odds, `p0` (odds before cover), roll, outcome, the
  square the tracer ends on, and what it struck. The details follow it as
  their own events, so a replay or a text bench can describe a shot from
  events alone.
- Rule parts with sound split off: `kill_unit` (rule part of
  `on_unit_death`) and `hit_unit` (rule part of `damage_unit`, which is now
  only alien melee). `on_unit_death(g, ..)` and `damage_unit(g, ..)` are
  thin wrappers: rule, play, end check.
- Slice 5 seam: `explode` still mixes rules with sound and picture. It now
  shares `damage_obstacle`, `debris` and `on_unit_death`.

One roll per round (R5): `p0 = combine_odds(base, height, range, 0)` and
`percent` the final odds. `round_outcome(roll, percent, p0, points)`:
- roll < percent: a hit.
- percent <= roll < p0: the cover caused the miss. That band is shared out
  between the cover things in order from the shooter, each taking its own
  points (10 or 25), cut off at p0. The thing whose share holds the roll is
  struck: a rock, stone block or crystal takes the damage roll; a friend
  takes a full hit. Smoke stops nothing: a roll in its share is an ordinary
  stray.
- roll >= p0: an ordinary stray.

Strays (R2, R3, R4, T1, T2, T5), in `fly_stray`:
- The miss distance is today's scatter formula, in squares
  (`stray_miss_squares`). The side comes from a logged "stray angle" roll,
  0 to 359, on the battle dice. The aim point is `stray_aim`.
- `stray_squares` walks from the shooter through the aim point to the
  weapon's comfortable range (pistol 8, rifle 16, cannon 14, heavy 14,
  launcher 18), clipped to the map. It leaves out the shooter's and the
  target's squares. It steps with `LineSteps`, the same stepping `walk_line`
  uses.
- On the way: a full blocker (tree, wall, hulls, power plant) stops it and
  is struck. Ground two levels up stops it. A living unit: "stray hits unit"
  roll, 50 standing, 25 kneeling (R4). A rock or stone block 25, a crystal
  10 (T1, to confirm). Each roll is logged and is a `PassRoll` event.
- Every square on the way is rolled for, the first square after the
  shooter too (R4). The cover rule's first-square exemption does not apply
  to strays.
- Never the target, never the shooter (T2, to confirm). Weapons with no
  comfortable range make no stray (T5, to confirm): the round ends at the
  target's square and strikes nothing.
- The tracer ends on the square the rule says the round ended: a struck
  unit's chest, a struck square, or the last square of a spent stray.
- Smoke in a square is not struck (R5). But smoke adding up to 18 or more
  along the stray's squares, counted from the shooter, fully blocks it, as
  in `walk_line` (R3). The stray ends there and strikes nothing.

Damage (R6, R7, R8): `hit_unit` is the one damage rule for aimed hits,
cover hits on a friend, strays and alien melee. `roll_damage` (`data.rs`)
is one logged roll, 50 to 150 percent of the weapon's power, for every hit,
explosions too. Scenery struck takes the same roll off its hit points
(`strike_scenery`, `damage_obstacle`): a wall at 40 or less falls to low
cover, at 0 or less any obstacle is cleared, as in `explode`. Hulls and the
power plant cannot be broken by shots (`obstacle_shot_breakable`). Trees
(40) and walls (80) already had hit points in `gen_map`, so T3 needed no new
numbers.

Kills and morale (R9, R10):
- `kill_unit` gives the kill and the killer's morale gain only when the
  killer is on the other side. A stray kill of an enemy counts in full.
- `death_morale_loss` is the death arithmetic, unchanged. A unit hit by its
  own side and alive: its side loses `friendly_fire_loss` of it, 25 percent
  if unhurt, 50 percent if hurt (constants `FF_MORALE_*`). Killed: the full
  death loss, through `kill_unit`. Same set of units as a death (T4, to
  confirm): every other living unit of that side, no sight test. Aliens the
  same.

Incidents (R11): `Battle.incidents` keeps every friendly-fire hit (shooter,
victim, names, soldier numbers, turn, unhurt/hurt/killed) and every stray
kill of an enemy. `finalize` moves them to `Game.last_incidents`, beside
`last_roll_log`. Nothing reads them yet.

Other behaviour that changed with it:
- Grenades and launcher blasts also stop giving a kill for one's own side.
- Every hit, not just blasts, rolls 50 to 150 percent; incendiary and stun
  blasts too.
- The log lines of a death and the shot refusals now come from
  `play_shot`; the words are unchanged.

Tests (`odds_tests`, no window): the outcome split at its edges, the stray
walk (reach, map edge, skipped squares), the stray numbers, the damage range,
the morale fractions, friendly-fire morale and no credit, stray kill credit,
refusals spending nothing, a stray breaking a tree and not a hull, and 600
shots checking that the one roll and the tracer line agree. Not tested: the
picture and sound (`play_shot`) and the self-test, which need a window.

### Package 1. Groundwork: honest records, and removing the Trestle

Goes with graphics phase 3 (lights). No art needed. Everything else
depends on 1a.

**1a. Mission records** (medium, Opus)
- `Soldier` (`data.rs` 442–464) gains a small `history` record: missions,
  times wounded, times panicked, friends lost, kills by creature kind.
- `MissionSpec` (`main.rs` 341–355) gains `kind` (normal, signpost, moon)
  and `gravity`. It is built by hand in three places: `main.rs` 755,
  `main.rs` 2376 and `start_selftest` (`tactical.rs` 2507). Each needs the
  new fields.
- `Game` (`main.rs` 357–450) gains a `story_flags` list and a
  `lost_soldiers` list (abducted, mind-taken, left on the field). Each
  `LostSoldier` keeps name, look, stats, rank and `pool_id` (package 10),
  so the human worker can be built from it.
- `finalize` (`tactical.rs` 2568–2600) writes back who fought, who was
  wounded, who was left behind, and corpses per creature kind.
- `finish_battle` (`main.rs` 906–1031) adds 1 to `missions` and sets
  `wounded_days` from wounds.
- Limits: none. Self-test: unchanged if no new dice are rolled. Setting
  `wounded_days` changes who can deploy (`tactical.rs` 555), so later
  campaign missions play differently. That is intended.

**1a-2. Ways to lose a soldier alive** (new; medium, Opus)

Without this the human worker (package 7) never appears, because nobody
can ever be "read".

- A withdraw action: the squad leaves, and whoever is not aboard stays.
- A soldier can be unconscious without being dead. Today stun at or above
  full health kills the unit (`hit_unit`, `tactical.rs` 1652, then
  `kill_unit` 1304–1323).
- An enemy Reach take, or an abduction. Today the enemy only routs
  (`tactical.rs` 1424–1436).
- `finalize` (`tactical.rs` 2568) writes "left on the field" and "taken"
  into `lost_soldiers`, not `dead`.
- Limits: none. Self-test: must give the same result file.
- Needs 1a.

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
- Split `update_ai` (2073–2142) so it calls one function per creature
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
- Rule in `resolve_shot` (`tactical.rs` 1451–1510): several pellets, strong close
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
and the six portrait looks, Opus the rules and the overlays on the head, Sonnet
the soldier file screen and the callers. Needs 1a: without it the mission
count is always 0.

**3a. Trait data** (changed by the third session)
- Two tables in `data.rs`, beside `RANKS` (466):
  - `STRENGTHS`, 8 rows: BRAVE, COLD, EMPATH, PROTECTIVE, RECKLESS, STEADY,
    SUPERSTITIOUS, LONER. All eight are strengths. Each row holds its good
    effect and its built-in catch.
  - `FEARS`, 6 rows: the dark, fire, tight spaces, the Glow, water, sharks.
    Water and sharks carry a "sea only" flag.
- `Soldier` (`data.rs` 442–464) gains `strength`, `fear`, `strength_seen`,
  `fear_seen` and a list of `earned` traits.
- `make_soldier` (`main.rs` 674–686) deals one strength and one fear.
- Suggested: on land it never deals a "sea only" fear, so no land soldier
  gets a fear that can never show. `LORE.md` only says these two "stay
  asleep" on land, which could also mean a soldier may carry one unseen.
  `ROADMAP.md` reads it the same way as here. To confirm (open question
  19).

**3b. Fears** (changed by the third session)
- A fear bites only where the thing is present. Each fear row is "a thing"
  plus "how the game can tell it is here".
- How the game can tell, for the four land fears. These tests are
  suggestions; `LORE.md` names the fears but not how they are measured:

| Fear | Test | Ready today? |
|---|---|---|
| The dark | `Battle.ambient < 0.32`. This is the game's own night test (`tactical.rs` 2149). | Yes |
| Fire | A tile with `Tile.fire > 0` within a few tiles of the soldier. | Yes |
| Tight spaces | Count the blocked tiles around the soldier. Start from the list the sight test uses (`los` and `line_of_fire`, `tactical.rs`: obstacles 2, 4, 10, 12, 13), plus tiles two levels up. | Yes |
| The Glow | A pool tile, or a unit hit by the fluid, in sight. | No. Waits for the Brooder (package 4). |

- Water and sharks: rows in the table, no rule written. They do nothing on
  land and wait for the sea campaign. **The old plan's rain trigger and the
  heavy-rain flag are dropped.**
- Effect: not decided. `LORE.md` says a fear "bites" where the thing is
  present, not what the bite is. Suggested: a morale hit at the start of
  the soldier's turn while the thing is present, fed into
  `check_unit_panic` (`tactical.rs` 1254–1269).
- Things to watch:
  - Morale climbs 15 every turn (`tactical.rs` 934) and panic is only
    tested below 50 (812). The fear's hit must be bigger than 15 a turn, or
    it never bites.
  - "Dark" is one number for the whole battle, not per tile. Lighting per
    tile lives in the drawing code and the rules must not read it. So a
    soldier afraid of the dark is afraid for a whole night mission.

**3c. Where the trait rules hook in**
- STEADY: `hit_chance` (`data.rs` 581–590), where wounds cut accuracy. It is the helper behind `ShotOdds.base`.
- BRAVE, COLD, EMPATH, LONER, SUPERSTITIOUS: the team morale loss in
  `on_unit_death` and `kill_unit` (`tactical.rs` 1271–1323) and `check_unit_panic`.
- PROTECTIVE and RECKLESS: `opportunity_fire` (`tactical.rs` 2177–2223) and
  `start_player_turn` (1380–1404).
- EMPATH "can tell a copy from a real person": used by the late human
  worker in package 7.
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
- Each earned trait may also carry a portrait mark, used by 3e.

**3e. Faces that change with service** (rewritten by the third session)
- Six stages, 0 to 5, from new recruit to the thousand-yard stare. Plus
  marks from events: scars from wounds, and changes from earned traits.
- A small `PortraitState`: stage, scar count, earned-trait marks. It is
  worked out fresh from the 1a `history` each time. It is never stored.
- `ui::portrait` (`ui.rs` 205) takes only a `Look` today. It will take the
  `PortraitState` too. Changed in the sixth session (2026-11-13): the
  portrait will be a picture of a 3D head built from the soldier's slider
  values (package 10b), not a player's picture.
- Callers to update: `tactical.rs` 3605 and 3753; `base.rs` 410, 429, 1819.
- Two drawing paths until 10b is built:
  - **The drawn face** (what exists today) changes its own shapes for each
    stage.
  - **The built head** (package 10b) shows the stages and marks on the
    head: lighting, grime, darker eyes, scars. The game builds the head,
    so it knows where the eyes are and no crop step is needed. How wear
    shows on the head is to settle.
- How many missions each stage takes is not decided. Keep the five
  thresholds as named constants in one place.
- No drawing state goes into the simulation.
- Needs 1a (missions, times wounded), 3d (earned marks), 10b (the head).

**Limits for the package**
- No fixed table is touched.
- **Self-test at risk.** Dealing traits in `make_soldier` adds dice rolls
  and shifts every roll after it. Give traits their own set of dice,
  started from the soldier's id number, so the main run of dice does not
  move.

### Package 4. Creature rules

Goes with graphics phase 5 (creatures). The taker's fluid and the Cinder's
fires go with phase 3. Large overall; each creature is medium. Fable writes
a design sheet where `LORE.md` leaves gaps, Opus writes the rule, Sonnet
adds one self-test case per creature.

All of these use the hook and the status block from 1c. Build in this
order: sides, Bloat, Kite, Gorger, Maul, Cinder, Brooder, Lathe.

**4a. More than two sides (do this once, first)** (medium, Opus)

The third session needs a wild side (a Maul with its spike broken, and
every Cinder). The Brooder needs civilians. Both change the same code, so
add both in one step.

- `Side` (`tactical.rs` 15) gains `Wild` and `Civilian`.
- `Unit` (`tactical.rs` 31–83) gains `home_side`: the side a unit goes back
  to. Today a Reach-taken unit always goes back to `Side::Alien`
  (`start_alien_turn`, `tactical.rs` 951–953). With a home side, a taken
  wild unit goes back to wild.
- `soldier_idx` becomes "maybe none". Today every enemy carries
  `soldier_idx: 0` (`tactical.rs` 613), and `finalize` writes every
  `Side::Player` unit back to that soldier (1908–1913). So a taken enemy
  that dies while on our side would mark soldier 0 dead. Fix this here.
- Every place that tests the side must be re-read:
  - `update_ai` (2073–2142) only moves `Side::Alien` and only hunts
    `Side::Player`. A wild unit hunts whoever is nearest.
  - `check_end` (1773–1784), through `alive_aliens` (1027): do living wild
    units stop the battle from ending? Not decided (open question 12).
  - The right-click target test (1642) allows `Side::Alien` only.
  - `finalize` counts kills and captives by `Side::Alien` (2570, 2574).
  - The two helpers at 625–628.
  - The drawing choice (2447–2448) picks soldier or alien shape by side.
    It must pick by what the unit is.
- Reaction fire already works for any number of sides. It tests "not my
  side" (`opportunity_fire`, 2186).
- Turn order: `Phase` (`tactical.rs` 12) has no wild turn. The simplest
  rule is that wild units act at the end of the enemy turn.
- Self-test: must give the same result file. No wild unit or civilian is
  in it.

**4b. The creatures**

- **Bloat (kind 9).** Bursts into a thick cloud when shot or when it
  reaches a target. The cloud blocks sight (the Kite's too) and chokes.
  - Reuse the smoke part of `explode` (`tactical.rs` 1887–2025), called
    from `on_unit_death` (1271–1290). Sight already fails once smoke along
    the line adds up to 18 (`los`, 665–666).
  - Choking is new: stun each turn in `tick_smoke_and_fire` (1325–1378).
    Tag the cloud so ordinary smoke grenades do not choke.
- **Kite (kind 1).** Marks a soldier; its shot dazes.
  - Dazed: half time units and no reaction fire for one turn. Touches
    `fire`, `opportunity_fire` and `start_player_turn`.
  - Marked: a Lash may shoot a marked soldier it cannot see, for as long
    as the Kite can see that soldier. That is an exception in the sight
    test (`los`, 650) for the Lash only. Overhead cover, smoke above, or
    killing the Kite ends the mark.
- **Gorger (kind 7).** A buried ambush that shows signs.
  - New `buried` flag. Hidden from the player until spotted or until it
    attacks. Note that
    `compute_visibility` (630) lights tiles, not units, so check how
    enemies on lit tiles are drawn first.
  - Signs (third session): shifting ground and dust. A spot check runs as
    a soldier moves, a few tiles out. How many tiles is not decided.
  - HANDHELD RADAR finds one reliably. The radar is item slot 25
    (`data.rs` 90) and is sold in the market (`base.rs` 1336), but it does
    nothing in battle today. This is its first use: a new soldier action
    that reveals buried units nearby.
  - Any dice rolled for the spot check must stay off the self-test path.
  - Its corpse (item slot 57) becomes refugee food in package 5.
- **Maul (kind 3).** An animal with no weapon, held by a control spike.
  - Its table row (`tactical.rs` 1788) names "ARC TORCH". Replace that
    with a magma blast: heat damage (type 1) through `explode`.
  - It walks to corpses and eats them, human or alien, with the weapons
    and kit they carry. It eats unconscious soldiers too. Each meal raises
    `fullness`.
  - The spike (third session): `Unit` gains `spike_hp`. When it reaches 0
    the Maul's side becomes `Wild`, and so does its `home_side`. It then
    attacks whoever is nearest, the enemy included.
  - How the player hits the spike is not decided (open question 11).
    `fire` (1437) aims at a whole unit. Facing is worked out in
    `hit_unit` (1624–1631). So the cheap rule would be "hits
    from behind damage the spike". A called shot needs a new button and a new
    aim mode.
  - Its corpse gives Slag by fullness (ties to 2c and 2d). The numbers are
    left for play. Keep them as named constants.
  - It cannot be freed: block it in the capture step of `finalize`
    (1898–1906) or at the freeing step in package 6.
  - Research id `"STUDY_MAUL"` stays (`data.rs` 862).
- **Cinder (kind 10).** The Maul's young: a glowing cube-shaped rock that
  rolls around.
  - It is never spawned today: no pool in `roll_alien` holds a 10. By
    `LORE.md` it breaks off a full Maul or comes out of a Maul's remains.
    So it is added to the battle while it runs, not rolled at the start.
  - It is always `Side::Wild`. Nothing controls it. It eats and sets
    fires, and goes for the enemy's dead as readily as ours.
  - `Unit` gains `growth`. Eating raises it, and the Cinder gets bigger
    and stronger. **It never changes `alien_kind` during a battle.** (The
    earlier plan said it did. The third session ruled that out: it only
    becomes a Maul between battles.)
  - Its row in `alien_def` (`tactical.rs` 1795) must be rewritten. Today
    it is a slow tank: 95 health, 34 time units, front armour 60. The lore
    says small and quick.
  - Fire does not spread at the moment. The Cinder sets fires directly
    through `Tile.fire`.
  - What a Cinder that outlives the battle does afterwards is not decided
    (open question 13).
  - Art rule, for `ART_BIBLE.md` when the Cinder is designed: its cube is
    rough, rounded, orange and tumbling. Registrar cubes are black, exact
    and still. The two must never look related.
- **Brooder, the taker (kind 5).** A hovering machine that spits fluid.
  - `Tile` (`tactical.rs` 18–28) gains a `pool` amount. `Battle::new`
    (`tactical.rs` 908–1025) places civilians on missions that suit them.
    The civilian side comes from 4a.
  - A civilian melts in one spit and then runs at soldiers. Soldiers take
    several, because armour soaks it up.
  - Pools hurt units standing in them. Acid, damage type 7, is already a
    row in `DAMAGE_MODS`.
  - Pools and fluid-hit units are what sets off the fear of the Glow (3b).
  - Neutralising spray (decided): item slot 14 plus one research row.
  - Its cube core as loot is a suggestion only, not decided. If wanted:
    item slot 15.
  - Its table row (1790) must change. Today it is a close-combat creature
    that does not float.
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
- Order: 4a before everything. Brooder before Lathe. Maul before Cinder.
  2c before Slag drops.

### Package 5. MESS HALL, SCRUB ROOM, REFUGEE QUARTERS, infection

Goes with graphics phase 7 (base modules and base people). Medium for 5a,
large for 5b. Opus for the rules, Sonnet for the rows and screens, Fable
for the icons and the look of the base interior.

**5a. Buildings, food, infection and sabotage**

- Three new `FacilityType` values (`data.rs` 665–683).
- `FACILITY_CATALOG` grows from 17 to 20 rows (705–723).
- `FacilityDef` (686–703) gains `mess_cap`, `scrub_cap`, `refugee_cap`.
  Every existing row needs the three new values.
- `MonthlyReport` (`data.rs` 1238–1254) gains food and scrubbing lines.
- Capacity sums go beside the others (`main.rs` 304–330). Food use and
  scrubbing go in `daily_tick` (`main.rs` 1138). Costs go in the expenses
  part of `monthly_tick` (1077–1100).
- **Hunger** (third session):
  - Decided: soldiers must be fed and the mess costs money to run.
    Refugees eat Gorger, and dead Gorgers are fine.
  - Not decided: what exactly makes a soldier "unfed" (no Mess Hall, a
    Mess Hall too small for the head count, or no money to run it). Open
    question 20.
  - Suggested: `Base` gains `gorger_meat`, and `Soldier` gains
    `hungry_days`. `daily_tick` uses up Gorger meat for each refugee. For
    human soldiers the simplest test is `mess_cap` against the head count,
    with the running cost in `monthly_tick`. A separate `meals` stock is
    only needed if the user wants food to be bought.
  - An unfed soldier starts a battle with less than 100 morale. Today
    every soldier starts at 100 (`tactical.rs` 578).
  - An unfed soldier recovers more slowly: the countdown of
    `wounded_days` (`main.rs` 1304–1306) slows down. This only matters
    once 1a sets `wounded_days` at all.
  - An unfed refugee loses stats, then leaves: removed from
    `Game.soldiers` with a log line. Nobody starves on screen.
  - Soldiers are not tied to a base today. "Which mess feeds whom" needs
    a simple rule, for example one shared food count for all bases.
  - How much a refugee eats is left for play. Named constants.
- **Infection** (third session):
  - `Base` gains an `infection` number. The more alien-derived equipment a
    base builds and uses, the higher it goes. The Scrub Room pushes it
    down, at a cost in power and resources, and never to zero. Refugees
    add nothing: by `LORE.md` a freed alien is clean.
  - Low risk brings sabotage, rolled in `daily_tick`. Three kinds:
    - A module goes dark. `BaseFacility` (`main.rs` 61–66) gains an
      "offline days" number, and the capacity sums skip it.
    - Research is lost. Cut `ActiveResearch.progress` (`main.rs` 85).
    - A craft is grounded. `CraftInstance.ready` is worked out again every
      day (`main.rs` 1297), so a flag would be wiped. It needs a "grounded
      days" number that the daily line respects.
  - High risk brings the battle inside the base. That is 5b.
  - The thresholds and odds are left for play. Named constants.
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
  - Sabotage dice run in `daily_tick`, not in battle, so the self-test is
    safe.
- Needs 2d for Gorger meat as food. Refugee Quarters is only useful with
  package 6.

**5b. The battle inside the base** (new; large, Opus, Fable for the look)

- There is no map of the inside of a base. `gen_map` (`tactical.rs` 774)
  only makes outdoor maps. So this needs a second map builder. It turns
  the 6×6 base grid (`Base.grid`, `main.rs` 105) into rooms and corridors,
  one room per module.
- Enemies start in the Workshop and the Supply Bay ("stepping out of the
  workshop and stores").
- `SiteKind::BaseDefense` (`main.rs` 24) exists and has never been used.
  This is what it is for.
- The battle starts with no site and no transport.
  `launch_selected_site` (`main.rs` 749) is the only way to start a battle
  today, and it needs a site picked on the globe. A second start path is
  needed.
- The landing craft is always drawn (`draw_landing_craft`, called at
  `tactical.rs` 2408) and always cut into the map. Both must be
  switchable off.
- Who defends: the soldiers at that base. Soldiers are not tied to a base
  today (see hunger above). The same simple rule must cover both.
- Needs the infection number from 5a. "The open door" in package 7 is its
  first use. An ordinary attack on a base could reuse the same map later.
- Self-test: safe. It uses the outdoor path.

### Package 6. Freed aliens ("refugees") as soldiers

Goes with graphics phases 4 and 5. Large. Opus, with Fable for the species
sheets. Needs packages 1a, 3, 4a and 5.

- `Soldier` gains `species` (human, Regent, Tender, Lash).
- A species table in `data.rs`: starting numbers, allowed weapons, allowed
  armour.
- `CapturedAlienSpec` (`data.rs` 491–496) gains `freed`.
- One new research row for the line that removes the control hardware
  (`RESEARCH_SURFACE` is exactly 20 rows; change the number).
- A "free" button on the Holding panel (`base.rs` 1038).
- Freeing moves a captive from `Base.captured_aliens` to `Game.soldiers`.
  Suggested: only if Refugee Quarters has room.
- **Study or free.** Live study kills the captive today (the log line at
  `main.rs` 1178 says so). If that stays, one captive can be studied or
  freed, not both, and the Holding panel must make that choice plain.
  Whether it stays is not decided (open question 21). `LORE.md` does not
  say that study kills.
- `Battle::new` (`tactical.rs` 554–593) must build a player unit from the
  species table.
- `draw_soldier` (2888) or `draw_alien` (3067) must be chosen by species,
  not by side. 4a already changes that line (2447–2448).
- What each species does (third session):
  - **Regent.** Uses Reach. Cannot use human weapons. The Reach panel
    (`draw_tactical_psi`, `tactical.rs` 3978) already works for a player
    unit. But every soldier holds a weapon position, and position 0 is the
    service pistol. "No weapon" needs a value of its own.
  - **Tender.** Small and frail. A scout that senses Reach users and enemy
    machines. That is a new reveal rule beside `compute_visibility`
    (`tactical.rs` 630). Suggested: it marks Regents (the only kind that
    uses Reach today) and the two machines (kinds 5 and 6) without lighting
    their tiles. How far it reaches, and whether walls stop it, is not
    decided.
  - **Lash.** The best marksman. Its first shot at any new target costs
    extra time. `Unit` gains a short list of targets already pardoned, and
    `fire` (1437) adds the extra cost when the target is not on it.
- **Harnesses** (third session). No refugee can wear human armour. The
  workshop makes each kind its own harness.
  - Three new rows in `ARMORS` (`data.rs` 480–487). It is exactly 6 today,
    so the count changes. Add them at the end: 0–2 stay land, 3–5 stay
    dive.
  - Each `ArmorDef` needs to say which species may wear it.
  - The land and dive swap in `Battle::new` (`tactical.rs` 560–571) picks
    armour by number (0 or 3). It needs a species check, or a refugee ends
    up in a human suit.
  - Three new rows in `WORKSHOP_BLUEPRINTS` (`data.rs` 903). It is exactly
    14 today, so that count changes too.
- Stat growth after a win (`main.rs` 944–953) and promotions (986) assume
  a human. Decide per species whether they grow and hold rank.
- Limits: `ARMORS` and `WORKSHOP_BLUEPRINTS` counts, as above. Self-test
  is safe; its soldier is human.

### Package 7. Signpost missions and the human worker

Goes with graphics phase 8 (environment, civilians). Medium to large for
the framework, small per script. Opus for the framework and rules, Fable
for the scripts, lines and sound design. Needs 1a and 1a-2. The civilian
side comes from 4a.

**7a. The framework** (widened by the third session)

There are five signposts now. `LORE.md` calls them all "signpost
missions", but as written they do not all need a battle of their own. The
"Shape" column below is our reading of each one, not a decision: a screen
with no battle, a moment inside an ordinary battle, or a mission of its
own.

| Signpost | Shape | Where the trigger is | What it needs |
|---|---|---|---|
| First words | A screen, no battle | First live captive admitted to Holding (`main.rs` 927–931) | The event screen, which exists but is never fed. By the lore the gibberish is built from words the player's own staff have said. The game has no staff speech, so suggested sources are soldier names and the log. |
| The split vote | A moment in the next ordinary battle | Research `"STUDY_REGENT"` finishes (the study branch at `main.rs` 1176) | The 1c hook. A Regent must be in that battle. |
| The same face | Its own mission | First entry in `lost_soldiers` | 1a-2, and the worker in 7b |
| Roll call | Its own mission, a rescue | The worker batch number reaches "late" | Civilians (4a), the late worker (7b), a goal that is not "kill everything" |
| The open door | A battle inside the base | Infection first runs high | Package 5a and 5b |

- A `SignpostDef` table: id, trigger, shape, script. It uses
  `story_flags`, `lost_soldiers` and `MissionSpec.kind` from 1a.
- `Game` gains `pending_beats`: moments waiting for the next battle.
- `MissionSpec` gains a `goal`. Today the only goal is "no living aliens"
  (`check_end`, `tactical.rs` 1088–1099). The finale needs another goal
  too, so build it once, here.
- The event screen (`draw_event`, `main.rs` 2123–2141) reads `event_title`
  and `event_body` (2129, 2132). Nothing ever sets them. "First words" is
  the first thing to feed it.
- Triggers are checked in `daily_tick` and `finish_battle`.
- "The same face" builds its enemies from the highest-ranked entry in
  `lost_soldiers`.
- Limits: `SiteKind` (`main.rs` 24) has a value that has never been used,
  `BaseDefense`. That one is now spoken for by 5b. Add `Signpost` for the
  missions here.

**7b. The human worker** (medium; Opus for rules, Fable for sound and lines)

- It first appears after the Registrar has read one of the player's
  soldiers. That means a soldier in `lost_soldiers`, so it needs 1a-2.
- `Game` gains `worker_batch`, a number that rises over the campaign. How
  many batches there are is not decided.
- A new creature kind 11, added at the end of `alien_def`
  (`tactical.rs` 1767). Its numbers, name and face come from a
  `LostSoldier`, not from the table row. By the lore workers are also made
  from abducted civilians. Those have no record today; a random `Look` and
  name would do.
- It needs the human shape while on the enemy side. 4a's "draw by what the
  unit is" covers this.
- **Early batch.** No weapon. It runs at the squad, clings, and calls out.
  Clinging is a new hold effect in the 1c status block. This settles
  "whether it fights": early ones do not.
- **Late batch.** It stands on the civilian side until revealed, then uses
  human weapons. A soldier with EMPATH can tell.
- **Voices.** `audio.rs` only has generated sound effects and no speech.
  For now a "voice" is lines of text plus generated calls (new `gen_*`
  functions) that move closer to speech as the batch number rises. The
  idea that a copy uses altered lines of the soldier it was read from
  waits until soldiers have lines.
- **Pool soldiers.** A copy made from a pool soldier would wear a real
  person's name and face. Whether that is allowed is the user's call
  (open question 14). Until it is answered, build the copy with the drawn
  face, not the pool soldier's face.

### Package 8. The Moon finale

Goes with graphics phase 9 (craft and the final sites). Large. Fable for
the ending, the maps and the difficulty curve, Opus for the rules. Needs
package 6 (Tenders), package 7 (the goal and the script system) and
graphics phase 3 (dark-side light).

- `MissionSpec.gravity`, and a map source for the lunar surface.
- A height-map loader. `gen_map` makes one sort of layout with a single
  height layer, so real lunar maps need a new map path.
- A `dark_side` light flag for `scene_light`.
- `make_final_mission` (`main.rs` 889–904) still puts the site on Earth's
  globe. It needs a Moon target.
- Low gravity: longer throws (`throw_grenade`, `tactical.rs` 1354, range 12
  tiles today) and cheaper climbs in `walk_cost` (689).
- Victory cannot be "no living aliens" (`check_end`, 1088–1099), because
  the Registrar cannot be killed. It uses the `goal` from 7a.
- **No pass mark** (third session). Every living freed Tender weakens the
  Seed's defences. There is no fixed number.
  - No new data. Count the living `Soldier`s whose species is Tender.
  - `make_final_mission` fixes 8 enemies at tier 6 (`main.rs` 894–900).
    Replace that with a strength worked out from the Tender count, at the
    moment the mission launches, not when the site appears.
  - "Brings to the Moon" is not settled: alive anywhere, or aboard the
    transport (open question 15). Aboard collides with the squad size.
  - The curve from "nearly impossible" to "fair" is Fable's to draw. Keep
    it as one small table.
- **The 8-wins shortcut.** The final site also appears after 8 wins with
  no research (`main.rs` 1007–1008). Suggested: remove it, because a
  player could reach the Moon without ever having freed a Tender. Not
  decided (open question 22).
- **Fix the win text.** It says the Seed "is dead" (`main.rs` 2155). By
  the lore it was overruled, not killed.
- The last word, "there are many", goes on the end screen (`draw_end`,
  `main.rs` 2143). The land win flags are set at `main.rs` 997–1003.
- Limits: the final mission is found by research id `"SEED_LOCATION"`
  (`main.rs` 1189). Keep that id. Self-test is safe if gravity defaults to
  normal.

### Package 9. Multiplayer: join or be wiped out

Version 3.0. A design note only for now. No code and no art yet. Large.
Opus for the split, Fable for the design.

- It needs the plain, save-ready records from packages 1, 3 and 6.
- It needs the simulation split from the drawing. Both live in
  `tactical.rs` and share the `Battle` struct today.

### Package 10. The soldier pool (new, from the third session)

A page reached from the title screen. The player makes soldiers there
once, with a name, a look and (from 10b) a face made in the character
designer. Every campaign started afterwards
pulls them in. No earlier package covers this. Goes with graphics phase 4
(soldiers).

**Where the pool lives: outside the repository**

- The pool is the first real player file the game writes. Today the game
  only writes test results and screenshots.
- It lives in the player's own data folder, never in the project folder.
  On this machine (Linux) that is `~/.local/share/defending-earth/`. On
  Windows it is under `%APPDATA%`. On macOS it is under
  `~/Library/Application Support/`.
- Inside it: one `pool.json` (names, looks, designer slider values, the
  "start with me" tick). No pictures are stored, so there is no
  `portraits/` folder (sixth session, 2026-11-13).
- **Pool data and names must never be committed.** The repository is
  public. Rules that keep it so:
  - The game never reads or writes pool data anywhere under the project
    folder. No "copy next to the game" fallback.
  - No real name or photo goes into `assets/`, `checks/`, `shots/`,
    `docs/`, test data or example files. Tests and examples use made-up
    names and the drawn face.
  - `shots/` is not in `.gitignore` today. So a screenshot that shows a
    pool soldier would be published with the next commit. See the debug
    rule below.
- A new small file, `src/pool.rs`, holds all the disk code: find the
  folder, load, save. Nothing in `tactical.rs` touches
  the disk. The game already uses `serde_json`, which can write
  `pool.json`.
- Finding the folder needs either one small new dependency (an outside
  code library the game pulls in) or a few lines of our own that read the
  system's settings. Either is fine.

**10a. Storage, the pool page, typing, arrival** (medium; Opus for storage
and arrival, Sonnet for the page)

- `PoolSoldier`: a `pool_id`, name, `Look`, "start with me" tick. The
  designer's slider values (10b) are added later. No stats and no traits:
  those are dealt fresh in every campaign and hidden like anyone else's.
- A new `Screen::Pool` in the `Screen` list (`main.rs` 21). A "SOLDIER
  POOL" button on `draw_title` (`main.rs` 1567), beside the three buttons
  there now (1586–1593).
- The look form already exists in `draw_editor` (`base.rs` 1807–1958). But
  it edits `g.soldiers[g.selected_soldier]` directly (1816). It must be
  pointed at "the look being edited", so the same form serves a campaign
  soldier and a pool entry.
- **Name typing is new.** Nothing in the game takes typed text. The editor
  only has "RANDOMIZE NAME". The engine offers `get_char_pressed`, which
  the game never calls. A small text box is needed: letters, backspace, a
  length limit, and only characters the game's font can draw.
- **Arrival.** `make_soldier` (`main.rs` 674) has four callers:
  - the land start, 8 soldiers (`main.rs` 564–565);
  - the sea start (`main.rs` 612–613);
  - HIRE on the soldier list (`base.rs` 381);
  - BUY in the market (`base.rs` 1366).
  Ticked pool soldiers take places in the starting 8. The rest turn up at
  random when the player hires. Suggested: if more than 8 are ticked, the
  extras arrive by hiring.
- `Soldier` gains `pool_id` (maybe none). Suggested: one pool soldier
  appears at most once per campaign. Dying in one campaign does not remove
  them from the pool; they are available again in the next.
- Stats, strength and fear are rolled as for any recruit.
- Things to watch:
  - **Debug runs must ignore the pool.** `--demo`, `--screenshot`,
    `--self-test` and `--ai-test` all start a campaign (`new_campaign` is
    called at `main.rs` 2327, 2374, 2389, 2419 and 2425). If they loaded
    the pool, private faces would land in `shots/` and the self-test
    result would change from one machine to the next. The pool loads only
    from the two real "start campaign" buttons (`main.rs` 1668, 1707).
  - **Dice.** Choosing whether a hire is a pool soldier must not use the
    main run of dice. Use a separate set, as for traits in package 3.
  - A broken or missing `pool.json` must not stop the game. Start with an
    empty pool and say so on the pool page.
- Needs nothing else. It can be built before package 3; traits are added
  when 3a exists.

**10b. The hand-driven character designer** (medium; Opus for the head
builder, Fable for the slider list, Sonnet for the screen)

Changed in the sixth session (2026-11-13). The old 10b (picture import
and crop, one picture per soldier) is gone.

- **The face:** a detailed face with many sliders. The slider list is to
  settle.
- **The body:** simple. Build, height within a small range, skin, hair
  style and colour, and a voice.
- **Also set by the player:** name, nickname, a short background (flavour
  only), and a favoured role (a head start on one ribbon, 3f; no stats).
- **Only slider values are saved** in `pool.json`. No picture is stored.
- **The head is built from the slider values**, once, when the soldier
  loads. The close-up portrait is a picture of that head. Today
  `ui::portrait` takes only a `Look` (`ui.rs` 205) and no head builder
  exists. How a built head is drawn and kept is to settle with the head
  builder.
- A soldier can be made with no photo at all.
- Needs 10a, 1a and 3e. The six stages and marks (3e) apply to the head.

**10c. The photo step** (to measure, Opus)

- **Optional.** A photo sets the face sliders. The player can then change
  them by hand. The photo is read once and thrown away. Only slider
  values are saved.
- **The mapping is ours to write.** No open photo-to-slider tool was
  found. The plan is to find face landmarks in the photo, then set sliders
  to match. Check the licence of any code or weights before use.
- **File types.** The engine reads PNG and TGA only. Its picture reader is
  built without JPG (macroquad 0.4.16 asks for just those two). Phone
  photos are JPG. So either the game takes PNG only, or the picture reader
  is added as a dependency of our own with JPG switched on. Not decided
  (open question 16).
- **Getting the file in.** Not settled. Two ways:
  - Drag the file onto the game window. The engine has
    `get_dropped_files`, and miniquad 0.4.11 has code for it on Linux,
    macOS and Windows. The game has never used it, so it is untested here.
  - A fallback that needs no dragging: an `import/` folder inside the data
    folder.
- Needs 10b.

| Part | What | Size | Who |
|---|---|---|---|
| 10a | Storage, page, typing, arrival | Medium | Opus for storage and arrival, Sonnet for the page |
| 10b | Hand-driven character designer | Medium | Opus for the head builder, Fable for the slider list, Sonnet for the screen |
| 10c | Photo step | To measure | Opus |

### Order

1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9. Packages 2 and 3 can run side by side
after 1. Package 5a can start once 2d is in.

- 1a-2 must be in before 7b, or the human worker never appears.
- 4a (sides) must be in before the Maul, the Cinder, the Brooder,
  package 6 and package 7.
- 5b (the base interior) must be in before "the open door" in package 7.
- The `goal` in 7a must be in before package 8.
- 10a stands alone and can be built any time. 10b waits for 3e and 1a. 10c comes after 10b.

### Still open (the user decides; ask through the question tool)

What the third session closed, and what is left of each old question:

| # | Question | Where it stands |
|---|---|---|
| 1 | Strengths and flaws | Closed. All eight are strengths with a catch. Flaws are fears. The numbers are open. |
| 2 | Which fears | Closed. Six: the dark, fire, tight spaces, the Glow, water, sharks. The last two are sea-only. |
| 3 | Maul numbers and control | Partly closed. Control is the spike. Slag and eating numbers are left for play. |
| 4 | The mess runs out; Gorgers | Partly closed. Dead Gorgers are fine. Hunger is decided. How many per refugee is left for play. |
| 5 | Infection and an inside attack | Partly closed. Sabotage, then a battle inside. The numbers are left for play. |
| 6 | What each freed species can do | Closed in outline. The numbers are open. |
| 7 | The human worker | Partly closed. First appearance and the two kinds of batch are decided. How many batches is open. |
| 8 | More signposts | Closed for now: five. |
| 9 | What a joined multiplayer player controls | Open. |

New questions the third session raises:

10. How many missions each of the six portrait stages takes.
11. How the player aims at the Maul's spike: from behind, or a called
    shot.
12. Whether living wild units stop a battle from ending.
13. What a Cinder that survives a battle does between battles.
14. Whether a copy may wear a pool soldier's real name and face.
15. Whether a Tender counts for the finale if alive at base, or only if
    aboard the transport.
16. Photo step (10c): JPG as well, or PNG only.
17. Whether the Glow fear reacts to a soldier's own cored weapon, which is
    filled with Glow.
18. Whether each refugee species gains stats and rank as humans do.
19. Water and shark fears on land: never dealt to a land soldier (what
    this file and `ROADMAP.md` assume), or dealt and simply never seen.
20. What makes a soldier "unfed": no Mess Hall, one too small for the head
    count, or no money to run it. And whether human food is a stock the
    player buys or only a running cost.
21. Whether live study still kills the captive, so that each captive is
    either studied or freed.
22. Whether to remove the rule that shows the final site after 8 wins
    without research.
23. What the bite of a fear is (a morale hit is suggested), and how each
    land fear is measured.
