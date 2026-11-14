# DEFENDING EARTH — ROADMAP

Working draft 3, 2026-11-13. Companions: `ART_BIBLE.md` (look), `LORE.md`
(story), `NAMES.md` (every name in the game), `STATUS.md` (where the work
stands right now).

## Where we are

A working single-player squad-tactics game in Rust (macroquad): globe, base,
research, interception, 26×26 tactical battles, two campaigns (surface and
underwater). As of 0.9 its names, creature roster, item tables and data are
this project's own (listed in `NAMES.md`). The core rules (time units,
reaction fire, morale, line of sight, smoke and fire) stay. Some rule
formulas and starting values are placeholders that play as written today
and are marked "under review for 2.0" in `MECHANICS.md`.

Version 0.9 is done. Inside 1.0, graphics phases 1 and 2 of 9 are done
(lighting, time of day, weather, biomes, seasons). Next is phase 3:
flashlights, flares, fire.

There are now two lists of work, side by side:
- **The graphics pass** (phases 1 to 9): how the game looks.
- **The game systems track** (packages 1 to 9, added 2026-11-13): the rules
  and records behind the lore decisions in `LORE.md`. It has its own section
  further down, after the versions.

Words changed on screen today: the structure material is SPINE, the power
material is GLOW, "Psi" is REACH, the Derived Battery is the BREACHER
BATTERY, the Heron is a twin-rotor transport helicopter, and the land war
ends on the far side of the Moon. Names inside the code were left alone.

## Versions

### 0.9 — Our own identity (done 2026-11-13)
The step that makes everything after it ours. It does not change how the game
plays, only what things are called and whose numbers they are.

- [x] Replace every placeholder name in UI, logs and docs (organisation,
      craft, facilities, research topics, materials, final missions).
- [x] Replace the creature roster with the working roster in `ART_BIBLE.md`
      (one table in `tactical.rs`, plus the name-matching code that reads it).
- [x] Replace the two old item data files with our own item table:
      our names, our values, same structure so no screen breaks.
- [x] Clear out old working notes from code comments, `MECHANICS.md` and
      `mechanics.json`; rewrite the sheet as our own rules reference.
- [x] Rewrite `README.md` and `CREDITS.md`.
- [x] Re-run the self-test and re-shoot every reference screenshot.

Known couplings to change together (from the code read): weapon lookups by
display name, creature armour class by name substring, research ids for the
two final missions, blueprint ids for four craft, the sale price table, stores
keyed by item name.

Checked: release build clean, self-test passes, three leftover-name sweeps of
source, docs and the built program came back clean. The two old data files
and the old pictures that showed old names were deleted on 2026-11-13.

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
   from Poly Haven sky photographs are also still to do. Added 2026-11-13:
   the taker's fluid glows. It lights the ground, the pools it leaves and
   the people it has hit, so it is built here with the other lights. Cinder
   fires and the Maul's magma use the same fire work.
4. Soldiers: appearance generator, visible kit and history, wounds, burning.
   Added 2026-11-13: close-up portraits that change with service.
5. Enemy creatures. The decided roster is ten kinds: three peoples, five
   animals and two machines (the taker and the builder). The Trestle is
   dropped from the design; it is still in the code until package 1b.
6. Weapon, armour and item designs across the three design steps. Now
   includes the shotgun, the knife and the material Slag.
7. Base modules. Now includes the Mess Hall, Scrub Room and Refugee
   Quarters.
8. Environment objects: trees, plants, rocks, buildings, walls, fences,
   crates and field clutter for all seven biomes, with broken versions.
   Civilians and the human worker are here too.
9. Vehicles and craft: player craft, enemy craft (flying, landed and
   crashed), ground vehicles seen in towns and farms. The final sites are
   here: the far side of the Moon (low gravity, ground from real lunar
   surface maps, lit only by the enemy's own lights) and, later, the sea
   site.

Phases 4 to 9 together must cover **every** thing in the game. The full list,
the motions each thing needs, and which AI model does which part are in
"Design and animation catalogue" below. A phase is not done until every row
it owns is designed, built, animated and judged in screenshots.

#### Design and animation catalogue (added 2026-11-13, the user's request)

Every thing goes through the same five steps:
1. **Design sheet.** What it is, what it is for, how it looks, written and
   agreed with the user (creatures: lore first, in `LORE.md`).
2. **Model.** Built in Blender by script, with baked textures, checked in a
   test render.
3. **Rig.** A skeleton inside it, for anything that bends or has moving parts.
4. **Motions.** Each motion in its list, posed from reference frames, checked
   as a strip of pictures.
5. **In the game.** Exported, drawn by the game, judged in screenshots at
   the real camera height.

Counts come from `NAMES.md` and the code today; sea rows wait until the sea
campaign is picked up again. Updated 2026-11-13 for the day's lore
decisions. Where a row says "today" and "then", the second number is the
count once the matching game systems package is in.

**Everything the player starts with looks present-day.** Starting weapons,
armour, vehicles and base modules look like what armies have now, nothing
near-future. They are our own designs of ordinary types: no real maker's
name, badge or exact airframe. The three design steps (see `ART_BIBLE.md`)
start from there.

| Group | What must be designed | How many today | Phase |
|---|---|---|---|
| Soldiers | Body builds, faces, skin, hair; the appearance generator | 1 generator, many results | 4 |
| Soldier portraits | The close-up face at each stage of service: new and jittery at no missions, the thousand-yard stare at fifty, steps in between. Also the file entry for a trait not yet seen ("UNKNOWN") | about 5 stages on every face (number to settle) | 4 |
| Soldier armour | Every armour, in its design step, worn and damaged. The first step looks present-day | 6 | 4 and 6 |
| Freed aliens as soldiers ("refugees") | Regent, Tender and Lash with the control hardware removed, in our squad: markings, kit they can carry, portrait | 3 species | 4 and 5 |
| Land creatures | Every kind, plus its corpse or wreck and its captive form. People: Regent, Tender, Lash. Animals: Gorger, Kite, Bloat, Maul, Cinder. Machines: see the next row. The Trestle gets no design: it is to be removed (package 1b) and its table slot will stay empty | 11 in the code today, then 10 | 5 |
| The two machines | The taker (Brooder): hovering plates around a cube core; forms for travelling, scouting, spitting and melting. The builder (Lathe): blade legs around a sphere core; forms for travelling, repair, making cover and building a taker. Closed and armoured when moving, core exposed when working | 2 kinds, about 4 forms each (counted in the 10 above) | 5 |
| The human worker | Copies of real people, improving batch by batch: first plainly unfinished, last able to pass among civilians. They wear the faces of lost soldiers and taken civilians | about 3 batches (number to settle) | 8, faces from 4 |
| Sea creatures | Every kind in the sea table (deferred) | 9 kinds | 5, later |
| Soldier weapons | Every weapon and its ammunition, held, slung, dropped. New: SHOTGUN with shells, and KNIFE | 32 in the weapon list today (14 land, 16 sea, 2 used only by creatures), then 34 | 6 |
| Other items | Grenades, smoke, trip mine, radar, sonar, trauma kit, flares, lights, pack. New: neutralising spray (mid-game) | about 10 today, then about 11 | 3 and 6 |
| Materials and recovered parts | Each material and fuel as an object in stores and in wrecks. SPINE (cut from wrecks and bodies), GLOW (the taker's fluid, in a pool and in a container). New: SLAG (Maul remains), the taker's cube core, the builder's sphere core, Gorger meat as food | 6 today, then 10 | 6 |
| Base modules | Every module, outside and inside, with broken versions. New: MESS HALL, SCRUB ROOM, REFUGEE QUARTERS. Renamed today: BREACHER BATTERY, REACH DAMPER, REACH WING | 17 today, then 20 | 7 |
| Base people | Small figures of soldiers, scientists and engineers at work (the user's base-view idea). New: refugees, who eat in the mess and live in their quarters | 3 roles today, then 4 | 7 |
| Trees and plants | Per biome: 2 to 4 trees, bushes, ground plants, crops; each in four seasons | 7 biomes | 8 |
| Rocks and ground forms | Boulders, cliffs, dunes, ice ridges, river banks | per biome | 8 |
| Buildings | Per biome: houses, barns, sheds, shops, with insides and broken versions | 7 biomes | 8 |
| Field clutter | Walls, fences, hedges, crates, hay bales, log piles, signs, poles, wells, street things | about 30 | 8 |
| Civilians and animals | Townspeople and livestock (they are what the collector craft takes). Each civilian also needs a "hit by the fluid" look: glowing, melting | to be decided with the user | 8, glow from 3 |
| Player craft | Every craft, outside, inside (the troop bay is on the battlefield), on the ground and in the air. The Heron is a twin-rotor transport helicopter with a rear ramp; the starting fighter is an ordinary jet. Both present-day in look | 5 land, 5 sea later | 9 |
| Enemy craft | Every class: flying, landed, crashed, and its inside | 4 land, 1 sea later | 9 |
| Craft weapons | Each weapon as fitted to a craft | 12 | 9 |
| Ground vehicles | Cars, trucks, tractors, wrecks as cover | about 8 | 9 |
| The two final sites | Land: the Seed on the far side of the Moon. Ground built from real lunar surface maps; a sky showing the line between the lit and dark halves; the fight is on the dark side, lit by the Registrar's own lights; the cube core. Sea: Verity Deep (deferred, kept as it is) | 2 (1 now, 1 later) | 9, after lore; lights from 3 |
| The ending | The freed Tenders' vote and merge into a maker, a form nobody has seen before; the end screen with the last word | 1 scene | 9, after lore |
| Effects | Muzzle flash, tracers, smoke, fire, blasts, dust kicks, arcs, blood, debris. New: glowing taker fluid (spit, pools, on bodies), Bloat cloud, magma blast, shotgun spread, slow Moon dust | about 15 today, then about 20 | 3, then each phase |

Motions each group needs:

| Group | Motions |
|---|---|
| Soldiers | Stand, walk, run, crouch walk, kneel, go prone, turn; aim, fire (single, burst, auto), reload, throw, use item, heal another; climb, drop, open door; hit reactions by body part, stagger, panic, burning; fall dead in several ways, lie wounded, be carried; idle fidgets. Each weapon class changes the carry, aim and reload. New: pump and fire the shotgun, draw the knife and strike; fear reactions (flinch, freeze, back away); low-gravity versions of walk, jump, throw and fall for the Moon |
| Soldier portraits | Small face motions per service stage: quick eyes and fidgets when new, stillness and a fixed stare when worn |
| Land creatures | For every kind: idle, move in its own way (walk, slither with a real body wave, glide, hover, drift, crawl), turn, attack (each kind of attack), take a hit, die, lie dead; the electrical flicker that stops at death. Special acts by kind: Regent reaches a soldier; Kite circles, marks, fires its daze shot; Bloat drifts and bursts; Gorger lies buried, erupts, drags down; Maul blasts magma, eats bodies and kit; Cinder breaks off a Maul, sets fires, eats, grows |
| The two machines | Shape-changing between forms: plates slide, turn and fold through each other, exact and silent, never like a living thing. Taker: travel, scout, open, spit, melt a body or wreck into a pool. Builder: travel, open, repair, raise cover, build a taker over several turns. Both: core exposed while working; shut down and fall apart |
| Freed aliens as soldiers | Their creature motions, changed for our side: carrying our kit, following orders. The Lash asks pardon before it shoots. Tenders gather, vote and merge in the ending |
| Husks | A taken soldier's motions, made wrong: delayed, off balance |
| The human worker | Per batch: early ones move wrongly and stiffly, late ones move like people. Calling out (the "duck calls"), begging, approaching the squad |
| Civilians and animals | Stand, walk, run, cower, fall. New: run toward soldiers, hide behind them, go to the craft when ordered; melt, panic and run at soldiers when hit by the fluid |
| Player craft | Fly, hover, land, take off; rotors (two on the Heron), ramps and doors; gear; damage and smoke |
| Enemy craft | Fly, hover, land, crash; doors; power glow; breaking up |
| Ground vehicles | Parked; burning; wrecked (they do not drive in battles) |
| Base modules and people | Doors, lifts, machines running, lights; people walking, working, sleeping, eating in the mess; the Scrub Room running |
| Trees, plants, cloth | Wind sway by strength of wind; fall when destroyed; burn |
| Buildings and clutter | Doors and windows; break in stages; burn; collapse |
| Weapons and items | Moving parts when fired and reloaded; shells; magazines; pin and lever on grenades |

Two things must be built before any of the motions can be shown, at the
start of phase 4:
- **The animation system.** The engine cannot bend a model. We write the
  part that reads bone positions for the current moment and bends the model
  on the graphics card, then test it on one soldier walking.
- **The model pipeline.** One agreed file format out of Blender, one loader
  in the game, one script that bakes textures and writes both. Every later
  model goes through it.

#### Which AI model does what

Fable is needed where the work is judged by eye or where one mistake is hard
to see. Opus and Sonnet are cheaper and good enough for code that can be
checked by building it and running the self-test.

| Work | Model | Why |
|---|---|---|
| Design sheets, lore, art direction with the user | Fable | Taste and consistency across the whole game |
| Blender modelling, texturing and rigging by script | Fable | Judged by reading renders; shape mistakes are subtle |
| Posing motions from reference frames; judging motion strips | Fable | Weight and timing are judged by eye |
| Shaders, lighting, weather, effects in `gfx.rs` | Fable | Small errors look almost right |
| Screenshot rounds and the harsh art-director judging | Fable | The standing brief depends on it |
| Animation system design (bone maths, blending between motions) | Fable | Wrong maths gives bent limbs that are hard to trace |
| Animation system and model loader, written to a finished design | Opus | Clear target, checked by a test model |
| Blender export script, file format reader and writer | Opus | Exact but checkable: export, load, compare |
| Game rules, new mechanics, enemy behaviour, save slot | Opus | Logic that the self-test and unit tests can check |
| Finding bugs in rules or drawing code | Opus, Fable if it stays unfound | Start cheaper |
| Wiring a finished model or motion into the game (tables, names, which motion plays when) | Sonnet | Repeated, pattern-following work |
| Menus and screens that only lay out text and boxes | Sonnet | Simple and easy to check by screenshot |
| Debug flags, self-test cases, small refactors, renames | Sonnet | Mechanical |
| Keeping `STATUS.md`, `CODE_MAP.md`, `NAMES.md` in step with the code | Sonnet, checked by Opus | Reading and comparing |
| Batch jobs: baking many textures, exporting many models with one proven script | Sonnet | The script is already proven by Fable |

Added 2026-11-13 for the game systems track. Same convention, new rows:

| Work | Model | Why |
|---|---|---|
| Trait sheet, fear list, earned-trait list, portrait stages (with the user) | Fable | Design and story; the user decides, Fable keeps it consistent |
| Creature rule sheets where `LORE.md` leaves gaps; species sheets for refugees | Fable | Lore first, then rules |
| Signpost mission scripts; the worker's voice ("duck calls") sound design; the ending | Fable | Judged by feel, not by a test |
| Lunar maps and dark-side lighting | Fable | Judged by eye |
| Records, loot, trait and fear rules, creature behaviour, base rules, mission framework, low gravity | Opus | Logic the self-test can check |
| Splitting the simulation from the drawing (for 3.0) | Opus | Large but checkable: results must not change |
| Table rows (weapons, items, modules), the Slag column, the soldier file screen, new self-test cases, removing the Trestle | Sonnet, checked by Opus | Pattern-following work |
| Keeping `MECHANICS.md`, `mechanics.json` and `NAMES.md` in step after each package | Sonnet, checked by Opus | Reading and comparing |

Rule of thumb per phase: Fable makes the first of each kind (first tree,
first creature, first motion) and judges all results; Opus builds the
systems; Sonnet repeats a proven recipe across the rest of the list.

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

More gaps found on 2026-11-13. The game systems track below fixes them:
- A soldier's mission count never goes up, and wounded soldiers never
  actually rest (package 1).
- Every win pays the same fixed Spine and Glow and one generic corpse.
  Nothing is read from the battlefield (package 2).
- Every enemy kind behaves the same way (packages 1 and 4).
- There are only two sides. Civilians do not exist yet (package 4).
- Soldiers have no working close-quarters attack. A stun baton is in the
  weapon list, but no rule makes it need a target on the next tile
  (package 2).
- The final land mission is still placed on Earth's globe and is won by
  killing everything (package 8).

### 2.0 — Our version
- One merged campaign: land and sea in the same war, balanced by playtest.
- The Registrar storyline (`LORE.md`) replaces today's two-finale
  structure.
- Deeper systems chosen from the ideas list below.
- New rules sheet written from our own design, not placeholder values.
- The repository was created early, on 2026-11-13, at the user's request:
  https://github.com/iannitram/defending-earth (public).

### 3.0 — Multiplayer, server-hosted
- Players claim countries and run their defence from inside them.
- Trade between players.
- Collaboration with the enemy as a real option, with a cost.
- Soldiers cross between players' games: a soldier taken in one game can
  appear as an enemy in another.
- Join or be wiped out (the user's idea, 2026-11-13): a player who loses is
  offered a choice, to join the Registrar's side and keep playing or to be
  wiped out. What a joined player controls is still open. See package 9 of
  the game systems track.

Groundwork that should start early so 3.0 is not a rewrite:
- Soldier, wound and history records stay small, self-contained and easy
  to write to a file and read back (also needed for saves in 1.0).
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

## Game systems track (added 2026-11-13, the user's request)

This is a second list of work, separate from the graphics pass. The
graphics pass is about how things look. This track is about rules and
records: the things decided in `LORE.md` that the game does not do yet.

It does not change the order of the graphics phases. Each package below
names the graphics phase it fits best with, so the rule and the picture
for the same thing can be made close together.

The plan comes from a read of the code. Nothing has been built yet.

### Words used here

- **Record:** a small set of numbers the game keeps about something, such
  as how many missions a soldier has fought.
- **Self-test:** a fixed battle the game plays by itself and compares with
  a stored result. If the result changes by accident, a rule was broken.
- **Dice:** the game's random numbers. They are "seeded": the same starting
  number always gives the same rolls, so a battle can be replayed exactly.
- **Slot:** a numbered place in a fixed table. Slots are never renumbered,
  because other parts of the game point at them by number.
- **Hook:** a marked place in the code where a rule for one thing can be
  added without touching the rules for everything else.
- **Size:** small is about one work session. Medium is a few. Large is
  many, split into parts.

### Facts that shape the whole track

- There is no save game yet. So nothing here can break a save. Every new
  record is kept small and plain, so the save slot (1.0) and multiplayer
  (3.0) can store it later.
- A soldier's mission count never goes up today. Traits, portraits and
  earned traits all need that fixed first. That is why package 1 comes
  first.
- Any new dice roll in the self-test's path changes its result. Each
  package says whether the self-test is safe. Where it is not, the stored
  result is re-made on purpose and compared.
- New rows go at the end of tables. Nothing is renumbered.
- Names inside the code stay as they are (for example the fields are still
  called `lattice` and `electrolyte`). Only what the player reads changed.

### The packages at a glance

| # | Package | Needs first | Pairs with graphics phase | Size | Model |
|---|---|---|---|---|---|
| 1 | Groundwork: honest records, Trestle removal, a behaviour hook per creature | nothing | 3 (no art needed) | 2 medium, 1 small | Opus; Sonnet for the Trestle, checked by Opus |
| 2 | SHOTGUN, KNIFE, SLAG, real loot | 1 | 6; Glow pools with 3 | 2 medium, 1 small to medium, 1 small | Opus for rules, Sonnet for table rows |
| 3 | Soldier traits, fears, earned traits, changing portraits | 1 | 4 | large | Fable for the sheets, Opus for rules, Sonnet for the file screen |
| 4 | Creature rules, one kind at a time | 1; Slag from 2 | 5; taker fluid and Cinder fires with 3 | large (7 medium parts) | Fable for sheets, Opus for rules, Sonnet for self-test cases |
| 5 | MESS HALL, SCRUB ROOM, REFUGEE QUARTERS, infection tax | real loot from 2 | 7 | medium | Opus for rules, Sonnet for rows and screens, Fable for icons |
| 6 | Freed aliens ("refugees") as soldiers | 1, 3 and 5 | 4 and 5 | large | Opus, with Fable for species sheets |
| 7 | Signpost missions and the human worker | 1; civilians from 4 help | 8 | medium, then small per signpost | Opus for the framework, Fable for scripts and sound |
| 8 | The Moon finale | 6, 7, and graphics phase 3 | 9 | large | Fable for ending and maps, Opus for rules |
| 9 | Multiplayer: join or be wiped out | records from 1, 3 and 6 | none (version 3.0) | large | Opus for the split, Fable for the design |

**Order:** 1, 2, 3, 4, 5, 6, 7, 8, 9. Packages 2 and 3 can be worked on at
the same time once 1 is in. Package 5 can start as soon as real loot (part
of package 2) is in.

### Package 1. Groundwork

No art needed. Can be done alongside graphics phase 3.

**1a. Mission records.** Opus. Medium. Needs nothing.
- Adds: each soldier gets a small history: missions fought, times wounded,
  times panicked, friends lost, kills by creature kind.
- Adds: each mission gets a kind (normal, signpost, Moon) and a gravity
  value.
- Adds: the campaign keeps a list of story events that have happened, and
  a list of lost soldiers (abducted, mind-taken, left on the field).
- Fixes: the mission count goes up after each battle. Wounded soldiers now
  really rest for some days.
- Self-test: safe, as long as no new dice are rolled.
- Everything else in this track depends on this part.

**1b. Remove the Trestle.** Sonnet, checked by Opus. Small. Needs nothing.
- Takes the Trestle out of the list of enemies that can appear.
- Its creature slot and its two item slots (the wreck and the disabled
  unit) are marked RESERVED and left empty. Nothing is renumbered.
- Its armour class stays in the table. The Lathe may want it later.
- Docs to bring in step: `MECHANICS.md`, `mechanics.json`, `NAMES.md`,
  `ART_BIBLE.md`.
- Self-test: safe.

**1c. A behaviour hook per creature.** Opus. Medium. Needs nothing.
- Today every enemy runs the same behaviour. This splits it so each kind
  has its own place to put rules. At first every kind behaves as it does
  today.
- Each unit gets a small status block: dazed, marked, melting, buried, how
  full it is, how many turns it has been working.
- Enemies that bite or claw all share one stand-in weapon row today
  ("NATURAL WEAPON"). Each kind gets its own.
- Done when: the self-test result is exactly the same as before.
- Package 4 needs this.

### Package 2. Shotgun, knife, Slag and real loot

Pairs with graphics phase 6 (weapons, items, materials). Collecting Glow
from pools pairs with phase 3, where the glowing fluid is drawn.

**2a. SHOTGUN.** Opus for the rule, Sonnet for the rows. Small to medium.
- Adds the gun and its shells as two new items, and one weapon row at the
  end of the weapon list. Added to the starting stock.
- Rule: several pellets per shot. Strong up close, weak far away.
- Self-test: safe if only shotguns roll pellets.

**2b. KNIFE.** Opus. Medium.
- Adds the knife as an item and a weapon row, modelled on the dive knife
  already in the sea table.
- Rule: soldiers cannot attack at close quarters today. This adds it: the
  target must be on the next tile, no ammunition is used, and the knife is
  carried as a second weapon.
- The Gorger and melting civilians are what make it matter.
- Self-test: safe.

**2c. SLAG.** Sonnet. Small.
- Adds Slag as a third material beside Spine and Glow, with a Slag cost on
  every workshop blueprint (zero for now).
- Shown in the workshop cost check, the stock readout and the market list.
- Needs the Maul rule (package 4) to produce it. Until then a debug flag
  can grant some for testing.
- Self-test: safe.

**2d. Loot from the battlefield.** Opus. Medium. Needs 1a.
- Replaces the fixed payout after every win.
- Glow comes from the pools still standing at the end. Spine comes from
  wrecks and bodies. Corpses are counted by creature kind. A Maul's corpse
  gives Slag by how full it was.
- Until pools exist (package 4), a base amount stays so the player does
  not run dry.
- Self-test: safe. The economy changes, so the rules sheet
  (`MECHANICS.md`) is updated.

### Package 3. Soldier traits, fears, earned traits, changing portraits

Pairs with graphics phase 4 (soldiers). Large. Needs 1a: without it the
mission count is always zero.

**3a. Traits.** Fable for the sheet, Opus for the rule.
- Each recruit is rolled one strength and one flaw.
- Each trait in the table has four parts: a name, whether it is a strength
  or a flaw, a **trigger** (what makes it show itself) and an **effect**.
- The eight starters are BRAVE, COLD, EMPATH, PROTECTIVE, RECKLESS, STEADY,
  SUPERSTITIOUS and LONER. What each does is in `LORE.md`.
- **Hidden until triggered.** The first time a trait's trigger fires in
  battle, the battle log says so and the trait is written on the soldier's
  file. Until then the file shows "UNKNOWN".

**3b. Fears.** A fear is a kind of flaw. It only bites where the feared
thing is present, so a soldier may serve for years before it shows.
- Every fear is built from two parts: **a thing**, and **how the game can
  tell the thing is here**. There are only two ways to tell:
  1. *Place:* the soldier is in or next to it.
  2. *Creature in sight:* the soldier can see a given kind of creature.
- **AFRAID OF WATER** (place). Suggested triggers, for the user to
  confirm: an underwater mission; standing next to a water tile; heavy
  rain. Two small jobs come with it. Land maps have no water tiles today
  (no rivers or ponds), so that trigger waits until they exist. And heavy
  rain needs a proper yes-or-no flag (today it is found by reading the
  weather's name).
- **AFRAID OF SHARKS** (creature in sight). It triggers when one given sea
  creature kind is in view. The sea campaign is deferred, so this fear
  sleeps on land. It needs no new sea design, only the "creature in sight"
  trigger.
- **"And so on."** The same two triggers cover any later fear. Examples to
  choose from, none decided: fire, the dark, heights, one land creature
  kind, being alone.
- **What a fear does:** while the thing is present, the soldier takes a
  morale hit at the start of each of their turns. Low morale leads to
  panic, as it already does.

**3c. Where the trait rules act.**
- STEADY: the accuracy rule, where wounds lower aim.
- BRAVE, COLD, EMPATH, LONER, SUPERSTITIOUS: the morale loss when a
  squadmate dies, and the panic check.
- PROTECTIVE and RECKLESS: reaction fire and the start of the soldier's
  turn.

**3d. Earned traits.** Opus.
- A list of rules. Each rule is a test on the soldier's history from 1a and
  the trait it grants (shell shock after too much, competence after
  surviving alone, others to be designed).
- Checked once per soldier after every mission, next to promotions.

**3e. Portraits that change with service.** Fable for the stages, Sonnet
for the wiring.
- The portrait drawing gets one more input: a service stage worked out
  from the mission count. No missions is new and jittery. Fifty is the
  thousand-yard stare.
- The stage is a plain number. Nothing about drawing goes into the rules.

**Self-test: at risk.** Rolling traits for a recruit adds dice rolls, and
every later roll would shift. The fix is a separate set of dice for traits,
seeded from the soldier's own number, so the main sequence does not move.

### Package 4. Creature rules

Pairs with graphics phase 5 (creatures). The taker's fluid and the
Cinder's fires pair with phase 3. Large overall. Needs 1c for all.

Each creature is its own medium job for Opus. Where `LORE.md` leaves a
gap, Fable writes the rule sheet with the user first. Sonnet adds one new
self-test case per creature. Build them one at a time, in this order:

1. **Bloat.** When it is shot, or when it reaches a target, it bursts
   into a cloud that blocks sight (the Kite's too) and chokes. The player
   can burst it over the enemy. It reuses the smoke already in the game.
   Choking is new: it stuns a little each turn. Ordinary smoke grenades do
   not choke.
2. **Kite.** It marks a soldier. Its daze shot does no damage but halves
   the soldier's time units and stops reaction fire for one turn. A Lash
   in range may shoot a marked soldier it cannot see, for as long as the
   Kite can see them. Killing the Kite, cover overhead, or smoke above
   ends the mark.
3. **Gorger.** It lies buried and unseen, feels footsteps, then erupts,
   seizes a soldier and drags the remains below. How close a soldier must
   be to spot it first is to settle. Its corpse becomes refugee food in
   package 5.
4. **Maul.** An animal with no weapon. It blasts magma (heat damage). It
   walks to corpses and dropped kit and eats them, and eats unconscious
   soldiers. Each meal makes it fuller. Its corpse gives Slag by fullness.
   It can never be captured for freeing. Its study topic stays.
5. **Cinder.** The Maul's young. It is in the creature table but never
   appears in a battle today. Cinders break off a full Maul or come out of
   its remains. They set fires, eat and, if left, grow. Whether one can
   become a full Maul within one battle is to settle. Needs the Maul
   first.
6. **Brooder, the taker.** A hovering machine that spits glowing fluid.
   - Adds pools: a tile can hold fluid, and the fluid hurts whoever stands
     in it.
   - Adds a third side: civilians. A civilian melts after one spit and
     then runs at soldiers. A soldier takes several spits, because armour
     soaks it up.
   - Adds a neutralising spray (a mid-game item with its own research
     topic) and the cube core as loot.
   - With a third side, every rule that asks "is that the enemy?" must be
     read again: how a battle ends, reaction fire, and what the player can
     target.
7. **Lathe, the builder.** It repairs machines, makes cover and, after
   several undisturbed turns beside a pool, builds one new taker. Heavily
   armoured while travelling, exposed while working. The sphere core is
   loot. Needs the Brooder first.

Self-test: safe as long as the Tender's path is not touched. The surface
research table has a fixed length, so adding the spray topic means making
it one row longer.

### Package 5. Mess Hall, Scrub Room, Refugee Quarters, infection tax

Pairs with graphics phase 7 (base modules and base people). Medium. Needs
real loot (2d) for Gorger meat. Refugee Quarters is only useful once
package 6 is in.

- Adds three modules. The module list grows from 17 to 20.
- Each module type gets a capacity: meals for the Mess Hall, scrubbing for
  the Scrub Room, beds for the Refugee Quarters.
- Food is used and scrubbing is done every day. The mess costs money to
  run. Scrubbing costs power and resources. The monthly report shows a
  line for each.
- **Infection tax.** The base keeps one infection number. Building and
  using alien-derived equipment raises it. Refugees do not: once freed
  they are clean. The Scrub Room lowers it, never to zero. A high number
  means sabotage, turned staff and attacks that begin inside the base.
- The Barracks text says it includes a mess today. It gets reworded.
- The program cannot be compiled (turned from code into a runnable game)
  until all three modules have a colour and an icon. That is a useful
  safety net, not a problem.
- Self-test: safe.
- Model: Opus for the rules, Sonnet for the rows and screens, Fable for
  the icons.

### Package 6. Freed aliens ("refugees") as soldiers

Pairs with graphics phases 4 and 5. Large. Needs packages 1, 3 and 5.

- Adds a species to each soldier: human, Regent, Tender or Lash. Each
  species has its own starting numbers.
- Adds a "free" action on the holding screen. It moves a captive into the
  squad list if the Refugee Quarters has room.
- A battle must be able to build one of our units from a creature, and
  draw it as that creature even though it is on our side.
- Refugees may not be able to wear human armour. To settle.
- The Maul can never be freed.
- Self-test: safe. Its soldiers are human.
- Model: Opus, with Fable for the species sheets.

### Package 7. Signpost missions and the human worker

Pairs with graphics phase 8 (environment, civilians). Medium for the
framework, small for each further signpost. Needs 1a. The civilian side
from package 4 helps.

- Adds a table of signposts. Each has a trigger and a mission script.
- Triggers are checked every day and after every battle: first live
  capture, first soldier taken, first Regent studied, share of
  alien-derived kit past a set level.
- **"The same face"** builds its enemies from the highest-ranked soldier
  in the lost-soldiers list from 1a.
- **The human worker** is a new unit. Whether it fights is to settle. It
  has a voice stage number. Its "duck calls" are sounds the game makes itself. They
  start digital and senseless and come closer to speech as the stage
  rises, batch by batch.
- A new site kind, Signpost, is added for these missions.
- Self-test: safe.
- Model: Opus for the framework, Fable for scripts and sound design.

### Package 8. The Moon finale

Pairs with graphics phase 9 (craft and the final sites). Large. Needs
package 6 (Tenders), package 7 (mission scripts) and graphics phase 3
(dark-side lighting).

- The final mission moves off Earth's globe to a target on the Moon. The
  research topic that finds it keeps its place in the code.
- **Low gravity:** longer throws, cheaper climbs, higher jumps.
- **Real lunar ground:** a new way to build a map from height data.
  Today's map builder makes its hills from random numbers, in three
  height steps, and cannot read real height data. So this is new work.
- **Dark side:** a light setting where the Registrar's own lights are the
  only light.
- **Winning** cannot mean "no enemy left alive", because the Registrar
  cannot be killed. The mission gets a scripted goal instead.
- **The vote.** How it goes depends on how many Tenders the player has
  freed. With enough, they vote and merge into a maker. What happens with
  too few is to settle.
- **The last word**, "there are many", goes on the end screen.
- Self-test: safe if gravity is normal unless a mission says otherwise.
- Model: Fable for the ending and the maps, Opus for the rules.

### Package 9. Multiplayer: join or be wiped out

Version 3.0. A design note only for now. No code and no graphics pairing.

- A player who loses may join the Registrar's side and keep playing, or be
  wiped out.
- It needs the plain records from packages 1, 3 and 6.
- It needs the simulation split from the drawing. Both live in one file
  today.
- Size: large. Model: Opus for the split, Fable for the design.

### Still to decide with the user

`LORE.md` does not settle these. They go to the user in question rounds
before the package that needs them starts.

1. Which of the eight starter traits are strengths and which are flaws,
   and their numbers (package 3).
2. Which fears beyond water and sharks, and what exactly sets off the
   fear of water (package 3).
3. Maul numbers: Slag from a full and an empty corpse, how fast it eats,
   how the Registrar controls it (packages 2 and 4).
4. What happens when the mess runs out; how many Gorgers a refugee needs;
   dead or alive (package 5).
5. Infection tax and scrubbing numbers; what an attack from inside the
   base looks like (package 5).
6. What each freed species can and cannot do (package 6).
7. The human worker's look and first appearance; how many batches;
   whether it fights (package 7).
8. More signposts and their triggers (package 7).
9. What a joined multiplayer player controls (package 9).
10. How many portrait stages there are between new and fifty missions
    (package 3).
11. How many freed Tenders are "enough" for the vote, and what the ending
    is with too few (package 8).
12. Whether a Cinder can grow into a full Maul within one battle; how
    close a soldier must be to spot a buried Gorger (package 4).

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
  deciphering. Note: `LORE.md` now says the Registrar's first messages
  mirror the players' own chat, used wrongly, and improve from there. The
  two versions need bringing together.

- From the lore sessions (details in `LORE.md`): infection tax on alien
  technology and scrubbing research; freeing captives to serve as soldiers;
  signpost missions set off by what the player has done ("The same face").
  These are now scheduled in the game systems track (packages 5, 6 and 7).
  Still unscheduled: base defence, including raids to recover a captive.

- The Mess Hall: a base building that feeds soldiers (running cost), sells
  researched stims and medicines, and, once refugees (freed aliens) exist,
  must be restocked with Gorgers to feed them (details in `LORE.md`). The
  building and the feeding are in package 5. Ordering stims and medicines
  there is not scheduled yet.

- Civilians on terror missions (the user's design, 2026-11-13): they run
  TOWARD the soldiers to be saved, and hide beside or behind them, which
  makes stealth hard. They only react to soldiers they can see or gunfire
  they can hear; otherwise they hide where they are. A soldier can spend a time unit to order one to "get
  to the extraction vehicle"; the civilian then runs to the craft and stays
  there. A civilian who reaches the craft, or is alive when the mission
  ends, counts as saved. Aliens can follow them to the craft to kill them.
- Melting panickers: civilians (and, after enough hits, soldiers) spat on
  by the taker (the Brooder) run at soldiers and damage them on contact;
  the pools left behind damage anyone who steps in them (details in
  `LORE.md`). Scheduled in package 4. The rest of the civilian design above
  (hiding, being ordered to the craft, counting as saved) is not scheduled
  yet.
- Footprints in snow: soldiers, civilians and aliens leave tracks. While it
  is still snowing the tracks fade over time. Aliens can see and follow
  tracks, how well depending on the species' intelligence; the player can
  read alien tracks the same way.

### Presentation
- Your own face on a soldier (the user's idea, 2026-11-13): the player
  gives the game a photograph and it is fitted to a soldier's face, to go
  with naming them. Belongs with the soldier appearance generator (graphics
  phase 4), and must work with the portrait stages that change with
  service. To settle: how the photo is fitted (a face texture on our head
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
