# DEFENDING EARTH — ROADMAP

Working draft 2, 2026-11-13. Companions: `ART_BIBLE.md` (look), `LORE.md`
(story), `NAMES.md` (every name in the game), `STATUS.md` (where the work
stands right now).

## Where we are

A working single-player squad-tactics game in Rust (macroquad): globe, base,
research, interception, 26×26 tactical battles, two campaigns (surface and
underwater). As of 0.9 its names, creature roster, item tables and data are
this project's own (listed in `NAMES.md`). The genre mechanics (time units,
reaction fire, morale, line of sight, smoke and fire) are ours to keep. Some
rule formulas and starting values are placeholders that play as written today
and are marked "under review for 2.0" in `MECHANICS.md`.

Version 0.9 is done. Inside 1.0, graphics phases 1 and 2 of 7 are done
(lighting, time of day, weather, biomes, seasons). Next is phase 3:
flashlights, flares, fire.

## Versions

### 0.9 — Our own identity (done 2026-11-13)
The step that makes everything after it ours. It does not change how the game
plays, only what things are called and whose numbers they are.

- [x] Replace franchise names in UI, logs and docs (organisation, craft,
      facilities, research topics, materials, final missions).
- [x] Replace the creature roster with the working roster in `ART_BIBLE.md`
      (one table in `tactical.rs`, plus the name-matching code that reads it).
- [x] Replace the two inherited item data files with our own item table:
      our names, our values, same structure so no screen breaks.
- [x] Remove executable addresses and provenance notes from code
      comments, `MECHANICS.md` and `mechanics.json`; rewrite the sheet as our
      own rules reference.
- [x] Rewrite `README.md` and `CREDITS.md`.
- [x] Re-run the self-test and re-shoot every reference screenshot.

Known couplings to change together (from the code read): weapon lookups by
display name, creature armour class by name substring, research ids for the
two final missions, blueprint ids for four craft, the sale price table, stores
keyed by item name.

Checked: release build clean, self-test passes, three leftover-name sweeps of
source, docs and the built program came back clean. The two old data files
are out of the app (kept outside it in `.work/removed/`). The old
pictures that showed old names were moved out of the app on 2026-11-13.

### 1.0 — The single-player game, looking right
Two separate campaigns chosen on the campaign screen. Goal: prove the look and
the initial mechanics.

Graphics pass, in order (each phase ends with screenshot rounds and a
frame-time check):
1. Lighting pipeline, time of day and sky states on one biome; situation map
   day/night tied to the clock, real city lights. **Done 2026-11-13.**
2. Weather, remaining biomes, season and location selection. **Done
   2026-11-13** (see `STATUS.md` for what it covers and its known flaws).
3. Flashlights, flares, fire. **Next.** Overcast, storm and night skies
   from Poly Haven sky photographs are also still to do.
4. Soldiers: appearance generator, visible kit and history, wounds, burning.
5. Enemy creatures.
6. Weapon, armour and item designs across the three design steps.
7. Base modules.

Debug controls added along the way (typed after the program name, before
`--demo`; full notes in `CODE_MAP.md`):
- `--clock "2027-06-21 8am"`: set the date and local time.
- `--weather clear|rain|snow|dust|storm|blizzard|duststorm`: force the weather.
- `--site <lat,lon>`: put the battle at a real place; biome and season follow.
- `--biome <n>`: force a biome by number.
- `--seed <n>`: build the same map again.
- `--view wide|tele|low`: pick a camera view.
- `--t <seconds>`: how long to run before the screenshot.
- `--frametime`: show the frame-time readout.
- `--screenshot <file>`: save a picture and exit.

How every graphics phase is run (the user's standing brief):
- At least three improvement rounds. Each round: take screenshots, read
  every one, judge it as a harsh art director and name the three weakest
  things, fix those, repeat.
- Target 60 frames a second at 1920×1080. If under 50, make it faster before
  adding anything; lower the internal resolution before cutting effects.
- Report in five lines or fewer: what changed, best moment and the flags to
  see it, known flaws, what the next phase needs.
- The look (`ART_BIBLE.md`): camera stays high; helicopter long-lens aerial
  documentary. Not wanted: plastic default materials, uniform tiling
  textures, bloom over everything, cartoon fire, flat ambient fill.
- Blender bakes are scripted, tested small first, and each baked file is
  checked. Models are made by us; CC0 Poly Haven surfaces and skies allowed.

#### Phase 1 work order (done 2026-11-13)
All eleven steps are done; kept as a record of how the port was made.
Result and leftovers are in `STATUS.md`. File and function names are in `CODE_MAP.md`.
Phase 2 has since changed three things in this list: weather is now drawn
in 3D (steps 3 and 7 say 2D), `--weather` has three heavy forms and
`--site` was added (step 10).

1. `src/main.rs`: add `mod sky; mod gfx;`; give `Game` a `gfx: gfx::Gfx`
   made with `Gfx::new(1920, 1080)` after the window exists; add debug
   fields (clock override, cover/storm override, view name, frame-time
   readout on/off).
2. `src/tactical.rs`, mesh builder: `MB` keeps its `quad/tri/bx` calls (so
   the hundreds of call sites survive) but writes lighting-format vertices:
   real face normal, a current material (`mb.mat = gfx::mat::BUILT` etc.),
   unlit colour, ambient occlusion in alpha. Two buffers: `lit` (drawn in
   shadow and scene passes) and `over` (see-through, drawn in overlay
   passes: smoke, move tiles, selection ring, halo) plus `glow` (fire,
   embers, enemy hull lights, power-plant core). A colour with alpha < 1
   routes to `over` automatically. Split into meshes of at most 60,000
   vertices.
3. Remove the old lighting: `shade` and `fogged` return the colour
   unchanged, `bx` stops darkening sides, `cast_shadow` and the unit blob
   discs are deleted, `sky()` and the grading rectangles in
   `draw_surface_fx` go (rain/snow/dust streaks stay as 2D for now; phase 2
   redoes weather).
4. Ground: fill a 26×26 ground map from `biome_ground` per tile and call
   `set_ground_map`; add wide apron quads (about 260 m) around the map in a
   matching colour so the battlefield sits in country instead of floating.
5. Light: build a `sky::Moment` from `Game.date` (or `--clock`), site
   latitude/longitude from `MissionSpec`; cover/storm from weather (clear 0,
   rain cover 1 + wet 1, snow cover 0.8, dust cover 0.3; `--weather`
   overrides). Underwater: fixed hand-made look (dim blue-green sun from
   above, heavy haze) through the same passes.
6. Camera in `make_camera` only: same aim point, about 3.3× further back,
   field of view about 14°, `z_near`/`z_far` set around the distance.
   Check mouse picking (`screen_to_tile`) and name tags (`project`) still
   line up; they use the same function.
7. `draw()`: shadow_begin → lit meshes → scene_begin → lit meshes →
   overlay_begin(0) → `over` + tracers → overlay_begin(glow) → `glow` →
   finish → 2D weather → overlays → HUD.
8. Build, run `--self-test` (must still pass: drawing only changed), then
   `--demo tactical --screenshot`.
9. Globe (`src/globe.rs` + situation map in `Game::draw`): pass
   `sky::globe_sun_dir(moment)` as the sun instead of the frozen one; add
   `assets/earth_lights.png` as a texture, lit on the night side with a
   brightness threshold that falls as the sun gets lower, so big cities
   come on first.
10. Debug flags in `main`: `--clock "2027-06-21 8am"`, `--t <seconds to run
    before the screenshot>`, `--biome <n>`, `--weather <clear|rain|snow|dust>`,
    `--seed <n>`, `--view <wide|default|tele|low>`, `--frametime`.
11. Three judged rounds, frame-time check, report; update `STATUS.md`.

Risks seen in advance: per-frame mesh rebuild is drawn twice (shadow and
scene) so watch frame time; unit and prop boxes have no ambient occlusion,
so they may look pasted on until contact darkening is added under them;
the old 44° view showed a horizon, the long lens shows only ground, so the
apron must look good.

#### Phase 2 record (done 2026-11-13)
What it delivered. Result, best view and known flaws are in `STATUS.md`.

- Weather drawn in 3D inside the scene (`draw_weather` in
  `src/tactical.rs`): rain streaks with splashes and puddles, snowfall with
  blowing drift, dust. Heavy forms: storm, blizzard, duststorm.
- Rain streaks are drawn as if the shutter were open 1/30 of a second.
  The art bible asks for 1/100, but shorter streaks look like white specks
  from this height.
- Snow lies on ground, roofs, trees and crates; sand has wind ripples. The
  ground shader (the small program that colours the ground) gets the
  amounts through one setting called `surf` in `src/gfx.rs`.
- All seven land biomes in the world map (`assets/earth_data.png`) draw and
  follow the season: 0 temperate, 1 farmland, 5 mountain, 6 rain forest,
  7 desert, 9 ice, 11 northern forest.
- `--site <lat,lon>` picks a real place. The battle screen labels the hour
  (MIDDAY, EVENING and so on) and, in the tropics, WET SEASON or DRY SEASON.
- Game rule change: the world-map lookup was 180° of longitude off, so
  every site got the terrain of the far side of the planet. Fixed in
  `data::world_index`. Missions everywhere now get the right terrain.
- Two things to keep: the draw buffer set in `src/main.rs`
  (`gl_set_drawcall_buffer_capacity(65_535, 400_000)`) must stay at or
  under 65,535 vertices (corner points) or drawing breaks; and the lens
  has an exposure offset (`ev_bias` in `gfx::Lens`) that brightens snow
  scenes.
- 5.56 ms per frame at 1920×1080. Self-test passes.

Mechanical gaps found during the code read, to schedule inside 1.0:
- No save or continue.
- Fog of war is computed but not drawn; enemies show on unseen tiles.
- The battlefield is a single height layer: no stacked floors, roofs or doors.
- The generator ignores mission type, craft size and time; every map has the
  same layout.
- Fire does not spread and nothing has a flammability value.
- Only the enemy gets reaction fire.

### 2.0 — Our version
- One merged campaign: land and sea in the same war, balanced by playtest.
- The Registrar storyline (`LORE.md`) replaces the inherited two-finale
  structure.
- Deeper systems chosen from the ideas list below.
- New rules sheet written from our own design, not inherited values.
- The repository was created early, on 2026-11-13, at the user's request:
  https://github.com/iannitram/defending-earth (public).

### 3.0 — Multiplayer, server-hosted
- Players claim countries and run their defence from inside them.
- Trade between players.
- Collaboration with the enemy as a real option, with a cost.
- Soldiers cross between players' games: a soldier taken in one game can
  appear as an enemy in another.

Groundwork that should start early so 3.0 is not a rewrite:
- Soldier, wound and history records stay small, self-contained and
  serialisable (also needed for saves in 1.0).
- All randomness stays seeded and reproducible per mission.
- Simulation kept separate from drawing, so a server can run a battle without
  a window. Today they share one file.

### 4.0 — Mixed reality (the user's idea, 2026-11-13)
A headset version where the game sits in the player's room.
- The situation map floats in front of the player.
- The base stands on a real table, with its readouts and the decisions
  waiting to be made hanging overhead.
- Battles play out as a tabletop scene the player can walk around, lean
  into, and scale up or down by hand.

Groundwork that keeps this possible: simulation separate from drawing (also
needed for 3.0), no rule that depends on a fixed camera, and models that hold
up when seen close and from any side.

## Ideas (unsorted; we sort these into versions together)

### Lore
Lore now lives in `LORE.md`: the enemy is the Registrar, a cataloguing
machine with nobody home, whose "preserve all life" means reading a thing
completely and destroying the original. Decided so far and still open are
both listed there.

### Mechanics
- **Husks.** A soldier who is mind-controlled or taken by a parasite is not
  just lost: they persist, with their face, kit and record, and can turn up in
  another player's game.
- **"DO IT."** When one of your soldiers is about to fire the killing shot on
  a husk, the husk yells it. Implies: husks keep the soldier's voice, and
  something of the person is still inside.
- Collaborating with the enemy: what a player gains (technology, safety,
  reissued soldiers?) and what it costs them with other players.
- Country identity: each player's uniforms, kit and base style start from
  their country's real present-day look.
- Trade: gear, research, captured material, soldiers on loan.
- Research changes the look of gear as well as its numbers (three design
  steps, see `ART_BIBLE.md`).

- Multiplayer (3.0): the aliens get into the in-game chat. At first the
  messages are in their own script; over the campaign they decipher, a few
  characters at a time, into plain words addressed to the players ("No one
  has to die." "We bring good tidings from across the universe."). Messages
  are triggered by events and game state (a lost soldier, a state taking
  separate terms, a player falling behind), and can be sent to one player
  privately. This is the route by which the aliens try to turn a player.
  To settle: what a turned player gains and can do; how other players can
  tell real chat from alien chat; captured-alien research speeding up the
  deciphering.

- From the lore sessions (details in `LORE.md`): infection tax on alien
  technology and scrubbing research; freeing captives to serve as soldiers;
  base defence, including raids to recover a captive; signpost missions set
  off by what the player has done ("The same face").

- The mess hall: a base building that feeds soldiers (running cost), sells
  researched stims and medicines, and, once defectors exist, must be
  restocked with Gorgers to feed them (details in `LORE.md`).

- Civilians on terror missions (the user's design, 2026-11-13): they run
  TOWARD the soldiers to be saved, and hide beside or behind them, which
  makes stealth hard. They only react to soldiers they can see or gunfire
  they can hear; otherwise they hide where they are. A soldier can spend a time unit to order one to "get
  to the extraction vehicle"; the civilian then runs to the craft and stays
  there. A civilian who reaches the craft, or is alive when the mission
  ends, counts as saved. Aliens can follow them to the craft to kill them.
- Melting panickers: civilians (and, after enough hits, soldiers) spat on
  by the field reader run at soldiers and damage them on contact; the pools
  left behind damage anyone who steps in them (details in `LORE.md`).
- Footprints in snow: soldiers, civilians and aliens leave tracks. While it
  is still snowing the tracks fade over time. Aliens can see and follow
  tracks, how well depending on the species' intelligence; the player can
  read alien tracks the same way.

### Presentation
- Your own face on a soldier (the user's idea, 2026-11-13): the player
  gives the game a photograph and it is fitted to a soldier's face, to go
  with naming them. Belongs with the soldier appearance generator (graphics
  phase 4). To settle: how the photo is fitted (a face texture on our head
  model is the cheap route), that photos stay on the player's own machine,
  and what happens to such faces in 3.0 when soldiers cross between games.
- Life in the base view (the user's idea, 2026-11-13): small figures of
  soldiers and scientists going about what fits the hour (training, eating,
  sleeping, lab work, repairs). Simple figures are enough. Belongs with base
  modules (graphics phase 7) and the mess hall idea above.
- Situation map: night sweeping across the globe, major cities lighting up as it
  passes. (Done in graphics phase 1.)
- Battles filmed as aerial documentary footage (see `ART_BIBLE.md`).
- Weather that follows the real climate of the site's location and season.
- Real places. City raids are the showpiece missions, and each should feel
  like the place it happens in. Three levels, cheapest first:
  1. Regional dress (planned with the 1.0 biomes): each of roughly a dozen
     world regions gets its own building shapes, roof and wall materials,
     road markings, driving side, vehicles, street furniture, vegetation and
     signs in the local script and style. Made by us; no real brands.
  2. Real street plans: build the map from the open street map of the actual
     neighbourhood (OpenStreetMap, free to use with credit), so roads, blocks,
     rivers and parks sit where they really are, filled with level-1 pieces.
  3. Landmark missions: a small number of hand-made famous cities with one
     recognisable landmark each. Costly per city; famous buildings and
     skylines can carry their own rights, so check each one first.
  Not planned: satellite or street-level photos of real places (licences
  forbid it, and they would not match our lighting).
