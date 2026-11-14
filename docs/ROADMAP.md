# DEFENDING EARTH — ROADMAP

Working draft 4, 2026-11-13. Companions: `ART_BIBLE.md` (look), `LORE.md`
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
- **The game systems track** (package 0 and packages 1 to 10, added
  2026-11-13): the rules and records behind the lore decisions in `LORE.md`.
  It has its own section further down, after the versions. It was revised the
  same day for the third lore session, which added package 10, the soldier
  pool. Package 0, the test bench, was added after that and comes first.

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
   Added 2026-11-13: close-up portraits that change with service, in six
   stages, with marks from events (scars, earned traits). The same six
   stages apply to the head and portrait of a soldier made in the soldier
   pool's character designer (package 10; sixth session, 2026-11-13: this
   replaces laying them over a player's own picture). The soldier pool
   screen is designed here too. Also here: ribbon designs (ten ribbons,
   five levels each), the ribbon board and the dress uniform (package 3f),
   and the head built from sliders. **Before
   any character or vehicle is modelled, the scale sheet is made** (see
   "The scale sheet" under the design and animation catalogue). Nothing in
   phases 4 to 9 is modelled until it exists and its one open number is
   settled.
5. Enemy creatures. The decided roster is ten kinds: three peoples, five
   animals and two machines (the taker and the builder). The Trestle is
   dropped from the design; it is still in the code until package 1b.
   The Cinder is a glowing cube-shaped rock that rolls. The Maul carries
   the Registrar's control spike. Refugee harnesses are designed here.
6. Weapon, armour and item designs across the three design steps. Now
   includes the shotgun, the knife and the material Slag.
7. Base modules. Now includes the Mess Hall, Scrub Room and Refugee
   Quarters. The inside of the base must also work as a battlefield
   (package 5b).
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
   agreed with the user (creatures: lore first, in `LORE.md`). Its size is
   declared in real units in the scale sheet (see below).
2. **Model.** Built in Blender by script, with baked textures, checked in a
   test render. The script reads its size from the scale sheet, and the
   automatic size checks must pass.
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
| Soldier portraits | The close-up face at six stages of service, from new and jittery to the thousand-yard stare. On top of the stage come event marks: one scar for each time wounded, and a change for each earned trait. Changed in the sixth session (2026-11-13): the portrait is a picture of the soldier's 3D head (see the head row below), so each stage and mark is made for the head, not laid over a player's picture. The earlier plan of two versions (the game's own drawn faces, and overlays over a player's picture) is replaced. Also the file entry for a strength or fear not yet seen ("UNKNOWN") | 6 stages, plus scars and about one mark per earned trait | 4 |
| Soldier pool screen | The page reached from the start screen: the list of pool soldiers, the character designer (face sliders, body, hair, voice), boxes to type a name, nickname and short background, the favoured-role choice, the "start with me" tick, the service record (every ribbon and citation ever earned, campaigns served, times fallen), and later the optional photo step. Sixth session (2026-11-13): replaces the picture import and crop step | 1 screen, with a photo step added later | 4 |
| The head built from sliders | A 3D head built once from a soldier's slider values when the soldier loads. The detailed face sliders, plus a simple body: build, height in a small range, skin, hair style and colour. The portrait is a picture of this head. The face slider list is to settle. Research note in package 10 | 1 head builder, many results | 4 |
| Ribbons | Ten ribbons (rifle, shotgun, knife, explosives, medic, scout, overwatch, Reach, survivor, creatures), five levels each. A higher level replaces the lower one on the board, so each level needs its own design. Names and designs are to settle | 10 ribbons times 5 levels, 50 designs | 4 |
| The dress uniform and ribbon board | The dress uniform, and the ribbon board on it where a soldier's ribbons are worn. To settle: how it looks, and where the player sees it (the soldier's file, the pool page) | 1 uniform, 1 board | 4 |
| The book of citations | The page where a hidden ability appears once one of the player's soldiers earns it, with how it was earned. To settle: its look | 1 screen | 4 |
| Soldier armour | Every armour, in its design step, worn and damaged. The first step looks present-day. New: three refugee harnesses (next row) | 6 today, then 9 | 4 and 6 |
| Freed aliens as soldiers ("refugees") | Regent, Tender and Lash with the control hardware removed, in our squad: markings and portrait. None can wear human armour, so each kind gets its own workshop-made harness, shaped to its body. The Regent uses Reach and cannot use human weapons. The Tender is small and frail. The Lash is a marksman; which weapons a freed Lash or Tender carries is to settle | 3 species, 3 harnesses | 4 and 5 |
| Land creatures | Every kind, plus its corpse or wreck and its captive form. People: Regent, Tender, Lash. Animals: Gorger, Kite, Bloat, Maul, Cinder. Machines: see the next row. The Maul has the Registrar's control spike driven into its molten core, and a wild look once the spike is gone. The Cinder is a glowing cube-shaped rock, rough, rounded, orange and tumbling, almost cute, in several sizes as it grows. It must never be confused with the Registrar's cubes, which are black, exact and still. The Gorger needs its buried signs: shifting ground and dust. The Trestle gets no design: it is to be removed (package 1b) and its table slot will stay empty | 11 in the code today, then 10 | 5 |
| The two machines | The taker (Brooder): hovering plates around a cube core; forms for travelling, scouting, spitting and melting. The builder (Lathe): blade legs around a sphere core; forms for travelling, repair, making cover and building a taker. Closed and armoured when moving, core exposed when working | 2 kinds, about 4 forms each (counted in the 10 above) | 5 |
| The human worker | Copies of real people, improving batch by batch: first plainly unfinished, last able to pass among civilians. They wear the faces of lost soldiers and taken civilians. Early batches are unarmed: no weapon in the hands at all. Late batches carry human weapons and dress as civilians | about 3 batches (number to settle) | 8, faces from 4 |
| Sea creatures | Every kind in the sea table (deferred) | 9 kinds | 5, later |
| Soldier weapons | Every weapon and its ammunition, held, slung, dropped. New: SHOTGUN with shells, and KNIFE | 32 in the weapon list today (14 land, 16 sea, 2 used only by creatures), then 34 | 6 |
| Other items | Grenades, smoke, trip mine, radar, sonar, trauma kit, flares, lights, pack. New: neutralising spray (mid-game) | about 10 today, then about 11 | 3 and 6 |
| Materials and recovered parts | Each material and fuel as an object in stores and in wrecks. SPINE (cut from wrecks and bodies), GLOW (the taker's fluid, in a pool and in a container). New: SLAG (Maul remains), the taker's cube core, the builder's sphere core, Gorger meat as food | 6 today, then 10 | 6 |
| Base modules | Every module, outside and inside, with broken versions and a "gone dark" version for sabotage. The insides join up into rooms and corridors that a battle can be fought in (package 5b). New: MESS HALL, SCRUB ROOM, REFUGEE QUARTERS. Renamed today: BREACHER BATTERY, REACH DAMPER, REACH WING | 17 today, then 20 | 7 |
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
| The ending | The freed Tenders' vote and merge into a maker, a form nobody has seen before; the end screen with the last word. There is no pass mark, so the scene must work with one Tender or with many | 1 scene | 9, after lore |
| Effects | Muzzle flash, tracers, smoke, fire, blasts, dust kicks, arcs, blood, debris. New: glowing taker fluid (spit, pools, on bodies), Bloat cloud, magma blast, shotgun spread, slow Moon dust, the ground shifting and dusting over a buried Gorger, the Maul's spike breaking, sparks from a rolling Cinder | about 15 today, then about 23 | 3, then each phase |

Motions each group needs:

| Group | Motions |
|---|---|
| Soldiers | Stand, walk, run, crouch walk, kneel, go prone, turn; aim, fire (single, burst, auto), reload, throw, use item, heal another; climb, drop, open door; hit reactions by body part, stagger, panic, burning; fall dead in several ways, lie wounded, be carried; idle fidgets. Each weapon class changes the carry, aim and reload. New: pump and fire the shotgun, draw the knife and strike; fear reactions (flinch, freeze, back away), one set for each land fear: the dark, fire, tight spaces and the Glow; lie unconscious, be dragged clear, withdraw to the craft; struggle when an early human worker clings on; use the handheld radar; low-gravity versions of walk, jump, throw and fall for the Moon |
| Soldier portraits | Small face motions for each of the six service stages: quick eyes and fidgets when new, stillness and a fixed stare when worn. Since the sixth session the portrait is a picture of the soldier's 3D head, so these motions are to settle against that |
| Land creatures | For every kind: idle, move in its own way (walk, slither with a real body wave, glide, hover, drift, crawl), turn, attack (each kind of attack), take a hit, die, lie dead; the electrical flicker that stops at death. Special acts by kind: Regent reaches a soldier; Kite circles, marks, fires its daze shot; Bloat drifts and bursts; Regent hesitates and is recycled ("The split vote"); Gorger lies buried with the ground shifting above it, erupts, drags down; Maul blasts magma, eats bodies and kit, loses its spike and goes wild; Cinder breaks off a Maul, rolls and tumbles as a cube, sets fires, eats, swells a size at a time |
| The two machines | Shape-changing between forms: plates slide, turn and fold through each other, exact and silent, never like a living thing. Taker: travel, scout, open, spit, melt a body or wreck into a pool. Builder: travel, open, repair, raise cover, build a taker over several turns. Both: core exposed while working; shut down and fall apart |
| Freed aliens as soldiers | Their creature motions, changed for our side: wearing a harness, following orders. The Regent reaches, with no human weapon in hand. The Tender stops and senses Reach and machines. The Lash asks pardon before its first shot at each target. Tenders gather, vote and merge in the ending |
| Husks | A taken soldier's motions, made wrong: delayed, off balance |
| The human worker | Per batch: early ones move wrongly and stiffly, late ones move like people. Early: run at the squad, cling to a soldier, call out (the "duck calls"). Late: stand among civilians, beg, call soldiers by name, then draw a weapon |
| Civilians and animals | Stand, walk, run, cower, fall. New: run toward soldiers, hide behind them, go to the craft when ordered; melt, panic and run at soldiers when hit by the fluid |
| Player craft | Fly, hover, land, take off; rotors (two on the Heron), ramps and doors; gear; damage and smoke |
| Enemy craft | Fly, hover, land, crash; doors; power glow; breaking up |
| Ground vehicles | Parked; burning; wrecked (they do not drive in battles) |
| Base modules and people | Doors, lifts, machines running, lights; a module going dark; people walking, working, sleeping, eating in the mess; the Scrub Room running |
| Trees, plants, cloth | Wind sway by strength of wind; fall when destroyed; burn |
| Buildings and clutter | Doors and windows; break in stages; burn; collapse |
| Weapons and items | Moving parts when fired and reloaded; shells; magazines; pin and lever on grenades |

Three things must be built before any model or motion can be shown, at the
start of phase 4. The first one comes before every character or vehicle
model:
- **The scale sheet.** See the next section.
- **The animation system.** The engine cannot bend a model. We write the
  part that reads bone positions for the current moment and bends the model
  on the graphics card, then test it on one soldier walking.
- **The model pipeline.** One agreed file format out of Blender, one loader
  in the game, one script that bakes textures and writes both. Every later
  model goes through it.

#### Animation list (proposed, to settle with the owner)

Added 2026-11-13 at the owner's request. **This whole list is proposed. The
owner has not agreed it.** It is drawn from rules already decided. It feeds
graphics phase 4 (soldiers; the animation system is built at its start),
phase 5 (creatures and refugees) and phase 8 (the human worker). It sits
beside the "Motions each group needs" table above, which is the older and
wider list. Entries with no support in `LORE.md` or `NAMES.md` say "to
settle".

*Soldiers, moving*
- Walk, run, injured run, panicked run.
- Moving while mind-taken (a Husk). To settle: how a taken soldier moves
  and whether it is a separate set.
- Kneel down and stand up. Going up or down a level (climb, drop).

*Soldiers, acting*
- Snap shot, aimed shot, auto burst, shotgun shot, knife attack, reload.
- Throw: grenade and smoke grenade. Throwing a flare is to settle (flares
  are named only as a light).
- Use a TRAUMA KIT. Go on OVERWATCH. Reaction shot.

*Soldiers, states*
- Idle, idle wounded.
- Panic. To settle: whether it looks frozen or cowering.
- Stunned (the lore has stun weapons; how a stunned soldier falls is to
  settle). Death (shot, fire, blast).
- A fear biting: the dark, fire, tight spaces, the Glow.
- A hidden trait showing itself.

*Refugees*
- Regent using Reach, Tender scouting, Lash aiming and firing (the Lash asks
  pardon before its first shot at each target).

*Creatures* (every kind in the land roster: Regent, Tender, Lash, Gorger,
Kite, Bloat, Maul, Cinder, the taker, the builder)
- For each: idle, move, attack, hit, death.
- Special ones the lore already states: a buried Gorger showing signs and
  coming up; the Maul going wild when its control spike is destroyed;
  Cinders rolling, breaking off a Maul and growing; the taker spitting
  Glow.

*Human worker*
- One set per batch stage as it improves (number of batches to settle).

*Replay*
- Cinematic camera moves. To settle: what the cinematic view is.

#### The scale sheet (decided 2026-11-13 by the user and the lead)

Made at the start of graphics phase 4, before any character or vehicle is
modelled. It is also a step in the catalogue above (steps 1 and 2).

- **One small sheet of sizes.** Both the game and the Blender model scripts
  read it. There is only one copy.
- **One anchor.** A battlefield square is a set number of metres across.
  That number is **to settle with the user, and it must be settled before
  modelling starts.** A soldier is 1.8 m tall.
- **Everything else is worked out from the anchor:** door widths, low cover
  (waist high) and full cover (head high), ceiling heights, weapon ranges
  in metres, throw distance, and jump height on the Moon.
- **Each object is declared in real units.** For example: the Heron's cabin
  seats 12, and each seat is so wide. A script works out the object's size
  and how many squares it covers.
- **Automatic checks.** Every model fits its squares. Soldiers fit through
  every door. A size chart is printed with a soldier standing beside each
  creature and vehicle.
- **Honest cheats are written down once, in the sheet.** For example:
  rotors overhang but do not block squares, and cabins are a little tighter
  than real ones. True-size vehicles are very large next to a playable map.

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
| Animation system and model loader, written to a finished design | Fable | Bone and blending mistakes show as bent limbs, seen only by eye |
| Blender export script, file format reader and writer | Opus | Exact but checkable: export, load, compare |
| Game rules, new mechanics, enemy behaviour, save slot | Opus | Logic that the self-test and unit tests can check |
| Rule bugs | Opus, Fable if it stays unfound | Start cheaper |
| Drawing, lighting and motion bugs | Fable | Small errors look almost right and are seen only by eye |
| Which motion plays when, and the timing of blends between motions | Fable | Judged by watching it move |
| Table rows and names for a finished model or motion | Sonnet | Repeated, pattern-following work |
| Menus and screens that only lay out text and boxes | Fable for the layout, Sonnet builds it | Simple and easy to check by screenshot |
| Debug flags, self-test cases, small refactors, renames | Sonnet | Mechanical |
| Keeping `STATUS.md`, `CODE_MAP.md`, `NAMES.md` in step with the code | Sonnet, checked by Opus | Reading and comparing |
| Batch jobs: baking many textures, exporting many models with one proven script | Fable | Fable runs the proven script and looks at every result; a bad bake is only caught by eye |

Added 2026-11-13 for the game systems track. Same convention, new rows:

| Work | Model | Why |
|---|---|---|
| Trait sheet, fear list, earned-trait list, portrait stages (with the user) | Fable | Design and story; the user decides, Fable keeps it consistent |
| Creature rule sheets where `LORE.md` leaves gaps; species sheets for refugees | Fable | Lore first, then rules |
| Signpost mission scripts; the worker's voice ("duck calls") sound design; the ending | Fable | Judged by feel, not by a test |
| Lunar maps and dark-side lighting | Fable | Judged by eye |
| Records, loot, trait and fear rules, creature behaviour, base rules, mission framework, low gravity | Opus | Logic the self-test can check |
| Splitting the simulation from the drawing (for 3.0) | Opus | Large but checkable: results must not change |
| Table rows (weapons, items, modules), the Slag column, new self-test cases, removing the Trestle | Sonnet, checked by Opus | Pattern-following work |
| The soldier file screen | Fable for the layout, Sonnet builds it, checked by Opus | A screen that lays out boxes and text |
| Keeping `MECHANICS.md`, `mechanics.json` and `NAMES.md` in step after each package | Sonnet, checked by Opus | Reading and comparing |

Added later on 2026-11-13, after the third lore session:

| Work | Model | Why |
|---|---|---|
| The six portrait looks, scars and earned-trait marks | Fable | Judged by eye |
| Building the head from slider values (10b) | Fable | A face that is slightly wrong is caught only by eye. Sixth session (2026-11-13): replaces drawing overlays on a player's picture |
| The photo step (10c) | Opus | Exact work with geometry and files. Sixth session (2026-11-13): replaces the picture crop |
| Soldier pool storage on disk and how pool soldiers arrive in a campaign | Opus | The game's first player file; a mistake here loses or leaks someone's soldiers |
| The soldier pool screen and its name box | Fable for the layout, Sonnet builds it, checked by Opus | A screen that lays out boxes and text |
| Sides (wild, civilian), ways to lose a soldier alive, mission goals | Opus | Rules that touch how every battle ends |
| The base-interior map builder | Opus for the map and rules, Fable for the look | Checkable layout first, then judged by eye |
| How much each Tender weakens the Seed (the curve) | Fable for the curve, Opus for the rule | Feel first, then a plain number rule |
| The human worker's text lines and generated calls | Fable | Judged by feel |

Added later on 2026-11-13, for package 0, the test bench:

| Work | Model | Why |
|---|---|---|
| The split of the bench's parts, and the format of its reports (event report, dice log, win-rate and money tables) | Fable | Design: a bad format hides the numbers the sub-agents must read |
| Separating the rules from the drawing, so the same rule code runs with no window | Opus | Large but checkable: results must not change |
| The proper save system (the file, its version, what is stored) | Opus | A mistake here loses or corrupts a campaign |
| The three scripted players (careless, average, careful), the report writers, the command-line options | Sonnet, checked by Opus | Pattern-following work with a clear target |
| The IRON MAN and FIXED DICE options on the new-campaign page | Fable for the layout, Sonnet builds it, checked by Opus | Menu wiring on top of the save system |
| The scale sheet, its size checks and the size chart | Fable for the design, Sonnet for the check scripts | Sizes and cheats are judged by eye; the checks are mechanical |

**Working rule (decided 2026-11-13, amended the same day).** Design decisions are made by Fable. All design and animation jobs are done by Fable itself, not by a sub-agent: the look, models, textures, pictures, motion, and judging any of them. Other actions go to a sub-agent on the smallest model that can reasonably do them: rule code, editing documents, commits, pushes and mechanical batch jobs.

Rule of thumb per phase: Fable makes everything that is judged by eye (every model, texture, motion, picture and screen layout) and judges all results; Opus builds the rule systems and file handling; Sonnet does table rows, wiring to a finished layout, documents and checks.

Changed 2026-11-13 after the amendment: moved to Fable: the animation system and model loader, which motion plays when, batch bakes and exports, portrait overlays, the head builder (10b), drawing and motion bugs, and screen layouts. Rule code stays with Opus and Sonnet.

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
- There are only two sides. Civilians and wild animals do not exist yet
  (packages 1c and 4).
- Soldiers have no working close-quarters attack. A stun baton is in the
  weapon list, but no rule makes it need a target on the next tile
  (package 2).
- The final land mission is still placed on Earth's globe and is won by
  killing everything (package 8).

Three more, found while planning the third-session decisions:
- The game writes nothing to disk that belongs to the player. The soldier
  pool will be its first player file (package 10).
- A soldier cannot be lost alive. A knocked-out soldier counts as dead,
  nobody can withdraw, and the enemy never takes anyone. So the Registrar
  can never "read" a soldier (package 1a-2).
- There is no map of the inside of a base, so no battle can start there
  (package 5b).

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
- Still open for 3.0: what happens to a soldier-pool soldier (name and
  slider values) when that soldier crosses into another player's game.
  Pool data is private (package 10), so the safe starting answer is that
  it never leaves the player's machine. (Sixth session: there is no
  stored picture any more, only slider values.)

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

Added after that on 2026-11-13, decided by the user and the lead: **package
0, the test bench.** It is the first package of the track, before every
other one. It is numbered 0 so that the numbers of packages 1 to 10 and every
cross-reference to them stay as they were.

Revised later on 2026-11-13 for the third lore session ("Third session:
the open questions" in `LORE.md`). What changed:
- New **package 10, the soldier pool** (the user's idea).
- New part **1a-2**: ways to lose a soldier alive.
- New part **5b**: a map of the inside of the base.
- Package 1c now also adds the wild side and the civilian side.
- Packages 3, 4, 5, 6, 7 and 8 are rewritten where the answers changed
  them.

### Words used here

- **Record:** a small set of numbers the game keeps about something, such
  as how many missions a soldier has fought.
- **Self-test:** a fixed battle the game plays by itself and compares with
  a stored result. If the result changes by accident, a rule was broken.
- **Test bench:** a way to run battles and whole campaigns from the command
  line, with no window, and print reports. A test tool, not a second game.
- **Seed:** the one number a campaign starts from. Each battle's dice start
  from it and the battle's number, so the same seed replays the same battle
  exactly.
- **Dice:** the game's random numbers. They are "seeded": the same starting
  number always gives the same rolls, so a battle can be replayed exactly.
- **Slot:** a numbered place in a fixed table. Slots are never renumbered,
  because other parts of the game point at them by number.
- **Hook:** a marked place in the code where a rule for one thing can be
  added without touching the rules for everything else.
- **Side:** which team a unit is on. Today there are two: ours and the
  enemy's.
- **Goal:** what ends a mission in a win. Today the only goal is "no enemy
  left alive".
- **Overlay:** a see-through layer drawn on top of a picture, such as grime
  or a scar.
- **Size:** small is about one work session. Medium is a few. Large is
  many, split into parts.

### Facts that shape the whole track

- There is no save game yet. So nothing here can break a save. Every new
  record is kept small and plain, so the save slot (1.0) and multiplayer
  (3.0) can store it later. Package 0 builds the proper save system, so
  from then on the records it writes are a file format that must be
  versioned.
- A soldier's mission count never goes up today. Traits, portraits and
  earned traits all need that fixed first. That is why package 1 comes
  first among the packages that change the rules (package 0 comes before it).
- Any new dice roll in the self-test's path changes its result. Each
  package says whether the self-test is safe. Where it is not, the stored
  result is re-made on purpose and compared.
- New rows go at the end of tables. Nothing is renumbered.
- Names inside the code stay as they are (for example the fields are still
  called `lattice` and `electrolyte`). Only what the player reads changed.
- The game writes no player file today. The soldier pool (package 10) is
  the first. It lives in the player's own data folder, never in the
  project folder, because the project is published.
- Exact numbers are set with the test bench, then checked by playing, once
  each system exists. Until then every such number is a named value kept in
  one place, so it is easy to find and change. The bench runs thousands of
  battles with the same seeds, so a change to one number can be compared
  fairly.

### The packages at a glance

| # | Package | Needs first | Pairs with graphics phase | Size | Model |
|---|---|---|---|---|---|
| 0 | The test bench: battles and campaigns with no window, repeatable dice, scripted players, event report, dice log, agent play, proper save system, IRON MAN and FIXED DICE | nothing | none; phase 3 may come first or run alongside | medium to large (an estimate: the tangle of rules and drawing is not measured yet) | Fable for the split and report format, Opus for separating rules from drawing, Sonnet for scripted players, reports and menu options to Fable's layout |
| 1 | Groundwork: honest records, ways to lose a soldier alive, Trestle removal, a behaviour hook per creature, the wild and civilian sides | 0 | 3 (no art needed) | 3 medium, 1 small | Opus; Sonnet for the Trestle, checked by Opus |
| 2 | SHOTGUN, KNIFE, SLAG, real loot | 1 | 6; Glow pools with 3 | 2 medium, 1 small to medium, 1 small | Opus for rules, Sonnet for table rows |
| 3 | Soldier strengths, fears, earned traits, six-stage portraits, ribbons (3f) and footprints (3g) | 1 | 4 | large | Fable for the sheets, the six looks and the ribbon and citation design, Opus for rules, Fable for overlays, Sonnet for the file screen to Fable's layout |
| 4 | Creature rules, one kind at a time, with wild Mauls and Cinders | 1; Slag from 2 | 5; taker fluid and Cinder fires with 3 | large (7 medium parts, the Maul part larger) | Fable for sheets, Opus for rules, Sonnet for self-test cases |
| 5 | MESS HALL and hunger, SCRUB ROOM, REFUGEE QUARTERS, infection and sabotage; 5b the base-interior map | real loot from 2; 1a | 7 | medium, plus 5b large | Opus for rules and the map, Sonnet for rows and screens to Fable's layout, Fable for icons and the interior look |
| 6 | Freed aliens ("refugees") as soldiers, with harnesses | 1, 3 and 5 | 4 and 5 | large | Opus, with Fable for species sheets |
| 7 | Five signposts and the human worker | 1 (all parts); civilians from 4; 5b for "The open door" | 8 | medium to large for the framework, medium for the worker, small per signpost | Opus for the framework and rules, Fable for scripts, lines and sound |
| 8 | The Moon finale, with no pass mark | 6, 7, and graphics phase 3 | 9 | large | Fable for ending, maps and the curve, Opus for rules |
| 9 | Multiplayer: join or be wiped out | records from 1, 3 and 6 | none (version 3.0) | large | Opus for the split, Fable for the design |
| 10 | The soldier pool: 10a storage, screen, typing and arrival; 10b the hand-driven character designer; 10c the photo step | 10a nothing; 10b needs 10a, 1a and 3e; 10c needs 10b | 4 | 2 medium, and 10c to measure | Opus for storage and arrival, Fable for the head builder and the slider list, Sonnet for the screen to Fable's layout |

**Order:** 0, then 1 and 10a, then 2, 3, 10b, 4, 5, 6, 7, 8, 9. Part 10c (the photo step) comes after 10b.
- Package 0 comes before every other systems package. Graphics phase 3 may
  come first or run alongside it.
- Package 10a needs nothing, so it can be built beside package 1, once
  package 0 is in.
  The player can then make their soldiers, without pictures, straight away.
- Packages 2 and 3 can be worked on at the same time once 1 is in.
- Package 10b (the hand-driven designer) follows 3e, because the six
  stages and marks apply to the head it builds. Package 10c (the photo
  step) comes last, after 10b.
- Package 3f (ribbons) needs 3d and 1a, because it counts what soldiers
  do. Its level targets come from the test bench (package 0).
- Package 5 can start as soon as real loot (part of package 2) is in. Part
  5b can wait until just before package 7.

### Package 0. The test bench (decided 2026-11-13 by the user and the lead)

No art needed. First package of the track. Graphics phase 3 may come first
or run alongside.

**What it is.** A way to run battles and whole campaigns from the command
line with no window, and print reports. It is a test tool, not a second game
to maintain.

**The real work.** The rules and the drawing are tangled together today,
mostly in `src/tactical.rs` and `src/main.rs`. The rules must be pulled apart
from the drawing so they can run with no window. The size is an estimate
(medium to large) because the tangle has not been measured. **The bench must
use the exact same rule code as the real game, never a copy.** This is the
same split that versions 3.0 and 4.0 and package 9 need.

**The parts.**
- **Repeatable dice.** Every campaign starts from a seed. The campaign has
  its own dice, and each battle has its own, started from the campaign seed
  and the battle's number. Visual things use separate dice that are never
  logged. So the same seed replays the same battle exactly.
- **Scripted players, built in, at three skill levels:** careless, average,
  careful. They play thousands of battles or campaigns in minutes, with no AI
  cost. This is where the statistics come from: win rates, losses, money over
  time, and how many freed Tenders are alive at the end.
- **Event report.** Every signpost, trait trigger, fear and other event is
  listed with how often it fired. Anything at zero is broken or unreachable.
- **Dice log.** Every rule roll is logged with a label (who, what, odds,
  result). The player sees this roll log only after the battle, on the
  debrief. The bench compares the chance shown to the player with what
  actually happened over thousands of tries, and flags any gap. If the screen says 100 percent, it
  must mean 100 percent.
- **Agent play.** A text mode where the game describes the situation, an AI
  agent types an order, and the game reports the result. A few campaigns
  played this way find exploits a scripted player never tries. A person can
  type into it too.
- **A proper save system.** It needs the same save and seed work, so it is
  built here. Normal play: save at any moment, many slots, plus an autosave
  at the start of each turn. Each save records the game version. Until 1.0 an
  old save that no longer fits is refused with a clear message; from 1.0 on,
  old saves are upgraded. Details in `LORE.md`, "Fifth session".
- **IRON MAN MODE.** Chosen on the new-campaign page, next to the seed, and
  locked for the whole campaign. One save slot, saved by the game after every
  decision. When the campaign ends, won or lost, the save is kept as a
  finished record that can be looked at but not played on.
- **FIXED DICE.** Chosen on the new-campaign page and locked for the whole
  campaign. The save keeps where the dice are, so reloading and doing the
  same thing gives the same result. Doing something else first may give a
  different result. This stops reloading to fish for a better roll.

**How it is used.** Sub-agents read the reports, change one number, re-run
with the same seeds, and compare. Several can test different changes at once.

**What it does not do.** It measures whether the maths is balanced. It does
not measure whether the game is fun or readable. That is still judged by
playing.

**Every later systems package** says its numbers are tuned with the test
bench ("set with the test bench, then checked by playing").

**Models.** Fable designs the split and the report format. Opus does the
separation of rules from drawing. Sonnet writes the scripted players, the
reports and the menu options, to Fable's layout. The save system is Opus work, as the save slot
already is in the model table.

**To settle.** The report file format and the exact command-line options.
Which numbers the bench changes first. The open questions from the fifth
session are numbers 31 to 34 in "Still to decide with the user".

**Self-test:** the separation must leave the stored result exactly the same.

**Rule fixes decided in the fourth session.** A code review found six rules
that were not what was intended. They are made once the rules can run in
the test bench, and their numbers are set there, then checked by playing.
Details are in `LORE.md` under "Fourth session (2026-11-13): rule fixes
found in the code review".
- Dark and sight: darkness cuts human sight, light restores it, aliens and
  refugees see in the dark.
- Overwatch: reaction shots only for units put on OVERWATCH, the same for
  soldiers and aliens.
- Movement highlight: reachable squares use the real walking rule.
- Close attacks: each close-fighting creature has its own attack numbers.
- Enemy craft shots: hit as often as their accuracy number says.
- Reach roll: one roll for soldiers and aliens alike.

**How the code is restructured for the bench.** Decided in the fifth session
(2026-11-13). The rules for each step are in `LORE.md`, "Fifth session
(2026-11-13): dice, shot odds, battle rules and saves". The order inside
package 0:
1. Dice and shot odds, together. **Built 2026-11-13, not yet committed.**
   Three kinds of dice with a roll log, and one shot-odds rule for the shown
   and rolled number, with height, range and cover (decided in the eighth
   session, see `LORE.md`). Not built: the ROLL LOG screen, the seed on the
   new-campaign page, saves, IRON MAN MODE and FIXED DICE.
2. Battle rules, built in slices. Each slice leaves the game running:
   - Slice 1: shooting and damage. **Built 2026-11-13, not yet committed.**
     The shot rules return events and need no window. It covers stray
     shots (the missed shot flies on along its tracer), cover hits,
     scenery damage (rocks, blocks, crystals, trees, walls), morale after
     friendly fire, a record of each friendly-fire incident, and the new
     damage roll of 50 to 150 percent. Decided in the eighth session's
     later part and the ninth session, see `LORE.md`. Items marked "to
     confirm" there are built as written.
   - Slice 2: movement and overwatch.
   - Slice 3: morale.
   - Slice 4: alien moves.
   - Slice 5: Reach and explosives. Blasts that hurt your own side are
     written to the incident record.
3. Campaign state, then campaign actions (hire, build, research, make, buy,
   sell).
   Decision G (eighth session): the same seed gives the same campaign
   events whatever the timing of the player's clicks. Step 3 does this by
   giving the world, the recruits and the alien crews their own dice. Not
   built now.

A finished battle hands the campaign one outcome record. Battle replays are
planned for but not built now.

### Package 1. Groundwork

No art needed. Can be done alongside graphics phase 3.

**1a. Mission records.** Opus. Medium. Needs nothing.
- Adds: each soldier gets a small history: missions fought, times wounded,
  times panicked, friends lost, kills by creature kind.
- Adds: each mission gets a kind (normal, signpost, Moon), a gravity
  value and a goal. Every mission's goal starts as "no enemy left alive",
  as today. Packages 7 and 8 add other goals.
- Adds: the campaign keeps a list of story events that have happened, and
  a list of lost soldiers (abducted, mind-taken, left on the field). Each
  entry keeps the soldier's name, look, numbers and rank, so a copy can be
  built from it later.
- Fixes: the mission count goes up after each battle. Wounded soldiers now
  really rest for some days.
- Self-test: safe, as long as no new dice are rolled.
- Everything else in this track depends on this part.

**1a-2. Ways to lose a soldier alive.** Opus. Medium. Needs 1a. New.
- Why: the human worker first appears "after the Registrar has read one of
  the player's soldiers". Today that can never happen. A knocked-out
  soldier is counted as dead, the squad cannot withdraw, and the enemy
  never takes anyone.
- Adds: a soldier can be unconscious without being dead. They can be woken
  or carried.
- Adds: a withdraw action. The squad leaves, and anyone not aboard is left
  on the field.
- Adds: the enemy can take a soldier, by Reach or by carrying them off.
  Today its Reach can only frighten.
- Adds: after the battle, every soldier taken or left behind is written
  into the lost-soldiers list from 1a.
- The Maul's rule "eats unconscious soldiers" (package 4) also needs this.
- Self-test: at risk if anyone is knocked out in the fixed battle. Check,
  and re-make the stored result on purpose if it moves.

**1b. Remove the Trestle.** Sonnet, checked by Opus. Small. Needs nothing.
- Takes the Trestle out of the list of enemies that can appear.
- Its creature slot and its two item slots (the wreck and the disabled
  unit) are marked RESERVED and left empty. Nothing is renumbered.
- Its armour class stays in the table. The Lathe may want it later.
- Docs to bring in step: `MECHANICS.md`, `mechanics.json`, `NAMES.md`,
  `ART_BIBLE.md`.
- Self-test: safe.

**1c. A behaviour hook per creature, and more sides.** Opus. Medium. Needs
nothing.
- Today every enemy runs the same behaviour. This splits it so each kind
  has its own place to put rules. At first every kind behaves as it does
  today.
- Each unit gets a small status block: dazed, marked, melting, buried,
  held, how full it is, how much it has grown, how many turns it has been
  working.
- Enemies that bite or claw all share one stand-in weapon row today
  ("NATURAL WEAPON"). Each kind gets its own.
- **Two new sides, added together: wild and civilian.** Wild units obey
  nobody and attack whoever is nearest. Civilians are bystanders. Doing
  both at once means the rules below are re-read once, not twice.
- **A home side.** Each unit remembers which side it started on. Today a
  unit released from Reach always goes to the enemy, whoever it belonged
  to. It will go home instead.
- **Enemy units stop pointing at a soldier.** Today every enemy quietly
  points at soldier number 0 in the squad list. A taken alien that died
  while on our side would mark that soldier dead. The pointer becomes
  "none" for anything that is not one of our soldiers.
- With more than two sides, every rule that asks "is that the enemy?" is
  read again: who the enemy hunts, how a battle ends, what the player may
  target, who counts as a kill or a captive afterwards, and which drawing
  a unit gets. Reaction fire already asks "not my side", so it is fine.
- Wild units take their turn at the end of the enemy's turn. That is the
  simplest way, with no new turn.
- No wild unit or civilian appears in any battle yet. Package 4 puts them
  there.
- Done when: the self-test result is exactly the same as before.
- Packages 4 and 7 need this.

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
- The Gorger, melting civilians and clinging human workers are what make
  it matter.
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
- Gorger corpses are counted as Gorger meat. Dead ones are fine. The mess
  (package 5) feeds refugees with it.
- How much Slag a full or an empty Maul gives is a named value, set with
  the test bench, then checked by playing.
- Until pools exist (package 4), a base amount stays so the player does
  not run dry.
- Self-test: safe. The economy changes, so the rules sheet
  (`MECHANICS.md`) is updated.

### Package 3. Soldier strengths, fears, earned traits, changing portraits, ribbons

Pairs with graphics phase 4 (soldiers). Large. Needs 1a: without it the
mission count is always zero.

**3a. Two pools: strengths and fears.** Fable for the sheet, Opus for the
rule.
- There are two separate lists. Every recruit is dealt **one strength and
  one fear**.
- **Strengths (8).** BRAVE, COLD, EMPATH, PROTECTIVE, RECKLESS, STEADY,
  SUPERSTITIOUS and LONER. All eight are strengths. Each has its catch
  built in (for example EMPATH can tell a copy from a person, but is hurt
  by civilian deaths). What each does is in `LORE.md`.
- **Fears (6).** The dark, fire, tight spaces, the Glow, water and sharks.
  Fears are the flaws.
- **Water and sharks are sea-only.** They do nothing on land and stay
  asleep until the sea campaign exists.
  - Proposed, to confirm with the user: they are not dealt at all in a
    land campaign. Otherwise a recruit could get a flaw that can never
    show. `LORE.md` only says they "stay asleep".
- Each entry has a name, a **trigger** (what makes it show itself) and an
  **effect**.
- **Hidden until triggered.** The first time a strength or a fear shows
  itself in battle, the battle log says so and it is written on the
  soldier's file. Until then the file shows "UNKNOWN". The soldier keeps
  two small "seen" marks, one for each.
- The numbers behind each strength are set with the test bench, then
  checked by playing.

**3b. The four land fears.** Opus. A fear only bites where the feared
thing is present, so a soldier may serve for years before it shows.

| Fear | How the game can tell | Ready today? |
|---|---|---|
| The dark | The battle is at night. The game already has a night test | Yes |
| Fire | A burning tile within a few tiles of the soldier | Yes |
| Tight spaces | Count the blocked tiles around the soldier: walls and high ground | Yes |
| The Glow | A pool, or someone hit by the fluid, in the soldier's sight | No. Waits for the Brooder (package 4) |

- **What a fear does** (proposed; `LORE.md` only says a fear "bites" where
  the thing is present, so this goes to the user first): while the thing
  is present, the soldier takes a morale hit at the start of each of their
  turns. Low morale leads to panic, as it already does.
- **The hit must be big enough.** Morale climbs back by 15 every turn
  today, and panic is only checked below 50. A fear that costs less than
  it gets back would never bite.
- **The dark is the whole battle for now.** The game knows night from day,
  not which tiles are lit. So a soldier afraid of the dark is afraid for
  all of a night battle. Flashlights and flares (graphics phase 3) may
  let us do better later.
- **Dropped work.** The earlier plan had rain triggers and a heavy-rain
  flag for the fear of water. Water does nothing on land, so both are
  gone.

**3c. Where the strength rules act.**
- STEADY: the accuracy rule, where wounds lower aim.
- BRAVE, COLD, EMPATH, LONER, SUPERSTITIOUS: the morale loss when a
  squadmate dies, and the panic check.
- PROTECTIVE and RECKLESS: reaction fire and the start of the soldier's
  turn.
- EMPATH also tells a late human worker from a real civilian (package 7).

**3d. Earned traits.** Opus.
- A list of rules. Each rule is a test on the soldier's history from 1a and
  the trait it grants (shell shock after too much, competence after
  surviving alone, others to be designed).
- Checked once per soldier after every mission, next to promotions.
- Each earned trait also leaves a mark on the portrait (3e).

**3e. Portraits in six stages, with event marks.** Medium. Fable for the
six looks and the overlays, Sonnet for the five places the
portrait is shown.
- **Six stages**, from new recruit to the thousand-yard stare. The stage
  comes from the mission count. How many missions each stage takes is set
  with the test bench, then checked by playing.
- **Event marks** on top of the stage: a scar for each time wounded, and a
  change for each earned trait.
- The game works the portrait's state out from the soldier's history each
  time. It is never stored, so it can never go out of step.
- **How it is drawn** (changed in the sixth session, 2026-11-13). The
  portrait is a picture of the soldier's 3D head, built from the soldier's
  slider values (package 10). The stages and event marks apply to the
  head: lighting, grime, darker eyes, scars. This replaces the earlier
  plan of two ways (the game's own drawn face, and overlays on a player's
  picture). Until the head builder (10b) exists, the game's own drawn face
  stays.
- Because the game builds the head, it knows where the eyes are. No crop
  step is needed.
- The state is plain numbers. Nothing about drawing goes into the rules.

**3f. Ribbons and citations.** Medium to large. Fable for the ribbon and
citation design, Opus for the counting and the rules, Sonnet for the file
screen and the book of citations, to Fable's layout. Needs 1a and 3d. Added in the sixth
session (2026-11-13). Pairs with graphics phase 4 for the ribbon designs.
- **Ribbons replace jobs.** There are no classes and no fixed path. A
  soldier grows by what they do, again and again.
- **Eleven ribbons:** ten to start, and a luck ribbon added in the ninth
  session. The ten are rifle, shotgun, knife, explosives, medic,
  scout (spotting enemies first), overwatch (reaction shots), Reach,
  survivor (living through heavy damage gives a defence bonus over time)
  and creatures. The luck ribbon grows from stray kills, hits made at very
  low odds, and living through shots that should have hit. Its name and
  what it gives are to settle. Ribbon names and designs are to settle.
- **Five levels each.** A higher level replaces the lower one on the
  ribbon board on the dress uniform.
- **Creature ribbons.** Kills for beasts and machines. Captures for people
  (Tender, Lash, Regent): taking them alive earns the ribbon. Saving
  citizens on missions also earns a ribbon. Reason: living freed Tenders
  make the finale easier, so the ribbons reward what the story wants.
- **Only uses that matter count.** Healing a real wound, hitting an enemy,
  a grenade that hurts something. Less is gained for repeating within one
  battle.
- **Level targets come from the test bench.** Reaching the top level
  should take the whole campaign, not the first three months. The numbers
  for each level are set with the test bench, then checked by playing.
- **Favoured role.** A pool soldier's favoured role (package 10) gives a
  head start on one ribbon. It changes no stats.
- **Pool soldiers start clean.** No ribbons carry over between campaigns.
  The pool page keeps the full service record (package 10).
- **Ribbon history** (seventh session). Every ribbon level remembers where
  and when it was earned: the mission, the date, and the deed that tipped
  it over. The ribbon board reads like the soldier's history. So the
  counting must record the mission, the date and the deed at each level-up.
- **Hidden abilities (citations).** A mix of ribbons unlocks a special
  ability the player can stumble into. Starting four, accepted by the
  owner as a good starting point. Exact effects are to settle with the
  test bench.
  - RANGER (rifle plus scout): sees further, and gets one overwatch shot
    each turn that costs no time units.
  - BREACHER (shotgun plus explosives): the first shot after going
    through a door or a blown wall costs no time units.
  - GUARDIAN (medic plus overwatch): a free reaction shot at anything that
    attacks a soldier they healed this battle.
  - GHOST (knife plus scout): enemies that have not spotted them cannot
    take reaction shots at them, and they leave no footprints (3g).
  - More hidden abilities are wanted. Which ones is to settle.
- **The book of citations.** A hidden ability stays hidden until one of
  the player's soldiers earns it. Then it appears in the book with how it
  was earned, and stays known in every later campaign.
- **To settle:** how ribbons and citations relate to the earned traits in
  3d.
- **Friendly-fire after-effects (planned, ninth session).** The shooter
  may gain a bad trait, and the victim may lose trust in the shooter.
  These read the incident record that package 0, slice 1 keeps. Which bad
  traits and how trust is lost are to settle.
- Self-test: ribbon counting reads the events and rolls no dice, so it
  should not shift the main dice. The citations' free shots touch time
  units and reaction fire, so check them when they are built.

**Stories players find themselves (seventh session, 2026-11-13).** Four
more items. Which package each belongs to is to settle. Nothing is built.
- **Bonds.** Soldiers bond by fighting side by side. A bonded partner
  nearby steadies morale; a partner hurt or killed hits it hard. Losing a
  partner rolls shaken or hardened. Leans on the morale rules and on the
  dice (package 0). All numbers, how a bond is shown, and which dice are to
  settle.
- **Moment of the battle.** The debrief picks one moment from the roll log
  and saves it to the service record. Leans on the package 0 roll log. How
  it is picked is to settle.
- **Memorial wall.** A base screen of every fallen soldier, kept in the
  finished record. Leans on the service record (package 10a) and on ribbon
  history (3f).
- **Deployments.** A pool soldier's past campaigns on their record, with a
  small nod for returning veterans (for example a mark on the dress
  uniform). No effect on stats. Leans on the service record and on package
  10a. The mark's design is to settle.
- **Shielding** (eighth session, later part, 2026-11-13). A soldier in the
  line of fire makes an enemy shot at the friend behind them miss, when the
  shot would have hit without them. The game notices and records it on both
  soldiers. It counts toward the GUARDIAN citation and toward the two
  soldiers' bond, and can be picked as the moment of the battle. Built with
  ribbons, not before. It leans on the stray-shot rule in package 0, slice 1.

**3g. Footprints.** Medium. Opus. Added in the sixth session
(2026-11-13). Pairs with the weather work and with the older idea under
"Ideas, Mechanics" (footprints in snow).
- Units leave footprints that show in bad weather: snow, rain and the
  like. A GHOST (3f) leaves none.
- To settle: who can read tracks (soldiers, aliens or both), how long they
  last, which weather and ground count.
- Not scheduled beyond this until those are settled.

**Self-test: at risk.** Dealing a strength and a fear to a recruit adds
dice rolls, and every later roll would shift. The fix is a separate set of
dice for these, seeded from the soldier's own number, so the main sequence
does not move. To settle: how this fits the three kinds of dice decided in
the fifth session (campaign, battle, visual).

### Package 4. Creature rules

Pairs with graphics phase 5 (creatures). The taker's fluid and the
Cinder's fires pair with phase 3. Large overall. Needs 1c for all.

Each creature is its own medium job for Opus. Where `LORE.md` leaves a
gap, Fable writes the rule sheet with the user first. Sonnet adds one new
self-test case per creature. Build them one at a time, in this order:

1. **Bloat.** When it is shot, or when it reaches a target, it bursts
   into a cloud that blocks sight (the Kite's too) and chokes. The player
   can burst it over the enemy. It reuses the smoke already in the game.
   Choking is new. `LORE.md` does not say what it does. Proposed, to
   settle: it stuns a little each turn, and ordinary smoke grenades do not
   choke.
2. **Kite.** It marks a soldier. Its daze shot does no damage but halves
   the soldier's time units and stops reaction fire for one turn. A Lash
   in range may shoot a marked soldier it cannot see, for as long as the
   Kite can see them. Killing the Kite, cover overhead, or smoke above
   ends the mark.
3. **Gorger.** It lies buried and unseen, feels footsteps, then erupts,
   seizes a soldier and drags the remains below.
   - **Signs.** A buried Gorger shows shifting ground and dust. As a
     soldier moves, the game checks whether they spot one within a few
     tiles. How many tiles is set with the test bench,
     then checked by playing.
   - **The handheld radar finds one reliably.** The radar is already an
     item and is sold in the market, but it does nothing in a battle
     today. It gets its first battle use: an action that shows buried
     units nearby.
   - Any dice rolled for spotting must stay off the self-test's path.
   - Its corpse becomes refugee food (2d, then package 5).
4. **Maul.** An animal with no weapon. It blasts magma (heat damage). It
   walks to corpses and dropped kit and eats them, and eats unconscious
   soldiers (needs 1a-2). Each meal makes it fuller. Its corpse gives Slag
   by fullness. It can never be captured for freeing. Its study topic
   stays. This part is larger than the others, because of the spike.
   - **The control spike.** A piece of Registrar hardware is driven into
     its molten core. The spike has its own small health.
   - **Going wild.** Destroy the spike and the Maul moves to the wild side
     (1c). It attacks whoever is nearest, the enemy included.
   - **How to hit the spike** is to settle with the user. The cheap rule:
     the game already knows which face of a unit a shot struck, so shots
     from behind damage the spike. The other way is a called shot (the
     player picks the spike as the target), which needs new controls.
5. **Cinder.** The Maul's young: a glowing cube-shaped rock that rolls.
   It is in the creature table but never appears in a battle today.
   - Cinders break off a full Maul or come out of its remains, in the
     middle of a battle.
   - **They are wild from the start.** Nothing controls them. They set
     fires and eat, and go for the enemy's dead as readily as ours.
   - **Growth.** Each meal makes a Cinder bigger and stronger, through a
     growth number. It stays a Cinder for the whole battle. It only
     becomes a full Maul between battles.
   - **Its table row is rewritten.** Today the row describes a slow,
     heavily armoured thing. It becomes small and fast, as the lore says.
   - What a Cinder that survives a battle does afterwards is to settle.
   - Art rule for `ART_BIBLE.md` when the Cinder is designed: its cube
     shape has no link to the Registrar's cube sign.
   - Needs the Maul first.
6. **Brooder, the taker.** A hovering machine that spits glowing fluid.
   - Adds pools: a tile can hold fluid, and the fluid hurts whoever stands
     in it.
   - Puts civilians on the battlefield, using the civilian side from 1c.
     A civilian melts after one spit and then runs at soldiers. A soldier
     takes several spits, because armour soaks it up.
   - Adds a neutralising spray (a mid-game item with its own research
     topic) and the cube core as loot.
   - Switches on the fear of the Glow (3b).
7. **Lathe, the builder.** It repairs machines, makes cover and, after
   several undisturbed turns beside a pool, builds one new taker. Heavily
   armoured while travelling, exposed while working. The sphere core is
   loot. Needs the Brooder first.

Whether a battle can end while wild units are still alive is to settle
with the user before the Maul part starts.

Self-test: safe as long as the Tender's path is not touched. The surface
research table has a fixed length, so adding the spray topic means making
it one row longer.

### Package 5. Mess Hall, Scrub Room, Refugee Quarters, infection

Pairs with graphics phase 7 (base modules and base people). Medium, plus
one large new part (5b). Needs real loot (2d) for Gorger meat, and 1a so
that resting really happens. Refugee Quarters is only useful once package
6 is in.

**5a. The three modules, hunger and sabotage.** Medium. Opus for the
rules, Sonnet for the rows and screens to Fable's layout, Fable for the icons.
- Adds three modules. The module list grows from 17 to 20.
- Each module type gets a capacity: meals for the Mess Hall, scrubbing for
  the Scrub Room, beds for the Refugee Quarters.
- Food is used and scrubbing is done every day. The mess costs money to
  run. Scrubbing costs power and resources. The monthly report shows a
  line for each.
- **Hunger.** The base keeps a stock of meals and a stock of Gorger meat.
  Each day everyone eats: soldiers from the meals, refugees from the
  Gorger meat. How much each eats is one of the figures set with the test
  bench, then checked by playing.
- **Alien food topic.** `LORE.md` says an "alien food" research topic
  opens after the research that lets us free captives. It is what lets
  the mess feed refugees. The surface research table has a fixed length,
  so it becomes one row longer.
  - Unfed soldiers start their next battle with lower morale, and recover
    from wounds more slowly.
  - Unfed refugees weaken (their strength and skill numbers drop), then
    leave. The log says so.
  - Nobody starves to death on screen.
  - Soldiers are not tied to one base today. So "which mess feeds whom"
    needs a simple rule, to settle. The simplest: all meals count as one
    stock.
- **Infection.** The base keeps one infection number. Building and using
  alien-derived equipment raises it. Refugees do not: once freed they are
  clean. The Scrub Room pushes it back down, never to zero.
- **Low risk brings sabotage.** One of three things happens:
  - a module goes dark for some days;
  - some research progress is lost;
  - a craft is grounded for some days.
- **High risk brings a battle inside the base**, with no warning. That is
  part 5b.
- The Barracks text says it includes a mess today. It gets reworded.
- The program cannot be compiled (turned from code into a runnable game)
  until all three modules have a colour and an icon. That is a useful
  safety net, not a problem.
- All hunger and infection figures are named values, set with the test bench, then checked by
  playing.
- Self-test: safe.

**5b. The base-interior map.** Large. Opus for the map and rules, Fable
for the look. New. Needs 5a's infection number.
- A second map builder. It turns the base's grid of modules (6 by 6) into
  rooms and corridors that a battle can be fought in. Today's map builder
  only makes outdoor maps.
- Enemies start inside: they step out of the workshop and the stores.
  In the game those modules are called MACHINE SHOP and SUPPLY BAY.
- The battle starts with no mission site on the globe and no transport.
  Today a battle can only be started from a site, and the landing craft
  is always drawn. Both must become optional.
- The game already has an unused "base defence" site kind. This is what
  finally uses it.
- "The open door" (package 7) is the first battle fought on this map.
  Ordinary base defence, still on the ideas list, can use it later.
- Self-test: safe. The fixed battle is outdoors.

### Package 6. Freed aliens ("refugees") as soldiers

Pairs with graphics phases 4 and 5. Large. Needs packages 1, 3 and 5.

- Adds a species to each soldier: human, Regent, Tender or Lash. A species
  table gives each its starting numbers, the weapons it may use and the
  armour it may wear.
- Adds a research topic that removes the control hardware from a
  captive. `LORE.md` says freeing needs it. Only Regents, Tenders and
  Lashes can be freed.
- Adds a "free" action on the holding screen, once that topic is done. It
  moves a captive into the squad list if the Refugee Quarters has room.
- A battle must be able to build one of our units from a creature, and
  draw it as that creature even though it is on our side. Today the side
  alone decides the drawing.
- **Regent.** Uses Reach. Cannot use human weapons. The Reach controls
  already work for any of our units. But every soldier must hold a weapon
  today, so "no weapon" needs to become a proper value.
- **Tender.** Small and frail. A scout that senses Reach and enemy
  machines. The sense is a new rule beside line of sight: it marks Reach
  users and machines (the taker and the builder). How far it reaches, and
  whether it works through walls, is to settle.
- **Weapons for the Tender and the Lash.** `LORE.md` rules human weapons
  out for the Regent only. What a freed Tender or Lash may carry is to
  settle.
- **Lash.** The best marksman the player can have. It must ask each
  target's pardon: its first shot at any new target costs extra time. The
  unit keeps a list of targets it has already asked.
- **Harnesses.** None of the three can wear human armour. The workshop
  makes each kind its own harness: three new suits (the armour list grows
  from 6 to 9) and three new workshop blueprints (14 to 17). The checks
  that decide what a soldier may wear need to know the species.
- **Promotions and the growth of a soldier's numbers** after a battle
  assume humans today. Each
  species needs its own rule, or none.
- **Study or free.** Studying a live captive uses the captive up today. So
  a captive can be studied or freed, not both. Whether that stays is to
  settle.
- The Maul can never be freed.
- Self-test: safe. Its soldiers are human.
- Model: Opus, with Fable for the species sheets.

### Package 7. Signposts and the human worker

Pairs with graphics phase 8 (environment, civilians). Needs all of package
1: records (1a), ways to lose a soldier (1a-2) and the sides (1c). Needs
civilians from package 4. "The open door" needs 5b.

**7a. The signpost framework.** Medium to large. Opus.
- A table of signposts. There are five. Each has a trigger and a
  **shape**. The triggers and the stories are from `LORE.md`. The shapes
  are this plan's reading of them, to confirm with the user: `LORE.md`
  calls all five "missions", but two of them read more like a story
  screen and a moment inside another battle.

| Signpost | Shape (proposed) | Set off by | Needs |
|---|---|---|---|
| First words | A story screen. No battle | The first live capture | The game's event screen, which exists but is never used today |
| The split vote | A scripted moment inside the next ordinary battle | First studying a Regent | The hook from 1c; a Regent in that battle |
| The same face | Its own mission | The first soldier the Registrar has read | 1a-2 |
| Roll call | Its own mission, with a rescue goal | A late batch of human workers appears | Civilians, the worker (7b), a goal |
| The open door | A battle inside the player's base | Infection risk first runs high | Package 5a and 5b |

- Triggers are checked every day and after every battle.
- **First words.** In the holding block the captive starts speaking the
  Registrar's first gibberish, built from words the player's own staff
  have said. The staff say nothing in the game today. Proposed, to
  settle: the words are lifted from what the game already has, the
  soldiers' names and the log.
- **The split vote.** A Regent hesitates, lets a soldier live, and is
  recycled in front of the squad. The campaign keeps a short list of
  moments waiting for the next battle.
- **The same face** builds its enemies from the highest-ranked soldier in
  the lost-soldiers list.
- **Roll call.** A rescue mission where the "civilians" call the soldiers
  by name, in the voices of people the player lost.
- **The open door** teaches scrubbing.
- **Goals.** "Roll call" cannot be won by "no enemy left alive". Nor can
  the finale. The goal from 1a gets its first other values here: rescue,
  and scripted. Package 8 reuses them.
- A new site kind, Signpost, is added for the two that are their own
  missions.
- Each further signpost is then a small job. Fable writes the scripts.

**7b. The human worker.** Medium. Opus for the rules, Fable for the lines
and sound. Needs 1a-2.
- **First appearance:** after the Registrar has read one of the player's
  soldiers.
- A new unit, added at the end of the creature table. It is built from a
  lost soldier's numbers and wears their face. It needs a human-shaped
  drawing while on the enemy's side.
- The campaign keeps a batch number. It rises over time. How many batches
  there are is set with the test bench, then checked by playing.
- **Early batches carry no weapon.** They run at the squad, cling to a
  soldier and call out in broken recorded lines. Clinging is a new "held"
  effect in the status block from 1c. This closes the old question of
  whether the worker fights.
- **Late batches hide among civilians.** They stand on the civilian side
  until they show themselves, then use human weapons. An EMPATH can tell
  them apart.
- **Voices.** The game has made-up sound effects and no speech. So for now
  the "voice" is lines of text plus calls the game generates. They start
  digital and senseless and come closer to speech with each batch. The
  idea of copies using a lost soldier's own lines waits until soldiers
  have lines.
- **Pool soldiers.** A lost soldier may be one the player made in the
  soldier pool, possibly made from a real person's photo. Whether a copy may
  wear that soldier's head is to settle (sixth session: the head is built
  from slider values, not a stored picture). Until the user answers, a copy uses the drawn face
  only.
- Self-test: safe.

### Package 8. The Moon finale

Pairs with graphics phase 9 (craft and the final sites). Large. Needs
package 6 (Tenders), package 7 (goals and scripts) and graphics phase 3
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
  cannot be killed. The mission uses a scripted goal from package 7.
- **No pass mark.** Every living freed Tender the player brings to the
  Moon weakens the Seed's defences. With very few the mission is nearly
  impossible. With many it is fair. There is no fixed number.
  - Today the final mission always has the same fixed enemy force. That is
    replaced by a strength worked out from the Tender count when the
    mission launches.
  - The curve (how much each Tender helps) is designed by Fable and set with
    the test bench, then checked by playing.
  - To settle: does a Tender count if it is alive anywhere, or only if it
    is aboard the transport? The second runs into the squad size limit.
- **The vote.** The Tenders vote and merge into a maker.
- **The last word**, "there are many", goes on the end screen.
- **Two fixes.** Today the final site also appears after 8 wins, with no
  research. That shortcut goes. And the win text says the Seed "is dead",
  which the lore says cannot be true. It gets reworded.
- Self-test: safe if gravity is normal unless a mission says otherwise.
- Model: Fable for the ending, the maps and the curve, Opus for the rules.

### Package 9. Multiplayer: join or be wiped out

Version 3.0. A design note only for now. No code and no graphics pairing.

- A player who loses may join the Registrar's side and keep playing, or be
  wiped out. What a joined player controls is still open.
- It needs the plain records from packages 1, 3 and 6.
- It needs the simulation split from the drawing. Both live in one file
  today. Package 0 (the test bench) does that split first.
- Soldier pool data (names, slider values) is private. What crosses between games must be
  decided before any soldier does.
- Size: large. Model: Opus for the split, Fable for the design.

### Package 10. The soldier pool (the user's idea, 2026-11-13)

A page reached from the game's start screen. The player creates soldiers
there once. Soldiers from the pool are pulled at random into every
campaign started afterwards. So the player and their friends can turn up
in any game without being made again.

Pairs with graphics phase 4 (soldiers and portraits). Three parts.
**Changed in the sixth session (2026-11-13):** soldiers are made in a
character designer driven by hand, and a photo is an optional later step.
This replaces the earlier plan of one picture per soldier with the stages
laid over it. The old 10b (picture import and crop) is gone. The new 10b
and 10c are below.

**10a. Storage, screen, typing and arrival.** Medium. Opus for storage and
arrival, Sonnet for the screen to Fable's layout. Needs nothing, so it can be built first.
- **What a pool soldier is.** A name, a look, and a "start with me" tick.
  The designer's fields (10b) are added to it later. Until 10b is built, a
  pool soldier uses the game's drawn face and the existing look form.
- **Dealt fresh.** The strength and the fear are random in every campaign
  and hidden like anyone else's. Their other numbers (aim, health and so
  on) are rolled as for any recruit. That last part is this plan's
  reading; `LORE.md` only names the strength and the fear.
- **Where it is kept.** One small file listing the pool. It lives in the
  player's own data folder on their machine (on Linux, under
  `~/.local/share/defending-earth/`). Never in the project folder.
- **The screen.** A new SOLDIER POOL button on the start screen opens the
  page: a list of pool soldiers, with add, edit and remove. The look form
  from the existing soldier editor is reused, pointed at a pool entry,
  until 10b replaces it.
- **Typing a name.** The game has no text typing anywhere today. The
  soldier editor only has RANDOMIZE NAME. A small text box is new work.
  Once it exists, the ordinary soldier editor can use it too.
- **Arrival.**
  - Ticked soldiers take places in the starting squad of 8.
  - The others turn up at random among the recruits offered later, when
    the player hires or buys.
  - Proposed, to confirm: each pool soldier appears once per campaign. A
    soldier in a campaign remembers which pool entry it came from.
  - To settle with the user: how often a pool soldier is offered, and
    what happens with more than 8 ticks.
- **Death and the service record** (sixth session). A pool soldier who
  dies in a campaign stays in the pool for future campaigns. Each campaign
  starts them clean: no ribbons carry over. The pool page keeps their full
  service record: every ribbon and citation ever earned, campaigns served,
  times fallen. So the pool file must also hold this record, and the
  campaign must report it back when a campaign ends or a soldier falls.
  The record needs ribbons (3f), so it fills in after 3f is built.
- **Privacy.** Names and slider values stay on the player's machine.
  - The pool is switched off in all four debug runs (`--demo`,
    `--screenshot`, `--self-test`, `--ai-test`). Otherwise a private face
    could land in the screenshots folder, which is published, and the
    self-test's result would change from machine to machine.
  - Nothing from the pool is ever written into the project folder.
- Self-test: safe, because the pool is off there.

**10b. The hand-driven character designer.** Medium. Fable for the head
builder and the slider list, Sonnet for the screen to Fable's layout. Needs 10a, 1a
and 3e. Built before 10c.
- **The face:** a detailed face with many sliders. The slider list is to
  settle.
- **The body:** simple. Build, height within a small range, skin, hair
  style and colour, and a voice.
- **Also set by the player:** name, nickname, a short background (a line
  or two, flavour only), and a favoured role. The favoured role gives a
  head start on one ribbon (3f). It changes no stats. Traits stay random
  and hidden each campaign.
- **Only slider values are saved.** The pool file stores the values, never
  a picture.
- **The head is built from the slider values.** The soldier's 3D head is
  built once, when the soldier loads, so the engine needs nothing per
  frame. The close-up portrait is a picture of that head.
- **The six stages and event marks** from 3e apply to the head and its
  portrait: lighting, grime, darker eyes, and a scar for each wound.
  Wear applies to the head too.
- **A soldier can be made with no photo at all.** The designer works fully
  by hand.
- **Research note** (facts from a web check on 2026-11-13, to re-check
  before use):
  - MPFB2 is an open slider-driven human maker for Blender. Its add-on
    code is GPL-3.0 and its assets are CC0. It is the best open base.
  - Traps: most well-known photo-to-3D-head research code depends on
    non-commercial face data (Basel Face Model, Surrey model, older
    FLAME), and several face model weight files are non-commercial.
- Self-test: safe.

**10c. The photo step.** To measure. Opus. Needs 10b. Built second, after
the hand-driven sliders.
- **Optional.** A photo sets the face sliders for the player, in fine
  detail. The player can then change them by hand.
- **Read once, thrown away.** The photo is read once. Only slider values
  are saved. Nothing private is stored.
- **The mapping is ours to write.** No open photo-to-slider tool was
  found. The plan: find face landmarks in the photo, then set sliders to
  match.
- **Research note** (facts from a web check on 2026-11-13, to re-check
  before use):
  - The MediaPipe face landmarker has Apache-2.0 code. The licence of its
    model file is NOT yet checked.
  - The FLAME 2023 Open head model is CC BY 4.0. Every other FLAME version
    is academic only.
  - Most well-known photo-to-3D-head research code depends on
    non-commercial face data. Check any code or weights before use.
- **Getting a photo in.** How it gets into the game (drag onto the window,
  an import folder) is not settled; the older notes on it are in "Still to
  decide". Whether to read JPG, which phone photos are, is part of the same
  question: the game reads PNG and TGA today.
- Self-test: safe.

### Still to decide with the user

The third lore session closed most of the earlier list. Closed, for the
record:
- All eight starter traits are strengths. Flaws are fears.
- The fears: the dark, fire, tight spaces, the Glow, water and sharks.
  Water and sharks do nothing on land.
- Portraits have six stages, plus event marks.
- The Maul is controlled by a spike in its core.
- A Cinder never becomes a Maul during a battle.
- Buried Gorgers show signs; the radar finds them.
- Dead Gorgers stock the mess. Hunger weakens; it does not kill.
- Low infection means sabotage; high means a battle inside the base.
- What each freed species can do, and that each needs its own harness.
- The human worker appears after a soldier has been read. Early batches
  are unarmed; late ones hide and shoot.
- Five signposts, for now.
- The ending has no pass mark.

Still open. These go to the user in question rounds before the package
that needs them starts.

*Numbers, set with the test bench and checked by playing once the system exists:*
1. What each strength and each fear is worth (package 3).
2. How many missions each of the six portrait stages takes (package 3).
3. Slag from a full and an empty Maul; how fast it eats; the spike's
   health (packages 2 and 4).
4. How far away a careful soldier spots a buried Gorger (package 4).
5. Hunger figures, and how much Gorger meat a refugee needs (package 5).
6. Infection and scrubbing figures (package 5).
7. Numbers for each refugee species; the Lash's extra time for a pardon
   (package 6).
8. How many batches of human workers there are (package 7).
9. How much each Tender weakens the Seed (package 8).

*Questions raised by the third-session answers:*
10. How the player aims at the Maul's spike: by shooting from behind, or
    by a called shot (package 4).
11. Whether a battle can end while wild units are still alive (packages
    1c and 4).
12. What a Cinder that survives a battle does between battles (package
    4).
13. Whether the fear of the Glow reacts to a soldier's own CORED weapon,
    which is filled with Glow (package 3).
14. Which mess feeds which soldiers, since soldiers are not tied to a
    base (package 5).
15. Whether a captive can be both studied and freed (package 6).
16. Whether a Tender counts for the ending if it is alive at base, or
    only if it is aboard the transport (package 8).
17. Whether the game should read JPG pictures, or PNG only (package 10c; the photo is now optional and read once, see the sixth session).
18. Whether a human worker copied from a pool soldier may wear that
    person's face. Since the sixth session the face is slider values, not a
    picture (packages 7 and 10).

19. Whether water and shark fears are left out of the deal in a land
    campaign, or dealt and left asleep (package 3).
20. What a fear does when it bites. The plan proposes a morale hit every
    turn (package 3).
21. What the Bloat's choking does (package 4).
22. Which weapons a freed Tender or Lash may carry, and whether the
    Tender's sense works through walls (package 6).
23. The shape of each signpost: "First words" as a story screen and "The
    split vote" as a moment in an ordinary battle, or full missions. And
    where the captive's first words come from (package 7).
24. Pool soldiers: once per campaign or more, how often one is offered,
    and what happens with more than 8 "start with me" ticks (package 10).

*Still open from before:*
25. What a joined multiplayer player controls, and what happens to pool
    soldiers' heads (slider values) when soldiers cross between games
    (package 9).

*Added 2026-11-13 with the test bench and the scale sheet:*
26. How many metres across a battlefield square is. It must be settled with
    the user before any modelling starts (the scale sheet, graphics phase 4).
27. The test bench's report file format and command-line options (package
    0). Its real size, once the tangle of rules and drawing has been
    measured.

*Added 2026-11-13 with the fourth-session rule fixes (see `LORE.md`):*
28. How the alien side decides which aliens go on overwatch.
29. How far humans see in the dark.
30. Whether weather also cuts sight.

*Added 2026-11-13 with the fifth session (dice, shot odds, battle rules and
saves; see `LORE.md`):*
31. What the cinematic replay view is (package 0 plans for replays but does
    not build them).
32. The animation list itself: what to trim or add (see "Animation list" in
    the design and animation catalogue).
33. How many save slots there are.
34. ~~Which things in the way lower the odds (low wall, smoke, a tree,
    another unit) and by how much (package 0).~~ Settled in the eighth
    session (see `LORE.md`): smoke and crystal minus 10, rock, stone block
    and a friendly unit minus 25. Enemy and neutral units on the line are
    number 47.

*Added 2026-11-13 with the sixth session (the character designer and ribbons;
see `LORE.md`):*
35. Ribbon names and designs: eleven ribbons since the ninth session (ten
    plus the luck ribbon), five levels each (packages 3f and graphics
    phase 4).
36. More hidden abilities, and the exact effects of RANGER, BREACHER,
    GUARDIAN and GHOST (package 3f, set with the test bench).
37. Footprints: who reads them (soldiers, aliens or both), how long they
    last, and which weather and ground count (package 3g).
38. How ribbons and citations relate to the earned traits in 3d (package 3f).
39. The face slider list (package 10b).
40. The licence of the MediaPipe face landmarker's model file (package
    10c). Not yet checked.

*Added 2026-11-13 with the seventh session (stories players find themselves;
see `LORE.md`):*
41. Bonds: the numbers (how fast a bond forms, how much morale moves, how
    near is "nearby"), and how a bond is shown to the player.
42. Losing a bonded partner: which dice make the shaken-or-hardened roll
    (campaign or battle), the odds, how many missions it lasts, and the
    bonus.
43. Moment of the battle: how the debrief picks the moment.
44. The mark for returning veterans' deployments (for example on the dress
    uniform). Also confirm "Deployments" as the name for a pool soldier's
    past campaigns.
45. Which package each of bonds, moment of the battle, memorial wall and
    deployments belongs to.

*Added 2026-11-13 with the eighth session (shot range, cover and the campaign
seed; see `LORE.md`):*
46. Range for the Grenade, Melee and Heal kinds. For now they have no range
    effect.
47. Enemy or neutral units standing on the line of a shot. For now they have
    no effect.
(The starting numbers for range and cover are tuned with the test bench.)

*Added 2026-11-13 with the later part of the eighth session (shielding and
friendly fire; see `LORE.md`):*
48. ~~How a stray shot's path is worked out, and what it can hit.~~
    Settled in the ninth session (R2, R3, R4, R5; see `LORE.md`). Scenery
    on the path is number 54.
49. ~~The odds of a stray shot hitting something.~~ Settled in the ninth
    session (R4: 50 in 100 standing, 25 in 100 kneeling; R5: a cover hit
    when the roll lands in the cover's points). Scenery odds are number 54.
50. Which bad traits a shooter may gain after friendly fire. Still open
    (R11; planned in package 3f).
51. ~~The size of the squad's morale hit after friendly fire.~~ Settled in
    the ninth session (R10: 25 percent, 50 percent or the full death loss).
52. ~~Whether a friend in the line still lowers the odds by 25 once stray
    shots exist.~~ Settled in the ninth session (R5): it does, and the
    friend is hit when the roll lands in their 25 points.
53. Whether friendly fire is also written on service records. The game now
    keeps an incident record (R11); whether it shows on service records is
    still open.

*Added 2026-11-13 with the ninth session (stray shots, friendly fire and
damage; see `LORE.md`). Numbers 54 to 58 are the assistant's readings, to
confirm with the owner. They are built as written and marked "to confirm"
in the code:*
54. ~~T1: a stray passing scenery that does not fully block strikes a rock or
    stone block 25 in 100 and a crystal 10 in 100.~~ Settled in the ninth
    session (confirmed 2026-11-13).
55. ~~T2: a stray never hits the unit it was aimed at and never the shooter.~~
    Settled in the ninth session (confirmed 2026-11-13).
56. T3: starting hit points for trees (60) and walls (80), if the map data
    has none.
57. ~~T4: friendly-fire morale reaches the same units a death's morale loss
    reaches today, with no sight test.~~ Settled in the ninth session
    (confirmed 2026-11-13, for now; a sight test may come later).
58. ~~T5: grenade, melee and heal kinds produce no stray.~~ Settled in the
    ninth session (confirmed 2026-11-13).

*To settle from the ninth session:*
59. How trust is lost after friendly fire: how the bond is damaged, and when
    it becomes a grudge (package 3f).
60. The luck ribbon's name (package 3f, NAMES.md).
61. What the luck ribbon gives the soldier (package 3f).

## Ideas (unsorted; we sort these into versions together)

Moved out of this list on 2026-11-13, because they are now decided and
scheduled in the game systems track:
- Infection tax, sabotage and scrubbing: package 5. The attack that starts
  inside the base: package 5b.
- The Mess Hall, feeding and hunger: package 5.
- Freeing captives to serve as soldiers: package 6.
- Signpost missions, now five of them: package 7.
- Melting panickers and the taker's pools: package 4.
- Your own face on a soldier, made once and used in every campaign: the
  soldier pool and its character designer, package 10.

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

- Base defence: ordinary attacks on a base, including raids to recover a
  captive we are holding. Not scheduled yet. The inside-the-base map from
  package 5b is what it would be fought on.

- Ordering researched stims and healing medicines at the Mess Hall. Not
  scheduled yet. (The building, the feeding and hunger are in package 5.)

- Civilians on terror missions (the user's design, 2026-11-13): they run
  TOWARD the soldiers to be saved, and hide beside or behind them, which
  makes stealth hard. They only react to soldiers they can see or gunfire
  they can hear; otherwise they hide where they are. A soldier can spend a time unit to order one to "get
  to the extraction vehicle"; the civilian then runs to the craft and stays
  there. A civilian who reaches the craft, or is alive when the mission
  ends, counts as saved. Aliens can follow them to the craft to kill them.
  The civilian side itself is scheduled (package 1c, then package 4), and
  "Roll call" (package 7) needs a rescue goal. The rest of this design
  (hiding, being ordered to the craft, counting as saved) is not scheduled
  yet.
- Footprints in snow: soldiers, civilians and aliens leave tracks. While it
  is still snowing the tracks fade over time. Aliens can see and follow
  tracks, how well depending on the species' intelligence; the player can
  read alien tracks the same way.
  Partly decided in the sixth session (2026-11-13): units leave footprints
  in bad weather, and a GHOST leaves none. Who can read tracks, how long
  they last and which weather counts are still to settle. See package 3g.

### Presentation
- Your own face on the soldier's body in battle (the user's idea,
  2026-11-13). The close-up portrait part of this idea is now decided and
  scheduled as the soldier pool (package 10). What is left as an idea:
  fitting the photograph onto the 3D head seen on the battlefield (a face
  texture on our head model is the cheap route).
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
